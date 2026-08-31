//! Atomic durable direct-root Plus properties behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    CDML_NAMESPACE, DocumentObjectIdV1, PlusPropertiesPatchV1, PlusPropertiesPatchV1Error,
    PlusPropertyChangeV1, PresentationFactProvenanceV1, PresentationFontFaceV1,
    PresentationRootProjectionV1, Rgb24V1, element_name,
};
use xot::Xot;

const SOURCE: &str = concat!(
    "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
    "xmlns:v=\"urn:vendor\"><c:plus id=\"p\" font_size=\"14\" color=\"#000\" ",
    "background-color=\"#ffffff\" keep=\"yes\"><c:point x=\"10\" y=\"20\"/>",
    "<v:opaque retained=\"yes\"/></c:plus><v:root/></c:cdml>"
);

fn patch(session: &DocumentSession, changes: Vec<PlusPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetPlusProperties {
        patch: PlusPropertiesPatchV1::new(plus_object_id(session), changes)
            .expect("valid Plus patch"),
    })
}

fn test_object_id() -> DocumentObjectIdV1 {
    DocumentObjectIdV1::from_entropy_bytes([0; 16])
}

fn plus_object_id(session: &DocumentSession) -> DocumentObjectIdV1 {
    let revision = session.snapshot().expect("snapshot").revision();
    let observation = session.observe(revision).expect("observation");
    observation
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Plus { .. } => {
                Some(entry.root().target().document_object_id().clone())
            }
            _ => None,
        })
        .expect("expected direct-root Plus")
}

fn plus(observation: &crate::SessionDocumentObservationV1) -> &crate::PlusProjectionV1 {
    observation
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Plus { plus } => Some(plus),
            _ => None,
        })
        .expect("expected direct-root Plus")
}

#[test]
fn plus_properties_commit_once_preserve_extensions_and_follow_history() {
    let changes = vec![
        PlusPropertyChangeV1::FontFace(PresentationFontFaceV1::MoleculeLabel),
        PlusPropertyChangeV1::FontSize(18),
        PlusPropertyChangeV1::Color(Rgb24V1::new("#AbC").unwrap()),
        PlusPropertyChangeV1::BackgroundColor(None),
    ];
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, patch(&session, changes))
        .expect("patch must commit");
    let projected = plus(changed.observation());
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(projected.font().font_face().id(), "molecule_label");
    assert_eq!(projected.font().size().value(), 18.0);
    assert_eq!(projected.font().color().as_str(), "#aabbcc");
    assert_eq!(
        projected.font().font_face_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(projected.background().color(), None);
    assert_eq!(
        projected.background().color_provenance(),
        PresentationFactProvenanceV1::Root
    );

    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("keep=\"yes\""));
    assert!(cdml.contains("retained=\"yes\""));
    assert!(cdml.contains("<v:root"));
    let mut tree = Xot::new();
    let document = tree.parse(cdml).expect("candidate XML must parse");
    let root = tree.document_element(document).unwrap();
    let plus_node = tree
        .children(root)
        .find(|node| element_name(&tree, *node).is_some_and(|(local, _)| local == "plus"))
        .unwrap();
    let children = tree
        .children(plus_node)
        .filter_map(|node| element_name(&tree, node))
        .collect::<Vec<_>>();
    assert_eq!(
        children,
        vec![
            ("point".to_owned(), CDML_NAMESPACE.to_owned()),
            ("font".to_owned(), CDML_NAMESPACE.to_owned()),
            ("opaque".to_owned(), "urn:vendor".to_owned()),
        ]
    );

    let undone = session.undo(1).expect("one patch must undo once");
    assert_eq!(
        plus(undone.observation()).font().font_face().id(),
        "molecule_label"
    );
    let redone = session.redo(2).expect("one patch must redo once");
    assert_eq!(
        plus(redone.observation()).font().font_face().id(),
        "molecule_label"
    );
}

#[test]
fn plus_properties_reject_invalid_intent_and_targets_without_mutation() {
    assert_eq!(
        PlusPropertiesPatchV1::new(test_object_id(), vec![PlusPropertyChangeV1::FontSize(3)]),
        Err(PlusPropertiesPatchV1Error::FontSizeOutOfRange)
    );
    assert_eq!(
        PlusPropertiesPatchV1::new(
            test_object_id(),
            vec![
                PlusPropertyChangeV1::Color(Rgb24V1::new("#000").unwrap()),
                PlusPropertyChangeV1::Color(Rgb24V1::new("#fff").unwrap()),
            ],
        ),
        Err(PlusPropertiesPatchV1Error::DuplicateChange { property: "color" })
    );

    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    let unknown =
        PlusPropertiesPatchV1::new(test_object_id(), vec![PlusPropertyChangeV1::FontSize(18)])
            .unwrap();
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetPlusProperties { patch: unknown })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownPlus(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let cross_kind_source = SOURCE.replace(
        "<v:root/>",
        "<c:text id=\"t\"><c:point x=\"0\" y=\"0\"/><c:ftext>x</c:ftext></c:text><v:root/>",
    );
    let mut cross_kind = DocumentSession::load(&cross_kind_source).expect("source loads");
    let plus_id = plus_object_id(&cross_kind);
    let revision = cross_kind.snapshot().expect("snapshot").revision();
    let foreign_id = cross_kind
        .observe(revision)
        .expect("observation")
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Text { .. } => {
                Some(entry.root().target().document_object_id().clone())
            }
            _ => None,
        })
        .expect("Text root has a durable ID");
    assert_ne!(foreign_id, plus_id);
    let foreign = PlusPropertiesPatchV1::new(foreign_id, vec![PlusPropertyChangeV1::FontSize(18)])
        .expect("valid cross-kind patch");
    let before = cross_kind.snapshot().expect("snapshot");
    assert!(matches!(
        cross_kind.apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetPlusProperties { patch: foreign })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownPlus(_)
        ))
    ));
    assert_eq!(cross_kind.snapshot().expect("snapshot"), before);

    let ambiguous_source = SOURCE.replace(
        "<v:opaque retained=\"yes\"/>",
        "<c:font/><c:font/><v:opaque retained=\"yes\"/>",
    );
    let mut ambiguous = DocumentSession::load(&ambiguous_source).expect("source loads");
    let before = ambiguous.snapshot().expect("snapshot");
    assert!(matches!(
        ambiguous.apply_document_operation_v1(
            0,
            patch(
                &ambiguous,
                vec![PlusPropertyChangeV1::FontFace(
                    PresentationFontFaceV1::MoleculeLabel
                )]
            )
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::AmbiguousPlusFonts(_))
        ))
    ));
    assert_eq!(ambiguous.snapshot().expect("snapshot"), before);
}

#[test]
fn stale_plus_properties_patch_is_atomic_and_equal_intent_is_history_free() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let equal = session
        .apply_document_operation_v1(0, patch(&session, vec![PlusPropertyChangeV1::FontSize(14)]))
        .expect("equal patch must be accepted");
    assert_eq!(equal.observation().snapshot().revision(), 0);
    session
        .apply_document_operation_v1(0, patch(&session, vec![PlusPropertyChangeV1::FontSize(18)]))
        .expect("first change must commit");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            patch(&session, vec![PlusPropertyChangeV1::FontSize(20)])
        ),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);
}
