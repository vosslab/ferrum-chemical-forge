//! Physical verification of the selected molecule-label font contract.

use std::sync::OnceLock;

use ferrum_render_contract::{
    MOLECULE_LABEL_RESOURCE_ID, MOLECULE_LABEL_SHA256, MoleculeLabelScalarCapability,
    classify_molecule_label_scalar,
};
use ttf_parser::{Face, GlyphId};

use crate::{FontAssetDescriptor, RenderError};

pub(crate) fn verify_molecule_label_contract(
    descriptor: &FontAssetDescriptor,
    face: &Face<'_>,
) -> Result<(), RenderError> {
    if descriptor.resource_id() != MOLECULE_LABEL_RESOURCE_ID
        || descriptor.sha256() != MOLECULE_LABEL_SHA256
    {
        return Err(RenderError::InvalidRequest(
            "verified Atkinson Hyperlegible Next asset does not match the shared admission resource"
                .to_owned(),
        ));
    }
    static VERIFIED: OnceLock<Result<(), String>> = OnceLock::new();
    VERIFIED
        .get_or_init(|| verify_molecule_label_scalar_table(face))
        .as_ref()
        .map_err(|detail| {
            RenderError::InvalidRequest(format!(
                "Atkinson Hyperlegible Next admission contract mismatch: {detail}"
            ))
        })
        .copied()
}

fn verify_molecule_label_scalar_table(face: &Face<'_>) -> Result<(), String> {
    for value in 0_u32..=char::MAX as u32 {
        let Some(scalar) = char::from_u32(value) else {
            continue;
        };
        if scalar.is_control() {
            continue;
        }
        let physical = face
            .glyph_index(scalar)
            .filter(|glyph| glyph.0 != 0)
            .and_then(|glyph| {
                let advance = face.glyph_hor_advance(glyph)?;
                (advance > 0).then_some((glyph, face.glyph_bounding_box(glyph)))
            });
        let expected = classify_molecule_label_scalar(scalar);
        match (expected, physical) {
            (Some(MoleculeLabelScalarCapability::Outlined), Some((_, Some(_)))) => {}
            (Some(MoleculeLabelScalarCapability::WhitespaceAdvanceOnly), Some((_, None)))
                if scalar.is_whitespace() => {}
            (Some(MoleculeLabelScalarCapability::LineFeed), None) => {}
            (None, None) => {}
            (expected, physical) => {
                return Err(format!(
                    "scalar U+{value:04X} has contract {expected:?} but physical capability {}",
                    physical_capability_name(physical)
                ));
            }
        }
    }
    Ok(())
}

fn physical_capability_name(value: Option<(GlyphId, Option<ttf_parser::Rect>)>) -> &'static str {
    match value {
        Some((_, Some(_))) => "outlined",
        Some((_, None)) => "outline-less",
        None => "absent-or-nonadvancing",
    }
}
