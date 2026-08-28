//! Pure, capability-free lowering of complete molecule facts to native chemistry graphs.

use ferrum_chemistry::{
    AtomicNumber, BondOrder as ChemistryBondOrder, Coordinates, MolAtom, MolBond, MolGraph,
    MolGraphError, Point2,
};
use ferrum_core::{BondOrder, BondStyle};
use ferrum_document_projection::{
    DirectMoleculeGraphEndpoint, DirectMoleculeGraphFacts, NonAtomVertexKindV1,
};
use thiserror::Error;

/// A native graph and its exact positional edges; no durable document identity is retained.
pub struct LoweredMoleculeGraph {
    graph: MolGraph,
    edges: Vec<(usize, usize)>,
}
impl LoweredMoleculeGraph {
    #[must_use]
    pub fn into_parts(self) -> (MolGraph, Vec<(usize, usize)>) {
        (self.graph, self.edges)
    }
}

/// Closed pure-lowering failures.
#[derive(Debug, Error)]
pub enum MoleculeGraphLoweringError {
    #[error("native chemistry requires a molecule with at least one atom")]
    EmptyMolecule,
    #[error("native chemistry does not yet support {count} {kind} vertices")]
    UnsupportedVertex { kind: &'static str, count: usize },
    #[error("atom {atom_index} has no element for native chemistry")]
    MissingElement { atom_index: usize },
    #[error("atom {atom_index} element {element:?} is not supported: {source}")]
    InvalidElement {
        atom_index: usize,
        element: String,
        #[source]
        source: MolGraphError,
    },
    #[error("atom {atom_index} has unsupported {fact}")]
    UnsupportedAtomFact {
        atom_index: usize,
        fact: &'static str,
    },
    #[error("bond {bond_index} has a non-atom or unresolved endpoint")]
    UnsupportedBondEndpoint { bond_index: usize },
    #[error("bond {bond_index} has a drawing style not supported by native chemistry")]
    UnsupportedBondStyle { bond_index: usize },
    #[error("bond {bond_index} has an unsupported or absent bond order")]
    UnsupportedBondOrder { bond_index: usize },
    #[error(transparent)]
    Graph(#[from] MolGraphError),
    #[error("native chemistry graph could not reserve owned storage")]
    ResourceAllocation,
}

/// Failure from lowering with a caller-owned observation of each valid atom.
///
/// The callback runs after the shared atom-fact validation has succeeded and
/// before coordinate or bond validation begins.  This lets a higher typed
/// facade retain its own identity invariants without duplicating chemistry.
#[derive(Debug)]
pub enum MoleculeGraphLoweringWithAtomError<E> {
    Lowering(MoleculeGraphLoweringError),
    AtomCallback(E),
}

/// Lower one complete pure fact set. Atom input order defines graph position.
pub fn lower_direct_molecule_graph(
    facts: &DirectMoleculeGraphFacts,
) -> Result<LoweredMoleculeGraph, MoleculeGraphLoweringError> {
    lower_direct_molecule_graph_with_validated_atom(facts, |_| {
        Ok::<(), std::convert::Infallible>(())
    })
    .map_err(|error| match error {
        MoleculeGraphLoweringWithAtomError::Lowering(error) => error,
        MoleculeGraphLoweringWithAtomError::AtomCallback(never) => match never {},
    })
}

/// Lower facts while observing each atom immediately after shared validation.
pub fn lower_direct_molecule_graph_with_validated_atom<E>(
    facts: &DirectMoleculeGraphFacts,
    mut on_validated_atom: impl FnMut(usize) -> Result<(), E>,
) -> Result<LoweredMoleculeGraph, MoleculeGraphLoweringWithAtomError<E>> {
    if facts.atoms().is_empty() {
        return Err(MoleculeGraphLoweringWithAtomError::Lowering(
            MoleculeGraphLoweringError::EmptyMolecule,
        ));
    }
    for selected_kind in [
        NonAtomVertexKindV1::CompactGroup,
        NonAtomVertexKindV1::MoleculeText,
        NonAtomVertexKindV1::Query,
    ] {
        let count = facts
            .non_atoms()
            .iter()
            .filter(|value| value.kind() == selected_kind)
            .count();
        if count == 0 {
            continue;
        }
        let kind = match selected_kind {
            NonAtomVertexKindV1::CompactGroup => "group",
            NonAtomVertexKindV1::MoleculeText => "molecule text",
            NonAtomVertexKindV1::Query => "query",
        };
        return Err(MoleculeGraphLoweringWithAtomError::Lowering(
            MoleculeGraphLoweringError::UnsupportedVertex { kind, count },
        ));
    }
    let mut atoms = Vec::new();
    atoms.try_reserve_exact(facts.atoms().len()).map_err(|_| {
        MoleculeGraphLoweringWithAtomError::Lowering(MoleculeGraphLoweringError::ResourceAllocation)
    })?;
    let mut points = Vec::new();
    if facts.include_coordinates() {
        points.try_reserve_exact(facts.atoms().len()).map_err(|_| {
            MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::ResourceAllocation,
            )
        })?;
    }
    for (atom_index, atom) in facts.atoms().iter().enumerate() {
        for (fact, present) in [
            ("authored valence", atom.valence().is_some()),
            ("authored multiplicity", atom.multiplicity().is_some()),
            ("authored free sites", atom.free_sites().is_some()),
        ] {
            if present {
                return Err(MoleculeGraphLoweringWithAtomError::Lowering(
                    MoleculeGraphLoweringError::UnsupportedAtomFact { atom_index, fact },
                ));
            }
        }
        let element = atom
            .element()
            .ok_or(MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::MissingElement { atom_index },
            ))?;
        let atomic_number = AtomicNumber::from_symbol(element).map_err(|source| {
            MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::InvalidElement {
                    atom_index,
                    element: element.to_owned(),
                    source,
                },
            )
        })?;
        atoms.push(
            MolAtom::new(
                atomic_number,
                atom.formal_charge(),
                atom.isotope(),
                atom.explicit_hydrogens(),
                false,
            )
            .map_err(|error| {
                MoleculeGraphLoweringWithAtomError::Lowering(MoleculeGraphLoweringError::Graph(
                    error,
                ))
            })?,
        );
        on_validated_atom(atom_index).map_err(MoleculeGraphLoweringWithAtomError::AtomCallback)?;
        if facts.include_coordinates() {
            points.push(
                Point2::new(atom.position().x(), -atom.position().y()).map_err(|error| {
                    MoleculeGraphLoweringWithAtomError::Lowering(MoleculeGraphLoweringError::Graph(
                        error,
                    ))
                })?,
            );
        }
    }
    let mut bonds = Vec::new();
    bonds.try_reserve_exact(facts.bonds().len()).map_err(|_| {
        MoleculeGraphLoweringWithAtomError::Lowering(MoleculeGraphLoweringError::ResourceAllocation)
    })?;
    let mut edges = Vec::new();
    edges.try_reserve_exact(facts.bonds().len()).map_err(|_| {
        MoleculeGraphLoweringWithAtomError::Lowering(MoleculeGraphLoweringError::ResourceAllocation)
    })?;
    for (bond_index, bond) in facts.bonds().iter().enumerate() {
        if bond
            .style()
            .is_some_and(|style| style != &BondStyle::Normal)
        {
            return Err(MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::UnsupportedBondStyle { bond_index },
            ));
        }
        let endpoint = |endpoint| match endpoint {
            DirectMoleculeGraphEndpoint::Atom(index) if index < facts.atoms().len() => Ok(index),
            _ => Err(MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::UnsupportedBondEndpoint { bond_index },
            )),
        };
        let start = endpoint(bond.start())?;
        let end = endpoint(bond.end())?;
        let order = match bond.order() {
            Some(BondOrder::Single) => ChemistryBondOrder::Single,
            Some(BondOrder::Double) => ChemistryBondOrder::Double,
            Some(BondOrder::Triple) => ChemistryBondOrder::Triple,
            Some(BondOrder::Aromatic | BondOrder::Other(_)) | None => {
                return Err(MoleculeGraphLoweringWithAtomError::Lowering(
                    MoleculeGraphLoweringError::UnsupportedBondOrder { bond_index },
                ));
            }
        };
        bonds.push(MolBond::new(start, end, order, false));
        edges.push((start, end));
    }
    Ok(LoweredMoleculeGraph {
        graph: MolGraph::new(
            atoms,
            bonds,
            facts
                .include_coordinates()
                .then(|| Coordinates::new(points)),
        )
        .map_err(|error| {
            MoleculeGraphLoweringWithAtomError::Lowering(MoleculeGraphLoweringError::Graph(error))
        })?,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document_projection::{
        DirectMoleculeGraphAtomFact, DirectMoleculeGraphAtomInput, DirectMoleculeGraphBondFact,
        NonAtomVertexFact, Point3V1,
    };

    fn atom(element: Option<&str>) -> DirectMoleculeGraphAtomFact {
        DirectMoleculeGraphAtomFact::new(DirectMoleculeGraphAtomInput {
            element: element.map(str::to_owned),
            position: Point3V1::new(2.0, 3.0, 0.0).expect("finite test point"),
            formal_charge: Some(1),
            isotope: Some(13),
            explicit_hydrogens: Some(1),
            valence: None,
            multiplicity: None,
            free_sites: None,
        })
    }

    fn facts(
        atoms: Vec<DirectMoleculeGraphAtomFact>,
        bonds: Vec<DirectMoleculeGraphBondFact>,
        non_atoms: Vec<NonAtomVertexFact>,
    ) -> DirectMoleculeGraphFacts {
        DirectMoleculeGraphFacts::new(atoms, bonds, non_atoms, true)
    }

    #[test]
    fn lowers_supported_bond_orders_and_flips_document_y_coordinates() {
        for order in [BondOrder::Single, BondOrder::Double, BondOrder::Triple] {
            let lowered = lower_direct_molecule_graph(&facts(
                vec![atom(Some("C")), atom(Some("O"))],
                vec![DirectMoleculeGraphBondFact::new(
                    DirectMoleculeGraphEndpoint::Atom(0),
                    DirectMoleculeGraphEndpoint::Atom(1),
                    Some(order),
                    None,
                )],
                Vec::new(),
            ))
            .expect("supported bond lowers");
            let (graph, edges) = lowered.into_parts();
            assert_eq!(graph.atoms().len(), 2);
            assert_eq!(edges, vec![(0, 1)]);
            assert_eq!(
                graph.coordinates().expect("coordinates requested").points()[0].y(),
                -3.0
            );
        }
    }

    #[test]
    fn refuses_every_non_atom_kind_and_closed_malformed_inputs() {
        for kind in [
            NonAtomVertexKindV1::CompactGroup,
            NonAtomVertexKindV1::MoleculeText,
            NonAtomVertexKindV1::Query,
        ] {
            assert!(matches!(
                lower_direct_molecule_graph(&facts(
                    vec![atom(Some("C"))],
                    Vec::new(),
                    vec![NonAtomVertexFact::new(kind, 1)],
                )),
                Err(MoleculeGraphLoweringError::UnsupportedVertex { .. })
            ));
        }
        assert!(matches!(
            lower_direct_molecule_graph(&facts(Vec::new(), Vec::new(), Vec::new())),
            Err(MoleculeGraphLoweringError::EmptyMolecule)
        ));
        assert!(matches!(
            lower_direct_molecule_graph(&facts(vec![atom(None)], Vec::new(), Vec::new())),
            Err(MoleculeGraphLoweringError::MissingElement { .. })
        ));
        assert!(matches!(
            lower_direct_molecule_graph(&facts(vec![atom(Some("Nope"))], Vec::new(), Vec::new())),
            Err(MoleculeGraphLoweringError::InvalidElement { .. })
        ));
        for endpoint in [
            DirectMoleculeGraphEndpoint::NonAtom,
            DirectMoleculeGraphEndpoint::Unknown,
            DirectMoleculeGraphEndpoint::Missing,
            DirectMoleculeGraphEndpoint::Atom(9),
        ] {
            assert!(matches!(
                lower_direct_molecule_graph(&facts(
                    vec![atom(Some("C"))],
                    vec![DirectMoleculeGraphBondFact::new(
                        endpoint,
                        DirectMoleculeGraphEndpoint::Atom(0),
                        Some(BondOrder::Single),
                        None
                    )],
                    Vec::new(),
                )),
                Err(MoleculeGraphLoweringError::UnsupportedBondEndpoint { .. })
            ));
        }
        assert!(matches!(
            lower_direct_molecule_graph(&facts(
                vec![atom(Some("C"))],
                vec![DirectMoleculeGraphBondFact::new(
                    DirectMoleculeGraphEndpoint::Atom(0),
                    DirectMoleculeGraphEndpoint::Atom(0),
                    Some(BondOrder::Single),
                    Some(BondStyle::Wedge)
                )],
                Vec::new(),
            )),
            Err(MoleculeGraphLoweringError::UnsupportedBondStyle { .. })
        ));
        assert!(matches!(
            lower_direct_molecule_graph(&facts(
                vec![atom(Some("C"))],
                vec![DirectMoleculeGraphBondFact::new(
                    DirectMoleculeGraphEndpoint::Atom(0),
                    DirectMoleculeGraphEndpoint::Atom(0),
                    None,
                    None
                )],
                Vec::new(),
            )),
            Err(MoleculeGraphLoweringError::UnsupportedBondOrder { .. })
        ));
    }

    #[test]
    fn chooses_closed_non_atom_category_precedence_regardless_of_source_order() {
        let refusal = lower_direct_molecule_graph(&facts(
            vec![atom(Some("C"))],
            Vec::new(),
            vec![
                NonAtomVertexFact::new(NonAtomVertexKindV1::Query, 1),
                NonAtomVertexFact::new(NonAtomVertexKindV1::MoleculeText, 2),
                NonAtomVertexFact::new(NonAtomVertexKindV1::CompactGroup, 3),
                NonAtomVertexFact::new(NonAtomVertexKindV1::CompactGroup, 4),
            ],
        ));
        assert!(matches!(
            refusal,
            Err(MoleculeGraphLoweringError::UnsupportedVertex {
                kind: "group",
                count: 2
            })
        ));
    }

    #[test]
    fn validates_atom_facts_before_observer_and_coordinates_or_bonds() {
        let invalid_atom = atom(None);
        let malformed_bond = DirectMoleculeGraphBondFact::new(
            DirectMoleculeGraphEndpoint::Missing,
            DirectMoleculeGraphEndpoint::Atom(0),
            Some(BondOrder::Aromatic),
            Some(BondStyle::Wedge),
        );
        let mut observed = false;
        let result = lower_direct_molecule_graph_with_validated_atom(
            &facts(vec![invalid_atom], vec![malformed_bond], Vec::new()),
            |_| {
                observed = true;
                Ok::<(), ()>(())
            },
        );
        assert!(matches!(
            result,
            Err(MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::MissingElement { atom_index: 0 }
            ))
        ));
        assert!(!observed);

        let mut callbacks = Vec::new();
        let result = lower_direct_molecule_graph_with_validated_atom(
            &facts(
                vec![atom(Some("C"))],
                vec![DirectMoleculeGraphBondFact::new(
                    DirectMoleculeGraphEndpoint::Missing,
                    DirectMoleculeGraphEndpoint::Atom(0),
                    Some(BondOrder::Single),
                    None,
                )],
                Vec::new(),
            ),
            |atom_index| {
                callbacks.push(atom_index);
                Ok::<(), ()>(())
            },
        );
        assert!(matches!(
            result,
            Err(MoleculeGraphLoweringWithAtomError::Lowering(
                MoleculeGraphLoweringError::UnsupportedBondEndpoint { bond_index: 0 }
            ))
        ));
        assert_eq!(callbacks, vec![0]);
    }

    #[test]
    fn refuses_authored_atom_fields_and_unsupported_bond_orders() {
        for field in ["valence", "multiplicity", "free_sites"] {
            let mut input = DirectMoleculeGraphAtomInput {
                element: Some("C".to_owned()),
                position: Point3V1::new(0.0, 0.0, 0.0).expect("finite point"),
                formal_charge: None,
                isotope: None,
                explicit_hydrogens: None,
                valence: None,
                multiplicity: None,
                free_sites: None,
            };
            match field {
                "valence" => input.valence = Some(4),
                "multiplicity" => input.multiplicity = Some(1),
                "free_sites" => input.free_sites = Some(1),
                _ => unreachable!(),
            }
            assert!(matches!(
                lower_direct_molecule_graph(&facts(
                    vec![DirectMoleculeGraphAtomFact::new(input)],
                    Vec::new(),
                    Vec::new(),
                )),
                Err(MoleculeGraphLoweringError::UnsupportedAtomFact { .. })
            ));
        }
        for order in [BondOrder::Aromatic, BondOrder::Other(7)] {
            assert!(matches!(
                lower_direct_molecule_graph(&facts(
                    vec![atom(Some("C")), atom(Some("O"))],
                    vec![DirectMoleculeGraphBondFact::new(
                        DirectMoleculeGraphEndpoint::Atom(0),
                        DirectMoleculeGraphEndpoint::Atom(1),
                        Some(order),
                        None,
                    )],
                    Vec::new(),
                )),
                Err(MoleculeGraphLoweringError::UnsupportedBondOrder { bond_index: 0 })
            ));
        }
    }
}
