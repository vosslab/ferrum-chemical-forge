//! Structured XML mutation for complete, validated molecule insertions.

use std::collections::{BTreeMap, BTreeSet};

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, DocumentBondOrderV1, DocumentBondPresentationV1,
    DocumentDirectedBondDepictionV1, DocumentDoubleBondCarrierMarkDepictionV1,
    DocumentDoubleBondCarrierMarkV1, DocumentDoubleBondConfigurationV1, DocumentDoubleBondStereoV1,
    DocumentObjectIdV1, DocumentStereoDepictionReportV1, DocumentStereoLigandV1,
    DocumentStereoSemanticReportV1, DocumentTetrahedralParityV1, DocumentTetrahedralStereoV1,
    INTERCHANGE_IMPORT_NAMESPACE_V1, InterchangeRecordInsertionV1, MoleculeInsertionAtomV1,
    MoleculeInsertionBondV1, MoleculeInsertionRequestV1, MoleculeInsertionV1, PersistentId,
    Point3V1, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    /// Return the admitted durable stereo semantics for one direct molecule.
    ///
    /// The descriptor is decoded from the retained canonical CDML rather than
    /// inferred from drawing wedges, so reopening a document retains source facts.
    pub fn molecule_stereo_semantics_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<Option<DocumentStereoSemanticReportV1>, TypedDocumentError> {
        self.molecule_stereo_reports_v1(molecule_id)
            .map(|reports| reports.and_then(|(semantics, _)| semantics))
    }

    /// Return the admitted durable stereo drawing facts for one direct molecule.
    ///
    /// This typed observation keeps projection and rendering consumers out of
    /// the raw CDML tree. A carrier mark remains a drawing fact and does not
    /// derive E/Z chemistry from geometry.
    pub fn molecule_stereo_depictions_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<Option<DocumentStereoDepictionReportV1>, TypedDocumentError> {
        self.molecule_stereo_reports_v1(molecule_id)
            .map(|reports| reports.and_then(|(_, depictions)| depictions))
    }

    fn molecule_stereo_reports_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
    ) -> Result<
        Option<(
            Option<DocumentStereoSemanticReportV1>,
            Option<DocumentStereoDepictionReportV1>,
        )>,
        TypedDocumentError,
    > {
        let Some(record) = self.resolve_document_object_id(molecule_id) else {
            return Ok(None);
        };
        if record.class() != super::TypedClass::Molecule || record.path().components().len() != 1 {
            return Ok(None);
        }
        let Some(source_id) = record.attribute("id") else {
            return Ok(None);
        };
        let tree = &self.indexed().xml.tree;
        let root = tree
            .document_element(self.indexed().xml.document)
            .expect("a parsed CDML document has a document element");
        let molecule = tree.children(root).find(|node| {
            is_cdml_element(tree, *node, "molecule")
                && attribute(tree, *node, "id") == Some(source_id)
        });
        let Some(molecule) = molecule else {
            return Ok(None);
        };
        decode_molecule_stereo_reports(tree, molecule).map(Some)
    }

    pub(crate) fn with_insert_molecule(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        molecule: &MoleculeInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(
            molecule_id,
            atom_ids,
            bond_ids,
            molecule,
            None,
            None,
            None,
        )
    }

    pub(crate) fn with_insert_molecule_request(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        request: &MoleculeInsertionRequestV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(
            molecule_id,
            atom_ids,
            bond_ids,
            request.molecule(),
            None,
            request.stereo_semantics(),
            request.stereo_depictions(),
        )
    }

    pub(crate) fn with_insert_interchange_record(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        record: &InterchangeRecordInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(
            molecule_id,
            atom_ids,
            bond_ids,
            record.request().molecule(),
            Some(record),
            record.request().stereo_semantics(),
            record.request().stereo_depictions(),
        )
    }

    fn with_insert_molecule_record(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        molecule: &MoleculeInsertionV1,
        interchange_record: Option<&InterchangeRecordInsertionV1>,
        stereo_semantics: Option<&super::DocumentStereoSemanticReportV1>,
        stereo_depictions: Option<&super::DocumentStereoDepictionReportV1>,
    ) -> Result<Self, TypedDocumentError> {
        if atom_ids.len() != molecule.atoms().len() || bond_ids.len() != molecule.bonds().len() {
            return Err(TypedDocumentError::InsertionIdentityCountMismatch);
        }
        let mut supplied_ids = BTreeSet::new();
        for identifier in std::iter::once(molecule_id)
            .chain(atom_ids.iter())
            .chain(bond_ids.iter())
        {
            if self.indexed().resolve_id(identifier).is_some() || !supplied_ids.insert(identifier) {
                return Err(TypedDocumentError::DuplicateInsertionId(identifier.clone()));
            }
        }

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let namespace = valid_cdml_namespace(namespace);
        let names = InsertionNames::new(&mut indexed.xml.tree, namespace);
        let molecule_node = indexed.xml.tree.new_element(names.molecule);
        indexed
            .xml
            .tree
            .set_attribute(molecule_node, names.id, molecule_id.as_str());
        if let Some(name) = molecule.name() {
            indexed
                .xml
                .tree
                .set_attribute(molecule_node, names.name, name);
        } else if let Some(record) = interchange_record
            && !record.title().is_empty()
            && xml_attribute_safe(record.title())
        {
            indexed
                .xml
                .tree
                .set_attribute(molecule_node, names.name, record.title());
        }

        for (atom_id, atom) in atom_ids.iter().zip(molecule.atoms()) {
            append_atom(&mut indexed.xml.tree, molecule_node, &names, atom_id, atom)?;
        }
        for (bond_id, bond) in bond_ids.iter().zip(molecule.bonds()) {
            let bond_node = indexed.xml.tree.new_element(names.bond);
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.id, bond_id.as_str());
            indexed.xml.tree.set_attribute(
                bond_node,
                names.bond_type,
                bond.presentation().cdml_token(),
            );
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.start, atom_ids[bond.start()].as_str());
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.end, atom_ids[bond.end()].as_str());
            indexed
                .xml
                .tree
                .append(molecule_node, bond_node)
                .map_err(TypedDocumentError::Mutation)?;
        }
        if let Some(record) = interchange_record {
            append_interchange_metadata(&mut indexed.xml.tree, molecule_node, record)?;
        }
        let request = MoleculeInsertionRequestV1::with_stereo_reports(
            molecule.clone(),
            stereo_semantics.cloned(),
            stereo_depictions.cloned(),
        )
        .map_err(|_| TypedDocumentError::InvalidStereoSemantics)?;
        require_ez_depictions(request.stereo_semantics(), request.stereo_depictions())?;
        if let Some(stereo_semantics) = request.stereo_semantics() {
            append_stereo_semantics(
                &mut indexed.xml.tree,
                molecule_node,
                &names,
                molecule,
                stereo_semantics,
            )?;
        }
        if let Some(stereo_depictions) = request.stereo_depictions() {
            append_stereo_depictions(
                &mut indexed.xml.tree,
                molecule_node,
                &names,
                stereo_depictions,
            )?;
        }
        indexed
            .xml
            .tree
            .append(root, molecule_node)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}

/// Validate every first-class stereo child before a typed document is admitted.
///
/// The typed record recognizer deliberately preserves unknown CDML for general
/// structural fidelity. This canonical child is different: it carries chemistry
/// facts, so accepting malformed content as opaque would silently lose meaning.
pub(crate) fn validate_document_stereo_semantics(
    indexed: &super::IndexedDocument,
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

fn decode_molecule_stereo_reports(
    tree: &Xot,
    molecule: Node,
) -> Result<
    (
        Option<DocumentStereoSemanticReportV1>,
        Option<DocumentStereoDepictionReportV1>,
    ),
    TypedDocumentError,
> {
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
        return Ok((None, None));
    }
    let request = MoleculeInsertionRequestV1::with_stereo_reports(
        stereo_validation_molecule(tree, &atom_nodes, &bonds)?,
        semantics,
        depictions,
    )
    .map_err(|_| TypedDocumentError::InvalidStereoSemantics)?;
    require_ez_depictions(request.stereo_semantics(), request.stereo_depictions())?;
    Ok((
        request.stereo_semantics().cloned(),
        request.stereo_depictions().cloned(),
    ))
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
            double_bonds.push(decode_double_bond(tree, child, atoms, &bonds)?);
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
    DocumentDirectedBondDepictionV1::new(bond_index, start, end, presentation)
        .map_err(|_| TypedDocumentError::InvalidStereoSemantics)
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
) -> Result<(Vec<Option<u16>>, Vec<(usize, usize, DocumentBondOrderV1)>), TypedDocumentError> {
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
        graph_bonds.push((*start, *end, order));
    }
    Ok((explicit_hydrogens, graph_bonds))
}

fn stereo_validation_molecule(
    tree: &Xot,
    atoms: &[Node],
    bonds: &[Node],
) -> Result<MoleculeInsertionV1, TypedDocumentError> {
    let (explicit_hydrogens, graph_bonds) = molecule_stereo_graph_facts(tree, atoms, bonds)?;
    let atoms = explicit_hydrogens
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
    let bonds = graph_bonds
        .into_iter()
        .map(|(start, end, order)| MoleculeInsertionBondV1::new(start, end, order))
        .collect();
    MoleculeInsertionV1::new(atoms, bonds).map_err(|_| TypedDocumentError::InvalidStereoSemantics)
}

fn require_ez_depictions(
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

fn attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (namespace.is_empty() && local == expected).then_some(value.as_str())
    })
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(local, namespace)| local == expected && namespace == CDML_NAMESPACE)
}

pub(crate) fn valid_cdml_namespace(namespace: String) -> String {
    if namespace == CDML_NAMESPACE {
        namespace
    } else {
        unreachable!("TypedDocument accepts only no-namespace or CDML roots")
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InsertionNames {
    pub(crate) molecule: xot::NameId,
    pub(crate) atom: xot::NameId,
    pub(crate) point: xot::NameId,
    pub(crate) bond: xot::NameId,
    pub(crate) id: xot::NameId,
    pub(crate) name: xot::NameId,
    pub(crate) charge: xot::NameId,
    pub(crate) isotope: xot::NameId,
    pub(crate) explicit_hydrogens: xot::NameId,
    pub(crate) x: xot::NameId,
    pub(crate) y: xot::NameId,
    pub(crate) z: xot::NameId,
    pub(crate) bond_type: xot::NameId,
    pub(crate) start: xot::NameId,
    pub(crate) end: xot::NameId,
    pub(crate) haworth_position: xot::NameId,
    pub(crate) stereo_semantics: xot::NameId,
    pub(crate) stereo_depictions: xot::NameId,
    pub(crate) tetrahedral: xot::NameId,
    pub(crate) double_bond_stereo: xot::NameId,
    pub(crate) tetrahedral_directed_bond: xot::NameId,
    pub(crate) double_bond_carrier_mark: xot::NameId,
    pub(crate) center: xot::NameId,
    pub(crate) ligands: xot::NameId,
    pub(crate) parity: xot::NameId,
    pub(crate) bond_index: xot::NameId,
    pub(crate) start_ligand: xot::NameId,
    pub(crate) end_ligand: xot::NameId,
    pub(crate) configuration: xot::NameId,
    pub(crate) double_bond_index: xot::NameId,
    pub(crate) carrier_bond_index: xot::NameId,
    pub(crate) presentation: xot::NameId,
    pub(crate) mark: xot::NameId,
}

impl InsertionNames {
    pub(crate) fn new(tree: &mut Xot, namespace: String) -> Self {
        Self {
            molecule: element_name_id(tree, "molecule", &namespace),
            atom: element_name_id(tree, "atom", &namespace),
            point: element_name_id(tree, "point", &namespace),
            bond: element_name_id(tree, "bond", &namespace),
            id: tree.add_name("id"),
            name: tree.add_name("name"),
            charge: tree.add_name("charge"),
            isotope: tree.add_name("isotope"),
            explicit_hydrogens: tree.add_name("explicit_hydrogens"),
            x: tree.add_name("x"),
            y: tree.add_name("y"),
            z: tree.add_name("z"),
            bond_type: tree.add_name("type"),
            start: tree.add_name("start"),
            end: tree.add_name("end"),
            haworth_position: tree.add_name("haworth_position"),
            stereo_semantics: element_name_id(tree, "stereoSemantics", &namespace),
            stereo_depictions: element_name_id(tree, "stereoDepictions", &namespace),
            tetrahedral: element_name_id(tree, "tetrahedral", &namespace),
            double_bond_stereo: element_name_id(tree, "doubleBond", &namespace),
            tetrahedral_directed_bond: element_name_id(tree, "tetrahedralDirectedBond", &namespace),
            double_bond_carrier_mark: element_name_id(tree, "doubleBondCarrierMark", &namespace),
            center: tree.add_name("center"),
            ligands: tree.add_name("ligands"),
            parity: tree.add_name("parity"),
            bond_index: tree.add_name("bondIndex"),
            start_ligand: tree.add_name("startLigand"),
            end_ligand: tree.add_name("endLigand"),
            configuration: tree.add_name("configuration"),
            double_bond_index: tree.add_name("doubleBondIndex"),
            carrier_bond_index: tree.add_name("carrierBondIndex"),
            presentation: tree.add_name("presentation"),
            mark: tree.add_name("mark"),
        }
    }
}

fn append_stereo_semantics(
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

fn append_stereo_depictions(
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

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}

pub(crate) fn append_atom(
    tree: &mut Xot,
    molecule_node: Node,
    names: &InsertionNames,
    atom_id: &PersistentId,
    atom: &super::MoleculeInsertionAtomV1,
) -> Result<(), TypedDocumentError> {
    let atom_node = tree.new_element(names.atom);
    tree.set_attribute(atom_node, names.id, atom_id.as_str());
    tree.set_attribute(atom_node, names.name, atom.element());
    set_optional_attribute(tree, atom_node, names.charge, atom.formal_charge());
    set_optional_attribute(tree, atom_node, names.isotope, atom.isotope());
    set_optional_attribute(
        tree,
        atom_node,
        names.explicit_hydrogens,
        atom.explicit_hydrogens(),
    );
    let point_node = tree.new_element(names.point);
    let position = atom.position();
    tree.set_attribute(point_node, names.x, position.x().to_string());
    tree.set_attribute(point_node, names.y, position.y().to_string());
    tree.set_attribute(point_node, names.z, position.z().to_string());
    tree.append(atom_node, point_node)
        .map_err(TypedDocumentError::Mutation)?;
    tree.append(molecule_node, atom_node)
        .map_err(TypedDocumentError::Mutation)
}

fn set_optional_attribute<T: ToString>(
    tree: &mut Xot,
    node: Node,
    name: xot::NameId,
    value: Option<T>,
) {
    if let Some(value) = value {
        tree.set_attribute(node, name, value.to_string());
    }
}

fn append_interchange_metadata(
    tree: &mut Xot,
    molecule_node: Node,
    record: &InterchangeRecordInsertionV1,
) -> Result<(), TypedDocumentError> {
    let namespace = tree.add_namespace(INTERCHANGE_IMPORT_NAMESPACE_V1);
    let record_name = tree.add_name_ns("interchange-record", namespace);
    let property_name = tree.add_name_ns("property", namespace);
    let encoding_name = tree.add_name("encoding");
    let title_name = tree.add_name("title");
    let name_name = tree.add_name("name");
    let value_name = tree.add_name("value");

    let record_node = tree.new_element(record_name);
    let prefix = tree.add_prefix("ferrum-interchange");
    tree.set_namespace(record_node, prefix, namespace);
    tree.set_attribute(record_node, encoding_name, "utf8-hex-v1");
    tree.set_attribute(record_node, title_name, utf8_hex(record.title()));
    for property in record.properties() {
        let property_node = tree.new_element(property_name);
        tree.set_attribute(property_node, name_name, utf8_hex(property.name()));
        tree.set_attribute(property_node, value_name, utf8_hex(property.value()));
        tree.append(record_node, property_node)
            .map_err(TypedDocumentError::Mutation)?;
    }
    tree.append(molecule_node, record_node)
        .map_err(TypedDocumentError::Mutation)
}

fn utf8_hex(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.bytes() {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn xml_attribute_safe(value: &str) -> bool {
    value.chars().all(|character| {
        matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
            || ('\u{20}'..='\u{D7FF}').contains(&character)
            || ('\u{E000}'..='\u{FFFD}').contains(&character)
            || ('\u{10000}'..='\u{10FFFF}').contains(&character)
    })
}
