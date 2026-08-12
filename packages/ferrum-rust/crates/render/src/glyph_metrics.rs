//! Toolkit-neutral, fully laid-out glyph measurement for render-plan generation.

use crate::{AtomLabelFacts, AtomLabelFontProfile, RenderError, TextRun};
#[cfg(test)]
use crate::{PositiveFinite, RenderPoint, TextScript};

/// Finite atom-label extents relative to the declared atom position.
///
/// Bounds contain the text-operation origin. This permits a bond renderer to
/// clip a ray from the atom position without guessing the label's placement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphBounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl GlyphBounds {
    /// Construct finite, nonempty bounds containing the atom-local origin.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Self, RenderError> {
        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return Err(RenderError::InvalidRequest(
                "glyph bounds must be finite".to_owned(),
            ));
        }
        if min_x >= max_x
            || min_y >= max_y
            || min_x > 0.0
            || max_x < 0.0
            || min_y > 0.0
            || max_y < 0.0
        {
            return Err(RenderError::InvalidRequest(
                "glyph bounds must be nonempty and contain their local origin".to_owned(),
            ));
        }
        Ok(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Return the left extent relative to the label origin.
    #[must_use]
    pub const fn min_x(self) -> f64 {
        self.min_x
    }
    /// Return the lower extent relative to the label origin.
    #[must_use]
    pub const fn min_y(self) -> f64 {
        self.min_y
    }
    /// Return the right extent relative to the label origin.
    #[must_use]
    pub const fn max_x(self) -> f64 {
        self.max_x
    }
    /// Return the upper extent relative to the label origin.
    #[must_use]
    pub const fn max_y(self) -> f64 {
        self.max_y
    }
}

/// Fully placed semantic runs and the bounds of those exact runs.
#[derive(Clone, Debug, PartialEq)]
pub struct LaidOutAtomLabel {
    runs: Vec<TextRun>,
    bounds: GlyphBounds,
}

impl LaidOutAtomLabel {
    /// Construct a nonempty, fully positioned label and its clipping bounds.
    pub fn new(runs: Vec<TextRun>, bounds: GlyphBounds) -> Result<Self, RenderError> {
        if runs.is_empty() {
            return Err(RenderError::InvalidRequest(
                "laid-out atom label requires at least one run".to_owned(),
            ));
        }
        Ok(Self { runs, bounds })
    }
    /// Return drawing runs with fully explicit local geometry.
    #[must_use]
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }
    /// Return the clipping rectangle for those exact drawing runs.
    #[must_use]
    pub const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }
}

/// Shapes and measures one complete atom label without leaking toolkit types.
///
/// Implementors own any font-system state. They must return the same explicit
/// run geometry that a consumer paints and origin-containing finite bounds for
/// those exact runs; otherwise they return an actionable error.
pub trait GlyphMetrics {
    /// Lay out and measure the exact label under the exact requested font profile.
    fn layout_atom_label(
        &self,
        label: &AtomLabelFacts,
        font: &AtomLabelFontProfile,
    ) -> Result<LaidOutAtomLabel, RenderError>;
}

/// A deterministic, font-independent layout engine available only to render tests.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeterministicGlyphMetrics {
    character_width: PositiveFinite,
    ascent: PositiveFinite,
    descent: PositiveFinite,
}

#[cfg(test)]
impl DeterministicGlyphMetrics {
    /// Construct deterministic metrics in the same units as the render plan.
    #[must_use]
    pub const fn new(
        character_width: PositiveFinite,
        ascent: PositiveFinite,
        descent: PositiveFinite,
    ) -> Self {
        Self {
            character_width,
            ascent,
            descent,
        }
    }
}

#[cfg(test)]
impl GlyphMetrics for DeterministicGlyphMetrics {
    fn layout_atom_label(
        &self,
        label: &AtomLabelFacts,
        font: &AtomLabelFontProfile,
    ) -> Result<LaidOutAtomLabel, RenderError> {
        let scale = PositiveFinite::new(0.65)?;
        let pieces = label.text_pieces();
        let widths: Result<Vec<f64>, RenderError> = pieces
            .iter()
            .map(|(text, script)| {
                let scale = if *script == TextScript::Baseline {
                    1.0
                } else {
                    scale.get()
                };
                let width = self.character_width.get()
                    * text.chars().count() as f64
                    * font.size().get()
                    * scale;
                if width.is_finite() && width > 0.0 {
                    Ok(width)
                } else {
                    Err(RenderError::InvalidRequest(
                        "deterministic glyph width is not finite and positive".to_owned(),
                    ))
                }
            })
            .collect();
        let widths = widths?;
        let total_width: f64 = widths.iter().sum();
        if !total_width.is_finite() || total_width <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "deterministic label width is not finite and positive".to_owned(),
            ));
        }
        let mut cursor = -total_width / 2.0;
        let mut runs = Vec::with_capacity(pieces.len());
        let mut min_x: f64 = 0.0;
        let mut max_x: f64 = 0.0;
        let mut min_y: f64 = 0.0;
        let mut max_y: f64 = 0.0;
        for ((text, script), width) in pieces.into_iter().zip(widths) {
            let run_scale = if script == TextScript::Baseline {
                PositiveFinite::new(1.0)?
            } else {
                scale
            };
            let y = match script {
                TextScript::Baseline => 0.0,
                TextScript::Subscript => -self.descent.get() * font.size().get() * 0.8,
                TextScript::Superscript => self.ascent.get() * font.size().get() * 0.55,
            };
            let origin = RenderPoint::new(cursor, y)?;
            let ascent = self.ascent.get() * font.size().get() * run_scale.get();
            let descent = self.descent.get() * font.size().get() * run_scale.get();
            if !ascent.is_finite() || !descent.is_finite() || !y.is_finite() {
                return Err(RenderError::InvalidRequest(
                    "deterministic glyph layout is not finite".to_owned(),
                ));
            }
            min_x = min_x.min(cursor);
            max_x = max_x.max(cursor + width);
            min_y = min_y.min(y - descent);
            max_y = max_y.max(y + ascent);
            runs.push(TextRun::new(text, script, origin, run_scale)?);
            cursor += width;
        }
        LaidOutAtomLabel::new(runs, GlyphBounds::new(min_x, min_y, max_x, max_y)?)
    }
}
