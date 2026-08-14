//! Atomic durable atom-properties patch behavior.

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1, TypedDocumentError,
};
use crate::{
    AtomPropertiesPatchV1, AtomPropertiesPatchV1Error, AtomPropertyChangeV1, CDML_NAMESPACE,
    PositiveFiniteV1, Rgb24V1, VisibilityV1, element_name,
};
use xot::Xot;

const PROPERTY_SOURCE: &str = concat!(
    "<cdml xmlns:v=\"urn:vendor\"><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\" charge=\"2\" valency=\"4\" isotope=\"13\" ",
    "multiplicity=\"3\" show=\"no\" hydrogens=\"off\" vendor_keep=\"yes\">",
    "<point x=\"1\" y=\"2\"/><font family=\"Courier\" size=\"11\" ",
    "vendor_keep=\"yes\"/><ftext>keep</ftext><v:keep/></atom>",
    "</molecule><v:opaque id=\"retained\"/></cdml>"
);

fn patch(changes: Vec<AtomPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetAtomProperties {
        patch: AtomPropertiesPatchV1::new("a", changes).expect("valid atom patch"),
    })
}

#[test]
fn atom_properties_commit_once_preserve_extensions_and_follow_history() {
    let changes = vec![
        AtomPropertyChangeV1::Element("O".to_owned()),
        AtomPropertyChangeV1::FormalCharge(-1),
        AtomPropertyChangeV1::Valence(Some(2)),
        AtomPropertyChangeV1::Isotope(Some(18)),
        AtomPropertyChangeV1::Multiplicity(2),
        AtomPropertyChangeV1::Show(true),
        AtomPropertyChangeV1::ShowHydrogens(true),
        AtomPropertyChangeV1::FontSize(PositiveFiniteV1::new(15.0).unwrap()),
        AtomPropertyChangeV1::LabelColor(Rgb24V1::new("#A0B1c2").unwrap()),
    ];
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let changed = session
        .submit(0, patch(changes))
        .expect("patch must commit");
    let atom = &changed.observation().projection().molecules()[0].atoms()[0];
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(atom.element(), Some("O"));
    assert_eq!(atom.formal_charge(), Some(-1));
    assert_eq!(atom.valence(), Some(2));
    assert_eq!(atom.isotope(), Some(18));
    assert_eq!(atom.multiplicity(), Some(2));
    assert_eq!(atom.show(), Some(VisibilityV1::Enabled));
    assert_eq!(atom.hydrogens(), Some(VisibilityV1::Enabled));
    assert_eq!(atom.label_font().unwrap().size().unwrap().value(), 15.0);
    assert_eq!(
        atom.label_font().unwrap().color().unwrap().as_str(),
        "#a0b1c2"
    );
    let cdml = changed.observation().snapshot().cdml();
    assert!(cdml.contains("vendor_keep=\"yes\""));
    assert!(cdml.contains("<v:keep"));
    assert!(cdml.contains("<v:opaque"));

    let undone = session.undo(1).expect("one patch must undo once");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0].element(),
        Some("C")
    );
    let redone = session.redo(2).expect("one patch must redo once");
    assert_eq!(
        redone.observation().projection().molecules()[0].atoms()[0].element(),
        Some("O")
    );
}

#[test]
fn atom_properties_clear_optional_defaults_without_materializing_other_facts() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let changed = session
        .submit(
            0,
            patch(vec![
                AtomPropertyChangeV1::FormalCharge(0),
                AtomPropertyChangeV1::Valence(None),
                AtomPropertyChangeV1::Isotope(None),
                AtomPropertyChangeV1::Multiplicity(1),
            ]),
        )
        .expect("default intent must clear optional facts");
    let atom = &changed.observation().projection().molecules()[0].atoms()[0];
    assert_eq!(atom.formal_charge(), None);
    assert_eq!(atom.valence(), None);
    assert_eq!(atom.isotope(), None);
    assert_eq!(atom.multiplicity(), None);
    assert_eq!(atom.show(), Some(VisibilityV1::Disabled));
    assert_eq!(atom.hydrogens(), Some(VisibilityV1::Disabled));
}

#[test]
fn empty_and_equal_atom_properties_patches_are_history_free() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let empty = session.submit(0, patch(Vec::new())).expect("empty patch");
    assert_eq!(empty.observation().snapshot().revision(), 0);
    let equal = session
        .submit(
            0,
            patch(vec![AtomPropertyChangeV1::Element("C".to_owned())]),
        )
        .expect("equal property patch");
    assert_eq!(equal.observation().snapshot().revision(), 0);
}

#[test]
fn stale_atom_properties_patch_does_not_change_the_authoritative_snapshot() {
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let changed = session
        .submit(0, patch(vec![AtomPropertyChangeV1::FormalCharge(-1)]))
        .expect("initial patch must commit");
    let before = session.snapshot().expect("snapshot");

    assert!(matches!(
        session.submit(
            0,
            patch(vec![AtomPropertyChangeV1::Element("O".to_owned())])
        ),
        Err(DocumentSessionError::RevisionConflict {
            expected: 0,
            actual: 1
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);
    assert_eq!(
        session.snapshot().expect("snapshot").digest(),
        changed.observation().snapshot().digest()
    );
}

#[test]
fn atom_properties_intent_rejects_duplicates_and_invalid_scalar_meaning() {
    assert!(matches!(
        AtomPropertiesPatchV1::new(
            "a",
            vec![
                AtomPropertyChangeV1::FormalCharge(1),
                AtomPropertyChangeV1::FormalCharge(2),
            ],
        ),
        Err(AtomPropertiesPatchV1Error::DuplicateChange { .. })
    ));
    assert_eq!(
        AtomPropertiesPatchV1::new("a", vec![AtomPropertyChangeV1::Element("C<".to_owned())]),
        Err(AtomPropertiesPatchV1Error::InvalidElement)
    );
    assert_eq!(
        AtomPropertiesPatchV1::new("a", vec![AtomPropertyChangeV1::Isotope(Some(0))]),
        Err(AtomPropertiesPatchV1Error::ZeroIsotope)
    );
    assert_eq!(
        AtomPropertiesPatchV1::new("a", vec![AtomPropertyChangeV1::Multiplicity(0)]),
        Err(AtomPropertiesPatchV1Error::ZeroMultiplicity)
    );
}

#[test]
fn atom_properties_reject_unknown_atom_or_ambiguous_font_without_state_change() {
    let unknown =
        AtomPropertiesPatchV1::new("missing", vec![AtomPropertyChangeV1::FormalCharge(1)]).unwrap();
    let mut session = DocumentSession::load(PROPERTY_SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.submit(
            0,
            SessionOperation::V1(SessionOperationV1::SetAtomProperties { patch: unknown })
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownAtom(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot"), before);

    let ambiguous_source = PROPERTY_SOURCE.replace(
        "<font family=\"Courier\" size=\"11\" vendor_keep=\"yes\"/>",
        "<font size=\"11\"/><font color=\"#000000\"/>",
    );
    let mut ambiguous = DocumentSession::load(&ambiguous_source).expect("ambiguous source loads");
    let before = ambiguous.snapshot().expect("snapshot");
    assert!(matches!(
        ambiguous.submit(
            0,
            patch(vec![AtomPropertyChangeV1::FontSize(
                PositiveFiniteV1::new(12.0).unwrap()
            )])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(TypedDocumentError::AmbiguousAtomFonts(_))
        ))
    ));
    assert_eq!(ambiguous.snapshot().expect("snapshot"), before);
}

#[test]
fn atom_properties_create_a_canonical_font_without_disturbing_foreign_font_children() {
    let source = concat!(
        "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\" ",
        "xmlns:f=\"urn:foreign\"><c:molecule id=\"m\"><c:atom id=\"a\" name=\"C\">",
        "<c:point x=\"1\" y=\"2\"/><f:font foreign_keep=\"yes\"><f:opaque/>",
        "</f:font><c:ftext>retained label</c:ftext></c:atom></c:molecule></c:cdml>"
    );
    let mut session = DocumentSession::load(source).expect("namespaced source must load");
    let changed = session
        .submit(
            0,
            patch(vec![
                AtomPropertyChangeV1::FontSize(PositiveFiniteV1::new(14.0).unwrap()),
                AtomPropertyChangeV1::LabelColor(Rgb24V1::new("#123456").unwrap()),
            ]),
        )
        .expect("foreign font is not an ambiguous CDML font");
    let atom = &changed.observation().projection().molecules()[0].atoms()[0];
    assert_eq!(atom.label_font().unwrap().size().unwrap().value(), 14.0);
    assert_eq!(
        atom.label_font().unwrap().color().unwrap().as_str(),
        "#123456"
    );

    let cdml = changed.observation().snapshot().cdml();
    let mut tree = Xot::new();
    let document = tree.parse(cdml).expect("candidate XML must parse");
    let root = tree.document_element(document).expect("document has root");
    let molecule = tree
        .children(root)
        .find(|node| element_name(&tree, *node).is_some_and(|(local, _)| local == "molecule"))
        .expect("direct molecule retained");
    let atom = tree
        .children(molecule)
        .find(|node| element_name(&tree, *node).is_some_and(|(local, _)| local == "atom"))
        .expect("direct atom retained");
    let children = tree
        .children(atom)
        .filter_map(|node| element_name(&tree, node))
        .collect::<Vec<_>>();
    assert_eq!(
        children,
        vec![
            ("point".to_owned(), CDML_NAMESPACE.to_owned()),
            ("font".to_owned(), "urn:foreign".to_owned()),
            ("ftext".to_owned(), CDML_NAMESPACE.to_owned()),
            ("font".to_owned(), CDML_NAMESPACE.to_owned()),
        ]
    );
    assert!(cdml.contains("foreign_keep=\"yes\""));
    assert!(cdml.contains("opaque"));
}
