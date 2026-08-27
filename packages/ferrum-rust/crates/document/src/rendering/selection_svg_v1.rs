//! Native selected-root SVG composition from one immutable document observation.

use std::collections::{HashMap, HashSet};

use crate::{
    DocumentRenderObservationErrorV1, SessionDocumentObservationV1,
    derive_document_render_observation_from_accepted_operation_v1,
};
use ferrum_document_projection::{DocumentObjectIdV1, MoleculeProjectionV1};
use ferrum_render::{
    DocumentContentBoundsErrorV1, DocumentRenderArtifactV1, DocumentRenderOutcomeV1,
    DocumentRenderPlanCompositionError, DocumentRenderPlanV1, RenderViewportV1, SvgDocumentV1,
    SvgOutputBudgetV1, SvgRenderError, compose_document_render_plan_v1,
    fit_document_render_plan_to_content_v1, render_document_plan_to_svg_with_budget_v1,
};
use thiserror::Error;

/// Stable schema for a selected-root native SVG receipt.
pub const DOCUMENT_SELECTION_SVG_SCHEMA_V1: &str = "ferrum-document-selection-svg-v1";

/// One nonempty, duplicate-free set of exact durable selected document objects.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSvgSelectionV1 {
    objects: Vec<DocumentObjectIdV1>,
}

impl DocumentSvgSelectionV1 {
    /// Validate an owned selection without assigning meaning to caller order.
    pub fn new(objects: Vec<DocumentObjectIdV1>) -> Result<Self, DocumentSelectionSvgErrorV1> {
        if objects.is_empty() {
            return Err(DocumentSelectionSvgErrorV1::EmptySelection);
        }
        let mut unique = HashSet::new();
        unique
            .try_reserve(objects.len())
            .map_err(|_| DocumentSelectionSvgErrorV1::ResourceExhausted)?;
        if objects.iter().any(|object| !unique.insert(object)) {
            return Err(DocumentSelectionSvgErrorV1::DuplicateSelection);
        }
        Ok(Self { objects })
    }

    /// Return the caller-supplied durable selectors.
    #[must_use]
    pub fn objects(&self) -> &[DocumentObjectIdV1] {
        &self.objects
    }
}

/// One direct document root retained by a selected-root render plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentSelectionSvgRootV1 {
    target: DocumentObjectIdV1,
    paint_order: u32,
}

impl DocumentSelectionSvgRootV1 {
    /// Return the exact durable document root target.
    #[must_use]
    pub const fn target(&self) -> &DocumentObjectIdV1 {
        &self.target
    }

    /// Return the exact render paint order.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }
}

/// Completed selected-root SVG paired with exact source and selection provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentSelectionSvgV1 {
    schema: &'static str,
    source_revision: u64,
    source_digest: [u8; 32],
    selected_objects: Vec<DocumentObjectIdV1>,
    selected_roots: Vec<DocumentSelectionSvgRootV1>,
    artifact: DocumentRenderArtifactV1<SvgDocumentV1>,
}

impl DocumentSelectionSvgV1 {
    /// Return the closed receipt schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the immutable source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the immutable source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return selected objects in canonical document order.
    #[must_use]
    pub fn selected_objects(&self) -> &[DocumentObjectIdV1] {
        &self.selected_objects
    }

    /// Return unique retained direct roots in document order.
    #[must_use]
    pub fn selected_roots(&self) -> &[DocumentSelectionSvgRootV1] {
        &self.selected_roots
    }

    /// Return the conservative content-fitted SVG viewport.
    #[must_use]
    pub const fn viewport(&self) -> RenderViewportV1 {
        self.artifact.report().page()
    }

    /// Return the structurally validated SVG document.
    #[must_use]
    pub const fn svg(&self) -> &SvgDocumentV1 {
        self.artifact.artifact()
    }
}

/// Render exact selected objects as complete direct roots from one observation.
///
/// Selecting an atom, bond, or durable molecule retains its complete molecule
/// render root. Selecting presentation content retains that exact direct root.
/// Unselected exclusions are irrelevant; a selected excluded root is refused.
pub fn render_document_selection_to_svg_v1(
    observation: &SessionDocumentObservationV1,
    selection: DocumentSvgSelectionV1,
    output_budget: SvgOutputBudgetV1,
) -> Result<DocumentSelectionSvgV1, DocumentSelectionSvgErrorV1> {
    authenticate_observation(observation)?;
    let selected = resolve_selection(observation, selection.objects())?;
    let render_observation =
        derive_document_render_observation_from_accepted_operation_v1(observation)?;
    let complete_plan = compose_document_render_plan_v1(render_observation.resolved())?;
    let selected_plan = select_plan_roots(&complete_plan, &selected.root_targets)?;
    let fitted_plan = fit_document_render_plan_to_content_v1(&selected_plan)?;
    let artifact = render_document_plan_to_svg_with_budget_v1(&fitted_plan, output_budget)?;
    let selected_roots = selected_plan
        .outcomes()
        .iter()
        .map(|outcome| DocumentSelectionSvgRootV1 {
            target: outcome.target().document_object_id().clone(),
            paint_order: outcome.paint_order(),
        })
        .collect();
    Ok(DocumentSelectionSvgV1 {
        schema: DOCUMENT_SELECTION_SVG_SCHEMA_V1,
        source_revision: observation.snapshot().revision(),
        source_digest: *observation.snapshot().digest(),
        selected_objects: selected.objects,
        selected_roots,
        artifact,
    })
}

/// Failure before a selected-root SVG becomes publishable.
#[derive(Debug, Error)]
pub enum DocumentSelectionSvgErrorV1 {
    /// The caller supplied no selected durable object.
    #[error("selected SVG requires at least one durable selected object")]
    EmptySelection,
    /// The caller repeated a durable object selector.
    #[error("selected SVG object IDs must be unique")]
    DuplicateSelection,
    /// Snapshot and projection provenance did not describe one accepted state.
    #[error("selected SVG observation provenance disagrees")]
    ObservationProvenanceMismatch,
    /// A selector was absent from the immutable projection.
    #[error("selected SVG selection is not an exact durable projected object")]
    UnknownSelectedObject,
    /// A selector resolved more than once and therefore had no unique meaning.
    #[error("selected SVG selection is ambiguous in the document projection")]
    AmbiguousSelectedObject,
    /// A selected source root has no current native render equivalent.
    #[error("selected SVG root is excluded by the native render profile")]
    SelectedRootExcluded,
    /// A supposedly selected root had no outcome in the composed plan.
    #[error("selected SVG root did not match its authenticated render plan")]
    RootPlanMismatch,
    /// Exact observation depiction failed.
    #[error(transparent)]
    Observation(#[from] DocumentRenderObservationErrorV1),
    /// The authenticated roots could not form one renderer-neutral plan.
    #[error(transparent)]
    Composition(#[from] DocumentRenderPlanCompositionError),
    /// The selected content could not form a finite fitted viewport.
    #[error(transparent)]
    Bounds(#[from] DocumentContentBoundsErrorV1),
    /// The bounded native SVG sink refused the completed artifact.
    #[error(transparent)]
    Render(#[from] SvgRenderError),
    /// Selection or result storage could not be reserved.
    #[error("selected SVG could not reserve result storage")]
    ResourceExhausted,
}

struct SelectedFact {
    object: DocumentObjectIdV1,
    root_target: DocumentObjectIdV1,
}

struct ResolvedSelection {
    objects: Vec<DocumentObjectIdV1>,
    root_targets: HashSet<DocumentObjectIdV1>,
}

fn authenticate_observation(
    observation: &SessionDocumentObservationV1,
) -> Result<(), DocumentSelectionSvgErrorV1> {
    if observation.snapshot().revision() != observation.projection().revision()
        || observation.snapshot().digest() != observation.projection().digest()
    {
        return Err(DocumentSelectionSvgErrorV1::ObservationProvenanceMismatch);
    }
    Ok(())
}

fn resolve_selection(
    observation: &SessionDocumentObservationV1,
    requested: &[DocumentObjectIdV1],
) -> Result<ResolvedSelection, DocumentSelectionSvgErrorV1> {
    let mut match_counts = HashMap::new();
    match_counts
        .try_reserve(requested.len())
        .map_err(|_| DocumentSelectionSvgErrorV1::ResourceExhausted)?;
    for object in requested {
        match_counts.insert(object, 0_u8);
    }
    let mut facts = Vec::new();
    facts
        .try_reserve(requested.len())
        .map_err(|_| DocumentSelectionSvgErrorV1::ResourceExhausted)?;
    for molecule in observation.projection().molecules() {
        collect_molecule_facts(molecule, &mut match_counts, &mut facts)?;
    }
    for entry in observation.projection().presentation_stack().entries() {
        let target = entry.root().target();
        let object = target.document_object_id();
        if let Some(count) = match_counts.get_mut(object) {
            *count = count.saturating_add(1);
            facts.push(SelectedFact {
                object: object.clone(),
                root_target: object.clone(),
            });
        }
    }
    if match_counts.values().any(|count| *count == 0) {
        return Err(DocumentSelectionSvgErrorV1::UnknownSelectedObject);
    }
    if match_counts.values().any(|count| *count > 1) {
        return Err(DocumentSelectionSvgErrorV1::AmbiguousSelectedObject);
    }
    let mut objects = Vec::new();
    let mut root_targets = HashSet::new();
    objects
        .try_reserve_exact(facts.len())
        .map_err(|_| DocumentSelectionSvgErrorV1::ResourceExhausted)?;
    for fact in facts {
        objects.push(fact.object);
        root_targets.insert(fact.root_target);
    }
    Ok(ResolvedSelection {
        objects,
        root_targets,
    })
}

fn collect_molecule_facts(
    molecule: &MoleculeProjectionV1,
    match_counts: &mut HashMap<&DocumentObjectIdV1, u8>,
    facts: &mut Vec<SelectedFact>,
) -> Result<(), DocumentSelectionSvgErrorV1> {
    let root_target = molecule.document_object_id();
    let object = molecule.document_object_id();
    if let Some(count) = match_counts.get_mut(object) {
        *count = count.saturating_add(1);
        facts.push(SelectedFact {
            object: object.clone(),
            root_target: root_target.clone(),
        });
    }
    for atom in molecule.atoms() {
        let object = atom.document_object_id();
        if let Some(count) = match_counts.get_mut(object) {
            *count = count.saturating_add(1);
            facts.push(SelectedFact {
                object: object.clone(),
                root_target: root_target.clone(),
            });
        }
    }
    for bond in molecule.bonds() {
        let object = bond.document_object_id();
        if let Some(count) = match_counts.get_mut(object) {
            *count = count.saturating_add(1);
            facts.push(SelectedFact {
                object: object.clone(),
                root_target: root_target.clone(),
            });
        }
    }
    Ok(())
}

fn select_plan_roots(
    plan: &DocumentRenderPlanV1,
    selected_targets: &HashSet<DocumentObjectIdV1>,
) -> Result<DocumentRenderPlanV1, DocumentSelectionSvgErrorV1> {
    let mut outcomes = Vec::new();
    outcomes
        .try_reserve_exact(selected_targets.len())
        .map_err(|_| DocumentSelectionSvgErrorV1::ResourceExhausted)?;
    for outcome in plan.outcomes() {
        if !selected_targets.contains(outcome.target().document_object_id()) {
            continue;
        }
        match outcome {
            DocumentRenderOutcomeV1::Root(_) => outcomes.push(outcome.clone()),
            DocumentRenderOutcomeV1::Exclusion(_) => {
                return Err(DocumentSelectionSvgErrorV1::SelectedRootExcluded);
            }
        }
    }
    if outcomes.len() != selected_targets.len() {
        return Err(DocumentSelectionSvgErrorV1::RootPlanMismatch);
    }
    DocumentRenderPlanV1::new(plan.provenance(), plan.page(), outcomes)
        .map_err(|_| DocumentSelectionSvgErrorV1::RootPlanMismatch)
}
