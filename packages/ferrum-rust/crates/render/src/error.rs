//! Error and diagnostic types for declarative render plans.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::RenderTarget;
use crate::render_target::RenderPlanEntryContextV1;

/// A non-fatal diagnostic for document content outside an available render slice.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum RenderIssueKind {
    /// A valid document feature has no renderer implementation yet.
    UnsupportedFeature { feature: String },
    /// A target cannot form a complete, correct operation batch.
    UnrenderableTarget { reason: String },
}

impl RenderIssueKind {
    pub(crate) fn validate(&self) -> Result<(), RenderError> {
        let detail = match self {
            Self::UnsupportedFeature { feature } => feature,
            Self::UnrenderableTarget { reason } => reason,
        };
        if detail.trim().is_empty() {
            return Err(RenderError::InvalidRequest(
                "render issue detail must not be blank".to_owned(),
            ));
        }
        Ok(())
    }
}

/// A target-specific non-fatal render diagnostic.
///
/// An issue owns a durable target and its contractual paint order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderIssue {
    target: RenderTarget,
    paint_order: u32,
    kind: RenderIssueKind,
}

impl RenderIssue {
    /// Construct a non-fatal target diagnostic.
    pub fn new(
        target: RenderTarget,
        paint_order: u32,
        kind: RenderIssueKind,
    ) -> Result<Self, RenderError> {
        kind.validate()?;
        Ok(Self {
            target,
            paint_order,
            kind,
        })
    }

    /// Construct a diagnostic from renderer-local source facts.
    pub(crate) fn from_context(
        context: RenderPlanEntryContextV1,
        kind: RenderIssueKind,
    ) -> Result<Self, RenderError> {
        Self::new(context.target().clone(), context.paint_order(), kind)
    }

    /// Return the excluded target and its stable document position.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }

    /// Return the contractual paint order for this issue.
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }

    /// Return the non-fatal reason that this target has no render batch.
    #[must_use]
    pub const fn kind(&self) -> &RenderIssueKind {
        &self.kind
    }

    pub(crate) fn validate(&self) -> Result<(), RenderError> {
        self.kind.validate()
    }
}

impl<'de> Deserialize<'de> for RenderIssue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireIssue {
            target: RenderTarget,
            paint_order: u32,
            kind: RenderIssueKind,
        }
        let wire = WireIssue::deserialize(deserializer)?;
        Self::new(wire.target, wire.paint_order, wire.kind).map_err(serde::de::Error::custom)
    }
}

/// Failures at the native render-plan boundary.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum RenderError {
    /// A value does not meet the durable render-plan grammar.
    #[error("invalid render plan: {0}")]
    InvalidRequest(String),
    /// JSON could not be decoded into a validated render plan.
    #[error("invalid render-plan JSON: {0}")]
    InvalidJson(String),
    /// Serializing a valid plan failed unexpectedly.
    #[error("could not serialize render plan: {0}")]
    Serialization(String),
    /// Allocating a caller-owned validated render structure failed.
    #[error("could not allocate render structure")]
    ResourceExhausted,
}
