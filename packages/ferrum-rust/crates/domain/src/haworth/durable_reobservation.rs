//! Rebuild a checked direct-Haworth depiction from durable, already-decoded facts.

use std::collections::{BTreeMap, HashSet};

use ferrum_core::{RecordId, RecordKind};

use crate::haworth::{
    AuthoredDirectGlycosidicHaworthBondRoleV1, AuthoredDirectGlycosidicHaworthDepictionV1,
    DirectGlycosidicHaworthAuthoringAtomElementV1, DirectGlycosidicHaworthBondStyleV1,
    DirectGlycosidicHaworthPositionV1, HaworthError, HaworthPoint, RingForm,
};

/// One already-decoded atom in the exact durable direct-Haworth child order.
#[derive(Clone, Debug, PartialEq)]
pub struct DurableDirectGlycosidicHaworthAtomFactV1 {
    atom: RecordId,
    element: DirectGlycosidicHaworthAuthoringAtomElementV1,
    point: HaworthPoint,
    authored_child_order: u32,
}

impl DurableDirectGlycosidicHaworthAtomFactV1 {
    /// Retain one durable atom fact decoded by the document authority.
    #[must_use]
    pub const fn new(
        atom: RecordId,
        element: DirectGlycosidicHaworthAuthoringAtomElementV1,
        point: HaworthPoint,
        authored_child_order: u32,
    ) -> Self {
        Self {
            atom,
            element,
            point,
            authored_child_order,
        }
    }
}

/// One already-decoded bond in the exact durable direct-Haworth child order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableDirectGlycosidicHaworthBondFactV1 {
    bond: RecordId,
    endpoints: [RecordId; 2],
    role: AuthoredDirectGlycosidicHaworthBondRoleV1,
    token: DirectGlycosidicHaworthBondStyleV1,
    haworth_position: Option<DirectGlycosidicHaworthPositionV1>,
    authored_child_order: u32,
}

impl DurableDirectGlycosidicHaworthBondFactV1 {
    /// Retain one durable bond fact decoded by the document authority.
    #[must_use]
    pub const fn new(
        bond: RecordId,
        endpoints: [RecordId; 2],
        role: AuthoredDirectGlycosidicHaworthBondRoleV1,
        token: DirectGlycosidicHaworthBondStyleV1,
        haworth_position: Option<DirectGlycosidicHaworthPositionV1>,
        authored_child_order: u32,
    ) -> Self {
        Self {
            bond,
            endpoints,
            role,
            token,
            haworth_position,
            authored_child_order,
        }
    }
}

/// One ring's positional durable atom and bond sequences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DurableDirectGlycosidicHaworthRingFactV1 {
    ring_form: RingForm,
    atoms_in_canonical_cycle_order: Vec<RecordId>,
    bonds_in_canonical_cycle_order: Vec<RecordId>,
}

impl DurableDirectGlycosidicHaworthRingFactV1 {
    /// Retain one fixed-form ring sequence from the durable molecule children.
    #[must_use]
    pub const fn new(
        ring_form: RingForm,
        atoms_in_canonical_cycle_order: Vec<RecordId>,
        bonds_in_canonical_cycle_order: Vec<RecordId>,
    ) -> Self {
        Self {
            ring_form,
            atoms_in_canonical_cycle_order,
            bonds_in_canonical_cycle_order,
        }
    }
}

/// Complete durable evidence for the closed direct-glycosidic Haworth profile.
///
/// This native value is deliberately not a CDML type or wire format. The
/// document crate decodes durable facts before constructing it.
#[derive(Clone, Debug, PartialEq)]
pub struct DurableDirectGlycosidicHaworthProfileV1 {
    atoms_in_canonical_order: Vec<DurableDirectGlycosidicHaworthAtomFactV1>,
    bonds_in_canonical_order: Vec<DurableDirectGlycosidicHaworthBondFactV1>,
    rings: [DurableDirectGlycosidicHaworthRingFactV1; 2],
}

impl DurableDirectGlycosidicHaworthProfileV1 {
    /// Retain already-decoded facts for checked domain re-observation.
    #[must_use]
    pub const fn new(
        atoms_in_canonical_order: Vec<DurableDirectGlycosidicHaworthAtomFactV1>,
        bonds_in_canonical_order: Vec<DurableDirectGlycosidicHaworthBondFactV1>,
        rings: [DurableDirectGlycosidicHaworthRingFactV1; 2],
    ) -> Self {
        Self {
            atoms_in_canonical_order,
            bonds_in_canonical_order,
            rings,
        }
    }
}

/// Rebuild an authored depiction from the exact closed durable profile.
///
/// # Errors
///
/// Returns [`HaworthError`] when facts are not two positional five- or
/// six-member C/O rings followed by their two directed ordinary bridges.
pub fn authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1(
    profile: DurableDirectGlycosidicHaworthProfileV1,
) -> Result<AuthoredDirectGlycosidicHaworthDepictionV1, HaworthError> {
    let [first_ring, second_ring] = &profile.rings;
    let first_count = first_ring.ring_form.vertex_count();
    let second_count = second_ring.ring_form.vertex_count();
    let atom_count = first_count + second_count + 1;
    let ring_bond_count = first_count + second_count;
    if profile.atoms_in_canonical_order.len() != atom_count
        || profile.bonds_in_canonical_order.len() != ring_bond_count + 2
    {
        return Err(invalid_profile());
    }
    validate_ring_sequence(first_ring, &profile.atoms_in_canonical_order[..first_count])?;
    validate_ring_sequence(
        second_ring,
        &profile.atoms_in_canonical_order[first_count..first_count + second_count],
    )?;
    let exterior = &profile.atoms_in_canonical_order[atom_count - 1];
    if exterior.element != DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen {
        return Err(invalid_profile());
    }
    validate_atoms(&profile.atoms_in_canonical_order)?;
    validate_bond_identities(&profile.bonds_in_canonical_order, atom_count)?;

    let first_atoms = &profile.atoms_in_canonical_order[..first_count];
    let second_atoms = &profile.atoms_in_canonical_order[first_count..first_count + second_count];
    let first_bonds = &profile.bonds_in_canonical_order[..first_count];
    let second_bonds = &profile.bonds_in_canonical_order[first_count..first_count + second_count];
    validate_ring_bonds(first_ring, first_atoms, first_bonds)?;
    validate_ring_bonds(second_ring, second_atoms, second_bonds)?;
    validate_bridges(
        &profile.bonds_in_canonical_order[ring_bond_count..],
        first_atoms,
        second_atoms,
        exterior,
    )?;

    let mut canonical_atoms = Vec::new();
    canonical_atoms
        .try_reserve(profile.atoms_in_canonical_order.len())
        .map_err(|_| allocation_failure())?;
    for fact in &profile.atoms_in_canonical_order {
        canonical_atoms.push((fact.atom.clone(), fact.authored_child_order));
    }
    let mut canonical_bonds = Vec::new();
    canonical_bonds
        .try_reserve(profile.bonds_in_canonical_order.len())
        .map_err(|_| allocation_failure())?;
    for fact in &profile.bonds_in_canonical_order {
        canonical_bonds.push((
            fact.bond.clone(),
            fact.endpoints.clone(),
            fact.role,
            fact.token,
            fact.haworth_position,
            fact.authored_child_order,
        ));
    }
    let mut coordinates = BTreeMap::new();
    for fact in &profile.atoms_in_canonical_order {
        coordinates.insert(fact.atom.clone(), fact.point);
    }
    let mut ring_bonds = Vec::new();
    ring_bonds
        .try_reserve(ring_bond_count)
        .map_err(|_| allocation_failure())?;
    for fact in first_bonds.iter().chain(second_bonds) {
        let Some(haworth_position) = fact.haworth_position else {
            return Err(invalid_profile());
        };
        ring_bonds.push((
            fact.bond.clone(),
            fact.endpoints.clone(),
            fact.token,
            haworth_position,
            fact.authored_child_order,
        ));
    }
    let mut bridge_bonds = Vec::new();
    bridge_bonds
        .try_reserve(2)
        .map_err(|_| allocation_failure())?;
    for fact in &profile.bonds_in_canonical_order[ring_bond_count..] {
        bridge_bonds.push((
            fact.bond.clone(),
            fact.endpoints.clone(),
            fact.authored_child_order,
        ));
    }
    let rings = [
        (
            first_ring.ring_form,
            first_ring.bonds_in_canonical_cycle_order.clone(),
        ),
        (
            second_ring.ring_form,
            second_ring.bonds_in_canonical_cycle_order.clone(),
        ),
    ];
    AuthoredDirectGlycosidicHaworthDepictionV1::from_durable_profile(
        rings,
        coordinates,
        ring_bonds,
        bridge_bonds,
        canonical_atoms,
        canonical_bonds,
    )
}

fn validate_ring_sequence(
    ring: &DurableDirectGlycosidicHaworthRingFactV1,
    atoms: &[DurableDirectGlycosidicHaworthAtomFactV1],
) -> Result<(), HaworthError> {
    if ring.atoms_in_canonical_cycle_order.len() != ring.ring_form.vertex_count()
        || ring.bonds_in_canonical_cycle_order.len() != ring.ring_form.vertex_count()
        || ring
            .atoms_in_canonical_cycle_order
            .iter()
            .zip(atoms)
            .any(|(identity, fact)| identity != &fact.atom)
        || atoms.first().map(|fact| fact.element)
            != Some(DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen)
        || atoms[1..]
            .iter()
            .any(|fact| fact.element != DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon)
    {
        return Err(invalid_profile());
    }
    Ok(())
}

fn validate_atoms(atoms: &[DurableDirectGlycosidicHaworthAtomFactV1]) -> Result<(), HaworthError> {
    let mut identities = HashSet::new();
    identities
        .try_reserve(atoms.len())
        .map_err(|_| allocation_failure())?;
    for (index, fact) in atoms.iter().enumerate() {
        if fact.atom.kind() != RecordKind::Atom
            || !fact.point.x.is_finite()
            || !fact.point.y.is_finite()
            || fact.authored_child_order != u32::try_from(index).map_err(|_| invalid_profile())?
            || !identities.insert(fact.atom.clone())
        {
            return Err(invalid_profile());
        }
    }
    Ok(())
}

fn validate_bond_identities(
    bonds: &[DurableDirectGlycosidicHaworthBondFactV1],
    atom_count: usize,
) -> Result<(), HaworthError> {
    let mut identities = HashSet::new();
    identities
        .try_reserve(bonds.len())
        .map_err(|_| allocation_failure())?;
    for (index, fact) in bonds.iter().enumerate() {
        if fact.bond.kind() != RecordKind::Bond
            || fact.authored_child_order
                != u32::try_from(atom_count + index).map_err(|_| invalid_profile())?
            || !identities.insert(fact.bond.clone())
        {
            return Err(invalid_profile());
        }
    }
    Ok(())
}

fn validate_ring_bonds(
    ring: &DurableDirectGlycosidicHaworthRingFactV1,
    atoms: &[DurableDirectGlycosidicHaworthAtomFactV1],
    bonds: &[DurableDirectGlycosidicHaworthBondFactV1],
) -> Result<(), HaworthError> {
    let count = ring.ring_form.vertex_count();
    let q_index = bonds
        .iter()
        .position(|fact| {
            fact.role == AuthoredDirectGlycosidicHaworthBondRoleV1::Ring
                && fact.token == DirectGlycosidicHaworthBondStyleV1::Q1
                && fact.haworth_position == Some(DirectGlycosidicHaworthPositionV1::Front)
        })
        .ok_or_else(invalid_profile)?;
    for (index, fact) in bonds.iter().enumerate() {
        let previous = (q_index + count - 1) % count;
        let next = (q_index + 1) % count;
        let expected = if index == q_index {
            (
                DirectGlycosidicHaworthBondStyleV1::Q1,
                Some(DirectGlycosidicHaworthPositionV1::Front),
                [
                    atoms[index].atom.clone(),
                    atoms[(index + 1) % count].atom.clone(),
                ],
            )
        } else if index == previous {
            (
                DirectGlycosidicHaworthBondStyleV1::W1,
                Some(DirectGlycosidicHaworthPositionV1::Front),
                [atoms[index].atom.clone(), atoms[q_index].atom.clone()],
            )
        } else if index == next {
            (
                DirectGlycosidicHaworthBondStyleV1::W1,
                Some(DirectGlycosidicHaworthPositionV1::Front),
                [
                    atoms[(index + 1) % count].atom.clone(),
                    atoms[index].atom.clone(),
                ],
            )
        } else {
            (
                DirectGlycosidicHaworthBondStyleV1::N1,
                Some(DirectGlycosidicHaworthPositionV1::Back),
                [
                    atoms[index].atom.clone(),
                    atoms[(index + 1) % count].atom.clone(),
                ],
            )
        };
        if fact.role != AuthoredDirectGlycosidicHaworthBondRoleV1::Ring
            || fact.token != expected.0
            || fact.haworth_position != expected.1
            || fact.endpoints != expected.2
            || fact.bond != ring.bonds_in_canonical_cycle_order[index]
        {
            return Err(invalid_profile());
        }
    }
    Ok(())
}

fn validate_bridges(
    bridges: &[DurableDirectGlycosidicHaworthBondFactV1],
    first_atoms: &[DurableDirectGlycosidicHaworthAtomFactV1],
    second_atoms: &[DurableDirectGlycosidicHaworthAtomFactV1],
    exterior: &DurableDirectGlycosidicHaworthAtomFactV1,
) -> Result<(), HaworthError> {
    for (bridge, ring) in bridges.iter().zip([first_atoms, second_atoms]) {
        if bridge.role != AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge
            || bridge.token != DirectGlycosidicHaworthBondStyleV1::N1
            || bridge.haworth_position.is_some()
            || bridge.endpoints[1] != exterior.atom
            || !ring.iter().any(|atom| {
                atom.element == DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon
                    && atom.atom == bridge.endpoints[0]
            })
        {
            return Err(invalid_profile());
        }
    }
    Ok(())
}

fn invalid_profile() -> HaworthError {
    HaworthError::InvalidSpec("durable facts do not match the closed direct Haworth profile")
}

fn allocation_failure() -> HaworthError {
    HaworthError::ResourceExhausted
}
