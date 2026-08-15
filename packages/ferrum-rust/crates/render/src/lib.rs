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
mod composite_recording_v1;
mod direct_draw_stream_v1;
mod direct_glycosidic_haworth;
mod directed_stereo_bond;
mod document_artifact_v1;
mod document_bond_replacement_v1;
mod document_content_bounds_v1;
mod document_plan_v1;
mod document_vector_v1;
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
mod scene_path_v2;
mod shape_ops;
mod standalone_text;
mod svg_backend;
mod verified_telex_glyph_metrics;

/// Atom and bond source facts plus their render-plan builder.
pub use atom_bond::{
    AtomBondRenderRequest, AtomLabelFacts, AtomLabelFontProfile, AtomMarkRenderFacts,
    AtomMarkRenderKind, AtomNumberLabelFacts, AtomRenderTarget, BondRenderTarget, BondStyle,
    TargetVisibility, build_atom_bond_plan,
};
/// In-process-only renderer profile for durable committed direct Haworth facts.
pub use authored_direct_glycosidic_haworth::{
    AuthoredDirectGlycosidicHaworthRenderPlanV1, AuthoredDirectGlycosidicHaworthRenderRequestV1,
    lower_authored_direct_glycosidic_haworth_v1,
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
/// Source-owned directed stereo-bond geometry for committed batches and previews.
pub use directed_stereo_bond::build_directed_bond_preview_ops;
/// Renderer-neutral receipt for a completed whole-page artifact.
pub use document_artifact_v1::{DocumentRenderArtifactV1, DocumentRenderReportV1};
/// Checked in-process selective replacement of one molecule's bond outcomes.
pub use document_bond_replacement_v1::{
    DocumentBondReplacementErrorV1, DocumentRenderCompositeV1, compose_document_bond_replacement_v1,
};
/// Conservative content fitting over the shared lowered document draw stream.
pub use document_content_bounds_v1::{
    DocumentContentBoundsErrorV1, fit_document_render_plan_to_content_v1,
};
/// Renderer-neutral whole-page document composition model.
pub use document_plan_v1::{
    DocumentRenderContentV1, DocumentRenderExclusionV1, DocumentRenderIdentityV1,
    DocumentRenderOutcomeV1, DocumentRenderPlanV1, DocumentRenderRootV1, DocumentTextLayoutV1,
    DocumentTextOpV1, RenderViewportV1,
};
/// Checked generic vector operations for direct document roots.
pub use document_vector_v1::{
    DocumentVectorOpV1, DocumentVectorRootV1, PathCommandV1, StrokeV1, VectorFillRuleV1,
    VectorStrokeLineCapV1, VectorStrokeLineJoinV1,
};
/// Rendering errors and explicit target diagnostics.
pub use error::{RenderError, RenderIssue, RenderIssueKind};
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
    RenderSchemaVersion, RenderTarget, Rgb24, TextOp, TextRun,
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

#[cfg(test)]
mod tests;
