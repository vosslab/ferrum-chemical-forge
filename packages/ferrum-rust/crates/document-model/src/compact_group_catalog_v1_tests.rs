use super::{
    CompactGroupCatalogKeyV1, attached_compact_group_authoring_keys_v1, is_admitted_atom_symbol_v1,
    materialization_recipe_v1, supports_attached_compact_group_authoring_v1,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn attached_authoring_capability_is_a_closed_borrowed_catalog_slice() {
    let keys: &'static [CompactGroupCatalogKeyV1] = attached_compact_group_authoring_keys_v1();
    assert!(keys.iter().all(|&key| {
        supports_attached_compact_group_authoring_v1(key)
            && materialization_recipe_v1(key).is_some()
    }));
}

#[test]
fn catalog_identity_binds_label_and_attachment_sites() {
    let methyl = CompactGroupCatalogKeyV1::parse("methyl").expect("known key");
    assert_eq!(methyl.label(), "Me");
    assert_eq!(CompactGroupCatalogKeyV1::from_label("Me"), Some(methyl));
    assert!(methyl.supports_attachment_index(0));
    assert!(!methyl.supports_attachment_index(1));
    assert_eq!(CompactGroupCatalogKeyV1::from_label("C"), None);
}

#[test]
fn carboxyl_identity_has_its_persisted_key_and_label() {
    let carboxyl = CompactGroupCatalogKeyV1::parse("carboxyl").expect("known key");
    assert_eq!(carboxyl.label(), "COOH");
    assert_eq!(CompactGroupCatalogKeyV1::from_label("COOH"), Some(carboxyl));
}

#[test]
fn acyl_chloride_identity_has_its_persisted_key_and_label() {
    let acyl_chloride = CompactGroupCatalogKeyV1::parse("acyl_chloride").expect("known key");
    assert_eq!(acyl_chloride.label(), "COCl");
    assert_eq!(
        CompactGroupCatalogKeyV1::from_label("COCl"),
        Some(acyl_chloride)
    );
}

#[test]
fn atom_symbol_grammar_has_exact_ascii_boundaries() {
    for symbol in ["C", "Cl", "Uuo"] {
        assert!(is_admitted_atom_symbol_v1(symbol), "{symbol}");
    }
    for symbol in ["", "c", "ABC", "abc", "Clll", "C1"] {
        assert!(!is_admitted_atom_symbol_v1(symbol), "{symbol}");
    }
}

#[test]
fn closed_materialization_recipes_do_not_interpret_labels() {
    let methyl =
        materialization_recipe_v1(CompactGroupCatalogKeyV1::Methyl).expect("methyl recipe");
    assert_eq!(methyl.attachment_atom_role, "attachment_carbon");
    let nitro = materialization_recipe_v1(CompactGroupCatalogKeyV1::Nitro).expect("nitro recipe");
    assert_eq!(nitro.attachment_atom_role, "attachment_nitrogen");
    let ethyl = materialization_recipe_v1(CompactGroupCatalogKeyV1::Ethyl).expect("ethyl recipe");
    assert_eq!(ethyl.attachment_atom_role, "attachment_carbon");
    let methoxy =
        materialization_recipe_v1(CompactGroupCatalogKeyV1::Methoxy).expect("methoxy recipe");
    assert_eq!(methoxy.attachment_atom_role, "attachment_oxygen");
    let hydroxymethyl = materialization_recipe_v1(CompactGroupCatalogKeyV1::Hydroxymethyl)
        .expect("hydroxymethyl recipe");
    assert_eq!(hydroxymethyl.attachment_atom_role, "attachment_carbon");
    let acyl_chloride = materialization_recipe_v1(CompactGroupCatalogKeyV1::AcylChloride)
        .expect("acyl chloride recipe");
    assert_eq!(acyl_chloride.attachment_atom_role, "attachment_carbon");
    assert!(materialization_recipe_v1(CompactGroupCatalogKeyV1::Phenyl).is_none());
}

#[test]
fn ethyl_recipe_encodes_neutral_linear_carbon_topology() {
    let ethyl = materialization_recipe_v1(CompactGroupCatalogKeyV1::Ethyl).expect("ethyl recipe");
    assert_eq!(ethyl.atoms.len(), 2);
    assert_eq!(ethyl.atoms[0].role, "attachment_carbon");
    assert_eq!(ethyl.atoms[0].element, "C");
    assert_eq!(ethyl.atoms[0].formal_charge, None);
    assert_eq!((ethyl.atoms[0].x, ethyl.atoms[0].y), (0.0, 0.0));
    assert_eq!(ethyl.atoms[1].role, "terminal_carbon");
    assert_eq!(ethyl.atoms[1].element, "C");
    assert_eq!(ethyl.atoms[1].formal_charge, None);
    assert_eq!((ethyl.atoms[1].x, ethyl.atoms[1].y), (24.0, 0.0));
    assert_eq!(ethyl.bonds.len(), 1);
    assert_eq!(ethyl.bonds[0].start_role, "attachment_carbon");
    assert_eq!(ethyl.bonds[0].end_role, "terminal_carbon");
    assert_eq!(
        ethyl.bonds[0].order,
        super::CompactGroupRecipeBondOrderV1::Single
    );
    assert_eq!(
        ethyl.bonds[0].presentation,
        super::CompactGroupRecipeBondPresentationV1::Normal
    );
}

#[test]
fn methoxy_recipe_encodes_neutral_oxygen_attached_topology() {
    let methoxy =
        materialization_recipe_v1(CompactGroupCatalogKeyV1::Methoxy).expect("methoxy recipe");
    assert_eq!(methoxy.attachment_atom_role, "attachment_oxygen");
    assert_eq!(methoxy.atoms.len(), 2);
    assert_eq!(methoxy.atoms[0].role, "attachment_oxygen");
    assert_eq!(methoxy.atoms[0].element, "O");
    assert_eq!(methoxy.atoms[0].formal_charge, None);
    assert_eq!((methoxy.atoms[0].x, methoxy.atoms[0].y), (0.0, 0.0));
    assert_eq!(methoxy.atoms[1].role, "methyl_carbon");
    assert_eq!(methoxy.atoms[1].element, "C");
    assert_eq!(methoxy.atoms[1].formal_charge, None);
    assert_eq!((methoxy.atoms[1].x, methoxy.atoms[1].y), (24.0, 0.0));
    assert_eq!(methoxy.bonds.len(), 1);
    assert_eq!(methoxy.bonds[0].start_role, "attachment_oxygen");
    assert_eq!(methoxy.bonds[0].end_role, "methyl_carbon");
    assert_eq!(
        methoxy.bonds[0].order,
        super::CompactGroupRecipeBondOrderV1::Single
    );
    assert_eq!(
        methoxy.bonds[0].presentation,
        super::CompactGroupRecipeBondPresentationV1::Normal
    );
}

#[test]
fn hydroxymethyl_recipe_encodes_neutral_carbon_attached_topology() {
    let hydroxymethyl = materialization_recipe_v1(CompactGroupCatalogKeyV1::Hydroxymethyl)
        .expect("hydroxymethyl recipe");
    assert_eq!(hydroxymethyl.attachment_atom_role, "attachment_carbon");
    assert_eq!(
        (
            hydroxymethyl.atoms[0].role,
            hydroxymethyl.atoms[0].element,
            hydroxymethyl.atoms[0].formal_charge,
        ),
        ("attachment_carbon", "C", None)
    );
    assert_eq!(
        (
            hydroxymethyl.atoms[1].role,
            hydroxymethyl.atoms[1].element,
            hydroxymethyl.atoms[1].formal_charge,
        ),
        ("hydroxyl_oxygen", "O", None)
    );
    assert_eq!(
        (
            hydroxymethyl.bonds[0].start_role,
            hydroxymethyl.bonds[0].end_role,
            hydroxymethyl.bonds[0].order,
            hydroxymethyl.bonds[0].presentation,
        ),
        (
            "attachment_carbon",
            "hydroxyl_oxygen",
            super::CompactGroupRecipeBondOrderV1::Single,
            super::CompactGroupRecipeBondPresentationV1::Normal,
        )
    );
}

#[test]
fn carboxyl_recipe_encodes_neutral_carbonyl_and_hydroxyl_topology() {
    let carboxyl =
        materialization_recipe_v1(CompactGroupCatalogKeyV1::Carboxyl).expect("carboxyl recipe");
    assert_eq!(carboxyl.attachment_atom_role, "attachment_carbon");
    assert_eq!(
        carboxyl.atoms,
        [
            super::CompactGroupRecipeAtomV1 {
                role: "attachment_carbon",
                element: "C",
                formal_charge: None,
                x: 0.0,
                y: 0.0,
            },
            super::CompactGroupRecipeAtomV1 {
                role: "carbonyl_oxygen",
                element: "O",
                formal_charge: None,
                x: 24.0,
                y: 18.0,
            },
            super::CompactGroupRecipeAtomV1 {
                role: "hydroxyl_oxygen",
                element: "O",
                formal_charge: None,
                x: 24.0,
                y: -18.0,
            },
        ]
    );
    assert_eq!(
        carboxyl.bonds,
        [
            super::CompactGroupRecipeBondV1 {
                start_role: "attachment_carbon",
                end_role: "carbonyl_oxygen",
                order: super::CompactGroupRecipeBondOrderV1::Double,
                presentation: super::CompactGroupRecipeBondPresentationV1::Normal,
            },
            super::CompactGroupRecipeBondV1 {
                start_role: "attachment_carbon",
                end_role: "hydroxyl_oxygen",
                order: super::CompactGroupRecipeBondOrderV1::Single,
                presentation: super::CompactGroupRecipeBondPresentationV1::Normal,
            },
        ]
    );
}

#[test]
fn acyl_chloride_recipe_encodes_neutral_carbonyl_and_chlorine_topology() {
    let acyl_chloride = materialization_recipe_v1(CompactGroupCatalogKeyV1::AcylChloride)
        .expect("acyl chloride recipe");
    assert_eq!(acyl_chloride.attachment_atom_role, "attachment_carbon");

    let attachment_carbon = acyl_chloride
        .atoms
        .iter()
        .find(|atom| atom.role == acyl_chloride.attachment_atom_role)
        .expect("acyl chloride attachment-carbon focus");
    assert_eq!(
        (attachment_carbon.element, attachment_carbon.formal_charge),
        ("C", None)
    );
    assert_eq!((attachment_carbon.x, attachment_carbon.y), (0.0, 0.0));

    let carbonyl_oxygen = acyl_chloride
        .atoms
        .iter()
        .find(|atom| atom.role == "carbonyl_oxygen")
        .expect("acyl chloride carbonyl-oxygen role");
    assert_eq!(
        (carbonyl_oxygen.element, carbonyl_oxygen.formal_charge),
        ("O", None)
    );
    assert_eq!((carbonyl_oxygen.x, carbonyl_oxygen.y), (24.0, 18.0));

    let chlorine = acyl_chloride
        .atoms
        .iter()
        .find(|atom| atom.role == "chlorine")
        .expect("acyl chloride chlorine role");
    assert_eq!((chlorine.element, chlorine.formal_charge), ("Cl", None));
    assert_eq!((chlorine.x, chlorine.y), (24.0, -18.0));

    let carbon_oxygen = acyl_chloride
        .bonds
        .iter()
        .find(|bond| {
            bond.start_role == attachment_carbon.role && bond.end_role == carbonyl_oxygen.role
        })
        .expect("acyl chloride C=O semantic endpoints");
    assert_eq!(
        (carbon_oxygen.order, carbon_oxygen.presentation),
        (
            super::CompactGroupRecipeBondOrderV1::Double,
            super::CompactGroupRecipeBondPresentationV1::Normal,
        )
    );

    let carbon_chlorine = acyl_chloride
        .bonds
        .iter()
        .find(|bond| bond.start_role == attachment_carbon.role && bond.end_role == chlorine.role)
        .expect("acyl chloride C-Cl semantic endpoints");
    assert_eq!(
        (carbon_chlorine.order, carbon_chlorine.presentation),
        (
            super::CompactGroupRecipeBondOrderV1::Single,
            super::CompactGroupRecipeBondPresentationV1::Normal,
        )
    );
}

#[test]
fn cyano_recipe_encodes_neutral_carbon_nitrogen_triple_topology() {
    let cyano = materialization_recipe_v1(CompactGroupCatalogKeyV1::Cyano).expect("cyano recipe");
    assert_eq!(cyano.attachment_atom_role, "attachment_carbon");
    let attachment_carbon = cyano
        .atoms
        .iter()
        .find(|atom| atom.role == "attachment_carbon")
        .expect("cyano attachment carbon");
    let terminal_nitrogen = cyano
        .atoms
        .iter()
        .find(|atom| atom.role == "terminal_nitrogen")
        .expect("cyano terminal nitrogen");
    assert_eq!(
        (
            attachment_carbon.element,
            attachment_carbon.formal_charge,
            terminal_nitrogen.element,
            terminal_nitrogen.formal_charge,
        ),
        ("C", None, "N", None)
    );
    let carbon_nitrogen = cyano
        .bonds
        .iter()
        .find(|bond| bond.start_role == "attachment_carbon" && bond.end_role == "terminal_nitrogen")
        .expect("cyano carbon-nitrogen bond");
    assert_eq!(
        (carbon_nitrogen.order, carbon_nitrogen.presentation),
        (
            super::CompactGroupRecipeBondOrderV1::Triple,
            super::CompactGroupRecipeBondPresentationV1::Normal,
        )
    );
}

#[test]
fn nitro_recipe_encodes_its_closed_formal_charge_topology() {
    let nitro = materialization_recipe_v1(CompactGroupCatalogKeyV1::Nitro).expect("nitro recipe");
    assert_eq!(nitro.atoms.len(), 3);
    assert_eq!(nitro.atoms[0].role, "attachment_nitrogen");
    assert_eq!(nitro.atoms[0].formal_charge, Some(1));
    assert_eq!(nitro.atoms[1].role, "double_oxygen");
    assert_eq!(nitro.atoms[1].formal_charge, None);
    assert_eq!(nitro.atoms[2].role, "single_oxygen");
    assert_eq!(nitro.atoms[2].formal_charge, Some(-1));
    assert_eq!(nitro.bonds.len(), 2);
    assert_eq!(
        nitro.bonds[0].order,
        super::CompactGroupRecipeBondOrderV1::Double
    );
    assert_eq!(
        nitro.bonds[1].order,
        super::CompactGroupRecipeBondOrderV1::Single
    );
}

#[test]
fn phenyl_recipe_is_the_closed_neutral_kekule_cycle() {
    let phenyl =
        materialization_recipe_v1(CompactGroupCatalogKeyV1::Phenyl).expect("phenyl recipe");
    assert_eq!(CompactGroupCatalogKeyV1::Phenyl.as_str(), "phenyl");
    assert_eq!(CompactGroupCatalogKeyV1::Phenyl.label(), "Ph");
    assert!(CompactGroupCatalogKeyV1::Phenyl.supports_attachment_index(0));
    assert!(!CompactGroupCatalogKeyV1::Phenyl.supports_attachment_index(1));
    assert_eq!(phenyl.attachment_atom_role, "attachment_carbon");
    let atoms: BTreeMap<_, _> = phenyl
        .atoms
        .iter()
        .map(|atom| {
            (
                atom.role,
                (atom.element, atom.formal_charge, atom.x, atom.y),
            )
        })
        .collect();
    assert_eq!(
        atoms,
        BTreeMap::from([
            ("attachment_carbon", ("C", None, 0.0, 0.0)),
            ("ortho_upper_carbon", ("C", None, 12.0, -20.784609690826528)),
            ("meta_upper_carbon", ("C", None, 36.0, -20.784609690826528)),
            ("para_carbon", ("C", None, 48.0, 0.0)),
            ("meta_lower_carbon", ("C", None, 36.0, 20.784609690826528)),
            ("ortho_lower_carbon", ("C", None, 12.0, 20.784609690826528)),
        ])
    );
    let bonds: BTreeMap<_, _> = phenyl
        .bonds
        .iter()
        .map(|bond| {
            (
                (bond.start_role, bond.end_role),
                (bond.order, bond.presentation),
            )
        })
        .collect();
    assert_eq!(
        bonds,
        BTreeMap::from([
            (
                ("attachment_carbon", "ortho_upper_carbon"),
                (
                    super::CompactGroupRecipeBondOrderV1::Double,
                    super::CompactGroupRecipeBondPresentationV1::Normal
                )
            ),
            (
                ("ortho_upper_carbon", "meta_upper_carbon"),
                (
                    super::CompactGroupRecipeBondOrderV1::Single,
                    super::CompactGroupRecipeBondPresentationV1::Normal
                )
            ),
            (
                ("meta_upper_carbon", "para_carbon"),
                (
                    super::CompactGroupRecipeBondOrderV1::Double,
                    super::CompactGroupRecipeBondPresentationV1::Normal
                )
            ),
            (
                ("para_carbon", "meta_lower_carbon"),
                (
                    super::CompactGroupRecipeBondOrderV1::Single,
                    super::CompactGroupRecipeBondPresentationV1::Normal
                )
            ),
            (
                ("meta_lower_carbon", "ortho_lower_carbon"),
                (
                    super::CompactGroupRecipeBondOrderV1::Double,
                    super::CompactGroupRecipeBondPresentationV1::Normal
                )
            ),
            (
                ("ortho_lower_carbon", "attachment_carbon"),
                (
                    super::CompactGroupRecipeBondOrderV1::Single,
                    super::CompactGroupRecipeBondPresentationV1::Normal
                )
            ),
        ])
    );
    let roles: BTreeSet<_> = atoms.keys().copied().collect();
    let endpoints: BTreeSet<_> = bonds
        .keys()
        .flat_map(|(start, end)| [*start, *end])
        .collect();
    assert_eq!(endpoints, roles);
}
