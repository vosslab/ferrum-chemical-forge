use std::collections::BTreeMap;

use crate::haworth::{
    StandaloneDGlucoseHaworthRecipeV1, StandaloneHaworthBondTokenV1, StandaloneHaworthPositionV1,
    standalone_d_glucose_haworth_recipe_v1,
};

fn bond_for_roles(
    receipt: &crate::haworth::StandaloneDGlucoseHaworthReceiptV1,
    roles: &BTreeMap<&str, usize>,
    start: &str,
    end: &str,
) -> crate::haworth::StandaloneHaworthBondV1 {
    *receipt
        .bonds()
        .iter()
        .find(|bond| bond.start() == roles[start] && bond.end() == roles[end])
        .expect("closed recipe contains its declared directed edge")
}

#[test]
fn standalone_d_glucose_recipes_preserve_graph_faces_and_haworth_presentation() {
    for (recipe, closure_oxygen, side_chain_carbon, chain_carbon, beta) in [
        (
            StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose,
            "O5",
            "C5",
            "C6",
            false,
        ),
        (
            StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose,
            "O5",
            "C5",
            "C6",
            true,
        ),
        (
            StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucofuranose,
            "O4",
            "C4",
            "C5",
            false,
        ),
        (
            StandaloneDGlucoseHaworthRecipeV1::BetaDGlucofuranose,
            "O4",
            "C4",
            "C5",
            true,
        ),
    ] {
        let receipt = standalone_d_glucose_haworth_recipe_v1(recipe).expect("closed recipe");
        let roles = receipt
            .atoms()
            .iter()
            .enumerate()
            .map(|(index, atom)| (atom.role(), index))
            .collect::<BTreeMap<_, _>>();
        let c1 = receipt.atoms()[roles["C1"]].local();
        let o1 = receipt.atoms()[roles["O1"]].local();
        let side_chain = receipt.atoms()[roles[side_chain_carbon]].local();
        let c6 = receipt.atoms()[roles["C6"]].local();

        assert!(
            receipt
                .atoms()
                .iter()
                .all(|atom| atom.local().x.is_finite() && atom.local().y.is_finite())
                && receipt
                    .atoms()
                    .iter()
                    .filter(|atom| atom.element() == "C")
                    .count()
                    == 6
                && receipt
                    .atoms()
                    .iter()
                    .filter(|atom| atom.element() == "O")
                    .count()
                    == 6
                && receipt.bonds().len() == 12
                && bond_for_roles(&receipt, &roles, closure_oxygen, "C1").token()
                    == StandaloneHaworthBondTokenV1::N1
                && bond_for_roles(&receipt, &roles, side_chain_carbon, chain_carbon).token()
                    == StandaloneHaworthBondTokenV1::N1
                && (chain_carbon == "C6"
                    || bond_for_roles(&receipt, &roles, chain_carbon, "C6").token()
                        == StandaloneHaworthBondTokenV1::N1)
                && bond_for_roles(&receipt, &roles, "C6", "O6").token()
                    == StandaloneHaworthBondTokenV1::N1
                && receipt
                    .bonds()
                    .iter()
                    .all(|bond| bond.start() != bond.end())
        );
        assert_eq!(
            (
                bond_for_roles(&receipt, &roles, "C2", "C3").token(),
                bond_for_roles(&receipt, &roles, "C1", "C2").token(),
                bond_for_roles(&receipt, &roles, "C4", "C3").token(),
                bond_for_roles(&receipt, &roles, "C2", "C3").position(),
                bond_for_roles(&receipt, &roles, "C1", "C2").position(),
                bond_for_roles(&receipt, &roles, "C4", "C3").position(),
                o1.y < c1.y,
                c6.y < side_chain.y,
            ),
            (
                StandaloneHaworthBondTokenV1::Q1,
                StandaloneHaworthBondTokenV1::W1,
                StandaloneHaworthBondTokenV1::W1,
                Some(StandaloneHaworthPositionV1::Front),
                Some(StandaloneHaworthPositionV1::Front),
                Some(StandaloneHaworthPositionV1::Front),
                beta,
                true,
            ),
        );
    }
}
