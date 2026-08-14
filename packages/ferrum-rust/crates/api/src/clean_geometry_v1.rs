//! Native clean-geometry preparation behind the narrow chemistry engine boundary.

use std::collections::HashSet;

use ferrum_chemistry::ChemEngine;
use ferrum_document::{
    CleanGeometryMoleculeV1, CleanGeometryUpdateV1, CleanGeometryUpdateV1Error,
    CoreProjectionError, DocumentObjectIdV1, SessionDocumentObservationV1, TypedDocument,
    TypedDocumentError,
};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2, place_molecule_depiction_v1};
use thiserror::Error;

use crate::molecule_coordinate_generation_v1::{MoleculeCoordinateBuildError, coordinate_graph};

/// Prepare one atomic selected-molecule clean-geometry update.
pub fn build_clean_geometry_update_v1<E: ChemEngine>(
    engine: &E,
    observation: &SessionDocumentObservationV1,
    molecule_ids: &[DocumentObjectIdV1],
    target_spacing_points: f64,
) -> Result<CleanGeometryUpdateV1, CleanGeometryBuildError> {
    if !target_spacing_points.is_finite() || target_spacing_points <= 0.0 {
        return Err(CleanGeometryBuildError::InvalidTargetSpacing);
    }
    if molecule_ids.is_empty() {
        return Err(CleanGeometryBuildError::EmptyMolecules);
    }
    let mut unique = HashSet::with_capacity(molecule_ids.len());
    if molecule_ids.iter().any(|id| !unique.insert(id.clone())) {
        return Err(CleanGeometryBuildError::DuplicateMolecule);
    }
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    let targets = molecule_ids
        .iter()
        .map(|molecule_id| {
            let molecule = document.core_molecule(molecule_id)?.ok_or_else(|| {
                CleanGeometryBuildError::UnknownMolecule {
                    object_id: molecule_id.as_str().to_owned(),
                }
            })?;
            let (graph, current_points, edges, _z_values) =
                coordinate_graph(&molecule).map_err(|source| CleanGeometryBuildError::Target {
                    object_id: molecule_id.as_str().to_owned(),
                    source,
                })?;
            if edges.is_empty() {
                return Err(CleanGeometryBuildError::UnbondedMolecule {
                    object_id: molecule_id.as_str().to_owned(),
                });
            }
            Ok((molecule_id.clone(), graph, current_points, edges))
        })
        .collect::<Result<Vec<_>, CleanGeometryBuildError>>()?;
    let mut molecules = Vec::with_capacity(targets.len());
    for (molecule_id, graph, current_points, edges) in targets {
        let generated = engine.generate_2d_coordinates(&graph).map_err(|source| {
            CleanGeometryBuildError::Target {
                object_id: molecule_id.as_str().to_owned(),
                source: MoleculeCoordinateBuildError::Chemistry(source),
            }
        })?;
        if generated.points().len() != graph.atoms().len() {
            return Err(CleanGeometryBuildError::GeneratedAtomCountMismatch {
                object_id: molecule_id.as_str().to_owned(),
                expected: graph.atoms().len(),
                actual: generated.points().len(),
            });
        }
        let generated_points = generated
            .points()
            .iter()
            .map(|point| Point2::new(point.x(), point.y()))
            .collect::<Result<Vec<_>, _>>()?;
        let anchor = centroid(&current_points)?;
        let placement = MoleculePlacementV1::new(target_spacing_points, anchor)?;
        let positions = place_molecule_depiction_v1(&generated_points, &edges, placement)?;
        molecules.push(CleanGeometryMoleculeV1::new(molecule_id, positions)?);
    }
    CleanGeometryUpdateV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        molecules,
    )
    .map_err(Into::into)
}

fn centroid(points: &[Point2]) -> Result<Point2, GeometryError> {
    if points.is_empty() {
        return Err(GeometryError::EmptyCoordinateSet);
    }
    let count = points.len() as f64;
    Point2::new(
        points.iter().map(|point| point.x()).sum::<f64>() / count,
        points.iter().map(|point| point.y()).sum::<f64>() / count,
    )
}

/// Failure while preparing one clean-geometry update.
#[derive(Debug, Error)]
pub enum CleanGeometryBuildError {
    /// The explicit requested spacing was invalid.
    #[error("clean geometry target spacing must be finite and greater than zero")]
    InvalidTargetSpacing,
    /// At least one durable target is required.
    #[error("clean geometry requires at least one molecule")]
    EmptyMolecules,
    /// A target appeared more than once.
    #[error("clean geometry molecule targets must be unique")]
    DuplicateMolecule,
    /// The immutable snapshot could not be reparsed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// A durable target could not be projected as a core molecule.
    #[error(transparent)]
    CoreProjection(#[from] CoreProjectionError),
    /// A durable molecule selector was absent from this snapshot.
    #[error("document object is not a durable molecule in this snapshot: {object_id}")]
    UnknownMolecule { object_id: String },
    /// Clean geometry requires at least one typed bond per target.
    #[error("clean geometry requires a bonded molecule: {object_id}")]
    UnbondedMolecule { object_id: String },
    /// Native chemistry returned a coordinate count that cannot match source atoms.
    #[error(
        "clean geometry for {object_id} expected {expected} generated points but received {actual}"
    )]
    GeneratedAtomCountMismatch {
        /// Durable molecule selector whose result was malformed.
        object_id: String,
        /// Source-order atom count passed through the chemistry graph.
        expected: usize,
        /// Coordinate count returned by the chemistry engine.
        actual: usize,
    },
    /// One selected molecule could not cross the exact chemistry boundary.
    #[error("cannot clean molecule {object_id}: {source}")]
    Target {
        object_id: String,
        #[source]
        source: MoleculeCoordinateBuildError,
    },
    /// Generated or placed geometry was invalid.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// The complete prepared update rejected its own shape.
    #[error(transparent)]
    Update(#[from] CleanGeometryUpdateV1Error),
}
