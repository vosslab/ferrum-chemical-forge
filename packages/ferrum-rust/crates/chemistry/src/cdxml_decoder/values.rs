//! Scalar and closed-table validation for CDXML-C1.

use super::*;

pub(super) const MAX_IDENTIFIER_BYTES: usize = 128;
pub(super) const MAX_ATTRIBUTE_VALUE_BYTES: usize = 1_024;
pub(super) const MAX_COORDINATE: f64 = 100_000.0;

pub(super) fn has_entity_reference(value: &str) -> bool {
    value.contains('&')
}

pub(super) fn validate_id(value: &str) -> Result<()> {
    if value.is_empty() {
        return refused(CdxmlRefusalReasonV1::InvalidScalar);
    }
    if value.len() > MAX_IDENTIFIER_BYTES {
        return refused(CdxmlRefusalReasonV1::IdentifierBytesLimit);
    }
    if has_entity_reference(value) || value.contains('\0') {
        return refused(CdxmlRefusalReasonV1::EntityForbidden);
    }
    Ok(())
}

pub(super) fn decimal(value: &str) -> Result<f64> {
    if value.is_empty()
        || value.trim() != value
        || !value.as_bytes().iter().enumerate().all(|(index, byte)| {
            byte.is_ascii_digit() || *byte == b'.' || (index == 0 && matches!(*byte, b'+' | b'-'))
        })
    {
        return refused(CdxmlRefusalReasonV1::InvalidCoordinate);
    }
    let parsed = value.parse::<f64>().map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidCoordinate,
    })?;
    if !parsed.is_finite() {
        return refused(CdxmlRefusalReasonV1::CoordinateNotFinite);
    }
    if parsed.abs() > MAX_COORDINATE {
        return refused(CdxmlRefusalReasonV1::CoordinateOutOfRange);
    }
    Ok(parsed)
}

pub(super) fn coordinate_pair(value: &str) -> Result<(f64, f64)> {
    let fields = value.split_ascii_whitespace().collect::<Vec<_>>();
    if fields.len() != 2 {
        return refused(CdxmlRefusalReasonV1::InvalidCoordinate);
    }
    Ok((decimal(fields[0])?, decimal(fields[1])?))
}

pub(super) fn element_number(value: &str) -> Result<AtomicNumber> {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return refused(CdxmlRefusalReasonV1::InvalidScalar);
    }
    let value = value.parse::<u8>().map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidScalar,
    })?;
    AtomicNumber::try_from(value).map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidScalar,
    })
}

pub(super) fn element_symbol(value: &str) -> Result<AtomicNumber> {
    if has_entity_reference(value) {
        return refused(CdxmlRefusalReasonV1::EntityForbidden);
    }
    AtomicNumber::from_symbol(value).map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidScalar,
    })
}

pub(super) fn allowed_root_attribute(name: &str) -> bool {
    matches!(
        name,
        "CreationProgram"
            | "Name"
            | "BoundingBox"
            | "WindowPosition"
            | "WindowSize"
            | "WindowIsZoomed"
            | "FractionalWidths"
            | "InterpretChemically"
            | "ShowAtomQuery"
            | "ShowBondQuery"
            | "LabelFont"
            | "CaptionFont"
            | "HashSpacing"
            | "MarginWidth"
            | "LineWidth"
            | "BoldWidth"
            | "BondLength"
            | "BondSpacing"
            | "ChainAngle"
            | "PrintMargins"
            | "MacPrintInfo"
    )
}
pub(super) fn allowed_page_attribute(name: &str) -> bool {
    matches!(
        name,
        "HeightPages"
            | "WidthPages"
            | "BoundingBox"
            | "Name"
            | "PageDefinition"
            | "Header"
            | "Footer"
            | "PrintMargins"
            | "Color"
            | "BackgroundColor"
    )
}
pub(super) fn allowed_font_attribute(name: &str) -> bool {
    matches!(name, "id" | "name" | "charset" | "family")
}
pub(super) fn allowed_color_attribute(name: &str) -> bool {
    matches!(name, "r" | "g" | "b" | "a")
}

/// DTD-declared text properties accepted only because V1 intentionally drops
/// label presentation while retaining the direct `s` element symbol.
pub(super) fn allowed_text_attribute(name: &str) -> bool {
    matches!(
        name,
        "p" | "BoundingBox"
            | "Justification"
            | "LabelAlignment"
            | "LineHeight"
            | "CaptionLineHeight"
            | "Interpretation"
            | "color"
            | "font"
            | "size"
            | "face"
    )
}

/// DTD-declared direct text-span styling that has no molecule-only meaning.
pub(super) fn allowed_span_attribute(name: &str) -> bool {
    matches!(name, "font" | "size" | "face" | "color")
}
