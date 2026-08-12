//! Versioned importer for the historical compact carbohydrate notation.
//!
//! This namespace keeps its types out of `ferrum_domain::sugar`'s root. New
//! Ferrum domain APIs accept explicit typed carbohydrate facts rather than
//! extend this migration-only notation.

pub use super::semantic::{
    Anomer, BackboneToken, FootnoteFamily, FootnoteKey, LegacyCompactSugarCodeV1,
    LegacyCompactSugarRenderRequestV1, RingForm, SugarPosition, SugarPrefix, SugarSeries,
};
pub use super::syntax::LegacyCompactSugarCodeV1Error;
