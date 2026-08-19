//! Closed conversion from an owned complete chemistry graph into document insertion facts.

use crate::{
    MoleculeInsertionAtomV1, MoleculeInsertionBondOrderV1, MoleculeInsertionBondV1,
    MoleculeInsertionV1, MoleculeInsertionV1Error, Point3V1, ProjectionError,
};
use ferrum_chemistry::{AtomChirality, BondDirection, BondOrder, BondStereo, MolGraph};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2, place_molecule_depiction_v1};
use thiserror::Error;

/// Convert one ABI-owned complete graph into a detached, CDML-representable insertion.
///
/// This is deliberately independent of the parser that produced `graph`. It repeats
/// document-boundary checks for complete coordinates, finite values, closed chemistry
/// facts, and persistable bond orders before it allocates a document candidate.
pub fn build_complete_graph_molecule_insertion_v1(
    graph: &MolGraph,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, CompleteGraphMoleculeInsertionError> {
    validate_supported_complete_graph_facts_v1(graph)?;
    build_complete_graph_molecule_insertion_from_validated_facts_v1(graph, placement)
}

/// Convert a graph after its parser-specific facts have been validated.
///
/// Parser coordinators may accept a source-owned representation detail that is
/// redundant with facts they persist. They must perform that narrower validation
/// before calling this helper; coordinate, aromaticity, placement, and insertion
/// validation still happen here.
pub fn build_complete_graph_molecule_insertion_from_validated_facts_v1(
    graph: &MolGraph,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, CompleteGraphMoleculeInsertionError> {
    validate_resolved_aromaticity_v1(graph)?;
    let coordinates = graph
        .coordinates()
        .ok_or(CompleteGraphMoleculeInsertionError::MissingCoordinates)?;
    if coordinates.points().len() != graph.atoms().len() {
        return Err(
            CompleteGraphMoleculeInsertionError::CoordinateCountMismatch {
                atom_count: graph.atoms().len(),
                coordinate_count: coordinates.points().len(),
            },
        );
    }
    let source_points = coordinates
        .points()
        .iter()
        .enumerate()
        .map(|(atom_index, point)| {
            Point2::new(point.x(), point.y()).map_err(|_| {
                CompleteGraphMoleculeInsertionError::NonFiniteCoordinate { atom_index }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let edges = graph
        .bonds()
        .iter()
        .map(|bond| (bond.start(), bond.end()))
        .collect::<Vec<_>>();
    let placed = place_molecule_depiction_v1(&source_points, &edges, placement)?;
    let atoms = graph
        .atoms()
        .iter()
        .zip(placed)
        .map(|(atom, position)| {
            let position = Point3V1::new(position.x(), position.y(), 0.0)?;
            MoleculeInsertionAtomV1::new(
                atom.atomic_number().symbol(),
                position,
                atom.formal_charge().filter(|value| *value != 0),
                atom.isotope(),
                atom.explicit_hydrogens().filter(|value| *value != 0),
            )
            .map_err(CompleteGraphMoleculeInsertionError::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = graph
        .bonds()
        .iter()
        .map(|bond| {
            let order = persistable_bond_order_v1(bond.order(), bond.start(), bond.end())?;
            Ok(MoleculeInsertionBondV1::new(
                bond.start(),
                bond.end(),
                order,
            ))
        })
        .collect::<Result<Vec<_>, CompleteGraphMoleculeInsertionError>>()?;
    MoleculeInsertionV1::new(atoms, bonds).map_err(CompleteGraphMoleculeInsertionError::from)
}

/// Reject chemistry facts with no exact V1 document encoding.
///
/// SMILES calls this before requesting Kekule assignment. Parser routes that do
/// not perform that resolution must call `validate_resolved_aromaticity_v1` too.
pub fn validate_supported_complete_graph_facts_v1(
    graph: &MolGraph,
) -> Result<(), CompleteGraphMoleculeInsertionError> {
    validate_supported_complete_graph_facts_with_inchi_hydrogen_policy_v1(graph, false)
}

/// Validate an InChI graph whose complete hydrogen counts make RDKit's
/// `no_implicit` bit a parser-owned representation detail.
///
/// The native InChI boundary supplies an explicit hydrogen count for every atom.
/// CDML persists each positive count; zero is the format default. Other atom and
/// bond facts remain subject to the ordinary closed insertion grammar.
pub(crate) fn validate_supported_inchi_complete_graph_facts_v1(
    graph: &MolGraph,
) -> Result<(), CompleteGraphMoleculeInsertionError> {
    validate_supported_complete_graph_facts_with_inchi_hydrogen_policy_v1(graph, true)
}

/// Validate the fixed legacy peptide-template graph representation.
///
/// This intentionally does not broaden generic SMILES insertion. The template
/// source uses bracket-derived no-implicit and tetrahedral flags as native
/// parser representation details; the legacy writer persisted the corresponding
/// ordinary achiral CDML topology, charge, isotope, and explicit-H facts.
pub fn validate_supported_peptide_template_complete_graph_facts_v1(
    graph: &MolGraph,
) -> Result<(), CompleteGraphMoleculeInsertionError> {
    for (index, atom) in graph.atoms().iter().enumerate() {
        let unsupported = if atom.chirality() == AtomChirality::Other {
            Some("unrecognized chirality")
        } else if atom.radical_electrons() != 0 {
            Some("radical electrons")
        } else if atom.no_implicit() && atom.explicit_hydrogens().is_none() {
            Some("no-implicit-hydrogen policy without an explicit hydrogen count")
        } else if atom.atom_map_number().is_some() {
            Some("atom-map number")
        } else {
            None
        };
        if let Some(fact) = unsupported {
            return Err(CompleteGraphMoleculeInsertionError::UnsupportedAtomFact {
                atom_index: index,
                fact,
            });
        }
    }
    validate_supported_complete_graph_bonds_v1(graph)
}

fn validate_supported_complete_graph_facts_with_inchi_hydrogen_policy_v1(
    graph: &MolGraph,
    inchi_hydrogens_are_complete: bool,
) -> Result<(), CompleteGraphMoleculeInsertionError> {
    for (index, atom) in graph.atoms().iter().enumerate() {
        let unsupported = if atom.chirality() != AtomChirality::Unspecified {
            Some("chirality")
        } else if atom.radical_electrons() != 0 {
            Some("radical electrons")
        } else if atom.no_implicit()
            && !(inchi_hydrogens_are_complete && atom.explicit_hydrogens().is_some())
        {
            Some("no-implicit-hydrogen policy")
        } else if atom.atom_map_number().is_some() {
            Some("atom-map number")
        } else {
            None
        };
        if let Some(fact) = unsupported {
            return Err(CompleteGraphMoleculeInsertionError::UnsupportedAtomFact {
                atom_index: index,
                fact,
            });
        }
    }
    validate_supported_complete_graph_bonds_v1(graph)
}

fn validate_supported_complete_graph_bonds_v1(
    graph: &MolGraph,
) -> Result<(), CompleteGraphMoleculeInsertionError> {
    for (index, bond) in graph.bonds().iter().enumerate() {
        let unsupported = if bond.stereo() != BondStereo::None {
            Some("stereochemistry")
        } else if bond.direction() != BondDirection::None {
            Some("drawing direction")
        } else if bond.stereo_atoms().is_some() {
            Some("stereo atom references")
        } else {
            None
        };
        if let Some(fact) = unsupported {
            return Err(CompleteGraphMoleculeInsertionError::UnsupportedBondFact {
                bond_index: index,
                fact,
            });
        }
        if bond.order() == BondOrder::Quadruple {
            return Err(CompleteGraphMoleculeInsertionError::UnsupportedBondOrder {
                start: bond.start(),
                end: bond.end(),
                order: bond.order(),
            });
        }
    }
    Ok(())
}

/// Reject aromatic flags that have not been converted to exact Kekule facts.
pub(crate) fn validate_resolved_aromaticity_v1(
    graph: &MolGraph,
) -> Result<(), CompleteGraphMoleculeInsertionError> {
    for (index, atom) in graph.atoms().iter().enumerate() {
        if atom.is_aromatic() {
            return Err(CompleteGraphMoleculeInsertionError::UnsupportedAtomFact {
                atom_index: index,
                fact: "unresolved aromaticity",
            });
        }
    }
    for (index, bond) in graph.bonds().iter().enumerate() {
        if bond.is_aromatic() {
            return Err(CompleteGraphMoleculeInsertionError::UnsupportedBondFact {
                bond_index: index,
                fact: "unresolved aromaticity",
            });
        }
    }
    Ok(())
}

fn persistable_bond_order_v1(
    order: BondOrder,
    start: usize,
    end: usize,
) -> Result<MoleculeInsertionBondOrderV1, CompleteGraphMoleculeInsertionError> {
    match order {
        BondOrder::Single => Ok(MoleculeInsertionBondOrderV1::Single),
        BondOrder::Double => Ok(MoleculeInsertionBondOrderV1::Double),
        BondOrder::Triple => Ok(MoleculeInsertionBondOrderV1::Triple),
        BondOrder::Aromatic | BondOrder::Quadruple => {
            Err(CompleteGraphMoleculeInsertionError::UnsupportedBondOrder { start, end, order })
        }
    }
}

/// Failure while converting an already-owned complete graph for document insertion.
#[derive(Debug, Error)]
pub enum CompleteGraphMoleculeInsertionError {
    /// The graph omitted the complete 2D coordinate set required for placement.
    #[error("complete molecule has no complete 2D coordinate set")]
    MissingCoordinates,
    /// The graph's coordinate list no longer aligns with its atom list.
    #[error("complete molecule has {coordinate_count} coordinates for {atom_count} atoms")]
    CoordinateCountMismatch {
        atom_count: usize,
        coordinate_count: usize,
    },
    /// A source coordinate is not finite at the document conversion boundary.
    #[error("complete molecule coordinate for atom {atom_index} is not finite")]
    NonFiniteCoordinate { atom_index: usize },
    /// Placement could not produce finite, nondegenerate document geometry.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// A finite document point unexpectedly failed projection validation.
    #[error(transparent)]
    Position(#[from] ProjectionError),
    /// The closed insertion graph rejected a converted chemistry fact.
    #[error(transparent)]
    Insertion(#[from] MoleculeInsertionV1Error),
    /// One atom carries a fact without a proven V1 insertion encoding.
    #[error("complete molecule atom {atom_index} has {fact}, which V1 insertion cannot encode yet")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    /// One bond carries a fact without a proven V1 insertion encoding.
    #[error(
        "complete molecule bond {bond_index} has {fact}, which V1 insertion cannot \
         encode yet"
    )]
    UnsupportedBondFact {
        bond_index: usize,
        fact: &'static str,
    },
    /// This V1 writer has not established an exact encoding for the bond order.
    #[error(
        "complete molecule bond {start}-{end} has {order:?} order, which V1 insertion cannot \
         encode yet"
    )]
    UnsupportedBondOrder {
        start: usize,
        end: usize,
        order: BondOrder,
    },
}

#[cfg(test)]
#[path = "complete_graph_molecule_insertion_v1_tests.rs"]
mod tests;
