//! CDML stereo validation and serialization for molecule insertion.

use std::collections::BTreeMap;

use xot::{Node, Xot};

use super::super::{
    CDML_NAMESPACE, DocumentBondOrderV1, DocumentBondPresentationV1,
    DocumentDirectedBondDepictionV1, DocumentDoubleBondCarrierMarkDepictionV1,
    DocumentDoubleBondCarrierMarkV1, DocumentDoubleBondConfigurationV1, DocumentDoubleBondStereoV1,
    DocumentStereoDepictionReportV1, DocumentStereoLigandV1, DocumentStereoSemanticReportV1,
    DocumentTetrahedralParityV1, DocumentTetrahedralStereoV1, MoleculeInsertionAtomV1,
    MoleculeInsertionBondV1, MoleculeInsertionRequestV1, MoleculeInsertionV1, Point3V1,
    TypedDocumentError, element_name,
};
use super::InsertionNames;

#[derive(Clone)]
pub(super) struct MoleculeStereoReports {
    pub(super) semantics: Option<DocumentStereoSemanticReportV1>,
    pub(super) depictions: Option<DocumentStereoDepictionReportV1>,
}

struct MoleculeStereoGraph {
    explicit_hydrogens: Vec<Option<u16>>,
    bonds: Vec<MoleculeStereoGraphBond>,
}

struct MoleculeStereoGraphBond {
    start: usize,
    end: usize,
    order: DocumentBondOrderV1,
}

/// Validate every first-class stereo child before a typed document is admitted.
///
/// The typed record recognizer deliberately preserves unknown CDML for general
/// structural fidelity. This canonical child is different: it carries chemistry
/// facts, so accepting malformed content as opaque would silently lose meaning.
pub(crate) fn validate_document_stereo_semantics(
    indexed: &super::super::IndexedDocument,
) -> Result<(), TypedDocumentError> {
    let tree = &indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .expect("an indexed CDML document has a document element");
    for molecule in tree
        .children(root)
        .filter(|node| is_cdml_element(tree, *node, "molecule"))
    {
        let _ = decode_molecule_stereo_reports(tree, molecule)?;
    }
    Ok(())
}

pub(super) fn decode_molecule_stereo_reports(
    tree: &Xot,
    molecule: Node,
) -> Result<MoleculeStereoReports, TypedDocumentError> {
    let atom_nodes = tree
        .children(molecule)
        .filter(|node| is_cdml_element(tree, *node, "atom"))
        .collect::<Vec<_>>();
    let atoms = atom_nodes.len();
    let bonds: Vec<Node> = tree
        .children(molecule)
        .filter(|node| is_cdml_element(tree, *node, "bond"))
        .collect();
    let semantic_nodes: Vec<Node> = tree
        .children(molecule)
        .filter(|node| is_cdml_element(tree, *node, "stereoSemantics"))
        .collect();
    if semantic_nodes.len() > 1 {
        return Err(TypedDocumentError::UnsupportedStereoSemantics {
            field: "stereoSemantics multiplicity",
        });
    }
    let semantics = semantic_nodes
        .into_iter()
        .next()
        .map(|semantic| decode_molecule_stereo_semantics(tree, semantic, atoms, &bonds))
        .transpose()?;
    let depiction_nodes = tree
        .children(molecule)
        .filter(|node| is_cdml_element(tree, *node, "stereoDepictions"))
        .collect::<Vec<_>>();
    if depiction_nodes.len() > 1 {
        return Err(TypedDocumentError::UnsupportedStereoSemantics {
            field: "stereoDepictions multiplicity",
        });
    }
    let depictions = depiction_nodes
        .into_iter()
        .next()
        .map(|depiction| decode_molecule_stereo_depictions(tree, depiction, atoms, &bonds))
        .transpose()?;
    if semantics.is_none() && depictions.is_none() {
        return Ok(MoleculeStereoReports {
            semantics: None,
            depictions: None,
        });
    }
    let request = MoleculeInsertionRequestV1::with_stereo_reports(
        stereo_validation_molecule(tree, &atom_nodes, &bonds)?,
        semantics,
        depictions,
    )
    .map_err(|_| TypedDocumentError::InvalidStereoSemantics)?;
    require_ez_depictions(request.stereo_semantics(), request.stereo_depictions())?;
    Ok(MoleculeStereoReports {
        semantics: request.stereo_semantics().cloned(),
        depictions: request.stereo_depictions().cloned(),
    })
}

fn decode_molecule_stereo_semantics(
    tree: &Xot,
    semantic: Node,
    atoms: usize,
    bonds: &[Node],
) -> Result<DocumentStereoSemanticReportV1, TypedDocumentError> {
    require_exact_attributes(tree, semantic, &[])?;
    let mut tetrahedral = Vec::new();
    let mut double_bonds = Vec::new();
    for child in tree.children(semantic) {
        if tree
            .text_str(child)
            .is_some_and(|text| !text.trim().is_empty())
        {
            return Err(TypedDocumentError::MalformedStereoSemantics {
                field: "stereoSemantics text",
            });
        }
        if is_cdml_element(tree, child, "tetrahedral") {
            tetrahedral.push(decode_tetrahedral(tree, child, atoms)?);
        } else if is_cdml_element(tree, child, "doubleBond") {
            double_bonds.push(decode_double_bond(tree, child, atoms, bonds)?);
        } else if element_name(tree, child).is_some() {
            return Err(TypedDocumentError::UnsupportedStereoSemantics {
                field: "stereoSemantics child",
            });
        }
    }
    Ok(DocumentStereoSemanticReportV1::new(
        tetrahedral,
        double_bonds,
    ))
}

fn decode_molecule_stereo_depictions(
    tree: &Xot,
    depiction: Node,
    atoms: usize,
    bonds: &[Node],
) -> Result<DocumentStereoDepictionReportV1, TypedDocumentError> {
    require_exact_attributes(tree, depiction, &[])?;
    let mut directed_bonds = Vec::new();
    let mut double_bond_carrier_marks = Vec::new();
    for child in tree.children(depiction) {
        if tree
            .text_str(child)
            .is_some_and(|text| !text.trim().is_empty())
        {
            return Err(TypedDocumentError::MalformedStereoSemantics {
                field: "stereoDepictions text",
            });
        }
        if is_cdml_element(tree, child, "tetrahedralDirectedBond") {
            directed_bonds.push(decode_tetrahedral_directed_bond(tree, child, atoms, bonds)?);
        } else if is_cdml_element(tree, child, "doubleBondCarrierMark") {
            double_bond_carrier_marks.push(decode_double_bond_carrier_mark(tree, child)?);
        } else if element_name(tree, child).is_some() {
            return Err(TypedDocumentError::UnsupportedStereoSemantics {
                field: "stereoDepictions child",
            });
        }
    }
    let report = DocumentStereoDepictionReportV1::new(directed_bonds, double_bond_carrier_marks);
    if report.is_empty() {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    }
    Ok(report)
}

fn decode_tetrahedral_directed_bond(
    tree: &Xot,
    node: Node,
    atoms: usize,
    bonds: &[Node],
) -> Result<DocumentDirectedBondDepictionV1, TypedDocumentError> {
    require_exact_attributes(tree, node, &["bondIndex", "start", "end", "presentation"])?;
    reject_element_children(tree, node, "tetrahedralDirectedBond child")?;
    let bond_index = required_usize(tree, node, "bondIndex")?;
    let start = required_usize(tree, node, "start")?;
    let end = required_usize(tree, node, "end")?;
    if start >= atoms || end >= atoms || bonds.get(bond_index).is_none() {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    }
    let presentation = match required_attribute(tree, node, "presentation")? {
        "w1" => DocumentBondPresentationV1::SolidWedge,
        "h1" => DocumentBondPresentationV1::HashedWedge,
        _ => {
            return Err(TypedDocumentError::UnsupportedStereoSemantics {
                field: "presentation",
            });
        }
    };
    Ok(DocumentDirectedBondDepictionV1::new(
        bond_index,
        start,
        end,
        presentation,
    ))
}

fn decode_double_bond_carrier_mark(
    tree: &Xot,
    node: Node,
) -> Result<DocumentDoubleBondCarrierMarkDepictionV1, TypedDocumentError> {
    require_exact_attributes(tree, node, &["doubleBondIndex", "carrierBondIndex", "mark"])?;
    reject_element_children(tree, node, "doubleBondCarrierMark child")?;
    let mark = match required_attribute(tree, node, "mark")? {
        "up" => DocumentDoubleBondCarrierMarkV1::Up,
        "down" => DocumentDoubleBondCarrierMarkV1::Down,
        _ => return Err(TypedDocumentError::UnsupportedStereoSemantics { field: "mark" }),
    };
    Ok(DocumentDoubleBondCarrierMarkDepictionV1::new(
        required_usize(tree, node, "doubleBondIndex")?,
        required_usize(tree, node, "carrierBondIndex")?,
        mark,
    ))
}

fn molecule_stereo_graph_facts(
    tree: &Xot,
    atoms: &[Node],
    bonds: &[Node],
) -> Result<MoleculeStereoGraph, TypedDocumentError> {
    let mut atom_indices = BTreeMap::new();
    let mut explicit_hydrogens = Vec::with_capacity(atoms.len());
    for (index, atom) in atoms.iter().enumerate() {
        let Some(id) = attribute(tree, *atom, "id") else {
            return Err(TypedDocumentError::InvalidStereoSemantics);
        };
        if atom_indices.insert(id, index).is_some() {
            return Err(TypedDocumentError::InvalidStereoSemantics);
        }
        let hydrogens = match attribute(tree, *atom, "explicit_hydrogens") {
            Some(value) => match value.parse::<u16>() {
                Ok(value) if value > 0 => Some(value),
                _ => return Err(TypedDocumentError::InvalidStereoSemantics),
            },
            None => None,
        };
        explicit_hydrogens.push(hydrogens);
    }
    let mut graph_bonds = Vec::with_capacity(bonds.len());
    for bond in bonds {
        let (Some(start_id), Some(end_id)) = (
            attribute(tree, *bond, "start"),
            attribute(tree, *bond, "end"),
        ) else {
            return Err(TypedDocumentError::InvalidStereoSemantics);
        };
        let (Some(start), Some(end)) = (atom_indices.get(start_id), atom_indices.get(end_id))
        else {
            return Err(TypedDocumentError::InvalidStereoSemantics);
        };
        let order = match attribute(tree, *bond, "type") {
            Some("n1" | "w1" | "h1") => DocumentBondOrderV1::Single,
            Some("n2") => DocumentBondOrderV1::Double,
            Some("n3") => DocumentBondOrderV1::Triple,
            _ => return Err(TypedDocumentError::InvalidStereoSemantics),
        };
        graph_bonds.push(MoleculeStereoGraphBond {
            start: *start,
            end: *end,
            order,
        });
    }
    Ok(MoleculeStereoGraph {
        explicit_hydrogens,
        bonds: graph_bonds,
    })
}

fn stereo_validation_molecule(
    tree: &Xot,
    atoms: &[Node],
    bonds: &[Node],
) -> Result<MoleculeInsertionV1, TypedDocumentError> {
    let graph = molecule_stereo_graph_facts(tree, atoms, bonds)?;
    let atoms = graph
        .explicit_hydrogens
        .into_iter()
        .map(|explicit_hydrogens| {
            MoleculeInsertionAtomV1::new(
                "C",
                Point3V1::new(0.0, 0.0, 0.0).expect("literal origin is finite"),
                None,
                None,
                explicit_hydrogens,
            )
            .map_err(|_| TypedDocumentError::InvalidStereoSemantics)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = graph
        .bonds
        .into_iter()
        .map(|bond| MoleculeInsertionBondV1::new(bond.start, bond.end, bond.order))
        .collect();
    MoleculeInsertionV1::new(atoms, bonds).map_err(|_| TypedDocumentError::InvalidStereoSemantics)
}

pub(super) fn require_ez_depictions(
    semantics: Option<&DocumentStereoSemanticReportV1>,
    depictions: Option<&DocumentStereoDepictionReportV1>,
) -> Result<(), TypedDocumentError> {
    let Some(semantics) = semantics else {
        return Ok(());
    };
    if semantics.double_bonds().is_empty() {
        return Ok(());
    }
    let Some(depictions) = depictions else {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    };
    if semantics.double_bonds().iter().any(|double_bond| {
        !depictions
            .double_bond_carrier_marks()
            .iter()
            .any(|mark| mark.double_bond_index() == double_bond.bond_index())
    }) {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    }
    Ok(())
}

fn decode_tetrahedral(
    tree: &Xot,
    node: Node,
    atoms: usize,
) -> Result<DocumentTetrahedralStereoV1, TypedDocumentError> {
    require_exact_attributes(tree, node, &["center", "ligands", "parity"])?;
    reject_element_children(tree, node, "tetrahedral child")?;
    let center = required_usize(tree, node, "center")?;
    if center >= atoms {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    }
    let ligands = required_attribute(tree, node, "ligands")?
        .split(',')
        .map(|value| match value {
            "H" => Ok(DocumentStereoLigandV1::ExplicitHydrogen),
            _ => {
                let index = parse_usize(value, "ligands")?;
                if index >= atoms {
                    return Err(TypedDocumentError::InvalidStereoSemantics);
                }
                Ok(DocumentStereoLigandV1::Atom(index))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ligands: [DocumentStereoLigandV1; 4] = ligands
        .try_into()
        .map_err(|_| TypedDocumentError::MalformedStereoSemantics { field: "ligands" })?;
    let parity = match required_attribute(tree, node, "parity")? {
        "clockwise" => DocumentTetrahedralParityV1::Clockwise,
        "counterClockwise" => DocumentTetrahedralParityV1::CounterClockwise,
        _ => {
            return Err(TypedDocumentError::UnsupportedStereoSemantics { field: "parity" });
        }
    };
    DocumentTetrahedralStereoV1::new(center, ligands, parity)
        .map_err(|_| TypedDocumentError::InvalidStereoSemantics)
}

fn decode_double_bond(
    tree: &Xot,
    node: Node,
    atoms: usize,
    bonds: &[Node],
) -> Result<DocumentDoubleBondStereoV1, TypedDocumentError> {
    require_exact_attributes(
        tree,
        node,
        &["bondIndex", "startLigand", "endLigand", "configuration"],
    )?;
    reject_element_children(tree, node, "doubleBond child")?;
    let bond_index = required_usize(tree, node, "bondIndex")?;
    let start_ligand = required_usize(tree, node, "startLigand")?;
    let end_ligand = required_usize(tree, node, "endLigand")?;
    let Some(bond) = bonds.get(bond_index) else {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    };
    if attribute(tree, *bond, "type") != Some("n2") || start_ligand >= atoms || end_ligand >= atoms
    {
        return Err(TypedDocumentError::InvalidStereoSemantics);
    }
    let configuration = match required_attribute(tree, node, "configuration")? {
        "E" => DocumentDoubleBondConfigurationV1::E,
        "Z" => DocumentDoubleBondConfigurationV1::Z,
        _ => {
            return Err(TypedDocumentError::UnsupportedStereoSemantics {
                field: "configuration",
            });
        }
    };
    DocumentDoubleBondStereoV1::new(bond_index, start_ligand, end_ligand, configuration)
        .map_err(|_| TypedDocumentError::InvalidStereoSemantics)
}

fn require_exact_attributes(
    tree: &Xot,
    node: Node,
    expected: &[&str],
) -> Result<(), TypedDocumentError> {
    if tree.attributes(node).len() != expected.len()
        || tree.attributes(node).iter().any(|(name, _)| {
            let (local, namespace) = tree.name_ns_str(name);
            !namespace.is_empty() || !expected.contains(&local)
        })
    {
        return Err(TypedDocumentError::UnsupportedStereoSemantics { field: "attribute" });
    }
    for name in expected {
        let count = tree
            .attributes(node)
            .iter()
            .filter(|(candidate, _)| tree.name_ns_str(*candidate).0 == *name)
            .count();
        if count != 1 {
            return Err(TypedDocumentError::MalformedStereoSemantics { field: "attribute" });
        }
    }
    Ok(())
}

fn reject_element_children(
    tree: &Xot,
    node: Node,
    field: &'static str,
) -> Result<(), TypedDocumentError> {
    for child in tree.children(node) {
        if element_name(tree, child).is_some()
            || tree
                .text_str(child)
                .is_some_and(|text| !text.trim().is_empty())
        {
            return Err(TypedDocumentError::UnsupportedStereoSemantics { field });
        }
    }
    Ok(())
}

fn required_attribute<'a>(
    tree: &'a Xot,
    node: Node,
    name: &'static str,
) -> Result<&'a str, TypedDocumentError> {
    attribute(tree, node, name).ok_or(TypedDocumentError::MalformedStereoSemantics { field: name })
}

fn required_usize(tree: &Xot, node: Node, name: &'static str) -> Result<usize, TypedDocumentError> {
    parse_usize(required_attribute(tree, node, name)?, name)
}

fn parse_usize(value: &str, field: &'static str) -> Result<usize, TypedDocumentError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(TypedDocumentError::MalformedStereoSemantics { field });
    }
    value
        .parse()
        .map_err(|_| TypedDocumentError::MalformedStereoSemantics { field })
}

pub(super) fn attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (namespace.is_empty() && local == expected).then_some(value.as_str())
    })
}

pub(super) fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(local, namespace)| local == expected && namespace == CDML_NAMESPACE)
}

pub(super) fn append_stereo_semantics(
    tree: &mut Xot,
    molecule_node: Node,
    names: &InsertionNames,
    molecule: &MoleculeInsertionV1,
    report: &super::DocumentStereoSemanticReportV1,
) -> Result<(), TypedDocumentError> {
    let report = report
        .clone()
        .canonicalize_for_molecule(molecule)
        .map_err(|_| TypedDocumentError::InvalidStereoSemantics)?;
    let semantic_node = tree.new_element(names.stereo_semantics);
    for tetrahedral in report.tetrahedral() {
        let mut encoded_ligands = Vec::new();
        for ligand in tetrahedral.ligands() {
            match ligand {
                DocumentStereoLigandV1::Atom(index) => {
                    encoded_ligands.push(index.to_string());
                }
                DocumentStereoLigandV1::ExplicitHydrogen => encoded_ligands.push("H".to_owned()),
            }
        }
        let node = tree.new_element(names.tetrahedral);
        tree.set_attribute(node, names.center, tetrahedral.center().to_string());
        tree.set_attribute(node, names.ligands, encoded_ligands.join(","));
        tree.set_attribute(
            node,
            names.parity,
            match tetrahedral.parity() {
                DocumentTetrahedralParityV1::Clockwise => "clockwise",
                DocumentTetrahedralParityV1::CounterClockwise => "counterClockwise",
            },
        );
        tree.append(semantic_node, node)
            .map_err(TypedDocumentError::Mutation)?;
    }
    for double_bond in report.double_bonds() {
        let node = tree.new_element(names.double_bond_stereo);
        tree.set_attribute(node, names.bond_index, double_bond.bond_index().to_string());
        tree.set_attribute(
            node,
            names.start_ligand,
            double_bond.start_ligand().to_string(),
        );
        tree.set_attribute(node, names.end_ligand, double_bond.end_ligand().to_string());
        tree.set_attribute(
            node,
            names.configuration,
            match double_bond.configuration() {
                DocumentDoubleBondConfigurationV1::E => "E",
                DocumentDoubleBondConfigurationV1::Z => "Z",
            },
        );
        tree.append(semantic_node, node)
            .map_err(TypedDocumentError::Mutation)?;
    }
    tree.append(molecule_node, semantic_node)
        .map_err(TypedDocumentError::Mutation)
}

pub(super) fn append_stereo_depictions(
    tree: &mut Xot,
    molecule_node: Node,
    names: &InsertionNames,
    report: &DocumentStereoDepictionReportV1,
) -> Result<(), TypedDocumentError> {
    let depiction_node = tree.new_element(names.stereo_depictions);
    for directed_bond in report.directed_bonds() {
        let node = tree.new_element(names.tetrahedral_directed_bond);
        tree.set_attribute(
            node,
            names.bond_index,
            directed_bond.bond_index().to_string(),
        );
        let (start, end) = directed_bond.endpoints();
        tree.set_attribute(node, names.start, start.to_string());
        tree.set_attribute(node, names.end, end.to_string());
        tree.set_attribute(
            node,
            names.presentation,
            match directed_bond.presentation() {
                DocumentBondPresentationV1::SolidWedge => "w1",
                DocumentBondPresentationV1::HashedWedge => "h1",
                DocumentBondPresentationV1::Normal(_) => {
                    return Err(TypedDocumentError::InvalidStereoSemantics);
                }
            },
        );
        tree.append(depiction_node, node)
            .map_err(TypedDocumentError::Mutation)?;
    }
    for mark in report.double_bond_carrier_marks() {
        let node = tree.new_element(names.double_bond_carrier_mark);
        tree.set_attribute(
            node,
            names.double_bond_index,
            mark.double_bond_index().to_string(),
        );
        tree.set_attribute(
            node,
            names.carrier_bond_index,
            mark.carrier_bond_index().to_string(),
        );
        tree.set_attribute(
            node,
            names.mark,
            match mark.mark() {
                DocumentDoubleBondCarrierMarkV1::Up => "up",
                DocumentDoubleBondCarrierMarkV1::Down => "down",
            },
        );
        tree.append(depiction_node, node)
            .map_err(TypedDocumentError::Mutation)?;
    }
    tree.append(molecule_node, depiction_node)
        .map_err(TypedDocumentError::Mutation)
}
