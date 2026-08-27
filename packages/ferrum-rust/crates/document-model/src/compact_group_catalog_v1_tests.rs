use super::{CompactGroupCatalogKeyV1, materialization_recipe_v1};

fn recipe(key: CompactGroupCatalogKeyV1) -> super::CompactGroupMaterializationRecipeV1 {
    materialization_recipe_v1(key).expect("delivered catalog key has a materialization recipe")
}

fn atom<'a>(
    recipe: &'a super::CompactGroupMaterializationRecipeV1,
    role: &str,
) -> &'a super::CompactGroupRecipeAtomV1 {
    recipe
        .atoms
        .iter()
        .find(|atom| atom.role == role)
        .expect("recipe contains its chemically addressed atom")
}

fn assert_bond(
    recipe: &super::CompactGroupMaterializationRecipeV1,
    start_role: &str,
    end_role: &str,
    order: super::CompactGroupRecipeBondOrderV1,
) {
    assert!(recipe.bonds.iter().any(|bond| {
        bond.start_role == start_role
            && bond.end_role == end_role
            && bond.order == order
            && bond.presentation == super::CompactGroupRecipeBondPresentationV1::Normal
    }));
}

#[test]
fn methyl_recipe_focuses_a_neutral_attachment_carbon() {
    let methyl = recipe(CompactGroupCatalogKeyV1::Methyl);
    let carbon = atom(&methyl, methyl.attachment_atom_role);
    assert_eq!(
        (carbon.role, carbon.element, carbon.formal_charge),
        ("attachment_carbon", "C", None)
    );
}

#[test]
fn nitro_recipe_focuses_charged_nitrogen_with_charge_separated_oxygen_bonds() {
    let nitro = recipe(CompactGroupCatalogKeyV1::Nitro);
    assert_eq!(
        atom(&nitro, nitro.attachment_atom_role).formal_charge,
        Some(1)
    );
    assert_eq!(atom(&nitro, "double_oxygen").formal_charge, None);
    assert_eq!(atom(&nitro, "single_oxygen").formal_charge, Some(-1));
    assert_bond(
        &nitro,
        "attachment_nitrogen",
        "double_oxygen",
        super::CompactGroupRecipeBondOrderV1::Double,
    );
    assert_bond(
        &nitro,
        "attachment_nitrogen",
        "single_oxygen",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}

#[test]
fn ethyl_recipe_connects_the_focus_carbon_to_a_neutral_terminal_carbon() {
    let ethyl = recipe(CompactGroupCatalogKeyV1::Ethyl);
    assert_eq!(atom(&ethyl, ethyl.attachment_atom_role).element, "C");
    assert_eq!(atom(&ethyl, "terminal_carbon").formal_charge, None);
    assert_bond(
        &ethyl,
        "attachment_carbon",
        "terminal_carbon",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}

#[test]
fn methoxy_recipe_focuses_oxygen_bonded_to_methyl_carbon() {
    let methoxy = recipe(CompactGroupCatalogKeyV1::Methoxy);
    assert_eq!(atom(&methoxy, methoxy.attachment_atom_role).element, "O");
    assert_eq!(atom(&methoxy, "methyl_carbon").element, "C");
    assert_bond(
        &methoxy,
        "attachment_oxygen",
        "methyl_carbon",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}

#[test]
fn hydroxymethyl_recipe_focuses_carbon_bonded_to_neutral_hydroxyl_oxygen() {
    let hydroxymethyl = recipe(CompactGroupCatalogKeyV1::Hydroxymethyl);
    assert_eq!(
        atom(&hydroxymethyl, hydroxymethyl.attachment_atom_role).element,
        "C"
    );
    assert_eq!(
        (
            atom(&hydroxymethyl, "hydroxyl_oxygen").element,
            atom(&hydroxymethyl, "hydroxyl_oxygen").formal_charge,
        ),
        ("O", None)
    );
    assert_bond(
        &hydroxymethyl,
        "attachment_carbon",
        "hydroxyl_oxygen",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}

#[test]
fn carboxyl_recipe_focuses_carbonyl_carbon_with_neutral_oxygen_relations() {
    let carboxyl = recipe(CompactGroupCatalogKeyV1::Carboxyl);
    assert_eq!(atom(&carboxyl, carboxyl.attachment_atom_role).element, "C");
    assert_eq!(
        (
            atom(&carboxyl, "carbonyl_oxygen").formal_charge,
            atom(&carboxyl, "hydroxyl_oxygen").formal_charge,
        ),
        (None, None)
    );
    assert_bond(
        &carboxyl,
        "attachment_carbon",
        "carbonyl_oxygen",
        super::CompactGroupRecipeBondOrderV1::Double,
    );
    assert_bond(
        &carboxyl,
        "attachment_carbon",
        "hydroxyl_oxygen",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}

#[test]
fn cyano_recipe_focuses_carbon_triple_bonded_to_neutral_nitrogen() {
    let cyano = recipe(CompactGroupCatalogKeyV1::Cyano);
    assert_eq!(
        (
            atom(&cyano, cyano.attachment_atom_role).element,
            atom(&cyano, "terminal_nitrogen").element,
        ),
        ("C", "N")
    );
    assert_bond(
        &cyano,
        "attachment_carbon",
        "terminal_nitrogen",
        super::CompactGroupRecipeBondOrderV1::Triple,
    );
}

#[test]
fn acyl_chloride_recipe_focuses_carbonyl_carbon_with_chlorine_relation() {
    let acyl_chloride = recipe(CompactGroupCatalogKeyV1::AcylChloride);
    assert_eq!(
        (
            atom(&acyl_chloride, acyl_chloride.attachment_atom_role).element,
            atom(&acyl_chloride, "chlorine").element,
        ),
        ("C", "Cl")
    );
    assert_bond(
        &acyl_chloride,
        "attachment_carbon",
        "carbonyl_oxygen",
        super::CompactGroupRecipeBondOrderV1::Double,
    );
    assert_bond(
        &acyl_chloride,
        "attachment_carbon",
        "chlorine",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}

#[test]
fn phenyl_recipe_focuses_carbon_in_a_normal_alternating_ring() {
    let phenyl = recipe(CompactGroupCatalogKeyV1::Phenyl);
    assert_eq!(
        (
            atom(&phenyl, phenyl.attachment_atom_role).element,
            atom(&phenyl, "para_carbon").element,
        ),
        ("C", "C")
    );
    assert_bond(
        &phenyl,
        "attachment_carbon",
        "ortho_upper_carbon",
        super::CompactGroupRecipeBondOrderV1::Double,
    );
    assert_bond(
        &phenyl,
        "meta_upper_carbon",
        "para_carbon",
        super::CompactGroupRecipeBondOrderV1::Double,
    );
    assert_bond(
        &phenyl,
        "para_carbon",
        "meta_lower_carbon",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
    assert_bond(
        &phenyl,
        "ortho_lower_carbon",
        "attachment_carbon",
        super::CompactGroupRecipeBondOrderV1::Single,
    );
}
