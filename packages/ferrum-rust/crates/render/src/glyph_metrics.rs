//! Toolkit-neutral, fully laid-out glyph measurement for render-plan generation.

use crate::{AtomLabelFacts, AtomLabelFontProfile, RenderError, RenderPoint, TextRun, TextScript};

/// Finite visible-ink extents relative to a label-local origin.
///
/// Atom-label layout receipts remain crate-private. This immutable bounds DTO
/// stays public because document-render consumes compact-group hit bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphBounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl GlyphBounds {
    /// Construct finite, nonempty visible-ink bounds.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Self, RenderError> {
        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return Err(RenderError::InvalidRequest(
                "glyph bounds must be finite".to_owned(),
            ));
        }
        if min_x >= max_x || min_y >= max_y {
            return Err(RenderError::InvalidRequest(
                "glyph bounds must be nonempty".to_owned(),
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

    /// Canonicalize a verified core outline as symmetric bounds at the origin.
    ///
    /// The caller must first prove that the underlying glyph placement is the
    /// exact Telex-centered placement. This representation removes only the
    /// final floating-point addition residual from an otherwise centered
    /// outline; it never makes an arbitrary translated outline valid.
    pub(crate) fn canonical_centered_at_origin(self) -> Result<Self, RenderError> {
        let half_width = (self.max_x - self.min_x) / 2.0;
        let half_height = (self.max_y - self.min_y) / 2.0;
        Self::new(-half_width, -half_height, half_width, half_height)
    }
}

/// Atom-label geometry that attaches bonds to the structural element run.
///
/// The source atom identity, rather than a later bond lowerer, determines the
/// core run.  This keeps alignment valid for labels with hydrogens and charge
/// annotations whose total advance is intentionally asymmetric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AtomLabelAttachmentGeometry {
    core_element_ink_bounds: GlyphBounds,
    core_element_ink_center: RenderPoint,
}

impl AtomLabelAttachmentGeometry {
    /// Construct verified core-element ink geometry.
    pub(crate) fn new(core_element_ink_bounds: GlyphBounds) -> Result<Self, RenderError> {
        let calculated_center = RenderPoint::new(
            (core_element_ink_bounds.min_x() + core_element_ink_bounds.max_x()) / 2.0,
            (core_element_ink_bounds.min_y() + core_element_ink_bounds.max_y()) / 2.0,
        )?;
        if calculated_center.x() != 0.0 || calculated_center.y() != 0.0 {
            return Err(RenderError::InvalidRequest(
                "atom-label core element ink must be centered at the local atom origin".to_owned(),
            ));
        }
        Ok(Self {
            core_element_ink_bounds,
            core_element_ink_center: RenderPoint::new(0.0, 0.0)?,
        })
    }

    /// Return the exact structural-element ink rectangle.
    #[must_use]
    pub(crate) const fn core_element_ink_bounds(self) -> GlyphBounds {
        self.core_element_ink_bounds
    }

    /// Return the exact structural-element ink center.
    #[must_use]
    pub(crate) const fn core_element_ink_center(self) -> RenderPoint {
        self.core_element_ink_center
    }
}

/// Fully placed semantic runs and the bounds of those exact runs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LaidOutAtomLabel {
    runs: Vec<TextRun>,
    bounds: GlyphBounds,
    attachment: AtomLabelAttachmentGeometry,
    core_element_run_index: u32,
}

impl LaidOutAtomLabel {
    /// Construct a nonempty, fully positioned label and its clipping bounds.
    pub(crate) fn new(
        runs: Vec<TextRun>,
        bounds: GlyphBounds,
        attachment: AtomLabelAttachmentGeometry,
        core_element_run_index: u32,
    ) -> Result<Self, RenderError> {
        if runs.is_empty() {
            return Err(RenderError::InvalidRequest(
                "laid-out atom label requires at least one run".to_owned(),
            ));
        }
        let center = attachment.core_element_ink_center();
        if center.x() != 0.0 || center.y() != 0.0 {
            return Err(RenderError::InvalidRequest(
                "atom-label core element ink center must equal the local atom origin".to_owned(),
            ));
        }
        let core = attachment.core_element_ink_bounds();
        if core.min_x() < bounds.min_x()
            || core.min_y() < bounds.min_y()
            || core.max_x() > bounds.max_x()
            || core.max_y() > bounds.max_y()
        {
            return Err(RenderError::InvalidRequest(
                "atom-label core element ink must lie within the full visible ink bounds"
                    .to_owned(),
            ));
        }
        let core_index = usize::try_from(core_element_run_index).map_err(|_| {
            RenderError::InvalidRequest("atom-label core run index is not addressable".to_owned())
        })?;
        let core_run = runs.get(core_index).ok_or_else(|| {
            RenderError::InvalidRequest(
                "atom-label core run index is outside laid-out label runs".to_owned(),
            )
        })?;
        if core_run.script() != TextScript::Baseline {
            return Err(RenderError::InvalidRequest(
                "atom-label core run must use baseline script".to_owned(),
            ));
        }
        Ok(Self {
            runs,
            bounds,
            attachment,
            core_element_run_index,
        })
    }
    /// Return drawing runs with fully explicit local geometry.
    #[must_use]
    pub(crate) fn runs(&self) -> &[TextRun] {
        &self.runs
    }
    /// Return the clipping rectangle for those exact drawing runs.
    #[must_use]
    pub(crate) const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }

    /// Return the structural attachment geometry for the exact positioned runs.
    #[must_use]
    pub(crate) const fn attachment(&self) -> AtomLabelAttachmentGeometry {
        self.attachment
    }

    /// Return the source-issued structural element run in `runs`.
    ///
    /// This is not inferred by the atom lowerer or a presentation consumer.
    #[must_use]
    pub(crate) const fn core_element_run_index(&self) -> u32 {
        self.core_element_run_index
    }
}

/// Shapes and measures one complete atom label without leaking toolkit types.
///
/// Implementors own any font-system state. They must return the same explicit
/// run geometry that a consumer paints and finite visible-ink bounds for those
/// exact runs; otherwise they return an actionable error.
pub(crate) trait GlyphMetrics {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AtomLabelRenderV1, FerrumFontEnvironmentV1, FontFace, InkBoundsV1, PositiveFinite,
        RenderPaintV3, RenderPoint, Rgb24, TextOp, TextScript, VerifiedTelexGlyphMetrics,
    };

    fn size(value: f64) -> PositiveFinite {
        PositiveFinite::new(value).expect("test font size is valid")
    }

    fn font() -> AtomLabelFontProfile {
        AtomLabelFontProfile::new(
            FontFace::telex_regular(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint is valid")),
        )
    }

    #[test]
    fn verified_telex_issues_exact_centered_structural_core_runs() {
        let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
        let metrics =
            VerifiedTelexGlyphMetrics::new(&environment).expect("Telex metrics are available");
        for (element, charge, hydrogens) in [("C", 0, 0), ("O", 0, 0), ("Cl", 0, 0), ("N", 1, 3)] {
            let facts = AtomLabelFacts::new(element, None, charge, hydrogens)
                .expect("test atom facts are admitted");
            let layout = metrics
                .layout_atom_label(&facts, &font())
                .expect("verified Telex lays out test label");
            assert_eq!(layout.core_element_run_index(), 0);
            assert_eq!(layout.runs()[0].text(), element);
            assert_eq!(layout.runs()[0].script(), TextScript::Baseline);
            let core = layout.attachment().core_element_ink_bounds();
            assert_eq!((core.min_x() + core.max_x()) / 2.0, 0.0);
            assert_eq!((core.min_y() + core.max_y()) / 2.0, 0.0);
            let text = TextOp::new(
                RenderPoint::new(0.0, 0.0).expect("test origin is finite"),
                layout.runs().to_vec(),
                FontFace::telex_regular(),
                size(12.0),
                RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
                30,
            )
            .expect("verified Telex text is valid");
            let core_index = layout.core_element_run_index();
            let full = InkBoundsV1::from_glyph_bounds(
                metrics
                    .v1_atom_label_ink_bounds(&text, core_index as usize)
                    .expect("canonical full label ink is available"),
            );
            let selected_core = InkBoundsV1::from_glyph_bounds(
                metrics
                    .v1_centered_core_run_ink_bounds(&text, core_index as usize)
                    .expect("canonical core ink is available"),
            );
            AtomLabelRenderV1::new(None, text, core_index, full, selected_core)
                .expect("durable atom label accepts the issued core run and bounds");
        }
    }

    #[test]
    fn attachment_rejects_tiny_forged_nonzero_core_center() {
        let forged = GlyphBounds::new(-1.0, -1.0, 1.0 + 1.0e-15, 1.0)
            .expect("forged bounds are geometrically nonempty");
        assert!(AtomLabelAttachmentGeometry::new(forged).is_err());
    }

    #[test]
    fn durable_label_rejects_reordered_baseline_run_as_the_structural_core() {
        let environment = FerrumFontEnvironmentV1::load().expect("bundled Telex is verified");
        let metrics =
            VerifiedTelexGlyphMetrics::new(&environment).expect("Telex metrics are available");
        let facts = AtomLabelFacts::new("N", None, 1, 3).expect("test atom facts are admitted");
        let layout = metrics
            .layout_atom_label(&facts, &font())
            .expect("verified Telex lays out ammonium");
        let mut reordered_runs = layout.runs().to_vec();
        reordered_runs.swap(0, 1);
        assert_eq!(reordered_runs[0].script(), TextScript::Baseline);
        assert_eq!(reordered_runs[1].script(), TextScript::Baseline);
        let text = TextOp::new(
            RenderPoint::new(0.0, 0.0).expect("test origin is finite"),
            reordered_runs,
            FontFace::telex_regular(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
            30,
        )
        .expect("reordered runs retain exact glyph placements");
        let core_index = layout.core_element_run_index();
        let full = InkBoundsV1::from_glyph_bounds(
            metrics
                .v1_text_ink_bounds(&text)
                .expect("reordered text still has exact Telex ink"),
        );
        let original_core =
            InkBoundsV1::from_glyph_bounds(layout.attachment().core_element_ink_bounds());
        assert!(AtomLabelRenderV1::new(None, text, core_index, full, original_core).is_err());
    }
}
