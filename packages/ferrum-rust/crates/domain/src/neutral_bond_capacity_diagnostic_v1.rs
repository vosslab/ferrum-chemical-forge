//! Closed neutral bond-capacity arithmetic for ordinary authored molecules.

use thiserror::Error;

/// One atom admitted to the V1 neutral capacity calculation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralBondCapacityAtomV1 {
    /// Durable authored atom identifier when the source supplied one.
    pub source_id: Option<String>,
    /// Canonical source element spelling.
    pub element: String,
    /// Retained explicit-hydrogen source fact used by bounded demand arithmetic.
    pub explicit_hydrogens: NeutralBondCapacityExplicitHydrogensFactV1,
    /// Retained authored charge fact; V1 only admits its zero-valued forms.
    pub formal_charge: NeutralBondCapacityFormalChargeFactV1,
}

/// Authored formal-charge presence and its neutral-defaulted calculation value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NeutralBondCapacityFormalChargeFactV1 {
    /// Whether CDML explicitly supplied this source fact.
    pub was_authored: bool,
    /// Authored value, or zero only when the fact was absent.
    pub value_or_zero: i32,
}

/// Authored explicit-hydrogen presence and its calculation value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NeutralBondCapacityExplicitHydrogensFactV1 {
    /// Whether CDML explicitly supplied this source fact.
    pub was_authored: bool,
    /// Authored value, or zero only when the fact was absent.
    pub value_or_zero: u16,
}

/// One already-admitted incident bond demand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NeutralBondCapacityBondV1 {
    /// First atom index in source order.
    pub start: usize,
    /// Second atom index in source order.
    pub end: usize,
    /// Closed bond-order demand: one, two, or three.
    pub order: u8,
}

/// A supported atom's bounded neutral-capacity result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NeutralBondCapacityAtomOutcomeV1 {
    /// Total explicit-H plus incident bond-order demand fits the V1 table.
    WithinCapacity { demand: u16, capacity: u16 },
    /// Total explicit-H plus incident bond-order demand exceeds the V1 table.
    ExceedsCapacity { demand: u16, capacity: u16 },
}

/// One source-order atom result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NeutralBondCapacityAtomRecordV1 {
    /// Durable authored atom identifier when available.
    pub source_id: Option<String>,
    /// Canonical V1 table element.
    pub element: String,
    /// Retained explicit-hydrogen source fact.
    pub explicit_hydrogens: NeutralBondCapacityExplicitHydrogensFactV1,
    /// Retained formal-charge source fact.
    pub formal_charge: NeutralBondCapacityFormalChargeFactV1,
    /// Rust-owned neutral capacity result.
    pub outcome: NeutralBondCapacityAtomOutcomeV1,
}

/// Arithmetic failures after a document adapter has admitted the closed grammar.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum NeutralBondCapacityErrorV1 {
    /// An adapter supplied an index outside its atom input.
    #[error("bond capacity input contains an out-of-range atom endpoint")]
    EndpointOutOfRange,
    /// An adapter supplied a bond demand outside the closed grammar.
    #[error("bond capacity input contains an unsupported bond order")]
    UnsupportedBondOrder,
    /// Exact source demand cannot fit the V1 receipt integer.
    #[error("bond capacity demand exceeds V1 receipt arithmetic")]
    DemandOverflow,
    /// An admitted atom was not in the finite neutral capacity table.
    #[error("bond capacity input contains an unsupported element")]
    UnsupportedElement,
}

/// Evaluate already-admitted neutral ordinary atoms without mutating source facts.
pub fn evaluate_neutral_bond_capacity_v1(
    atoms: &[NeutralBondCapacityAtomV1],
    bonds: &[NeutralBondCapacityBondV1],
) -> Result<Vec<NeutralBondCapacityAtomRecordV1>, NeutralBondCapacityErrorV1> {
    let mut demands = Vec::new();
    demands
        .try_reserve_exact(atoms.len())
        .map_err(|_| NeutralBondCapacityErrorV1::DemandOverflow)?;
    for atom in atoms {
        demands.push(atom.explicit_hydrogens.value_or_zero);
    }
    for bond in bonds {
        if !matches!(bond.order, 1..=3) {
            return Err(NeutralBondCapacityErrorV1::UnsupportedBondOrder);
        }
        for endpoint in [bond.start, bond.end] {
            let demand = demands
                .get_mut(endpoint)
                .ok_or(NeutralBondCapacityErrorV1::EndpointOutOfRange)?;
            *demand = demand
                .checked_add(u16::from(bond.order))
                .ok_or(NeutralBondCapacityErrorV1::DemandOverflow)?;
        }
    }
    let mut records = Vec::new();
    records
        .try_reserve_exact(atoms.len())
        .map_err(|_| NeutralBondCapacityErrorV1::DemandOverflow)?;
    for (atom, demand) in atoms.iter().zip(demands) {
        let capacity = neutral_capacity(atom.element.as_str())
            .ok_or(NeutralBondCapacityErrorV1::UnsupportedElement)?;
        let outcome = if demand > capacity {
            NeutralBondCapacityAtomOutcomeV1::ExceedsCapacity { demand, capacity }
        } else {
            NeutralBondCapacityAtomOutcomeV1::WithinCapacity { demand, capacity }
        };
        records.push(NeutralBondCapacityAtomRecordV1 {
            source_id: atom.source_id.clone(),
            element: atom.element.clone(),
            explicit_hydrogens: atom.explicit_hydrogens,
            formal_charge: atom.formal_charge,
            outcome,
        });
    }
    Ok(records)
}

fn neutral_capacity(element: &str) -> Option<u16> {
    match element {
        "H" | "F" | "Cl" | "Br" | "I" => Some(1),
        "O" => Some(2),
        "B" | "N" => Some(3),
        "C" => Some(4),
        _ => None,
    }
}

#[cfg(test)]
mod neutral_bond_capacity_diagnostic_v1_tests;
