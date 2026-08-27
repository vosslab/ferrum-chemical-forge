use crate::{DirectCdmlSemanticIndexV1, TypedDocument};

fn strict_reaction_document() -> TypedDocument {
    TypedDocument::parse(concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\">",
        "<molecule id=\"left\"><atom id=\"left-atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"right\"><atom id=\"right-atom\" name=\"O\"><point x=\"1\" y=\"0\"/></atom></molecule>",
        "<arrow id=\"arrow\"/><reaction id=\"reaction\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"arrow\"/></reaction>",
        "</cdml>"
    ))
    .expect("fixture admits")
}

#[test]
fn durable_binding_refuses_corrupted_reaction_identity() {
    let mut document = strict_reaction_document();
    document.corrupt_direct_document_object_id_for_test("reaction");
    let semantic = DirectCdmlSemanticIndexV1::from_document(&document);

    assert!(semantic.bind_durable_reactions_v1(&document).is_err());
}

#[test]
fn durable_binding_refuses_corrupted_strict_member_identity() {
    let mut document = strict_reaction_document();
    document.corrupt_direct_document_object_id_for_test("left");
    let semantic = DirectCdmlSemanticIndexV1::from_document(&document);

    assert!(semantic.bind_durable_reactions_v1(&document).is_err());
}
