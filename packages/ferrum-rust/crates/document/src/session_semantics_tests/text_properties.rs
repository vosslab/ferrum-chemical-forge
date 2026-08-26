//! Atomic durable direct-root Text properties behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    DocumentObjectIdV1, PresentationFactProvenanceV1, PresentationFontFaceV1,
    PresentationRootProjectionV1, Rgb24V1, TextEditRunV1, TextEditStyleV1, TextPropertiesPatchV1,
    TextPropertiesPatchV1Error, TextPropertyChangeV1,
};

const SOURCE: &str = concat!(
    "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
    "xmlns:v=\"urn:vendor\"><c:text id=\"t\" background-color=\"#fff\" keep=\"yes\">",
    "<c:point x=\"10\" y=\"20\"/><v:between retained=\"yes\"/>",
    "<c:font family=\"Telex\" size=\"12\" color=\"#000\" v:font-keep=\"yes\">",
    "<v:font-child/></c:font><c:ftext>old</c:ftext></c:text><v:root/></c:cdml>",
);

fn run(text: &str, styles: Vec<TextEditStyleV1>) -> TextEditRunV1 {
    TextEditRunV1::new(text, styles).expect("valid Text edit run")
}

fn patch(session: &DocumentSession, changes: Vec<TextPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetTextProperties {
        patch: TextPropertiesPatchV1::new(text_object_id(session), changes)
            .expect("valid Text patch"),
    })
}

fn test_object_id() -> DocumentObjectIdV1 {
    DocumentObjectIdV1::from_entropy_bytes([0; 16])
}

fn text_object_id(session: &DocumentSession) -> DocumentObjectIdV1 {
    let revision = session.snapshot().expect("snapshot").revision();
    let observation = session.observe(revision).expect("observation");
    observation
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
        .expect("expected direct-root Text")
}

fn text(observation: &crate::SessionDocumentObservationV1) -> &crate::TextProjectionV1 {
    observation
        .projection()
        .presentation_stack()
        .entries()
        .iter()
        .find_map(|entry| match entry.root() {
            PresentationRootProjectionV1::Text { text } => Some(text),
            _ => None,
        })
        .expect("expected direct-root Text")
}

#[test]
fn text_properties_commit_semantic_runs_preserve_extensions_and_follow_history() {
    let runs = vec![
        run("H", vec![]),
        run("2", vec![TextEditStyleV1::Subscript]),
        run("O <&>", vec![]),
    ];
    let changes = vec![
        TextPropertyChangeV1::Runs(runs),
        TextPropertyChangeV1::FontFace(PresentationFontFaceV1::TelexRegularV1),
        TextPropertyChangeV1::FontSize(18),
        TextPropertyChangeV1::Color(Rgb24V1::new("#AbC").unwrap()),
        TextPropertyChangeV1::BackgroundColor(None),
    ];
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(0, patch(&session, changes))
        .expect("patch must commit");
    let projected = text(changed.observation());
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(projected.font().font_face().id(), "telex_regular_v1");
    assert_eq!(projected.font().size().value(), 18.0);
    assert_eq!(projected.font().color().as_str(), "#aabbcc");
    assert_eq!(projected.background().color(), None);
    assert_eq!(
        projected.background().color_provenance(),
        PresentationFactProvenanceV1::Root
    );
    assert_eq!(projected.runs().len(), 3);
    assert_eq!(projected.runs()[0].text(), "H");
    assert_eq!(
        projected.runs()[1].styles(),
        &[crate::PresentationTextStyleV1::Subscript]
    );
    assert_eq!(projected.runs()[2].text(), "O <&>");

    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("keep=\"yes\""));
    assert!(cdml.contains("retained=\"yes\""));
    assert!(cdml.contains("font-keep=\"yes\""));
    assert!(cdml.contains("<v:font-child"));
    assert!(cdml.contains("<v:root"));

    let undone = session.undo(1).expect("one patch must undo once");
    assert_eq!(text(undone.observation()).runs()[0].text(), "old");
    let redone = session.redo(2).expect("one patch must redo once");
    assert_eq!(text(redone.observation()).runs()[2].text(), "O <&>");
}

#[test]
fn text_properties_reject_invalid_intent_and_targets_without_mutation() {
    assert!(matches!(
        DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><text id=\"t\"><point x=\"0\" y=\"0\"/><font family=\"Arial\"/><ftext>x</ftext></text></cdml>"
        ),
        Err(DocumentSessionError::Load(
            TypedDocumentError::UnsupportedTextFace { .. }
        ))
    ));
    assert_eq!(
        TextEditRunV1::new(
            "x",
            vec![TextEditStyleV1::Subscript, TextEditStyleV1::Superscript]
        ),
        Err(TextPropertiesPatchV1Error::ConflictingScriptStyles)
    );
    assert_eq!(
        TextPropertiesPatchV1::new(
            test_object_id(),
            vec![TextPropertyChangeV1::Runs(vec![run(" \n", vec![])])]
        ),
        Err(TextPropertiesPatchV1Error::BlankText)
    );
    assert_eq!(
        TextPropertiesPatchV1::new(test_object_id(), vec![TextPropertyChangeV1::FontSize(3)]),
        Err(TextPropertiesPatchV1Error::FontSizeOutOfRange)
    );

    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    let unknown = TextPropertiesPatchV1::new(
        test_object_id(),
        vec![TextPropertyChangeV1::Runs(vec![run("new", vec![])])],
    )
    .unwrap();
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetTextProperties { patch: unknown })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownText(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let cross_kind_source = SOURCE.replace(
        "<v:root/>",
        "<c:plus id=\"p\"><c:point x=\"0\" y=\"0\"/></c:plus><v:root/>",
    );
    let mut cross_kind = DocumentSession::load(&cross_kind_source).expect("source loads");
    let text_id = text_object_id(&cross_kind);
    let revision = cross_kind.snapshot().expect("snapshot").revision();
    let foreign_id = cross_kind
        .observe(revision)
        .expect("observation")
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
        .expect("Plus root has a durable ID");
    assert_ne!(foreign_id, text_id);
    let foreign = TextPropertiesPatchV1::new(foreign_id, vec![TextPropertyChangeV1::FontSize(18)])
        .expect("valid cross-kind patch");
    let before = cross_kind.snapshot().expect("snapshot");
    assert!(matches!(
        cross_kind.apply_document_operation_v1(
            0,
            SessionOperation::V1(SessionOperationV1::SetTextProperties { patch: foreign })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownText(_)
        ))
    ));
    assert_eq!(cross_kind.snapshot().expect("snapshot"), before);

    let ambiguous_source =
        SOURCE.replace("<c:ftext>old</c:ftext>", "<c:font/><c:ftext>old</c:ftext>");
    let mut ambiguous = DocumentSession::load(&ambiguous_source).expect("source loads");
    let before = ambiguous.snapshot().expect("snapshot");
    assert!(matches!(
        ambiguous.apply_document_operation_v1(
            0,
            patch(&ambiguous, vec![TextPropertyChangeV1::FontSize(18)])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::AmbiguousTextFonts(_))
        ))
    ));
    assert_eq!(ambiguous.snapshot().expect("snapshot"), before);
}

#[test]
fn typed_text_face_alias_is_canonicalized_before_session_state_exists() {
    let session = DocumentSession::load(
        "<cdml xmlns=\"urn:ferrum:cdml\"><text id=\"t\"><point x=\"0\" y=\"0\"/><font family=\"Telex Regular\"/><ftext>x</ftext></text></cdml>",
    )
    .expect("approved Telex alias must load");
    assert!(
        session
            .snapshot()
            .expect("snapshot")
            .cdml()
            .contains("family=\"Telex\"")
    );
}

#[test]
fn stale_text_properties_patch_is_atomic_and_equal_intent_is_history_free() {
    let mut session = DocumentSession::load(SOURCE).expect("source must load");
    let equal = session
        .apply_document_operation_v1(0, patch(&session, vec![TextPropertyChangeV1::FontSize(12)]))
        .expect("equal patch must be accepted");
    assert_eq!(equal.observation().snapshot().revision(), 0);
    session
        .apply_document_operation_v1(0, patch(&session, vec![TextPropertyChangeV1::FontSize(18)]))
        .expect("first change must commit");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.apply_document_operation_v1(
            0,
            patch(&session, vec![TextPropertyChangeV1::FontSize(20)])
        ),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);
}
