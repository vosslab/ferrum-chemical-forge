use super::{
    NeutralBondCapacityAtomOutcomeV1, NeutralBondCapacityAtomV1, NeutralBondCapacityBondV1,
    NeutralBondCapacityExplicitHydrogensFactV1, NeutralBondCapacityFormalChargeFactV1,
    evaluate_neutral_bond_capacity_v1,
};

fn neutral_atom(element: &str, explicit_hydrogens: u16) -> NeutralBondCapacityAtomV1 {
    NeutralBondCapacityAtomV1 {
        source_id: None,
        element: element.to_owned(),
        explicit_hydrogens: NeutralBondCapacityExplicitHydrogensFactV1 {
            was_authored: true,
            value_or_zero: explicit_hydrogens,
        },
        formal_charge: NeutralBondCapacityFormalChargeFactV1 {
            was_authored: false,
            value_or_zero: 0,
        },
    }
}

#[test]
fn explicit_hydrogens_and_incident_orders_produce_ordered_capacity_outcomes() {
    let atoms = [
        NeutralBondCapacityAtomV1 {
            source_id: Some("within-c".to_owned()),
            ..neutral_atom("C", 3)
        },
        NeutralBondCapacityAtomV1 {
            source_id: Some("excess-c".to_owned()),
            ..neutral_atom("C", 4)
        },
        NeutralBondCapacityAtomV1 {
            source_id: Some("o1".to_owned()),
            ..neutral_atom("O", 0)
        },
    ];
    let records = evaluate_neutral_bond_capacity_v1(
        &atoms,
        &[
            NeutralBondCapacityBondV1 {
                start: 0,
                end: 2,
                order: 1,
            },
            NeutralBondCapacityBondV1 {
                start: 1,
                end: 2,
                order: 1,
            },
        ],
    )
    .expect("closed fixture evaluates");

    assert_eq!(
        records[0].outcome,
        NeutralBondCapacityAtomOutcomeV1::WithinCapacity {
            demand: 4,
            capacity: 4,
        }
    );
    assert_eq!(
        records[1].outcome,
        NeutralBondCapacityAtomOutcomeV1::ExceedsCapacity {
            demand: 5,
            capacity: 4,
        }
    );
}

#[test]
fn supported_neutral_elements_are_within_capacity_at_their_authored_demand() {
    let atoms = [
        ("H", 1),
        ("B", 3),
        ("C", 4),
        ("N", 3),
        ("O", 2),
        ("F", 1),
        ("Cl", 1),
        ("Br", 1),
        ("I", 1),
    ]
    .into_iter()
    .map(|(element, explicit_hydrogens)| neutral_atom(element, explicit_hydrogens))
    .collect::<Vec<_>>();

    let records = evaluate_neutral_bond_capacity_v1(&atoms, &[]).expect("closed table evaluates");

    assert!(records.iter().all(|record| {
        matches!(
            record.outcome,
            NeutralBondCapacityAtomOutcomeV1::WithinCapacity { demand, capacity }
                if demand == capacity
        )
    }));
}
