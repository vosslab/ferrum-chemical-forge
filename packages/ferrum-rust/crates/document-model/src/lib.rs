//! Renderer-readable, immutable facts about a complete retained CDML document.
//!
//! This crate deliberately has no session, history, generated identifiers, or
//! toolkit state.  It is the lower ownership boundary used by renderer contracts.

use thiserror::Error;
use xmlparser::{ElementEnd, Token, Tokenizer};

mod render_model_v1;
pub use render_model_v1::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompleteCdmlDocumentV1 {
    source: String,
    direct_roots: Vec<String>,
    contains_rich_text: bool,
}

impl CompleteCdmlDocumentV1 {
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn direct_roots(&self) -> &[String] {
        &self.direct_roots
    }

    #[must_use]
    pub const fn contains_rich_text(&self) -> bool {
        self.contains_rich_text
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[non_exhaustive]
pub enum CompleteCdmlModelErrorV1 {
    #[error("complete CDML is not well-formed XML")]
    MalformedXml,
    #[error("complete CDML does not have one cdml root")]
    InvalidRoot,
}

/// Parse only the immutable top-level facts required by renderer admission.
pub fn inspect_complete_cdml_v1(
    source: &str,
) -> Result<CompleteCdmlDocumentV1, CompleteCdmlModelErrorV1> {
    let mut depth = 0_u32;
    let mut root_seen = false;
    let mut direct_roots = Vec::new();
    let mut contains_rich_text = false;
    for token in Tokenizer::from(source) {
        match token.map_err(|_| CompleteCdmlModelErrorV1::MalformedXml)? {
            Token::ElementStart { local, .. } => {
                let name = local.as_str();
                if depth == 0 {
                    if root_seen || name != "cdml" {
                        return Err(CompleteCdmlModelErrorV1::InvalidRoot);
                    }
                    root_seen = true;
                } else if depth == 1 {
                    direct_roots.push(name.to_owned());
                }
                if name == "ftext" {
                    contains_rich_text = true;
                }
                depth = depth
                    .checked_add(1)
                    .ok_or(CompleteCdmlModelErrorV1::MalformedXml)?;
            }
            Token::ElementEnd {
                end: ElementEnd::Close(_, _),
                ..
            }
            | Token::ElementEnd {
                end: ElementEnd::Empty,
                ..
            } => {
                depth = depth
                    .checked_sub(1)
                    .ok_or(CompleteCdmlModelErrorV1::MalformedXml)?;
            }
            _ => {}
        }
    }
    if !root_seen || depth != 0 {
        return Err(CompleteCdmlModelErrorV1::InvalidRoot);
    }
    Ok(CompleteCdmlDocumentV1 {
        source: source.to_owned(),
        direct_roots,
        contains_rich_text,
    })
}
