//! Closed native-SMILES lowering for one detached direct-Haworth receipt.
//!
//! This module deliberately accepts a much smaller profile than general SMILES:
//! exactly two C/O five- or six-member cycles and one exterior oxygen bridge.
//! It preserves the adapter graph facts by rejecting facts the authoring receipt
//! cannot encode; it neither normalizes nor kekulizes input.

use std::collections::{BTreeSet, VecDeque};

use super::{
    DirectGlycosidicHaworthAuthoringReceiptV1, DirectGlycosidicHaworthTopologyV1, HaworthError,
    HaworthPoint, HaworthTopologyBuilder, HaworthVertex, RingForm,
    direct_glycosidic_haworth_authoring_receipt_v1,
};
use ferrum_chemistry::{
    AtomChirality, AtomicNumber, BondDirection, BondOrder as ChemistryBondOrder, BondStereo,
    ChemEngine, ChemistryError, MolAtom, MolBond, MolGraph, NativeChemEngine,
    validate_smiles_input,
};
use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
use ferrum_geometry::MoleculePlacementV1;
use thiserror::Error;

/// A detached authoring receipt and its destination translation.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedDirectHaworthFromSmilesV1 {
    receipt: DirectGlycosidicHaworthAuthoringReceiptV1,
    translation: HaworthPoint,
}

impl PreparedDirectHaworthFromSmilesV1 {
    /// Return the frozen domain-owned receipt for later session preparation.
    #[must_use]
    pub const fn receipt(&self) -> &DirectGlycosidicHaworthAuthoringReceiptV1 {
        &self.receipt
    }

    /// Return the receipt-local translation which preserves placement centering.
    #[must_use]
    pub const fn translation(&self) -> HaworthPoint {
        self.translation
    }
}

/// Build a detached direct-Haworth receipt with the concrete native engine.
pub fn build_direct_haworth_from_smiles_v1(
    engine: &NativeChemEngine,
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDirectHaworthFromSmilesV1, DirectHaworthFromSmilesBuildErrorV1> {
    build_with_engine(engine, smiles, placement)
}

/// Build the closed Haworth profile from Rust-parsed structural SMILES.
///
/// This intentionally admits only the atom and bond grammar that the closed
/// direct-glycosidic profile can represent: uppercase C/O atoms, branches, and
/// one-digit ring closures. It never consults an optional chemistry engine.
pub fn build_direct_haworth_from_text_smiles_v1(
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDirectHaworthFromSmilesV1, DirectHaworthFromSmilesBuildErrorV1> {
    let graph = parse_closed_haworth_smiles(smiles)?;
    build_from_graph(&graph, placement)
}

#[cfg(test)]
pub(crate) fn build_direct_haworth_from_smiles_with_engine_for_test<E: ChemEngine>(
    engine: &E,
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDirectHaworthFromSmilesV1, DirectHaworthFromSmilesBuildErrorV1> {
    build_with_engine(engine, smiles, placement)
}

fn build_with_engine<E: ChemEngine>(
    engine: &E,
    smiles: &str,
    placement: MoleculePlacementV1,
) -> Result<PreparedDirectHaworthFromSmilesV1, DirectHaworthFromSmilesBuildErrorV1> {
    validate_smiles_input(smiles).map_err(DirectHaworthFromSmilesBuildErrorV1::InvalidInput)?;
    let parsed = engine.smiles_to_molecule(smiles)?;
    build_from_graph(parsed.molecule(), placement)
}

fn build_from_graph(
    graph: &MolGraph,
    placement: MoleculePlacementV1,
) -> Result<PreparedDirectHaworthFromSmilesV1, DirectHaworthFromSmilesBuildErrorV1> {
    validate_raw_facts(graph)?;
    let (molecule, topology) = lower_closed_profile(graph)?;
    let receipt = direct_glycosidic_haworth_authoring_receipt_v1(
        &molecule,
        topology,
        placement.bond_length(),
    )
    .map_err(DirectHaworthFromSmilesBuildErrorV1::Authoring)?;
    let translation = placement_translation(&receipt, placement)?;
    Ok(PreparedDirectHaworthFromSmilesV1 {
        receipt,
        translation,
    })
}

fn parse_closed_haworth_smiles(
    smiles: &str,
) -> Result<MolGraph, DirectHaworthFromSmilesBuildErrorV1> {
    if smiles.is_empty() || smiles.len() > 4_096 {
        return Err(DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
            reason: "SMILES must contain between 1 and 4096 bytes",
        });
    }
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let mut current = None;
    let mut branches = Vec::new();
    let mut rings: [Option<usize>; 10] = [None; 10];
    for byte in smiles.bytes() {
        match byte {
            b'C' | b'O' => {
                let symbol = if byte == b'C' { "C" } else { "O" };
                let atom = MolAtom::new(
                    AtomicNumber::from_symbol(symbol).map_err(|_| {
                        DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
                            reason: "SMILES contains an unsupported atom",
                        }
                    })?,
                    None,
                    None,
                    None,
                    false,
                )
                .map_err(|_| DirectHaworthFromSmilesBuildErrorV1::Resource {
                    reason: "could not reserve structural SMILES atom storage",
                })?;
                let next = atoms.len();
                atoms.push(atom);
                if let Some(previous) = current {
                    bonds.push(MolBond::new(
                        previous,
                        next,
                        ChemistryBondOrder::Single,
                        false,
                    ));
                }
                current = Some(next);
            }
            b'(' => branches.push(current.ok_or(
                DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
                    reason: "branch has no preceding atom",
                },
            )?),
            b')' => {
                current = Some(branches.pop().ok_or(
                    DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
                        reason: "branch close has no matching open",
                    },
                )?)
            }
            b'1'..=b'9' => {
                let atom = current.ok_or(DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
                    reason: "ring closure has no preceding atom",
                })?;
                let slot = usize::from(byte - b'0');
                if let Some(other) = rings[slot].take() {
                    bonds.push(MolBond::new(other, atom, ChemistryBondOrder::Single, false));
                } else {
                    rings[slot] = Some(atom);
                }
            }
            _ => {
                return Err(DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
                    reason: "only uppercase C/O atoms, branches, and one-digit ring closures are supported",
                });
            }
        }
    }
    if current.is_none() || !branches.is_empty() || rings.iter().any(Option::is_some) {
        return Err(DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
            reason: "SMILES has an incomplete branch or ring closure",
        });
    }
    MolGraph::new(atoms, bonds, None).map_err(|_| {
        DirectHaworthFromSmilesBuildErrorV1::SmilesSyntax {
            reason: "SMILES does not form a valid simple molecular graph",
        }
    })
}

fn validate_raw_facts(graph: &MolGraph) -> Result<(), DirectHaworthFromSmilesBuildErrorV1> {
    for (index, atom) in graph.atoms().iter().enumerate() {
        if !matches!(atom.atomic_number().symbol(), "C" | "O") {
            return Err(DirectHaworthFromSmilesBuildErrorV1::UnsupportedAtomFact {
                index,
                fact: "an element other than C or O",
            });
        }
        let invalid = [
            (
                atom.formal_charge().unwrap_or_default() != 0,
                "formal charge",
            ),
            (atom.isotope().is_some(), "isotope"),
            (
                atom.explicit_hydrogens().unwrap_or_default() != 0,
                "explicit hydrogens",
            ),
            (atom.is_aromatic(), "aromaticity"),
            (atom.chirality() != AtomChirality::Unspecified, "chirality"),
            (atom.radical_electrons() != 0, "radical electrons"),
            (atom.no_implicit(), "no-implicit flag"),
            (atom.atom_map_number().is_some(), "atom map number"),
        ];
        if let Some((_, fact)) = invalid.into_iter().find(|(invalid, _)| *invalid) {
            return Err(DirectHaworthFromSmilesBuildErrorV1::UnsupportedAtomFact { index, fact });
        }
    }
    for (index, bond) in graph.bonds().iter().enumerate() {
        let invalid = [
            (
                bond.order() != ChemistryBondOrder::Single,
                "non-single order",
            ),
            (bond.is_aromatic(), "aromaticity"),
            (bond.stereo() != BondStereo::None, "stereo"),
            (bond.direction() != BondDirection::None, "direction"),
            (bond.stereo_atoms().is_some(), "stereo atom references"),
        ];
        if let Some((_, fact)) = invalid.into_iter().find(|(invalid, _)| *invalid) {
            return Err(DirectHaworthFromSmilesBuildErrorV1::UnsupportedBondFact { index, fact });
        }
    }
    Ok(())
}

fn lower_closed_profile(
    graph: &MolGraph,
) -> Result<(Molecule, DirectGlycosidicHaworthTopologyV1), DirectHaworthFromSmilesBuildErrorV1> {
    let atom_count = graph.atoms().len();
    if !(11..=13).contains(&atom_count) || graph.bonds().len() != atom_count + 1 {
        return Err(DirectHaworthFromSmilesBuildErrorV1::Profile {
            reason: "requires exactly two five- or six-member C/O cycles and one bridge oxygen",
        });
    }
    let adjacency = adjacency(graph)?;
    let candidates: Vec<_> = graph
        .atoms()
        .iter()
        .enumerate()
        .filter(|(index, atom)| {
            atom.atomic_number().symbol() == "O" && adjacency[*index].len() == 2
        })
        .filter_map(|(bridge, _)| select_candidate(graph, &adjacency, bridge).ok())
        .collect();
    let [candidate] = candidates.as_slice() else {
        return Err(DirectHaworthFromSmilesBuildErrorV1::Profile {
            reason: "requires one uniquely classifiable exterior degree-two oxygen bridge",
        });
    };
    let molecule = temporary_molecule(graph)?;
    let build_ring = |ring: &RingCandidate| {
        let vertices = ring
            .cycle
            .iter()
            .map(|index| HaworthVertex {
                atom: molecule.atoms()[*index].identity().clone(),
            })
            .collect();
        HaworthTopologyBuilder::new(
            ring.form,
            molecule.atoms()[ring.layout_attachment].identity().clone(),
            vertices,
        )
        .build(&molecule)
    };
    let rings = [
        build_ring(&candidate.rings[0]).map_err(DirectHaworthFromSmilesBuildErrorV1::Topology)?,
        build_ring(&candidate.rings[1]).map_err(DirectHaworthFromSmilesBuildErrorV1::Topology)?,
    ];
    let topology = DirectGlycosidicHaworthTopologyV1::classify(
        &molecule,
        rings,
        molecule.atoms()[candidate.bridge].identity().clone(),
        candidate
            .bridge_bonds
            .map(|index| molecule.bonds()[index].identity().clone()),
    )
    .map_err(DirectHaworthFromSmilesBuildErrorV1::Topology)?;
    Ok((molecule, topology))
}

#[derive(Clone)]
struct RingCandidate {
    cycle: Vec<usize>,
    form: RingForm,
    layout_attachment: usize,
}
#[derive(Clone)]
struct Candidate {
    bridge: usize,
    bridge_bonds: [usize; 2],
    rings: [RingCandidate; 2],
}

fn adjacency(
    graph: &MolGraph,
) -> Result<Vec<Vec<(usize, usize)>>, DirectHaworthFromSmilesBuildErrorV1> {
    let mut result = vec![Vec::new(); graph.atoms().len()];
    for (bond_index, bond) in graph.bonds().iter().enumerate() {
        if bond.start() >= result.len() || bond.end() >= result.len() {
            return Err(DirectHaworthFromSmilesBuildErrorV1::Profile {
                reason: "has invalid bond endpoints",
            });
        }
        result[bond.start()].push((bond.end(), bond_index));
        result[bond.end()].push((bond.start(), bond_index));
    }
    Ok(result)
}

fn select_candidate(
    graph: &MolGraph,
    adjacency: &[Vec<(usize, usize)>],
    bridge: usize,
) -> Result<Candidate, ()> {
    let [(first, first_bond), (second, second_bond)] = adjacency[bridge].as_slice() else {
        return Err(());
    };
    if graph.atoms()[*first].atomic_number().symbol() != "C"
        || graph.atoms()[*second].atomic_number().symbol() != "C"
    {
        return Err(());
    }
    let components = components_without(adjacency, bridge);
    if components.len() != 2 {
        return Err(());
    }
    let first_component = components
        .iter()
        .position(|part| part.contains(first))
        .ok_or(())?;
    let second_component = components
        .iter()
        .position(|part| part.contains(second))
        .ok_or(())?;
    if first_component == second_component {
        return Err(());
    }
    let left = cycle_component(graph, adjacency, bridge, components[0].clone());
    let right = cycle_component(graph, adjacency, bridge, components[1].clone());
    let (Ok(left), Ok(right)) = (left, right) else {
        return Err(());
    };
    Ok(Candidate {
        bridge,
        bridge_bonds: [*first_bond, *second_bond],
        rings: [
            RingCandidate {
                cycle: left,
                form: ring_form(components[0].len())?,
                layout_attachment: oxygen_neighbor(graph, adjacency, bridge, &components[0])?,
            },
            RingCandidate {
                cycle: right,
                form: ring_form(components[1].len())?,
                layout_attachment: oxygen_neighbor(graph, adjacency, bridge, &components[1])?,
            },
        ],
    })
}

fn components_without(adjacency: &[Vec<(usize, usize)>], omitted: usize) -> Vec<BTreeSet<usize>> {
    let mut unseen: BTreeSet<_> = (0..adjacency.len())
        .filter(|index| *index != omitted)
        .collect();
    let mut result = Vec::new();
    while let Some(start) = unseen.iter().next().copied() {
        let mut part = BTreeSet::new();
        let mut queue = VecDeque::from([start]);
        unseen.remove(&start);
        while let Some(current) = queue.pop_front() {
            part.insert(current);
            for &(next, _) in &adjacency[current] {
                if next != omitted && unseen.remove(&next) {
                    queue.push_back(next);
                }
            }
        }
        result.push(part);
    }
    result
}

fn cycle_component(
    graph: &MolGraph,
    adjacency: &[Vec<(usize, usize)>],
    bridge: usize,
    component: BTreeSet<usize>,
) -> Result<Vec<usize>, ()> {
    if !(5..=6).contains(&component.len())
        || component
            .iter()
            .filter(|index| graph.atoms()[**index].atomic_number().symbol() == "O")
            .count()
            != 1
    {
        return Err(());
    }
    if component.iter().any(|index| {
        adjacency[*index]
            .iter()
            .filter(|(next, _)| *next != bridge)
            .count()
            != 2
    }) {
        return Err(());
    }
    let start = component
        .iter()
        .find(|index| graph.atoms()[**index].atomic_number().symbol() == "O")
        .copied()
        .ok_or(())?;
    let mut cycle = vec![start];
    let mut previous = bridge;
    let mut current = start;
    loop {
        let next = adjacency[current]
            .iter()
            .filter_map(|(next, _)| (*next != bridge && *next != previous).then_some(*next))
            .min()
            .ok_or(())?;
        if next == start {
            break;
        }
        if cycle.contains(&next) || !component.contains(&next) {
            return Err(());
        }
        cycle.push(next);
        previous = current;
        current = next;
    }
    (cycle.len() == component.len()).then_some(cycle).ok_or(())
}

fn ring_form(size: usize) -> Result<RingForm, ()> {
    match size {
        5 => Ok(RingForm::Furanose),
        6 => Ok(RingForm::Pyranose),
        _ => Err(()),
    }
}

/// Pick only the deterministic layout vertex; direct classification separately
/// retains the real bridge-attached carbon and makes no biochemical claim.
fn oxygen_neighbor(
    graph: &MolGraph,
    adjacency: &[Vec<(usize, usize)>],
    bridge: usize,
    component: &BTreeSet<usize>,
) -> Result<usize, ()> {
    let oxygen = component
        .iter()
        .find(|index| graph.atoms()[**index].atomic_number().symbol() == "O")
        .copied()
        .ok_or(())?;
    adjacency[oxygen]
        .iter()
        .filter_map(|(next, _)| (*next != bridge && component.contains(next)).then_some(*next))
        .min()
        .ok_or(())
}
fn temporary_molecule(graph: &MolGraph) -> Result<Molecule, DirectHaworthFromSmilesBuildErrorV1> {
    let mut atoms = Vec::with_capacity(graph.atoms().len());
    for (index, atom) in graph.atoms().iter().enumerate() {
        atoms.push(Atom::new(
            Some(identifier("atom", index)?),
            Some(atom.atomic_number().symbol().to_owned()),
            Position::new(index as f64, 0.0, 0.0)?,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )?);
    }
    let mut bonds = Vec::with_capacity(graph.bonds().len());
    for (index, bond) in graph.bonds().iter().enumerate() {
        bonds.push(Bond::new(
            Some(identifier("bond", index)?),
            VertexRef::Atom(atoms[bond.start()].identity().clone()),
            VertexRef::Atom(atoms[bond.end()].identity().clone()),
            None,
            Some(BondOrder::Single),
            None,
            Some(false),
            None,
        )?);
    }
    Molecule::new(
        Some(Identifier::new("native-direct-haworth-smiles-v1")?),
        None,
        atoms,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds,
        None,
    )
    .map_err(Into::into)
}

fn identifier(kind: &str, index: usize) -> Result<Identifier, ferrum_core::InvalidIdentifier> {
    Identifier::new(format!("native-direct-haworth-{kind}-{index}"))
}

fn placement_translation(
    receipt: &DirectGlycosidicHaworthAuthoringReceiptV1,
    placement: MoleculePlacementV1,
) -> Result<HaworthPoint, DirectHaworthFromSmilesBuildErrorV1> {
    let atoms = receipt.atoms_in_canonical_order();
    let count = atoms.len() as f64;
    let x = atoms.iter().map(|atom| atom.local().x).sum::<f64>() / count;
    let y = atoms.iter().map(|atom| atom.local().y).sum::<f64>() / count;
    let translation = HaworthPoint {
        x: placement.anchor().x() - x,
        y: placement.anchor().y() - y,
    };
    (translation.x.is_finite() && translation.y.is_finite())
        .then_some(translation)
        .ok_or(DirectHaworthFromSmilesBuildErrorV1::Resource {
            reason: "receipt centroid is not finite",
        })
}

/// Failures while building one closed direct-Haworth receipt off-session.
#[derive(Debug, Error)]
pub enum DirectHaworthFromSmilesBuildErrorV1 {
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error(transparent)]
    InvalidInput(ChemistryError),
    #[error("structural SMILES syntax rejection: {reason}")]
    SmilesSyntax { reason: &'static str },
    #[error("SMILES atom {index} has unsupported {fact}")]
    UnsupportedAtomFact { index: usize, fact: &'static str },
    #[error("SMILES bond {index} has unsupported {fact}")]
    UnsupportedBondFact { index: usize, fact: &'static str },
    #[error("direct Haworth SMILES profile rejection: {reason}")]
    Profile { reason: &'static str },
    #[error(transparent)]
    Topology(HaworthError),
    #[error(transparent)]
    Authoring(HaworthError),
    #[error("direct Haworth SMILES resource failure: {reason}")]
    Resource { reason: &'static str },
    #[error(transparent)]
    Core(#[from] ferrum_core::ModelError),
    #[error(transparent)]
    Identifier(#[from] ferrum_core::InvalidIdentifier),
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_chemistry::{ChemEngine, Coordinates, KekulizeOptions, SmilesMolecule};

    struct UnusedEngine;

    impl ChemEngine for UnusedEngine {
        fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
            unreachable!("input validation happens before adapter use")
        }

        fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
            unreachable!("input validation happens before adapter use")
        }

        fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
            unreachable!("input validation happens before adapter use")
        }
    }

    #[test]
    fn rejects_invalid_smiles_before_native_adapter_use() {
        let placement = MoleculePlacementV1::new(
            5.0,
            ferrum_geometry::Point2::new(0.0, 0.0).expect("finite placement"),
        )
        .expect("valid placement");
        assert!(matches!(
            build_direct_haworth_from_smiles_with_engine_for_test(&UnusedEngine, "", placement),
            Err(DirectHaworthFromSmilesBuildErrorV1::InvalidInput(_))
        ));
    }
}
