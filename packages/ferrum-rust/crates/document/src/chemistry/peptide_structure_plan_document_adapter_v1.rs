//! Typed peptide-plan conversion into a detached document molecule candidate.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_chemistry::{
    AtomicNumber, BondOrder, ChemEngine, ChemistryError, KekulizeOptions, MolAtom, MolBond,
    MolGraph, MolGraphError,
};
use ferrum_domain::peptide::structure_plan_v1::{
    PeptideAtomElementV1, PeptideAtomIdV1, PeptideAtomStereochemistryV1, PeptideBondOrderV1,
    PeptideFormalChargeV1, PeptideStructurePlanV1,
};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2, place_molecule_depiction_v1};
use thiserror::Error;

use crate::{
    DocumentBondOrderV1, DocumentStereoLigandV1, DocumentStereoSemanticReportV1,
    DocumentStereoSemanticsErrorV1, DocumentTetrahedralParityV1, DocumentTetrahedralStereoV1,
    MoleculeInsertionAtomV1, MoleculeInsertionBondV1, MoleculeInsertionV1,
    MoleculeInsertionV1Error, Point3V1, PreparedDocumentMoleculeV2, ProjectionError,
};

/// Prepare one immutable peptide structure plan without constructing a source string.
pub fn prepare_peptide_structure_plan_for_document_v1<E: ChemEngine + ?Sized>(
    engine: &E,
    plan: &PeptideStructurePlanV1,
    placement: MoleculePlacementV1,
) -> Result<PreparedDocumentMoleculeV2, PeptideStructurePlanDocumentPreparationErrorV1> {
    let aromatic_atoms = plan
        .bonds()
        .iter()
        .filter(|bond| bond.order() == PeptideBondOrderV1::Aromatic)
        .flat_map(|bond| [bond.start(), bond.end()])
        .collect::<BTreeSet<_>>();
    let (atoms, atom_indexes) = plan_atoms(plan, &aromatic_atoms)?;
    let bonds = plan_bonds(plan, &atom_indexes)?;
    let graph = MolGraph::new(atoms.clone(), bonds.clone(), None)?;
    let graph = if aromatic_atoms.is_empty() {
        graph
    } else {
        let options = KekulizeOptions::new(true, true, 100)
            .expect("approved peptide kekulization options are statically valid");
        engine.kekulize(&graph, options)?
    };
    let coordinates = engine.generate_2d_coordinates(&graph)?;
    let graph = MolGraph::new(
        graph.atoms().to_vec(),
        graph.bonds().to_vec(),
        Some(coordinates),
    )?;
    let insertion = document_insertion(&graph, placement)?;
    let stereo = tetrahedral_semantics(plan, &atom_indexes, &graph)?;
    PreparedDocumentMoleculeV2::with_stereo_semantics(insertion, stereo).map_err(Into::into)
}

fn plan_atoms(
    plan: &PeptideStructurePlanV1,
    aromatic_atoms: &BTreeSet<PeptideAtomIdV1>,
) -> Result<
    (Vec<MolAtom>, BTreeMap<PeptideAtomIdV1, usize>),
    PeptideStructurePlanDocumentPreparationErrorV1,
> {
    let mut atoms = Vec::new();
    let mut indexes = BTreeMap::new();
    atoms
        .try_reserve(plan.atoms().len())
        .map_err(|_| PeptideStructurePlanDocumentPreparationErrorV1::ResourceAllocation)?;
    for atom in plan.atoms() {
        let index = atoms.len();
        if indexes.insert(atom.id(), index).is_some() {
            return Err(
                PeptideStructurePlanDocumentPreparationErrorV1::DuplicateAtom { atom: atom.id() },
            );
        }
        atoms.push(MolAtom::new(
            atomic_number(atom.element())?,
            formal_charge(atom.formal_charge()),
            None,
            explicit_hydrogens(atom.stereochemistry()),
            aromatic_atoms.contains(&atom.id()),
        )?);
    }
    Ok((atoms, indexes))
}

fn plan_bonds(
    plan: &PeptideStructurePlanV1,
    indexes: &BTreeMap<PeptideAtomIdV1, usize>,
) -> Result<Vec<MolBond>, PeptideStructurePlanDocumentPreparationErrorV1> {
    let mut bonds = Vec::new();
    bonds
        .try_reserve(plan.bonds().len())
        .map_err(|_| PeptideStructurePlanDocumentPreparationErrorV1::ResourceAllocation)?;
    for bond in plan.bonds() {
        let start = *indexes.get(&bond.start()).ok_or(
            PeptideStructurePlanDocumentPreparationErrorV1::UnknownBondEndpoint {
                bond: bond.id().zero_based(),
            },
        )?;
        let end = *indexes.get(&bond.end()).ok_or(
            PeptideStructurePlanDocumentPreparationErrorV1::UnknownBondEndpoint {
                bond: bond.id().zero_based(),
            },
        )?;
        bonds.push(MolBond::new(start, end, bond_order(bond.order()), false));
    }
    Ok(bonds)
}

fn document_insertion(
    graph: &MolGraph,
    placement: MoleculePlacementV1,
) -> Result<MoleculeInsertionV1, PeptideStructurePlanDocumentPreparationErrorV1> {
    let coordinates = graph
        .coordinates()
        .ok_or(PeptideStructurePlanDocumentPreparationErrorV1::MissingGeneratedCoordinates)?;
    let points = coordinates
        .points()
        .iter()
        .map(|point| Point2::new(point.x(), point.y()))
        .collect::<Result<Vec<_>, _>>()?;
    let edges = graph
        .bonds()
        .iter()
        .map(|bond| (bond.start(), bond.end()))
        .collect::<Vec<_>>();
    let placed = place_molecule_depiction_v1(&points, &edges, placement)?;
    let atoms = graph
        .atoms()
        .iter()
        .zip(placed)
        .map(|(atom, point)| {
            Ok::<_, PeptideStructurePlanDocumentPreparationErrorV1>(MoleculeInsertionAtomV1::new(
                atom.atomic_number().symbol(),
                Point3V1::new(point.x(), point.y(), 0.0)?,
                atom.formal_charge().filter(|charge| *charge != 0),
                atom.isotope(),
                atom.explicit_hydrogens().filter(|count| *count != 0),
            )?)
        })
        .collect::<Result<Vec<_>, PeptideStructurePlanDocumentPreparationErrorV1>>()?;
    let bonds = graph
        .bonds()
        .iter()
        .enumerate()
        .map(|(index, bond)| {
            Ok(MoleculeInsertionBondV1::new(
                bond.start(),
                bond.end(),
                document_bond_order(bond.order()).ok_or(
                    PeptideStructurePlanDocumentPreparationErrorV1::UnsupportedBondOrder {
                        bond: index,
                        order: bond.order(),
                    },
                )?,
            ))
        })
        .collect::<Result<Vec<_>, PeptideStructurePlanDocumentPreparationErrorV1>>()?;
    MoleculeInsertionV1::new(atoms, bonds).map_err(Into::into)
}

fn tetrahedral_semantics(
    plan: &PeptideStructurePlanV1,
    indexes: &BTreeMap<PeptideAtomIdV1, usize>,
    graph: &MolGraph,
) -> Result<DocumentStereoSemanticReportV1, PeptideStructurePlanDocumentPreparationErrorV1> {
    let mut tetrahedral = Vec::new();
    for atom in plan.atoms() {
        let parity = match atom.stereochemistry() {
            PeptideAtomStereochemistryV1::Unspecified => continue,
            PeptideAtomStereochemistryV1::TetrahedralS => {
                DocumentTetrahedralParityV1::CounterClockwise
            }
            PeptideAtomStereochemistryV1::TetrahedralR => DocumentTetrahedralParityV1::Clockwise,
        };
        let center = *indexes.get(&atom.id()).ok_or(
            PeptideStructurePlanDocumentPreparationErrorV1::MissingStereoCenter { atom: atom.id() },
        )?;
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
        ligands.sort_by_key(|ligand| match ligand {
            DocumentStereoLigandV1::Atom(index) => *index,
            DocumentStereoLigandV1::ExplicitHydrogen => usize::MAX,
        });
        if graph.atoms()[center].explicit_hydrogens() == Some(1) {
            ligands.push(DocumentStereoLigandV1::ExplicitHydrogen);
        }
        let ligands = ligands.try_into().map_err(|_| {
            PeptideStructurePlanDocumentPreparationErrorV1::UnrepresentableTetrahedral { center }
        })?;
        tetrahedral.push(
            DocumentTetrahedralStereoV1::new(center, ligands, parity).map_err(|_| {
                PeptideStructurePlanDocumentPreparationErrorV1::UnrepresentableTetrahedral {
                    center,
                }
            })?,
        );
    }
    Ok(DocumentStereoSemanticReportV1::new(tetrahedral, Vec::new()))
}

fn atomic_number(element: PeptideAtomElementV1) -> Result<AtomicNumber, MolGraphError> {
    match element {
        PeptideAtomElementV1::Carbon => AtomicNumber::try_from(6),
        PeptideAtomElementV1::Nitrogen => AtomicNumber::try_from(7),
        PeptideAtomElementV1::Oxygen => AtomicNumber::try_from(8),
        PeptideAtomElementV1::Sulfur => AtomicNumber::try_from(16),
    }
}

const fn formal_charge(charge: PeptideFormalChargeV1) -> Option<i32> {
    match charge {
        PeptideFormalChargeV1::Neutral => None,
        PeptideFormalChargeV1::PositiveOne => Some(1),
        PeptideFormalChargeV1::NegativeOne => Some(-1),
    }
}

const fn explicit_hydrogens(stereo: PeptideAtomStereochemistryV1) -> Option<u16> {
    match stereo {
        PeptideAtomStereochemistryV1::Unspecified => None,
        PeptideAtomStereochemistryV1::TetrahedralS | PeptideAtomStereochemistryV1::TetrahedralR => {
            Some(1)
        }
    }
}

const fn bond_order(order: PeptideBondOrderV1) -> BondOrder {
    match order {
        PeptideBondOrderV1::Single => BondOrder::Single,
        PeptideBondOrderV1::Double => BondOrder::Double,
        PeptideBondOrderV1::Aromatic => BondOrder::Aromatic,
    }
}

const fn document_bond_order(order: BondOrder) -> Option<DocumentBondOrderV1> {
    match order {
        BondOrder::Single => Some(DocumentBondOrderV1::Single),
        BondOrder::Double => Some(DocumentBondOrderV1::Double),
        BondOrder::Aromatic | BondOrder::Triple | BondOrder::Quadruple => None,
    }
}

/// Closed refusal categories for typed peptide-plan document preparation.
#[derive(Debug, Error)]
pub enum PeptideStructurePlanDocumentPreparationErrorV1 {
    /// The immutable plan repeated a semantic atom identity.
    #[error("peptide structure plan repeats semantic atom {atom:?}")]
    DuplicateAtom { atom: PeptideAtomIdV1 },
    /// A plan bond named an atom absent from its atom list.
    #[error("peptide structure plan bond {bond} names an absent atom")]
    UnknownBondEndpoint { bond: usize },
    /// A chiral plan atom did not remain available during conversion.
    #[error("peptide structure plan stereo center {atom:?} is absent")]
    MissingStereoCenter { atom: PeptideAtomIdV1 },
    /// The profile's stated tetrahedral fact did not have four durable ligands.
    #[error("peptide structure plan tetrahedral center {center} is not document-representable")]
    UnrepresentableTetrahedral { center: usize },
    /// The chemistry engine omitted the coordinates it was asked to generate.
    #[error("peptide structure plan coordinate generation omitted coordinates")]
    MissingGeneratedCoordinates,
    /// Chemistry returned a bond order with no exact document persistence mapping.
    #[error("peptide structure plan bond {bond} has unsupported {order:?} order after preparation")]
    UnsupportedBondOrder { bond: usize, order: BondOrder },
    /// Typed chemistry graph construction rejected a mapped plan fact.
    #[error(transparent)]
    Graph(#[from] MolGraphError),
    /// Native coordinate generation failed for the typed graph.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// Document placement rejected generated finite geometry.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// A placed point did not satisfy document projection constraints.
    #[error(transparent)]
    Position(#[from] ProjectionError),
    /// The detached molecule insertion rejected a mapped graph fact.
    #[error(transparent)]
    Insertion(#[from] MoleculeInsertionV1Error),
    /// Document stereo admission rejected a graph-relative descriptor.
    #[error(transparent)]
    Stereo(#[from] DocumentStereoSemanticsErrorV1),
    /// The adapter could not reserve owned conversion storage.
    #[error("peptide structure plan document conversion could not reserve storage")]
    ResourceAllocation,
}

#[cfg(test)]
#[path = "peptide_structure_plan_document_adapter_v1_tests.rs"]
mod tests;
