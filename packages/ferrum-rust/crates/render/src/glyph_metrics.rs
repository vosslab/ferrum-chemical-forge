//! Toolkit-neutral, fully laid-out glyph measurement for render-plan generation.

use ferrum_geometry::Vector2;

use crate::glyph_outline_support::GlyphOutlineSupport;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, PositiveFinite, RenderError, RenderPoint, TextRun,
    TextScript, VerifiedMoleculeLabelGlyphMetrics,
};

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
    /// exact Atkinson Hyperlegible Next-centered placement. This representation removes only the
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
    core_outline_support: GlyphOutlineSupport,
    core_element_run_index: u32,
    non_core_run_ink_bounds: Vec<GlyphBounds>,
}

impl LaidOutAtomLabel {
    /// Construct a nonempty, fully positioned label and its clipping bounds.
    pub(crate) fn new(
        runs: Vec<TextRun>,
        bounds: GlyphBounds,
        attachment: AtomLabelAttachmentGeometry,
        core_outline_support: GlyphOutlineSupport,
        core_element_run_index: u32,
        non_core_run_ink_bounds: Vec<GlyphBounds>,
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
            core_outline_support,
            core_element_run_index,
            non_core_run_ink_bounds,
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

    /// Return exact verified-Atkinson Hyperlegible Next outline support for the structural run.
    #[must_use]
    pub(crate) const fn core_outline_support(&self) -> &GlyphOutlineSupport {
        &self.core_outline_support
    }

    /// Return the source-issued structural element run in `runs`.
    ///
    /// This is not inferred by the atom lowerer or a presentation consumer.
    #[must_use]
    pub(crate) const fn core_element_run_index(&self) -> u32 {
        self.core_element_run_index
    }

    /// Return exact non-core Atkinson Hyperlegible Next run ink rectangles for directional clipping.
    #[must_use]
    pub(crate) fn non_core_run_ink_bounds(&self) -> &[GlyphBounds] {
        &self.non_core_run_ink_bounds
    }

    /// Move an explicit hydrogen group away from a sole rightward bond.
    ///
    /// The isotope keeps its conventional upper-left placement, the hydrogen
    /// and its count move to the left baseline, and a formal charge returns to
    /// the core's upper-right instead of retaining advance reserved by the
    /// moved hydrogen. The structural run and its outline support do not move.
    pub(crate) fn avoid_rightward_bond_with_explicit_hydrogen(
        self,
        direction: Vector2,
        spacing: PositiveFinite,
        size: PositiveFinite,
        metrics: &VerifiedMoleculeLabelGlyphMetrics,
    ) -> Result<Self, RenderError> {
        if direction.x() <= 0.25 {
            return Ok(self);
        }
        let core_index = self.core_element_run_index as usize;
        let hydrogen_index = core_index + 1;
        let Some(hydrogen) = self.runs.get(hydrogen_index) else {
            return Ok(self);
        };
        if hydrogen.text() != "H" || hydrogen.script() != TextScript::Baseline {
            return Ok(self);
        }

        let core_bounds = self.attachment.core_element_ink_bounds();
        let mut run_bounds = Vec::with_capacity(self.runs.len());
        let mut non_core_bounds = self.non_core_run_ink_bounds.iter().copied();
        for index in 0..self.runs.len() {
            run_bounds.push(if index == core_index {
                core_bounds
            } else {
                non_core_bounds
                    .next()
                    .expect("laid-out non-core bounds match run identity")
            });
        }

        let mut group_end = hydrogen_index;
        if self
            .runs
            .get(hydrogen_index + 1)
            .is_some_and(|run| run.script() == TextScript::Subscript)
        {
            group_end += 1;
        }
        let hydrogen_right = run_bounds[hydrogen_index..=group_end]
            .iter()
            .map(|bounds| bounds.max_x())
            .fold(f64::NEG_INFINITY, f64::max);
        let hydrogen_shift = core_bounds.min_x() - spacing.get() - hydrogen_right;

        let mut runs = self.runs;
        for index in hydrogen_index..=group_end {
            runs[index] = translated_run(&runs[index], hydrogen_shift)?;
            run_bounds[index] = translated_bounds(run_bounds[index], hydrogen_shift)?;
        }

        let charge_index = runs.len() - 1;
        if charge_index > group_end
            && runs[charge_index].script() == TextScript::Superscript
            && runs[charge_index].text().ends_with(['+', '-'])
        {
            let charge_shift =
                core_bounds.max_x() + spacing.get() * 0.25 - run_bounds[charge_index].min_x();
            runs[charge_index] = translated_run(&runs[charge_index], charge_shift)?;
            run_bounds[charge_index] = translated_bounds(run_bounds[charge_index], charge_shift)?;
        }

        let text_origin = RenderPoint::new(0.0, 0.0)?;
        let mut bounds = None;
        let mut updated_non_core = Vec::with_capacity(self.non_core_run_ink_bounds.len());
        for (index, run) in runs.iter().enumerate() {
            let run_bounds = metrics.run_ink_bounds_at(text_origin, size, run)?;
            bounds = Some(match bounds {
                Some(existing) => union_bounds(existing, run_bounds)?,
                None => run_bounds,
            });
            if index != core_index {
                updated_non_core.push(run_bounds);
            }
        }
        let bounds = union_bounds(
            bounds.expect("laid-out label has at least its structural run"),
            core_bounds,
        )?;
        Self::new(
            runs,
            bounds,
            self.attachment,
            self.core_outline_support,
            self.core_element_run_index,
            updated_non_core,
        )
    }
}

fn translated_run(run: &TextRun, x: f64) -> Result<TextRun, RenderError> {
    TextRun::new(
        run.text(),
        run.script(),
        RenderPoint::new(run.origin().x() + x, run.origin().y())?,
        run.glyphs().to_vec(),
        run.scale(),
    )
}

fn translated_bounds(bounds: GlyphBounds, x: f64) -> Result<GlyphBounds, RenderError> {
    GlyphBounds::new(
        bounds.min_x() + x,
        bounds.min_y(),
        bounds.max_x() + x,
        bounds.max_y(),
    )
}

fn union_bounds(first: GlyphBounds, second: GlyphBounds) -> Result<GlyphBounds, RenderError> {
    GlyphBounds::new(
        first.min_x().min(second.min_x()),
        first.min_y().min(second.min_y()),
        first.max_x().max(second.max_x()),
        first.max_y().max(second.max_y()),
    )
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
        AtomLabelRenderV1, FerrumFontEnvironment, FontFace, InkBoundsV1, PositiveFinite,
        RenderPaintV3, RenderPoint, Rgb24, TextOp, TextScript, VerifiedMoleculeLabelGlyphMetrics,
    };
    use ferrum_geometry::Vector2;

    fn size(value: f64) -> PositiveFinite {
        PositiveFinite::new(value).expect("test font size is valid")
    }

    fn font() -> AtomLabelFontProfile {
        AtomLabelFontProfile::new(
            FontFace::molecule_label(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint is valid")),
        )
    }

    #[test]
    fn verified_molecule_label_font_issues_exact_centered_structural_core_runs() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        for (element, charge, hydrogens) in [("C", 0, 0), ("O", 0, 0), ("Cl", 0, 0), ("N", 1, 3)] {
            let facts = AtomLabelFacts::new(element, None, charge, hydrogens)
                .expect("test atom facts are admitted");
            let layout = metrics
                .layout_atom_label(&facts, &font())
                .expect("verified Atkinson Hyperlegible Next lays out test label");
            assert_eq!(layout.core_element_run_index(), 0);
            assert_eq!(layout.runs()[0].text(), element);
            assert_eq!(layout.runs()[0].script(), TextScript::Baseline);
            let core = layout.attachment().core_element_ink_bounds();
            assert_eq!((core.min_x() + core.max_x()) / 2.0, 0.0);
            assert_eq!((core.min_y() + core.max_y()) / 2.0, 0.0);
            let text = TextOp::new(
                RenderPoint::new(0.0, 0.0).expect("test origin is finite"),
                layout.runs().to_vec(),
                FontFace::molecule_label(),
                size(12.0),
                RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
                30,
            )
            .expect("verified Atkinson Hyperlegible Next text is valid");
            let core_index = layout.core_element_run_index();
            let full = InkBoundsV1::from_glyph_bounds(
                metrics
                    .atom_label_ink_bounds(&text, core_index as usize)
                    .expect("canonical full label ink is available"),
            );
            let selected_core = InkBoundsV1::from_glyph_bounds(
                metrics
                    .centered_core_run_ink_bounds(&text, core_index as usize)
                    .expect("canonical core ink is available"),
            );
            AtomLabelRenderV1::new(None, text, core_index, size(1.5), full, selected_core)
                .expect("durable atom label accepts the issued core run and bounds");
        }
    }

    #[test]
    fn verified_molecule_label_font_outline_support_matches_cardinal_core_bounds() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        for element in ["C", "O", "Cl", "I"] {
            let facts =
                AtomLabelFacts::new(element, None, 0, 0).expect("test atom facts are admitted");
            let layout = metrics
                .layout_atom_label(&facts, &font())
                .expect("verified Atkinson Hyperlegible Next lays out test label");
            let core = layout.attachment().core_element_ink_bounds();
            let support = layout.core_outline_support();
            let cases = [
                (Vector2::new(1.0, 0.0).expect("direction"), core.max_x()),
                (Vector2::new(-1.0, 0.0).expect("direction"), -core.min_x()),
                (Vector2::new(0.0, 1.0).expect("direction"), core.max_y()),
                (Vector2::new(0.0, -1.0).expect("direction"), -core.min_y()),
            ];
            for (direction, expected) in cases {
                let actual = support.directional_extent(direction);
                assert!(
                    (actual - expected).abs() < 1.0e-12,
                    "{element}: {actual} != {expected}"
                );
            }
        }
    }

    #[test]
    fn sole_rightward_bond_places_explicit_hydrogen_left_of_the_core() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        let facts = AtomLabelFacts::new("C", Some(13), 1, 3).expect("decorated carbon facts");
        let canonical = metrics
            .layout_atom_label(&facts, &font())
            .expect("canonical decorated layout");
        let unchanged = canonical
            .clone()
            .avoid_rightward_bond_with_explicit_hydrogen(
                Vector2::new(-1.0, 0.0).expect("leftward direction"),
                size(0.6),
                size(12.0),
                &metrics,
            )
            .expect("leftward bond keeps canonical layout");
        assert_eq!(unchanged, canonical);

        let placed = canonical
            .clone()
            .avoid_rightward_bond_with_explicit_hydrogen(
                Vector2::new(1.0, 0.0).expect("rightward direction"),
                size(0.6),
                size(12.0),
                &metrics,
            )
            .expect("rightward bond relocates decorations");
        assert_eq!(placed.runs()[0].origin(), canonical.runs()[0].origin());
        assert_eq!(placed.runs()[1].origin(), canonical.runs()[1].origin());
        assert!(placed.runs()[2].origin().x() < placed.runs()[1].origin().x());
        assert!(placed.runs()[3].origin().x() < placed.runs()[1].origin().x());
        assert!(placed.runs()[4].origin().x() < canonical.runs()[4].origin().x());
        assert_eq!(
            placed.core_outline_support(),
            canonical.core_outline_support()
        );
        let text = TextOp::new(
            RenderPoint::new(0.0, 0.0).expect("test origin"),
            placed.runs().to_vec(),
            FontFace::molecule_label(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
            30,
        )
        .expect("relocated label text");
        assert_eq!(
            placed.bounds(),
            metrics
                .atom_label_ink_bounds(&text, placed.core_element_run_index() as usize)
                .expect("relocated runs retain exact Atkinson Hyperlegible Next bounds")
        );
    }

    #[test]
    fn attachment_rejects_tiny_forged_nonzero_core_center() {
        let forged = GlyphBounds::new(-1.0, -1.0, 1.0 + 1.0e-15, 1.0)
            .expect("forged bounds are geometrically nonempty");
        assert!(AtomLabelAttachmentGeometry::new(forged).is_err());
    }
    #[test]
    fn durable_label_rejects_reordered_baseline_run_as_the_structural_core() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        let facts = AtomLabelFacts::new("N", None, 1, 3).expect("test atom facts are admitted");
        let layout = metrics
            .layout_atom_label(&facts, &font())
            .expect("verified Atkinson Hyperlegible Next lays out ammonium");
        let mut reordered_runs = layout.runs().to_vec();
        reordered_runs.swap(0, 1);
        assert_eq!(reordered_runs[0].script(), TextScript::Baseline);
        assert_eq!(reordered_runs[1].script(), TextScript::Baseline);
        let text = TextOp::new(
            RenderPoint::new(0.0, 0.0).expect("test origin is finite"),
            reordered_runs,
            FontFace::molecule_label(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
            30,
        )
        .expect("reordered runs retain exact glyph placements");
        let core_index = layout.core_element_run_index();
        let full = InkBoundsV1::from_glyph_bounds(
            metrics
                .text_ink_bounds(&text)
                .expect("reordered text still has exact Atkinson Hyperlegible Next ink"),
        );
        let original_core =
            InkBoundsV1::from_glyph_bounds(layout.attachment().core_element_ink_bounds());
        assert!(
            AtomLabelRenderV1::new(None, text, core_index, size(1.5), full, original_core).is_err()
        );
    }
}
