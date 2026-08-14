//! Haworth request and validated single-ring depiction types.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::{RecordId, RecordKind};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The named ring form requested by a prior parser or explicit caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RingForm {
    /// A six-membered C5O ring.
    Pyranose,
    /// A five-membered C4O ring.
    Furanose,
}

impl RingForm {
    pub(crate) const fn vertex_count(self) -> usize {
        match self {
            Self::Pyranose => 6,
            Self::Furanose => 5,
        }
    }
}

/// The sole fixed orientation emitted by this single-ring layout profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CanonicalOrientation {
    /// Ring oxygen is at upper right; declared C-1 is the rightmost neighbour.
    OxygenUpperRight,
}

/// One explicitly selected cycle vertex.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HaworthVertex {
    /// The atom identity. Only atom identities are accepted by validation.
    pub atom: RecordId,
}

/// An explicit, graph-validated, isolated single-ring topology.
///
/// Vertices are canonicalized as oxygen, then the directed carbon sequence,
/// ending at the caller-declared anomeric carbon. This API deliberately has no
/// multi-ring or substituent semantics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct HaworthTopology {
    ring_form: RingForm,
    vertices: Vec<HaworthVertex>,
    bond_ids: Vec<RecordId>,
}

impl HaworthTopology {
    /// Construct topology after the crate-local graph validator has established
    /// the canonical ring and bond invariants.
    pub(crate) fn from_validated(
        ring_form: RingForm,
        vertices: Vec<HaworthVertex>,
        bond_ids: Vec<RecordId>,
    ) -> Self {
        Self {
            ring_form,
            vertices,
            bond_ids,
        }
    }

    /// Return the declared ring form.
    #[must_use]
    pub const fn ring_form(&self) -> RingForm {
        self.ring_form
    }

    /// Return the canonical oxygen-first cyclic vertices.
    #[must_use]
    pub fn vertices(&self) -> &[HaworthVertex] {
        &self.vertices
    }

    /// Return the matching cyclic bond identities in the same edge order.
    #[must_use]
    pub fn bond_ids(&self) -> &[RecordId] {
        &self.bond_ids
    }
}

/// Builds topology only after graph validation and canonicalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HaworthTopologyBuilder {
    ring_form: RingForm,
    anomeric_atom: RecordId,
    selected_cycle: Vec<HaworthVertex>,
}

impl HaworthTopologyBuilder {
    /// Begin an explicit selected cycle with its declared anomeric carbon.
    ///
    /// Storage direction and starting point carry no layout meaning. The
    /// selected cycle is normalized so the declared C-1 is the final vertex,
    /// adjacent to oxygen at the right of the fixed template.
    #[must_use]
    pub fn new(
        ring_form: RingForm,
        anomeric_atom: RecordId,
        selected_cycle: Vec<HaworthVertex>,
    ) -> Self {
        Self {
            ring_form,
            anomeric_atom,
            selected_cycle,
        }
    }

    /// Validate this cycle in `molecule` and produce immutable topology.
    pub fn build(self, molecule: &ferrum_core::Molecule) -> Result<HaworthTopology, HaworthError> {
        crate::haworth::validate::validate_topology(
            self.ring_form,
            self.anomeric_atom,
            self.selected_cycle,
            molecule,
        )
    }
}

/// A pure geometry request over a validated isolated single ring.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct HaworthLayoutRequest {
    /// A prior explicit topology validation result.
    pub topology: HaworthTopology,
    /// The positive drawing-unit edge length.
    pub scale: f64,
}

/// Stable semantic role for a single front-face edge.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum WedgeEdgeRole {
    /// The central wide edge.
    Center,
    /// The shoulder reached before the center in canonical ring order.
    LeftShoulder,
    /// The shoulder reached after the center in canonical ring order.
    RightShoulder,
}

/// The semantic face of a projected feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Face {
    /// Feature visually projects toward the reader.
    Front,
    /// Feature visually projects away from the reader.
    Back,
}

/// A drawing-semantic treatment for a cycle bond.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum BondDepiction {
    /// An ordinary back-face ring edge.
    Back { face: Face },
    /// One of the uniquely named edges of the visible Haworth face.
    HaworthFront {
        edge_role: WedgeEdgeRole,
        face: Face,
    },
}

/// A finite serializable point owned by the Haworth API boundary.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct HaworthPoint {
    /// Horizontal coordinate in Ferrum drawing units.
    pub x: f64,
    /// Vertical coordinate in Ferrum drawing units, using y-up geometry.
    pub y: f64,
}

/// A validated, immutable result of the isolated single-ring planner.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthDepiction {
    ring_form: RingForm,
    coordinates: BTreeMap<RecordId, HaworthPoint>,
    bonds: BTreeMap<RecordId, BondDepiction>,
    bounds: [HaworthPoint; 2],
    orientation: CanonicalOrientation,
}

impl HaworthDepiction {
    pub(crate) fn new(
        ring_form: RingForm,
        coordinates: BTreeMap<RecordId, HaworthPoint>,
        bonds: BTreeMap<RecordId, BondDepiction>,
        bounds: [HaworthPoint; 2],
    ) -> Result<Self, HaworthError> {
        let depiction = Self {
            ring_form,
            coordinates,
            bonds,
            bounds,
            orientation: CanonicalOrientation::OxygenUpperRight,
        };
        depiction.validate()?;
        Ok(depiction)
    }

    /// Return the rendered ring form.
    #[must_use]
    pub const fn ring_form(&self) -> RingForm {
        self.ring_form
    }

    /// Return coordinates keyed by atom identity.
    #[must_use]
    pub fn coordinates(&self) -> &BTreeMap<RecordId, HaworthPoint> {
        &self.coordinates
    }

    /// Return per-bond depiction semantics keyed by bond identity.
    #[must_use]
    pub fn bonds(&self) -> &BTreeMap<RecordId, BondDepiction> {
        &self.bonds
    }

    /// Return bounds derived exactly from the finite coordinates.
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }

    /// Return the fixed orientation of this layout profile.
    #[must_use]
    pub const fn orientation(&self) -> CanonicalOrientation {
        self.orientation
    }

    fn validate(&self) -> Result<(), HaworthError> {
        let expected = self.ring_form.vertex_count();
        if self.coordinates.len() != expected || self.bonds.len() != expected {
            return Err(HaworthError::InvalidSpec(
                "depiction must contain one atom and bond entry per ring member",
            ));
        }
        if self
            .coordinates
            .iter()
            .any(|(id, point)| id.kind() != RecordKind::Atom || !is_finite(*point))
            || self.bonds.keys().any(|id| id.kind() != RecordKind::Bond)
        {
            return Err(HaworthError::InvalidSpec(
                "depiction identities and coordinates must be finite typed records",
            ));
        }
        let derived_bounds = derive_bounds(&self.coordinates)?;
        if !self.bounds.iter().copied().all(is_finite) || self.bounds != derived_bounds {
            return Err(HaworthError::InvalidSpec(
                "depiction bounds must be finite and derived from coordinates",
            ));
        }
        let mut roles = BTreeSet::new();
        let mut front_count = 0;
        for depiction in self.bonds.values() {
            match depiction {
                BondDepiction::Back { face: Face::Back } => {}
                BondDepiction::HaworthFront {
                    edge_role,
                    face: Face::Front,
                } => {
                    front_count += 1;
                    roles.insert(*edge_role);
                }
                _ => {
                    return Err(HaworthError::InvalidSpec(
                        "depiction bond face must match its semantic role",
                    ));
                }
            }
        }
        if front_count != 3
            || roles.len() != 3
            || !roles.contains(&WedgeEdgeRole::Center)
            || !roles.contains(&WedgeEdgeRole::LeftShoulder)
            || !roles.contains(&WedgeEdgeRole::RightShoulder)
        {
            return Err(HaworthError::InvalidSpec(
                "depiction must contain each Haworth front-edge role exactly once",
            ));
        }
        Ok(())
    }
}

fn is_finite(point: HaworthPoint) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn derive_bounds(
    coordinates: &BTreeMap<RecordId, HaworthPoint>,
) -> Result<[HaworthPoint; 2], HaworthError> {
    let mut points = coordinates.values();
    let first = points
        .next()
        .copied()
        .ok_or(HaworthError::InvalidSpec("depiction has no coordinates"))?;
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

#[derive(Serialize, Deserialize)]
struct HaworthDepictionWire {
    ring_form: RingForm,
    coordinates: Vec<(RecordId, HaworthPoint)>,
    bonds: Vec<(RecordId, BondDepiction)>,
    bounds: [HaworthPoint; 2],
}

impl Serialize for HaworthDepiction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        HaworthDepictionWire {
            ring_form: self.ring_form,
            coordinates: self
                .coordinates
                .iter()
                .map(|(id, point)| (id.clone(), *point))
                .collect(),
            bonds: self
                .bonds
                .iter()
                .map(|(id, depiction)| (id.clone(), *depiction))
                .collect(),
            bounds: self.bounds,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HaworthDepiction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = HaworthDepictionWire::deserialize(deserializer)?;
        let coordinate_count = wire.coordinates.len();
        let bond_count = wire.bonds.len();
        let coordinates: BTreeMap<_, _> = wire.coordinates.into_iter().collect();
        let bonds: BTreeMap<_, _> = wire.bonds.into_iter().collect();
        if coordinates.len() != coordinate_count || bonds.len() != bond_count {
            return Err(serde::de::Error::custom(
                "depiction wire entries must not repeat identities",
            ));
        }
        Self::new(wire.ring_form, coordinates, bonds, wire.bounds).map_err(serde::de::Error::custom)
    }
}

/// Typed failures from Haworth planning.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum HaworthError {
    /// A supplied semantic or geometry request was invalid.
    #[error("invalid Haworth request: {0}")]
    InvalidSpec(&'static str),
    /// The selected graph does not fit this profile.
    #[error("unsupported Haworth topology: {0}")]
    UnsupportedTopology(&'static str),
    /// The explicitly selected graph has changed since selection.
    #[error("stale Haworth topology: {0}")]
    StaleTopology(&'static str),
    /// Finite placement could not be represented.
    #[error("Haworth geometry is unplaceable: {0}")]
    Unplaceable(&'static str),
    /// Required storage for a checked Haworth operation could not be reserved.
    #[error("Haworth resource allocation failed")]
    ResourceExhausted,
}
