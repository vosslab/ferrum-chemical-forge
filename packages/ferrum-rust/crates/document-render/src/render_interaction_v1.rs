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
    CompleteDocumentIdentityFactsV1, DirectCdmlRootKindV1, DirectCdmlSemanticIndexV1,
    DocumentFenceV1, DocumentRenderObservationV1, DocumentSession, DocumentSessionError,
    DocumentSmartsSnapshotErrorV1, PreparedDocumentSmartsSnapshotV1, PresentationCreationGestureV1,
    PresentationGestureErrorV1, PresentationGestureKindV1, PresentationGesturePoint2V1,
    PresentationGestureSnapPolicyV1, PresentationGestureStyleV1, PresentationRecordKindV1,
    SessionOperation, SessionOperationResultV1, SessionOperationV1, StructureDeletionReceiptV1,
    TopLevelRootKindV1, TopLevelRootSelectorV1, TopLevelTransformModeV1, TopLevelTransformV1,
};
use ferrum_geometry::{HexGrid, Point2};
use ferrum_render::{
    PathOpV2, PresentationRenderPlanV1, PresentationRenderRootV1, RenderOp, ScenePathCommandV2,
    measure_molecule_render_plan_bounds_v1, render_presentation_stack_v1,
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
    /// A first-class compact-group label derived by the Rust renderer.
    CompactGroup,
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
    CompactGroup,
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
            StructureInteractionGeometryV1::CompactGroup => self.bounds.contains_point(x, y),
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
            StructureInteractionGeometryV1::CompactGroup => self.bounds.contained_by(rectangle),
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
    #[error("the prospective structural deletion cannot be rendered")]
    UnrenderableCandidate,
    #[error("structural selection cannot span more than one direct molecule")]
    CrossMoleculeSelection,
    #[error("a structural target no longer belongs to the observed molecule")]
    UnsupportedTarget,
    #[error("the current document cannot supply a complete SMARTS target set")]
    UnsupportedDocument,
}

#[derive(Debug)]
pub struct RenderInteractionSessionV1 {
    session: DocumentSession,
    origin: u64,
}

#[path = "render_interaction_helpers_v1.rs"]
mod render_interaction_helpers_v1;
#[path = "render_interaction_session_v1.rs"]
mod render_interaction_session_v1;

use render_interaction_helpers_v1::*;

#[cfg(test)]
#[path = "render_interaction_tests_v1.rs"]
mod render_interaction_tests_v1;
