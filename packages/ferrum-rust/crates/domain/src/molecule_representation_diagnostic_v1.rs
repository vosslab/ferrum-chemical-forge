//! Pure source-representation diagnostics for one immutable molecule.

use ferrum_core::{Bond, BondOrder, Molecule, NonAtomVertex};

use crate::{
    MoleculeDiagnosticCodeV1, MoleculeDiagnosticFindingErrorV1, MoleculeDiagnosticFindingV1,
    MoleculeDiagnosticLocationV1, MoleculeDiagnosticRecoveryV1, MoleculeDiagnosticSeverityV1,
};

/// Diagnose source facts that require a more explicit chemical representation.
///
/// The returned findings preserve the molecule's text, group, then bond source
/// order. The scanner borrows its input and neither normalizes nor repairs it.
pub fn diagnose_molecule_representation_v1(
    molecule: &Molecule,
) -> Result<Vec<MoleculeDiagnosticFindingV1>, MoleculeDiagnosticFindingErrorV1> {
    let finding_capacity = molecule
        .texts()
        .len()
        .saturating_add(molecule.groups().len())
        .saturating_add(molecule.bonds().len());
    let mut findings = Vec::new();
    findings
        .try_reserve_exact(finding_capacity)
        .map_err(|_| MoleculeDiagnosticFindingErrorV1::ResourceAllocation)?;

    for vertex in molecule.texts() {
        findings.push(vertex_finding(
            vertex,
            MoleculeDiagnosticCodeV1::TextAtomPresent,
        )?);
    }
    for vertex in molecule.groups() {
        findings.push(vertex_finding(
            vertex,
            MoleculeDiagnosticCodeV1::UnexpandedGroupPresent,
        )?);
    }
    for bond in molecule.bonds() {
        if bond.order() == Some(BondOrder::Other(0)) {
            findings.push(zero_order_bond_finding(bond)?);
        }
    }
    Ok(findings)
}

fn vertex_finding(
    vertex: &NonAtomVertex,
    code: MoleculeDiagnosticCodeV1,
) -> Result<MoleculeDiagnosticFindingV1, MoleculeDiagnosticFindingErrorV1> {
    MoleculeDiagnosticFindingV1::new(
        MoleculeDiagnosticSeverityV1::Warning,
        code,
        MoleculeDiagnosticRecoveryV1::ChooseSupportedRepresentation,
        MoleculeDiagnosticLocationV1::Vertex {
            source_identifier: source_identifier(vertex.source_id()),
        },
        None,
    )
}

fn zero_order_bond_finding(
    bond: &Bond,
) -> Result<MoleculeDiagnosticFindingV1, MoleculeDiagnosticFindingErrorV1> {
    MoleculeDiagnosticFindingV1::new(
        MoleculeDiagnosticSeverityV1::Warning,
        MoleculeDiagnosticCodeV1::ZeroOrderBond,
        MoleculeDiagnosticRecoveryV1::CorrectChemicalFacts,
        MoleculeDiagnosticLocationV1::Bond {
            source_identifier: source_identifier(bond.source_id()),
        },
        None,
    )
}

fn source_identifier(identifier: Option<&ferrum_core::Identifier>) -> Option<String> {
    identifier.map(|value| value.as_str().to_owned())
}

#[cfg(test)]
mod molecule_representation_diagnostic_v1_tests {
    use ferrum_core::{
        Atom, Bond, BondOrder, Identifier, Molecule, NonAtomVertex, Position, RecordKind, VertexRef,
    };

    use super::*;

    fn atom(identifier: &str, x: f64) -> Atom {
        Atom::new(
            Some(Identifier::new(identifier).expect("identifier")),
            Some("C".to_owned()),
            Position::new(x, 0.0, 0.0).expect("position"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .expect("atom")
    }

    fn vertex(kind: RecordKind, source_identifier: Option<&str>) -> NonAtomVertex {
        NonAtomVertex::new(
            kind,
            source_identifier.map(|value| Identifier::new(value).expect("identifier")),
            source_identifier.is_none().then_some(0),
        )
        .expect("vertex")
    }

    fn bond(
        source_identifier: Option<&str>,
        start: &Atom,
        end: &Atom,
        order: Option<BondOrder>,
    ) -> Bond {
        Bond::new(
            source_identifier.map(|value| Identifier::new(value).expect("identifier")),
            VertexRef::Atom(start.identity().clone()),
            VertexRef::Atom(end.identity().clone()),
            None,
            order,
            None,
            Some(false),
            source_identifier.is_none().then_some(0),
        )
        .expect("bond")
    }

    fn molecule(
        atoms: Vec<Atom>,
        groups: Vec<NonAtomVertex>,
        texts: Vec<NonAtomVertex>,
        bonds: Vec<Bond>,
    ) -> Molecule {
        Molecule::new(
            Some(Identifier::new("molecule").expect("identifier")),
            None,
            atoms,
            groups,
            texts,
            Vec::new(),
            bonds,
            None,
        )
        .expect("molecule")
    }

    #[test]
    fn reports_text_and_group_vertices_in_their_source_orders() {
        let molecule = molecule(
            Vec::new(),
            vec![
                vertex(RecordKind::Group, Some("group-second")),
                vertex(RecordKind::Group, Some("group-first")),
            ],
            vec![
                vertex(RecordKind::Text, Some("text-second")),
                vertex(RecordKind::Text, Some("text-first")),
            ],
            Vec::new(),
        );

        let findings = diagnose_molecule_representation_v1(&molecule).expect("findings");
        let observed: Vec<_> = findings
            .iter()
            .map(|finding| (finding.code(), finding.location().clone()))
            .collect();
        assert_eq!(
            observed,
            vec![
                (
                    MoleculeDiagnosticCodeV1::TextAtomPresent,
                    MoleculeDiagnosticLocationV1::Vertex {
                        source_identifier: Some("text-second".to_owned()),
                    },
                ),
                (
                    MoleculeDiagnosticCodeV1::TextAtomPresent,
                    MoleculeDiagnosticLocationV1::Vertex {
                        source_identifier: Some("text-first".to_owned()),
                    },
                ),
                (
                    MoleculeDiagnosticCodeV1::UnexpandedGroupPresent,
                    MoleculeDiagnosticLocationV1::Vertex {
                        source_identifier: Some("group-second".to_owned()),
                    },
                ),
                (
                    MoleculeDiagnosticCodeV1::UnexpandedGroupPresent,
                    MoleculeDiagnosticLocationV1::Vertex {
                        source_identifier: Some("group-first".to_owned()),
                    },
                ),
            ]
        );
    }

    #[test]
    fn reports_only_explicit_zero_order_bonds_in_source_order() {
        let left = atom("left", 0.0);
        let right = atom("right", 1.0);
        let molecule = molecule(
            vec![left.clone(), right.clone()],
            Vec::new(),
            Vec::new(),
            vec![
                bond(None, &left, &right, None),
                bond(
                    Some("zero-second"),
                    &left,
                    &right,
                    Some(BondOrder::Other(0)),
                ),
                bond(Some("single"), &left, &right, Some(BondOrder::Single)),
            ],
        );

        let findings = diagnose_molecule_representation_v1(&molecule).expect("findings");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code(), MoleculeDiagnosticCodeV1::ZeroOrderBond);
        assert_eq!(
            findings[0].location(),
            &MoleculeDiagnosticLocationV1::Bond {
                source_identifier: Some("zero-second".to_owned()),
            }
        );
        assert_eq!(
            findings[0].recovery(),
            MoleculeDiagnosticRecoveryV1::CorrectChemicalFacts
        );
    }

    #[test]
    fn accepts_clean_ordinary_input_without_mutation() {
        let left = atom("left", 0.0);
        let right = atom("right", 1.0);
        let molecule = molecule(
            vec![left.clone(), right.clone()],
            Vec::new(),
            Vec::new(),
            vec![bond(Some("single"), &left, &right, Some(BondOrder::Single))],
        );
        let original = molecule.clone();

        assert!(
            diagnose_molecule_representation_v1(&molecule)
                .expect("findings")
                .is_empty()
        );
        assert_eq!(molecule, original);
    }

    #[test]
    fn reports_idless_subjects_without_derived_debug_identity() {
        let left = atom("left", 0.0);
        let right = atom("right", 1.0);
        let molecule = molecule(
            vec![left.clone(), right.clone()],
            vec![vertex(RecordKind::Group, None)],
            vec![vertex(RecordKind::Text, None)],
            vec![bond(None, &left, &right, Some(BondOrder::Other(0)))],
        );

        let findings = diagnose_molecule_representation_v1(&molecule).expect("findings");
        let observed: Vec<_> = findings
            .iter()
            .map(|finding| finding.location().clone())
            .collect();
        assert_eq!(
            observed,
            vec![
                MoleculeDiagnosticLocationV1::Vertex {
                    source_identifier: None,
                },
                MoleculeDiagnosticLocationV1::Vertex {
                    source_identifier: None,
                },
                MoleculeDiagnosticLocationV1::Bond {
                    source_identifier: None,
                },
            ]
        );
        assert!(findings.iter().all(|finding| finding.detail().is_none()));
    }
}
