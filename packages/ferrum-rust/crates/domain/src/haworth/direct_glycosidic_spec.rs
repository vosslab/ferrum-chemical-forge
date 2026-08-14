//! Durable, renderer-neutral Haworth drawing facts for a checked direct profile.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::RecordId;

use crate::haworth::{
    BondDepiction, DirectGlycosidicHaworthFragmentV1, HaworthError, HaworthPoint, RingForm,
};

const INVALID_RING_POLICY: &str =
    "direct depiction spec must contain one q, two w, and remaining n bonds per ring";

/// The closed CDML single-bond depiction type for one direct Haworth ring edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectGlycosidicHaworthBondStyleV1 {
    /// The `q1` Haworth front-edge bond type.
    Q1,
    /// The directed `w1` Haworth shoulder bond type.
    W1,
    /// The `n1` ordinary back-edge bond type.
    N1,
}

/// The closed Haworth depth value for a direct ring edge.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectGlycosidicHaworthPositionV1 {
    /// The ring edge is on the visible Haworth face.
    Front,
    /// The ring edge is behind the visible Haworth face.
    Back,
}

/// One selected ring bond with durable direct-Haworth depiction semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicHaworthRingBondSpecV1 {
    bond: RecordId,
    endpoints: [RecordId; 2],
    style: DirectGlycosidicHaworthBondStyleV1,
    haworth_position: DirectGlycosidicHaworthPositionV1,
    source_order: usize,
}

impl DirectGlycosidicHaworthRingBondSpecV1 {
    /// Return the durable bond identity.
    #[must_use]
    pub const fn bond(&self) -> &RecordId {
        &self.bond
    }

    /// Return ordered endpoint identities for this durable depiction.
    #[must_use]
    pub const fn endpoints(&self) -> &[RecordId; 2] {
        &self.endpoints
    }

    /// Return the closed CDML `q1`, `w1`, or `n1` type.
    #[must_use]
    pub const fn style(&self) -> DirectGlycosidicHaworthBondStyleV1 {
        self.style
    }

    /// Return the durable Haworth front or back position.
    #[must_use]
    pub const fn haworth_position(&self) -> DirectGlycosidicHaworthPositionV1 {
        self.haworth_position
    }

    /// Return the copied snapshot-local molecule-record source position.
    #[must_use]
    pub const fn source_order(&self) -> usize {
        self.source_order
    }
}

/// One selected ordinary bridge bond, deliberately without Haworth style or depth.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicHaworthBridgeBondSpecV1 {
    bond: RecordId,
    endpoints: [RecordId; 2],
    source_order: usize,
}

impl DirectGlycosidicHaworthBridgeBondSpecV1 {
    /// Return the durable bridge-bond identity.
    #[must_use]
    pub const fn bond(&self) -> &RecordId {
        &self.bond
    }

    /// Return the retained `[ring carbon, exterior oxygen]` endpoints.
    #[must_use]
    pub const fn endpoints(&self) -> &[RecordId; 2] {
        &self.endpoints
    }

    /// Return the copied snapshot-local molecule-record source position.
    #[must_use]
    pub const fn source_order(&self) -> usize {
        self.source_order
    }
}

/// One canonical ring's form and cyclic bond-role sequence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicHaworthRingSpecV1 {
    ring_form: RingForm,
    bonds_in_canonical_cycle_order: Vec<RecordId>,
}

impl DirectGlycosidicHaworthRingSpecV1 {
    /// Return the canonical ring form.
    #[must_use]
    pub const fn ring_form(&self) -> RingForm {
        self.ring_form
    }

    /// Return ring bonds in their canonical cycle order, not source or paint order.
    #[must_use]
    pub fn bonds_in_canonical_cycle_order(&self) -> &[RecordId] {
        &self.bonds_in_canonical_cycle_order
    }
}

/// Owned direct-glycosidic depiction facts copied from one checked fragment.
///
/// Source orders remain snapshot-local molecule-record facts. They never define
/// map iteration, document child order, or renderer paint order.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthDepictionSpecV1 {
    rings: [DirectGlycosidicHaworthRingSpecV1; 2],
    coordinates: BTreeMap<RecordId, HaworthPoint>,
    ring_bonds: BTreeMap<RecordId, DirectGlycosidicHaworthRingBondSpecV1>,
    bridge_bonds: BTreeMap<RecordId, DirectGlycosidicHaworthBridgeBondSpecV1>,
    atom_source_orders: BTreeMap<RecordId, usize>,
    bond_source_orders: BTreeMap<RecordId, usize>,
    bounds: [HaworthPoint; 2],
}

impl DirectGlycosidicHaworthDepictionSpecV1 {
    /// Return canonical rings in topology order.
    #[must_use]
    pub const fn rings(&self) -> &[DirectGlycosidicHaworthRingSpecV1; 2] {
        &self.rings
    }

    /// Return copied coordinates keyed by selected atom identity.
    #[must_use]
    pub fn coordinates(&self) -> &BTreeMap<RecordId, HaworthPoint> {
        &self.coordinates
    }

    /// Return selected ring-bond depiction facts keyed by bond identity.
    #[must_use]
    pub fn ring_bonds(&self) -> &BTreeMap<RecordId, DirectGlycosidicHaworthRingBondSpecV1> {
        &self.ring_bonds
    }

    /// Return selected ordinary bridge-bond facts keyed by bond identity.
    #[must_use]
    pub fn bridge_bonds(&self) -> &BTreeMap<RecordId, DirectGlycosidicHaworthBridgeBondSpecV1> {
        &self.bridge_bonds
    }

    /// Return copied snapshot-local source positions for selected atoms.
    #[must_use]
    pub fn atom_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.atom_source_orders
    }

    /// Return copied snapshot-local source positions for selected bonds.
    #[must_use]
    pub fn bond_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.bond_source_orders
    }

    /// Return finite bounds copied from the checked fragment.
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }
}

/// Convert a checked fragment into owned direct-Haworth depiction facts.
///
/// # Errors
///
/// Returns [`HaworthError::InvalidSpec`] if direct role translation cannot
/// establish the required q1/w1/n1 convention or shoulder direction.
pub fn direct_glycosidic_haworth_depiction_spec_v1(
    fragment: &DirectGlycosidicHaworthFragmentV1,
) -> Result<DirectGlycosidicHaworthDepictionSpecV1, HaworthError> {
    let rings =
        fragment
            .topology()
            .rings()
            .each_ref()
            .map(|ring| DirectGlycosidicHaworthRingSpecV1 {
                ring_form: ring.topology().ring_form(),
                bonds_in_canonical_cycle_order: ring.topology().bond_ids().to_vec(),
            });
    let mut ring_bonds = BTreeMap::new();
    for ring in fragment.topology().rings() {
        let cycle = ring.topology().bond_ids();
        let q_bond = cycle.iter().find(|bond| {
            matches!(
                fragment.ring_depictions().get(*bond),
                Some(BondDepiction::HaworthFront {
                    edge_role: crate::haworth::WedgeEdgeRole::Center,
                    face: crate::haworth::Face::Front,
                })
            )
        });
        let q_bond = q_bond.ok_or(HaworthError::InvalidSpec(
            "direct depiction spec has inconsistent ring depiction semantics",
        ))?;
        let q_endpoints = fragment
            .ring_edges()
            .get(q_bond)
            .ok_or(HaworthError::InvalidSpec(
                "direct depiction spec is missing selected ring bond facts",
            ))?;
        for bond in cycle {
            let depiction =
                fragment
                    .ring_depictions()
                    .get(bond)
                    .ok_or(HaworthError::InvalidSpec(
                        "direct depiction spec is missing selected ring bond facts",
                    ))?;
            let canonical_endpoints =
                fragment
                    .ring_edges()
                    .get(bond)
                    .ok_or(HaworthError::InvalidSpec(
                        "direct depiction spec is missing selected ring bond facts",
                    ))?;
            let source_order =
                *fragment
                    .bond_source_orders()
                    .get(bond)
                    .ok_or(HaworthError::InvalidSpec(
                        "direct depiction spec is missing selected ring bond facts",
                    ))?;
            let (style, haworth_position, endpoints) = match depiction {
                BondDepiction::HaworthFront {
                    edge_role: crate::haworth::WedgeEdgeRole::Center,
                    face: crate::haworth::Face::Front,
                } => (
                    DirectGlycosidicHaworthBondStyleV1::Q1,
                    DirectGlycosidicHaworthPositionV1::Front,
                    canonical_endpoints.clone(),
                ),
                BondDepiction::HaworthFront {
                    edge_role:
                        crate::haworth::WedgeEdgeRole::LeftShoulder
                        | crate::haworth::WedgeEdgeRole::RightShoulder,
                    face: crate::haworth::Face::Front,
                } => (
                    DirectGlycosidicHaworthBondStyleV1::W1,
                    DirectGlycosidicHaworthPositionV1::Front,
                    shoulder_endpoints(canonical_endpoints, q_endpoints)?,
                ),
                BondDepiction::Back {
                    face: crate::haworth::Face::Back,
                } => (
                    DirectGlycosidicHaworthBondStyleV1::N1,
                    DirectGlycosidicHaworthPositionV1::Back,
                    canonical_endpoints.clone(),
                ),
                _ => {
                    return Err(HaworthError::InvalidSpec(
                        "direct depiction spec has inconsistent ring depiction semantics",
                    ));
                }
            };
            if ring_bonds
                .insert(
                    bond.clone(),
                    DirectGlycosidicHaworthRingBondSpecV1 {
                        bond: bond.clone(),
                        endpoints,
                        style,
                        haworth_position,
                        source_order,
                    },
                )
                .is_some()
            {
                return Err(HaworthError::InvalidSpec(
                    "direct depiction spec has inconsistent ring depiction semantics",
                ));
            }
        }
    }
    let mut bridge_bonds = BTreeMap::new();
    for bond in fragment.topology().bridge().bonds() {
        let endpoints = fragment
            .bridge_edges()
            .get(bond)
            .ok_or(HaworthError::InvalidSpec(
                "direct depiction spec has inconsistent bridge bond facts",
            ))?;
        let source_order =
            *fragment
                .bond_source_orders()
                .get(bond)
                .ok_or(HaworthError::InvalidSpec(
                    "direct depiction spec has inconsistent bridge bond facts",
                ))?;
        if bridge_bonds
            .insert(
                bond.clone(),
                DirectGlycosidicHaworthBridgeBondSpecV1 {
                    bond: bond.clone(),
                    endpoints: endpoints.clone(),
                    source_order,
                },
            )
            .is_some()
        {
            return Err(HaworthError::InvalidSpec(
                "direct depiction spec has inconsistent bridge bond facts",
            ));
        }
    }
    let spec = DirectGlycosidicHaworthDepictionSpecV1 {
        rings,
        coordinates: fragment.coordinates().clone(),
        ring_bonds,
        bridge_bonds,
        atom_source_orders: fragment.atom_source_orders().clone(),
        bond_source_orders: fragment.bond_source_orders().clone(),
        bounds: fragment.bounds(),
    };
    validate_spec(&spec, fragment)?;
    Ok(spec)
}

fn shoulder_endpoints(
    endpoints: &[RecordId; 2],
    q_endpoints: &[RecordId; 2],
) -> Result<[RecordId; 2], HaworthError> {
    let shared: Vec<_> = endpoints
        .iter()
        .filter(|endpoint| q_endpoints.contains(endpoint))
        .cloned()
        .collect();
    if shared.len() != 1 {
        return Err(HaworthError::InvalidSpec(
            "direct depiction spec has inconsistent Haworth front-edge adjacency",
        ));
    }
    let outer = endpoints
        .iter()
        .find(|endpoint| *endpoint != &shared[0])
        .cloned()
        .ok_or(HaworthError::InvalidSpec(
            "direct depiction spec has inconsistent Haworth front-edge adjacency",
        ))?;
    Ok([outer, shared[0].clone()])
}

fn validate_spec(
    spec: &DirectGlycosidicHaworthDepictionSpecV1,
    fragment: &DirectGlycosidicHaworthFragmentV1,
) -> Result<(), HaworthError> {
    let selected_ring_bonds: BTreeSet<_> = fragment.ring_edges().keys().cloned().collect();
    if spec.ring_bonds.keys().cloned().collect::<BTreeSet<_>>() != selected_ring_bonds {
        return Err(HaworthError::InvalidSpec(
            "direct depiction spec is missing selected ring bond facts",
        ));
    }
    for (ring, ring_spec) in fragment.topology().rings().iter().zip(spec.rings.iter()) {
        let cycle = ring.topology().bond_ids();
        if ring_spec.ring_form != ring.topology().ring_form()
            || ring_spec.bonds_in_canonical_cycle_order != cycle
        {
            return Err(HaworthError::InvalidSpec(
                "direct depiction spec has inconsistent ring depiction semantics",
            ));
        }
        let mut q = None;
        let mut shoulders = Vec::new();
        for bond in cycle {
            let record = spec.ring_bonds.get(bond).ok_or(HaworthError::InvalidSpec(
                "direct depiction spec is missing selected ring bond facts",
            ))?;
            let canonical = fragment
                .ring_edges()
                .get(bond)
                .ok_or(HaworthError::InvalidSpec(
                    "direct depiction spec is missing selected ring bond facts",
                ))?;
            let depiction =
                fragment
                    .ring_depictions()
                    .get(bond)
                    .ok_or(HaworthError::InvalidSpec(
                        "direct depiction spec is missing selected ring bond facts",
                    ))?;
            match (depiction, record.style, record.haworth_position) {
                (
                    BondDepiction::HaworthFront {
                        edge_role: crate::haworth::WedgeEdgeRole::Center,
                        face: crate::haworth::Face::Front,
                    },
                    DirectGlycosidicHaworthBondStyleV1::Q1,
                    DirectGlycosidicHaworthPositionV1::Front,
                ) => {
                    if record.endpoints != *canonical || q.replace((bond, record)).is_some() {
                        return Err(HaworthError::InvalidSpec(INVALID_RING_POLICY));
                    }
                }
                (
                    BondDepiction::HaworthFront {
                        edge_role:
                            crate::haworth::WedgeEdgeRole::LeftShoulder
                            | crate::haworth::WedgeEdgeRole::RightShoulder,
                        face: crate::haworth::Face::Front,
                    },
                    DirectGlycosidicHaworthBondStyleV1::W1,
                    DirectGlycosidicHaworthPositionV1::Front,
                ) => {
                    shoulders.push((bond, record, canonical));
                }
                (
                    BondDepiction::Back {
                        face: crate::haworth::Face::Back,
                    },
                    DirectGlycosidicHaworthBondStyleV1::N1,
                    DirectGlycosidicHaworthPositionV1::Back,
                ) => {
                    if record.endpoints != *canonical {
                        return Err(HaworthError::InvalidSpec(INVALID_RING_POLICY));
                    }
                }
                _ => {
                    return Err(HaworthError::InvalidSpec(INVALID_RING_POLICY));
                }
            }
        }
        let Some((q_bond, q_record)) = q else {
            return Err(HaworthError::InvalidSpec(INVALID_RING_POLICY));
        };
        if shoulders.len() != 2
            || !cycle_adjacent(cycle, q_bond, shoulders[0].0)
            || !cycle_adjacent(cycle, q_bond, shoulders[1].0)
            || shoulders[0].0 == shoulders[1].0
        {
            return Err(HaworthError::InvalidSpec(INVALID_RING_POLICY));
        }
        let mut shared_endpoints = BTreeSet::new();
        for (_, shoulder, canonical) in shoulders {
            let expected = shoulder_endpoints(canonical, &q_record.endpoints)?;
            if shoulder.endpoints != expected
                || !shared_endpoints.insert(shoulder.endpoints[1].clone())
            {
                return Err(HaworthError::InvalidSpec(
                    "direct depiction spec has inconsistent Haworth front-edge adjacency",
                ));
            }
        }
    }
    Ok(())
}

fn cycle_adjacent(cycle: &[RecordId], first: &RecordId, second: &RecordId) -> bool {
    let Some(index) = cycle.iter().position(|bond| bond == first) else {
        return false;
    };
    let previous = if index == 0 {
        cycle.len() - 1
    } else {
        index - 1
    };
    cycle[previous] == *second || cycle[(index + 1) % cycle.len()] == *second
}
