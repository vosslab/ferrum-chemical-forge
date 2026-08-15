//! Durable, explicit-only fragment annotations owned by one molecule.

use std::collections::HashSet;

use thiserror::Error;
use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, DocumentObjectIdV1, PersistentId, TypedClass, TypedDocument,
    TypedDocumentError, element_name,
};

/// One supported explicit fragment record, expressed without retained XML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExplicitFragmentRecordV1 {
    fragment_id: PersistentId,
    name: String,
    molecule_id: DocumentObjectIdV1,
    bond_ids: Vec<PersistentId>,
    atom_ids: Vec<PersistentId>,
}

impl DocumentExplicitFragmentRecordV1 {
    #[must_use]
    pub fn fragment_id(&self) -> &PersistentId {
        &self.fragment_id
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub fn bond_ids(&self) -> &[PersistentId] {
        &self.bond_ids
    }
    #[must_use]
    pub fn atom_ids(&self) -> &[PersistentId] {
        &self.atom_ids
    }
}

/// Frozen list facts for the exact retained document revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentExplicitFragmentObservationV1 {
    records: Vec<DocumentExplicitFragmentRecordV1>,
    has_retained_fragment_metadata: bool,
}

impl DocumentExplicitFragmentObservationV1 {
    #[must_use]
    pub fn records(&self) -> &[DocumentExplicitFragmentRecordV1] {
        &self.records
    }
    #[must_use]
    pub const fn has_retained_fragment_metadata(&self) -> bool {
        self.has_retained_fragment_metadata
    }
}

/// Candidate construction failures at the explicit-fragment boundary.
#[derive(Debug, Error)]
pub enum DocumentExplicitFragmentErrorV1 {
    #[error("explicit fragment requires a nonblank name")]
    BlankName,
    #[error("explicit fragment requires at least one selected atom or bond")]
    EmptySelection,
    #[error("explicit fragment selection contains a duplicate member")]
    DuplicateSelection,
    #[error("explicit fragment molecule selector is not one direct molecule")]
    InvalidMolecule,
    #[error(
        "explicit fragment member is not one direct ordinary atom or bond in that molecule: {0}"
    )]
    InvalidMember(PersistentId),
    #[error("explicit fragment bond has unsupported direct endpoints: {0}")]
    InvalidBond(PersistentId),
    #[error("explicit fragment identifier is already retained: {0}")]
    DuplicateFragmentId(PersistentId),
    #[error("explicit fragment could not reserve admitted document-derived storage")]
    ResourceExhausted,
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
}

/// Fully checked detached candidate facts, before a session supplies an ID.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplicitFragmentCandidateV1 {
    name: String,
    molecule_id: DocumentObjectIdV1,
    bond_ids: Vec<PersistentId>,
    atom_ids: Vec<PersistentId>,
}

impl ExplicitFragmentCandidateV1 {
    pub(crate) fn record(&self, fragment_id: PersistentId) -> DocumentExplicitFragmentRecordV1 {
        DocumentExplicitFragmentRecordV1 {
            fragment_id,
            name: self.name.clone(),
            molecule_id: self.molecule_id.clone(),
            bond_ids: self.bond_ids.clone(),
            atom_ids: self.atom_ids.clone(),
        }
    }
}

impl TypedDocument {
    pub(crate) fn prepare_explicit_fragment_v1(
        &self,
        molecule_id: &DocumentObjectIdV1,
        name: &str,
        selected_atom_ids: &[PersistentId],
        selected_bond_ids: &[PersistentId],
    ) -> Result<ExplicitFragmentCandidateV1, DocumentExplicitFragmentErrorV1> {
        let name = copy_trimmed_name(name)?;
        if selected_atom_ids.is_empty() && selected_bond_ids.is_empty() {
            return Err(DocumentExplicitFragmentErrorV1::EmptySelection);
        }
        let molecule_record = self
            .resolve_document_object_id(molecule_id)
            .filter(|record| {
                record.class() == TypedClass::Molecule && record.path().components().len() == 1
            })
            .ok_or(DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let source_id = molecule_record
            .attribute("id")
            .ok_or(DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let source_id = PersistentId::new(source_id.to_owned())
            .map_err(|_| DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let tree = &self.indexed().xml.tree;
        let molecule = direct_molecule(tree, self.indexed().xml.document, &source_id)
            .ok_or(DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let (atom_order, bond_order, endpoints) = molecule_members(tree, molecule)?;
        let mut atom_selected = selected_set(selected_atom_ids)?;
        let bond_selected = selected_set(selected_bond_ids)?;
        for bond in &bond_selected {
            let (start, end) = endpoints
                .get(bond)
                .ok_or_else(|| DocumentExplicitFragmentErrorV1::InvalidMember(bond.clone()))?;
            atom_selected.insert(start.clone());
            atom_selected.insert(end.clone());
        }
        if !atom_selected
            .iter()
            .all(|id| atom_order.iter().any(|candidate| candidate == id))
        {
            let id = atom_selected
                .iter()
                .find(|id| !atom_order.iter().any(|candidate| candidate == *id))
                .expect("failed membership has an ID");
            return Err(DocumentExplicitFragmentErrorV1::InvalidMember(id.clone()));
        }
        if !bond_selected
            .iter()
            .all(|id| bond_order.iter().any(|candidate| candidate == id))
        {
            let id = bond_selected
                .iter()
                .find(|id| !bond_order.iter().any(|candidate| candidate == *id))
                .expect("failed membership has an ID");
            return Err(DocumentExplicitFragmentErrorV1::InvalidMember(id.clone()));
        }
        Ok(ExplicitFragmentCandidateV1 {
            name,
            molecule_id: molecule_id.clone(),
            bond_ids: bond_order
                .into_iter()
                .filter(|id| bond_selected.contains(id))
                .collect(),
            atom_ids: atom_order
                .into_iter()
                .filter(|id| atom_selected.contains(id))
                .collect(),
        })
    }

    pub(crate) fn apply_explicit_fragment_v1(
        &self,
        candidate: &ExplicitFragmentCandidateV1,
        fragment_id: &PersistentId,
    ) -> Result<TypedDocument, DocumentExplicitFragmentErrorV1> {
        if self.indexed().resolve_id(fragment_id).is_some() {
            return Err(DocumentExplicitFragmentErrorV1::DuplicateFragmentId(
                fragment_id.clone(),
            ));
        }
        let mut detached = self.detached_candidate()?;
        let indexed = detached.detached_indexed_mut();
        let record = indexed.xml.tree.document_element(indexed.xml.document).ok();
        let source = self
            .resolve_document_object_id(candidate.molecule_id())
            .and_then(|record| record.attribute("id"));
        let source = source.ok_or(DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let molecule_id = PersistentId::new(source.to_owned())
            .map_err(|_| DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let molecule = direct_molecule(&indexed.xml.tree, indexed.xml.document, &molecule_id)
            .ok_or(DocumentExplicitFragmentErrorV1::InvalidMolecule)?;
        let _ = record;
        write_record(&mut indexed.xml.tree, molecule, fragment_id, candidate)?;
        let xml = detached.to_xml().map_err(TypedDocumentError::Serialize)?;
        TypedDocument::parse(&xml).map_err(Into::into)
    }
}

impl ExplicitFragmentCandidateV1 {
    fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
}

/// Observe only exact V1 records; all other retained fragment metadata is summarized.
pub fn observe_explicit_fragments_v1(
    document: &TypedDocument,
) -> DocumentExplicitFragmentObservationV1 {
    let mut records = Vec::new();
    let mut retained = false;
    for molecule in document.root().children_of(TypedClass::Molecule) {
        let Some(molecule_id) = DocumentObjectIdV1::from_record(molecule) else {
            continue;
        };
        for fragment in molecule.children_of(TypedClass::Fragment) {
            match exact_record(fragment, &molecule_id, document) {
                Some(record) => records.push(record),
                None => retained = true,
            }
        }
    }
    DocumentExplicitFragmentObservationV1 {
        records,
        has_retained_fragment_metadata: retained,
    }
}

fn exact_record(
    fragment: &super::TypedRecord,
    molecule_id: &DocumentObjectIdV1,
    document: &TypedDocument,
) -> Option<DocumentExplicitFragmentRecordV1> {
    if fragment.attribute("type") != Some("explicit")
        || fragment.typed_attributes().len() != 2
        || !fragment.unknown_attributes().is_empty()
        || !fragment.diagnostics().is_empty()
        || !only_whitespace(fragment)
    {
        return None;
    }
    let fragment_id = PersistentId::new(fragment.attribute("id")?.to_owned()).ok()?;
    let mut children = fragment.typed_children().iter();
    let name_record = children.next()?.record();
    if name_record.class() != TypedClass::FragmentName
        || !name_record.typed_attributes().is_empty()
        || !name_record.unknown_attributes().is_empty()
        || !name_record.typed_children().is_empty()
        || !name_record.unrecognized_children().is_empty()
        || !name_record.diagnostics().is_empty()
    {
        return None;
    }
    let name = name_record.text_content();
    if name.trim().is_empty() || name != name.trim() {
        return None;
    }
    let mut bonds = Vec::new();
    let mut atoms = Vec::new();
    let mut seen = HashSet::new();
    let mut saw_atom = false;
    for child in children {
        let child = child.record();
        if child.typed_attributes().len() != 1
            || !child.unknown_attributes().is_empty()
            || !child.diagnostics().is_empty()
            || !child.typed_children().is_empty()
            || !only_whitespace(child)
        {
            return None;
        }
        let id = PersistentId::new(child.attribute("id")?.to_owned()).ok()?;
        if !seen.insert(id.clone()) {
            return None;
        }
        match child.class() {
            TypedClass::FragmentBond if !saw_atom => bonds.push(id),
            TypedClass::FragmentVertex => {
                saw_atom = true;
                atoms.push(id);
            }
            _ => return None,
        }
    }
    if atoms.is_empty() || !members_prove(document, molecule_id, &bonds, &atoms) {
        return None;
    }
    Some(DocumentExplicitFragmentRecordV1 {
        fragment_id,
        name,
        molecule_id: molecule_id.clone(),
        bond_ids: bonds,
        atom_ids: atoms,
    })
}

fn members_prove(
    document: &TypedDocument,
    molecule_id: &DocumentObjectIdV1,
    bonds: &[PersistentId],
    atoms: &[PersistentId],
) -> bool {
    let Ok(candidate) = document.prepare_explicit_fragment_v1(molecule_id, "x", atoms, bonds)
    else {
        return false;
    };
    candidate.bond_ids == bonds && candidate.atom_ids == atoms
}

fn only_whitespace(record: &super::TypedRecord) -> bool {
    record.unrecognized_children().iter().all(|child| matches!(child.node(), super::UnrecognizedNode::Text(text) if text.trim().is_empty()))
}

fn selected_set(
    values: &[PersistentId],
) -> Result<HashSet<PersistentId>, DocumentExplicitFragmentErrorV1> {
    let mut result = HashSet::new();
    result
        .try_reserve(values.len())
        .map_err(|_| DocumentExplicitFragmentErrorV1::ResourceExhausted)?;
    for value in values {
        if !result.insert(value.clone()) {
            return Err(DocumentExplicitFragmentErrorV1::DuplicateSelection);
        }
    }
    Ok(result)
}

type Endpoints = std::collections::HashMap<PersistentId, (PersistentId, PersistentId)>;
fn molecule_members(
    tree: &Xot,
    molecule: Node,
) -> Result<(Vec<PersistentId>, Vec<PersistentId>, Endpoints), DocumentExplicitFragmentErrorV1> {
    let count = tree.children(molecule).count();
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    let mut endpoints = Endpoints::new();
    atoms
        .try_reserve_exact(count)
        .map_err(|_| DocumentExplicitFragmentErrorV1::ResourceExhausted)?;
    bonds
        .try_reserve_exact(count)
        .map_err(|_| DocumentExplicitFragmentErrorV1::ResourceExhausted)?;
    endpoints
        .try_reserve(count)
        .map_err(|_| DocumentExplicitFragmentErrorV1::ResourceExhausted)?;
    for node in tree.children(molecule) {
        if is_core(tree, node, "atom") {
            atoms.push(node_id(tree, node, None)?);
        }
    }
    for node in tree.children(molecule) {
        if !is_core(tree, node, "bond") {
            continue;
        }
        let id = node_id(tree, node, None)?;
        let start = node_id(tree, node, Some("start"))
            .map_err(|_| DocumentExplicitFragmentErrorV1::InvalidBond(id.clone()))?;
        let end = node_id(tree, node, Some("end"))
            .map_err(|_| DocumentExplicitFragmentErrorV1::InvalidBond(id.clone()))?;
        if !atoms.contains(&start) || !atoms.contains(&end) {
            return Err(DocumentExplicitFragmentErrorV1::InvalidBond(id));
        }
        bonds.push(id.clone());
        endpoints.insert(id, (start, end));
    }
    Ok((atoms, bonds, endpoints))
}

fn write_record(
    tree: &mut Xot,
    molecule: Node,
    fragment_id: &PersistentId,
    candidate: &ExplicitFragmentCandidateV1,
) -> Result<(), DocumentExplicitFragmentErrorV1> {
    let namespace = element_name(tree, molecule)
        .map(|(_, namespace)| namespace)
        .unwrap_or_default();
    let fragment = new_element(tree, "fragment", &namespace);
    let id = tree.add_name("id");
    let kind = tree.add_name("type");
    tree.set_attribute(fragment, id, fragment_id.as_str());
    tree.set_attribute(fragment, kind, "explicit");
    let name = new_element(tree, "name", &namespace);
    tree.append(fragment, name)
        .map_err(TypedDocumentError::Mutation)?;
    let text = tree.new_text(&candidate.name);
    tree.append(name, text)
        .map_err(TypedDocumentError::Mutation)?;
    for bond in &candidate.bond_ids {
        append_member(tree, fragment, "bond", bond, &namespace)?;
    }
    for atom in &candidate.atom_ids {
        append_member(tree, fragment, "vertex", atom, &namespace)?;
    }
    tree.append(molecule, fragment)
        .map_err(TypedDocumentError::Mutation)?;
    Ok(())
}
fn append_member(
    tree: &mut Xot,
    fragment: Node,
    kind: &str,
    value: &PersistentId,
    namespace: &str,
) -> Result<(), DocumentExplicitFragmentErrorV1> {
    let node = new_element(tree, kind, namespace);
    let id = tree.add_name("id");
    tree.set_attribute(node, id, value.as_str());
    tree.append(fragment, node)
        .map_err(TypedDocumentError::Mutation)?;
    Ok(())
}
fn new_element(tree: &mut Xot, local: &str, namespace: &str) -> Node {
    let name = if namespace.is_empty() {
        tree.add_name(local)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local, namespace)
    };
    tree.new_element(name)
}
fn direct_molecule(tree: &Xot, document: Node, id: &PersistentId) -> Option<Node> {
    let root = tree.document_element(document).ok()?;
    exactly_one(tree.children(root).filter(|node| {
        is_core(tree, *node, "molecule") && attr(tree, *node, "id") == Some(id.as_str())
    }))
}
fn exactly_one(mut nodes: impl Iterator<Item = Node>) -> Option<Node> {
    let node = nodes.next()?;
    nodes.next().is_none().then_some(node)
}
fn node_id(
    tree: &Xot,
    node: Node,
    field: Option<&str>,
) -> Result<PersistentId, DocumentExplicitFragmentErrorV1> {
    PersistentId::new(
        attr(tree, node, field.unwrap_or("id"))
            .ok_or(DocumentExplicitFragmentErrorV1::InvalidMolecule)?
            .to_owned(),
    )
    .map_err(|_| DocumentExplicitFragmentErrorV1::InvalidMolecule)
}
fn attr<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (local == expected && namespace.is_empty()).then_some(value.as_str())
    })
}
fn is_core(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(name, namespace)| {
        name == expected && (namespace.is_empty() || namespace == CDML_NAMESPACE)
    })
}
fn copy_trimmed_name(value: &str) -> Result<String, DocumentExplicitFragmentErrorV1> {
    let value = value.trim();
    if value.is_empty() {
        return Err(DocumentExplicitFragmentErrorV1::BlankName);
    }
    let mut result = String::new();
    result
        .try_reserve_exact(value.len())
        .map_err(|_| DocumentExplicitFragmentErrorV1::ResourceExhausted)?;
    result.push_str(value);
    Ok(result)
}
