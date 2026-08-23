//! Closed formal-electron oxidation-state reduction for materialized H/C/N/O graphs.

use std::collections::VecDeque;

use thiserror::Error;

use crate::{BondOrder, MolGraph};

/// The sole formal-electron convention implemented by this reducer.
pub const OXIDATION_STATE_CONVENTION_V1: &str = "formal-electron-assignment-hcno-v1";

const MAX_ATOMS: usize = 256;
const MAX_BONDS: usize = 512;
const MAX_COMPONENTS: usize = 64;

/// One scalar oxidation-state observation for a selected graph atom.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OxidationStateObservationV1 {
    /// The complete graph satisfied the V1 profile and invariant.
    Accepted {
        /// The selected atom's formal oxidation number.
        oxidation_number: i16,
    },
    /// The graph is structurally valid but outside the closed V1 profile.
    Unavailable {
        /// The sole deterministic reason no oxidation number is reported.
        reason: OxidationStateUnavailableReasonV1,
    },
}

/// Closed reasons that a valid molecular graph is outside the V1 profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OxidationStateUnavailableReasonV1 {
    ElementOutsideProfile,
    FormalChargeUnavailable,
    HydrogenTopologyUnsupported,
    AromaticityUnsupported,
    RadicalUnsupported,
    BondOrderUnavailable,
    BondOrderUnsupported,
    NonAtomVertexUnsupported,
    CoordinationOrDelocalizationUnsupported,
    ComponentInvariantFailed,
    ArithmeticOverflow,
}

/// A bounded request resource exceeded by the complete direct-root graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OxidationStateResourceV1 {
    Atoms,
    Bonds,
    Components,
}

/// Opaque approval for one exact bounded primitive-count profile.
///
/// Only [`admit_oxidation_state_root_v1`] can create this value. Keeping the
/// admitted counts private prevents a caller from changing the admitted size
/// profile. This capability proves resource bounds only; a document session
/// separately authenticates which durable root produced a lowered graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OxidationStateRootAdmissionV1 {
    atom_count: usize,
    bond_count: usize,
}

/// A request failure that cannot be represented as an atom-level unavailable observation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum OxidationStateErrorV1 {
    /// The caller did not select an atom in the supplied graph.
    #[error("selected atom position {selected_atom} is outside {atom_count} atoms")]
    SelectedAtomOutOfRange {
        selected_atom: usize,
        atom_count: usize,
    },
    /// A graph fact contradicts the `MolGraph` structural contract.
    #[error("molecular graph has an invalid bond endpoint")]
    InvalidGraphStructure,
    /// The direct root exceeds a V1 resource bound.
    #[error("oxidation-state V1 {resource:?} limit of {maximum} exceeded by {actual}")]
    ResourceLimit {
        resource: OxidationStateResourceV1,
        maximum: usize,
        actual: usize,
    },
}

/// Admit one direct root's primitive-count profile before lowering.
///
/// This is the sole authority for the V1 atom and bond resource bounds. The
/// returned capability is intentionally bound to these exact counts, not to a
/// durable root identity. The document caller retains that provenance.
pub fn admit_oxidation_state_root_v1(
    atom_count: usize,
    bond_count: usize,
) -> Result<OxidationStateRootAdmissionV1, OxidationStateErrorV1> {
    bound(OxidationStateResourceV1::Atoms, MAX_ATOMS, atom_count)?;
    bound(OxidationStateResourceV1::Bonds, MAX_BONDS, bond_count)?;
    Ok(OxidationStateRootAdmissionV1 {
        atom_count,
        bond_count,
    })
}

/// Observe one selected atom under `formal-electron-assignment-hcno-v1`.
///
/// The reducer validates and computes the complete graph before returning the
/// selected scalar. It has no document, serialization, or native-engine role.
pub fn observe_oxidation_state_v1(
    graph: &MolGraph,
    selected_atom: usize,
) -> Result<OxidationStateObservationV1, OxidationStateErrorV1> {
    let admission = admit_oxidation_state_root_v1(graph.atoms().len(), graph.bonds().len())?;
    observe_admitted_oxidation_state_v1(&admission, graph, selected_atom)
}

/// Observe a selected atom in a graph that matches a prior count admission.
///
/// The count check happens before adjacency allocation and rejects impossible
/// resource-profile wiring between the admission and lowered graph. A document
/// session separately authenticates root provenance before calling this API.
pub fn observe_admitted_oxidation_state_v1(
    admission: &OxidationStateRootAdmissionV1,
    graph: &MolGraph,
    selected_atom: usize,
) -> Result<OxidationStateObservationV1, OxidationStateErrorV1> {
    if graph.atoms().len() != admission.atom_count || graph.bonds().len() != admission.bond_count {
        return Err(OxidationStateErrorV1::InvalidGraphStructure);
    }
    if selected_atom >= graph.atoms().len() {
        return Err(OxidationStateErrorV1::SelectedAtomOutOfRange {
            selected_atom,
            atom_count: graph.atoms().len(),
        });
    }
    if let Some(reason) = validate_atoms(graph) {
        return Ok(OxidationStateObservationV1::Unavailable { reason });
    }
    if let Some(reason) = validate_bonds(graph) {
        return Ok(OxidationStateObservationV1::Unavailable { reason });
    }

    let adjacency = adjacency(graph)?;
    let oxidation_numbers = match oxidation_numbers(graph, &adjacency) {
        Ok(numbers) => numbers,
        Err(reason) => return Ok(OxidationStateObservationV1::Unavailable { reason }),
    };
    let charges = formal_charges(graph);
    match component_invariant(&adjacency, &oxidation_numbers, &charges)? {
        Ok(()) => Ok(OxidationStateObservationV1::Accepted {
            oxidation_number: oxidation_numbers[selected_atom],
        }),
        Err(reason) => Ok(OxidationStateObservationV1::Unavailable { reason }),
    }
}

fn bound(
    resource: OxidationStateResourceV1,
    maximum: usize,
    actual: usize,
) -> Result<(), OxidationStateErrorV1> {
    if actual > maximum {
        return Err(OxidationStateErrorV1::ResourceLimit {
            resource,
            maximum,
            actual,
        });
    }
    Ok(())
}

fn validate_atoms(graph: &MolGraph) -> Option<OxidationStateUnavailableReasonV1> {
    for atom in graph.atoms() {
        if !matches!(atom.atomic_number().get(), 1 | 6 | 7 | 8) {
            return Some(OxidationStateUnavailableReasonV1::ElementOutsideProfile);
        }
        if !matches!(atom.formal_charge(), Some(-4..=4)) {
            return Some(OxidationStateUnavailableReasonV1::FormalChargeUnavailable);
        }
        if atom.explicit_hydrogens() != Some(0) {
            return Some(OxidationStateUnavailableReasonV1::HydrogenTopologyUnsupported);
        }
        if atom.is_aromatic() {
            return Some(OxidationStateUnavailableReasonV1::AromaticityUnsupported);
        }
        if atom.radical_electrons() != 0 {
            return Some(OxidationStateUnavailableReasonV1::RadicalUnsupported);
        }
    }
    None
}

fn validate_bonds(graph: &MolGraph) -> Option<OxidationStateUnavailableReasonV1> {
    for bond in graph.bonds() {
        if bond.is_aromatic() || bond.order() == BondOrder::Aromatic {
            return Some(OxidationStateUnavailableReasonV1::BondOrderUnavailable);
        }
        if !matches!(
            bond.order(),
            BondOrder::Single | BondOrder::Double | BondOrder::Triple
        ) {
            return Some(OxidationStateUnavailableReasonV1::BondOrderUnsupported);
        }
    }
    None
}

fn adjacency(graph: &MolGraph) -> Result<Vec<Vec<(usize, i16)>>, OxidationStateErrorV1> {
    let mut adjacency = vec![Vec::new(); graph.atoms().len()];
    for bond in graph.bonds() {
        let order = bond_order(bond.order()).expect("validated bond order");
        if bond.start() >= adjacency.len() || bond.end() >= adjacency.len() {
            return Err(OxidationStateErrorV1::InvalidGraphStructure);
        }
        adjacency[bond.start()].push((bond.end(), order));
        adjacency[bond.end()].push((bond.start(), order));
    }
    Ok(adjacency)
}

fn bond_order(order: BondOrder) -> Option<i16> {
    match order {
        BondOrder::Single => Some(1),
        BondOrder::Double => Some(2),
        BondOrder::Triple => Some(3),
        BondOrder::Aromatic | BondOrder::Quadruple => None,
    }
}

fn formal_charges(graph: &MolGraph) -> Vec<i16> {
    graph
        .atoms()
        .iter()
        .map(|atom| i16::try_from(atom.formal_charge().expect("validated formal charge")))
        .collect::<Result<Vec<_>, _>>()
        .expect("V1 formal-charge bounds fit i16")
}

fn oxidation_numbers(
    graph: &MolGraph,
    adjacency: &[Vec<(usize, i16)>],
) -> Result<Vec<i16>, OxidationStateUnavailableReasonV1> {
    let charges = formal_charges(graph);
    let mut numbers = charges;
    for (index, neighbors) in adjacency.iter().enumerate() {
        for &(neighbor, order) in neighbors {
            let direction = electronegativity_rank(graph.atoms()[neighbor].atomic_number().get())
                .cmp(&electronegativity_rank(
                    graph.atoms()[index].atomic_number().get(),
                )) as i16;
            let contribution = order
                .checked_mul(direction)
                .ok_or(OxidationStateUnavailableReasonV1::ArithmeticOverflow)?;
            numbers[index] = checked_add(numbers[index], contribution)?;
        }
    }
    Ok(numbers)
}

fn electronegativity_rank(atomic_number: u8) -> i8 {
    match atomic_number {
        1 => 0,
        6 => 1,
        7 => 2,
        8 => 3,
        _ => unreachable!("validated H/C/N/O atom"),
    }
}

fn checked_add(left: i16, right: i16) -> Result<i16, OxidationStateUnavailableReasonV1> {
    left.checked_add(right)
        .ok_or(OxidationStateUnavailableReasonV1::ArithmeticOverflow)
}

fn component_invariant(
    adjacency: &[Vec<(usize, i16)>],
    oxidation_numbers: &[i16],
    formal_charges: &[i16],
) -> Result<Result<(), OxidationStateUnavailableReasonV1>, OxidationStateErrorV1> {
    let mut visited = vec![false; adjacency.len()];
    let mut components = 0;
    for root in 0..adjacency.len() {
        if visited[root] {
            continue;
        }
        components += 1;
        bound(
            OxidationStateResourceV1::Components,
            MAX_COMPONENTS,
            components,
        )?;
        let mut pending = VecDeque::from([root]);
        let mut oxidation_sum = 0_i16;
        let mut charge_sum = 0_i16;
        visited[root] = true;
        while let Some(atom) = pending.pop_front() {
            oxidation_sum = match checked_add(oxidation_sum, oxidation_numbers[atom]) {
                Ok(value) => value,
                Err(reason) => return Ok(Err(reason)),
            };
            charge_sum = match checked_add(charge_sum, formal_charges[atom]) {
                Ok(value) => value,
                Err(reason) => return Ok(Err(reason)),
            };
            for &(neighbor, _) in &adjacency[atom] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    pending.push_back(neighbor);
                }
            }
        }
        if oxidation_sum != charge_sum {
            return Ok(Err(
                OxidationStateUnavailableReasonV1::ComponentInvariantFailed,
            ));
        }
    }
    Ok(Ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtomChirality, AtomicNumber, MolAtom, MolBond};

    fn atom(symbol: &str, charge: i32) -> MolAtom {
        MolAtom::new(
            AtomicNumber::from_symbol(symbol).expect("H/C/N/O symbol"),
            Some(charge),
            None,
            Some(0),
            false,
        )
        .expect("valid atom")
    }

    fn graph(atoms: Vec<(&str, i32)>, bonds: &[(usize, usize, BondOrder)]) -> MolGraph {
        MolGraph::new(
            atoms
                .into_iter()
                .map(|(symbol, charge)| atom(symbol, charge))
                .collect(),
            bonds
                .iter()
                .map(|&(start, end, order)| MolBond::new(start, end, order, false))
                .collect(),
            None,
        )
        .expect("valid synthetic graph")
    }

    fn values(graph: &MolGraph) -> Vec<i16> {
        (0..graph.atoms().len())
            .map(|index| match observe_oxidation_state_v1(graph, index) {
                Ok(OxidationStateObservationV1::Accepted { oxidation_number }) => oxidation_number,
                result => panic!("expected accepted result, got {result:?}"),
            })
            .collect()
    }

    #[test]
    fn computes_the_accepted_materialized_corpus() {
        let corpus = [
            (
                graph(vec![("H", 0), ("H", 0)], &[(0, 1, BondOrder::Single)]),
                vec![0, 0],
            ),
            (
                graph(
                    vec![("O", 0), ("H", 0), ("H", 0)],
                    &[(0, 1, BondOrder::Single), (0, 2, BondOrder::Single)],
                ),
                vec![-2, 1, 1],
            ),
            (
                graph(
                    vec![("N", 0), ("H", 0), ("H", 0), ("H", 0)],
                    &[
                        (0, 1, BondOrder::Single),
                        (0, 2, BondOrder::Single),
                        (0, 3, BondOrder::Single),
                    ],
                ),
                vec![-3, 1, 1, 1],
            ),
            (
                graph(
                    vec![("C", 0), ("H", 0), ("H", 0), ("H", 0), ("H", 0)],
                    &[
                        (0, 1, BondOrder::Single),
                        (0, 2, BondOrder::Single),
                        (0, 3, BondOrder::Single),
                        (0, 4, BondOrder::Single),
                    ],
                ),
                vec![-4, 1, 1, 1, 1],
            ),
            (
                graph(
                    vec![("O", 0), ("C", 0), ("O", 0)],
                    &[(0, 1, BondOrder::Double), (1, 2, BondOrder::Double)],
                ),
                vec![-2, 4, -2],
            ),
            (
                graph(vec![("C", -1), ("O", 1)], &[(0, 1, BondOrder::Triple)]),
                vec![2, -2],
            ),
            (
                graph(
                    vec![("H", 0), ("O", 0), ("O", 0), ("H", 0)],
                    &[
                        (0, 1, BondOrder::Single),
                        (1, 2, BondOrder::Single),
                        (2, 3, BondOrder::Single),
                    ],
                ),
                vec![1, -1, -1, 1],
            ),
            (
                graph(
                    vec![("C", 0), ("O", 0), ("H", 0), ("H", 0), ("H", 0), ("H", 0)],
                    &[
                        (0, 1, BondOrder::Single),
                        (0, 2, BondOrder::Single),
                        (0, 3, BondOrder::Single),
                        (0, 4, BondOrder::Single),
                        (1, 5, BondOrder::Single),
                    ],
                ),
                vec![-2, -2, 1, 1, 1, 1],
            ),
            (
                graph(
                    vec![("N", 1), ("H", 0), ("H", 0), ("H", 0), ("H", 0)],
                    &[
                        (0, 1, BondOrder::Single),
                        (0, 2, BondOrder::Single),
                        (0, 3, BondOrder::Single),
                        (0, 4, BondOrder::Single),
                    ],
                ),
                vec![-3, 1, 1, 1, 1],
            ),
            (
                graph(vec![("O", -1), ("H", 0)], &[(0, 1, BondOrder::Single)]),
                vec![-2, 1],
            ),
        ];
        for (molecule, expected) in corpus {
            assert_eq!(values(&molecule), expected);
        }
    }

    #[test]
    fn preserves_component_charge_conservation_for_disconnected_graphs() {
        let molecule = graph(
            vec![
                ("O", 0),
                ("H", 0),
                ("H", 0),
                ("N", 1),
                ("H", 0),
                ("H", 0),
                ("H", 0),
                ("H", 0),
            ],
            &[
                (0, 1, BondOrder::Single),
                (0, 2, BondOrder::Single),
                (3, 4, BondOrder::Single),
                (3, 5, BondOrder::Single),
                (3, 6, BondOrder::Single),
                (3, 7, BondOrder::Single),
            ],
        );
        assert_eq!(values(&molecule), vec![-2, 1, 1, -3, 1, 1, 1, 1]);
    }

    #[test]
    fn is_independent_of_bond_input_order() {
        let atoms = vec![("C", 0), ("O", 0), ("H", 0), ("H", 0), ("H", 0), ("H", 0)];
        let forward = graph(
            atoms.clone(),
            &[
                (0, 1, BondOrder::Single),
                (0, 2, BondOrder::Single),
                (0, 3, BondOrder::Single),
                (0, 4, BondOrder::Single),
                (1, 5, BondOrder::Single),
            ],
        );
        let reversed = graph(
            atoms,
            &[
                (1, 5, BondOrder::Single),
                (0, 4, BondOrder::Single),
                (0, 3, BondOrder::Single),
                (0, 2, BondOrder::Single),
                (0, 1, BondOrder::Single),
            ],
        );
        assert_eq!(values(&forward), values(&reversed));
    }

    #[test]
    fn returns_closed_unavailable_reasons_for_profile_facts() {
        let missing_charge = MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::from_symbol("O").expect("oxygen"),
                    None,
                    None,
                    Some(0),
                    false,
                )
                .expect("atom"),
            ],
            Vec::new(),
            None,
        )
        .expect("graph");
        let implicit_hydrogen = MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::from_symbol("O").expect("oxygen"),
                    Some(0),
                    None,
                    None,
                    false,
                )
                .expect("atom"),
            ],
            Vec::new(),
            None,
        )
        .expect("graph");
        let sulfur = MolGraph::new(
            vec![
                atom("O", 0),
                MolAtom::new(
                    AtomicNumber::from_symbol("S").expect("sulfur"),
                    Some(0),
                    None,
                    Some(0),
                    false,
                )
                .expect("atom"),
            ],
            vec![MolBond::new(0, 1, BondOrder::Single, false)],
            None,
        )
        .expect("graph");
        let quadruple = graph(vec![("C", 0), ("C", 0)], &[(0, 1, BondOrder::Quadruple)]);
        let aromatic = MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::from_symbol("O").expect("oxygen"),
                    Some(0),
                    None,
                    Some(0),
                    true,
                )
                .expect("atom"),
            ],
            Vec::new(),
            None,
        )
        .expect("graph");
        let radical = MolGraph::new(
            vec![
                MolAtom::from_native(
                    AtomicNumber::from_symbol("O").expect("oxygen"),
                    0,
                    0,
                    0,
                    false,
                    AtomChirality::Unspecified,
                    1,
                    false,
                    0,
                )
                .expect("atom"),
            ],
            Vec::new(),
            None,
        )
        .expect("graph");
        let out_of_range_charge = MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::from_symbol("O").expect("oxygen"),
                    Some(5),
                    None,
                    Some(0),
                    false,
                )
                .expect("atom"),
            ],
            Vec::new(),
            None,
        )
        .expect("graph");
        assert_eq!(
            observe_oxidation_state_v1(&missing_charge, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::FormalChargeUnavailable
            })
        );
        assert_eq!(
            observe_oxidation_state_v1(&implicit_hydrogen, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::HydrogenTopologyUnsupported
            })
        );
        assert_eq!(
            observe_oxidation_state_v1(&sulfur, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::ElementOutsideProfile
            })
        );
        assert_eq!(
            observe_oxidation_state_v1(&quadruple, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::BondOrderUnsupported
            })
        );
        assert_eq!(
            observe_oxidation_state_v1(&aromatic, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::AromaticityUnsupported
            })
        );
        assert_eq!(
            observe_oxidation_state_v1(&radical, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::RadicalUnsupported
            })
        );
        assert_eq!(
            observe_oxidation_state_v1(&out_of_range_charge, 0),
            Ok(OxidationStateObservationV1::Unavailable {
                reason: OxidationStateUnavailableReasonV1::FormalChargeUnavailable
            })
        );
    }

    #[test]
    fn rejects_root_resource_excess_before_reduction() {
        let molecule = MolGraph::new((0..257).map(|_| atom("H", 0)).collect(), Vec::new(), None)
            .expect("structurally valid graph");
        assert_eq!(
            observe_oxidation_state_v1(&molecule, 0),
            Err(OxidationStateErrorV1::ResourceLimit {
                resource: OxidationStateResourceV1::Atoms,
                maximum: 256,
                actual: 257
            })
        );
    }

    #[test]
    fn admits_only_roots_inside_the_document_conversion_bounds() {
        assert_eq!(
            admit_oxidation_state_root_v1(257, 0),
            Err(OxidationStateErrorV1::ResourceLimit {
                resource: OxidationStateResourceV1::Atoms,
                maximum: 256,
                actual: 257,
            })
        );
        assert_eq!(
            admit_oxidation_state_root_v1(0, 513),
            Err(OxidationStateErrorV1::ResourceLimit {
                resource: OxidationStateResourceV1::Bonds,
                maximum: 512,
                actual: 513,
            })
        );
    }

    #[test]
    fn rejects_a_lowered_graph_that_does_not_match_its_admission() {
        let admission = admit_oxidation_state_root_v1(2, 1).expect("admitted primitive counts");
        let substituted = graph(vec![("H", 0)], &[]);
        assert_eq!(
            observe_admitted_oxidation_state_v1(&admission, &substituted, 0),
            Err(OxidationStateErrorV1::InvalidGraphStructure)
        );
    }

    #[test]
    fn admitted_reduction_preserves_the_normal_observation() {
        let molecule = graph(
            vec![("O", 0), ("H", 0), ("H", 0)],
            &[(0, 1, BondOrder::Single), (0, 2, BondOrder::Single)],
        );
        let admission = admit_oxidation_state_root_v1(3, 2).expect("admitted water root");
        assert_eq!(
            observe_admitted_oxidation_state_v1(&admission, &molecule, 0),
            observe_oxidation_state_v1(&molecule, 0)
        );
    }

    #[test]
    fn detects_checked_arithmetic_and_component_invariant_failures() {
        let adjacency = vec![Vec::new()];
        assert_eq!(
            component_invariant(&adjacency, &[i16::MAX], &[-1]),
            Ok(Err(
                OxidationStateUnavailableReasonV1::ComponentInvariantFailed
            ))
        );
        assert_eq!(
            checked_add(i16::MAX, 1),
            Err(OxidationStateUnavailableReasonV1::ArithmeticOverflow)
        );
    }
}
