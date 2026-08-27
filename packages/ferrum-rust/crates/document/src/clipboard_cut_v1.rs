//! Revision-bound preparation and atomic deletion for native Cut.

use std::collections::BTreeSet;

use thiserror::Error;
use xot::Node;

use super::{
    DocumentClipboardFragmentErrorV1, DocumentClipboardFragmentKindV1, DocumentClipboardFragmentV1,
    DocumentClipboardSelectionV1, DocumentObjectIdV1, PresentationRecordKindV1,
    PresentationRootDeletionSetV1, PresentationRootDeletionV1, SessionDocumentObservationV1,
    TypedClass, TypedDocument, TypedDocumentError, extract_document_clipboard_fragment_v1,
};

/// Stable schema identifier for [`DocumentClipboardCutPlanV1`].
pub const DOCUMENT_CLIPBOARD_CUT_SCHEMA_V1: &str = "ferrum-document-clipboard-cut-plan-v1";

/// One immutable fragment plus the exact source-derived deletion intent for native Cut.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentClipboardCutPlanV1 {
    schema: &'static str,
    fragment: DocumentClipboardFragmentV1,
    deletion: ClipboardCutDeletionV1,
}

impl DocumentClipboardCutPlanV1 {
    /// Return the closed Cut plan schema.
    #[must_use]
    pub fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the insertion-valid fragment that must be published before deletion.
    #[must_use]
    pub fn fragment(&self) -> &DocumentClipboardFragmentV1 {
        &self.fragment
    }

    /// Return the exact source revision authenticated by this plan.
    #[must_use]
    pub fn source_revision(&self) -> u64 {
        self.fragment.source_revision()
    }

    /// Return the exact source digest authenticated by this plan.
    #[must_use]
    pub fn source_digest(&self) -> &[u8; 32] {
        self.fragment.source_digest()
    }

    /// Return selected durable objects in canonical document order.
    #[must_use]
    pub fn selected_objects(&self) -> &[DocumentObjectIdV1] {
        self.fragment.selected_objects()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ClipboardCutDeletionV1 {
    Structure {
        molecule: DocumentObjectIdV1,
        selected: Vec<DocumentObjectIdV1>,
    },
    Presentation(PresentationRootDeletionSetV1),
}

/// Failure while preparing or composing one source-authenticated native Cut.
#[derive(Debug, Error)]
pub enum DocumentClipboardCutErrorV1 {
    /// Copy extraction rejected the selection before any deletion was planned.
    #[error(transparent)]
    Fragment(#[from] DocumentClipboardFragmentErrorV1),
    /// Copy's complete-root fallback would not match the user's selected deletion targets.
    #[error("clipboard Cut supports one structural selection or presentation roots only")]
    UnsupportedTopLevelSelection,
    /// A source-derived selector did not resolve while authenticating the plan.
    #[error("clipboard Cut durable identity did not resolve in the authenticated document")]
    IdentityInvariant,
    /// The caller's destination digest was not the session's current digest.
    #[error("clipboard Cut document digest does not match the current revision")]
    DigestMismatch,
    /// The prepared plan did not originate from the exact current document state.
    #[error("clipboard Cut plan does not belong to the current document state")]
    PlanProvenanceMismatch,
    /// Candidate construction or typed validation failed.
    #[error("clipboard Cut could not build a valid document candidate: {0}")]
    Typed(#[from] TypedDocumentError),
}

/// Prepare one insertion-valid fragment and exact deletion intent without mutation.
pub fn prepare_document_clipboard_cut_v1(
    observation: &SessionDocumentObservationV1,
    selection: DocumentClipboardSelectionV1,
) -> Result<DocumentClipboardCutPlanV1, DocumentClipboardCutErrorV1> {
    let fragment = extract_document_clipboard_fragment_v1(observation, selection)?;
    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    let deletion = match fragment.kind() {
        DocumentClipboardFragmentKindV1::Structure => {
            let molecule = fragment
                .copied_roots()
                .first()
                .cloned()
                .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
            ClipboardCutDeletionV1::Structure {
                molecule,
                selected: fragment.selected_objects().to_vec(),
            }
        }
        DocumentClipboardFragmentKindV1::TopLevel => ClipboardCutDeletionV1::Presentation(
            presentation_deletion(&document, fragment.selected_objects())?,
        ),
    };
    let plan = DocumentClipboardCutPlanV1 {
        schema: DOCUMENT_CLIPBOARD_CUT_SCHEMA_V1,
        fragment,
        deletion,
    };
    compose_document_clipboard_cut_candidate_v1(&document, &plan)?;
    Ok(plan)
}

fn presentation_deletion(
    document: &TypedDocument,
    selected: &[DocumentObjectIdV1],
) -> Result<PresentationRootDeletionSetV1, DocumentClipboardCutErrorV1> {
    let mut targets = Vec::with_capacity(selected.len());
    for object in selected {
        let record = document
            .resolve_document_object_id(object)
            .map_err(|_| DocumentClipboardCutErrorV1::IdentityInvariant)?
            .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
        if record.path().components().len() != 1 {
            return Err(DocumentClipboardCutErrorV1::UnsupportedTopLevelSelection);
        }
        let kind = presentation_kind(record.class())
            .ok_or(DocumentClipboardCutErrorV1::UnsupportedTopLevelSelection)?;
        targets.push(PresentationRootDeletionV1::new(object.clone(), kind));
    }
    PresentationRootDeletionSetV1::new(targets)
        .map_err(|_| DocumentClipboardCutErrorV1::IdentityInvariant)
}

const fn presentation_kind(class: TypedClass) -> Option<PresentationRecordKindV1> {
    match class {
        TypedClass::CanvasArrow => Some(PresentationRecordKindV1::Arrow),
        TypedClass::CanvasPlus => Some(PresentationRecordKindV1::Plus),
        TypedClass::CanvasText => Some(PresentationRecordKindV1::Text),
        TypedClass::Polyline => Some(PresentationRecordKindV1::Polyline),
        TypedClass::Rectangle => Some(PresentationRecordKindV1::Rectangle),
        TypedClass::Square => Some(PresentationRecordKindV1::Square),
        TypedClass::Oval => Some(PresentationRecordKindV1::Oval),
        TypedClass::Circle => Some(PresentationRecordKindV1::Circle),
        TypedClass::Polygon => Some(PresentationRecordKindV1::Polygon),
        _ => None,
    }
}

pub(crate) fn compose_document_clipboard_cut_candidate_v1(
    current: &TypedDocument,
    plan: &DocumentClipboardCutPlanV1,
) -> Result<TypedDocument, DocumentClipboardCutErrorV1> {
    match &plan.deletion {
        ClipboardCutDeletionV1::Structure { molecule, selected } => {
            compose_structure_cut(current, molecule, selected)
        }
        ClipboardCutDeletionV1::Presentation(deletions) => current
            .with_delete_presentation_roots(deletions)?
            .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant),
    }
}

fn compose_structure_cut(
    current: &TypedDocument,
    molecule_id: &DocumentObjectIdV1,
    selected: &[DocumentObjectIdV1],
) -> Result<TypedDocument, DocumentClipboardCutErrorV1> {
    let molecule = current
        .resolve_document_object_id(molecule_id)
        .map_err(|_| DocumentClipboardCutErrorV1::IdentityInvariant)?
        .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
    if molecule.class() != TypedClass::Molecule || molecule.path().components().len() != 1 {
        return Err(DocumentClipboardCutErrorV1::IdentityInvariant);
    }
    let molecule_index = molecule.path().components()[0];
    let mut selected_atoms = BTreeSet::new();
    let mut removed_structural_ids = BTreeSet::new();
    let mut removed_children = BTreeSet::new();
    for object in selected {
        let record = current
            .resolve_document_object_id(object)
            .map_err(|_| DocumentClipboardCutErrorV1::IdentityInvariant)?
            .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
        let path = record.path().components();
        if path.len() != 2 || path[0] != molecule_index {
            return Err(DocumentClipboardCutErrorV1::IdentityInvariant);
        }
        match record.class() {
            TypedClass::Atom => {
                let identifier = record
                    .attribute("id")
                    .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
                selected_atoms.insert(identifier);
                removed_structural_ids.insert(identifier);
            }
            TypedClass::Bond => {
                removed_structural_ids.insert(
                    record
                        .attribute("id")
                        .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?,
                );
            }
            _ => return Err(DocumentClipboardCutErrorV1::IdentityInvariant),
        }
        removed_children.insert(path[1]);
    }
    let atom_count = molecule.children_of(TypedClass::Atom).count();
    for child in molecule.typed_children() {
        let record = child.record();
        if record.class() != TypedClass::Bond {
            continue;
        }
        if record
            .attribute("start")
            .is_some_and(|id| selected_atoms.contains(id))
            || record
                .attribute("end")
                .is_some_and(|id| selected_atoms.contains(id))
        {
            let identifier = record
                .attribute("id")
                .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
            let child_index = record
                .path()
                .components()
                .last()
                .copied()
                .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
            removed_structural_ids.insert(identifier);
            removed_children.insert(child_index);
        }
    }
    for child in molecule.typed_children() {
        let record = child.record();
        if !super::typed_linear_form_metadata::is_exact_generated_linear_form_record(record) {
            continue;
        }
        let invalidated = record.typed_children().iter().any(|member| {
            matches!(
                member.record().class(),
                TypedClass::FragmentBond | TypedClass::FragmentVertex
            ) && member
                .record()
                .attribute("id")
                .is_some_and(|id| removed_structural_ids.contains(id))
        });
        if invalidated {
            let child_index = record
                .path()
                .components()
                .last()
                .copied()
                .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
            removed_children.insert(child_index);
        }
    }

    let mut candidate = current.detached_candidate()?;
    let indexed = candidate.detached_indexed_mut();
    let tree = &mut indexed.xml.tree;
    let root = tree
        .document_element(indexed.xml.document)
        .map_err(TypedDocumentError::Mutation)?;
    let molecule_node = element_child(tree, root, molecule_index)
        .ok_or(DocumentClipboardCutErrorV1::IdentityInvariant)?;
    if selected_atoms.len() == atom_count {
        tree.remove(molecule_node)
            .map_err(TypedDocumentError::Mutation)?;
    } else {
        let targets = tree
            .children(molecule_node)
            .filter(|node| tree.element(*node).is_some())
            .enumerate()
            .filter_map(|(index, node)| removed_children.contains(&(index as u32)).then_some(node))
            .collect::<Vec<_>>();
        if targets.len() != removed_children.len() {
            return Err(DocumentClipboardCutErrorV1::IdentityInvariant);
        }
        for target in targets {
            tree.remove(target).map_err(TypedDocumentError::Mutation)?;
        }
    }
    let serialized = candidate.to_xml().map_err(TypedDocumentError::from)?;
    TypedDocument::parse(&serialized).map_err(Into::into)
}

fn element_child(tree: &xot::Xot, parent: Node, index: u32) -> Option<Node> {
    tree.children(parent)
        .filter(|node| tree.element(*node).is_some())
        .nth(index as usize)
}
