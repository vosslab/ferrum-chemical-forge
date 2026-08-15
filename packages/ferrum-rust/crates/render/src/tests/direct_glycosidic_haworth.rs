use std::collections::{BTreeMap, BTreeSet};

use crate::direct_glycosidic_haworth::DirectGlycosidicHaworthDrawOpV1;
use crate::*;
use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
use ferrum_domain::haworth::{
    DirectGlycosidicHaworthFragmentRequestV1, DirectGlycosidicHaworthTopologyV1,
    HaworthTopologyBuilder, HaworthVertex, RingForm,
    assemble_direct_glycosidic_haworth_fragment_v1, direct_glycosidic_haworth_depiction_spec_v1,
};

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("a{index}")).expect("id")),
        Some(element.to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("position"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("atom")
}
fn bond(index: usize, first: &Atom, second: &Atom) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("b{index}")).expect("id")),
        VertexRef::Atom(first.identity().clone()),
        VertexRef::Atom(second.identity().clone()),
        None,
        Some(BondOrder::Single),
        None,
        Some(false),
        None,
    )
    .expect("bond")
}
fn spec(scale: f64) -> ferrum_domain::haworth::DirectGlycosidicHaworthDepictionSpecV1 {
    let atoms: Vec<_> = (0..12)
        .map(|index| atom(index, if index == 0 || index == 6 { "O" } else { "C" }))
        .chain(std::iter::once(atom(12, "O")))
        .collect();
    let bonds: Vec<_> = (0..6)
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % 6]))
        .chain((0..6).map(|index| bond(index + 6, &atoms[index + 6], &atoms[6 + (index + 1) % 6])))
        .chain([
            bond(12, &atoms[1], &atoms[12]),
            bond(13, &atoms[7], &atoms[12]),
        ])
        .collect();
    let molecule = Molecule::new(
        Some(Identifier::new("direct").expect("id")),
        None,
        atoms.clone(),
        vec![],
        vec![],
        vec![],
        bonds.clone(),
        None,
    )
    .expect("molecule");
    let ring = |offset: usize| {
        HaworthTopologyBuilder::new(
            RingForm::Pyranose,
            atoms[offset + 1].identity().clone(),
            atoms[offset..offset + 6]
                .iter()
                .map(|atom| HaworthVertex {
                    atom: atom.identity().clone(),
                })
                .collect::<Vec<_>>(),
        )
        .build(&molecule)
        .expect("ring")
    };
    let topology = DirectGlycosidicHaworthTopologyV1::classify(
        &molecule,
        [ring(0), ring(6)],
        atoms[12].identity().clone(),
        [bonds[12].identity().clone(), bonds[13].identity().clone()],
    )
    .expect("topology");
    let fragment =
        assemble_direct_glycosidic_haworth_fragment_v1(&DirectGlycosidicHaworthFragmentRequestV1 {
            topology,
            scale,
        })
        .expect("fragment");
    direct_glycosidic_haworth_depiction_spec_v1(&fragment).expect("spec")
}
fn request_with(scale: f64) -> DirectGlycosidicHaworthRenderRequestV1 {
    DirectGlycosidicHaworthRenderRequestV1::new(
        RenderProvenance::new(RenderRevision::new(0).expect("revision"), [8; 32]),
        spec(scale),
        Paint::rgb24(Rgb24::new("102030").expect("paint")),
        PositiveFinite::new(scale / 8.0).expect("line"),
        PositiveFinite::new(scale / 2.0).expect("wedge"),
    )
}
pub(crate) fn request() -> DirectGlycosidicHaworthRenderRequestV1 {
    request_with(8.0)
}

#[test]
fn direct_profile_partitions_closed_targets_and_uses_semantic_tiers() {
    let input = request();
    let plan = lower_direct_glycosidic_haworth_v1(&input).expect("profile");
    let expected: BTreeSet<_> = input
        .spec()
        .ring_bonds()
        .keys()
        .chain(input.spec().bridge_bonds().keys())
        .cloned()
        .collect();
    let mut tiers = BTreeMap::new();
    for operation in plan.operations() {
        let tier = match operation {
            DirectGlycosidicHaworthDrawOpV1::OrdinaryLine { .. } => "ordinary",
            DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke { .. } => "front stroke",
            DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge { .. } => "front wedge",
        };
        assert!(tiers.insert(operation.bond().clone(), tier).is_none());
    }
    assert_eq!(tiers.keys().cloned().collect::<BTreeSet<_>>(), expected);
    for bond in input.spec().bridge_bonds().keys() {
        assert_eq!(tiers.get(bond), Some(&"ordinary"));
    }
    for (bond, fact) in input.spec().ring_bonds() {
        let expected_tier = match fact.style() {
            ferrum_domain::haworth::DirectGlycosidicHaworthBondStyleV1::N1 => "ordinary",
            ferrum_domain::haworth::DirectGlycosidicHaworthBondStyleV1::Q1 => "front stroke",
            ferrum_domain::haworth::DirectGlycosidicHaworthBondStyleV1::W1 => "front wedge",
        };
        assert_eq!(tiers.get(bond), Some(&expected_tier));
    }
}
