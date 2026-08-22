//! Atomic durable bond-properties patch behavior.

use ferrum_core::{BondOrder, BondStyle};

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    BondPropertiesPatchV1, BondPropertiesPatchV1Error, BondPropertyChangeV1, CDML_NAMESPACE,
    DocumentBondOrderV1, DocumentBondStyleV1, NonZeroFiniteV1, PositiveFiniteV1, Rgb24V1,
    element_name,
};
use xot::Xot;

const PROPERTY_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"1\" y=\"2\"/></atom>",
    "<atom id=\"b\" name=\"O\"><point x=\"3\" y=\"2\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"w2\" center=\"no\" ",
    "line_width=\"1.5\" bond_width=\"-2\" wedge_width=\"3\" color=\"#A0B1C2\" ",
    "vendor_keep=\"yes\"><v:opaque/></bond></molecule><v:root_keep/></cdml>"
);

fn patch(changes: Vec<BondPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetBondProperties {
        patch: BondPropertiesPatchV1::new("ab", changes).expect("valid bond patch"),
    })
}

#[test]
fn bond_properties_commit_once_preserve_extensions_and_follow_history() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let changed = session
        .submit(
            0,
            patch(vec![
                BondPropertyChangeV1::Order(DocumentBondOrderV1::Triple),
                BondPropertyChangeV1::Style(DocumentBondStyleV1::Dashed),
                BondPropertyChangeV1::Center(Some(true)),
                BondPropertyChangeV1::LineWidth(Some(PositiveFiniteV1::new(2.5).unwrap())),
                BondPropertyChangeV1::BondWidth(Some(NonZeroFiniteV1::new(-4.0).unwrap())),
                BondPropertyChangeV1::WedgeWidth(Some(PositiveFiniteV1::new(5.0).unwrap())),
                BondPropertyChangeV1::Color(Some(Rgb24V1::new("#102030").unwrap())),
            ]),
        )
        .expect("patch must commit");
    let bond = &changed.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(bond.source_type(), Some("d3"));
    assert_eq!(bond.order(), Some(BondOrder::Triple));
    assert_eq!(bond.style(), Some(&BondStyle::Dashed));
    assert_eq!(bond.center(), Some(true));
    assert_eq!(bond.line_width().unwrap().value(), 2.5);
    assert_eq!(bond.bond_width().unwrap().value(), -4.0);
    assert_eq!(bond.wedge_width().unwrap().value(), 5.0);
    assert_eq!(bond.color().unwrap().as_str(), "#102030");
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("vendor_keep=\"yes\""));
    assert!(cdml.contains("<v:opaque"));
    assert!(cdml.contains("<v:root_keep"));
    assert!(cdml.contains("start=\"a\""));
    assert!(cdml.contains("end=\"b\""));

    let undone = session.undo(1).expect("one patch must undo once");
    assert_eq!(
        undone.observation().projection().molecules()[0].bonds()[0].source_type(),
        Some("w2")
    );
    let redone = session.redo(2).expect("one patch must redo once");
    assert_eq!(
        redone.observation().projection().molecules()[0].bonds()[0].source_type(),
        Some("d3")
    );
}

#[test]
fn bond_properties_preserve_known_component_and_clear_optional_facts() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let changed = session
        .submit(
            0,
            patch(vec![
                BondPropertyChangeV1::Order(DocumentBondOrderV1::Single),
                BondPropertyChangeV1::Center(None),
                BondPropertyChangeV1::LineWidth(None),
                BondPropertyChangeV1::BondWidth(None),
                BondPropertyChangeV1::WedgeWidth(None),
                BondPropertyChangeV1::Color(None),
            ]),
        )
        .expect("optional facts must clear");
    let bond = &changed.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(bond.source_type(), Some("w1"));
    assert_eq!(bond.style(), Some(&BondStyle::Wedge));
    assert_eq!(bond.center(), None);
    assert_eq!(bond.line_width(), None);
    assert_eq!(bond.bond_width(), None);
    assert_eq!(bond.wedge_width(), None);
    assert_eq!(bond.color(), None);

    let styled = session
        .submit(
            1,
            patch(vec![BondPropertyChangeV1::Style(
                DocumentBondStyleV1::HaworthFront,
            )]),
        )
        .expect("style preserves current order");
    assert_eq!(
        styled.observation().projection().molecules()[0].bonds()[0].source_type(),
        Some("q1")
    );
}

#[test]
fn haworth_front_rejects_non_single_final_type_without_state_change() {
    assert!(matches!(
        BondPropertiesPatchV1::new(
            "ab",
            vec![
                BondPropertyChangeV1::Style(DocumentBondStyleV1::HaworthFront),
                BondPropertyChangeV1::Order(DocumentBondOrderV1::Double),
            ],
        ),
        Err(BondPropertiesPatchV1Error::UnsupportedStyleOrder)
    ));

    let mut style_session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let style_before = style_session.snapshot().expect("snapshot");
    assert!(matches!(
        style_session.submit(
            0,
            patch(vec![BondPropertyChangeV1::Style(
                DocumentBondStyleV1::HaworthFront,
            )])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::UnsupportedBondStyleOrder(_))
        ))
    ));
    assert_eq!(style_session.snapshot().expect("snapshot"), style_before);

    let haworth_source = PROPERTY_SOURCE.replace("type=\"w2\"", "type=\"q1\"");
    let mut order_session = DocumentSession::load(&haworth_source).expect("source must load");
    let order_before = order_session.snapshot().expect("snapshot");
    assert!(matches!(
        order_session.submit(
            0,
            patch(vec![BondPropertyChangeV1::Order(
                DocumentBondOrderV1::Double,
            )])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::UnsupportedBondStyleOrder(_))
        ))
    ));
    assert_eq!(order_session.snapshot().expect("snapshot"), order_before);
}

#[test]
fn presentation_only_patch_leaves_opaque_type_untouched() {
    let source = PROPERTY_SOURCE.replace("type=\"w2\"", "type=\"mystery77\"");
    let mut session = DocumentSession::load(&source).expect("opaque type remains retained");
    let changed = session
        .submit(0, patch(vec![BondPropertyChangeV1::Center(Some(true))]))
        .expect("presentation patch does not interpret type");
    let bond = &changed.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(bond.source_type(), Some("mystery77"));
    assert_eq!(bond.center(), Some(true));
}

#[test]
fn empty_and_equal_bond_properties_patches_are_history_free() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let empty = session.submit(0, patch(Vec::new())).expect("empty patch");
    assert_eq!(empty.observation().snapshot().revision(), 0);
    let equal = session
        .submit(
            0,
            patch(vec![BondPropertyChangeV1::Style(
                DocumentBondStyleV1::Wedge,
            )]),
        )
        .expect("equal property patch");
    assert_eq!(equal.observation().snapshot().revision(), 0);
}

#[test]
fn bond_properties_reject_invalid_intent_target_and_type_without_state_change() {
    assert!(matches!(
        BondPropertiesPatchV1::new(
            "ab",
            vec![
                BondPropertyChangeV1::Center(Some(true)),
                BondPropertyChangeV1::Center(Some(false)),
            ],
        ),
        Err(BondPropertiesPatchV1Error::DuplicateChange { .. })
    ));
    assert_eq!(NonZeroFiniteV1::new(0.0), None);
    assert_eq!(NonZeroFiniteV1::new(f64::NAN), None);
    assert_eq!(NonZeroFiniteV1::new(f64::INFINITY), None);
    assert_eq!(PositiveFiniteV1::new(f64::NEG_INFINITY), None);
    assert_eq!(NonZeroFiniteV1::new(f64::MAX).unwrap().value(), f64::MAX);
    assert_eq!(PositiveFiniteV1::new(f64::MAX).unwrap().value(), f64::MAX);

    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    let unknown =
        BondPropertiesPatchV1::new("missing", vec![BondPropertyChangeV1::Center(Some(true))])
            .expect("intent is independently valid");
    assert!(matches!(
        session.submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetBondProperties { patch: unknown })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownBond(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let source = PROPERTY_SOURCE.replace("type=\"w2\"", "type=\"l2\"");
    let mut unsupported = DocumentSession::load(&source).expect("legacy source remains retained");
    let before = unsupported.snapshot().expect("snapshot");
    assert!(matches!(
        unsupported.submit(
            0,
            patch(vec![BondPropertyChangeV1::Order(
                DocumentBondOrderV1::Single
            )])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::UnsupportedBondType(_))
        ))
    ));
    assert_eq!(unsupported.snapshot().expect("snapshot"), before);
}

#[test]
fn stale_bond_properties_patch_does_not_change_authoritative_snapshot() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    session
        .submit(0, patch(vec![BondPropertyChangeV1::Center(Some(true))]))
        .expect("initial patch must commit");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.submit(0, patch(vec![BondPropertyChangeV1::Center(Some(false))])),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);
}

#[test]
fn bond_properties_mutate_alternate_cdml_namespace_without_disturbing_opaque_content() {
    let source = concat!(
        "<c:cdml xmlns:c=\"urn:ferrum:cdml\" ",
        "xmlns:f=\"urn:foreign\"><c:molecule id=\"m\"><c:atom id=\"a\" ",
        "name=\"C\"><c:point x=\"1\" y=\"2\"/></c:atom><c:atom id=\"b\" ",
        "name=\"O\"><c:point x=\"3\" y=\"2\"/></c:atom><c:bond id=\"ab\" ",
        "start=\"a\" end=\"b\" type=\"w2\" f:keep=\"yes\" keep=\"retained\">",
        "<f:opaque f:payload=\"preserve\"/><c:unknown keep=\"also-retained\"/>",
        "</c:bond></c:molecule></c:cdml>"
    );
    let mut session = DocumentSession::load(source).expect("namespaced source must load");
    let changed = session
        .submit(0, patch(vec![BondPropertyChangeV1::Center(Some(true))]))
        .expect("namespaced bond property patch must commit");
    let projection = &changed.observation().projection().molecules()[0].bonds()[0];
    assert_eq!(projection.source_id(), Some("ab"));
    assert_eq!(projection.source_order(), 2);
    assert_eq!(projection.start().source_id(), Some("a"));
    assert_eq!(projection.end().source_id(), Some("b"));
    assert_eq!(projection.source_type(), Some("w2"));
    assert_eq!(projection.center(), Some(true));

    let cdml = changed.observation().snapshot().cdml();
    let mut tree = Xot::new();
    let document = tree.parse(cdml).expect("candidate XML must parse");
    let root = tree.document_element(document).expect("document has root");
    let molecule = tree
        .children(root)
        .find(|node| {
            element_name(&tree, *node).is_some_and(|(local, namespace)| {
                local == "molecule" && namespace == CDML_NAMESPACE
            })
        })
        .expect("canonical-namespace molecule retained");
    let bond = tree
        .children(molecule)
        .find(|node| {
            element_name(&tree, *node)
                .is_some_and(|(local, namespace)| local == "bond" && namespace == CDML_NAMESPACE)
        })
        .expect("canonical-namespace bond retained");
    let id = tree.add_name("id");
    let start = tree.add_name("start");
    let end = tree.add_name("end");
    let center = tree.add_name("center");
    let keep = tree.add_name("keep");
    assert_eq!(tree.get_attribute(bond, id), Some("ab"));
    assert_eq!(tree.get_attribute(bond, start), Some("a"));
    assert_eq!(tree.get_attribute(bond, end), Some("b"));
    assert_eq!(tree.get_attribute(bond, center), Some("yes"));
    assert_eq!(tree.get_attribute(bond, keep), Some("retained"));
    let children = tree
        .children(bond)
        .filter_map(|node| element_name(&tree, node))
        .collect::<Vec<_>>();
    assert_eq!(
        children,
        vec![
            ("opaque".to_owned(), "urn:foreign".to_owned()),
            ("unknown".to_owned(), CDML_NAMESPACE.to_owned()),
        ]
    );
    assert!(cdml.contains("urn:foreign"));
    assert!(cdml.contains("payload=\"preserve\""));
}
