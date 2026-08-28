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
    CompleteDocumentIdentityFactsV1, DocumentFenceV1, DocumentObjectIdV1,
    DocumentRenderObservationV1, DocumentSession, DocumentSessionError,
    DocumentSmartsSnapshotErrorV1, PreparedDocumentSmartsSnapshotV1, PresentationCreationGestureV1,
    PresentationGestureErrorV1, PresentationGestureKindV1, PresentationGesturePoint2V1,
    PresentationGestureSnapPolicyV1, PresentationGestureStyleV1, PresentationRecordKindV1,
    SessionOperationResultV1, TopLevelRootKindV1,
};
use ferrum_render::{
    DocumentRenderContentV1, DocumentRenderOutcomeV1, PathOpV3, PresentationRenderPlanV1,
    PresentationRenderRootV1, RenderOp, ScenePathCommandV3, compose_document_render_plan_v1,
    measure_molecule_render_plan_bounds_v1, render_presentation_stack_v1,
};
use thiserror::Error;

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
    document_object_id: DocumentObjectIdV1,
    paint_order: u32,
    bounds: RenderInteractionBoundsV1,
    kind: TopLevelRootKindV1,
}
impl RenderInteractionRootV1 {
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
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
    document_object_id: DocumentObjectIdV1,
    paint_order: u32,
    kind: ReactionAuthoringChoiceKindV1,
    availability: ReactionAuthoringChoiceAvailabilityV1,
    label: String,
    bounds: RenderInteractionBoundsV1,
}
impl ReactionAuthoringChoiceV1 {
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
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

/// Immutable Rust-issued reaction-authoring facts carried by one direct-root observation.
///
/// This is deliberately not a selection, gesture, receipt, candidate, or
/// transaction. Its enclosing direct-root observation owns the session and
/// document fence that make these display facts current.
#[derive(Clone, Debug)]
pub struct ReactionAuthoringObservationV1 {
    choices: Vec<ReactionAuthoringChoiceV1>,
    exclusions: Vec<ReactionAuthoringExclusionV1>,
}
impl ReactionAuthoringObservationV1 {
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
        document_object_id: DocumentObjectIdV1,
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
    reaction_authoring: ReactionAuthoringObservationV1,
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
    document_object_id: DocumentObjectIdV1,
    reason: RenderInteractionExclusionReasonV1,
}
impl RenderInteractionExclusionV1 {
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
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
    /// Rust-classified role candidates for the renderer-admitted direct roots.
    #[must_use]
    pub const fn reaction_authoring(&self) -> &ReactionAuthoringObservationV1 {
        &self.reaction_authoring
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
}

/// One exact child hit envelope derived by Rust from the fenced document projection.
#[derive(Clone, Debug, PartialEq)]
pub struct StructureInteractionTargetV1 {
    molecule_object_id: DocumentObjectIdV1,
    object_id: DocumentObjectIdV1,
    source_order: u32,
    kind: StructureTargetKindV1,
    bounds: RenderInteractionBoundsV1,
    geometry: StructureInteractionGeometryV1,
}
#[derive(Clone, Debug, PartialEq)]
enum StructureInteractionGeometryV1 {
    Atom {
        x: f64,
        y: f64,
    },
    Bond {
        segments: Vec<StructureSegmentV1>,
        hit_slop: f64,
    },
    /// Bounds of one directed stereo primitive lowered by the Rust renderer.
    DirectedStereoBondEnvelope,
    CompactGroup,
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
    pub(crate) const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn molecule_object_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_object_id
    }
    #[must_use]
    pub const fn object_id(&self) -> &DocumentObjectIdV1 {
        &self.object_id
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
            StructureInteractionGeometryV1::Bond { segments, hit_slop } => {
                segments.iter().any(|segment| {
                    segment_distance(x, y, *segment) <= hit_slop.max(segment.stroke_radius)
                })
            }
            StructureInteractionGeometryV1::DirectedStereoBondEnvelope => {
                self.bounds.contains_point(x, y)
            }
            StructureInteractionGeometryV1::CompactGroup => self.bounds.contains_point(x, y),
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
            StructureInteractionGeometryV1::Bond { segments, .. } => {
                segments.iter().all(|segment| {
                    rectangle.left + segment.stroke_radius <= segment.start_x
                        && segment.start_x <= rectangle.right - segment.stroke_radius
                        && rectangle.top + segment.stroke_radius <= segment.start_y
                        && segment.start_y <= rectangle.bottom - segment.stroke_radius
                        && rectangle.left + segment.stroke_radius <= segment.end_x
                        && segment.end_x <= rectangle.right - segment.stroke_radius
                        && rectangle.top + segment.stroke_radius <= segment.end_y
                        && segment.end_y <= rectangle.bottom - segment.stroke_radius
                })
            }
            StructureInteractionGeometryV1::DirectedStereoBondEnvelope => {
                self.bounds.contained_by(rectangle)
            }
            StructureInteractionGeometryV1::CompactGroup => self.bounds.contained_by(rectangle),
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
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
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
    removed_atom_count: usize,
    removed_bond_count: usize,
    removed_compact_group_count: usize,
}
impl CommittedStructureDeletionV1 {
    #[must_use]
    pub fn result(&self) -> &SessionOperationResultV1 {
        &self.result
    }
    #[must_use]
    pub const fn removed_atom_count(&self) -> usize {
        self.removed_atom_count
    }
    #[must_use]
    pub const fn removed_bond_count(&self) -> usize {
        self.removed_bond_count
    }
    #[must_use]
    pub const fn removed_compact_group_count(&self) -> usize {
        self.removed_compact_group_count
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

#[derive(Debug)]
pub struct RenderInteractionTranslationGestureV1 {
    origin: u64,
    authoring_capability: ferrum_document::AuthoringCapabilityV1,
    selection: RenderInteractionSelectionV1,
    press_x: f64,
    press_y: f64,
    snap: RenderInteractionSnapV1,
}
#[derive(Clone, Debug)]
pub struct RenderInteractionTranslationPreviewV1 {
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
    #[error("the renderer rejected the prospective interaction document state")]
    RendererAdmission,
    #[error("the prospective structural deletion cannot be rendered")]
    UnrenderableCandidate,
    #[error("structural selection cannot span more than one direct molecule")]
    CrossMoleculeSelection,
    #[error("a structural target no longer belongs to the observed molecule")]
    UnsupportedTarget,
    #[error("select exactly one compact group without atoms or bonds before deleting it")]
    InvalidCompactGroupDeletionSelection,
    #[error("the compact group deletion topology requires document repair before retry")]
    InvalidCompactGroupDeletionTopology,
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
#[path = "root_translation_interaction_v1.rs"]
mod root_translation_interaction_v1;

use render_interaction_helpers_v1::*;

#[cfg(test)]
#[path = "render_interaction_tests_v1.rs"]
mod render_interaction_tests_v1;
