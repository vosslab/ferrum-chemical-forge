use std::collections::BTreeSet;

use super::{
    CoreProjectionError, PersistentId, TypedClass, TypedDiagnosticKind, TypedDocument, TypedRecord,
    UnrecognizedNode,
};

const AUTHORED: &str = include_str!("../../../../../tests/e2e/corpus/authored_document_forms.cdml");
const LEGACY: &str = include_str!("../../../../../tests/e2e/corpus/legacy_groups_template.cdml");
const OPAQUE: &str =
    include_str!("../../../../../tests/e2e/corpus/opaque_namespace_preservation.cdml");

fn collect_classes(record: &TypedRecord, classes: &mut BTreeSet<TypedClass>) {
    classes.insert(record.class());
    for child in record.typed_children() {
        collect_classes(child.record(), classes);
    }
}

fn child(record: &TypedRecord, class: TypedClass) -> &TypedRecord {
    record
        .children_of(class)
        .next()
        .unwrap_or_else(|| panic!("missing {} child", class.name()))
}

#[test]
fn authored_corpus_exercises_every_assigned_typed_class() {
    let document = TypedDocument::parse(AUTHORED).expect("authored corpus must type");
    let mut actual = BTreeSet::new();
    collect_classes(document.root(), &mut actual);
    let expected = BTreeSet::from([
        TypedClass::Cdml,
        TypedClass::Info,
        TypedClass::AuthorProgram,
        TypedClass::Author,
        TypedClass::Note,
        TypedClass::Metadata,
        TypedClass::MetadataDocument,
        TypedClass::Standard,
        TypedClass::StandardBond,
        TypedClass::StandardArrow,
        TypedClass::StandardAtom,
        TypedClass::Paper,
        TypedClass::Viewport,
        TypedClass::Molecule,
        TypedClass::CanvasArrow,
        TypedClass::CanvasPlus,
        TypedClass::CanvasText,
        TypedClass::Rectangle,
        TypedClass::Square,
        TypedClass::Oval,
        TypedClass::Circle,
        TypedClass::Polygon,
        TypedClass::Polyline,
        TypedClass::Reaction,
        TypedClass::ReactionReactant,
        TypedClass::ReactionProduct,
        TypedClass::ReactionArrow,
        TypedClass::ReactionCondition,
        TypedClass::ReactionPlus,
        TypedClass::ExternalData,
        TypedClass::Atom,
        TypedClass::Group,
        TypedClass::MoleculeText,
        TypedClass::Query,
        TypedClass::Bond,
        TypedClass::Template,
        TypedClass::Fragment,
        TypedClass::DisplayForm,
        TypedClass::UserData,
        TypedClass::FragmentName,
        TypedClass::FragmentBond,
        TypedClass::FragmentVertex,
        TypedClass::FragmentProperty,
        TypedClass::Point,
        TypedClass::Font,
        TypedClass::FormattedText,
        TypedClass::Mark,
    ]);

    assert_eq!(actual, expected);
}

#[test]
fn unfamiliar_attributes_never_demote_recognized_records() {
    let document = TypedDocument::parse(AUTHORED).expect("authored corpus must type");
    let molecule = child(document.root(), TypedClass::Molecule);
    let atom = child(molecule, TypedClass::Atom);

    assert_eq!(atom.class(), TypedClass::Atom);
    assert_eq!(atom.attribute("name"), Some("C"));
    assert_eq!(atom.unknown_attributes().len(), 1);
    assert_eq!(
        atom.unknown_attributes()[0].qualified_name(),
        "local_extension"
    );
    assert_eq!(atom.unknown_attributes()[0].value(), "literal");

    let serialized = document.to_xml().expect("typed tree must serialize");
    let reparsed = TypedDocument::parse(&serialized).expect("typed output must reparse");
    let reparsed_atom = child(
        child(reparsed.root(), TypedClass::Molecule),
        TypedClass::Atom,
    );
    assert_eq!(reparsed_atom.unknown_attributes()[0].value(), "literal");
}

#[test]
fn foreign_subtrees_remain_wholly_opaque() {
    let opaque = TypedDocument::parse(OPAQUE).expect("opaque corpus must type");
    let foreign = opaque
        .root()
        .unrecognized_children()
        .iter()
        .find_map(|child| match child.node() {
            UnrecognizedNode::Element { name, xml }
                if name.namespace() == "urn:vendor" && name.local_name() == "extension" =>
            {
                Some(xml)
            }
            _ => None,
        })
        .expect("foreign subtree must remain opaque");
    assert!(foreign.contains("q:widget"));
    assert!(foreign.contains("vendor-child"));
}

#[test]
fn preservation_only_containers_have_typed_identity_and_opaque_payloads() {
    let authored = TypedDocument::parse(AUTHORED).expect("authored corpus must type");
    let root = authored.root();
    let molecule = child(root, TypedClass::Molecule);

    let display_child = molecule
        .typed_children()
        .iter()
        .find(|child| child.record().class() == TypedClass::DisplayForm)
        .expect("display-form must have a typed container position");
    let user_child = molecule
        .typed_children()
        .iter()
        .find(|child| child.record().class() == TypedClass::UserData)
        .expect("user-data must have a typed container position");
    assert!(display_child.position() < user_child.position());

    for class in [TypedClass::DisplayForm, TypedClass::UserData] {
        let container = child(molecule, class);
        assert_eq!(container.class(), class);
        assert!(!container.path().components().is_empty());
        assert!(container.typed_attributes().is_empty());
        assert!(container.typed_children().is_empty());
        assert_eq!(container.unrecognized_children().len(), 1);
        assert!(matches!(
            container.unrecognized_children()[0].node(),
            UnrecognizedNode::Element { .. }
        ));
    }

    let display = child(molecule, TypedClass::DisplayForm);
    let user_data = child(molecule, TypedClass::UserData);
    assert_ne!(display.path(), user_data.path());

    let external = child(root, TypedClass::ExternalData);
    assert!(external.typed_attributes().is_empty());
    assert_eq!(external.unknown_attributes().len(), 1);
    assert_eq!(external.unknown_attributes()[0].qualified_name(), "id");
    assert_eq!(external.unknown_attributes()[0].value(), "external-opaque");
    assert!(external.typed_children().is_empty());
    assert_eq!(external.unrecognized_children().len(), 1);
    let identifier = PersistentId::new("external-opaque").expect("fixture ID must be valid");
    assert!(authored.indexed().resolve_id(&identifier).is_some());
}

#[test]
fn excess_child_is_retained_with_a_non_demoting_diagnostic() {
    let source = "<cdml><molecule><atom id=\"a\"><point x=\"1\" y=\"2\"/><point x=\"3\" y=\"4\"/></atom></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("diagnostic source must type");
    let atom = child(
        child(document.root(), TypedClass::Molecule),
        TypedClass::Atom,
    );

    assert_eq!(atom.class(), TypedClass::Atom);
    assert_eq!(atom.children_of(TypedClass::Point).count(), 1);
    assert_eq!(atom.unrecognized_children().len(), 1);
    assert_eq!(atom.diagnostics().len(), 1);
    assert_eq!(
        atom.diagnostics()[0].kind(),
        TypedDiagnosticKind::ExcessChild
    );
    assert_eq!(atom.diagnostics()[0].child_class(), TypedClass::Point);
    let serialized = document.to_xml().expect("diagnostic tree must serialize");
    assert_eq!(serialized.matches("<point").count(), 2);
}

#[test]
fn typed_documents_supply_the_validated_core_projection() {
    let authored = TypedDocument::parse(AUTHORED).expect("authored corpus must type");
    let projection = authored
        .core_projection()
        .expect("authored molecule must project");
    assert_eq!(projection.document_version(), Some("26.07"));
    assert_eq!(projection.molecules().len(), 1);
    assert_eq!(projection.molecules()[0].atoms().len(), 1);
    assert_eq!(projection.molecules()[0].bonds().len(), 3);

    let legacy = TypedDocument::parse(LEGACY).expect("legacy corpus must type");
    let projection = legacy
        .core_projection()
        .expect("legacy molecule must project");
    assert_eq!(projection.document_version(), Some("0.8"));
    assert_eq!(projection.molecules()[0].atoms().len(), 3);
    assert_eq!(projection.molecules()[0].bonds().len(), 2);
}

#[test]
fn core_projection_reports_missing_required_geometry() {
    let source = "<cdml><molecule><atom id=\"a\"/></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("source must type");
    let error = document
        .core_projection()
        .expect_err("an atom without a point cannot project");
    assert!(matches!(
        error,
        CoreProjectionError::MissingField { field: "point", .. }
    ));
}

#[test]
fn core_projection_reports_invalid_authored_scalars() {
    let source = "<cdml><molecule><atom id=\"a\" charge=\"many\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("source must type");
    let error = document
        .core_projection()
        .expect_err("a nonnumeric charge cannot project");
    assert!(matches!(
        error,
        CoreProjectionError::InvalidValue {
            field: "charge",
            value,
            ..
        } if value == "many"
    ));
}

#[test]
fn core_projection_reports_unresolved_bond_endpoints() {
    let source = "<cdml><molecule><atom id=\"a\"><point x=\"0\" y=\"0\"/></atom><bond start=\"a\" end=\"missing\"/></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("source must type");
    let error = document
        .core_projection()
        .expect_err("an unresolved endpoint cannot project");
    assert!(matches!(
        error,
        CoreProjectionError::UnknownVertex {
            field: "end",
            identifier,
            ..
        } if identifier == "missing"
    ));
}

#[test]
fn core_projection_reports_core_model_rejections() {
    let source =
        "<cdml><molecule><atom id=\"a\"><point x=\"NaN\" y=\"0\"/></atom></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("source must type");
    let error = document
        .core_projection()
        .expect_err("nonfinite geometry cannot project");
    assert!(matches!(error, CoreProjectionError::Model { .. }));
}
