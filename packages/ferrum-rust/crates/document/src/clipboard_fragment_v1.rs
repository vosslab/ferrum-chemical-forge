//! Read-only, revision-bound CDML fragment extraction for native Copy.

use std::collections::BTreeSet;

use thiserror::Error;
use xot::Node;

use super::{
    DocumentObjectIdV1, SessionDocumentObservationV1, TypedClass, TypedDocument,
    TypedDocumentError, TypedRecord, UnrecognizedNode, XmlSerializationError,
};

/// Stable schema identifier for [`DocumentClipboardFragmentV1`].
pub const DOCUMENT_CLIPBOARD_FRAGMENT_SCHEMA_V1: &str = "ferrum-document-clipboard-fragment-v1";

/// One nonempty, duplicate-free set of durable projected objects selected for Copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentClipboardSelectionV1 {
    objects: Vec<DocumentObjectIdV1>,
}

impl DocumentClipboardSelectionV1 {
    /// Validate one owned selection without assigning meaning to caller order.
    pub fn new(objects: Vec<DocumentObjectIdV1>) -> Result<Self, DocumentClipboardFragmentErrorV1> {
        if objects.is_empty() {
            return Err(DocumentClipboardFragmentErrorV1::EmptySelection);
        }
        for (index, object) in objects.iter().enumerate() {
            if objects[..index].contains(object) {
                return Err(DocumentClipboardFragmentErrorV1::DuplicateSelection);
            }
        }
        Ok(Self { objects })
    }

    /// Return the requested durable object selectors.
    #[must_use]
    pub fn objects(&self) -> &[DocumentObjectIdV1] {
        &self.objects
    }
}

/// Closed extraction mode selected from the complete durable selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentClipboardFragmentKindV1 {
    /// One connected atom/bond subset of one direct molecule.
    Structure,
    /// Complete selected direct roots in document order.
    TopLevel,
}

/// Immutable CDML fragment and exact source provenance produced for native Copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentClipboardFragmentV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    kind: DocumentClipboardFragmentKindV1,
    selected_objects: Vec<DocumentObjectIdV1>,
    copied_roots: Vec<DocumentObjectIdV1>,
    copied_atoms: Vec<DocumentObjectIdV1>,
    copied_bonds: Vec<DocumentObjectIdV1>,
    fragment_cdml: String,
}

impl DocumentClipboardFragmentV1 {
    /// Return the closed receipt schema.
    #[must_use]
    pub fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the authoritative source revision.
    #[must_use]
    pub fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the authoritative source digest.
    #[must_use]
    pub fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the selected extraction mode.
    #[must_use]
    pub fn kind(&self) -> DocumentClipboardFragmentKindV1 {
        self.kind
    }

    /// Return selected durable objects in canonical document order.
    #[must_use]
    pub fn selected_objects(&self) -> &[DocumentObjectIdV1] {
        &self.selected_objects
    }

    /// Return copied direct roots in document order.
    #[must_use]
    pub fn copied_roots(&self) -> &[DocumentObjectIdV1] {
        &self.copied_roots
    }

    /// Return copied direct atoms in molecule source order for a structural fragment.
    #[must_use]
    pub fn copied_atoms(&self) -> &[DocumentObjectIdV1] {
        &self.copied_atoms
    }

    /// Return copied direct bonds in molecule source order for a structural fragment.
    #[must_use]
    pub fn copied_bonds(&self) -> &[DocumentObjectIdV1] {
        &self.copied_bonds
    }

    /// Return the detached CDML fragment.
    #[must_use]
    pub fn fragment_cdml(&self) -> &str {
        &self.fragment_cdml
    }
}

/// Failure while extracting one exact native clipboard fragment.
#[derive(Debug, Error)]
pub enum DocumentClipboardFragmentErrorV1 {
    /// The caller supplied no selected durable object.
    #[error("clipboard Copy requires at least one durable selected object")]
    EmptySelection,
    /// The same durable selector occurred more than once.
    #[error("clipboard Copy selected object IDs must be unique")]
    DuplicateSelection,
    /// Snapshot and projection provenance did not describe one accepted state.
    #[error("clipboard Copy observation provenance disagrees")]
    ObservationProvenanceMismatch,
    /// A requested selector was absent or not selectable in the exact projection.
    #[error("clipboard Copy selection is not an exact durable projected object")]
    UnknownSelectedObject,
    /// A structural selection did not have one durable direct-root molecule.
    #[error("clipboard Copy structural selection has no durable direct molecule")]
    InvalidStructureRoot,
    /// The selected molecule is outside the closed partial-copy grammar.
    #[error("clipboard Copy molecule has unsupported partial-copy content")]
    UnsupportedStructure,
    /// The selected atom/bond subgraph was not connected after bond closure.
    #[error("clipboard Copy structural selection must be connected")]
    DisconnectedStructure,
    /// A retained typed snapshot could not be reconstructed.
    #[error("clipboard Copy could not parse the authoritative snapshot: {0}")]
    Typed(#[from] TypedDocumentError),
    /// The detached fragment could not be serialized.
    #[error("clipboard Copy could not serialize the detached fragment: {0}")]
    Serialize(#[from] XmlSerializationError),
    /// The retained XML tree refused one internal structural removal.
    #[error("clipboard Copy could not detach unselected XML content: {0}")]
    Mutation(#[source] xot::Error),
    /// A supposedly selected durable path no longer matched the re-parsed tree.
    #[error("clipboard Copy durable identity did not resolve in the retained tree")]
    IdentityInvariant,
    /// A selected-only fragment exceeded its normalized admitted source tree.
    #[error("clipboard Copy fragment exceeded its source-derived byte bound")]
    SourceBoundExceeded,
}

#[derive(Clone)]
enum SelectedFact {
    Structure {
        object: DocumentObjectIdV1,
        root: DocumentObjectIdV1,
        root_order: u32,
        child_order: u32,
        class: TypedClass,
    },
    Presentation {
        object: DocumentObjectIdV1,
        root_order: u32,
    },
}

impl SelectedFact {
    fn object(&self) -> &DocumentObjectIdV1 {
        match self {
            Self::Structure { object, .. } | Self::Presentation { object, .. } => object,
        }
    }

    fn order(&self) -> (u32, u32) {
        match self {
            Self::Structure {
                root_order,
                child_order,
                ..
            } => (*root_order, *child_order),
            Self::Presentation { root_order, .. } => (*root_order, 0),
        }
    }

    fn root(&self) -> &DocumentObjectIdV1 {
        match self {
            Self::Structure { root, .. } => root,
            Self::Presentation { object, .. } => object,
        }
    }
}

/// Extract one selected-only CDML fragment without mutating its source observation.
pub fn extract_document_clipboard_fragment_v1(
    observation: &SessionDocumentObservationV1,
    selection: DocumentClipboardSelectionV1,
) -> Result<DocumentClipboardFragmentV1, DocumentClipboardFragmentErrorV1> {
    authenticate_observation(observation)?;
    let mut selected = selected_facts(observation, selection.objects())?;
    selected.sort_by_key(SelectedFact::order);
    let canonical_selection = selected
        .iter()
        .map(|fact| fact.object().clone())
        .collect::<Vec<_>>();
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    let source_bytes = document.to_xml()?.len();
    let structure_root = one_structure_root(&selected);
    let extracted = match structure_root {
        Some(root) => extract_structure(document, root, &selected)?,
        None => extract_top_level(document, &selected)?,
    };
    if extracted.fragment_cdml.len() > source_bytes {
        return Err(DocumentClipboardFragmentErrorV1::SourceBoundExceeded);
    }
    TypedDocument::parse(&extracted.fragment_cdml)?;
    Ok(DocumentClipboardFragmentV1 {
        schema: DOCUMENT_CLIPBOARD_FRAGMENT_SCHEMA_V1,
        source_revision: observation.snapshot().revision(),
        source_digest: *observation.snapshot().digest(),
        kind: extracted.kind,
        selected_objects: canonical_selection,
        copied_roots: extracted.copied_roots,
        copied_atoms: extracted.copied_atoms,
        copied_bonds: extracted.copied_bonds,
        fragment_cdml: extracted.fragment_cdml,
    })
}

fn authenticate_observation(
    observation: &SessionDocumentObservationV1,
) -> Result<(), DocumentClipboardFragmentErrorV1> {
    let snapshot = observation.snapshot();
    let projection = observation.projection();
    if snapshot.revision() != projection.revision() || snapshot.digest() != projection.digest() {
        return Err(DocumentClipboardFragmentErrorV1::ObservationProvenanceMismatch);
    }
    Ok(())
}

fn selected_facts(
    observation: &SessionDocumentObservationV1,
    requested: &[DocumentObjectIdV1],
) -> Result<Vec<SelectedFact>, DocumentClipboardFragmentErrorV1> {
    let projection = observation.projection();
    let mut facts = Vec::new();
    for molecule in projection.molecules() {
        let root = molecule.id();
        for atom in molecule.atoms() {
            let Some(object) = atom.id().filter(|object| requested.contains(object)) else {
                continue;
            };
            let Some(root) = root else {
                return Err(DocumentClipboardFragmentErrorV1::InvalidStructureRoot);
            };
            facts.push(SelectedFact::Structure {
                object: object.clone(),
                root: root.clone(),
                root_order: molecule.source_order(),
                child_order: atom.source_order(),
                class: TypedClass::Atom,
            });
        }
        for bond in molecule.bonds() {
            let Some(object) = bond.id().filter(|object| requested.contains(object)) else {
                continue;
            };
            let Some(root) = root else {
                return Err(DocumentClipboardFragmentErrorV1::InvalidStructureRoot);
            };
            facts.push(SelectedFact::Structure {
                object: object.clone(),
                root: root.clone(),
                root_order: molecule.source_order(),
                child_order: bond.source_order(),
                class: TypedClass::Bond,
            });
        }
    }
    for root in projection.presentation_stack().roots() {
        let target = root.target();
        let Some(object) = target.id().filter(|object| requested.contains(object)) else {
            continue;
        };
        facts.push(SelectedFact::Presentation {
            object: object.clone(),
            root_order: target.source_order(),
        });
    }
    if facts.len() != requested.len() {
        return Err(DocumentClipboardFragmentErrorV1::UnknownSelectedObject);
    }
    Ok(facts)
}

fn one_structure_root(selected: &[SelectedFact]) -> Option<&DocumentObjectIdV1> {
    let SelectedFact::Structure { root, .. } = selected.first()? else {
        return None;
    };
    selected
        .iter()
        .all(|fact| matches!(fact, SelectedFact::Structure { root: other, .. } if other == root))
        .then_some(root)
}

struct ExtractedFragment {
    kind: DocumentClipboardFragmentKindV1,
    copied_roots: Vec<DocumentObjectIdV1>,
    copied_atoms: Vec<DocumentObjectIdV1>,
    copied_bonds: Vec<DocumentObjectIdV1>,
    fragment_cdml: String,
}

fn extract_top_level(
    mut document: TypedDocument,
    selected: &[SelectedFact],
) -> Result<ExtractedFragment, DocumentClipboardFragmentErrorV1> {
    let mut roots = selected
        .iter()
        .map(SelectedFact::root)
        .cloned()
        .collect::<Vec<_>>();
    roots.dedup();
    let mut keep_paths = Vec::new();
    for root in &roots {
        let Some(record) = document.resolve_document_object_id(root) else {
            return Err(DocumentClipboardFragmentErrorV1::IdentityInvariant);
        };
        if record.path().components().len() != 1 || !supported_root(record.class()) {
            return Err(DocumentClipboardFragmentErrorV1::IdentityInvariant);
        }
        keep_paths.push(record.path().components()[0]);
    }
    let indexed = document.detached_indexed_mut();
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .map_err(DocumentClipboardFragmentErrorV1::Mutation)?;
    retain_root_elements(tree, root, &keep_paths)?;
    let fragment_cdml = tree
        .to_string(indexed.xml.document)
        .map_err(XmlSerializationError::from)?;
    Ok(ExtractedFragment {
        kind: DocumentClipboardFragmentKindV1::TopLevel,
        copied_roots: roots,
        copied_atoms: Vec::new(),
        copied_bonds: Vec::new(),
        fragment_cdml,
    })
}

fn supported_root(class: TypedClass) -> bool {
    matches!(
        class,
        TypedClass::Molecule
            | TypedClass::CanvasArrow
            | TypedClass::CanvasPlus
            | TypedClass::CanvasText
            | TypedClass::Rectangle
            | TypedClass::Square
            | TypedClass::Oval
            | TypedClass::Circle
            | TypedClass::Polygon
            | TypedClass::Polyline
    )
}

fn extract_structure(
    mut document: TypedDocument,
    root_id: &DocumentObjectIdV1,
    selected: &[SelectedFact],
) -> Result<ExtractedFragment, DocumentClipboardFragmentErrorV1> {
    let root_record = document
        .resolve_document_object_id(root_id)
        .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
    validate_structure_root(root_record)?;
    let root_index = root_record.path().components()[0];
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    for child in root_record.typed_children() {
        match child.record().class() {
            TypedClass::Atom => atoms.push(child.record()),
            TypedClass::Bond => bonds.push(child.record()),
            TypedClass::Fragment
                if super::typed_linear_form_metadata::is_exact_generated_linear_form_record(
                    child.record(),
                ) => {}
            _ => return Err(DocumentClipboardFragmentErrorV1::UnsupportedStructure),
        }
    }
    validate_structure_graph(&atoms, &bonds)?;
    let selected_atoms = selected_object_set(selected, TypedClass::Atom);
    let selected_bonds = selected_object_set(selected, TypedClass::Bond);
    let (copied_atoms, copied_bonds, keep_children) =
        closed_structure(&atoms, &bonds, &selected_atoms, &selected_bonds)?;
    let indexed = document.detached_indexed_mut();
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .map_err(DocumentClipboardFragmentErrorV1::Mutation)?;
    let molecule = element_child(tree, root, root_index)
        .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
    retain_root_elements(tree, root, &[root_index])?;
    retain_element_children(tree, molecule, &keep_children)?;
    let fragment_cdml = tree
        .to_string(indexed.xml.document)
        .map_err(XmlSerializationError::from)?;
    Ok(ExtractedFragment {
        kind: DocumentClipboardFragmentKindV1::Structure,
        copied_roots: vec![root_id.clone()],
        copied_atoms,
        copied_bonds,
        fragment_cdml,
    })
}

fn validate_structure_root(record: &TypedRecord) -> Result<(), DocumentClipboardFragmentErrorV1> {
    if record.class() != TypedClass::Molecule
        || record.path().components().len() != 1
        || record.attribute("id").is_none()
        || !record.unknown_attributes().is_empty()
        || !record.diagnostics().is_empty()
    {
        return Err(DocumentClipboardFragmentErrorV1::UnsupportedStructure);
    }
    for child in record.unrecognized_children() {
        match child.node() {
            UnrecognizedNode::Text(text) if text.trim().is_empty() => {}
            _ => return Err(DocumentClipboardFragmentErrorV1::UnsupportedStructure),
        }
    }
    Ok(())
}

fn validate_structure_graph(
    atoms: &[&TypedRecord],
    bonds: &[&TypedRecord],
) -> Result<(), DocumentClipboardFragmentErrorV1> {
    if atoms.iter().any(|atom| atom.attribute("id").is_none()) {
        return Err(DocumentClipboardFragmentErrorV1::UnsupportedStructure);
    }
    for bond in bonds {
        let (Some(_identifier), Some(start), Some(end)) = (
            bond.attribute("id"),
            bond.attribute("start"),
            bond.attribute("end"),
        ) else {
            return Err(DocumentClipboardFragmentErrorV1::UnsupportedStructure);
        };
        if start == end
            || !atoms.iter().any(|atom| atom.attribute("id") == Some(start))
            || !atoms.iter().any(|atom| atom.attribute("id") == Some(end))
        {
            return Err(DocumentClipboardFragmentErrorV1::UnsupportedStructure);
        }
    }
    Ok(())
}

fn selected_object_set(
    selected: &[SelectedFact],
    class: TypedClass,
) -> BTreeSet<&DocumentObjectIdV1> {
    selected
        .iter()
        .filter_map(|fact| match fact {
            SelectedFact::Structure {
                object,
                class: actual,
                ..
            } if *actual == class => Some(object),
            _ => None,
        })
        .collect()
}

type ClosedStructure = (Vec<DocumentObjectIdV1>, Vec<DocumentObjectIdV1>, Vec<u32>);

fn closed_structure(
    atoms: &[&TypedRecord],
    bonds: &[&TypedRecord],
    selected_atoms: &BTreeSet<&DocumentObjectIdV1>,
    selected_bonds: &BTreeSet<&DocumentObjectIdV1>,
) -> Result<ClosedStructure, DocumentClipboardFragmentErrorV1> {
    let mut copied_atom_source_ids = BTreeSet::new();
    for atom in atoms {
        let object = DocumentObjectIdV1::from_record(atom)
            .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
        if selected_atoms.contains(&object) {
            copied_atom_source_ids.insert(
                atom.attribute("id")
                    .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?,
            );
        }
    }
    for bond in bonds {
        let object = DocumentObjectIdV1::from_record(bond)
            .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
        if selected_bonds.contains(&object) {
            copied_atom_source_ids.insert(
                bond.attribute("start")
                    .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?,
            );
            copied_atom_source_ids.insert(
                bond.attribute("end")
                    .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?,
            );
        }
    }
    let mut copied_atoms = Vec::new();
    let mut copied_bonds = Vec::new();
    let mut keep_children = Vec::new();
    for record in atoms.iter().chain(bonds.iter()) {
        let object = DocumentObjectIdV1::from_record(record)
            .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
        let keep = match record.class() {
            TypedClass::Atom => copied_atom_source_ids.contains(
                record
                    .attribute("id")
                    .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?,
            ),
            TypedClass::Bond => selected_bonds.contains(&object),
            _ => false,
        };
        if !keep {
            continue;
        }
        keep_children.push(
            *record
                .path()
                .components()
                .last()
                .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?,
        );
        match record.class() {
            TypedClass::Atom => copied_atoms.push(object),
            TypedClass::Bond => copied_bonds.push(object),
            _ => {}
        }
    }
    ensure_connected(atoms, bonds, &copied_atoms, &copied_bonds)?;
    Ok((copied_atoms, copied_bonds, keep_children))
}

fn ensure_connected(
    atoms: &[&TypedRecord],
    bonds: &[&TypedRecord],
    copied_atoms: &[DocumentObjectIdV1],
    copied_bonds: &[DocumentObjectIdV1],
) -> Result<(), DocumentClipboardFragmentErrorV1> {
    let mut pending = vec![
        copied_atoms
            .first()
            .ok_or(DocumentClipboardFragmentErrorV1::DisconnectedStructure)?,
    ];
    let mut visited = BTreeSet::new();
    while let Some(current) = pending.pop() {
        if !visited.insert(current) {
            continue;
        }
        let current_record = atoms
            .iter()
            .find(|atom| DocumentObjectIdV1::from_record(atom).as_ref() == Some(current))
            .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
        let current_source = current_record
            .attribute("id")
            .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
        for bond in bonds {
            let bond_object = DocumentObjectIdV1::from_record(bond)
                .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
            if !copied_bonds.contains(&bond_object) {
                continue;
            }
            let start = bond
                .attribute("start")
                .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
            let end = bond
                .attribute("end")
                .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
            let neighbour = if start == current_source {
                Some(end)
            } else if end == current_source {
                Some(start)
            } else {
                None
            };
            let Some(neighbour) = neighbour else {
                continue;
            };
            let neighbour_object = atoms
                .iter()
                .find(|atom| atom.attribute("id") == Some(neighbour))
                .and_then(|atom| DocumentObjectIdV1::from_record(atom))
                .ok_or(DocumentClipboardFragmentErrorV1::IdentityInvariant)?;
            if let Some(copied) = copied_atoms
                .iter()
                .find(|object| **object == neighbour_object)
            {
                pending.push(copied);
            }
        }
    }
    if visited.len() != copied_atoms.len() {
        return Err(DocumentClipboardFragmentErrorV1::DisconnectedStructure);
    }
    Ok(())
}

fn retain_root_elements(
    tree: &mut xot::Xot,
    root: Node,
    keep_element_indexes: &[u32],
) -> Result<(), DocumentClipboardFragmentErrorV1> {
    let children = tree.children(root).collect::<Vec<_>>();
    let mut element_index = 0_u32;
    for child in children {
        let keep = if tree.element(child).is_some() {
            let keep = keep_element_indexes.contains(&element_index);
            element_index += 1;
            keep
        } else {
            false
        };
        if !keep {
            tree.remove(child)
                .map_err(DocumentClipboardFragmentErrorV1::Mutation)?;
        }
    }
    Ok(())
}

fn retain_element_children(
    tree: &mut xot::Xot,
    parent: Node,
    keep_element_indexes: &[u32],
) -> Result<(), DocumentClipboardFragmentErrorV1> {
    retain_root_elements(tree, parent, keep_element_indexes)
}

fn element_child(tree: &xot::Xot, parent: Node, index: u32) -> Option<Node> {
    tree.children(parent)
        .filter(|node| tree.element(*node).is_some())
        .nth(index as usize)
}
