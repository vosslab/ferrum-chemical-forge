//! Rust-owned existing-molecule coordinate regeneration.

use ferrum_chemistry::{ChemEngine, ChemistryError, MolGraph, MolGraphError};
use ferrum_core::Molecule;
use ferrum_document::{
    CoreProjectionError, DocumentObjectIdV1, MoleculeCoordinateUpdateV1,
    MoleculeCoordinateUpdateV1Error, Point3V1, ProjectionError, SessionDocumentObservationV1,
    TypedDocument, TypedDocumentError,
};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2, place_molecule_depiction_v1};
use thiserror::Error;

use crate::document_molecule_graph_v1::{DocumentMoleculeGraphError, document_molecule_graph_v1};

/// Regenerate one molecule while preserving its current on-page centroid and scale.
///
/// The chemistry engine receives no existing coordinates. Ferrum derives placement
/// only after generation: the destination centroid is the mean of the current atom
/// positions, and a bonded molecule retains its current mean bond length. A
/// bondless molecule is translated without scaling.
pub fn build_molecule_coordinate_update_v1<E: ChemEngine>(
    engine: &E,
    observation: &SessionDocumentObservationV1,
    molecule_id: &DocumentObjectIdV1,
) -> Result<MoleculeCoordinateUpdateV1, MoleculeCoordinateBuildError> {
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    let molecule = document.core_molecule(molecule_id)?.ok_or_else(|| {
        MoleculeCoordinateBuildError::UnknownMolecule {
            object_id: molecule_id.as_str().to_owned(),
        }
    })?;
    let (graph, current_points, edges, z_values) = coordinate_graph(&molecule)?;
    let generated = engine.generate_2d_coordinates(&graph)?;
    let generated_points = generated
        .points()
        .iter()
        .map(|point| Point2::new(point.x(), point.y()))
        .collect::<Result<Vec<_>, _>>()?;
    let placement = retained_placement(&current_points, &edges)?;
    let placed = place_molecule_depiction_v1(&generated_points, &edges, placement)?;
    let positions = placed
        .into_iter()
        .zip(z_values)
        .map(|(point, z)| Point3V1::new(point.x(), point.y(), z))
        .collect::<Result<Vec<_>, _>>()?;
    MoleculeCoordinateUpdateV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        molecule_id.clone(),
        positions,
    )
    .map_err(Into::into)
}

pub(crate) type CoordinateGraphFacts = (MolGraph, Vec<Point2>, Vec<(usize, usize)>, Vec<f64>);

pub(crate) fn coordinate_graph(
    molecule: &Molecule,
) -> Result<CoordinateGraphFacts, MoleculeCoordinateBuildError> {
    let (graph, edges) = document_molecule_graph_v1(molecule)
        .map_err(MoleculeCoordinateBuildError::from)?
        .into_parts();
    let current_points = molecule
        .atoms()
        .iter()
        .map(|atom| Point2::new(atom.position().x(), atom.position().y()))
        .collect::<Result<Vec<_>, _>>()?;
    let z_values = molecule
        .atoms()
        .iter()
        .map(|atom| atom.position().z())
        .collect();
    Ok((graph, current_points, edges, z_values))
}

fn retained_placement(
    points: &[Point2],
    edges: &[(usize, usize)],
) -> Result<MoleculePlacementV1, MoleculeCoordinateBuildError> {
    let count = points.len() as f64;
    let anchor = Point2::new(
        points.iter().map(|point| point.x()).sum::<f64>() / count,
        points.iter().map(|point| point.y()).sum::<f64>() / count,
    )?;
    let bond_length = if edges.is_empty() {
        // The placement algorithm deliberately ignores scale for a bondless graph.
        1.0
    } else {
        let total = edges.iter().try_fold(0.0, |sum, &(start, end)| {
            let first = points
                .get(start)
                .ok_or(GeometryError::BondIndexOutOfBounds {
                    index: start,
                    len: points.len(),
                })?;
            let second = points.get(end).ok_or(GeometryError::BondIndexOutOfBounds {
                index: end,
                len: points.len(),
            })?;
            Ok::<_, GeometryError>(sum + first.distance_to(*second))
        })? / edges.len() as f64;
        if !total.is_finite() || total <= 0.0 {
            return Err(MoleculeCoordinateBuildError::NoUsableBondLength);
        }
        total
    };
    MoleculePlacementV1::new(bond_length, anchor).map_err(Into::into)
}

/// Failure while preparing a complete existing-molecule coordinate update.
#[derive(Debug, Error)]
pub enum MoleculeCoordinateBuildError {
    /// The immutable snapshot could not be reparsed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// Typed chemistry projection rejected source facts.
    #[error(transparent)]
    CoreProjection(#[from] CoreProjectionError),
    /// The durable selector was not a molecule in this snapshot.
    #[error("document object is not a durable molecule in this snapshot: {object_id}")]
    UnknownMolecule { object_id: String },
    /// Coordinate generation requires at least one atom.
    #[error("coordinate generation requires a molecule with at least one atom")]
    EmptyMolecule,
    /// The chemistry graph cannot silently discard a typed non-atom vertex.
    #[error("coordinate generation does not yet support {count} {kind} vertices")]
    UnsupportedVertex { kind: &'static str, count: usize },
    /// The retained graph unexpectedly repeated an atom identity.
    #[error("atom {atom_index} repeats an earlier durable identity")]
    DuplicateAtomIdentity { atom_index: usize },
    /// An atom omitted its required chemical element.
    #[error("atom {atom_index} has no element for coordinate generation")]
    MissingElement { atom_index: usize },
    /// An element spelling is outside the native engine's exact element domain.
    #[error("atom {atom_index} element {element:?} is not supported: {source}")]
    InvalidElement {
        atom_index: usize,
        element: String,
        #[source]
        source: MolGraphError,
    },
    /// An authored atom fact has no exact coordinate-generation mapping yet.
    #[error("atom {atom_index} has unsupported {fact}")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    /// A bond endpoint is not an ordinary atom in the selected molecule.
    #[error("bond {bond_index} has a non-atom or unresolved endpoint")]
    UnsupportedBondEndpoint { bond_index: usize },
    /// A drawing-specific bond style cannot be regenerated without losing meaning.
    #[error("bond {bond_index} has a drawing style not supported by coordinate regeneration")]
    UnsupportedBondStyle { bond_index: usize },
    /// A bond order has no exact V1 coordinate-generation mapping.
    #[error("bond {bond_index} has an unsupported or absent bond order")]
    UnsupportedBondOrder { bond_index: usize },
    /// The selected graph facts violated chemistry invariants.
    #[error(transparent)]
    Graph(#[from] MolGraphError),
    /// Native coordinate generation failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// Existing or generated geometry could not be placed safely.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// A bonded molecule had no positive current scale to preserve.
    #[error("the molecule's current bonded geometry has no positive mean bond length")]
    NoUsableBondLength,
    /// Converted document positions violated projection invariants.
    #[error(transparent)]
    Position(#[from] ProjectionError),
    /// The complete update rejected its own shape.
    #[error(transparent)]
    Update(#[from] MoleculeCoordinateUpdateV1Error),
    /// Exact owned graph conversion storage could not be allocated.
    #[error("coordinate-generation graph could not reserve owned storage")]
    ResourceAllocation,
}

impl From<DocumentMoleculeGraphError> for MoleculeCoordinateBuildError {
    fn from(error: DocumentMoleculeGraphError) -> Self {
        match error {
            DocumentMoleculeGraphError::EmptyMolecule => Self::EmptyMolecule,
            DocumentMoleculeGraphError::UnsupportedVertex { kind, count } => {
                Self::UnsupportedVertex { kind, count }
            }
            DocumentMoleculeGraphError::DuplicateAtomIdentity { atom_index } => {
                Self::DuplicateAtomIdentity { atom_index }
            }
            DocumentMoleculeGraphError::MissingElement { atom_index } => {
                Self::MissingElement { atom_index }
            }
            DocumentMoleculeGraphError::InvalidElement {
                atom_index,
                element,
                source,
            } => Self::InvalidElement {
                atom_index,
                element,
                source,
            },
            DocumentMoleculeGraphError::UnsupportedAtomFact { atom_index, fact } => {
                Self::UnsupportedAtomFact { atom_index, fact }
            }
            DocumentMoleculeGraphError::UnsupportedBondEndpoint { bond_index } => {
                Self::UnsupportedBondEndpoint { bond_index }
            }
            DocumentMoleculeGraphError::UnsupportedBondStyle { bond_index } => {
                Self::UnsupportedBondStyle { bond_index }
            }
            DocumentMoleculeGraphError::UnsupportedBondOrder { bond_index } => {
                Self::UnsupportedBondOrder { bond_index }
            }
            DocumentMoleculeGraphError::Graph(source) => Self::Graph(source),
            DocumentMoleculeGraphError::ResourceAllocation => Self::ResourceAllocation,
        }
    }
}
