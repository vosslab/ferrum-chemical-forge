//! Declarative, validated molecule render plans.
//!
//! # V1 wire contract
//!
//! `MoleculeRenderPlan` is the sole JSON boundary between Ferrum's authoritative
//! document projection and a disposable renderer. JSON accepts only
//! `ferrum-render-plan-v1`; unknown fields, variants, and future schemas are
//! rejected rather than guessed. A plan contains exactly one outcome for each
//! supplied target: a complete `RenderBatch` or a `RenderIssue`, never both.
//! Both outcome lists are strictly sorted by unique `source_order`, so a client
//! can merge them deterministically without inventing a tie breaker.
//!
//! Coordinates are finite Ferrum scene units with the document origin and axes
//! supplied by the authoritative projection. `Scene` line endpoints are scene
//! points. `AtomLocal` text origins are offsets from that batch's `anchor` in
//! the same coordinate system. The V1 grammar deliberately does not assign
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
//! Future operations, typography, and schema versions require a new validated
//! grammar revision; V1 intentionally provides no compatibility aliases.

mod atom_bond;
mod cairo_glyph_metrics;
mod error;
mod font_environment;
mod glyph_metrics;
mod haworth;
mod model;

/// Atom and bond source facts plus their render-plan builder.
pub use atom_bond::{
    AtomBondRenderRequest, AtomLabelFacts, AtomLabelFontProfile, AtomRenderTarget,
    BondRenderTarget, BondStyle, TargetVisibility, build_atom_bond_plan,
};
/// Cairo-backed glyph metrics using the verified Telex face.
pub use cairo_glyph_metrics::CairoGlyphMetrics;
/// Rendering errors and explicit target diagnostics.
pub use error::{RenderError, RenderIssue, RenderIssueKind};
/// Verified immutable Telex font asset environment.
pub use font_environment::{FerrumFontEnvironmentV1, FerrumFontId, FontAssetDescriptor};
#[cfg(test)]
pub use glyph_metrics::DeterministicGlyphMetrics;
/// Glyph-layout contract and exact layout bounds.
pub use glyph_metrics::{GlyphBounds, GlyphMetrics};
/// Haworth fragment lowering into the closed V1 render-plan grammar.
pub use haworth::{HaworthRenderRequest, lower_haworth_fragment};
/// Validated render-plan model and canonical JSON boundary.
pub use model::{
    BatchSpace, FontFace, LineOp, MoleculeRenderPlan, Paint, PositiveFinite, RenderBatch, RenderOp,
    RenderPoint, RenderRevision, RenderSchemaVersion, RenderTarget, Rgb24, TextOp, TextRun,
    TextScript,
};

#[cfg(test)]
mod tests;
