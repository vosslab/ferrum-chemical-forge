//! Declarative, validated molecule render plans.
//!
//! # V2 wire contract
//!
//! `MoleculeRenderPlan` is the sole JSON boundary between Ferrum's authoritative
//! document projection and a disposable renderer. JSON accepts only
//! `ferrum-render-plan-v2`; unknown fields, variants, and future schemas are
//! rejected rather than guessed. A plan contains exactly one outcome for each
//! supplied target: a complete `RenderBatch` or a `RenderIssue`, never both.
//! Both outcome lists are strictly sorted by unique `source_order`, so a client
//! can merge them deterministically without inventing a tie breaker.
//!
//! Coordinates are finite Ferrum scene units with the document origin and axes
//! supplied by the authoritative projection. `Scene` line endpoints are scene
//! points. `AtomLocal` text origins are offsets from that batch's `anchor` in
//! the same coordinate system. The V2 grammar deliberately does not assign
//! screen pixels, DPI, scaling, clipping, or toolkit defaults to a renderer.
//! Every accepted zero coordinate serializes as `0.0`, never `-0.0`.
//!
//! Within a batch, operations paint in strictly increasing `z` order. Equal
//! `z` values are invalid. Fonts, colors, and text are exact presentation facts:
//! no font substitution, color fallback, or invisible placeholder operation is
//! permitted by this grammar. A renderer that cannot honor a batch reports its
//! own display failure; it must not silently alter the plan. An
//! `UnsupportedFeature` or `UnrenderableTarget` issue is displayed as an
//! excluded target diagnostic and produces no batch.
//!
//! V2 adds source-owned finite scene paths for bond batches. Future operations,
//! typography, and schema versions require a new validated grammar revision.

mod atom_bond;
mod authored_direct_glycosidic_haworth;
mod bond_style;
mod compact_group;
mod complete_document_admission_v1;
mod composite_recording_v1;
mod direct_draw_stream_v1;
mod direct_glycosidic_haworth;
mod directed_stereo_bond;
mod document_artifact_v1;
mod document_bond_replacement_v1;
mod document_content_bounds_v1;
mod document_plan_v1;
mod document_precommit_overlay_v1;
mod document_vector_v1;
mod double_bond_carrier_mark;
mod draw_stream_molecule_v1;
mod draw_stream_v1;
mod error;
mod font_environment;
mod glyph_metrics;
mod glyph_placement;
mod haworth;
mod haworth_front_bond;
mod model;
mod pdf_backend;
mod png_backend;
mod presentation_path_v1;
mod render_target;
mod scene_path_v2;
mod shape_ops;
mod standalone_text;
mod svg_backend;
mod verified_telex_glyph_metrics;
mod document {
    pub(crate) mod depiction_profile;
    pub(crate) mod depiction_profile_resolution;
    pub(crate) mod observation;
    pub(crate) mod plan;
}
mod font {
    pub(crate) mod telex;
}
mod presentation {
    pub(crate) mod path;
    pub(crate) mod plan;
    pub(crate) mod plus;
    pub(crate) mod text;
    pub(crate) mod vector;
}

/// Atom and bond source facts plus their render-plan builder.
pub use atom_bond::{
    AtomBondRenderRequest, AtomLabelFacts, AtomLabelFontProfile, AtomMarkRenderFacts,
    AtomMarkRenderKind, AtomNumberLabelFacts, AtomRenderTarget, BondRenderTarget, TargetVisibility,
    build_atom_bond_plan,
};
/// In-process-only renderer profile for durable committed direct Haworth facts.
pub use authored_direct_glycosidic_haworth::{
    AuthoredDirectGlycosidicHaworthRenderPlanV1, AuthoredDirectGlycosidicHaworthRenderRequestV1,
    lower_authored_direct_glycosidic_haworth_v1,
};
/// Closed bond-style vocabulary shared by renderer lowerers.
pub use bond_style::BondStyle;
/// Closed compact-group label primitives issued from typed document projections.
pub use compact_group::{CompactGroupBondEndpointV1, CompactGroupRenderPrimitiveV1};
/// Pure lowering and classification for complete-document render admission.
pub use complete_document_admission_v1::{
    AcceptedCompleteRenderPresentationV1, AcceptedCompleteRenderRootV1, AcceptedCompleteRenderV1,
    AcceptedRenderOverlayRequestV1, AcceptedRenderOverlayTargetKindV1,
    AcceptedRenderOverlayTargetV1, COMPLETE_DOCUMENT_RENDERER_SCHEMA_V1,
    CompleteDocumentAdmissionErrorV1, admit_complete_document_render_v1,
    admit_complete_document_render_with_resolved_v1,
};
/// Internal desktop paint recording of an authenticated whole-document composite.
pub use composite_recording_v1::{
    CompositeFillRuleV1, CompositeFillV1, CompositeLineCapV1, CompositeLineJoinV1,
    CompositePaintKindV1, CompositePathCommandV1, CompositeRecordingBudgetV1,
    CompositeRecordingErrorV1, CompositeRecordingEventV1, CompositeRecordingResourceV1,
    CompositeRecordingV1, CompositeRootKindV1, CompositeStrokeV1, CompositeStyleV1,
    record_document_render_composite_v1,
};
/// In-process-only renderer profile for accepted direct glycosidic Haworth facts.
pub use direct_glycosidic_haworth::{
    DirectGlycosidicHaworthRenderPlanV1, DirectGlycosidicHaworthRenderRequestV1,
    lower_direct_glycosidic_haworth_v1,
};
/// Immutable document projection resolver, render plans, and byte sinks.
pub use document::depiction_profile::{
    DEPICTION_PROFILE_SCHEMA_V1, DEPICTION_RESOLUTION_SCHEMA_V1, DepictionError,
    DepictionIssueCodeV1, DepictionIssueV1, DepictionProfileV1, DepictionResolutionV1,
    DepictionSuppressionV1, DirectGlycosidicHaworthStyleV1, MoleculeMemberDepictionIssueV1,
    render_document_projection_v1, resolve_direct_glycosidic_haworth_style_v1,
};
pub use document::observation::{
    DocumentMoleculeRenderPlanV2, MoleculeRenderRootV1, RESOLVED_DOCUMENT_RENDER_SCHEMA_V1,
    RenderDocumentProvenanceV1, ResolvedDocumentRenderErrorV1, ResolvedDocumentRenderV1,
    ResolvedDocumentRenderWireV1, resolve_document_render_v1,
};
pub use document::plan::{DocumentRenderPlanCompositionError, compose_document_render_plan_v1};
/// Renderer-neutral receipt for a completed whole-page artifact.
pub use document_artifact_v1::{DocumentRenderArtifactV1, DocumentRenderReportV1};
/// Checked in-process selective replacement of one molecule's bond outcomes.
pub use document_bond_replacement_v1::{
    DocumentBondReplacementErrorV1, DocumentRenderCompositeV1, compose_document_bond_replacement_v1,
};
/// Conservative content fitting over the shared lowered document draw stream.
pub use document_content_bounds_v1::{
    DocumentContentBoundsErrorV1, MoleculeContentBoundsV1, fit_document_render_plan_to_content_v1,
    measure_molecule_render_plan_bounds_v1,
};
/// Renderer-neutral whole-page document composition model.
pub use document_plan_v1::{
    DocumentMoleculeRenderContentV1, DocumentRenderContentV1, DocumentRenderExclusionV1,
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, DocumentRenderRootV1, DocumentTextLayoutV1,
    DocumentTextOpV1, RenderViewportV1,
};
/// Immutable identifier-free precommit paint data for accepted document mutations.
pub use document_precommit_overlay_v1::{
    DocumentPrecommitOverlayV1, DocumentPrecommitPaintPrimitiveV1,
};
/// Checked generic vector operations for direct document roots.
pub use document_vector_v1::{
    DocumentVectorOpV1, DocumentVectorRootV1, PathCommandV1, StrokeV1, VectorFillRuleV1,
    VectorStrokeLineCapV1, VectorStrokeLineJoinV1,
};
/// Explicit, durable E/Z carrier-mark operation facts.
pub use double_bond_carrier_mark::{DoubleBondCarrierMarkDirectionV1, DoubleBondCarrierMarkOp};
/// Rendering errors and explicit target diagnostics.
pub use error::{RenderError, RenderIssue, RenderIssueKind};
pub use font::telex::{VerifiedTelexRegularV1, verified_telex_regular_v1};
/// Verified immutable Telex font asset environment.
pub use font_environment::{FerrumFontEnvironmentV1, FerrumFontId, FontAssetDescriptor};
/// Glyph-layout contract and exact layout bounds.
pub use glyph_metrics::{GlyphBounds, GlyphMetrics};
/// Closed Telex glyph identifiers, positions, and script roles.
pub use glyph_placement::{GlyphPlacement, TextScript};
/// Haworth fragment lowering into the closed V1 render-plan grammar.
pub use haworth::{HaworthRenderRequest, lower_haworth_fragment};
/// Source-owned V2 geometry for detached Haworth q1/w1 previews.
pub use haworth_front_bond::build_haworth_front_preview_ops;
/// Validated render-plan model and canonical JSON boundary.
pub use model::{
    BatchSpace, FontFace, LineOp, MaskOp, MoleculeRenderPlan, Paint, PositiveFinite, RenderBatch,
    RenderDisplayLayerV1, RenderOp, RenderPoint, RenderProvenance, RenderRevision,
    RenderSchemaVersion, Rgb24, TextOp, TextRun,
};
/// In-memory, outline-only vector PDF V1 lowering with explicit caller-owned limits.
pub use pdf_backend::{
    PdfComplexityResourceV1, PdfDocumentV1, PdfOutputBudgetV1, PdfPlanComplexityBudgetV1,
    PdfRenderComplexityObservationV1, PdfRenderError, PdfRenderRequestV1,
    render_document_plan_to_pdf_v1,
};
/// Bounded, in-memory PNG V1 lowering with caller-owned output limits.
pub use png_backend::{
    PngBackgroundV1, PngDocumentV1, PngOutputBudgetV1, PngPixelSizeV1, PngRenderError,
    PngRenderRequestV1, render_document_plan_to_png_v1,
};
pub use presentation::path::{
    PresentationPathCompositionErrorV1, lower_presentation_points_path_v1,
    lower_presentation_polyline_path_v1,
};
/// Pure renderer-owned plan for one immutable presentation stack.
pub use presentation::plan::{
    PRESENTATION_PREVIEW_RENDER_PLAN_SCHEMA_V1, PRESENTATION_RENDER_PLAN_SCHEMA_V1,
    PresentationPreviewRenderPlanV1, PresentationPreviewRenderRootV1, PresentationRenderBoundsV1,
    PresentationRenderPlanV1, PresentationRenderRootV1, lower_arrow_preview_v1,
    lower_standard_plus_preview_v1, render_presentation_stack_v1,
};
pub use presentation::plus::{DocumentPlusRenderV1, PresentationTextBoundsV1};
/// Renderer-owned geometry for interactive curved terminal-arrow previews.
pub use presentation::text::DocumentTextRenderV1;
#[doc(hidden)]
pub use presentation::vector::lower_presentation_vector_v1;
/// Toolkit-neutral lowering of authored control paths into frozen cubic commands.
pub use presentation_path_v1::{
    PathKindV1, PresentationPathErrorV1, PresentationPathV1, lower_authored_control_path_v1,
};
/// Stable visual and durable document identity for one render-plan target.
pub use render_target::RenderTarget;
/// Neutral V2 path facts shared by molecule render-plan consumers.
pub use scene_path_v2::{PathOpV2, ScenePathCommandV2, ScenePathStrokeV2};
pub use shape_ops::EllipseOp;
/// Exact fixed-content text layout issued by the verified Telex renderer.
pub use standalone_text::{
    CenteredTextLayout, PresentationGlyphRun, PresentationTextLayout, PresentationTextOp,
    PresentationTextSourceRun,
};
/// In-memory SVG V1 lowering for one validated molecule render plan.
pub use svg_backend::{
    SvgDocumentV1, SvgOutputBudgetV1, SvgRenderError, SvgViewportV1,
    render_direct_glycosidic_haworth_to_svg_v1, render_document_plan_to_svg_v1,
    render_document_plan_to_svg_with_budget_v1, render_plan_to_svg_v1,
};
/// Pure-Rust TrueType design metrics using the verified Telex face.
pub use verified_telex_glyph_metrics::{
    FontBaselineMetrics, GlyphRunMetrics, VerifiedTelexGlyphMetrics,
};

/// Maximum completed SVG bytes returned by the first local render profile.
pub const LOCAL_SVG_COMPLETED_BYTES_V1: usize = 64 * 1024 * 1024;
/// Maximum completed PDF bytes under the ordinary local V1 policy.
pub const LOCAL_PDF_COMPLETED_BYTES_V1: usize = 64 * 1024 * 1024;
/// Maximum counted PDF traversal items under the ordinary local V1 policy.
pub const LOCAL_PDF_PLAN_ITEMS_V1: usize = 1024 * 1024;
/// Maximum lowered PDF path commands under the ordinary local V1 policy.
pub const LOCAL_PDF_DRAW_PATH_COMMANDS_V1: usize = 8 * 1024 * 1024;
/// Maximum pre-allocation RGBA bytes under the ordinary local V1 policy.
pub const LOCAL_PNG_RAW_RGBA_BYTES_V1: usize = 256 * 1024 * 1024;
/// Maximum completed PNG bytes under the ordinary local V1 policy.
pub const LOCAL_PNG_ENCODED_BYTES_V1: usize = 64 * 1024 * 1024;

/// Build the complete ordinary local PDF policy from explicit caller caps.
pub fn local_pdf_render_request_v1(
    max_completed_bytes: usize,
    max_plan_items: usize,
    max_draw_path_commands: usize,
) -> Result<PdfRenderRequestV1, PdfRenderError> {
    Ok(PdfRenderRequestV1 {
        output: PdfOutputBudgetV1::new(max_completed_bytes)?,
        complexity: PdfPlanComplexityBudgetV1 {
            max_plan_items,
            max_draw_path_commands,
            max_exclusion_report_bytes: 0,
        },
    })
}

/// Build one local PNG request from exact caller-owned raster facts and caps.
#[must_use]
pub const fn local_png_render_request_v1(
    pixels: PngPixelSizeV1,
    background: PngBackgroundV1,
    max_raw_rgba_bytes: usize,
    max_encoded_bytes: usize,
) -> PngRenderRequestV1 {
    PngRenderRequestV1 {
        pixels,
        background,
        budget: PngOutputBudgetV1 {
            max_raw_rgba_bytes,
            max_encoded_bytes,
        },
    }
}

#[cfg(test)]
mod tests;
