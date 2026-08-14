//! Prepared, revision-bound whole-depiction straightening for direct molecules.

use std::collections::HashSet;

use ferrum_domain::repair::{RepairKind, RepairRequest, plan_repair_with_outcome};
use ferrum_geometry::Point2;
use thiserror::Error;

use super::{DocumentObjectIdV1, PersistentId, TypedDocument, TypedDocumentError};

/// One complete y-up layout calculated for a direct-root molecule.
#[derive(Clone, Debug, PartialEq)]
pub struct StraightenedDepictionMoleculeV1 {
    molecule_id: DocumentObjectIdV1,
    positions: Vec<Point2>,
    expected_positions: Vec<Point2>,
    applied_rotation_radians: f64,
}

impl StraightenedDepictionMoleculeV1 {
    fn new(
        molecule_id: DocumentObjectIdV1,
        positions: Vec<Point2>,
        expected_positions: Vec<Point2>,
        applied_rotation_radians: f64,
    ) -> Result<Self, StraightenDepictionUpdateV1Error> {
        if positions.is_empty() || expected_positions.len() != positions.len() {
            return Err(StraightenDepictionUpdateV1Error::InvalidPositions);
        }
        if !applied_rotation_radians.is_finite() {
            return Err(StraightenDepictionUpdateV1Error::InvalidRotation);
        }
        Ok(Self {
            molecule_id,
            positions,
            expected_positions,
            applied_rotation_radians,
        })
    }

    /// Return the durable direct-root molecule selector.
    #[must_use]
    pub fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return every calculated x/y coordinate in direct atom source order.
    ///
    /// These are the geometry planner's y-up coordinates. The document apply
    /// boundary owns conversion back to the persisted CDML coordinate convention.
    #[must_use]
    pub fn positions(&self) -> &[Point2] {
        &self.positions
    }

    pub(super) fn expected_positions(&self) -> &[Point2] {
        &self.expected_positions
    }

    /// Return the exact y-up, counter-clockwise rotation calculated by Ferrum.
    #[must_use]
    pub const fn applied_rotation_radians(&self) -> f64 {
        self.applied_rotation_radians
    }
}

/// Immutable complete straightening result tied to one observed document state.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedStraightenDepictionsV1 {
    source_revision: u64,
    source_digest: [u8; 32],
    molecules: Vec<StraightenedDepictionMoleculeV1>,
}

impl PreparedStraightenDepictionsV1 {
    pub(super) fn new(
        source_revision: u64,
        source_digest: [u8; 32],
        molecules: Vec<StraightenedDepictionMoleculeV1>,
    ) -> Result<Self, StraightenDepictionUpdateV1Error> {
        if molecules.is_empty() {
            return Err(StraightenDepictionUpdateV1Error::EmptyMolecules);
        }
        let mut unique = HashSet::with_capacity(molecules.len());
        if molecules
            .iter()
            .any(|molecule| !unique.insert(molecule.molecule_id.clone()))
        {
            return Err(StraightenDepictionUpdateV1Error::DuplicateMolecule);
        }
        Ok(Self {
            source_revision,
            source_digest,
            molecules,
        })
    }

    /// Return the source session revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the source document digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return selected molecule results in caller order.
    #[must_use]
    pub fn molecules(&self) -> &[StraightenedDepictionMoleculeV1] {
        &self.molecules
    }
}

/// Preparation failure before a session can issue a straightening result.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum StraightenDepictionUpdateV1Error {
    #[error("whole-depiction straightening requires at least one molecule")]
    EmptyMolecules,
    #[error("whole-depiction straightening molecule targets must be unique")]
    DuplicateMolecule,
    #[error("whole-depiction straightening requires complete finite atom positions")]
    InvalidPositions,
    #[error("whole-depiction straightening produced an invalid rotation")]
    InvalidRotation,
}

pub(super) fn prepare_molecule(
    document: &TypedDocument,
    molecule_id: &PersistentId,
    object_id: DocumentObjectIdV1,
    minimize_rotation: bool,
) -> Result<StraightenedDepictionMoleculeV1, TypedDocumentError> {
    let graph = super::typed_geometry_repair::molecule_graph(document, molecule_id)?;
    // `atom_ids` is a map for planner lookup. Obtain source order from the retained
    // document, rather than leaking the domain planner's identity ordering.
    let expected_positions = graph
        .atom_source_order
        .iter()
        .map(|atom_id| {
            let record_id = graph
                .atom_ids
                .iter()
                .find_map(|(record_id, source_id)| (source_id == atom_id).then_some(record_id))
                .expect("source order and graph IDs remain aligned");
            let point = graph
                .graph
                .vertices()
                .find_map(|(candidate, point)| (candidate == record_id).then_some(point))
                .expect("source graph and atom map remain aligned");
            (atom_id.clone(), point)
        })
        .collect::<Vec<_>>();
    let request = RepairRequest::new(graph.graph, RepairKind::Straighten { minimize_rotation });
    let outcome = plan_repair_with_outcome(&request).map_err(|error| {
        TypedDocumentError::GeometryRepairPlanning {
            molecule_id: molecule_id.clone(),
            detail: error.to_string(),
        }
    })?;
    let coordinates = outcome
        .straightened_coordinates()
        .expect("straightening outcomes always retain complete coordinates")
        .map(|(record_id, point)| (record_id.clone(), point))
        .collect::<std::collections::BTreeMap<_, _>>();
    let positions = graph
        .atom_ids
        .iter()
        .map(|(record_id, atom_id)| (atom_id, coordinates[record_id]))
        .collect::<std::collections::BTreeMap<_, _>>();
    let positions = expected_positions
        .iter()
        .map(|(atom_id, _)| positions[atom_id])
        .collect();
    let expected_positions = expected_positions
        .into_iter()
        .map(|(_, point)| point)
        .collect();
    StraightenedDepictionMoleculeV1::new(
        object_id,
        positions,
        expected_positions,
        outcome
            .applied_rotation_radians()
            .expect("straightening outcomes always retain their angle"),
    )
    .map_err(|error| TypedDocumentError::GeometryRepairPlanning {
        molecule_id: molecule_id.clone(),
        detail: error.to_string(),
    })
}
