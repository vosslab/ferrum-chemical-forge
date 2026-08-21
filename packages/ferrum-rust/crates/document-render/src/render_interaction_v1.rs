//! Render-evidence-backed direct-root interaction facade.
//!
//! This API-layer boundary combines the authoritative document session with its
//! exact render observation. It never exposes CDML, mutable projection objects,
//! or caller-constructed durable root sets.

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicU64, Ordering},
};

use ferrum_document::{
    ArrowGestureStyleV1, CompleteDocumentIdentityFactsV1, DirectCdmlRootKindV1,
    DirectCdmlSemanticIndexV1, DocumentFenceV1, DocumentSession, Point3V1,
    PresentationCreationGestureV1, PresentationGestureErrorV1, PresentationGestureKindV1,
    PresentationGesturePoint2V1, PresentationGestureSnapPolicyV1, PresentationRootProjectionV1,
    SessionOperation, SessionOperationResultV1, SessionOperationV1, StructureDeletionReceiptV1,
    TopLevelRootKindV1, TopLevelRootSelectorV1, TopLevelTransformModeV1, TopLevelTransformV1,
};
use ferrum_geometry::{HexGrid, Point2};
use ferrum_render::{
    PathOpV2, RenderObservationV1, RenderOp, ScenePathCommandV2,
    measure_molecule_render_plan_bounds_v1, observe_render_v1,
};
use thiserror::Error;

use crate::reaction_observation_v1;
pub use crate::reaction_observation_v1::{ReactionListObservationV1, ReactionSelectionV1};

const HIT_SLOP_PT_V1: f64 = 6.0;
const VIEW_HEX_GRID_SPACING_PT_V1: f64 = 40.0;
static NEXT_ORIGIN: AtomicU64 = AtomicU64::new(1);
static NEXT_CAPABILITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderInteractionBoundsV1 {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl RenderInteractionBoundsV1 {
    pub(crate) fn union(self, other: Self) -> Self {
        Self {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
    fn contains_point(self, x: f64, y: f64) -> bool {
        x >= self.left - HIT_SLOP_PT_V1
            && x <= self.right + HIT_SLOP_PT_V1
            && y >= self.top - HIT_SLOP_PT_V1
            && y <= self.bottom + HIT_SLOP_PT_V1
    }
    fn contained_by(self, other: Self) -> bool {
        self.left >= other.left
            && self.top >= other.top
            && self.right <= other.right
            && self.bottom <= other.bottom
    }
    fn translated(self, dx: f64, dy: f64) -> Self {
        Self {
            left: self.left + dx,
            top: self.top + dy,
            right: self.right + dx,
            bottom: self.bottom + dy,
        }
    }
    #[must_use]
    pub const fn left(self) -> f64 {
        self.left
    }
    #[must_use]
    pub const fn top(self) -> f64 {
        self.top
    }
    #[must_use]
    pub const fn right(self) -> f64 {
        self.right
    }
    #[must_use]
    pub const fn bottom(self) -> f64 {
        self.bottom
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderInteractionRootV1 {
    identifier: String,
    source_order: u32,
    bounds: RenderInteractionBoundsV1,
    kind: TopLevelRootKindV1,
}
impl RenderInteractionRootV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn bounds(&self) -> RenderInteractionBoundsV1 {
        self.bounds
    }
    #[must_use]
    pub const fn kind(&self) -> TopLevelRootKindV1 {
        self.kind
    }
}

/// Closed direct-root kinds accepted by the reaction-composer classifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionAuthoringChoiceKindV1 {
    Molecule,
    Arrow,
    Plus,
    ConditionText,
}

/// Read-only availability of one renderer-admitted reaction member selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionAuthoringChoiceAvailabilityV1 {
    Eligible,
    AlreadyInReaction,
}

/// One durable, renderer-observed direct root usable by the reaction composer.
#[derive(Clone, Debug, PartialEq)]
pub struct ReactionAuthoringChoiceV1 {
    identifier: String,
    source_order: u32,
    kind: ReactionAuthoringChoiceKindV1,
    availability: ReactionAuthoringChoiceAvailabilityV1,
    label: String,
    bounds: RenderInteractionBoundsV1,
}
impl ReactionAuthoringChoiceV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn kind(&self) -> ReactionAuthoringChoiceKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn availability(&self) -> ReactionAuthoringChoiceAvailabilityV1 {
        self.availability
    }
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
    #[must_use]
    pub const fn bounds(&self) -> RenderInteractionBoundsV1 {
        self.bounds
    }
}

/// Why an apparently direct root is not a reaction-member selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionAuthoringExclusionReasonV1 {
    DisplayOnly,
    Unrenderable,
    MissingSemanticIdentity,
    AmbiguousSemanticIdentity,
    KindMismatch,
}

/// Closed author recovery for an immutable reaction-composer exclusion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReactionAuthoringExclusionRecoveryV1 {
    ChooseSupportedMember,
    RepairDocument,
}

/// Renderer-issued diagnostic with no mutation or selection capability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionAuthoringExclusionV1 {
    diagnostic_key: String,
    reason: ReactionAuthoringExclusionReasonV1,
    recovery: ReactionAuthoringExclusionRecoveryV1,
    label: String,
}
impl ReactionAuthoringExclusionV1 {
    #[must_use]
    pub fn diagnostic_key(&self) -> &str {
        &self.diagnostic_key
    }
    #[must_use]
    pub const fn reason(&self) -> ReactionAuthoringExclusionReasonV1 {
        self.reason
    }
    #[must_use]
    pub const fn recovery(&self) -> ReactionAuthoringExclusionRecoveryV1 {
        self.recovery
    }
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// Immutable, session-origin-bound reaction-composer observation.
///
/// This is deliberately not a selection, gesture, receipt, candidate, or
/// transaction. Consumers may display its facts and must revalidate them before
/// retaining a panel across an external document mutation.
#[derive(Clone, Debug)]
pub struct ReactionAuthoringChoicesV1 {
    origin: u64,
    capability: u64,
    fence: DocumentFenceV1,
    choices: Vec<ReactionAuthoringChoiceV1>,
    exclusions: Vec<ReactionAuthoringExclusionV1>,
}
impl ReactionAuthoringChoicesV1 {
    #[must_use]
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
    #[must_use]
    pub fn choices(&self) -> &[ReactionAuthoringChoiceV1] {
        &self.choices
    }
    #[must_use]
    pub fn exclusions(&self) -> &[ReactionAuthoringExclusionV1] {
        &self.exclusions
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderInteractionModifierV1 {
    Replace,
    Toggle,
}
#[derive(Clone, Debug, PartialEq)]
pub enum RenderInteractionQueryV1 {
    Point {
        x: f64,
        y: f64,
        modifier: RenderInteractionModifierV1,
    },
    Marquee {
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        modifier: RenderInteractionModifierV1,
    },
    /// Resolve a known authorable root or diagnostic key without fallback geometry.
    Root {
        identifier: String,
        modifier: RenderInteractionModifierV1,
    },
    Clear,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderInteractionAxisV1 {
    Free,
    Horizontal,
    Vertical,
}
/// Closed View-level grid policy captured by one opaque translation gesture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderInteractionGridSnapPolicyV1 {
    Free,
    ViewHexGrid,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderInteractionSnapV1 {
    axis: RenderInteractionAxisV1,
    grid_policy: RenderInteractionGridSnapPolicyV1,
}
impl RenderInteractionSnapV1 {
    #[must_use]
    pub const fn free() -> Self {
        Self {
            axis: RenderInteractionAxisV1::Free,
            grid_policy: RenderInteractionGridSnapPolicyV1::Free,
        }
    }
    #[must_use]
    pub const fn new(axis: RenderInteractionAxisV1) -> Self {
        Self {
            axis,
            grid_policy: RenderInteractionGridSnapPolicyV1::Free,
        }
    }
    #[must_use]
    pub const fn with_grid(grid_policy: RenderInteractionGridSnapPolicyV1) -> Self {
        Self {
            axis: RenderInteractionAxisV1::Free,
            grid_policy,
        }
    }
    #[must_use]
    pub const fn with_grid_policy(
        axis: RenderInteractionAxisV1,
        grid_policy: RenderInteractionGridSnapPolicyV1,
    ) -> Self {
        Self { axis, grid_policy }
    }
}

#[derive(Clone, Debug)]
pub struct RenderInteractionObservationV1 {
    origin: u64,
    capability: u64,
    fence: DocumentFenceV1,
    roots: Vec<RenderInteractionRootV1>,
    exclusions: Vec<RenderInteractionExclusionV1>,
}
impl RenderInteractionObservationV1 {
    pub(crate) const fn capability(&self) -> u64 {
        self.capability
    }
}

/// Why a durable root cannot become an authoring target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderInteractionExclusionReasonV1 {
    UnrenderableDepiction,
    AmbiguousRootIdentifier,
    DisplayOnly,
}

/// Revision-bound diagnostic for one known but non-authorable durable root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderInteractionExclusionV1 {
    identifier: String,
    reason: RenderInteractionExclusionReasonV1,
}
impl RenderInteractionExclusionV1 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    #[must_use]
    pub const fn reason(&self) -> RenderInteractionExclusionReasonV1 {
        self.reason
    }
}
impl RenderInteractionObservationV1 {
    #[must_use]
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
    #[must_use]
    pub fn roots(&self) -> &[RenderInteractionRootV1] {
        &self.roots
    }
    /// Durable roots that are known but cannot become authoring targets.
    #[must_use]
    pub fn exclusions(&self) -> &[RenderInteractionExclusionV1] {
        &self.exclusions
    }
}

#[derive(Clone, Debug)]
pub struct RenderInteractionSelectionV1 {
    origin: u64,
    fence: DocumentFenceV1,
    roots: Vec<RenderInteractionRootV1>,
}

/// Closed target grammar for direct, render-issued molecular children.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructureTargetKindV1 {
    Atom,
    Bond,
    /// The renderer produced a non-line bond primitive that P0.3 must not
    /// reinterpret as an editable centerline.  It remains a durable,
    /// render-derived hit target so the caller receives a typed refusal.
    DisplayOnly,
}

/// One exact child hit envelope derived by Rust from the fenced document projection.
#[derive(Clone, Debug, PartialEq)]
pub struct StructureInteractionTargetV1 {
    molecule_id: String,
    identifier: String,
    source_order: u32,
    kind: StructureTargetKindV1,
    bounds: RenderInteractionBoundsV1,
    geometry: StructureInteractionGeometryV1,
}
#[derive(Clone, Debug, PartialEq)]
enum StructureInteractionGeometryV1 {
    Atom { x: f64, y: f64 },
    Bond { segments: Vec<StructureSegmentV1> },
    DisplayOnly,
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct StructureSegmentV1 {
    start_x: f64,
    start_y: f64,
    end_x: f64,
    end_y: f64,
    stroke_radius: f64,
}
impl StructureInteractionTargetV1 {
    #[must_use]
    pub fn molecule_id(&self) -> &str {
        &self.molecule_id
    }
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn kind(&self) -> StructureTargetKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn bounds(&self) -> RenderInteractionBoundsV1 {
        self.bounds
    }
    fn hit(&self, x: f64, y: f64) -> bool {
        match &self.geometry {
            StructureInteractionGeometryV1::Atom { x: ax, y: ay } => {
                (x - ax).hypot(y - ay) <= HIT_SLOP_PT_V1
            }
            StructureInteractionGeometryV1::Bond { segments } => segments.iter().any(|segment| {
                segment_distance(x, y, *segment) <= HIT_SLOP_PT_V1.max(segment.stroke_radius)
            }),
            StructureInteractionGeometryV1::DisplayOnly => self.bounds.contains_point(x, y),
        }
    }
    fn fully_contained_by(&self, rectangle: RenderInteractionBoundsV1) -> bool {
        match &self.geometry {
            StructureInteractionGeometryV1::Atom { x, y } => {
                rectangle.left <= *x
                    && *x <= rectangle.right
                    && rectangle.top <= *y
                    && *y <= rectangle.bottom
            }
            StructureInteractionGeometryV1::Bond { segments } => segments.iter().all(|segment| {
                rectangle.left + segment.stroke_radius <= segment.start_x
                    && segment.start_x <= rectangle.right - segment.stroke_radius
                    && rectangle.top + segment.stroke_radius <= segment.start_y
                    && segment.start_y <= rectangle.bottom - segment.stroke_radius
                    && rectangle.left + segment.stroke_radius <= segment.end_x
                    && segment.end_x <= rectangle.right - segment.stroke_radius
                    && rectangle.top + segment.stroke_radius <= segment.end_y
                    && segment.end_y <= rectangle.bottom - segment.stroke_radius
            }),
            StructureInteractionGeometryV1::DisplayOnly => self.bounds.contained_by(rectangle),
        }
    }
}

/// Opaque observation of renderable direct atom and bond targets.
#[derive(Clone, Debug)]
pub struct StructureInteractionObservationV1 {
    origin: u64,
    capability: u64,
    fence: DocumentFenceV1,
    targets: Vec<StructureInteractionTargetV1>,
}
impl StructureInteractionObservationV1 {
    #[must_use]
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
    #[must_use]
    pub fn targets(&self) -> &[StructureInteractionTargetV1] {
        &self.targets
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StructureInteractionQueryV1 {
    Point {
        x: f64,
        y: f64,
        modifier: RenderInteractionModifierV1,
    },
    Marquee {
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        modifier: RenderInteractionModifierV1,
    },
    Clear,
}

/// Opaque, one-molecule selection.  IDs are never caller-constructed at this boundary.
#[derive(Clone, Debug)]
pub struct StructureInteractionSelectionV1 {
    origin: u64,
    capability: u64,
    fence: DocumentFenceV1,
    targets: Vec<StructureInteractionTargetV1>,
}
impl StructureInteractionSelectionV1 {
    #[must_use]
    pub fn targets(&self) -> &[StructureInteractionTargetV1] {
        &self.targets
    }
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}

/// Authoritative receipt for the one atomic direct-structure deletion.
#[derive(Clone, Debug)]
pub struct CommittedStructureDeletionV1 {
    result: SessionOperationResultV1,
    removed_atoms: Vec<String>,
    removed_bonds: Vec<String>,
    components: Vec<StructureDeletionComponentFactsV1>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructureDeletionComponentFactsV1 {
    molecule_id: String,
    atom_ids: Vec<String>,
    bond_ids: Vec<String>,
}
impl StructureDeletionComponentFactsV1 {
    #[must_use]
    pub fn molecule_id(&self) -> &str {
        &self.molecule_id
    }
    #[must_use]
    pub fn atom_ids(&self) -> &[String] {
        &self.atom_ids
    }
    #[must_use]
    pub fn bond_ids(&self) -> &[String] {
        &self.bond_ids
    }
}
impl CommittedStructureDeletionV1 {
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
    #[must_use]
    pub fn removed_atoms(&self) -> &[String] {
        &self.removed_atoms
    }
    #[must_use]
    pub fn removed_bonds(&self) -> &[String] {
        &self.removed_bonds
    }
    #[must_use]
    pub fn components(&self) -> &[StructureDeletionComponentFactsV1] {
        &self.components
    }
}
impl RenderInteractionSelectionV1 {
    #[must_use]
    pub fn roots(&self) -> &[RenderInteractionRootV1] {
        &self.roots
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct RenderInteractionTranslationGestureV1 {
    origin: u64,
    capability: u64,
    selection: RenderInteractionSelectionV1,
    press_x: f64,
    press_y: f64,
    snap: RenderInteractionSnapV1,
}
#[derive(Clone, Debug)]
pub struct RenderInteractionTranslationPreviewV1 {
    capability: u64,
    dx: f64,
    dy: f64,
    bounds: Vec<RenderInteractionBoundsV1>,
}
impl RenderInteractionTranslationPreviewV1 {
    #[must_use]
    pub const fn dx(&self) -> f64 {
        self.dx
    }
    #[must_use]
    pub const fn dy(&self) -> f64 {
        self.dy
    }
    #[must_use]
    pub fn bounds(&self) -> &[RenderInteractionBoundsV1] {
        &self.bounds
    }
}
#[derive(Clone, Debug)]
pub struct CommittedRenderInteractionTranslationV1 {
    changed: bool,
    result: SessionOperationResultV1,
    selection: RenderInteractionSelectionV1,
}
impl CommittedRenderInteractionTranslationV1 {
    #[must_use]
    pub const fn changed(&self) -> bool {
        self.changed
    }
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
    #[must_use]
    pub fn selection(&self) -> &RenderInteractionSelectionV1 {
        &self.selection
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum RenderInteractionErrorV1 {
    #[error("interaction observation is stale; refresh and restart")]
    StaleRevision,
    #[error("interaction observation digest is stale; refresh and restart")]
    StaleDigest,
    #[error("interaction handle belongs to another session")]
    ForeignSession,
    #[error("interaction selection is no longer current")]
    SelectionChanged,
    #[error("a selected renderable root is required")]
    EmptySelection,
    #[error("pointer coordinates must be finite")]
    NonFinitePoint,
    #[error("marquee bounds must be finite and normalized")]
    InvalidRectangle,
    #[error("preview does not belong to this gesture")]
    PreviewMismatch,
    #[error("the named interaction root is not part of this observation")]
    NoTarget,
    #[error("one or more molecule roots are preserved but have no authorable render plan")]
    UnrenderableDepiction,
    #[error("the named molecule root is ambiguous in compatible CDML")]
    AmbiguousRootIdentifier,
    #[error("the named presentation root is display-only and cannot be moved")]
    DisplayOnly,
    #[error("could not obtain a coherent render-backed interaction observation")]
    Observation,
    #[error("the document session rejected the authorized interaction commit")]
    SessionConflict,
    #[error("structural selection cannot span more than one direct molecule")]
    CrossMoleculeSelection,
    #[error("a structural target no longer belongs to the observed molecule")]
    UnsupportedTarget,
    #[error("the current document cannot supply a complete SMARTS target set")]
    UnsupportedDocument,
}

/// Mutable session plus the sole render-backed direct-root interaction boundary.
#[derive(Debug)]
pub struct RenderInteractionSessionV1 {
    session: DocumentSession,
    origin: u64,
}
impl RenderInteractionSessionV1 {
    /// Validate that a root selection was issued by this live session and still
    /// refers to its current document fence. This exposes no selection facts.
    pub fn validate_render_interaction_selection_v1(
        &self,
        selection: &RenderInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        self.require_selection(selection)
    }

    #[must_use]
    pub fn new(session: DocumentSession) -> Self {
        Self {
            session,
            origin: NEXT_ORIGIN.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// Return the private renderer-session identity for sibling opaque bridges.
    ///
    /// This must not be confused with the embedded document-session identity:
    /// observations and selections are issued by this renderer boundary.
    #[must_use]
    pub(crate) const fn render_interaction_origin_v1(&self) -> u64 {
        self.origin
    }

    /// Begin a presentation gesture using the same Rust render facts that the
    /// committed Plus will expose. Arrow geometry remains document-owned.
    pub fn begin_presentation_creation_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        kind: PresentationGestureKindV1,
        start: PresentationGesturePoint2V1,
        style: ArrowGestureStyleV1,
        snap: PresentationGestureSnapPolicyV1,
    ) -> Result<PresentationCreationGestureV1, PresentationGestureErrorV1> {
        self.session
            .begin_presentation_creation_gesture_v1(fence, kind, start, style, snap)
    }

    pub fn observe_render_interaction_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<RenderInteractionObservationV1, RenderInteractionErrorV1> {
        self.require_fence(fence)?;
        let rendered = observe_render_v1(&self.session, fence.revision())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        if rendered.document().snapshot().revision() != fence.revision() {
            return Err(RenderInteractionErrorV1::StaleRevision);
        }
        if rendered.document().snapshot().digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        let identities = self
            .session
            .observe_complete_document_identity_facts_v1(fence.revision())
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        let (roots, exclusions) = roots_from_render(&rendered, &identities);
        Ok(RenderInteractionObservationV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            fence,
            roots,
            exclusions,
        })
    }

    /// Classify the exact current renderer-admitted roots for reaction authoring.
    ///
    /// The namespace-aware semantic index supplies durable kind and existing
    /// reaction membership; the direct-root interaction observation supplies
    /// only admitted geometry and identity. Neither source is sufficient alone.
    pub fn observe_reaction_authoring_choices_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<ReactionAuthoringChoicesV1, RenderInteractionErrorV1> {
        let roots = self.observe_render_interaction_v1(fence)?;
        let snapshot = self
            .session
            .snapshot()
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        let index = DirectCdmlSemanticIndexV1::parse(snapshot.cdml())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        let members = index
            .roots()
            .iter()
            .filter(|root| root.kind() == DirectCdmlRootKindV1::Reaction)
            .flat_map(|root| root.reaction_members().iter().cloned())
            .collect::<HashSet<_>>();
        let mut choices = Vec::new();
        let mut exclusions = Vec::new();
        let mut diagnosed = HashSet::new();
        for root in roots.roots() {
            let semantic = index
                .roots()
                .iter()
                .filter(|candidate| candidate.identifier() == Some(root.identifier()))
                .collect::<Vec<_>>();
            match semantic.as_slice() {
                [] => push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    root.identifier(),
                    ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity,
                    reaction_exclusion_label(
                        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity,
                        root.identifier(),
                    ),
                ),
                [semantic] => match reaction_choice_kind(semantic.kind(), root.kind()) {
                    Some(kind) => {
                        let availability = if members.contains(root.identifier()) {
                            ReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction
                        } else {
                            ReactionAuthoringChoiceAvailabilityV1::Eligible
                        };
                        choices.push(ReactionAuthoringChoiceV1 {
                            identifier: root.identifier().to_owned(),
                            source_order: root.source_order(),
                            kind,
                            availability,
                            label: reaction_choice_label(kind, root.identifier()),
                            bounds: root.bounds(),
                        });
                    }
                    None => {
                        let reason = reaction_root_exclusion_reason(semantic.kind(), root.kind());
                        push_reaction_exclusion(
                            &mut exclusions,
                            &mut diagnosed,
                            root.identifier(),
                            reason,
                            reaction_exclusion_label(reason, root.identifier()),
                        );
                    }
                },
                _ => push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    root.identifier(),
                    ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity,
                    reaction_exclusion_label(
                        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity,
                        root.identifier(),
                    ),
                ),
            }
        }
        for value in roots.exclusions() {
            let reason = match value.reason() {
                RenderInteractionExclusionReasonV1::DisplayOnly => {
                    ReactionAuthoringExclusionReasonV1::DisplayOnly
                }
                RenderInteractionExclusionReasonV1::UnrenderableDepiction => {
                    ReactionAuthoringExclusionReasonV1::Unrenderable
                }
                RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier => {
                    ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity
                }
            };
            let label = index
                .roots()
                .iter()
                .find(|root| root.identifier() == Some(value.identifier()))
                .and_then(|root| direct_reaction_choice_kind(root.kind()))
                .map_or_else(
                    || reaction_exclusion_label(reason, value.identifier()),
                    |kind| reaction_choice_label(kind, value.identifier()),
                );
            push_reaction_exclusion(
                &mut exclusions,
                &mut diagnosed,
                value.identifier(),
                reason,
                label,
            );
        }
        let observed = roots
            .roots()
            .iter()
            .map(RenderInteractionRootV1::identifier)
            .collect::<HashSet<_>>();
        for root in index.roots() {
            let Some(identifier) = root.identifier() else {
                continue;
            };
            let Some(kind) = direct_reaction_choice_kind(root.kind()) else {
                continue;
            };
            if !observed.contains(identifier) {
                push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    identifier,
                    ReactionAuthoringExclusionReasonV1::Unrenderable,
                    reaction_choice_label(kind, identifier),
                );
            }
        }
        choices.sort_by_key(ReactionAuthoringChoiceV1::source_order);
        Ok(ReactionAuthoringChoicesV1 {
            origin: self.origin,
            capability: roots.capability,
            fence,
            choices,
            exclusions,
        })
    }

    /// Return all retained direct reaction records with renderer-backed member facts.
    pub fn observe_reaction_list_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<ReactionListObservationV1, RenderInteractionErrorV1> {
        let rendered = self.observe_render_interaction_v1(fence)?;
        reaction_observation_v1::observe_reaction_list_v1(&self.session, self.origin, &rendered)
    }

    /// Refuse a foreign or stale reaction list without mutating CDML.
    pub fn validate_reaction_list_v1(
        &self,
        list: &ReactionListObservationV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        reaction_observation_v1::validate_reaction_list_v1(&self.session, self.origin, list)
    }

    /// Issue an opaque aggregate-selection capability from one fresh list fact.
    pub fn select_reaction_v1(
        &self,
        list: &ReactionListObservationV1,
        reaction_id: &str,
    ) -> Result<ReactionSelectionV1, RenderInteractionErrorV1> {
        reaction_observation_v1::select_reaction_v1(&self.session, self.origin, list, reaction_id)
    }

    /// Validate an opaque reaction selection before a future lifecycle mutation.
    pub fn validate_reaction_selection_v1(
        &self,
        selection: &ReactionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        reaction_observation_v1::validate_reaction_selection_v1(
            &self.session,
            self.origin,
            selection,
        )
    }

    /// Refuse a foreign or stale immutable composer observation without mutation.
    pub fn validate_reaction_authoring_choices_v1(
        &self,
        choices: &ReactionAuthoringChoicesV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if choices.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if choices.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(choices.fence)
    }

    /// Observe exact direct atom/bond target envelopes for structural selection.
    pub fn observe_structure_interaction_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<StructureInteractionObservationV1, RenderInteractionErrorV1> {
        self.require_fence(fence)?;
        let rendered = observe_render_v1(&self.session, fence.revision())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        if rendered.document().snapshot().digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        let mut targets = Vec::new();
        for molecule in rendered.document().projection().molecules() {
            let Some(molecule_id) = molecule.source_id() else {
                continue;
            };
            for atom in molecule.atoms() {
                let Some(identifier) = atom.source_id() else {
                    continue;
                };
                let point = atom.position();
                targets.push(StructureInteractionTargetV1 {
                    molecule_id: molecule_id.to_owned(),
                    identifier: identifier.to_owned(),
                    source_order: atom.source_order(),
                    kind: StructureTargetKindV1::Atom,
                    // `contains_point` owns the single shared hit slop.  The
                    // issued atom envelope itself stays at the atom anchor so
                    // nearby bond clicks are not accidentally atom hits.
                    bounds: square_bounds(point.x(), point.y(), 0.0),
                    geometry: StructureInteractionGeometryV1::Atom {
                        x: point.x(),
                        y: point.y(),
                    },
                });
            }
            let Some(plan) = rendered
                .molecule_plans()
                .iter()
                .find(|entry| entry.molecule().source_id() == Some(molecule_id))
            else {
                continue;
            };
            for bond in molecule.bonds() {
                let Some(identifier) = bond.source_id() else {
                    continue;
                };
                let operations = plan
                    .batches()
                    .iter()
                    .filter(|batch| batch.target().source_order() == bond.source_order())
                    .flat_map(|batch| batch.operations());
                let mut segments = Vec::new();
                let mut primitive_bounds = Vec::new();
                let mut has_path = false;
                for operation in operations {
                    match operation {
                        RenderOp::Line(line) => {
                            let segment = StructureSegmentV1 {
                                start_x: line.start().x(),
                                start_y: line.start().y(),
                                end_x: line.end().x(),
                                end_y: line.end().y(),
                                stroke_radius: line.width().get() / 2.0,
                            };
                            primitive_bounds.push(segment_bounds(std::slice::from_ref(&segment)));
                            segments.push(segment);
                        }
                        RenderOp::Path(path) => {
                            has_path = true;
                            primitive_bounds.push(path_bounds(path));
                        }
                        RenderOp::Text(_) | RenderOp::Mask(_) | RenderOp::Ellipse(_) => {}
                    }
                }
                if primitive_bounds.is_empty() {
                    continue;
                }
                targets.push(StructureInteractionTargetV1 {
                    molecule_id: molecule_id.to_owned(),
                    identifier: identifier.to_owned(),
                    source_order: bond.source_order(),
                    kind: if has_path {
                        StructureTargetKindV1::DisplayOnly
                    } else {
                        StructureTargetKindV1::Bond
                    },
                    bounds: union_bounds(&primitive_bounds),
                    geometry: if has_path {
                        StructureInteractionGeometryV1::DisplayOnly
                    } else {
                        StructureInteractionGeometryV1::Bond { segments }
                    },
                });
            }
        }
        targets.sort_by_key(StructureInteractionTargetV1::source_order);
        Ok(StructureInteractionObservationV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            fence,
            targets,
        })
    }

    /// Resolve one point, full-containment marquee, or clear request entirely in Rust.
    pub fn select_structure_interaction_v1(
        &self,
        observation: &StructureInteractionObservationV1,
        previous: Option<&StructureInteractionSelectionV1>,
        query: StructureInteractionQueryV1,
    ) -> Result<StructureInteractionSelectionV1, RenderInteractionErrorV1> {
        self.require_structure_observation(observation)?;
        if let Some(selection) = previous {
            self.require_structure_selection(selection)?;
        }
        let (candidates, toggle) = match query {
            StructureInteractionQueryV1::Clear => (Vec::new(), false),
            StructureInteractionQueryV1::Point { x, y, modifier } => {
                if !x.is_finite() || !y.is_finite() {
                    return Err(RenderInteractionErrorV1::NonFinitePoint);
                }
                let atoms = observation
                    .targets
                    .iter()
                    .filter(|target| target.kind == StructureTargetKindV1::Atom && target.hit(x, y))
                    .cloned()
                    .collect::<Vec<_>>();
                let values = if atoms.is_empty() {
                    let bonds = observation
                        .targets
                        .iter()
                        .filter(|target| {
                            target.kind == StructureTargetKindV1::Bond && target.hit(x, y)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if bonds.is_empty()
                        && observation.targets.iter().any(|target| {
                            target.kind == StructureTargetKindV1::DisplayOnly && target.hit(x, y)
                        })
                    {
                        return Err(RenderInteractionErrorV1::DisplayOnly);
                    }
                    bonds
                } else {
                    atoms
                };
                (values, modifier == RenderInteractionModifierV1::Toggle)
            }
            StructureInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                modifier,
            } => {
                if !left.is_finite()
                    || !top.is_finite()
                    || !right.is_finite()
                    || !bottom.is_finite()
                    || left > right
                    || top > bottom
                {
                    return Err(RenderInteractionErrorV1::InvalidRectangle);
                }
                let rectangle = RenderInteractionBoundsV1 {
                    left,
                    top,
                    right,
                    bottom,
                };
                let candidates = observation
                    .targets
                    .iter()
                    .filter(|target| {
                        target.kind != StructureTargetKindV1::DisplayOnly
                            && target.fully_contained_by(rectangle)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if observation.targets.iter().any(|target| {
                    target.kind == StructureTargetKindV1::DisplayOnly
                        && target.fully_contained_by(rectangle)
                }) {
                    return Err(RenderInteractionErrorV1::DisplayOnly);
                }
                (candidates, modifier == RenderInteractionModifierV1::Toggle)
            }
        };
        let mut targets = if toggle {
            toggle_structure_targets(
                previous.map_or_else(Vec::new, |value| value.targets.clone()),
                candidates,
            )
        } else {
            candidates
        };
        targets.sort_by_key(StructureInteractionTargetV1::source_order);
        if targets
            .iter()
            .map(StructureInteractionTargetV1::molecule_id)
            .collect::<HashSet<_>>()
            .len()
            > 1
        {
            return Err(RenderInteractionErrorV1::CrossMoleculeSelection);
        }
        Ok(StructureInteractionSelectionV1 {
            origin: self.origin,
            capability: observation.capability,
            fence: observation.fence,
            targets,
        })
    }

    /// Commit the opaque direct-child selection as one fenced structural mutation.
    pub fn commit_structure_deletion_v1(
        &mut self,
        selection: &StructureInteractionSelectionV1,
    ) -> Result<CommittedStructureDeletionV1, RenderInteractionErrorV1> {
        self.require_structure_selection(selection)?;
        if selection.targets.is_empty() {
            return Err(RenderInteractionErrorV1::EmptySelection);
        }
        let molecule_id = selection.targets[0].molecule_id.clone();
        if selection
            .targets
            .iter()
            .any(|target| target.molecule_id != molecule_id)
        {
            return Err(RenderInteractionErrorV1::CrossMoleculeSelection);
        }
        let atom_ids = selection
            .targets
            .iter()
            .filter(|target| target.kind == StructureTargetKindV1::Atom)
            .map(|target| target.identifier.clone())
            .collect::<Vec<_>>();
        let bond_ids = selection
            .targets
            .iter()
            .filter(|target| target.kind == StructureTargetKindV1::Bond)
            .map(|target| target.identifier.clone())
            .collect::<Vec<_>>();
        if selection.targets.iter().any(|target| {
            !matches!(
                target.kind,
                StructureTargetKindV1::Atom | StructureTargetKindV1::Bond
            )
        }) {
            return Err(RenderInteractionErrorV1::DisplayOnly);
        }
        let mut pending = self
            .session
            .prepare_delete_structure_v1(
                selection.fence.revision(),
                molecule_id,
                atom_ids,
                bond_ids,
            )
            .map_err(|_| RenderInteractionErrorV1::UnsupportedTarget)?;
        let receipt = pending.receipt().clone();
        let result = self
            .session
            .commit_delete_structure_v1(selection.fence.revision(), &mut pending)
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        let (removed_atoms, removed_bonds, components) = structure_deletion_receipt(receipt);
        Ok(CommittedStructureDeletionV1 {
            result,
            removed_atoms,
            removed_bonds,
            components,
        })
    }

    pub fn select_render_interaction_roots_v1(
        &self,
        observation: &RenderInteractionObservationV1,
        previous: Option<&RenderInteractionSelectionV1>,
        query: RenderInteractionQueryV1,
    ) -> Result<RenderInteractionSelectionV1, RenderInteractionErrorV1> {
        self.require_observation(observation)?;
        if let Some(value) = previous {
            self.require_selection(value)?;
        }
        let candidates = match &query {
            RenderInteractionQueryV1::Clear => Vec::new(),
            RenderInteractionQueryV1::Point { x, y, .. } => {
                if !x.is_finite() || !y.is_finite() {
                    return Err(RenderInteractionErrorV1::NonFinitePoint);
                }
                observation
                    .roots
                    .iter()
                    .filter(|root| root.bounds.contains_point(*x, *y))
                    .cloned()
                    .collect()
            }
            RenderInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                ..
            } => {
                if !left.is_finite()
                    || !top.is_finite()
                    || !right.is_finite()
                    || !bottom.is_finite()
                    || left > right
                    || top > bottom
                {
                    return Err(RenderInteractionErrorV1::InvalidRectangle);
                }
                let rectangle = RenderInteractionBoundsV1 {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                };
                observation
                    .roots
                    .iter()
                    .filter(|root| root.bounds.contained_by(rectangle))
                    .cloned()
                    .collect()
            }
            RenderInteractionQueryV1::Root { identifier, .. } => {
                if let Some(exclusion) = observation
                    .exclusions
                    .iter()
                    .find(|exclusion| exclusion.identifier == *identifier)
                {
                    return Err(match exclusion.reason {
                        RenderInteractionExclusionReasonV1::UnrenderableDepiction => {
                            RenderInteractionErrorV1::UnrenderableDepiction
                        }
                        RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier => {
                            RenderInteractionErrorV1::AmbiguousRootIdentifier
                        }
                        RenderInteractionExclusionReasonV1::DisplayOnly => {
                            RenderInteractionErrorV1::DisplayOnly
                        }
                    });
                }
                observation
                    .roots
                    .iter()
                    .find(|root| root.identifier == *identifier)
                    .cloned()
                    .map_or_else(
                        || Err(RenderInteractionErrorV1::NoTarget),
                        |root| Ok(vec![root]),
                    )?
            }
        };
        let toggle = matches!(
            query,
            RenderInteractionQueryV1::Point {
                modifier: RenderInteractionModifierV1::Toggle,
                ..
            } | RenderInteractionQueryV1::Marquee {
                modifier: RenderInteractionModifierV1::Toggle,
                ..
            } | RenderInteractionQueryV1::Root {
                modifier: RenderInteractionModifierV1::Toggle,
                ..
            }
        );
        let roots = if toggle {
            toggle_roots(
                previous.map_or_else(Vec::new, |value| value.roots.clone()),
                candidates,
            )
        } else {
            candidates
        };
        Ok(RenderInteractionSelectionV1 {
            origin: self.origin,
            fence: observation.fence,
            roots,
        })
    }

    pub fn begin_render_interaction_translation_v1(
        &self,
        selection: &RenderInteractionSelectionV1,
        press_x: f64,
        press_y: f64,
        snap: RenderInteractionSnapV1,
    ) -> Result<RenderInteractionTranslationGestureV1, RenderInteractionErrorV1> {
        self.require_selection(selection)?;
        if selection.is_empty() {
            return Err(RenderInteractionErrorV1::EmptySelection);
        }
        if !press_x.is_finite() || !press_y.is_finite() {
            return Err(RenderInteractionErrorV1::NonFinitePoint);
        }
        Ok(RenderInteractionTranslationGestureV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            selection: selection.clone(),
            press_x,
            press_y,
            snap,
        })
    }

    pub fn preview_render_interaction_translation_v1(
        &self,
        gesture: &RenderInteractionTranslationGestureV1,
        pointer_x: f64,
        pointer_y: f64,
    ) -> Result<RenderInteractionTranslationPreviewV1, RenderInteractionErrorV1> {
        self.require_gesture(gesture)?;
        self.require_selection(&gesture.selection)?;
        if !pointer_x.is_finite() || !pointer_y.is_finite() {
            return Err(RenderInteractionErrorV1::NonFinitePoint);
        }
        let (pointer_x, pointer_y, press_x, press_y) = match gesture.snap.grid_policy {
            RenderInteractionGridSnapPolicyV1::Free => {
                (pointer_x, pointer_y, gesture.press_x, gesture.press_y)
            }
            RenderInteractionGridSnapPolicyV1::ViewHexGrid => {
                let origin =
                    Point2::new(0.0, 0.0).map_err(|_| RenderInteractionErrorV1::Observation)?;
                let grid = HexGrid::new(VIEW_HEX_GRID_SPACING_PT_V1, origin)
                    .map_err(|_| RenderInteractionErrorV1::Observation)?;
                let pointer = grid
                    .snap(
                        Point2::new(pointer_x, pointer_y)
                            .map_err(|_| RenderInteractionErrorV1::NonFinitePoint)?,
                    )
                    .map_err(|_| RenderInteractionErrorV1::Observation)?;
                let press = grid
                    .snap(
                        Point2::new(gesture.press_x, gesture.press_y)
                            .map_err(|_| RenderInteractionErrorV1::NonFinitePoint)?,
                    )
                    .map_err(|_| RenderInteractionErrorV1::Observation)?;
                (pointer.x(), pointer.y(), press.x(), press.y())
            }
        };
        let (mut dx, mut dy) = (pointer_x - press_x, pointer_y - press_y);
        match gesture.snap.axis {
            RenderInteractionAxisV1::Free => {}
            RenderInteractionAxisV1::Horizontal => dy = 0.0,
            RenderInteractionAxisV1::Vertical => dx = 0.0,
        }
        Ok(RenderInteractionTranslationPreviewV1 {
            capability: gesture.capability,
            dx,
            dy,
            bounds: gesture
                .selection
                .roots
                .iter()
                .map(|root| root.bounds.translated(dx, dy))
                .collect(),
        })
    }

    pub fn commit_render_interaction_translation_v1(
        &mut self,
        gesture: &RenderInteractionTranslationGestureV1,
        preview: &RenderInteractionTranslationPreviewV1,
    ) -> Result<CommittedRenderInteractionTranslationV1, RenderInteractionErrorV1> {
        self.require_gesture(gesture)?;
        self.require_selection(&gesture.selection)?;
        if preview.capability != gesture.capability {
            return Err(RenderInteractionErrorV1::PreviewMismatch);
        }
        let targets = gesture
            .selection
            .roots
            .iter()
            .map(|root| {
                TopLevelRootSelectorV1::new(root.identifier.clone(), root.kind)
                    .map_err(|_| RenderInteractionErrorV1::SelectionChanged)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let transform = TopLevelTransformV1::new(
            targets,
            TopLevelTransformModeV1::Translate {
                dx: preview.dx,
                dy: preview.dy,
            },
        )
        .map_err(|_| RenderInteractionErrorV1::SelectionChanged)?;
        let result = self
            .session
            .submit(
                gesture.selection.fence.revision(),
                SessionOperation::V1(SessionOperationV1::TransformTopLevelRoots { transform }),
            )
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        Ok(CommittedRenderInteractionTranslationV1 {
            changed: preview.dx != 0.0 || preview.dy != 0.0,
            result,
            selection: gesture.selection.clone(),
        })
    }

    /// Validate a preview without mutating the document. Tool-specific bridge
    /// owners use this before deriving a renderer-admitted detached candidate.
    pub(crate) fn validate_render_interaction_translation_preview_v1(
        &self,
        gesture: &RenderInteractionTranslationGestureV1,
        preview: &RenderInteractionTranslationPreviewV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        self.require_gesture(gesture)?;
        self.require_selection(&gesture.selection)?;
        (preview.capability == gesture.capability)
            .then_some(())
            .ok_or(RenderInteractionErrorV1::PreviewMismatch)
    }

    fn require_fence(&self, fence: DocumentFenceV1) -> Result<(), RenderInteractionErrorV1> {
        let snapshot = self
            .session
            .snapshot()
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        if snapshot.revision() != fence.revision() {
            return Err(RenderInteractionErrorV1::StaleRevision);
        }
        if snapshot.digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        Ok(())
    }
    fn require_observation(
        &self,
        value: &RenderInteractionObservationV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if value.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(value.fence)
    }
    fn require_selection(
        &self,
        value: &RenderInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        self.require_fence(value.fence)
    }
    fn require_gesture(
        &self,
        value: &RenderInteractionTranslationGestureV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        self.require_fence(value.selection.fence)
    }
    fn require_structure_observation(
        &self,
        value: &StructureInteractionObservationV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if value.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(value.fence)
    }
    fn require_structure_selection(
        &self,
        value: &StructureInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if value.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(value.fence)
    }
}
impl Deref for RenderInteractionSessionV1 {
    type Target = DocumentSession;
    fn deref(&self) -> &Self::Target {
        &self.session
    }
}
impl DerefMut for RenderInteractionSessionV1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}

fn toggle_roots(
    mut current: Vec<RenderInteractionRootV1>,
    candidates: Vec<RenderInteractionRootV1>,
) -> Vec<RenderInteractionRootV1> {
    for candidate in candidates {
        if let Some(index) = current
            .iter()
            .position(|root| root.identifier == candidate.identifier)
        {
            current.remove(index);
        } else {
            current.push(candidate);
        }
    }
    current
}
fn square_bounds(x: f64, y: f64, radius: f64) -> RenderInteractionBoundsV1 {
    RenderInteractionBoundsV1 {
        left: x - radius,
        top: y - radius,
        right: x + radius,
        bottom: y + radius,
    }
}
fn segment_distance(x: f64, y: f64, segment: StructureSegmentV1) -> f64 {
    let dx = segment.end_x - segment.start_x;
    let dy = segment.end_y - segment.start_y;
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((x - segment.start_x) * dx + (y - segment.start_y) * dy) / length).clamp(0.0, 1.0)
    };
    (x - (segment.start_x + t * dx)).hypot(y - (segment.start_y + t * dy))
}
fn segment_bounds(segments: &[StructureSegmentV1]) -> RenderInteractionBoundsV1 {
    let first = segments[0];
    segments.iter().skip(1).fold(
        RenderInteractionBoundsV1 {
            left: first.start_x.min(first.end_x) - first.stroke_radius,
            top: first.start_y.min(first.end_y) - first.stroke_radius,
            right: first.start_x.max(first.end_x) + first.stroke_radius,
            bottom: first.start_y.max(first.end_y) + first.stroke_radius,
        },
        |bounds, value| RenderInteractionBoundsV1 {
            left: bounds
                .left
                .min(value.start_x.min(value.end_x) - value.stroke_radius),
            top: bounds
                .top
                .min(value.start_y.min(value.end_y) - value.stroke_radius),
            right: bounds
                .right
                .max(value.start_x.max(value.end_x) + value.stroke_radius),
            bottom: bounds
                .bottom
                .max(value.start_y.max(value.end_y) + value.stroke_radius),
        },
    )
}
fn union_bounds(values: &[RenderInteractionBoundsV1]) -> RenderInteractionBoundsV1 {
    let first = values[0];
    values
        .iter()
        .skip(1)
        .fold(first, |bounds, value| RenderInteractionBoundsV1 {
            left: bounds.left.min(value.left),
            top: bounds.top.min(value.top),
            right: bounds.right.max(value.right),
            bottom: bounds.bottom.max(value.bottom),
        })
}

/// Return a conservative, renderer-issued envelope for a path-only bond.
///
/// P0.3 intentionally refuses path depictions rather than collapsing a wedge,
/// hash, or future filled bond into a fake editable centerline.  The envelope
/// includes every lowered path command and physical stroke width solely to
/// route an actual displayed primitive to the typed `DisplayOnly` recovery.
fn path_bounds(path: &PathOpV2) -> RenderInteractionBoundsV1 {
    let mut points = Vec::new();
    for command in path.commands() {
        match command {
            ScenePathCommandV2::MoveTo(point) | ScenePathCommandV2::LineTo(point) => {
                points.push((point.x(), point.y()));
            }
            ScenePathCommandV2::CubicTo {
                control_1,
                control_2,
                end,
            } => {
                points.push((control_1.x(), control_1.y()));
                points.push((control_2.x(), control_2.y()));
                points.push((end.x(), end.y()));
            }
            ScenePathCommandV2::Close => {}
        }
    }
    let (first_x, first_y) = points[0];
    let bounds = points.iter().skip(1).fold(
        RenderInteractionBoundsV1 {
            left: first_x,
            top: first_y,
            right: first_x,
            bottom: first_y,
        },
        |bounds, (x, y)| RenderInteractionBoundsV1 {
            left: bounds.left.min(*x),
            top: bounds.top.min(*y),
            right: bounds.right.max(*x),
            bottom: bounds.bottom.max(*y),
        },
    );
    inflate_bounds(
        bounds,
        path.stroke().map_or(0.0, |stroke| stroke.width().get()),
    )
}
fn structure_deletion_receipt(
    receipt: StructureDeletionReceiptV1,
) -> (
    Vec<String>,
    Vec<String>,
    Vec<StructureDeletionComponentFactsV1>,
) {
    let atoms = receipt
        .removed_atom_ids()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    let bonds = receipt
        .removed_bond_ids()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    let components = receipt
        .components()
        .iter()
        .map(|component| StructureDeletionComponentFactsV1 {
            molecule_id: component.molecule_id().as_str().to_owned(),
            atom_ids: component
                .atom_ids()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            bond_ids: component
                .bond_ids()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
        })
        .collect();
    (atoms, bonds, components)
}
fn toggle_structure_targets(
    mut current: Vec<StructureInteractionTargetV1>,
    candidates: Vec<StructureInteractionTargetV1>,
) -> Vec<StructureInteractionTargetV1> {
    for candidate in candidates {
        if let Some(index) = current.iter().position(|value| {
            value.kind == candidate.kind
                && value.identifier == candidate.identifier
                && value.molecule_id == candidate.molecule_id
        }) {
            current.remove(index);
        } else {
            current.push(candidate);
        }
    }
    current
}
fn roots_from_render(
    rendered: &RenderObservationV1,
    identities: &CompleteDocumentIdentityFactsV1,
) -> (
    Vec<RenderInteractionRootV1>,
    Vec<RenderInteractionExclusionV1>,
) {
    let mut planned = HashMap::new();
    for entry in rendered.molecule_plans() {
        let Some(identifier) = entry.molecule().source_id() else {
            continue;
        };
        if let Ok(bounds) = measure_molecule_render_plan_bounds_v1(entry.plan()) {
            planned.insert(
                identifier.to_owned(),
                RenderInteractionBoundsV1 {
                    left: bounds.left(),
                    top: bounds.top(),
                    right: bounds.right(),
                    bottom: bounds.bottom(),
                },
            );
        }
    }
    let mut roots = Vec::new();
    let mut exclusions = Vec::new();
    let mut emitted_exclusions = HashSet::new();
    for molecule in rendered.document().projection().molecules() {
        let (Some(_), Some(identifier)) = (molecule.id(), molecule.source_id()) else {
            continue;
        };
        let exclusion = root_exclusion_reason(identifier, identities, planned.get(identifier));
        if exclusion.is_none() {
            let bounds = planned
                .get(identifier)
                .expect("authorable root has a measured render-plan bounds");
            roots.push(RenderInteractionRootV1 {
                identifier: identifier.to_owned(),
                source_order: molecule.source_order(),
                bounds: *bounds,
                kind: TopLevelRootKindV1::Molecule,
            });
        }
        if let Some(reason) = exclusion
            && emitted_exclusions.insert(identifier.to_owned())
        {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier: identifier.to_owned(),
                reason,
            });
        }
    }
    for root in rendered
        .document()
        .projection()
        .presentation_stack()
        .roots()
    {
        let target = root.target();
        let diagnostic_identifier = target.source_id().map_or_else(
            || target.projection_key().as_str().to_owned(),
            str::to_owned,
        );
        let Some(identifier) = target.source_id().filter(|_| target.id().is_some()) else {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier: diagnostic_identifier,
                reason: RenderInteractionExclusionReasonV1::DisplayOnly,
            });
            continue;
        };
        let bounds = presentation_bounds_from_render(root, rendered);
        let exclusion = root_exclusion_reason(identifier, identities, bounds.as_ref());
        if let Some(bounds) = bounds.filter(|_| exclusion.is_none()) {
            roots.push(RenderInteractionRootV1 {
                identifier: identifier.to_owned(),
                source_order: target.source_order(),
                bounds,
                kind: presentation_root_kind(root),
            });
        }
        if let Some(reason) = exclusion
            && emitted_exclusions.insert(identifier.to_owned())
        {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier: identifier.to_owned(),
                reason,
            });
        }
    }
    for issue in rendered
        .document()
        .projection()
        .presentation_stack()
        .issues()
    {
        let target = issue.target();
        let identifier = target.source_id().map_or_else(
            || target.projection_key().as_str().to_owned(),
            str::to_owned,
        );
        if emitted_exclusions.insert(identifier.clone()) {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier,
                reason: RenderInteractionExclusionReasonV1::DisplayOnly,
            });
        }
    }
    roots.sort_by_key(RenderInteractionRootV1::source_order);
    (roots, exclusions)
}

fn presentation_root_kind(root: &PresentationRootProjectionV1) -> TopLevelRootKindV1 {
    match root {
        PresentationRootProjectionV1::Arrow { .. } => TopLevelRootKindV1::Arrow,
        PresentationRootProjectionV1::Plus { .. } => TopLevelRootKindV1::Plus,
        PresentationRootProjectionV1::Text { .. } => TopLevelRootKindV1::Text,
        PresentationRootProjectionV1::Rectangle { .. } => TopLevelRootKindV1::Rectangle,
        PresentationRootProjectionV1::Square { .. } => TopLevelRootKindV1::Square,
        PresentationRootProjectionV1::Oval { .. } => TopLevelRootKindV1::Oval,
        PresentationRootProjectionV1::Circle { .. } => TopLevelRootKindV1::Circle,
        PresentationRootProjectionV1::Polygon { .. } => TopLevelRootKindV1::Polygon,
        PresentationRootProjectionV1::Polyline { .. }
        | PresentationRootProjectionV1::Wavy { .. }
        | PresentationRootProjectionV1::RoundBracket { .. } => TopLevelRootKindV1::Polyline,
    }
}

fn reaction_choice_kind(
    semantic: DirectCdmlRootKindV1,
    observed: TopLevelRootKindV1,
) -> Option<ReactionAuthoringChoiceKindV1> {
    match (semantic, observed) {
        (DirectCdmlRootKindV1::Molecule, TopLevelRootKindV1::Molecule) => {
            Some(ReactionAuthoringChoiceKindV1::Molecule)
        }
        (DirectCdmlRootKindV1::Arrow, TopLevelRootKindV1::Arrow) => {
            Some(ReactionAuthoringChoiceKindV1::Arrow)
        }
        (DirectCdmlRootKindV1::Plus, TopLevelRootKindV1::Plus) => {
            Some(ReactionAuthoringChoiceKindV1::Plus)
        }
        (DirectCdmlRootKindV1::Text, TopLevelRootKindV1::Text) => {
            Some(ReactionAuthoringChoiceKindV1::ConditionText)
        }
        _ => None,
    }
}

fn reaction_root_exclusion_reason(
    semantic: DirectCdmlRootKindV1,
    observed: TopLevelRootKindV1,
) -> ReactionAuthoringExclusionReasonV1 {
    if direct_reaction_choice_kind(semantic).is_some() {
        debug_assert!(reaction_choice_kind(semantic, observed).is_none());
        ReactionAuthoringExclusionReasonV1::KindMismatch
    } else {
        ReactionAuthoringExclusionReasonV1::DisplayOnly
    }
}

fn reaction_exclusion_recovery(
    reason: ReactionAuthoringExclusionReasonV1,
) -> ReactionAuthoringExclusionRecoveryV1 {
    match reason {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => {
            ReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember
        }
        ReactionAuthoringExclusionReasonV1::Unrenderable
        | ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity
        | ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity
        | ReactionAuthoringExclusionReasonV1::KindMismatch => {
            ReactionAuthoringExclusionRecoveryV1::RepairDocument
        }
    }
}

fn reaction_exclusion_label(
    reason: ReactionAuthoringExclusionReasonV1,
    identifier: &str,
) -> String {
    let description = match reason {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => "Display-only root",
        ReactionAuthoringExclusionReasonV1::Unrenderable => "Unrenderable root",
        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity => {
            "Root missing direct CDML identity"
        }
        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity => {
            "Root with ambiguous direct CDML identity"
        }
        ReactionAuthoringExclusionReasonV1::KindMismatch => {
            "Root with renderer/semantic kind mismatch"
        }
    };
    format!("{description} {identifier}")
}

fn push_reaction_exclusion(
    exclusions: &mut Vec<ReactionAuthoringExclusionV1>,
    diagnosed: &mut HashSet<String>,
    identifier: &str,
    reason: ReactionAuthoringExclusionReasonV1,
    label: String,
) {
    if diagnosed.insert(identifier.to_owned()) {
        exclusions.push(ReactionAuthoringExclusionV1 {
            diagnostic_key: identifier.to_owned(),
            reason,
            recovery: reaction_exclusion_recovery(reason),
            label,
        });
    }
}

fn direct_reaction_choice_kind(
    kind: DirectCdmlRootKindV1,
) -> Option<ReactionAuthoringChoiceKindV1> {
    match kind {
        DirectCdmlRootKindV1::Molecule => Some(ReactionAuthoringChoiceKindV1::Molecule),
        DirectCdmlRootKindV1::Arrow => Some(ReactionAuthoringChoiceKindV1::Arrow),
        DirectCdmlRootKindV1::Plus => Some(ReactionAuthoringChoiceKindV1::Plus),
        DirectCdmlRootKindV1::Text => Some(ReactionAuthoringChoiceKindV1::ConditionText),
        DirectCdmlRootKindV1::Reaction | DirectCdmlRootKindV1::Other => None,
    }
}

fn reaction_choice_label(kind: ReactionAuthoringChoiceKindV1, identifier: &str) -> String {
    let name = match kind {
        ReactionAuthoringChoiceKindV1::Molecule => "Molecule",
        ReactionAuthoringChoiceKindV1::Arrow => "Arrow",
        ReactionAuthoringChoiceKindV1::Plus => "Plus",
        ReactionAuthoringChoiceKindV1::ConditionText => "Condition text",
    };
    format!("{name} {identifier}")
}

fn presentation_bounds_from_render(
    root: &PresentationRootProjectionV1,
    rendered: &RenderObservationV1,
) -> Option<RenderInteractionBoundsV1> {
    match root {
        PresentationRootProjectionV1::Plus { plus } => rendered
            .plus_renders()
            .iter()
            .find(|value| value.target().projection_key() == plus.target().projection_key())
            .map(|value| text_bounds(value.anchor().x(), value.anchor().y(), value.bounds())),
        PresentationRootProjectionV1::Text { text } => rendered
            .text_renders()
            .iter()
            .find(|value| value.target().projection_key() == text.target().projection_key())
            .map(|value| text_bounds(value.anchor().x(), value.anchor().y(), value.bounds())),
        PresentationRootProjectionV1::Arrow { arrow } => {
            let points = arrow
                .axis_path()
                .points()
                .iter()
                .chain(arrow.heads().iter().flat_map(|head| head.points().iter()));
            bounds_from_points(points, arrow.stroke().width().value())
        }
        PresentationRootProjectionV1::Polyline { polyline }
        | PresentationRootProjectionV1::Wavy { polyline }
        | PresentationRootProjectionV1::RoundBracket { polyline } => bounds_from_points(
            polyline.path().points().iter(),
            polyline.stroke().width().value(),
        ),
        PresentationRootProjectionV1::Rectangle { shape }
        | PresentationRootProjectionV1::Square { shape }
        | PresentationRootProjectionV1::Oval { shape }
        | PresentationRootProjectionV1::Circle { shape } => {
            let bounds = shape.bounds();
            Some(inflate_bounds(
                RenderInteractionBoundsV1 {
                    left: bounds.left(),
                    top: bounds.top(),
                    right: bounds.right(),
                    bottom: bounds.bottom(),
                },
                shape.stroke().width().value(),
            ))
        }
        PresentationRootProjectionV1::Polygon { polygon } => bounds_from_points(
            polygon.path().points().iter(),
            polygon.stroke().width().value(),
        ),
    }
}

fn text_bounds(
    anchor_x: f64,
    anchor_y: f64,
    bounds: ferrum_render::PresentationTextBoundsV1,
) -> RenderInteractionBoundsV1 {
    RenderInteractionBoundsV1 {
        left: anchor_x + bounds.left(),
        top: anchor_y + bounds.top(),
        right: anchor_x + bounds.right(),
        bottom: anchor_y + bounds.bottom(),
    }
}

fn bounds_from_points<'a>(
    points: impl Iterator<Item = &'a Point3V1>,
    stroke_width: f64,
) -> Option<RenderInteractionBoundsV1> {
    let mut points = points.peekable();
    let first = *points.peek()?;
    let mut bounds = RenderInteractionBoundsV1 {
        left: first.x(),
        top: first.y(),
        right: first.x(),
        bottom: first.y(),
    };
    for point in points {
        bounds.left = bounds.left.min(point.x());
        bounds.top = bounds.top.min(point.y());
        bounds.right = bounds.right.max(point.x());
        bounds.bottom = bounds.bottom.max(point.y());
    }
    Some(inflate_bounds(bounds, stroke_width))
}

fn inflate_bounds(
    bounds: RenderInteractionBoundsV1,
    stroke_width: f64,
) -> RenderInteractionBoundsV1 {
    let half = stroke_width / 2.0;
    RenderInteractionBoundsV1 {
        left: bounds.left - half,
        top: bounds.top - half,
        right: bounds.right + half,
        bottom: bounds.bottom + half,
    }
}

fn root_exclusion_reason(
    identifier: &str,
    identities: &CompleteDocumentIdentityFactsV1,
    bounds: Option<&RenderInteractionBoundsV1>,
) -> Option<RenderInteractionExclusionReasonV1> {
    if identities.is_ambiguous_identifier(identifier) {
        Some(RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier)
    } else if bounds.is_none() {
        Some(RenderInteractionExclusionReasonV1::UnrenderableDepiction)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SOURCE: &str = "<cdml><molecule id=\"m1\"><atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"a2\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"n1\"/></molecule><molecule id=\"m2\"><atom id=\"a3\" name=\"N\"><point x=\"60\" y=\"0\"/></atom></molecule></cdml>";
    const MIXED_SOURCE: &str = "<cdml><molecule id=\"molecule\"><atom id=\"atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><plus id=\"plus\"><point x=\"40\" y=\"0\"/></plus></cdml>";
    fn fence(session: &RenderInteractionSessionV1) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    #[test]
    fn structural_line_hit_and_marquee_follow_the_rendered_stroke_not_its_box() {
        let source = concat!(
            "<cdml><molecule id=\"m\">",
            "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"b\" name=\"O\"><point x=\"20\" y=\"20\"/></atom>",
            "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/>",
            "</molecule></cdml>",
        );
        let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
        let observation = session
            .observe_structure_interaction_v1(fence(&session))
            .expect("observe");
        let corner = session
            .select_structure_interaction_v1(
                &observation,
                None,
                StructureInteractionQueryV1::Point {
                    x: 1.0,
                    y: 19.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("corner query");
        assert!(corner.is_empty());
        let line = session
            .select_structure_interaction_v1(
                &observation,
                None,
                StructureInteractionQueryV1::Point {
                    x: 10.0,
                    y: 10.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("line query");
        assert!(
            line.targets()
                .iter()
                .any(|target| target.kind() == StructureTargetKindV1::Bond)
        );
        let clipped = session
            .select_structure_interaction_v1(
                &observation,
                None,
                StructureInteractionQueryV1::Marquee {
                    left: 0.0,
                    top: 0.0,
                    right: 20.0,
                    bottom: 20.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("clipped marquee");
        assert!(
            !clipped
                .targets()
                .iter()
                .any(|target| target.kind() == StructureTargetKindV1::Bond)
        );
        let contained = session
            .select_structure_interaction_v1(
                &observation,
                None,
                StructureInteractionQueryV1::Marquee {
                    left: -1.0,
                    top: -1.0,
                    right: 21.0,
                    bottom: 21.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("stroke-contained marquee");
        assert!(
            contained
                .targets()
                .iter()
                .any(|target| target.kind() == StructureTargetKindV1::Bond)
        );
    }

    #[test]
    fn structural_path_bond_is_a_typed_display_only_target() {
        let source = concat!(
            "<cdml><molecule id=\"m\">",
            "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"b\" name=\"O\"><point x=\"30\" y=\"0\"/></atom>",
            "<bond id=\"ab\" type=\"w1\" start=\"a\" end=\"b\"/>",
            "</molecule></cdml>",
        );
        let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
        let observation = session
            .observe_structure_interaction_v1(fence(&session))
            .expect("observe");
        let display = observation
            .targets()
            .iter()
            .find(|target| target.identifier() == "ab")
            .expect("wedge target remains visible to the interaction facade");
        assert_eq!(display.kind(), StructureTargetKindV1::DisplayOnly);
        let bounds = display.bounds();
        assert!(matches!(
            session.select_structure_interaction_v1(
                &observation,
                None,
                StructureInteractionQueryV1::Point {
                    x: (bounds.left() + bounds.right()) / 2.0,
                    y: (bounds.top() + bounds.bottom()) / 2.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            ),
            Err(RenderInteractionErrorV1::DisplayOnly)
        ));
    }
    #[test]
    fn render_plan_controls_point_marquee_translate_and_undo() {
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let observation = session
            .observe_render_interaction_v1(fence(&session))
            .expect("observe");
        assert_eq!(observation.roots().len(), 2);
        let selected = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Point {
                    x: 0.0,
                    y: 0.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("point hit");
        assert_eq!(selected.roots().len(), 1);
        let clipped = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Marquee {
                    left: -1.0,
                    top: -1.0,
                    right: 10.0,
                    bottom: 1.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("marquee");
        assert!(clipped.is_empty());
        let gesture = session
            .begin_render_interaction_translation_v1(
                &selected,
                0.0,
                0.0,
                RenderInteractionSnapV1::free(),
            )
            .expect("begin");
        let preview = session
            .preview_render_interaction_translation_v1(&gesture, 5.0, -2.0)
            .expect("preview");
        let committed = session
            .commit_render_interaction_translation_v1(&gesture, &preview)
            .expect("commit");
        assert!(committed.changed());
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        assert_eq!(
            session
                .undo(1)
                .expect("undo")
                .observation()
                .snapshot()
                .revision(),
            2
        );
    }

    #[test]
    fn structure_selection_deletes_atom_and_incident_bond_in_one_fenced_commit() {
        let source = concat!(
            "<cdml><molecule id=\"m\">",
            "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
            "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/>",
            "</molecule></cdml>",
        );
        let mut session =
            RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
        let snapshot = session.snapshot().expect("snapshot");
        let observation = session
            .observe_structure_interaction_v1(DocumentFenceV1::new(
                snapshot.revision(),
                *snapshot.digest(),
            ))
            .expect("observe");
        let atom = session
            .select_structure_interaction_v1(
                &observation,
                None,
                StructureInteractionQueryV1::Point {
                    x: 0.0,
                    y: 0.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("atom select");
        assert_eq!(atom.targets().len(), 1);
        assert_eq!(atom.targets()[0].kind(), StructureTargetKindV1::Atom);
        let selection = session
            .select_structure_interaction_v1(
                &observation,
                Some(&atom),
                StructureInteractionQueryV1::Point {
                    x: 10.0,
                    y: 0.0,
                    modifier: RenderInteractionModifierV1::Toggle,
                },
            )
            .expect("bond toggle");
        assert_eq!(selection.targets().len(), 2);
        let commit = session
            .commit_structure_deletion_v1(&selection)
            .expect("delete");
        assert_eq!(commit.removed_atoms(), ["a"]);
        assert_eq!(commit.removed_bonds(), ["ab"]);
        let molecule = &commit.result().observation().projection().molecules()[0];
        assert_eq!(molecule.atoms().len(), 1);
        assert!(molecule.bonds().is_empty());
        assert!(matches!(
            session.commit_structure_deletion_v1(&selection),
            Err(RenderInteractionErrorV1::StaleRevision)
        ));
    }

    #[test]
    fn view_hex_grid_policy_snaps_preview_delta_in_rust() {
        let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        let observation = session
            .observe_render_interaction_v1(fence(&session))
            .expect("observe");
        let selection = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    identifier: "m1".to_owned(),
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("select");
        let raw = session
            .begin_render_interaction_translation_v1(
                &selection,
                0.0,
                0.0,
                RenderInteractionSnapV1::free(),
            )
            .expect("raw gesture");
        let snapped = session
            .begin_render_interaction_translation_v1(
                &selection,
                0.0,
                0.0,
                RenderInteractionSnapV1::with_grid_policy(
                    RenderInteractionAxisV1::Free,
                    RenderInteractionGridSnapPolicyV1::ViewHexGrid,
                ),
            )
            .expect("grid gesture");
        let raw_preview = session
            .preview_render_interaction_translation_v1(&raw, 38.0, 18.0)
            .expect("raw preview");
        let grid_preview = session
            .preview_render_interaction_translation_v1(&snapped, 38.0, 18.0)
            .expect("grid preview");
        assert_eq!((raw_preview.dx(), raw_preview.dy()), (38.0, 18.0));
        assert_ne!(
            (grid_preview.dx(), grid_preview.dy()),
            (raw_preview.dx(), raw_preview.dy())
        );
    }
    #[test]
    fn unsupported_and_foreign_handles_are_refused_without_mutation() {
        let unsupported = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/><ftext><b>rich</b></ftext></atom></molecule></cdml>";
        let session =
            RenderInteractionSessionV1::new(DocumentSession::load(unsupported).expect("load"));
        let observation = session
            .observe_render_interaction_v1(fence(&session))
            .expect("observe");
        assert!(observation.roots().is_empty());
        assert_eq!(observation.exclusions().len(), 1);
        assert_eq!(observation.exclusions()[0].identifier(), "m");
        assert_eq!(
            observation.exclusions()[0].reason(),
            RenderInteractionExclusionReasonV1::UnrenderableDepiction
        );
        assert!(matches!(
            session.select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    identifier: "m".to_owned(),
                    modifier: RenderInteractionModifierV1::Replace,
                }
            ),
            Err(RenderInteractionErrorV1::UnrenderableDepiction)
        ));
        assert!(
            session
                .select_render_interaction_roots_v1(
                    &observation,
                    None,
                    RenderInteractionQueryV1::Point {
                        x: 0.0,
                        y: 0.0,
                        modifier: RenderInteractionModifierV1::Replace,
                    }
                )
                .expect("blank is not an excluded-root refusal")
                .is_empty()
        );
        let other = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
        assert!(matches!(
            other.select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Clear
            ),
            Err(RenderInteractionErrorV1::ForeignSession)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    }

    #[test]
    fn fragment_member_idref_does_not_exclude_renderable_root() {
        let opaque_declaration_collision = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><extension><fragment><bond id=\"m\"/></fragment></extension></cdml>";
        assert!(DocumentSession::load(opaque_declaration_collision).is_err());
        let source = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><fragment><bond id=\"m\"/></fragment></molecule></cdml>";
        let mut session = RenderInteractionSessionV1::new(
            DocumentSession::load(source).expect("fragment reference fixture loads"),
        );
        let observation = session
            .observe_render_interaction_v1(fence(&session))
            .expect("fragment reference fixture observes");
        assert_eq!(observation.roots().len(), 1);
        let selection = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    identifier: "m".to_owned(),
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("IDREF does not make molecule ambiguous");
        let gesture = session
            .begin_render_interaction_translation_v1(
                &selection,
                0.0,
                0.0,
                RenderInteractionSnapV1::free(),
            )
            .expect("begin move");
        let preview = session
            .preview_render_interaction_translation_v1(&gesture, 3.0, 0.0)
            .expect("preview move");
        assert_eq!(
            session
                .commit_render_interaction_translation_v1(&gesture, &preview)
                .expect("IDREF-safe move commits")
                .result()
                .observation()
                .snapshot()
                .revision(),
            1
        );
    }

    #[test]
    fn idless_presentation_root_is_display_only_not_a_transform_target() {
        let session = RenderInteractionSessionV1::new(
            DocumentSession::load("<cdml><plus><point x=\"4\" y=\"5\"/></plus></cdml>")
                .expect("display-only fixture loads"),
        );
        let observation = session
            .observe_render_interaction_v1(fence(&session))
            .expect("display-only fixture observes");
        assert!(observation.roots().is_empty());
        let [exclusion] = observation.exclusions() else {
            panic!("idless root must have one diagnostic");
        };
        assert_eq!(
            exclusion.reason(),
            RenderInteractionExclusionReasonV1::DisplayOnly
        );
        assert!(matches!(
            session.select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Root {
                    identifier: exclusion.identifier().to_owned(),
                    modifier: RenderInteractionModifierV1::Replace,
                },
            ),
            Err(RenderInteractionErrorV1::DisplayOnly)
        ));
    }

    #[test]
    fn reaction_authoring_classifies_renderable_vectors_and_kind_mismatches() {
        assert_eq!(
            reaction_root_exclusion_reason(
                DirectCdmlRootKindV1::Other,
                TopLevelRootKindV1::Rectangle,
            ),
            ReactionAuthoringExclusionReasonV1::DisplayOnly
        );
        assert_eq!(
            reaction_root_exclusion_reason(
                DirectCdmlRootKindV1::Arrow,
                TopLevelRootKindV1::Rectangle,
            ),
            ReactionAuthoringExclusionReasonV1::KindMismatch
        );
        assert_eq!(
            reaction_exclusion_recovery(ReactionAuthoringExclusionReasonV1::KindMismatch),
            ReactionAuthoringExclusionRecoveryV1::RepairDocument
        );
    }

    #[test]
    fn mixed_molecule_and_plus_selection_moves_in_one_history_commit() {
        let mut session = RenderInteractionSessionV1::new(
            DocumentSession::load(MIXED_SOURCE).expect("mixed fixture loads"),
        );
        let observation = session
            .observe_render_interaction_v1(fence(&session))
            .expect("mixed fixture observes");
        assert_eq!(observation.roots().len(), 2);
        let molecule = session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Point {
                    x: 0.0,
                    y: 0.0,
                    modifier: RenderInteractionModifierV1::Replace,
                },
            )
            .expect("molecule hit");
        let selected = session
            .select_render_interaction_roots_v1(
                &observation,
                Some(&molecule),
                RenderInteractionQueryV1::Point {
                    x: 40.0,
                    y: 0.0,
                    modifier: RenderInteractionModifierV1::Toggle,
                },
            )
            .expect("plus render-layout hit");
        assert_eq!(selected.roots().len(), 2);
        let gesture = session
            .begin_render_interaction_translation_v1(
                &selected,
                0.0,
                0.0,
                RenderInteractionSnapV1::free(),
            )
            .expect("begin mixed move");
        let preview = session
            .preview_render_interaction_translation_v1(&gesture, 7.0, 4.0)
            .expect("preview mixed move");
        let committed = session
            .commit_render_interaction_translation_v1(&gesture, &preview)
            .expect("one mixed session operation");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
        let projection = committed.result().observation().projection();
        assert!((projection.molecules()[0].atoms()[0].position().x() - 7.0).abs() < 0.01);
        let PresentationRootProjectionV1::Plus { plus } =
            &projection.presentation_stack().roots()[0]
        else {
            panic!("fixture must retain plus");
        };
        assert!((plus.anchor().x() - 47.0).abs() < 0.01);
        assert!((plus.anchor().y() - 4.0).abs() < 0.01);
        assert_eq!(
            session
                .undo(1)
                .expect("one mixed undo")
                .observation()
                .snapshot()
                .revision(),
            2
        );
    }
}
