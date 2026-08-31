//! Pure-Rust TrueType design metrics for Ferrum's closed Atkinson Hyperlegible Next resource.

use std::sync::Arc;

use ferrum_document_model::CompactGroupCatalogKeyV1;
use ferrum_render_contract::{
    MOLECULE_LABEL_RESOURCE_ID, MoleculeLabelScalarCapability, classify_molecule_label_scalar,
    validate_molecule_label_text_segments,
};
use ttf_parser::{Face, GlyphId};

use crate::glyph_metrics::{
    AtomLabelAttachmentGeometry, GlyphBounds, GlyphMetrics, LaidOutAtomLabel,
};
use crate::glyph_outline_support::{
    molecule_label_glyph_outline_support, molecule_label_run_outline_support,
};
use crate::molecule_label_font_verification::verify_molecule_label_contract;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, CenteredTextLayout, FerrumFontEnvironment, FontFace,
    GlyphPlacement, PositiveFinite, PresentationGlyphRun, PresentationTextLayout,
    PresentationTextOp, PresentationTextSourceRun, RenderError, RenderPaintV3, RenderPoint, TextOp,
    TextRun, TextScript,
};
/// Exact unhinted Atkinson Hyperlegible Next extents for one fully specified text run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphRunMetrics {
    x_bearing: f64,
    y_bearing: f64,
    width: f64,
    height: f64,
    x_advance: f64,
    y_advance: f64,
}
/// Return whether an outline-less placement is the exact supported non-drawing whitespace.
///
/// Presentation layout retains the advance for supported whitespace, but a missing
/// outline is safe to omit only when this exact scalar maps back to the supplied
/// glyph ID. Newlines never reach a presentation glyph run, controls are rejected
/// during layout, and visible scalars must continue to have usable outlines.
pub(crate) fn is_verified_outlineless_whitespace_glyph(
    face: &Face<'_>,
    scalar: char,
    glyph_index: u32,
) -> bool {
    let Ok(glyph_id) = u16::try_from(glyph_index) else {
        return false;
    };
    matches!(
        classify_molecule_label_scalar(scalar),
        Some(MoleculeLabelScalarCapability::WhitespaceAdvanceOnly)
    ) && face.glyph_index(scalar) == Some(GlyphId(glyph_id))
}
impl GlyphRunMetrics {
    /// Return the horizontal bearing from the text origin.
    #[must_use]
    pub const fn x_bearing(self) -> f64 {
        self.x_bearing
    }
    /// Return the vertical bearing from the text origin.
    #[must_use]
    pub const fn y_bearing(self) -> f64 {
        self.y_bearing
    }
    /// Return the exact ink width.
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }
    /// Return the exact ink height.
    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
    /// Return the horizontal pen advance.
    #[must_use]
    pub const fn x_advance(self) -> f64 {
        self.x_advance
    }
    /// Return the vertical pen advance.
    #[must_use]
    pub const fn y_advance(self) -> f64 {
        self.y_advance
    }
}
/// Exact unhinted Atkinson Hyperlegible Next baseline metrics for one font size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontBaselineMetrics {
    ascent: f64,
    descent: f64,
    height: f64,
}
impl FontBaselineMetrics {
    /// Return the exact ascent above the baseline.
    #[must_use]
    pub const fn ascent(self) -> f64 {
        self.ascent
    }
    /// Return the exact descent below the baseline.
    #[must_use]
    pub const fn descent(self) -> f64 {
        self.descent
    }
    /// Return the exact font-line height.
    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
}
/// Pure-Rust exact metrics for the verified, unshaped Atkinson Hyperlegible Next face.
///
/// The verified resource has UPM 1000 and no kerning table. The closed contract
/// permits only a scalar-to-one-glyph mapping; it does not promise shaping,
/// fallback, or font discovery.  Geometry is design metrics multiplied by the
/// requested size and run scale.
#[derive(Debug)]
pub struct VerifiedMoleculeLabelGlyphMetrics {
    data: Arc<[u8]>,
    units_per_em: f64,
}
impl VerifiedMoleculeLabelGlyphMetrics {
    /// Measure exact visible ink for an already-validated V4 text operation.
    pub(crate) fn text_ink_bounds(&self, text: &TextOp) -> Result<GlyphBounds, RenderError> {
        let mut bounds: Option<GlyphBounds> = None;
        for run in text.runs() {
            let run_bounds = self.run_ink_bounds(text, run)?;
            bounds = Some(match bounds {
                Some(existing) => GlyphBounds::new(
                    existing.min_x().min(run_bounds.min_x()),
                    existing.min_y().min(run_bounds.min_y()),
                    existing.max_x().max(run_bounds.max_x()),
                    existing.max_y().max(run_bounds.max_y()),
                )?,
                None => run_bounds,
            });
        }
        bounds.ok_or_else(|| {
            RenderError::InvalidRequest(
                "text operation has no visible Atkinson Hyperlegible Next ink".to_owned(),
            )
        })
    }
    /// Measure a label's exact canonical ink envelope.
    ///
    /// The structurally selected core run uses the same symmetric float
    /// normalization as its durable attachment rectangle. Other runs retain
    /// their exact Atkinson Hyperlegible Next outline bounds.
    pub(crate) fn atom_label_ink_bounds(
        &self,
        text: &TextOp,
        core_element_run_index: usize,
    ) -> Result<GlyphBounds, RenderError> {
        let full = self.text_ink_bounds(text)?;
        let core = self.centered_core_run_ink_bounds(text, core_element_run_index)?;
        GlyphBounds::new(
            full.min_x().min(core.min_x()),
            full.min_y().min(core.min_y()),
            full.max_x().max(core.max_x()),
            full.max_y().max(core.max_y()),
        )
    }
    /// Verify and canonicalize the source-issued structural atom-label run.
    ///
    /// Atom layout places this one run at the negated mathematical center of
    /// its Atkinson Hyperlegible Next outline. The exact origin equality proves that a tiny wire
    /// translation cannot be hidden by canonical symmetric bounds.
    pub(crate) fn centered_core_run_ink_bounds(
        &self,
        text: &TextOp,
        index: usize,
    ) -> Result<GlyphBounds, RenderError> {
        let run = text.runs().get(index).ok_or_else(|| {
            RenderError::InvalidRequest(
                "atom-label core run index is outside label text".to_owned(),
            )
        })?;
        let layout = self.layout_unshaped_run(run.text(), text.size(), run.scale())?;
        self.validate_run(
            run.text(),
            run.script(),
            text.size(),
            run.scale(),
            run.glyphs(),
        )?;
        let expected_x = -((layout.min_x + layout.max_x) / 2.0);
        let expected_y = -((layout.min_y + layout.max_y) / 2.0);
        let absolute_x = text.origin().x() + run.origin().x();
        let absolute_y = text.origin().y() + run.origin().y();
        if absolute_x != expected_x || absolute_y != expected_y {
            return Err(RenderError::InvalidRequest(
                "atom-label core run must use the exact Atkinson Hyperlegible Next-centered origin"
                    .to_owned(),
            ));
        }
        self.run_ink_bounds(text, run)?
            .canonical_centered_at_origin()
    }
    fn run_ink_bounds(&self, text: &TextOp, run: &TextRun) -> Result<GlyphBounds, RenderError> {
        self.run_ink_bounds_at(text.origin(), text.size(), run)
    }
    /// Measure one positioned run without constructing a complete text operation.
    pub(crate) fn run_ink_bounds_at(
        &self,
        text_origin: RenderPoint,
        size: PositiveFinite,
        run: &TextRun,
    ) -> Result<GlyphBounds, RenderError> {
        self.validate_run(run.text(), run.script(), size, run.scale(), run.glyphs())?;
        let layout = self.layout_unshaped_run(run.text(), size, run.scale())?;
        GlyphBounds::new(
            text_origin.x() + run.origin().x() + layout.min_x,
            text_origin.y() + run.origin().y() + layout.min_y,
            text_origin.x() + run.origin().x() + layout.max_x,
            text_origin.y() + run.origin().y() + layout.max_y,
        )
    }
    /// Open immutable verified font bytes without reopening a filesystem path.
    pub fn new(environment: &FerrumFontEnvironment) -> Result<Self, RenderError> {
        let descriptor = environment.molecule_label();
        let face = open_face(descriptor.data())?;
        verify_molecule_label_contract(descriptor, &face)?;
        let units_per_em = f64::from(face.units_per_em());
        if !units_per_em.is_finite() || units_per_em <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "verified Atkinson Hyperlegible Next asset has an invalid units-per-em value"
                    .to_owned(),
            ));
        }
        Ok(Self {
            data: Arc::clone(descriptor.data()),
            units_per_em,
        })
    }
    /// Measure one nonempty run with the closed no-hinting, no-shaping contract.
    pub fn measure_text_run(
        &self,
        text: &str,
        size: PositiveFinite,
        scale: PositiveFinite,
    ) -> Result<GlyphRunMetrics, RenderError> {
        let layout = self.layout_unshaped_run(text, size, scale)?;
        Ok(GlyphRunMetrics {
            x_bearing: layout.min_x,
            y_bearing: layout.min_y,
            width: layout.max_x - layout.min_x,
            height: layout.max_y - layout.min_y,
            x_advance: layout.advance,
            y_advance: 0.0,
        })
    }
    /// Measure baseline facts from unhinted Atkinson Hyperlegible Next design metrics.
    pub fn baseline_metrics(
        &self,
        size: PositiveFinite,
    ) -> Result<FontBaselineMetrics, RenderError> {
        let metrics = FontBaselineMetrics {
            ascent: self.design_to_scene(
                i64::from(self.face()?.ascender()),
                size,
                PositiveFinite::new(1.0)?,
            )?,
            descent: -self.design_to_scene(
                i64::from(self.face()?.descender()),
                size,
                PositiveFinite::new(1.0)?,
            )?,
            height: self.design_to_scene(
                i64::from(self.face()?.height()),
                size,
                PositiveFinite::new(1.0)?,
            )?,
        };
        if [metrics.ascent, metrics.descent, metrics.height]
            .into_iter()
            .all(f64::is_finite)
        {
            Ok(metrics)
        } else {
            Err(RenderError::InvalidRequest(
                "Atkinson Hyperlegible Next baseline metrics must be finite".to_owned(),
            ))
        }
    }
    /// Lay out the closed fixed-content plus glyph around an anchor-local origin.
    ///
    /// This is not a general rich-text API. The contract admits only `+` here, uses the
    /// verified Atkinson Hyperlegible Next Regular face, and returns exact glyph IDs
    /// and ink bounds.
    pub fn layout_centered_plus(
        &self,
        size: PositiveFinite,
        paint: RenderPaintV3,
    ) -> Result<CenteredTextLayout, RenderError> {
        validate_molecule_label_text_segments(["+"]).map_err(molecule_label_admission_error)?;
        let scale = PositiveFinite::new(1.0)?;
        let layout = self.layout_unshaped_run("+", size, scale)?;
        let center_x = (layout.min_x + layout.max_x) / 2.0;
        let center_y = (layout.min_y + layout.max_y) / 2.0;
        let origin = RenderPoint::new(-center_x, -center_y)?;
        let run = TextRun::new(
            "+",
            TextScript::Baseline,
            RenderPoint::new(0.0, 0.0)?,
            layout.glyphs,
            scale,
        )?;
        let operation = TextOp::new(
            origin,
            vec![run],
            FontFace::molecule_label(),
            size,
            paint,
            20,
        )?;
        let bounds = GlyphBounds::new(
            layout.min_x - center_x,
            layout.min_y - center_y,
            layout.max_x - center_x,
            layout.max_y - center_y,
        )?;
        Ok(CenteredTextLayout::new(operation, bounds))
    }
    /// Lay out one closed compact-group label around an anchor-local origin.
    ///
    /// The accepted labels are derived solely from the document compact-group
    /// catalog. This is not a general rich-text entry point.
    pub(crate) fn layout_centered_compact_group_label(
        &self,
        catalog_key: CompactGroupCatalogKeyV1,
        size: PositiveFinite,
        paint: RenderPaintV3,
    ) -> Result<CenteredTextLayout, RenderError> {
        let label = catalog_key.label();
        let scale = PositiveFinite::new(1.0)?;
        let layout = self.layout_unshaped_run(label, size, scale)?;
        let center_x = (layout.min_x + layout.max_x) / 2.0;
        let center_y = (layout.min_y + layout.max_y) / 2.0;
        let origin = RenderPoint::new(-center_x, -center_y)?;
        let run = TextRun::new(
            label,
            TextScript::Baseline,
            RenderPoint::new(0.0, 0.0)?,
            layout.glyphs,
            scale,
        )?;
        let operation = TextOp::new(
            origin,
            vec![run],
            FontFace::molecule_label(),
            size,
            paint,
            20,
        )?;
        let bounds = GlyphBounds::new(
            layout.min_x - center_x,
            layout.min_y - center_y,
            layout.max_x - center_x,
            layout.max_y - center_y,
        )?;
        Ok(CenteredTextLayout::new(operation, bounds))
    }

    /// Lay out free-form direct-root Text with explicit newlines and script roles.
    ///
    /// This remains an unshaped scalar-to-glyph contract. Space glyphs may have no
    /// ink outline but retain their verified advance. Every other scalar must have
    /// both an Atkinson Hyperlegible Next glyph and finite outline. The authored
    /// anchor is the logical top-left of the first line, matching the direct-root
    /// Text coordinate model.
    pub fn layout_presentation_text(
        &self,
        source_runs: &[PresentationTextSourceRun],
        size: PositiveFinite,
        paint: RenderPaintV3,
    ) -> Result<PresentationTextLayout, RenderError> {
        validate_molecule_label_text_segments(
            source_runs.iter().map(PresentationTextSourceRun::text),
        )
        .map_err(molecule_label_admission_error)?;
        if source_runs.is_empty()
            || !source_runs
                .iter()
                .flat_map(|run| run.text().chars())
                .any(|character| !character.is_whitespace())
        {
            return Err(RenderError::InvalidRequest(
                "presentation Text must contain a visible character".to_owned(),
            ));
        }
        let baseline = self.baseline_metrics(size)?;
        let context = PresentationLayoutContext {
            size,
            baseline,
            baseline_scale: PositiveFinite::new(1.0)?,
            script_scale: PositiveFinite::new(0.65)?,
        };
        let mut line = 0_u64;
        let mut layout = PresentationLayoutAccumulator::default();
        for source in source_runs {
            let mut segment = String::new();
            for character in source.text().chars() {
                if character == '\n' {
                    self.append_presentation_segment(
                        &mut layout,
                        line,
                        &segment,
                        source.script(),
                        context,
                    )?;
                    segment.clear();
                    layout.finish_line();
                    line = line.checked_add(1).ok_or_else(|| {
                        RenderError::InvalidRequest(
                            "presentation Text has too many logical lines".to_owned(),
                        )
                    })?;
                } else {
                    segment.push(character);
                }
            }
            self.append_presentation_segment(
                &mut layout,
                line,
                &segment,
                source.script(),
                context,
            )?;
        }
        layout.complete_last_line();
        let line_count = line.checked_add(1).ok_or_else(|| {
            RenderError::InvalidRequest("presentation Text line count overflowed".to_owned())
        })?;
        let logical_bottom = line_count as f64 * baseline.height();
        let bounds = GlyphBounds::new(
            layout.min_ink_x.min(0.0),
            layout.min_ink_y.min(0.0),
            layout.max_ink_x.max(layout.max_line_x),
            layout.max_ink_y.max(logical_bottom),
        )?;
        Ok(PresentationTextLayout::new(
            PresentationTextOp::new(layout.runs, size, paint, 20)?,
            bounds,
        ))
    }
    fn append_presentation_segment(
        &self,
        layout: &mut PresentationLayoutAccumulator,
        line: u64,
        text: &str,
        script: TextScript,
        context: PresentationLayoutContext,
    ) -> Result<(), RenderError> {
        if text.is_empty() {
            return Ok(());
        }
        let scale = if script == TextScript::Baseline {
            context.baseline_scale
        } else {
            context.script_scale
        };
        let segment = self.layout_presentation_run(text, context.size, scale)?;
        let line_y = line as f64 * context.baseline.height();
        let baseline_y = line_y + context.baseline.ascent();
        let y = script_baseline_y(script, baseline_y, context.baseline);
        if let Some((left, top, right, bottom)) = segment.ink_bounds {
            layout.min_ink_x = layout.min_ink_x.min(layout.cursor_x + left);
            layout.min_ink_y = layout.min_ink_y.min(y + top);
            layout.max_ink_x = layout.max_ink_x.max(layout.cursor_x + right);
            layout.max_ink_y = layout.max_ink_y.max(y + bottom);
        }
        layout.runs.push(PresentationGlyphRun::new(
            text.to_owned(),
            script,
            RenderPoint::new(layout.cursor_x, y)?,
            segment.glyphs,
            scale,
        )?);
        layout.cursor_x += segment.advance;
        if !layout.cursor_x.is_finite() {
            return Err(RenderError::InvalidRequest(
                "presentation Text advance must remain finite".to_owned(),
            ));
        }
        Ok(())
    }
    /// Return closed Atkinson Hyperlegible Next scalar placements for a fully specified run.
    pub(crate) fn glyphs_for_run(
        &self,
        text: &str,
        size: PositiveFinite,
        scale: PositiveFinite,
    ) -> Result<Vec<GlyphPlacement>, RenderError> {
        Ok(self.layout_unshaped_run(text, size, scale)?.glyphs)
    }
    /// Verify inbound placements against the exact embedded Atkinson Hyperlegible Next resource.
    pub(crate) fn validate_run(
        &self,
        text: &str,
        script: TextScript,
        size: PositiveFinite,
        scale: PositiveFinite,
        glyphs: &[GlyphPlacement],
    ) -> Result<(), RenderError> {
        if !is_admitted_text_run(text, script) {
            return Err(RenderError::InvalidRequest(
                "text run is outside the closed atom-label, compact-group, or plus grammar"
                    .to_owned(),
            ));
        }
        if glyphs != self.glyphs_for_run(text, size, scale)? {
            return Err(RenderError::InvalidRequest(
                "text run glyphs do not match verified molecule-label layout".to_owned(),
            ));
        }
        Ok(())
    }
    fn design_to_scene(
        &self,
        design_units: i64,
        size: PositiveFinite,
        scale: PositiveFinite,
    ) -> Result<f64, RenderError> {
        let value = design_units as f64 / self.units_per_em * size.get() * scale.get();
        if value.is_finite() {
            Ok(value)
        } else {
            Err(RenderError::InvalidRequest(
                "Atkinson Hyperlegible Next design metric must remain finite".to_owned(),
            ))
        }
    }
    fn face(&self) -> Result<Face<'_>, RenderError> {
        // `new` verifies this immutable embedded byte resource once. Parsing it here
        // keeps the parser borrow tied directly to the retained `Arc` without a
        // self-referential font object.
        open_face(&self.data)
    }

    fn layout_unshaped_run(
        &self,
        text: &str,
        size: PositiveFinite,
        scale: PositiveFinite,
    ) -> Result<UnshapedRun, RenderError> {
        if text.trim().is_empty() || text.chars().any(char::is_control) {
            return Err(RenderError::InvalidRequest(
                "molecule-label glyph run requires visible text without controls".to_owned(),
            ));
        }
        let mut cursor = 0.0;
        let mut glyphs = Vec::with_capacity(text.chars().count());
        let face = self.face()?;
        for scalar in text.chars() {
            let glyph_id = face
                .glyph_index(scalar)
                .filter(|id| id.0 != 0)
                .ok_or_else(|| {
                    RenderError::InvalidRequest(
                        "Atkinson Hyperlegible Next text contains an unsupported or missing glyph"
                            .to_owned(),
                    )
                })?;
            let advance = face.glyph_hor_advance(glyph_id).ok_or_else(|| {
                RenderError::InvalidRequest(
                    "Atkinson Hyperlegible Next text contains a glyph without a horizontal advance"
                        .to_owned(),
                )
            })?;
            let origin = RenderPoint::new(cursor, 0.0)?;
            let advance = self.design_to_scene(i64::from(advance), size, scale)?;
            if !advance.is_finite() {
                return Err(RenderError::InvalidRequest(
                    "Atkinson Hyperlegible Next glyph positions must be finite".to_owned(),
                ));
            }
            glyphs.push(GlyphPlacement::new(u32::from(glyph_id.0), origin)?);
            cursor += advance;
        }
        let outline_bounds = molecule_label_glyph_outline_support(
            &face,
            RenderPoint::new(0.0, 0.0)?,
            &glyphs,
            size,
            scale,
        )?
        .axis_aligned_bounds()?;
        let (min_x, min_y, max_x, max_y) = (
            outline_bounds.min_x(),
            outline_bounds.min_y(),
            outline_bounds.max_x(),
            outline_bounds.max_y(),
        );
        if !cursor.is_finite() || cursor <= 0.0 || min_x >= max_x || min_y >= max_y {
            return Err(RenderError::InvalidRequest(
                "Atkinson Hyperlegible Next glyph run has no finite positive unshaped extent"
                    .to_owned(),
            ));
        }
        Ok(UnshapedRun {
            glyphs,
            advance: cursor,
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    fn layout_presentation_run(
        &self,
        text: &str,
        size: PositiveFinite,
        scale: PositiveFinite,
    ) -> Result<PresentationRunLayout, RenderError> {
        if text.is_empty() || text.chars().any(char::is_control) {
            return Err(RenderError::InvalidRequest(
                "presentation Atkinson Hyperlegible Next segment must be nonempty and single-line"
                    .to_owned(),
            ));
        }
        let face = self.face()?;
        let mut cursor = 0.0_f64;
        let mut ink_bounds: Option<(f64, f64, f64, f64)> = None;
        let mut glyphs = Vec::with_capacity(text.chars().count());
        for scalar in text.chars() {
            let glyph_id = face
                .glyph_index(scalar)
                .filter(|id| id.0 != 0)
                .ok_or_else(|| {
                    RenderError::InvalidRequest(
                        "presentation Text contains an unsupported molecule-label glyph".to_owned(),
                    )
                })?;
            if let Some(bounds) = face.glyph_bounding_box(glyph_id) {
                let left = cursor + self.design_to_scene(i64::from(bounds.x_min), size, scale)?;
                let top = -self.design_to_scene(i64::from(bounds.y_max), size, scale)?;
                let right = cursor + self.design_to_scene(i64::from(bounds.x_max), size, scale)?;
                let bottom = -self.design_to_scene(i64::from(bounds.y_min), size, scale)?;
                ink_bounds = Some(match ink_bounds {
                    Some((min_x, min_y, max_x, max_y)) => (
                        min_x.min(left),
                        min_y.min(top),
                        max_x.max(right),
                        max_y.max(bottom),
                    ),
                    None => (left, top, right, bottom),
                });
            } else if !scalar.is_whitespace() {
                return Err(RenderError::InvalidRequest(
                    "presentation Text contains a visible glyph without an outline".to_owned(),
                ));
            }
            let advance = face.glyph_hor_advance(glyph_id).ok_or_else(|| {
                RenderError::InvalidRequest(
                    "presentation Text glyph has no horizontal advance".to_owned(),
                )
            })?;
            glyphs.push(GlyphPlacement::new(
                u32::from(glyph_id.0),
                RenderPoint::new(cursor, 0.0)?,
            )?);
            cursor += self.design_to_scene(i64::from(advance), size, scale)?;
        }
        if !cursor.is_finite() || cursor <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "presentation Text segment must have a positive finite advance".to_owned(),
            ));
        }
        Ok(PresentationRunLayout {
            glyphs,
            advance: cursor,
            ink_bounds,
        })
    }
}

fn is_admitted_text_run(text: &str, script: TextScript) -> bool {
    match script {
        TextScript::Baseline => {
            if text == "+" || CompactGroupCatalogKeyV1::from_label(text).is_some() {
                return true;
            }
            let mut scalars = text.chars();
            let Some(first) = scalars.next() else {
                return false;
            };
            (first.is_ascii_uppercase()
                && scalars.clone().count() <= 2
                && scalars.all(|scalar| scalar.is_ascii_lowercase()))
                || text
                    .parse::<u64>()
                    .is_ok_and(|number| number > 0 && text == number.to_string())
        }
        TextScript::Subscript => text
            .parse::<u8>()
            .is_ok_and(|count| count >= 2 && text == count.to_string()),
        TextScript::Superscript => {
            if text
                .parse::<u16>()
                .is_ok_and(|mass| (1..=32_767).contains(&mass) && text == mass.to_string())
            {
                return true;
            }
            let Some(sign) = text.chars().last() else {
                return false;
            };
            if sign != '+' && sign != '-' {
                return false;
            }
            let magnitude = &text[..text.len() - sign.len_utf8()];
            magnitude.is_empty()
                || magnitude
                    .parse::<u8>()
                    .is_ok_and(|count| (2..=128).contains(&count) && magnitude == count.to_string())
        }
    }
}

impl GlyphMetrics for VerifiedMoleculeLabelGlyphMetrics {
    fn layout_atom_label(
        &self,
        label: &AtomLabelFacts,
        font: &AtomLabelFontProfile,
    ) -> Result<LaidOutAtomLabel, RenderError> {
        if font.face().as_str() != MOLECULE_LABEL_RESOURCE_ID {
            return Err(RenderError::InvalidRequest(format!(
                "verified molecule-label metrics require {MOLECULE_LABEL_RESOURCE_ID}"
            )));
        }
        validate_molecule_label_text_segments(
            label.text_pieces().iter().map(|(text, _)| text.as_str()),
        )
        .map_err(molecule_label_admission_error)?;
        let script_scale = PositiveFinite::new(0.65)?;
        let baseline = self.baseline_metrics(font.size())?;
        let pieces = label.text_pieces();
        let mut layouts = Vec::with_capacity(pieces.len());
        for (text, script) in &pieces {
            let scale = if *script == TextScript::Baseline {
                PositiveFinite::new(1.0)?
            } else {
                script_scale
            };
            layouts.push(self.layout_unshaped_run(text, font.size(), scale)?);
        }
        // An optional isotope precedes the structural element. Preserve the
        // structural run identity explicitly rather than inferring it later.
        let core_element_run_index = u32::from(label.isotope_mass_number().is_some());
        let core = layouts
            .get(core_element_run_index as usize)
            .expect("atom labels always include the structural element run");
        let core_min_x = core.min_x;
        let core_max_x = core.max_x;
        let core_min_y = core.min_y;
        let core_max_y = core.max_y;
        let core_center_x = (core_min_x + core_max_x) / 2.0;
        let core_center_y = (core_min_y + core_max_y) / 2.0;
        let prefix_advance = layouts
            .iter()
            .take(core_element_run_index as usize)
            .map(|layout| layout.advance)
            .sum::<f64>();
        let mut cursor = -core_center_x - prefix_advance;
        let baseline_y = -core_center_y;
        let mut runs = Vec::with_capacity(pieces.len());
        let mut non_core_run_ink_bounds = Vec::new();
        let mut ink_bounds: Option<(f64, f64, f64, f64)> = None;
        for (index, ((text, script), layout)) in pieces.into_iter().zip(layouts).enumerate() {
            let scale = if script == TextScript::Baseline {
                PositiveFinite::new(1.0)?
            } else {
                script_scale
            };
            let x = if index == core_element_run_index as usize {
                -core_center_x
            } else {
                cursor
            };
            let y = script_baseline_y(script, baseline_y, baseline);
            let run_bounds = GlyphBounds::new(
                x + layout.min_x,
                y + layout.min_y,
                x + layout.max_x,
                y + layout.max_y,
            )?;
            if index != core_element_run_index as usize {
                non_core_run_ink_bounds.push(run_bounds);
            }
            ink_bounds = Some(match ink_bounds {
                Some((min_x, min_y, max_x, max_y)) => (
                    min_x.min(x + layout.min_x),
                    min_y.min(y + layout.min_y),
                    max_x.max(x + layout.max_x),
                    max_y.max(y + layout.max_y),
                ),
                None => (
                    x + layout.min_x,
                    y + layout.min_y,
                    x + layout.max_x,
                    y + layout.max_y,
                ),
            });
            runs.push(TextRun::new(
                text,
                script,
                RenderPoint::new(x, y)?,
                layout.glyphs,
                scale,
            )?);
            cursor = x + layout.advance;
        }
        let Some((min_x, min_y, max_x, max_y)) = ink_bounds else {
            return Err(RenderError::InvalidRequest(
                "atom label has no finite Atkinson Hyperlegible Next ink".to_owned(),
            ));
        };
        let core_bounds = GlyphBounds::new(
            -core_center_x + core_min_x,
            baseline_y + core_min_y,
            -core_center_x + core_max_x,
            baseline_y + core_max_y,
        )?
        .canonical_centered_at_origin()?;
        let bounds = GlyphBounds::new(
            min_x.min(core_bounds.min_x()),
            min_y.min(core_bounds.min_y()),
            max_x.max(core_bounds.max_x()),
            max_y.max(core_bounds.max_y()),
        )?;
        let core_outline_support = molecule_label_run_outline_support(
            &self.face()?,
            &runs[core_element_run_index as usize],
            font.size(),
        )?;
        LaidOutAtomLabel::new(
            runs,
            bounds,
            AtomLabelAttachmentGeometry::new(core_bounds)?,
            core_outline_support,
            core_element_run_index,
            non_core_run_ink_bounds,
        )
    }
    fn layout_atom_number(
        &self,
        number: u64,
        font: &AtomLabelFontProfile,
    ) -> Result<TextRun, RenderError> {
        if number == 0 {
            return Err(RenderError::InvalidRequest(
                "atom number must be a positive integer".to_owned(),
            ));
        }
        if font.face().as_str() != MOLECULE_LABEL_RESOURCE_ID {
            return Err(RenderError::InvalidRequest(format!(
                "verified molecule-label metrics require {MOLECULE_LABEL_RESOURCE_ID}"
            )));
        }
        let text = number.to_string();
        validate_molecule_label_text_segments([text.as_str()])
            .map_err(molecule_label_admission_error)?;
        let scale = PositiveFinite::new(1.0)?;
        let layout = self.layout_unshaped_run(&text, font.size(), scale)?;
        TextRun::new(
            text,
            TextScript::Baseline,
            RenderPoint::new(0.0, 0.0)?,
            layout.glyphs,
            scale,
        )
    }
}

/// Place scripts in the renderer's y-down coordinates for every Atkinson Hyperlegible Next caller.
fn script_baseline_y(script: TextScript, baseline_y: f64, baseline: FontBaselineMetrics) -> f64 {
    match script {
        TextScript::Baseline => baseline_y,
        TextScript::Subscript => baseline_y + baseline.descent() * 0.8,
        TextScript::Superscript => baseline_y - baseline.ascent() * 0.55,
    }
}

fn molecule_label_admission_error(
    error: ferrum_render_contract::MoleculeLabelTextExclusion,
) -> RenderError {
    RenderError::InvalidRequest(format!(
        "Atkinson Hyperlegible Next text admission rejected input: {error:?}"
    ))
}

struct UnshapedRun {
    glyphs: Vec<GlyphPlacement>,
    advance: f64,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

#[derive(Clone, Copy)]
struct PresentationLayoutContext {
    size: PositiveFinite,
    baseline: FontBaselineMetrics,
    baseline_scale: PositiveFinite,
    script_scale: PositiveFinite,
}

#[derive(Default)]
struct PresentationLayoutAccumulator {
    runs: Vec<PresentationGlyphRun>,
    cursor_x: f64,
    max_line_x: f64,
    min_ink_x: f64,
    min_ink_y: f64,
    max_ink_x: f64,
    max_ink_y: f64,
}

impl PresentationLayoutAccumulator {
    fn finish_line(&mut self) {
        self.complete_last_line();
        self.cursor_x = 0.0;
    }

    fn complete_last_line(&mut self) {
        self.max_line_x = self.max_line_x.max(self.cursor_x);
    }
}

struct PresentationRunLayout {
    glyphs: Vec<GlyphPlacement>,
    advance: f64,
    ink_bounds: Option<(f64, f64, f64, f64)>,
}

fn open_face(data: &[u8]) -> Result<Face<'_>, RenderError> {
    Face::parse(data, 0).map_err(|error| {
        RenderError::InvalidRequest(format!(
            "could not parse verified Atkinson Hyperlegible Next bytes: {error}"
        ))
    })
}
