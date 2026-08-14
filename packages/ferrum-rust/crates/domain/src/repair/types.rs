//! Immutable repair requests and durable depiction graph validation.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::{GeometryError, Point2};
use thiserror::Error;

/// One finite coordinate carrying an atom's durable identity.
#[derive(Clone, Debug, PartialEq)]
pub struct DepictionVertex {
    atom_id: RecordId,
    coordinate: Point2,
}

impl DepictionVertex {
    /// Create a coordinate record for an atom identity.
    pub fn new(atom_id: RecordId, coordinate: Point2) -> Result<Self, RepairError> {
        if atom_id.kind() != RecordKind::Atom {
            return Err(RepairError::InvalidGraph(
                "depiction vertex identities must have atom kind",
            ));
        }
        Ok(Self {
            atom_id,
            coordinate,
        })
    }

    /// Return the durable atom identity.
    #[must_use]
    pub const fn atom_id(&self) -> &RecordId {
        &self.atom_id
    }

    /// Return the finite coordinate.
    #[must_use]
    pub const fn coordinate(&self) -> Point2 {
        self.coordinate
    }
}

/// One durable bond between two depicted atom identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DepictionBond {
    bond_id: RecordId,
    start: RecordId,
    end: RecordId,
}

impl DepictionBond {
    /// Create a bond record. Endpoint membership is checked by [`DepictionGraph::new`].
    pub fn new(bond_id: RecordId, start: RecordId, end: RecordId) -> Result<Self, RepairError> {
        if bond_id.kind() != RecordKind::Bond {
            return Err(RepairError::InvalidGraph(
                "depiction bond identities must have bond kind",
            ));
        }
        if start.kind() != RecordKind::Atom || end.kind() != RecordKind::Atom {
            return Err(RepairError::InvalidGraph(
                "depiction bond endpoints must have atom kind",
            ));
        }
        if start == end {
            return Err(RepairError::InvalidGraph(
                "depiction bonds must not be self-bonds",
            ));
        }
        Ok(Self {
            bond_id,
            start,
            end,
        })
    }

    /// Return the durable bond identity.
    #[must_use]
    pub const fn bond_id(&self) -> &RecordId {
        &self.bond_id
    }

    /// Return the first durable atom endpoint.
    #[must_use]
    pub const fn start(&self) -> &RecordId {
        &self.start
    }

    /// Return the second durable atom endpoint.
    #[must_use]
    pub const fn end(&self) -> &RecordId {
        &self.end
    }
}

/// A validated coordinate-only graph. It has no molecule-editing operations.
#[derive(Clone, Debug, PartialEq)]
pub struct DepictionGraph {
    vertices: BTreeMap<RecordId, Point2>,
    bonds: BTreeMap<RecordId, (RecordId, RecordId)>,
    vertex_source_order: Vec<RecordId>,
    bond_source_order: Vec<RecordId>,
}

impl DepictionGraph {
    /// Create a graph with unique durable identities and validated endpoints.
    pub fn new(
        vertices: Vec<DepictionVertex>,
        bonds: Vec<DepictionBond>,
    ) -> Result<Self, RepairError> {
        let mut vertex_map = BTreeMap::new();
        let mut vertex_source_order = Vec::with_capacity(vertices.len());
        for vertex in vertices {
            vertex_source_order.push(vertex.atom_id.clone());
            if vertex_map
                .insert(vertex.atom_id, vertex.coordinate)
                .is_some()
            {
                return Err(RepairError::InvalidGraph(
                    "depiction vertex identities must be unique",
                ));
            }
        }

        let mut bond_map = BTreeMap::new();
        let mut endpoint_pairs = BTreeSet::new();
        let mut bond_source_order = Vec::with_capacity(bonds.len());
        for bond in bonds {
            bond_source_order.push(bond.bond_id.clone());
            if !vertex_map.contains_key(&bond.start) || !vertex_map.contains_key(&bond.end) {
                return Err(RepairError::InvalidGraph(
                    "depiction bond endpoints must belong to the graph",
                ));
            }
            if bond_map
                .insert(bond.bond_id, (bond.start.clone(), bond.end.clone()))
                .is_some()
            {
                return Err(RepairError::InvalidGraph(
                    "depiction bond identities must be unique",
                ));
            }
            let pair = ordered_pair(bond.start, bond.end);
            if !endpoint_pairs.insert(pair) {
                return Err(RepairError::InvalidGraph(
                    "depiction graph cannot contain parallel bond endpoints",
                ));
            }
        }
        Ok(Self {
            vertices: vertex_map,
            bonds: bond_map,
            vertex_source_order,
            bond_source_order,
        })
    }

    /// Return vertices in durable-identity order.
    #[must_use]
    pub fn vertices(&self) -> impl ExactSizeIterator<Item = (&RecordId, Point2)> {
        self.vertices.iter().map(|(id, point)| (id, *point))
    }

    /// Return bonds in durable-identity order.
    #[must_use]
    pub fn bonds(&self) -> impl ExactSizeIterator<Item = (&RecordId, &RecordId, &RecordId)> {
        self.bonds
            .iter()
            .map(|(bond_id, (start, end))| (bond_id, start, end))
    }

    pub(crate) fn coordinates(&self) -> &BTreeMap<RecordId, Point2> {
        &self.vertices
    }

    pub(crate) fn edges(&self) -> &BTreeMap<RecordId, (RecordId, RecordId)> {
        &self.bonds
    }

    pub(crate) fn source_atom_order(&self) -> &[RecordId] {
        &self.vertex_source_order
    }

    pub(crate) fn source_edges(&self) -> impl Iterator<Item = (&RecordId, &RecordId)> {
        self.bond_source_order.iter().map(|bond_id| {
            let (start, end) = &self.bonds[bond_id];
            (start, end)
        })
    }
}

/// A supported coordinate-only repair operation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RepairKind {
    /// Snap each atom independently to a deterministic pointy-top hex lattice.
    SnapToHexGrid {
        /// Positive nearest-neighbor lattice spacing in drawing units.
        spacing: f64,
        /// Origin of the lattice in Ferrum's y-up coordinate frame.
        origin: Point2,
    },
    /// Rotate the whole depiction using Ferrum's M11 straightening arithmetic.
    Straighten {
        /// Preserve a near-grid orientation when M11's policy selects it.
        minimize_rotation: bool,
    },
    /// Snap only degree-one endpoints to canonical 30-degree directions.
    StraightenTerminalBonds,
    /// Set eligible non-ring bond lengths while preserving source directions.
    NormalizeBondLengths {
        /// Positive target length in the graph's drawing units.
        spacing: f64,
    },
    /// Regularize one simple ring and translate its attached acyclic components.
    NormalizeSingleRing {
        /// Positive target side length in the graph's drawing units.
        spacing: f64,
    },
    /// Snap movable non-ring outgoing bonds to distinct 60-degree slots.
    NormalizeBondAngles {
        /// Positive fallback length used only for coincident outgoing atoms.
        spacing: f64,
    },
}

/// An immutable request to calculate, not apply, a coordinate repair.
#[derive(Clone, Debug, PartialEq)]
pub struct RepairRequest {
    graph: DepictionGraph,
    kind: RepairKind,
}

impl RepairRequest {
    /// Combine a validated graph with one explicit repair operation.
    #[must_use]
    pub const fn new(graph: DepictionGraph, kind: RepairKind) -> Self {
        Self { graph, kind }
    }

    /// Return the selected immutable depiction graph.
    #[must_use]
    pub const fn graph(&self) -> &DepictionGraph {
        &self.graph
    }

    /// Return the requested coordinate operation.
    #[must_use]
    pub const fn kind(&self) -> RepairKind {
        self.kind
    }
}

/// The complete result of one immutable coordinate-repair plan.
///
/// Every outcome retains the guarded sparse patch used by existing callers.
/// Whole-depiction straightening additionally retains every calculated coordinate
/// in durable-identity order and the exact y-up counter-clockwise rotation that
/// produced them. Other repair kinds leave both straightening-specific results absent.
#[derive(Clone, Debug, PartialEq)]
pub struct RepairOutcome {
    patch: CoordinatePatch,
    straightened_coordinates: Option<BTreeMap<RecordId, Point2>>,
    applied_rotation_radians: Option<f64>,
}

impl RepairOutcome {
    pub(crate) fn from_patch(patch: CoordinatePatch) -> Self {
        Self {
            patch,
            straightened_coordinates: None,
            applied_rotation_radians: None,
        }
    }

    pub(crate) fn from_straightening(
        original: &BTreeMap<RecordId, Point2>,
        coordinates: BTreeMap<RecordId, Point2>,
        applied_rotation_radians: f64,
    ) -> Self {
        let patch = CoordinatePatch::from_candidates(
            original,
            coordinates.iter().map(|(id, point)| (id.clone(), *point)),
        );
        Self {
            patch,
            straightened_coordinates: Some(coordinates),
            applied_rotation_radians: Some(applied_rotation_radians),
        }
    }

    pub(crate) fn into_patch(self) -> CoordinatePatch {
        self.patch
    }

    /// Return the guarded sparse patch retained for compatibility with existing callers.
    #[must_use]
    pub const fn patch(&self) -> &CoordinatePatch {
        &self.patch
    }

    /// Return complete whole-depiction coordinates in durable-identity order.
    ///
    /// Only [`RepairKind::Straighten`] produces this result. The coordinates are
    /// retained before sparse patch filtering, so a no-rotation result remains
    /// observable even when [`Self::patch`] is empty.
    #[must_use]
    pub fn straightened_coordinates(
        &self,
    ) -> Option<impl ExactSizeIterator<Item = (&RecordId, Point2)>> {
        self.straightened_coordinates
            .as_ref()
            .map(|coordinates| coordinates.iter().map(|(id, point)| (id, *point)))
    }

    /// Return the exact y-up counter-clockwise rotation applied by straightening.
    ///
    /// Only [`RepairKind::Straighten`] produces an angle. A successful no-op
    /// straightening reports `Some(0.0)` rather than absence.
    #[must_use]
    pub const fn applied_rotation_radians(&self) -> Option<f64> {
        self.applied_rotation_radians
    }
}

/// One guarded coordinate replacement in a [`CoordinatePatch`].
///
/// `expected` is the coordinate used while planning. A persistence boundary
/// must verify it immediately before atomically applying `replacement`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateReplacement {
    expected: Point2,
    replacement: Point2,
}

impl CoordinateReplacement {
    /// Return the coordinate that must still be current before application.
    #[must_use]
    pub const fn expected(self) -> Point2 {
        self.expected
    }

    /// Return the planned replacement coordinate.
    #[must_use]
    pub const fn replacement(self) -> Point2 {
        self.replacement
    }
}

/// A sparse immutable, guarded replacement set. An empty patch is a successful no-op.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoordinatePatch {
    replacements: BTreeMap<RecordId, CoordinateReplacement>,
}

impl CoordinatePatch {
    pub(crate) fn from_candidates(
        original: &BTreeMap<RecordId, Point2>,
        candidates: impl IntoIterator<Item = (RecordId, Point2)>,
    ) -> Self {
        let replacements = candidates
            .into_iter()
            .filter_map(|(id, candidate)| {
                original.get(&id).and_then(|expected| {
                    (expected != &candidate).then_some((
                        id,
                        CoordinateReplacement {
                            expected: *expected,
                            replacement: candidate,
                        },
                    ))
                })
            })
            .collect();
        Self { replacements }
    }

    /// Return whether no coordinate needs replacement.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.replacements.is_empty()
    }

    /// Return guarded coordinate replacements in durable-identity order.
    ///
    /// Before applying any returned replacement, validate the entire patch
    /// with [`Self::validate_preconditions`] against one current snapshot.
    #[must_use]
    pub fn replacements(
        &self,
    ) -> impl ExactSizeIterator<Item = (&RecordId, CoordinateReplacement)> {
        self.replacements
            .iter()
            .map(|(id, replacement)| (id, *replacement))
    }

    /// Verify that every patched atom still has its planned source coordinate.
    ///
    /// A persistence boundary supplies one coherent current-coordinate snapshot,
    /// validates this method once, then applies every replacement atomically.
    /// Extra current coordinates are accepted; missing or changed patched atoms
    /// fail closed and leave application to the caller's transaction policy.
    pub fn validate_preconditions(
        &self,
        current_coordinates: impl IntoIterator<Item = (RecordId, Point2)>,
    ) -> Result<(), PatchPreconditionError> {
        let current = current_coordinates.into_iter().collect::<BTreeMap<_, _>>();
        for (atom_id, replacement) in &self.replacements {
            if current.get(atom_id) != Some(&replacement.expected) {
                return Err(PatchPreconditionError::StaleCoordinate {
                    atom_id: atom_id.clone(),
                });
            }
        }
        Ok(())
    }
}

/// A coordinate patch no longer matches the snapshot from which it was planned.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PatchPreconditionError {
    /// The target atom is missing or its coordinate changed after planning.
    #[error("coordinate patch is stale for atom {atom_id:?}")]
    StaleCoordinate {
        /// The durable atom identity that failed the application guard.
        atom_id: RecordId,
    },
}

/// A recoverable validation or planning failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum RepairError {
    /// Durable identities or endpoint relationships are not a valid depiction graph.
    #[error("invalid depiction graph: {0}")]
    InvalidGraph(&'static str),
    /// The requested graph shape needs a future, explicitly designed profile.
    #[error("unsupported depiction topology: {0}")]
    UnsupportedTopology(&'static str),
    /// A finite coordinate operation could not be represented.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
}

fn ordered_pair(first: RecordId, second: RecordId) -> (RecordId, RecordId) {
    if first <= second {
        (first, second)
    } else {
        (second, first)
    }
}
