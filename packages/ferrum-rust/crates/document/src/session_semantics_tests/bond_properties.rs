//! Atomic durable bond-presentation patch behavior.

use ferrum_core::{BondOrder, BondStyle};

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    BondPropertiesPatchV1, BondPropertiesPatchV1Error, BondPropertyChangeV1,
    DocumentBondPresentationV1, NonZeroFiniteV1, PositiveFiniteV1, Rgb24V1,
};

const PROPERTY_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"81\" y=\"2\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"w1\" center=\"no\" ",
    "line_width=\"1.5\" bond_width=\"2\" wedge_width=\"3\" color=\"#A0B1C2\" ",
    "vendor_keep=\"yes\"><v:opaque/></bond></molecule><v:root_keep/></cdml>"
);

fn patch(changes: Vec<BondPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetBondProperties {
        patch: BondPropertiesPatchV1::new("ab", changes).expect("valid bond patch"),
    })
}

#[test]
fn closed_presentation_patch_commits_preserves_extensions_and_follows_history() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let changed = session
        .apply_document_operation_v1(
            0,
            patch(vec![
                BondPropertyChangeV1::Presentation(DocumentBondPresentationV1::Dashed),
                BondPropertyChangeV1::Center(None),
                BondPropertyChangeV1::LineWidth(Some(PositiveFiniteV1::new(2.5).unwrap())),
                BondPropertyChangeV1::BondWidth(None),
                BondPropertyChangeV1::WedgeWidth(None),
                BondPropertyChangeV1::Color(Some(Rgb24V1::new("#102030").unwrap())),
            ]),
        )
        .expect("closed presentation patch commits");
    let bond = &changed.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(bond.source_type(), Some("d1"));
    assert_eq!(bond.order(), Some(BondOrder::Single));
    assert_eq!(bond.style(), Some(&BondStyle::Dashed));
    let snapshot = changed.observation().snapshot();
    assert!(snapshot.cdml().contains("vendor_keep=\"yes\""));
    assert!(snapshot.cdml().contains("<v:opaque"));
    let undone = session.undo(1).expect("undo");
    assert!(
        !undone
            .observation()
            .snapshot()
            .cdml()
            .contains("type=\"d1\"")
    );
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("redo");
    assert!(
        redone
            .observation()
            .snapshot()
            .cdml()
            .contains("type=\"d1\"")
    );
}

#[test]
fn closed_presentation_tokens_are_the_only_authorable_forms() {
    for token in ["n1", "n2", "n3", "w1", "h1", "q1", "b1", "d1", "s1"] {
        assert!(DocumentBondPresentationV1::from_cdml_token(token).is_some());
    }
    for token in ["w2", "h3", "b2", "d3", "s2", "a1", "o1"] {
        assert!(DocumentBondPresentationV1::from_cdml_token(token).is_none());
    }
}

#[test]
fn duplicate_presentation_patch_is_refused_before_document_lookup() {
    assert!(matches!(
        BondPropertiesPatchV1::new(
            "ab",
            vec![
                BondPropertyChangeV1::Presentation(DocumentBondPresentationV1::Bold),
                BondPropertyChangeV1::Presentation(DocumentBondPresentationV1::Wavy),
            ],
        ),
        Err(BondPropertiesPatchV1Error::DuplicateChange { .. })
    ));
}

#[test]
fn incompatible_scalar_properties_are_refused_atomically_after_presentation_resolution() {
    let incompatible_changes = [
        (BondPropertyChangeV1::Center(Some(false)), "center"),
        (
            BondPropertyChangeV1::BondWidth(Some(NonZeroFiniteV1::new(2.0).unwrap())),
            "bond_width",
        ),
        (
            BondPropertyChangeV1::WedgeWidth(Some(PositiveFiniteV1::new(2.0).unwrap())),
            "wedge_width",
        ),
    ];
    for (change, property) in incompatible_changes {
        let source = PROPERTY_SOURCE
            .replace(" center=\"no\"", "")
            .replace(" bond_width=\"2\"", "")
            .replace(" wedge_width=\"3\"", "");
        let mut session = DocumentSession::load(&source).expect("source must load");
        let before = session.snapshot().expect("snapshot");
        assert!(matches!(
            session
                .apply_document_operation_v1(
                    0,
                    patch(vec![
                        BondPropertyChangeV1::Presentation(DocumentBondPresentationV1::Dashed),
                        change,
                    ]),
                )
                ,
            Err(DocumentSessionError::Operation(SessionOperationError::Candidate(
                TypedDocumentError::IncompatibleBondPresentationProperty { property: actual, .. }
            ))) if actual == property
        ));
        assert_eq!(session.snapshot().expect("snapshot"), before);
    }
}

#[test]
fn presentation_change_must_clear_retained_incompatible_scalar_properties() {
    let source = PROPERTY_SOURCE.replace("type=\"w1\"", "type=\"n2\"");
    let mut session = DocumentSession::load(&source).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    assert!(
        session
            .apply_document_operation_v1(
                0,
                patch(vec![BondPropertyChangeV1::Presentation(
                    DocumentBondPresentationV1::SolidWedge,
                )]),
            )
            .is_err()
    );
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let changed = session
        .apply_document_operation_v1(
            0,
            patch(vec![
                BondPropertyChangeV1::Presentation(DocumentBondPresentationV1::SolidWedge),
                BondPropertyChangeV1::Center(None),
                BondPropertyChangeV1::BondWidth(None),
            ]),
        )
        .expect("explicit clears make the final state compatible");
    let bond = &changed.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(bond.source_type(), Some("w1"));
    assert_eq!(bond.center(), None);
    assert_eq!(bond.bond_width(), None);
    assert!(bond.wedge_width().is_some());
}
