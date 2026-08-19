//! API-owned verified glyph layout for direct-root Text labels.

use crate::{
    Paint, PositiveFinite, PresentationTextOp, PresentationTextSourceRun, RenderError, RenderPoint,
    Rgb24, TextScript, VerifiedTelexGlyphMetrics,
};
use ferrum_document::{
    PresentationRecordKindV1, PresentationTargetV1, PresentationTextStyleV1,
    Rgb24V1 as DocumentRgb24V1, TextProjectionV1,
};
use serde::{Deserialize, Deserializer, Serialize};

use crate::PresentationTextBoundsV1;

/// One direct-root Text label with exact verified Telex layout.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentTextRenderV1 {
    target: PresentationTargetV1,
    anchor: RenderPoint,
    source_runs: Vec<PresentationTextSourceRun>,
    operation: PresentationTextOp,
    bounds: PresentationTextBoundsV1,
    background: Option<Paint>,
}

impl<'de> Deserialize<'de> for DocumentTextRenderV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = TextRenderWireV1::deserialize(deserializer)?;
        Self::validated(
            wire.target,
            wire.anchor,
            wire.source_runs,
            wire.operation,
            wire.bounds,
            wire.background,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl DocumentTextRenderV1 {
    pub(crate) fn from_projection(
        text: &TextProjectionV1,
        metrics: &VerifiedTelexGlyphMetrics,
    ) -> Result<Self, RenderError> {
        let source_runs = source_runs(text)?;
        let foreground = paint(text.font().color())?;
        let layout = metrics.layout_presentation_text(
            &source_runs,
            PositiveFinite::new(text.font().size().value())?,
            foreground,
        )?;
        Ok(Self {
            target: text.target().clone(),
            anchor: RenderPoint::new(text.anchor().x(), text.anchor().y())?,
            source_runs,
            operation: layout.operation().clone(),
            bounds: PresentationTextBoundsV1::from_glyph_bounds(layout.bounds()),
            background: text.background().color().map(paint).transpose()?,
        })
    }

    fn validated(
        target: PresentationTargetV1,
        anchor: RenderPoint,
        source_runs: Vec<PresentationTextSourceRun>,
        operation: PresentationTextOp,
        bounds: PresentationTextBoundsV1,
        background: Option<Paint>,
    ) -> Result<Self, String> {
        if target.record_kind() != PresentationRecordKindV1::Text {
            return Err("Text render target has the wrong persistent kind".to_owned());
        }
        let environment =
            crate::FerrumFontEnvironmentV1::load().map_err(|error| error.to_string())?;
        let metrics =
            VerifiedTelexGlyphMetrics::new(&environment).map_err(|error| error.to_string())?;
        let expected = metrics
            .layout_presentation_text(&source_runs, operation.size(), operation.paint().clone())
            .map_err(|error| error.to_string())?;
        if expected.operation() != &operation {
            return Err("Text operation does not match verified Telex layout".to_owned());
        }
        if PresentationTextBoundsV1::from_glyph_bounds(expected.bounds()) != bounds {
            return Err("Text bounds do not match verified Telex layout".to_owned());
        }
        Ok(Self {
            target,
            anchor,
            source_runs,
            operation,
            bounds,
            background,
        })
    }

    /// Return the direct-root durable-or-local Text target.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    /// Return the authored top-left scene anchor.
    #[must_use]
    pub const fn anchor(&self) -> RenderPoint {
        self.anchor
    }

    /// Return normalized source runs used to validate this wire value.
    #[must_use]
    pub fn source_runs(&self) -> &[PresentationTextSourceRun] {
        &self.source_runs
    }

    /// Return exact positioned Telex glyph runs.
    #[must_use]
    pub const fn operation(&self) -> &PresentationTextOp {
        &self.operation
    }

    /// Return complete logical Text bounds relative to the authored anchor.
    #[must_use]
    pub const fn bounds(&self) -> PresentationTextBoundsV1 {
        self.bounds
    }

    /// Return the explicit optional background paint.
    #[must_use]
    pub fn background(&self) -> Option<&Paint> {
        self.background.as_ref()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextRenderWireV1 {
    target: PresentationTargetV1,
    anchor: RenderPoint,
    source_runs: Vec<PresentationTextSourceRun>,
    operation: PresentationTextOp,
    bounds: PresentationTextBoundsV1,
    background: Option<Paint>,
}

fn source_runs(text: &TextProjectionV1) -> Result<Vec<PresentationTextSourceRun>, RenderError> {
    text.runs()
        .iter()
        .map(|run| {
            let script = if run.styles().contains(&PresentationTextStyleV1::Subscript) {
                TextScript::Subscript
            } else if run.styles().contains(&PresentationTextStyleV1::Superscript) {
                TextScript::Superscript
            } else {
                TextScript::Baseline
            };
            PresentationTextSourceRun::new(run.text(), script)
        })
        .collect()
}

fn paint(color: &DocumentRgb24V1) -> Result<Paint, RenderError> {
    let digits = color.as_str().strip_prefix('#').unwrap_or(color.as_str());
    Ok(Paint::rgb24(Rgb24::new(digits)?))
}
