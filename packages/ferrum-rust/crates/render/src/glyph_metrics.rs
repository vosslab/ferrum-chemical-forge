//! Toolkit-neutral, fully laid-out glyph measurement for render-plan generation.

use crate::{AtomLabelFacts, AtomLabelFontProfile, RenderError, TextRun};

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

    /// Lay out one canonical positive decimal atom-number annotation.
    fn layout_atom_number(
        &self,
        number: u64,
        font: &AtomLabelFontProfile,
    ) -> Result<TextRun, RenderError>;
}
