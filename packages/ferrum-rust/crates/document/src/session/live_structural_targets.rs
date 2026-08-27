//! Durable live-session target lowering for structural mutation adapters.

use crate::{
    AtomRotationTargetV1, DocumentObjectIdV1, PersistentId, SessionOperationError,
    TopLevelRootKindV1, TopLevelRootSelectorV1, TypedClass,
};

use super::DocumentSession;

impl DocumentSession {
    /// Lower durable molecule/atom pairs after validating current containment.
    pub fn lower_live_atom_rotation_targets_v1(
        &self,
        targets: &[(DocumentObjectIdV1, DocumentObjectIdV1)],
    ) -> Result<Vec<AtomRotationTargetV1>, SessionOperationError> {
        let mut lowered = Vec::with_capacity(targets.len());
        for (molecule_object_id, atom_object_id) in targets {
            let molecule = self.live_root(molecule_object_id, TypedClass::Molecule)?;
            let mut atom = None;
            for child in molecule.typed_children() {
                let child_object_id =
                    crate::projection_identity_v1::projection_document_object_id_from_record_v1(
                        child.record(),
                    )
                    .map_err(|_| {
                        SessionOperationError::UnknownDocumentObject(
                            atom_object_id.as_str().to_owned(),
                        )
                    })?;
                if child_object_id == *atom_object_id {
                    atom = Some(child.record());
                    break;
                }
            }
            let atom = atom.ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(atom_object_id.as_str().to_owned())
            })?;
            if atom.class() != TypedClass::Atom {
                return Err(SessionOperationError::InvalidCreateBondTarget(
                    atom_object_id.as_str().to_owned(),
                ));
            }
            lowered.push(
                AtomRotationTargetV1::new(
                    source_id(molecule, molecule_object_id)?,
                    source_id(atom, atom_object_id)?,
                )
                .map_err(|_| {
                    SessionOperationError::InvalidCreateBondTarget(
                        atom_object_id.as_str().to_owned(),
                    )
                })?,
            );
        }
        Ok(lowered)
    }

    /// Lower durable molecule roots, with an empty selection meaning all current molecules.
    pub fn lower_live_geometry_repair_molecules_v1(
        &self,
        molecule_object_ids: &[DocumentObjectIdV1],
    ) -> Result<Vec<String>, SessionOperationError> {
        if molecule_object_ids.is_empty() {
            return self
                .current_document_v1()
                .root()
                .typed_children()
                .iter()
                .filter(|child| child.record().class() == TypedClass::Molecule)
                .map(|child| {
                    let record = child.record();
                    let object_id = crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)
                        .map_err(|_| {
                            SessionOperationError::UnknownDocumentObject("molecule".to_owned())
                        })?;
                    source_id(record, &object_id)
                })
                .collect();
        }
        molecule_object_ids
            .iter()
            .map(|object_id| {
                let record = self.live_root(object_id, TypedClass::Molecule)?;
                source_id(record, object_id)
            })
            .collect()
    }

    /// Lower complete direct roots after validating their closed kind.
    pub fn lower_live_top_level_roots_v1(
        &self,
        targets: &[(DocumentObjectIdV1, TopLevelRootKindV1)],
    ) -> Result<Vec<TopLevelRootSelectorV1>, SessionOperationError> {
        targets
            .iter()
            .map(|(object_id, kind)| {
                self.live_root(object_id, typed_class(*kind))?;
                Ok(TopLevelRootSelectorV1::new(object_id.clone(), *kind))
            })
            .collect()
    }

    fn live_root(
        &self,
        object_id: &DocumentObjectIdV1,
        class: TypedClass,
    ) -> Result<&crate::TypedRecord, SessionOperationError> {
        let record = self
            .current_document_v1()
            .resolve_document_object_id(object_id)
            .map_err(|_| {
                SessionOperationError::InvalidCreateAtomTarget(object_id.as_str().to_owned())
            })?
            .ok_or_else(|| {
                SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned())
            })?;
        if record.class() != class || record.path().components().len() != 1 {
            return Err(SessionOperationError::InvalidCreateAtomTarget(
                object_id.as_str().to_owned(),
            ));
        }
        Ok(record)
    }
}

fn source_id(
    record: &crate::TypedRecord,
    object_id: &DocumentObjectIdV1,
) -> Result<String, SessionOperationError> {
    let source = record.attribute("id").ok_or_else(|| {
        SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned())
    })?;
    PersistentId::new(source.to_owned())
        .map(|value| value.as_str().to_owned())
        .map_err(|_| SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned()))
}

const fn typed_class(kind: TopLevelRootKindV1) -> TypedClass {
    match kind {
        TopLevelRootKindV1::Molecule => TypedClass::Molecule,
        TopLevelRootKindV1::Arrow => TypedClass::CanvasArrow,
        TopLevelRootKindV1::Plus => TypedClass::CanvasPlus,
        TopLevelRootKindV1::Text => TypedClass::CanvasText,
        TopLevelRootKindV1::Rectangle => TypedClass::Rectangle,
        TopLevelRootKindV1::Square => TypedClass::Square,
        TopLevelRootKindV1::Oval => TypedClass::Oval,
        TopLevelRootKindV1::Circle => TypedClass::Circle,
        TopLevelRootKindV1::Polygon => TypedClass::Polygon,
        TopLevelRootKindV1::Polyline => TypedClass::Polyline,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.08\"><molecule id=\"m-a\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"m-b\"><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom></molecule><arrow id=\"arrow-a\" type=\"normal\" start=\"0 0\" end=\"10 0\"/></cdml>";

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
    fn durable_rotation_lowering_accepts_current_parent_child_and_refuses_foreign_child() {
        let session = DocumentSession::load(SOURCE).expect("document");
        let before = session.snapshot().expect("before");
        let molecule = object(&session, "m-a");
        let atom = object(&session, "a");
        let lowered = session
            .lower_live_atom_rotation_targets_v1(&[(molecule.clone(), atom)])
            .expect("current durable target lowers");
        assert_eq!(lowered[0].molecule_id().as_str(), "m-a");
        assert_eq!(lowered[0].atom_id().as_str(), "a");
        assert!(
            session
                .lower_live_atom_rotation_targets_v1(&[(molecule, object(&session, "b"))])
                .is_err()
        );
        let after = session.snapshot().expect("after");
        assert_eq!(after.revision(), before.revision());
        assert_eq!(after.digest(), before.digest());
    }

    #[test]
    fn durable_geometry_and_root_lowering_validate_current_kinds() {
        let session = DocumentSession::load(SOURCE).expect("document");
        let molecule = object(&session, "m-a");
        let all = session
            .lower_live_geometry_repair_molecules_v1(&[])
            .expect("empty selection resolves current molecules in Rust");
        assert_eq!(all, vec!["m-a", "m-b"]);
        assert!(
            session
                .lower_live_geometry_repair_molecules_v1(&[object(&session, "a")])
                .is_err()
        );
        assert!(
            session
                .lower_live_top_level_roots_v1(&[(molecule, TopLevelRootKindV1::Arrow)])
                .is_err()
        );
        assert!(
            session
                .lower_live_top_level_roots_v1(&[(
                    object(&session, "arrow-a"),
                    TopLevelRootKindV1::Arrow,
                )])
                .is_ok()
        );
    }
}
