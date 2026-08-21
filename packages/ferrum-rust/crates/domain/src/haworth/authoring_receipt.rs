//! Closed-source authoring facts for one direct-glycosidic Haworth molecule.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::{Molecule, RecordId};

use crate::haworth::{
    DirectGlycosidicHaworthBondStyleV1, DirectGlycosidicHaworthDepictionSpecV1,
    DirectGlycosidicHaworthFragmentRequestV1, DirectGlycosidicHaworthPositionV1,
    DirectGlycosidicHaworthTopologyV1, HaworthError, HaworthPoint,
    assemble_direct_glycosidic_haworth_fragment_v1, direct_glycosidic_haworth_depiction_spec_v1,
};

type DurableRingBondComponentsV1 = (
    RecordId,
    [RecordId; 2],
    DirectGlycosidicHaworthBondStyleV1,
    DirectGlycosidicHaworthPositionV1,
    u32,
);
type DurableCanonicalBondComponentsV1 = (
    RecordId,
    [RecordId; 2],
    AuthoredDirectGlycosidicHaworthBondRoleV1,
    DirectGlycosidicHaworthBondStyleV1,
    Option<DirectGlycosidicHaworthPositionV1>,
    u32,
);

/// The only atom elements authored by the closed direct Haworth profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectGlycosidicHaworthAuthoringAtomElementV1 {
    /// Carbon.
    Carbon,
    /// Oxygen.
    Oxygen,
}

fn bounds(points: impl Iterator<Item = HaworthPoint>) -> Result<[HaworthPoint; 2], HaworthError> {
    let mut points = points;
    let first = points.next().ok_or(HaworthError::InvalidSpec(
        "durable Haworth receipt has no coordinates",
    ))?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Ok([
        HaworthPoint { x: min_x, y: min_y },
        HaworthPoint { x: max_x, y: max_y },
    ])
}

/// One source atom selected into the detached authoring receipt.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthSelectedAtomFactV1 {
    source_atom_identity: RecordId,
    element: DirectGlycosidicHaworthAuthoringAtomElementV1,
    local: HaworthPoint,
}

impl DirectGlycosidicHaworthSelectedAtomFactV1 {
    /// Return the source identity used only for later durable-ID allocation.
    #[must_use]
    pub const fn source_atom_identity(&self) -> &RecordId {
        &self.source_atom_identity
    }

    /// Return the exact selected C/O element retained while authority was live.
    #[must_use]
    pub const fn element(&self) -> DirectGlycosidicHaworthAuthoringAtomElementV1 {
        self.element
    }

    /// Return the receipt-local Haworth coordinate.
    #[must_use]
    pub const fn local(&self) -> HaworthPoint {
        self.local
    }
}

/// One source bond selected into the detached authoring receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicHaworthSelectedBondFactV1 {
    source_bond_identity: RecordId,
    endpoints: [RecordId; 2],
    token: DirectGlycosidicHaworthBondStyleV1,
    haworth_position: Option<DirectGlycosidicHaworthPositionV1>,
}

impl DirectGlycosidicHaworthSelectedBondFactV1 {
    /// Return the source bond identity used only for later durable-ID allocation.
    #[must_use]
    pub const fn source_bond_identity(&self) -> &RecordId {
        &self.source_bond_identity
    }

    /// Return the canonical ordered endpoints.
    #[must_use]
    pub const fn endpoints(&self) -> &[RecordId; 2] {
        &self.endpoints
    }

    /// Return the closed `q1`, `w1`, or `n1` token.
    #[must_use]
    pub const fn token(&self) -> DirectGlycosidicHaworthBondStyleV1 {
        self.token
    }

    /// Return front/back only for ring edges.
    #[must_use]
    pub const fn haworth_position(&self) -> Option<DirectGlycosidicHaworthPositionV1> {
        self.haworth_position
    }
}

/// Immutable, detached facts for authoring one closed direct Haworth molecule.
///
/// This receipt deliberately omits source coordinates and all optional source
/// chemistry. Its private fields make the checked factory its normal boundary.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthAuthoringReceiptV1 {
    atoms_in_canonical_order: Vec<DirectGlycosidicHaworthSelectedAtomFactV1>,
    bonds_in_canonical_order: Vec<DirectGlycosidicHaworthSelectedBondFactV1>,
    bounds: [HaworthPoint; 2],
    local_scale: f64,
    source_spec: DirectGlycosidicHaworthDepictionSpecV1,
}

/// Checked direct-Haworth depiction rebound to newly authored durable records.
#[derive(Clone, Debug, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthDepictionV1 {
    rings: [AuthoredDirectGlycosidicHaworthRingV1; 2],
    coordinates: std::collections::BTreeMap<RecordId, HaworthPoint>,
    ring_bonds: std::collections::BTreeMap<RecordId, AuthoredDirectGlycosidicHaworthRingBondV1>,
    bridge_bonds: std::collections::BTreeMap<RecordId, AuthoredDirectGlycosidicHaworthBridgeBondV1>,
    canonical_atoms: Vec<AuthoredDirectGlycosidicHaworthCanonicalAtomV1>,
    canonical_bonds: Vec<AuthoredDirectGlycosidicHaworthCanonicalBondV1>,
    bounds: [HaworthPoint; 2],
}

/// One durable atom retained in the checked authoring canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthCanonicalAtomV1 {
    atom: RecordId,
    authored_child_order: u32,
}

/// The direct-Haworth role of one durable bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthoredDirectGlycosidicHaworthBondRoleV1 {
    /// A ring-cycle bond with an explicit Haworth depth.
    Ring,
    /// One of the two ordinary exterior-oxygen bridge bonds.
    Bridge,
}

/// One durable bond retained in the checked authoring canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthCanonicalBondV1 {
    bond: RecordId,
    endpoints: [RecordId; 2],
    role: AuthoredDirectGlycosidicHaworthBondRoleV1,
    token: DirectGlycosidicHaworthBondStyleV1,
    haworth_position: Option<DirectGlycosidicHaworthPositionV1>,
    authored_child_order: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthRingV1 {
    ring_form: crate::haworth::RingForm,
    bonds_in_canonical_cycle_order: Vec<RecordId>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthRingBondV1 {
    bond: RecordId,
    endpoints: [RecordId; 2],
    style: DirectGlycosidicHaworthBondStyleV1,
    haworth_position: DirectGlycosidicHaworthPositionV1,
    authored_child_order: u32,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredDirectGlycosidicHaworthBridgeBondV1 {
    bond: RecordId,
    endpoints: [RecordId; 2],
    authored_child_order: u32,
}

impl AuthoredDirectGlycosidicHaworthDepictionV1 {
    pub(super) fn from_durable_profile(
        rings: [(crate::haworth::RingForm, Vec<RecordId>); 2],
        coordinates: BTreeMap<RecordId, HaworthPoint>,
        ring_bonds: Vec<DurableRingBondComponentsV1>,
        bridge_bonds: Vec<(RecordId, [RecordId; 2], u32)>,
        canonical_atoms: Vec<(RecordId, u32)>,
        canonical_bonds: Vec<DurableCanonicalBondComponentsV1>,
    ) -> Result<Self, HaworthError> {
        let bounds = bounds(coordinates.values().copied())?;
        let rings = rings.map(|(ring_form, bonds_in_canonical_cycle_order)| {
            AuthoredDirectGlycosidicHaworthRingV1 {
                ring_form,
                bonds_in_canonical_cycle_order,
            }
        });
        let ring_bonds = ring_bonds
            .into_iter()
            .map(
                |(bond, endpoints, style, haworth_position, authored_child_order)| {
                    (
                        bond.clone(),
                        AuthoredDirectGlycosidicHaworthRingBondV1 {
                            bond,
                            endpoints,
                            style,
                            haworth_position,
                            authored_child_order,
                        },
                    )
                },
            )
            .collect();
        let bridge_bonds = bridge_bonds
            .into_iter()
            .map(|(bond, endpoints, authored_child_order)| {
                (
                    bond.clone(),
                    AuthoredDirectGlycosidicHaworthBridgeBondV1 {
                        bond,
                        endpoints,
                        authored_child_order,
                    },
                )
            })
            .collect();
        let canonical_atoms = canonical_atoms
            .into_iter()
            .map(
                |(atom, authored_child_order)| AuthoredDirectGlycosidicHaworthCanonicalAtomV1 {
                    atom,
                    authored_child_order,
                },
            )
            .collect();
        let canonical_bonds = canonical_bonds
            .into_iter()
            .map(
                |(bond, endpoints, role, token, haworth_position, authored_child_order)| {
                    AuthoredDirectGlycosidicHaworthCanonicalBondV1 {
                        bond,
                        endpoints,
                        role,
                        token,
                        haworth_position,
                        authored_child_order,
                    }
                },
            )
            .collect();
        Ok(Self {
            rings,
            coordinates,
            ring_bonds,
            bridge_bonds,
            canonical_atoms,
            canonical_bonds,
            bounds,
        })
    }

    #[must_use]
    pub const fn rings(&self) -> &[AuthoredDirectGlycosidicHaworthRingV1; 2] {
        &self.rings
    }
    #[must_use]
    pub fn coordinates(&self) -> &std::collections::BTreeMap<RecordId, HaworthPoint> {
        &self.coordinates
    }
    #[must_use]
    pub fn ring_bonds(
        &self,
    ) -> &std::collections::BTreeMap<RecordId, AuthoredDirectGlycosidicHaworthRingBondV1> {
        &self.ring_bonds
    }
    #[must_use]
    pub fn bridge_bonds(
        &self,
    ) -> &std::collections::BTreeMap<RecordId, AuthoredDirectGlycosidicHaworthBridgeBondV1> {
        &self.bridge_bonds
    }
    /// Return durable atoms in the original checked canonical authoring order.
    ///
    /// This sequence is retained at rebind time and is never recovered from the
    /// identity-keyed coordinate map.
    #[must_use]
    pub fn canonical_atoms(&self) -> &[AuthoredDirectGlycosidicHaworthCanonicalAtomV1] {
        &self.canonical_atoms
    }
    /// Return durable bonds in the original checked canonical authoring order.
    ///
    /// This sequence is retained at rebind time and is never recovered from the
    /// identity-keyed ring or bridge maps.
    #[must_use]
    pub fn canonical_bonds(&self) -> &[AuthoredDirectGlycosidicHaworthCanonicalBondV1] {
        &self.canonical_bonds
    }
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }
}
impl AuthoredDirectGlycosidicHaworthCanonicalAtomV1 {
    /// Return the durable atom identity.
    #[must_use]
    pub const fn atom(&self) -> &RecordId {
        &self.atom
    }
    /// Return the authored molecule-child position.
    #[must_use]
    pub const fn authored_child_order(&self) -> u32 {
        self.authored_child_order
    }
}
impl AuthoredDirectGlycosidicHaworthCanonicalBondV1 {
    /// Return the durable bond identity.
    #[must_use]
    pub const fn bond(&self) -> &RecordId {
        &self.bond
    }
    /// Return the exact directed durable endpoints.
    #[must_use]
    pub const fn endpoints(&self) -> &[RecordId; 2] {
        &self.endpoints
    }
    /// Return whether this is a ring or bridge bond.
    #[must_use]
    pub const fn role(&self) -> AuthoredDirectGlycosidicHaworthBondRoleV1 {
        self.role
    }
    /// Return the closed `q1`, `w1`, or `n1` token.
    #[must_use]
    pub const fn token(&self) -> DirectGlycosidicHaworthBondStyleV1 {
        self.token
    }
    /// Return front/back for ring bonds, or `None` for bridge bonds.
    #[must_use]
    pub const fn haworth_position(&self) -> Option<DirectGlycosidicHaworthPositionV1> {
        self.haworth_position
    }
    /// Return the authored molecule-child position.
    #[must_use]
    pub const fn authored_child_order(&self) -> u32 {
        self.authored_child_order
    }
}
impl AuthoredDirectGlycosidicHaworthRingV1 {
    #[must_use]
    pub const fn ring_form(&self) -> crate::haworth::RingForm {
        self.ring_form
    }
    #[must_use]
    pub fn bonds_in_canonical_cycle_order(&self) -> &[RecordId] {
        &self.bonds_in_canonical_cycle_order
    }
}
impl AuthoredDirectGlycosidicHaworthRingBondV1 {
    #[must_use]
    pub const fn bond(&self) -> &RecordId {
        &self.bond
    }
    #[must_use]
    pub const fn endpoints(&self) -> &[RecordId; 2] {
        &self.endpoints
    }
    #[must_use]
    pub const fn style(&self) -> DirectGlycosidicHaworthBondStyleV1 {
        self.style
    }
    #[must_use]
    pub const fn haworth_position(&self) -> DirectGlycosidicHaworthPositionV1 {
        self.haworth_position
    }
    #[must_use]
    pub const fn authored_child_order(&self) -> u32 {
        self.authored_child_order
    }
}
impl AuthoredDirectGlycosidicHaworthBridgeBondV1 {
    #[must_use]
    pub const fn bond(&self) -> &RecordId {
        &self.bond
    }
    #[must_use]
    pub const fn endpoints(&self) -> &[RecordId; 2] {
        &self.endpoints
    }
    #[must_use]
    pub const fn authored_child_order(&self) -> u32 {
        self.authored_child_order
    }
}

impl DirectGlycosidicHaworthAuthoringReceiptV1 {
    /// Return the checked source depiction used for detached rendering.
    #[must_use]
    pub const fn source_spec(&self) -> &DirectGlycosidicHaworthDepictionSpecV1 {
        &self.source_spec
    }

    /// Rebind this checked A1 receipt to exactly parallel durable document identities.
    pub fn authored_depiction_for_durable_commit_v1(
        &self,
        atoms: &[RecordId],
        bonds: &[RecordId],
        translation: HaworthPoint,
    ) -> Result<AuthoredDirectGlycosidicHaworthDepictionV1, HaworthError> {
        if !translation.x.is_finite()
            || !translation.y.is_finite()
            || atoms.len() != self.atoms_in_canonical_order.len()
            || bonds.len() != self.bonds_in_canonical_order.len()
        {
            return Err(HaworthError::InvalidSpec(
                "durable Haworth rebind inputs do not match the checked receipt",
            ));
        }
        if atoms
            .iter()
            .any(|id| id.kind() != ferrum_core::RecordKind::Atom)
            || bonds
                .iter()
                .any(|id| id.kind() != ferrum_core::RecordKind::Bond)
            || atoms.iter().collect::<BTreeSet<_>>().len() != atoms.len()
            || bonds.iter().collect::<BTreeSet<_>>().len() != bonds.len()
        {
            return Err(HaworthError::InvalidSpec(
                "durable Haworth rebind identities must be unique typed atom and bond records",
            ));
        }
        let atom_map: BTreeMap<_, _> = self
            .atoms_in_canonical_order
            .iter()
            .zip(atoms)
            .map(|(source, durable)| (source.source_atom_identity().clone(), durable.clone()))
            .collect();
        let bond_map: BTreeMap<_, _> = self
            .bonds_in_canonical_order
            .iter()
            .zip(bonds)
            .map(|(source, durable)| (source.source_bond_identity().clone(), durable.clone()))
            .collect();
        if atom_map.len() != atoms.len() || bond_map.len() != bonds.len() {
            return Err(HaworthError::InvalidSpec(
                "durable Haworth receipt canonical identities must be unique",
            ));
        }
        let canonical_atoms = atoms
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, atom)| {
                Ok(AuthoredDirectGlycosidicHaworthCanonicalAtomV1 {
                    atom,
                    authored_child_order: u32::try_from(index).map_err(|_| {
                        HaworthError::InvalidSpec("authored Haworth child order exceeds u32")
                    })?,
                })
            })
            .collect::<Result<Vec<_>, HaworthError>>()?;
        let canonical_bonds = self
            .bonds_in_canonical_order
            .iter()
            .zip(bonds)
            .enumerate()
            .map(|(index, (fact, bond))| {
                let role = match fact.haworth_position() {
                    Some(_) => AuthoredDirectGlycosidicHaworthBondRoleV1::Ring,
                    None => AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge,
                };
                let endpoints = fact.endpoints().each_ref().map(|source| {
                    atom_map
                        .get(source)
                        .cloned()
                        .ok_or(HaworthError::InvalidSpec(
                            "durable Haworth endpoint correspondence is incomplete",
                        ))
                });
                let [start, end] = [endpoints[0].clone()?, endpoints[1].clone()?];
                let authored_child_order =
                    u32::try_from(self.atoms_in_canonical_order.len() + index).map_err(|_| {
                        HaworthError::InvalidSpec("authored Haworth child order exceeds u32")
                    })?;
                match role {
                    AuthoredDirectGlycosidicHaworthBondRoleV1::Ring => {
                        let source = self
                            .source_spec
                            .ring_bonds()
                            .get(fact.source_bond_identity())
                            .ok_or(HaworthError::InvalidSpec(
                                "durable Haworth ring correspondence is incomplete",
                            ))?;
                        if source.endpoints() != fact.endpoints()
                            || source.style() != fact.token()
                            || Some(source.haworth_position()) != fact.haworth_position()
                        {
                            return Err(HaworthError::InvalidSpec(
                                "durable Haworth ring facts disagree with the checked receipt",
                            ));
                        }
                    }
                    AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge => {
                        let source = self
                            .source_spec
                            .bridge_bonds()
                            .get(fact.source_bond_identity())
                            .ok_or(HaworthError::InvalidSpec(
                                "durable Haworth bridge correspondence is incomplete",
                            ))?;
                        if source.endpoints() != fact.endpoints()
                            || fact.token() != DirectGlycosidicHaworthBondStyleV1::N1
                        {
                            return Err(HaworthError::InvalidSpec(
                                "durable Haworth bridge facts disagree with the checked receipt",
                            ));
                        }
                    }
                }
                Ok(AuthoredDirectGlycosidicHaworthCanonicalBondV1 {
                    bond: bond.clone(),
                    endpoints: [start, end],
                    role,
                    token: fact.token(),
                    haworth_position: fact.haworth_position(),
                    authored_child_order,
                })
            })
            .collect::<Result<Vec<_>, HaworthError>>()?;
        let mut coordinates = BTreeMap::new();
        for (source, point) in self.source_spec.coordinates() {
            let translated = HaworthPoint {
                x: point.x + translation.x,
                y: point.y + translation.y,
            };
            if !translated.x.is_finite() || !translated.y.is_finite() {
                return Err(HaworthError::InvalidSpec(
                    "durable Haworth translation is not finite",
                ));
            }
            coordinates.insert(
                atom_map
                    .get(source)
                    .ok_or(HaworthError::InvalidSpec(
                        "durable Haworth atom correspondence is incomplete",
                    ))?
                    .clone(),
                translated,
            );
        }
        let bond_child_orders: BTreeMap<_, _> = self
            .bonds_in_canonical_order
            .iter()
            .enumerate()
            .map(|(index, fact)| {
                u32::try_from(self.atoms_in_canonical_order.len() + index)
                    .map(|order| (fact.source_bond_identity().clone(), order))
                    .map_err(|_| {
                        HaworthError::InvalidSpec("authored Haworth child order exceeds u32")
                    })
            })
            .collect::<Result<_, _>>()?;
        let mut ring_bonds = BTreeMap::new();
        for (source, fact) in self.source_spec.ring_bonds() {
            let bond = bond_map
                .get(source)
                .ok_or(HaworthError::InvalidSpec(
                    "durable Haworth bond correspondence is incomplete",
                ))?
                .clone();
            let endpoints = fact
                .endpoints()
                .each_ref()
                .map(|id| {
                    atom_map
                        .get(id)
                        .ok_or(HaworthError::InvalidSpec(
                            "durable Haworth endpoint correspondence is incomplete",
                        ))
                        .cloned()
                })
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            let [start, end] = endpoints.try_into().map_err(|_| {
                HaworthError::InvalidSpec("durable Haworth endpoint correspondence is incomplete")
            })?;
            let order = *bond_child_orders
                .get(source)
                .ok_or(HaworthError::InvalidSpec(
                    "durable Haworth bond correspondence is incomplete",
                ))?;
            ring_bonds.insert(
                bond.clone(),
                AuthoredDirectGlycosidicHaworthRingBondV1 {
                    bond,
                    endpoints: [start, end],
                    style: fact.style(),
                    haworth_position: fact.haworth_position(),
                    authored_child_order: order,
                },
            );
        }
        let mut bridge_bonds = BTreeMap::new();
        for (source, fact) in self.source_spec.bridge_bonds() {
            let bond = bond_map
                .get(source)
                .ok_or(HaworthError::InvalidSpec(
                    "durable Haworth bond correspondence is incomplete",
                ))?
                .clone();
            let endpoints = fact
                .endpoints()
                .each_ref()
                .map(|id| {
                    atom_map
                        .get(id)
                        .ok_or(HaworthError::InvalidSpec(
                            "durable Haworth endpoint correspondence is incomplete",
                        ))
                        .cloned()
                })
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            let [start, end] = endpoints.try_into().map_err(|_| {
                HaworthError::InvalidSpec("durable Haworth endpoint correspondence is incomplete")
            })?;
            let order = *bond_child_orders
                .get(source)
                .ok_or(HaworthError::InvalidSpec(
                    "durable Haworth bond correspondence is incomplete",
                ))?;
            bridge_bonds.insert(
                bond.clone(),
                AuthoredDirectGlycosidicHaworthBridgeBondV1 {
                    bond,
                    endpoints: [start, end],
                    authored_child_order: order,
                },
            );
        }
        let mut authored_rings = Vec::with_capacity(2);
        for ring in self.source_spec.rings() {
            authored_rings.push(AuthoredDirectGlycosidicHaworthRingV1 {
                ring_form: ring.ring_form(),
                bonds_in_canonical_cycle_order: ring
                    .bonds_in_canonical_cycle_order()
                    .iter()
                    .map(|id| {
                        bond_map.get(id).cloned().ok_or(HaworthError::InvalidSpec(
                            "durable Haworth cycle correspondence is incomplete",
                        ))
                    })
                    .collect::<Result<_, _>>()?,
            });
        }
        let rings = authored_rings.try_into().map_err(|_| {
            HaworthError::InvalidSpec("durable Haworth receipt must retain two rings")
        })?;
        let bounds = bounds(coordinates.values().copied())?;
        Ok(AuthoredDirectGlycosidicHaworthDepictionV1 {
            rings,
            coordinates,
            ring_bonds,
            bridge_bonds,
            canonical_atoms,
            canonical_bonds,
            bounds,
        })
    }
    /// Return selected atoms in ring-zero, ring-one, then bridge-oxygen order.
    #[must_use]
    pub fn atoms_in_canonical_order(&self) -> &[DirectGlycosidicHaworthSelectedAtomFactV1] {
        &self.atoms_in_canonical_order
    }

    /// Return ring-cycle then bridge bonds in canonical authoring order.
    #[must_use]
    pub fn bonds_in_canonical_order(&self) -> &[DirectGlycosidicHaworthSelectedBondFactV1] {
        &self.bonds_in_canonical_order
    }

    /// Return finite receipt-local bounds.
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }

    /// Return the one positive finite local geometry scale.
    #[must_use]
    pub const fn local_scale(&self) -> f64 {
        self.local_scale
    }
}

/// Build detached authoring facts from one complete closed C/O source molecule.
///
/// # Errors
///
/// Returns [`HaworthError`] when the supplied classification is stale or mixed,
/// the source graph is not exactly the closed profile, or local layout fails.
pub fn direct_glycosidic_haworth_authoring_receipt_v1(
    molecule: &Molecule,
    classification: DirectGlycosidicHaworthTopologyV1,
    local_scale: f64,
) -> Result<DirectGlycosidicHaworthAuthoringReceiptV1, HaworthError> {
    if !local_scale.is_finite() || local_scale <= 0.0 {
        return Err(HaworthError::InvalidSpec(
            "scale must be finite and positive",
        ));
    }
    let reconstructed = DirectGlycosidicHaworthTopologyV1::classify(
        molecule,
        [
            classification.rings()[0].topology().clone(),
            classification.rings()[1].topology().clone(),
        ],
        classification.bridge().atom().clone(),
        classification.bridge().bonds().clone(),
    )
    .map_err(|_| HaworthError::StaleTopology("classification does not match molecule snapshot"))?;
    if reconstructed != classification {
        return Err(HaworthError::StaleTopology(
            "classification does not match molecule snapshot",
        ));
    }
    validate_closed_source_profile(molecule, &classification)?;

    let fragment = assemble_direct_glycosidic_haworth_fragment_v1(
        &DirectGlycosidicHaworthFragmentRequestV1 {
            topology: classification.clone(),
            scale: local_scale,
        },
    )?;
    let spec = direct_glycosidic_haworth_depiction_spec_v1(&fragment)?;
    let mut atoms_in_canonical_order = Vec::new();
    for ring in classification.rings() {
        for vertex in ring.topology().vertices() {
            let source = molecule
                .atoms()
                .iter()
                .find(|atom| atom.identity() == &vertex.atom)
                .ok_or(HaworthError::StaleTopology("selected atom is absent"))?;
            let element = exact_authoring_element(source.element())?;
            let local = *spec
                .coordinates()
                .get(&vertex.atom)
                .ok_or(HaworthError::InvalidSpec(
                    "depiction spec is missing selected atom coordinate",
                ))?;
            atoms_in_canonical_order.push(DirectGlycosidicHaworthSelectedAtomFactV1 {
                source_atom_identity: vertex.atom.clone(),
                element,
                local,
            });
        }
    }
    let bridge = classification.bridge().atom();
    let source = molecule
        .atoms()
        .iter()
        .find(|atom| atom.identity() == bridge)
        .ok_or(HaworthError::StaleTopology("bridge atom is absent"))?;
    atoms_in_canonical_order.push(DirectGlycosidicHaworthSelectedAtomFactV1 {
        source_atom_identity: bridge.clone(),
        element: exact_authoring_element(source.element())?,
        local: *spec
            .coordinates()
            .get(bridge)
            .ok_or(HaworthError::InvalidSpec(
                "depiction spec is missing bridge coordinate",
            ))?,
    });

    let mut bonds_in_canonical_order = Vec::new();
    for ring in spec.rings() {
        for bond in ring.bonds_in_canonical_cycle_order() {
            let fact = spec
                .ring_bonds()
                .get(bond)
                .ok_or(HaworthError::InvalidSpec(
                    "depiction spec is missing selected ring bond",
                ))?;
            bonds_in_canonical_order.push(DirectGlycosidicHaworthSelectedBondFactV1 {
                source_bond_identity: bond.clone(),
                endpoints: fact.endpoints().clone(),
                token: fact.style(),
                haworth_position: Some(fact.haworth_position()),
            });
        }
    }
    for bond in classification.bridge().bonds() {
        let fact = spec
            .bridge_bonds()
            .get(bond)
            .ok_or(HaworthError::InvalidSpec(
                "depiction spec is missing selected bridge bond",
            ))?;
        bonds_in_canonical_order.push(DirectGlycosidicHaworthSelectedBondFactV1 {
            source_bond_identity: bond.clone(),
            endpoints: fact.endpoints().clone(),
            token: DirectGlycosidicHaworthBondStyleV1::N1,
            haworth_position: None,
        });
    }
    Ok(DirectGlycosidicHaworthAuthoringReceiptV1 {
        atoms_in_canonical_order,
        bonds_in_canonical_order,
        bounds: spec.bounds(),
        local_scale,
        source_spec: spec,
    })
}

fn validate_closed_source_profile(
    molecule: &Molecule,
    classification: &DirectGlycosidicHaworthTopologyV1,
) -> Result<(), HaworthError> {
    if molecule.name().is_some()
        || !molecule.groups().is_empty()
        || !molecule.texts().is_empty()
        || !molecule.queries().is_empty()
    {
        return Err(HaworthError::UnsupportedTopology(
            "closed authoring source cannot contain molecule metadata or non-atom vertices",
        ));
    }
    let selected_atoms: BTreeSet<_> = classification
        .rings()
        .iter()
        .flat_map(|ring| {
            ring.topology()
                .vertices()
                .iter()
                .map(|vertex| vertex.atom.clone())
        })
        .chain(std::iter::once(classification.bridge().atom().clone()))
        .collect();
    let selected_bonds: BTreeSet<_> = classification
        .rings()
        .iter()
        .flat_map(|ring| ring.topology().bond_ids().iter().cloned())
        .chain(classification.bridge().bonds().iter().cloned())
        .collect();
    if molecule.atoms().len() != selected_atoms.len()
        || molecule.bonds().len() != selected_bonds.len()
        || molecule
            .atoms()
            .iter()
            .map(|atom| atom.identity())
            .collect::<BTreeSet<_>>()
            != selected_atoms.iter().collect()
        || molecule
            .bonds()
            .iter()
            .map(|bond| bond.identity())
            .collect::<BTreeSet<_>>()
            != selected_bonds.iter().collect()
    {
        return Err(HaworthError::UnsupportedTopology(
            "closed authoring source must contain exactly selected atoms and bonds",
        ));
    }
    if molecule.atoms().iter().any(|atom| {
        atom.formal_charge().is_some()
            || atom.isotope().is_some()
            || atom.explicit_hydrogens().is_some()
            || atom.valence().is_some()
            || atom.multiplicity().is_some()
            || atom.free_sites().is_some()
            || !matches!(atom.element(), Some("C" | "O"))
    }) {
        return Err(HaworthError::UnsupportedTopology(
            "closed authoring source cannot contain optional atom chemistry",
        ));
    }
    if molecule
        .bonds()
        .iter()
        .any(|bond| bond.source_type().is_some() || bond.style().is_some())
    {
        return Err(HaworthError::UnsupportedTopology(
            "closed authoring source cannot contain bond type or style facts",
        ));
    }
    Ok(())
}

fn exact_authoring_element(
    element: Option<&str>,
) -> Result<DirectGlycosidicHaworthAuthoringAtomElementV1, HaworthError> {
    match element {
        Some("C") => Ok(DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon),
        Some("O") => Ok(DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen),
        _ => Err(HaworthError::UnsupportedTopology(
            "closed authoring source atoms must be exact C or O",
        )),
    }
}
