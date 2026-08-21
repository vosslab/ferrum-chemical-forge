//! Renderer-neutral complete-document admission proof.
//!
//! A document session holds the mutable candidate.  This crate accepts only
//! immutable CDML facts and returns an opaque proof when every direct root is in
//! the currently implemented normal-rendering grammar.

use ferrum_document_model::{
    CompleteCdmlDocumentV1, CompleteCdmlModelErrorV1, inspect_complete_cdml_v1,
};
use thiserror::Error;

#[derive(Debug)]
pub struct PreflightedDocumentRenderV1 {
    document: CompleteCdmlDocumentV1,
}

impl PreflightedDocumentRenderV1 {
    #[must_use]
    pub fn source(&self) -> &str {
        self.document.source()
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompleteDocumentPreflightErrorV1 {
    #[error("candidate CDML cannot be inspected for complete rendering")]
    InvalidCdml,
    #[error("candidate contains a root excluded from complete rendering: {0}")]
    ExcludedRoot(String),
    #[error("candidate contains rich text excluded from complete rendering")]
    RichTextExcluded,
}

const NORMAL_RENDER_ROOTS: &[&str] = &[
    "info",
    "metadata",
    "standard",
    "paper",
    "viewport",
    "molecule",
    "arrow",
    "plus",
    "text",
    "rect",
    "oval",
    "circle",
    "square",
    "polyline",
    "reaction",
    "external-data",
];

/// Prove that a complete candidate has no renderer exclusion before authoring.
pub fn preflight_complete_document_v1(
    source: &str,
) -> Result<PreflightedDocumentRenderV1, CompleteDocumentPreflightErrorV1> {
    let document = inspect_complete_cdml_v1(source).map_err(map_model_error)?;
    if document.contains_rich_text() {
        return Err(CompleteDocumentPreflightErrorV1::RichTextExcluded);
    }
    for root in document.direct_roots() {
        if !NORMAL_RENDER_ROOTS.contains(&root.as_str()) {
            return Err(CompleteDocumentPreflightErrorV1::ExcludedRoot(root.clone()));
        }
    }
    Ok(PreflightedDocumentRenderV1 { document })
}

fn map_model_error(_: CompleteCdmlModelErrorV1) -> CompleteDocumentPreflightErrorV1 {
    CompleteDocumentPreflightErrorV1::InvalidCdml
}
