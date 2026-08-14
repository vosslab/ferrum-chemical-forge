use std::collections::BTreeSet;

use crate::haworth::{
    DirectGlycosidicHaworthBondStyleV1, DirectGlycosidicHaworthFragmentRequestV1,
    DirectGlycosidicHaworthPositionV1, RingForm, assemble_direct_glycosidic_haworth_fragment_v1,
    direct_glycosidic_haworth_depiction_spec_v1,
};

use super::direct_glycosidic_layout::topology;

fn fragment(reverse_request: bool) -> crate::haworth::DirectGlycosidicHaworthFragmentV1 {
    assemble_direct_glycosidic_haworth_fragment_v1(&DirectGlycosidicHaworthFragmentRequestV1 {
        topology: topology(
            RingForm::Furanose,
            RingForm::Pyranose,
            2,
            4,
            reverse_request,
        ),
        scale: 8.0,
    })
    .expect("checked fragment")
}

#[test]
fn canonical_input_variants_produce_the_same_direct_spec() {
    let forward = direct_glycosidic_haworth_depiction_spec_v1(&fragment(false)).expect("forward");
    let reversed = direct_glycosidic_haworth_depiction_spec_v1(&fragment(true)).expect("reverse");
    assert_eq!(forward, reversed);
}

#[test]
fn ring_specs_assign_exact_cdml_roles_depth_and_shoulder_direction() {
    let fragment = fragment(false);
    let spec = direct_glycosidic_haworth_depiction_spec_v1(&fragment).expect("specification");
    for ring in spec.rings() {
        let cycle = ring.bonds_in_canonical_cycle_order();
        let q_index = cycle
            .iter()
            .position(|bond| {
                spec.ring_bonds()[bond].style() == DirectGlycosidicHaworthBondStyleV1::Q1
            })
            .expect("one q1 edge");
        let q = &spec.ring_bonds()[&cycle[q_index]];
        assert_eq!(
            q.haworth_position(),
            DirectGlycosidicHaworthPositionV1::Front
        );
        assert_eq!(q.endpoints(), &fragment.ring_edges()[&cycle[q_index]]);
        for (index, bond) in cycle.iter().enumerate() {
            let record = &spec.ring_bonds()[bond];
            let canonical = &fragment.ring_edges()[bond];
            let adjacent = index == (q_index + cycle.len() - 1) % cycle.len()
                || index == (q_index + 1) % cycle.len();
            if adjacent {
                assert_eq!(
                    (record.style(), record.haworth_position()),
                    (
                        DirectGlycosidicHaworthBondStyleV1::W1,
                        DirectGlycosidicHaworthPositionV1::Front,
                    )
                );
                assert!(q.endpoints().contains(&record.endpoints()[1]));
                assert!(!q.endpoints().contains(&record.endpoints()[0]));
                assert_eq!(
                    record.endpoints().iter().collect::<BTreeSet<_>>(),
                    canonical.iter().collect::<BTreeSet<_>>(),
                );
            } else if index != q_index {
                assert_eq!(
                    (record.style(), record.haworth_position()),
                    (
                        DirectGlycosidicHaworthBondStyleV1::N1,
                        DirectGlycosidicHaworthPositionV1::Back,
                    )
                );
                assert_eq!(record.endpoints(), canonical);
            }
            assert_eq!(record.source_order(), fragment.bond_source_orders()[bond]);
        }
    }
}

#[test]
fn spec_preserves_fragment_provenance_and_keeps_bridge_bonds_plain() {
    let fragment = fragment(false);
    let spec = direct_glycosidic_haworth_depiction_spec_v1(&fragment).expect("specification");
    assert_eq!(spec.coordinates(), fragment.coordinates());
    assert_eq!(spec.atom_source_orders(), fragment.atom_source_orders());
    assert_eq!(spec.bond_source_orders(), fragment.bond_source_orders());
    assert_eq!(spec.bounds(), fragment.bounds());
    for (bond, bridge) in spec.bridge_bonds() {
        assert_eq!(bridge.endpoints(), &fragment.bridge_edges()[bond]);
        assert_eq!(bridge.source_order(), fragment.bond_source_orders()[bond]);
        assert!(!spec.ring_bonds().contains_key(bond));
    }
    let canonical: Vec<_> = spec
        .rings()
        .iter()
        .flat_map(|ring| ring.bonds_in_canonical_cycle_order().iter().cloned())
        .collect();
    let lookup_order: Vec<_> = spec.ring_bonds().keys().cloned().collect();
    assert_ne!(canonical, lookup_order);
    let expected_bonds: BTreeSet<_> = fragment
        .ring_edges()
        .keys()
        .chain(fragment.bridge_edges().keys())
        .cloned()
        .collect();
    let actual_bonds: BTreeSet<_> = spec
        .ring_bonds()
        .keys()
        .chain(spec.bridge_bonds().keys())
        .cloned()
        .collect();
    assert_eq!(actual_bonds, expected_bonds);
}
