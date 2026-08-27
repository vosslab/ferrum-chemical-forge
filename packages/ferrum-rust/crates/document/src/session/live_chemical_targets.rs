//! Durable target lowering for live chemical property and selection operations.

use crate::{DocumentObjectIdV1, PersistentId, SessionOperationError, TypedClass};

use super::DocumentSession;

impl DocumentSession {
    /// Lower durable molecule-owned members after validating exact current containment.
    pub fn lower_live_chemical_members_v1(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
        object_ids: &[DocumentObjectIdV1],
        expected_class: TypedClass,
    ) -> Result<Vec<PersistentId>, SessionOperationError> {
        let document = self.current_document_v1();
        let molecule = document
            .resolve_document_object_id(molecule_object_id)
            .map_err(|_| {
                SessionOperationError::InvalidLiveChemicalTarget(
                    molecule_object_id.as_str().to_owned(),
                )
            })?
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(molecule_object_id.as_str().to_owned())
            })?;
        if molecule.class() != TypedClass::Molecule || molecule.path().components().len() != 1 {
            return Err(SessionOperationError::InvalidLiveChemicalTarget(
                molecule_object_id.as_str().to_owned(),
            ));
        }
        let molecule_root = molecule.path().components().first();
        let mut lowered = Vec::with_capacity(object_ids.len());
        for object_id in object_ids {
            let record = document
                .resolve_document_object_id(object_id)
                .map_err(|_| {
                    SessionOperationError::InvalidLiveChemicalTarget(object_id.as_str().to_owned())
                })?
                .ok_or_else(|| {
                    SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned())
                })?;
            if record.class() != expected_class
                || record.path().components().len() != 2
                || record.path().components().first() != molecule_root
            {
                return Err(SessionOperationError::InvalidLiveChemicalTarget(
                    object_id.as_str().to_owned(),
                ));
            }
            lowered.push(source_id(record, object_id)?);
        }
        Ok(lowered)
    }

    /// Lower one fenced molecule-owned durable member to its private source address.
    pub fn lower_live_chemical_member_address_v1(
        &self,
        molecule_object_id: &DocumentObjectIdV1,
        object_id: &DocumentObjectIdV1,
        expected_class: TypedClass,
    ) -> Result<(PersistentId, PersistentId), SessionOperationError> {
        let object_id = self
            .lower_live_chemical_members_v1(
                molecule_object_id,
                std::slice::from_ref(object_id),
                expected_class,
            )?
            .into_iter()
            .next()
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(molecule_object_id.as_str().to_owned())
            })?;
        let molecule = self
            .current_document_v1()
            .resolve_document_object_id(molecule_object_id)
            .map_err(|_| {
                SessionOperationError::InvalidLiveChemicalTarget(
                    molecule_object_id.as_str().to_owned(),
                )
            })?
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(molecule_object_id.as_str().to_owned())
            })?;
        Ok((source_id(molecule, molecule_object_id)?, object_id))
    }

    /// Lower one durable top-level presentation root with its required typed form.
    pub fn lower_live_chemical_presentation_target_v1(
        &self,
        object_id: &DocumentObjectIdV1,
        target: LiveChemicalPresentationTargetV1,
    ) -> Result<String, SessionOperationError> {
        let record = self
            .current_document_v1()
            .resolve_document_object_id(object_id)
            .map_err(|_| {
                SessionOperationError::InvalidLiveChemicalTarget(object_id.as_str().to_owned())
            })?
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned())
            })?;
        if record.path().components().len() != 1
            || !target.matches(record.class(), record.attribute("style"))
        {
            return Err(SessionOperationError::InvalidLiveChemicalTarget(
                object_id.as_str().to_owned(),
            ));
        }
        Ok(source_id(record, object_id)?.as_str().to_owned())
    }

    /// Validate one complete ordered durable bracket pair in the current document.
    ///
    /// The public bracket command carries the durable pair itself.  The session
    /// only authenticates that those two objects are the current left and right
    /// members of one bracket pair; it deliberately does not lower a private
    /// source identifier back across that boundary.
    pub fn validate_live_bracket_pair_target_v1(
        &self,
        member_object_ids: &[DocumentObjectIdV1; 2],
    ) -> Result<(), SessionOperationError> {
        let document = self.current_document_v1();
        let [left_object_id, right_object_id] = member_object_ids;
        let left = document
            .resolve_document_object_id(left_object_id)
            .map_err(|_| {
                SessionOperationError::InvalidLiveChemicalTarget(left_object_id.as_str().to_owned())
            })?
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(left_object_id.as_str().to_owned())
            })?;
        let right = document
            .resolve_document_object_id(right_object_id)
            .map_err(|_| {
                SessionOperationError::InvalidLiveChemicalTarget(
                    right_object_id.as_str().to_owned(),
                )
            })?
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(right_object_id.as_str().to_owned())
            })?;
        let invalid =
            || SessionOperationError::InvalidLiveChemicalTarget(left_object_id.as_str().to_owned());
        if left.path().components().len() != 1
            || right.path().components().len() != 1
            || left.class() != TypedClass::Polyline
            || right.class() != TypedClass::Polyline
            || left.attribute("style") == Some("wavy")
            || right.attribute("style") == Some("wavy")
            || left.attribute("bracket_side") != Some("left")
            || right.attribute("bracket_side") != Some("right")
        {
            return Err(invalid());
        }
        let pair_membership = left.attribute("bracket_pair").ok_or_else(invalid)?;
        if left_object_id == right_object_id
            || right.attribute("bracket_pair") != Some(pair_membership)
        {
            return Err(invalid());
        }
        Ok(())
    }
}

/// The exact durable root family accepted by one chemical property adapter.
#[derive(Clone, Copy)]
pub enum LiveChemicalPresentationTargetV1 {
    Geometric,
    Wavy,
}

impl LiveChemicalPresentationTargetV1 {
    fn matches(self, class: TypedClass, style: Option<&str>) -> bool {
        match self {
            Self::Geometric => {
                matches!(
                    class,
                    TypedClass::Rectangle
                        | TypedClass::Square
                        | TypedClass::Oval
                        | TypedClass::Circle
                        | TypedClass::Polygon
                        | TypedClass::Polyline
                ) && style != Some("wavy")
            }
            Self::Wavy => class == TypedClass::Polyline && style == Some("wavy"),
        }
    }
}

fn source_id(
    record: &crate::TypedRecord,
    object_id: &DocumentObjectIdV1,
) -> Result<PersistentId, SessionOperationError> {
    let source = record.attribute("id").ok_or_else(|| {
        SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned())
    })?;
    PersistentId::new(source.to_owned())
        .map_err(|_| SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m-a\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"m-b\"><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom></molecule><rect id=\"shape\" x1=\"0\" y1=\"0\" x2=\"10\" y2=\"10\"/><polyline id=\"wave\" style=\"wavy\"><point x=\"0\" y=\"0\"/><point x=\"10\" y=\"0\"/></polyline></cdml>";

    fn object(session: &DocumentSession, source: &str) -> DocumentObjectIdV1 {
        session
            .current_document_v1()
            .document_object_id_for_source_id_v1(
                &PersistentId::new(source).expect("test source identifier"),
            )
            .expect("typed ingress persists the test record identity")
            .expect("typed ingress resolves the persisted document object identity")
    }

    #[test]
    fn durable_chemical_members_require_the_current_owner_and_kind() {
        let session = DocumentSession::load(SOURCE).expect("document");
        let before = session.snapshot().expect("before");
        let members = session
            .lower_live_chemical_members_v1(
                &object(&session, "m-a"),
                &[object(&session, "a")],
                TypedClass::Atom,
            )
            .expect("current owned atom lowers");
        assert_eq!(members[0].as_str(), "a");
        assert!(
            session
                .lower_live_chemical_members_v1(
                    &object(&session, "m-a"),
                    &[object(&session, "b")],
                    TypedClass::Atom,
                )
                .is_err()
        );
        assert_eq!(session.snapshot().expect("after"), before);
    }

    #[test]
    fn durable_chemical_member_address_preserves_the_validated_owner_pair() {
        let session = DocumentSession::load(SOURCE).expect("document");
        let owner = object(&session, "m-a");
        let atom = object(&session, "a");
        assert_eq!(
            session
                .lower_live_chemical_member_address_v1(&owner, &atom, TypedClass::Atom)
                .expect("current durable address lowers"),
            (
                PersistentId::new("m-a").expect("owner ID"),
                PersistentId::new("a").expect("atom ID")
            ),
        );
        assert!(
            session
                .lower_live_chemical_member_address_v1(
                    &owner,
                    &object(&session, "b"),
                    TypedClass::Atom,
                )
                .is_err()
        );
    }

    #[test]
    fn durable_presentation_lowering_accepts_only_its_closed_kind() {
        let session = DocumentSession::load(SOURCE).expect("document");
        assert_eq!(
            session
                .lower_live_chemical_presentation_target_v1(
                    &object(&session, "shape"),
                    LiveChemicalPresentationTargetV1::Geometric,
                )
                .expect("geometric target lowers"),
            "shape"
        );
        assert!(
            session
                .lower_live_chemical_presentation_target_v1(
                    &object(&session, "shape"),
                    LiveChemicalPresentationTargetV1::Wavy,
                )
                .is_err()
        );
    }

    #[test]
    fn durable_bracket_pair_validation_requires_both_current_ordered_members() {
        let source = "<cdml xmlns=\"urn:ferrum:cdml\"><polyline id=\"left\" bracket_pair=\"left\" bracket_side=\"left\"><point x=\"0\" y=\"0\"/><point x=\"0\" y=\"1\"/><point x=\"1\" y=\"1\"/><point x=\"1\" y=\"0\"/></polyline><polyline id=\"right\" bracket_pair=\"left\" bracket_side=\"right\"><point x=\"2\" y=\"0\"/><point x=\"2\" y=\"1\"/><point x=\"3\" y=\"1\"/><point x=\"3\" y=\"0\"/></polyline></cdml>";
        let session = DocumentSession::load(source).expect("document");
        let before = session.snapshot().expect("before");
        let left = object(&session, "left");
        let right = object(&session, "right");
        session
            .validate_live_bracket_pair_target_v1(&[left.clone(), right.clone()])
            .expect("complete durable pair validates");
        assert!(
            session
                .validate_live_bracket_pair_target_v1(&[right, left])
                .is_err()
        );
        assert!(
            session
                .validate_live_bracket_pair_target_v1(&[
                    object(&session, "left"),
                    DocumentObjectIdV1::from_entropy_bytes([0; 16]),
                ])
                .is_err()
        );
        assert_eq!(session.snapshot().expect("after"), before);
    }
}
