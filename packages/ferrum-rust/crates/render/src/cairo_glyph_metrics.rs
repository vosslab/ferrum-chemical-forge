//! Cairo and FreeType measurements for the verified Telex resource.

use cairo::{FontFace, FontOptions, HintMetrics, HintStyle, Matrix, ScaledFont};
use freetype::Library;

use crate::glyph_metrics::LaidOutAtomLabel;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, FerrumFontEnvironmentV1, FerrumFontId, GlyphBounds,
    GlyphMetrics, PositiveFinite, RenderError, RenderPoint, TextRun, TextScript,
};

/// Exact-face Cairo metrics backed by a verified Telex FreeType face.
#[derive(Debug)]
pub struct CairoGlyphMetrics {
    font_face: FontFace,
}

impl CairoGlyphMetrics {
    /// Open the verified Telex file with FreeType and retain it through Cairo's safe face owner.
    pub fn new(environment: &FerrumFontEnvironmentV1) -> Result<Self, RenderError> {
        let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
        let library = Library::init().map_err(font_error)?;
        let face = library.new_face(descriptor.path(), 0).map_err(font_error)?;
        if face.family_name().as_deref() != Some("Telex")
            || face.postscript_name().as_deref() != Some("Telex-Regular")
        {
            return Err(RenderError::InvalidRequest(
                "verified Telex asset has unexpected face metadata".to_owned(),
            ));
        }
        let font_face = FontFace::create_from_ft(&face).map_err(font_error)?;
        Ok(Self { font_face })
    }

    fn scaled_font(&self, size: PositiveFinite) -> Result<ScaledFont, RenderError> {
        let mut options = FontOptions::new().map_err(font_error)?;
        options.set_hint_style(HintStyle::None);
        options.set_hint_metrics(HintMetrics::Off);
        ScaledFont::new(
            &self.font_face,
            &Matrix::new(size.get(), 0.0, 0.0, size.get(), 0.0, 0.0),
            &Matrix::identity(),
            &options,
        )
        .map_err(font_error)
    }
}

impl GlyphMetrics for CairoGlyphMetrics {
    fn layout_atom_label(
        &self,
        label: &AtomLabelFacts,
        font: &AtomLabelFontProfile,
    ) -> Result<LaidOutAtomLabel, RenderError> {
        if font.face().as_str() != FerrumFontId::TelexRegular.resource_id() {
            return Err(RenderError::InvalidRequest(
                "Cairo glyph metrics require ferrum-telex-regular-v1".to_owned(),
            ));
        }
        let metrics = self.scaled_font(font.size())?;
        let script_scale = PositiveFinite::new(0.65)?;
        let baseline = metrics.extents();
        let pieces = label.text_pieces();
        let mut widths = Vec::with_capacity(pieces.len());
        for (text, script) in &pieces {
            let scale = if *script == TextScript::Baseline {
                1.0
            } else {
                script_scale.get()
            };
            let width = self
                .scaled_font(PositiveFinite::new(font.size().get() * scale)?)?
                .text_extents(text)
                .x_advance();
            if !width.is_finite() || width <= 0.0 {
                return Err(RenderError::InvalidRequest(
                    "Telex glyph run has no positive finite advance".to_owned(),
                ));
            }
            widths.push(width);
        }
        let total_width: f64 = widths.iter().sum();
        let mut cursor = -total_width / 2.0;
        let mut runs = Vec::with_capacity(pieces.len());
        let mut min_x: f64 = 0.0;
        let mut min_y: f64 = 0.0;
        let mut max_x: f64 = 0.0;
        let mut max_y: f64 = 0.0;
        for ((text, script), width) in pieces.into_iter().zip(widths) {
            let scale = if script == TextScript::Baseline {
                1.0
            } else {
                script_scale.get()
            };
            let y = match script {
                TextScript::Baseline => 0.0,
                TextScript::Subscript => -baseline.descent() * 0.8,
                TextScript::Superscript => baseline.ascent() * 0.55,
            };
            let extents = self
                .scaled_font(PositiveFinite::new(font.size().get() * scale)?)?
                .text_extents(&text);
            min_x = min_x.min(cursor + extents.x_bearing());
            max_x = max_x.max(cursor + extents.x_bearing() + extents.width());
            min_y = min_y.min(y + extents.y_bearing());
            max_y = max_y.max(y + extents.y_bearing() + extents.height());
            let run_scale = PositiveFinite::new(scale)?;
            runs.push(TextRun::new(
                text,
                script,
                RenderPoint::new(cursor, y)?,
                run_scale,
            )?);
            cursor += width;
        }
        LaidOutAtomLabel::new(runs, GlyphBounds::new(min_x, min_y, max_x, max_y)?)
    }
}

fn font_error(error: impl std::fmt::Display) -> RenderError {
    RenderError::InvalidRequest(format!("verified Telex font operation failed: {error}"))
}
