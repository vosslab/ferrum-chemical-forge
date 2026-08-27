//! Closed conversion from an owned complete chemistry graph into document insertion facts.

use super::{
    DocumentDirectedBondDepictionV1, DocumentDoubleBondCarrierMarkDepictionV1,
    DocumentDoubleBondCarrierMarkV1, DocumentDoubleBondConfigurationV1, DocumentDoubleBondStereoV1,
    DocumentStereoDepictionReportV1, DocumentStereoLigandV1, DocumentStereoSemanticReportV1,
    DocumentTetrahedralParityV1, DocumentTetrahedralStereoV1, PreparedDocumentMoleculeV2,
};
use crate::{
    DocumentBondOrderV1, DocumentBondPresentationV1, MoleculeInsertionAtomV1,
    MoleculeInsertionBondV1, MoleculeInsertionV1, MoleculeInsertionV1Error, Point3V1,
    ProjectionError,
};
use ferrum_chemistry::{
    AtomChirality, BondDirection, BondOrder, BondStereo, ChemEngine, KekulizeOptions, MolGraph,
};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2, place_molecule_depiction_v1};
use thiserror::Error;

/// Resolve, validate, and prepare one complete graph without mutating a document session.
pub fn prepare_complete_graph_for_document_v2<E: ChemEngine + ?Sized>(
    engine: &E,
    graph: &MolGraph,
    placement: MoleculePlacementV1,
) -> Result<PreparedDocumentMoleculeV2, DocumentMoleculePreparationErrorV2> {
    let options = KekulizeOptions::new(true, true, 100)
        .expect("the approved V2 kekulization options are statically valid");
    let resolved = if graph
        .atoms()
        .iter()
        .any(ferrum_chemistry::MolAtom::is_aromatic)
        || graph
            .bonds()
            .iter()
            .any(ferrum_chemistry::MolBond::is_aromatic)
    {
        engine
            .kekulize(graph, options)
            .map_err(|_| DocumentMoleculePreparationErrorV2::AromaticityResolutionFailed)?
    } else {
        graph.clone()
    };
    validate_resolved_aromaticity_v2(&resolved)?;
    validate_document_atom_facts_v2(&resolved)?;

    let tetrahedral = admit_tetrahedral_stereo_v2(&resolved)?;
    let directed_depictions = admit_directed_bond_depictions_v2(&resolved)?;
    let (double_bonds, double_bond_carrier_marks) = admit_double_bond_stereo_v2(&resolved)?;
    validate_native_stereo_directions_v2(
        &resolved,
        &directed_depictions,
        &double_bond_carrier_marks,
    )?;
    let insertion = build_document_insertion_v2(&resolved, placement, &directed_depictions)?;
    let semantics = DocumentStereoSemanticReportV1::new(tetrahedral, double_bonds);
    let depictions =
        DocumentStereoDepictionReportV1::new(directed_depictions, double_bond_carrier_marks);
    let semantics = (!semantics.is_empty()).then_some(semantics);
    let depictions = (!depictions.is_empty()).then_some(depictions);
    PreparedDocumentMoleculeV2::with_stereo_reports(insertion, semantics, depictions)
        .map_err(|_| DocumentMoleculePreparationErrorV2::InvalidStereoSemantics)
}

fn validate_document_atom_facts_v2(
    graph: &MolGraph,
) -> Result<(), DocumentMoleculePreparationErrorV2> {
    for (atom_index, atom) in graph.atoms().iter().enumerate() {
        let fact = unsupported_document_atom_fact_v2(
            atom.chirality(),
            atom.radical_electrons(),
            atom.no_implicit(),
            atom.explicit_hydrogens(),
            atom.atom_map_number(),
        );
        if let Some(fact) = fact {
            return Err(DocumentMoleculePreparationErrorV2::UnsupportedAtomFact {
                atom_index,
                fact,
            });
        }
    }
    Ok(())
}

fn unsupported_document_atom_fact_v2(
    chirality: AtomChirality,
    radical_electrons: u8,
    no_implicit: bool,
    explicit_hydrogens: Option<u16>,
    atom_map_number: Option<u32>,
) -> Option<&'static str> {
    if chirality == AtomChirality::Other {
        Some("chirality")
    } else if radical_electrons != 0 {
        Some("radical electrons")
    } else if no_implicit && explicit_hydrogens.is_none() {
        Some("no-implicit-hydrogen policy")
    } else if atom_map_number.is_some() {
        Some("atom-map number")
    } else {
        None
    }
}

fn admit_tetrahedral_stereo_v2(
    graph: &MolGraph,
) -> Result<Vec<DocumentTetrahedralStereoV1>, DocumentMoleculePreparationErrorV2> {
    let mut tetrahedral = Vec::new();
    for (center, atom) in graph.atoms().iter().enumerate() {
        let parity = match atom.chirality() {
            AtomChirality::Unspecified => continue,
            AtomChirality::TetrahedralCw => DocumentTetrahedralParityV1::Clockwise,
            AtomChirality::TetrahedralCcw => DocumentTetrahedralParityV1::CounterClockwise,
            AtomChirality::Other => unreachable!("unsupported chirality was rejected first"),
        };
        let mut ligands = graph
            .bonds()
            .iter()
            .filter_map(|bond| {
                if bond.start() == center {
                    Some(DocumentStereoLigandV1::Atom(bond.end()))
                } else if bond.end() == center {
                    Some(DocumentStereoLigandV1::Atom(bond.start()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        match atom.explicit_hydrogens() {
            Some(1) => ligands.push(DocumentStereoLigandV1::ExplicitHydrogen),
            Some(0) | None => {}
            Some(_) => {
                return Err(
                    DocumentMoleculePreparationErrorV2::UnrepresentableTetrahedral { center },
                );
            }
        }
        ligands.sort_by_key(|ligand| match ligand {
            DocumentStereoLigandV1::Atom(index) => (0, *index),
            DocumentStereoLigandV1::ExplicitHydrogen => (1, 0),
        });
        let ligands: [DocumentStereoLigandV1; 4] = ligands.try_into().map_err(|_| {
            DocumentMoleculePreparationErrorV2::UnrepresentableTetrahedral { center }
        })?;
        let descriptor = DocumentTetrahedralStereoV1::new(center, ligands, parity)?;
        tetrahedral.push(descriptor);
    }
    Ok(tetrahedral)
}

fn admit_directed_bond_depictions_v2(
    graph: &MolGraph,
) -> Result<Vec<DocumentDirectedBondDepictionV1>, DocumentMoleculePreparationErrorV2> {
    let depictions = graph
        .bonds()
        .iter()
        .enumerate()
        .filter_map(|(bond_index, bond)| {
            matches!(
                bond.direction(),
                BondDirection::BeginWedge | BondDirection::BeginDash
            )
            .then_some((bond_index, bond))
        })
        .map(|(bond_index, bond)| {
            let presentation = match bond.direction() {
                BondDirection::BeginWedge => DocumentBondPresentationV1::SolidWedge,
                BondDirection::BeginDash => DocumentBondPresentationV1::HashedWedge,
                _ => unreachable!("the enclosing filter admits only authored directed depictions"),
            };
            DocumentDirectedBondDepictionV1::new(bond_index, bond.start(), bond.end(), presentation)
        })
        .collect();
    Ok(depictions)
}

fn admit_double_bond_stereo_v2(
    graph: &MolGraph,
) -> Result<
    (
        Vec<DocumentDoubleBondStereoV1>,
        Vec<DocumentDoubleBondCarrierMarkDepictionV1>,
    ),
    DocumentMoleculePreparationErrorV2,
> {
    let mut descriptors = Vec::new();
    let mut carrier_marks = Vec::new();
    for (bond_index, bond) in graph.bonds().iter().enumerate() {
        match bond.stereo() {
            BondStereo::None => {
                if bond.stereo_atoms().is_some() {
                    return Err(DocumentMoleculePreparationErrorV2::InvalidStereoReference {
                        bond_index,
                    });
                }
            }
            BondStereo::E | BondStereo::Z | BondStereo::Cis | BondStereo::Trans => {
                if bond.order() != BondOrder::Double {
                    return Err(
                        DocumentMoleculePreparationErrorV2::UnrepresentableDoubleBondStereo {
                            bond_index,
                        },
                    );
                }
                let (start_ligand, end_ligand) = bond.stereo_atoms().ok_or(
                    DocumentMoleculePreparationErrorV2::InvalidStereoReference { bond_index },
                )?;
                if !valid_double_bond_reference(graph, bond, start_ligand, true)
                    || !valid_double_bond_reference(graph, bond, end_ligand, false)
                {
                    return Err(DocumentMoleculePreparationErrorV2::InvalidStereoReference {
                        bond_index,
                    });
                }
                let start_carrier = bond_index_between_v2(graph, bond.start(), start_ligand)
                    .expect("validated double-bond start ligand has a carrier bond");
                let end_carrier = bond_index_between_v2(graph, bond.end(), end_ligand)
                    .expect("validated double-bond end ligand has a carrier bond");
                let configuration = document_double_bond_configuration_v2(bond.stereo())
                    .expect("the enclosing match is limited to representable double-bond stereo");
                descriptors.push(DocumentDoubleBondStereoV1::new(
                    bond_index,
                    start_ligand,
                    end_ligand,
                    configuration,
                )?);
                let admitted_marks = admit_double_bond_carrier_marks_v2(
                    bond_index,
                    [
                        (start_carrier, &graph.bonds()[start_carrier]),
                        (end_carrier, &graph.bonds()[end_carrier]),
                    ],
                )?;
                carrier_marks.extend(require_double_bond_carrier_marks_v2(
                    bond_index,
                    admitted_marks,
                )?);
            }
            BondStereo::Any | BondStereo::Other => {
                return Err(DocumentMoleculePreparationErrorV2::UnsupportedStereoClass {
                    bond_index,
                });
            }
        }
    }
    Ok((descriptors, carrier_marks))
}

fn admit_double_bond_carrier_marks_v2(
    double_bond_index: usize,
    carriers: [(usize, &ferrum_chemistry::MolBond); 2],
) -> Result<Vec<DocumentDoubleBondCarrierMarkDepictionV1>, DocumentMoleculePreparationErrorV2> {
    let mut marks = Vec::new();
    for (carrier_bond_index, carrier) in carriers {
        match carrier.direction() {
            BondDirection::None => {}
            direction if native_ez_direction_is_carrier_v2(direction) => {
                if carrier.order() != BondOrder::Single {
                    return Err(
                        DocumentMoleculePreparationErrorV2::UnrepresentableDoubleBondDepiction {
                            bond_index: double_bond_index,
                        },
                    );
                }
                marks.push(DocumentDoubleBondCarrierMarkDepictionV1::new(
                    double_bond_index,
                    carrier_bond_index,
                    document_double_bond_carrier_mark_v2(direction)
                        .expect("the preceding direction admission is exhaustive"),
                ));
            }
            _ => {
                return Err(DocumentMoleculePreparationErrorV2::UnsupportedStereoClass {
                    bond_index: carrier_bond_index,
                });
            }
        }
    }
    Ok(marks)
}

fn require_double_bond_carrier_marks_v2(
    bond_index: usize,
    marks: Vec<DocumentDoubleBondCarrierMarkDepictionV1>,
) -> Result<Vec<DocumentDoubleBondCarrierMarkDepictionV1>, DocumentMoleculePreparationErrorV2> {
    if marks.is_empty() {
        return Err(
            DocumentMoleculePreparationErrorV2::UnrepresentableDoubleBondDepiction { bond_index },
        );
    }
    Ok(marks)
}

fn native_ez_direction_is_carrier_v2(direction: BondDirection) -> bool {
    matches!(
        direction,
        BondDirection::EndUpRight | BondDirection::EndDownRight
    )
}

fn document_double_bond_carrier_mark_v2(
    direction: BondDirection,
) -> Option<DocumentDoubleBondCarrierMarkV1> {
    match direction {
        BondDirection::EndUpRight => Some(DocumentDoubleBondCarrierMarkV1::Up),
        BondDirection::EndDownRight => Some(DocumentDoubleBondCarrierMarkV1::Down),
        BondDirection::None
        | BondDirection::BeginWedge
        | BondDirection::BeginDash
        | BondDirection::Other => None,
    }
}

fn validate_native_stereo_directions_v2(
    graph: &MolGraph,
    directed_depictions: &[DocumentDirectedBondDepictionV1],
    double_bond_carrier_marks: &[DocumentDoubleBondCarrierMarkDepictionV1],
) -> Result<(), DocumentMoleculePreparationErrorV2> {
    for (bond_index, bond) in graph.bonds().iter().enumerate() {
        let admitted = match bond.direction() {
            BondDirection::None => true,
            BondDirection::BeginWedge | BondDirection::BeginDash => directed_depictions
                .iter()
                .any(|depiction| depiction.bond_index() == bond_index),
            BondDirection::EndUpRight | BondDirection::EndDownRight => double_bond_carrier_marks
                .iter()
                .any(|mark| mark.carrier_bond_index() == bond_index),
            BondDirection::Other => false,
        };
        if !admitted {
            return Err(DocumentMoleculePreparationErrorV2::UnsupportedStereoClass { bond_index });
        }
    }
    Ok(())
}

fn bond_index_between_v2(graph: &MolGraph, first: usize, second: usize) -> Option<usize> {
    graph.bonds().iter().position(|candidate| {
        (candidate.start() == first && candidate.end() == second)
            || (candidate.start() == second && candidate.end() == first)
    })
}

fn document_double_bond_configuration_v2(
    stereo: BondStereo,
) -> Option<DocumentDoubleBondConfigurationV1> {
    match stereo {
        BondStereo::E | BondStereo::Trans => Some(DocumentDoubleBondConfigurationV1::E),
        BondStereo::Z | BondStereo::Cis => Some(DocumentDoubleBondConfigurationV1::Z),
        BondStereo::None | BondStereo::Any | BondStereo::Other => None,
    }
}

fn valid_double_bond_reference(
    graph: &MolGraph,
    bond: &ferrum_chemistry::MolBond,
    ligand: usize,
    start: bool,
) -> bool {
    let endpoint = if start { bond.start() } else { bond.end() };
    ligand < graph.atoms().len()
        && ligand != bond.start()
        && ligand != bond.end()
        && graph.bonds().iter().any(|candidate| {
            (candidate.start() == endpoint && candidate.end() == ligand)
                || (candidate.end() == endpoint && candidate.start() == ligand)
        })
}

fn validate_resolved_aromaticity_v2(
    graph: &MolGraph,
) -> Result<(), DocumentMoleculePreparationErrorV2> {
    if graph.atoms().iter().any(|atom| atom.is_aromatic())
        || graph.bonds().iter().any(|bond| bond.is_aromatic())
    {
        return Err(DocumentMoleculePreparationErrorV2::AromaticityResolutionFailed);
    }
    Ok(())
}

fn build_document_insertion_v2(
    graph: &MolGraph,
    placement: MoleculePlacementV1,
    directed_depictions: &[DocumentDirectedBondDepictionV1],
) -> Result<MoleculeInsertionV1, DocumentMoleculePreparationErrorV2> {
    let coordinates = graph
        .coordinates()
        .ok_or(DocumentMoleculePreparationErrorV2::MissingCoordinates)?;
    if coordinates.points().len() != graph.atoms().len() {
        return Err(
            DocumentMoleculePreparationErrorV2::CoordinateCountMismatch {
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
            Point2::new(point.x(), point.y())
                .map_err(|_| DocumentMoleculePreparationErrorV2::NonFiniteCoordinate { atom_index })
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
            .map_err(DocumentMoleculePreparationErrorV2::from)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = graph
        .bonds()
        .iter()
        .enumerate()
        .map(|(bond_index, bond)| {
            let order = persistable_bond_order_v2(bond.order(), bond.start(), bond.end())?;
            if let Some(depiction) = directed_depictions
                .iter()
                .find(|depiction| depiction.bond_index() == bond_index)
            {
                Ok(MoleculeInsertionBondV1::new_with_presentation(
                    depiction.endpoints().0,
                    depiction.endpoints().1,
                    depiction.presentation(),
                ))
            } else {
                Ok(MoleculeInsertionBondV1::new(
                    bond.start(),
                    bond.end(),
                    order,
                ))
            }
        })
        .collect::<Result<Vec<_>, DocumentMoleculePreparationErrorV2>>()?;
    MoleculeInsertionV1::new(atoms, bonds).map_err(DocumentMoleculePreparationErrorV2::from)
}

fn persistable_bond_order_v2(
    order: BondOrder,
    start: usize,
    end: usize,
) -> Result<DocumentBondOrderV1, DocumentMoleculePreparationErrorV2> {
    match order {
        BondOrder::Single => Ok(DocumentBondOrderV1::Single),
        BondOrder::Double => Ok(DocumentBondOrderV1::Double),
        BondOrder::Triple => Ok(DocumentBondOrderV1::Triple),
        BondOrder::Aromatic | BondOrder::Quadruple => {
            Err(DocumentMoleculePreparationErrorV2::UnsupportedBondOrder { start, end, order })
        }
    }
}

/// Closed refusal categories for detached V2 document molecule preparation.
#[derive(Debug, Error)]
pub enum DocumentMoleculePreparationErrorV2 {
    /// The chemistry engine did not return a durable non-aromatic graph.
    #[error("aromaticity resolution failed")]
    AromaticityResolutionFailed,
    /// A source stereo reference is missing, out of range, or not adjacent as required.
    #[error("stereo references for bond {bond_index} are invalid")]
    InvalidStereoReference { bond_index: usize },
    /// A prepared descriptor does not match the resulting document graph.
    #[error("prepared stereo semantics do not match the document graph")]
    InvalidStereoSemantics,
    /// A tetrahedral source fact cannot be represented exactly.
    #[error("tetrahedral stereo at atom {center} cannot be represented")]
    UnrepresentableTetrahedral { center: usize },
    /// An E/Z source fact lacks an exact durable double-bond descriptor.
    #[error("double-bond stereo at bond {bond_index} cannot be represented")]
    UnrepresentableDoubleBondStereo { bond_index: usize },
    /// An E/Z source fact lacks a native directional single-bond carrier for drawing.
    #[error("double-bond depiction at bond {bond_index} cannot be represented")]
    UnrepresentableDoubleBondDepiction { bond_index: usize },
    /// The source requests a stereo class outside the approved P0 slice.
    #[error("stereo class on bond {bond_index} is unsupported")]
    UnsupportedStereoClass { bond_index: usize },
    /// The graph omitted the complete 2D coordinate set required for placement.
    #[error("complete molecule has no complete 2D coordinate set")]
    MissingCoordinates,
    /// The graph coordinate list no longer aligns with its atom list.
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
    /// One atom carries a fact without a proven document encoding.
    #[error("complete molecule atom {atom_index} has unsupported {fact}")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    /// This document writer has not established an exact encoding for the bond order.
    #[error("complete molecule bond {start}-{end} has unsupported {order:?} order")]
    UnsupportedBondOrder {
        start: usize,
        end: usize,
        order: BondOrder,
    },
}

#[cfg(test)]
#[path = "complete_graph_document_preparation_tests.rs"]
mod tests;
