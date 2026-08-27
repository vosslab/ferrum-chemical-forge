//! Durable lowering for live direct-root presentation mutations.

use crate::{
    DocumentObjectIdV1, PresentationRecordKindV1, PresentationRootSelectorV1,
    SessionOperationError, TypedClass,
};

use super::DocumentSession;

impl DocumentSession {
    /// Lower exact durable presentation roots after validating current direct-root kinds.
    pub fn lower_live_presentation_roots_v1(
        &self,
        targets: &[(DocumentObjectIdV1, PresentationRecordKindV1)],
    ) -> Result<Vec<PresentationRootSelectorV1>, SessionOperationError> {
        targets
            .iter()
            .map(|(object_id, kind)| {
                let record = self
                    .current_document_v1()
                    .resolve_document_object_id(object_id)?
                    .ok_or_else(|| {
                        SessionOperationError::UnknownDocumentObject(object_id.as_str().to_owned())
                    })?;
                if record.path().components().len() != 1
                    || presentation_kind(record.class()) != Some(*kind)
                {
                    return Err(SessionOperationError::UnknownPresentationRoot(
                        object_id.as_str().to_owned(),
                    ));
                }
                Ok(PresentationRootSelectorV1::new(object_id.clone(), *kind))
            })
            .collect()
    }
}

const fn presentation_kind(class: TypedClass) -> Option<PresentationRecordKindV1> {
    match class {
        TypedClass::CanvasArrow => Some(PresentationRecordKindV1::Arrow),
        TypedClass::CanvasPlus => Some(PresentationRecordKindV1::Plus),
        TypedClass::CanvasText => Some(PresentationRecordKindV1::Text),
        TypedClass::Polyline => Some(PresentationRecordKindV1::Polyline),
        TypedClass::Rectangle => Some(PresentationRecordKindV1::Rectangle),
        TypedClass::Square => Some(PresentationRecordKindV1::Square),
        TypedClass::Oval => Some(PresentationRecordKindV1::Oval),
        TypedClass::Circle => Some(PresentationRecordKindV1::Circle),
        TypedClass::Polygon => Some(PresentationRecordKindV1::Polygon),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PersistentId;

    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><text id=\"note\"><point x=\"0\" y=\"0\"/>",
        "<ftext>note</ftext></text><plus id=\"plus\"><point x=\"1\" y=\"1\"/></plus>",
        "<molecule id=\"m\"><atom id=\"atom\" name=\"C\"><point x=\"2\" y=\"2\"/>",
        "</atom></molecule></cdml>",
    );

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
    fn durable_presentation_lowering_requires_current_direct_root_and_exact_kind() {
        let session = DocumentSession::load(SOURCE).expect("document");
        let before = session.snapshot().expect("before");
        let targets = session
            .lower_live_presentation_roots_v1(&[(
                object(&session, "note"),
                PresentationRecordKindV1::Text,
            )])
            .expect("current text root lowers");
        assert_eq!(targets[0].document_object_id(), &object(&session, "note"));
        assert!(
            session
                .lower_live_presentation_roots_v1(&[(
                    object(&session, "note"),
                    PresentationRecordKindV1::Plus,
                )])
                .is_err()
        );
        assert!(
            session
                .lower_live_presentation_roots_v1(&[(
                    object(&session, "atom"),
                    PresentationRecordKindV1::Text,
                )])
                .is_err()
        );
        assert_eq!(session.snapshot().expect("after"), before);
    }
}
