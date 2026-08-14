//! Pure-Rust TrueType design metrics for Ferrum's closed Telex V1 resource.

use std::sync::Arc;

use ttf_parser::{Face, GlyphId};

use crate::glyph_metrics::LaidOutAtomLabel;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, CenteredTextLayout, FerrumFontEnvironmentV1,
    FerrumFontId, FontFace, GlyphBounds, GlyphMetrics, GlyphPlacement, Paint, PositiveFinite,
    PresentationGlyphRun, PresentationTextLayout, PresentationTextOp, PresentationTextSourceRun,
    RenderError, RenderPoint, TextOp, TextRun, TextScript,
};

/// Exact unhinted Telex extents for one fully specified text run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphRunMetrics {
    x_bearing: f64,
    y_bearing: f64,
    width: f64,
    height: f64,
    x_advance: f64,
    y_advance: f64,
}

/// Return whether an outline-less placement is Telex's exact non-drawing whitespace.
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
    scalar.is_whitespace() && face.glyph_index(scalar) == Some(GlyphId(glyph_id))
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

/// Exact unhinted Telex baseline metrics for one font size.
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

/// Pure-Rust exact metrics for the verified, unshaped Telex V1 face.
///
/// The verified resource has UPM 1000 and no kerning table.  V1 deliberately
/// permits only a scalar-to-one-glyph mapping; it does not promise shaping,
/// fallback, or font discovery.  Geometry is design metrics multiplied by the
/// requested size and run scale.  The V1 receipt fixes these semantics for the
/// closed Telex corpus.
#[derive(Debug)]
pub struct VerifiedTelexGlyphMetrics {
    data: Arc<[u8]>,
    units_per_em: f64,
}

impl VerifiedTelexGlyphMetrics {
    /// Open the immutable verified Telex bytes without reopening a filesystem path.
    pub fn new(environment: &FerrumFontEnvironmentV1) -> Result<Self, RenderError> {
        let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
        let face = open_face(descriptor.data())?;
        let units_per_em = f64::from(face.units_per_em());
        if !units_per_em.is_finite() || units_per_em <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "verified Telex asset has an invalid units-per-em value".to_owned(),
            ));
        }
        Ok(Self {
            data: Arc::clone(descriptor.data()),
            units_per_em,
        })
    }

    /// Measure one nonempty Telex run with V1's no-hinting, no-shaping contract.
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

    /// Measure V1 baseline facts from unhinted Telex design metrics.
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
                "Telex baseline metrics must be finite".to_owned(),
            ))
        }
    }

    /// Lay out the closed fixed-content plus glyph around an anchor-local origin.
    ///
    /// This is not a general rich-text API. V1 admits only `+` here, uses the
    /// verified Telex Regular face, and returns exact glyph IDs and ink bounds.
    pub fn layout_centered_plus(
        &self,
        size: PositiveFinite,
        paint: Paint,
    ) -> Result<CenteredTextLayout, RenderError> {
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
            FontFace::telex_regular(),
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
    /// both a Telex glyph and finite outline. The authored anchor is the logical
    /// top-left of the first line, matching the direct-root Text coordinate model.
    pub fn layout_presentation_text(
        &self,
        source_runs: &[PresentationTextSourceRun],
        size: PositiveFinite,
        paint: Paint,
    ) -> Result<PresentationTextLayout, RenderError> {
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
        let y = match script {
            TextScript::Baseline => baseline_y,
            TextScript::Subscript => baseline_y + context.baseline.descent() * 0.8,
            TextScript::Superscript => baseline_y - context.baseline.ascent() * 0.55,
        };
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

    /// Return closed V1 Telex scalar placements for a fully specified run.
    pub(crate) fn v1_glyphs_for_run(
        &self,
        text: &str,
        size: PositiveFinite,
        scale: PositiveFinite,
    ) -> Result<Vec<GlyphPlacement>, RenderError> {
        Ok(self.layout_unshaped_run(text, size, scale)?.glyphs)
    }

    /// Verify inbound V1 placements against the exact embedded Telex resource.
    pub(crate) fn validate_v1_run(
        &self,
        text: &str,
        script: TextScript,
        size: PositiveFinite,
        scale: PositiveFinite,
        glyphs: &[GlyphPlacement],
    ) -> Result<(), RenderError> {
        if !is_v1_text_run(text, script) {
            return Err(RenderError::InvalidRequest(
                "text run is outside the closed V1 atom-label or plus grammar".to_owned(),
            ));
        }
        if glyphs != self.v1_glyphs_for_run(text, size, scale)? {
            return Err(RenderError::InvalidRequest(
                "text run glyph IDs or origins do not match verified Telex layout".to_owned(),
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
                "Telex design metric must remain finite".to_owned(),
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
                "Telex glyph run must contain visible text and no control characters".to_owned(),
            ));
        }
        let mut cursor = 0.0;
        let mut glyphs = Vec::with_capacity(text.chars().count());
        // These are the true outline extents.  They deliberately do not include
        // the run origin: callers that require an anchor-containing clipping
        // envelope construct that separate fact at their own boundary.
        let mut ink_bounds: Option<(f64, f64, f64, f64)> = None;
        for scalar in text.chars() {
            let face = self.face()?;
            let glyph_id = face
                .glyph_index(scalar)
                .filter(|id| id.0 != 0)
                .ok_or_else(|| {
                    RenderError::InvalidRequest(
                        "Telex text contains an unsupported or missing glyph".to_owned(),
                    )
                })?;
            let bounds = face.glyph_bounding_box(glyph_id).ok_or_else(|| {
                RenderError::InvalidRequest(
                    "Telex text contains a glyph without a finite outline".to_owned(),
                )
            })?;
            let advance = face.glyph_hor_advance(glyph_id).ok_or_else(|| {
                RenderError::InvalidRequest(
                    "Telex text contains a glyph without a horizontal advance".to_owned(),
                )
            })?;
            let origin = RenderPoint::new(cursor, 0.0)?;
            let left = origin.x() + self.design_to_scene(i64::from(bounds.x_min), size, scale)?;
            let top = origin.y() - self.design_to_scene(i64::from(bounds.y_max), size, scale)?;
            let right =
                left + self.design_to_scene(i64::from(bounds.x_max - bounds.x_min), size, scale)?;
            let bottom =
                top + self.design_to_scene(i64::from(bounds.y_max - bounds.y_min), size, scale)?;
            let advance = self.design_to_scene(i64::from(advance), size, scale)?;
            if ![left, top, right, bottom, advance]
                .into_iter()
                .all(f64::is_finite)
            {
                return Err(RenderError::InvalidRequest(
                    "Telex glyph positions must be finite".to_owned(),
                ));
            }
            ink_bounds = Some(match ink_bounds {
                Some((min_x, min_y, max_x, max_y)) => (
                    min_x.min(left),
                    min_y.min(top),
                    max_x.max(right),
                    max_y.max(bottom),
                ),
                None => (left, top, right, bottom),
            });
            glyphs.push(GlyphPlacement::new(u32::from(glyph_id.0), origin)?);
            cursor += advance;
        }
        let Some((min_x, min_y, max_x, max_y)) = ink_bounds else {
            return Err(RenderError::InvalidRequest(
                "Telex glyph run has no finite outline".to_owned(),
            ));
        };
        if !cursor.is_finite() || cursor <= 0.0 || min_x >= max_x || min_y >= max_y {
            return Err(RenderError::InvalidRequest(
                "Telex glyph run has no finite positive unshaped extent".to_owned(),
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
                "presentation Telex segment must be nonempty and single-line".to_owned(),
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
                        "presentation Text contains an unsupported Telex glyph".to_owned(),
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

fn is_v1_text_run(text: &str, script: TextScript) -> bool {
    match script {
        TextScript::Baseline => {
            if text == "+" {
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

impl GlyphMetrics for VerifiedTelexGlyphMetrics {
    fn layout_atom_label(
        &self,
        label: &AtomLabelFacts,
        font: &AtomLabelFontProfile,
    ) -> Result<LaidOutAtomLabel, RenderError> {
        if font.face().as_str() != FerrumFontId::TelexRegular.resource_id() {
            return Err(RenderError::InvalidRequest(
                "verified Telex glyph metrics require ferrum-telex-regular-v1".to_owned(),
            ));
        }
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
        let total_width: f64 = layouts.iter().map(|layout| layout.advance).sum();
        let mut cursor = -total_width / 2.0;
        let mut runs = Vec::with_capacity(pieces.len());
        let mut ink_bounds: Option<(f64, f64, f64, f64)> = None;
        for ((text, script), layout) in pieces.into_iter().zip(layouts) {
            let scale = if script == TextScript::Baseline {
                PositiveFinite::new(1.0)?
            } else {
                script_scale
            };
            let y = match script {
                TextScript::Baseline => 0.0,
                TextScript::Subscript => -baseline.descent() * 0.8,
                TextScript::Superscript => baseline.ascent() * 0.55,
            };
            ink_bounds = Some(match ink_bounds {
                Some((min_x, min_y, max_x, max_y)) => (
                    min_x.min(cursor + layout.min_x),
                    min_y.min(y + layout.min_y),
                    max_x.max(cursor + layout.max_x),
                    max_y.max(y + layout.max_y),
                ),
                None => (
                    cursor + layout.min_x,
                    y + layout.min_y,
                    cursor + layout.max_x,
                    y + layout.max_y,
                ),
            });
            runs.push(TextRun::new(
                text,
                script,
                RenderPoint::new(cursor, y)?,
                layout.glyphs,
                scale,
            )?);
            cursor += layout.advance;
        }
        let Some((min_x, min_y, max_x, max_y)) = ink_bounds else {
            return Err(RenderError::InvalidRequest(
                "atom label has no finite Telex ink".to_owned(),
            ));
        };
        // Bond clipping begins at the durable atom anchor, so this particular
        // boundary retains the origin-containing envelope required by
        // `GlyphBounds`.  It is intentionally distinct from `GlyphRunMetrics`.
        LaidOutAtomLabel::new(
            runs,
            GlyphBounds::new(
                min_x.min(0.0),
                min_y.min(0.0),
                max_x.max(0.0),
                max_y.max(0.0),
            )?,
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
        if font.face().as_str() != FerrumFontId::TelexRegular.resource_id() {
            return Err(RenderError::InvalidRequest(
                "verified Telex glyph metrics require ferrum-telex-regular-v1".to_owned(),
            ));
        }
        let text = number.to_string();
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

/// Exact unshaped V1 glyph placements and ink bounds for one semantic run.
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
        RenderError::InvalidRequest(format!("could not parse verified Telex bytes: {error}"))
    })
}
