//! Bounded CML scalar, attribute, and entity validation.

use std::collections::BTreeMap;

use xmlparser::{Reference, Stream};

use super::*;

pub(super) fn add_budget(
    current: usize,
    increment: usize,
    limit: usize,
    reason: CmlRefusalReasonV1,
) -> Result<usize> {
    let value = current
        .checked_add(increment)
        .ok_or(CmlDecoderErrorV1 { reason })?;
    if value > limit {
        refused(reason)
    } else {
        Ok(value)
    }
}
pub(super) fn only_default_namespace(pending: &Pending) -> Result<&str> {
    if pending.attributes.len() != 1 || pending.attributes[0].0 != "xmlns" {
        return refused(CmlRefusalReasonV1::NamespaceUnsupported);
    }
    Ok(&pending.attributes[0].1)
}
pub(super) fn require_no_attributes(pending: &Pending) -> Result<()> {
    if pending.attributes.is_empty() {
        Ok(())
    } else if pending.attributes[0].0 == "xmlns" {
        refused(CmlRefusalReasonV1::NamespaceUnsupported)
    } else {
        attribute_refusal(&pending.attributes[0].0)
    }
}
pub(super) fn optional_id(pending: &Pending) -> Result<Option<String>> {
    if pending.attributes.iter().any(|(name, _)| name == "xmlns") {
        return refused(CmlRefusalReasonV1::NamespaceUnsupported);
    }
    if pending.attributes.is_empty() {
        return Ok(None);
    }
    if pending.attributes.len() == 1 && pending.attributes[0].0 == "id" {
        return Ok(Some(pending.attributes[0].1.clone()));
    }
    if [
        "atomID",
        "elementType",
        "x2",
        "y2",
        "formalCharge",
        "isotopeNumber",
        "atomRefs2",
        "bondID",
        "order",
    ]
    .contains(&pending.attributes[0].0.as_str())
    {
        refused(CmlRefusalReasonV1::ArrayAttributeUnsupported)
    } else {
        refused(CmlRefusalReasonV1::AttributeUnsupported)
    }
}
pub(super) fn fields(
    pending: &Pending,
    allowed: &[&str],
) -> Result<BTreeMap<&'static str, String>> {
    let mut result = BTreeMap::new();
    for (name, value) in &pending.attributes {
        if !allowed.contains(&name.as_str()) {
            attribute_refusal(name)?;
        }
        let key = match name.as_str() {
            "id" => "id",
            "elementType" => "elementType",
            "x2" => "x2",
            "y2" => "y2",
            "formalCharge" => "formalCharge",
            "isotopeNumber" => "isotopeNumber",
            "atomRefs2" => "atomRefs2",
            "order" => "order",
            "builtin" => "builtin",
            _ => return refused(CmlRefusalReasonV1::AttributeUnsupported),
        };
        if result.insert(key, value.clone()).is_some() {
            return refused(CmlRefusalReasonV1::AttributeUnsupported);
        }
    }
    Ok(result)
}
pub(super) fn attribute_refusal(name: &str) -> Result<()> {
    if name == "xmlns" {
        refused(CmlRefusalReasonV1::NamespaceUnsupported)
    } else if [
        "atomID",
        "elementType",
        "x2",
        "y2",
        "formalCharge",
        "isotopeNumber",
        "atomRefs2",
        "bondID",
        "order",
    ]
    .contains(&name)
    {
        refused(CmlRefusalReasonV1::ArrayAttributeUnsupported)
    } else {
        refused(CmlRefusalReasonV1::AttributeUnsupported)
    }
}
pub(super) fn validate_id(value: &str) -> Result<()> {
    if value.is_empty() || value.len() > MAX_IDENTIFIER_BYTES || !value.is_ascii() {
        return refused(if value.len() > MAX_IDENTIFIER_BYTES {
            CmlRefusalReasonV1::IdentifierBytesLimit
        } else {
            CmlRefusalReasonV1::InvalidScalar
        });
    }
    let bytes = value.as_bytes();
    if !matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z' | b'_')
        || !bytes.iter().skip(1).all(
            |byte| matches!(*byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'.'),
        )
    {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    Ok(())
}
pub(super) fn coordinate(value: &str) -> Result<f64> {
    if !decimal(value) {
        return refused(CmlRefusalReasonV1::InvalidCoordinate);
    }
    let value = value.parse::<f64>().map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidCoordinate,
    })?;
    if !value.is_finite() {
        return refused(CmlRefusalReasonV1::CoordinateNotFinite);
    }
    if value.abs() > MAX_COORDINATE {
        return refused(CmlRefusalReasonV1::CoordinateOutOfRange);
    }
    Ok(value)
}
pub(super) fn signed(value: &str, minimum: i32, maximum: i32) -> Result<i32> {
    if value.is_empty()
        || value.trim() != value
        || !value.as_bytes().iter().enumerate().all(|(index, byte)| {
            byte.is_ascii_digit() || (index == 0 && matches!(*byte, b'+' | b'-'))
        })
    {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    let parsed = value.parse::<i32>().map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidScalar,
    })?;
    if !(minimum..=maximum).contains(&parsed) {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    Ok(parsed)
}
pub(super) fn unsigned(value: &str, minimum: u32, maximum: u32) -> Result<u32> {
    if value.is_empty() || !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    let parsed = value.parse::<u32>().map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidScalar,
    })?;
    if !(minimum..=maximum).contains(&parsed) {
        return refused(CmlRefusalReasonV1::InvalidScalar);
    }
    Ok(parsed)
}
pub(super) fn decimal(value: &str) -> bool {
    let value = value.strip_prefix(['+', '-']).unwrap_or(value);
    !value.is_empty()
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || *byte == b'.')
        && value.bytes().filter(|byte| *byte == b'.').count() <= 1
        && value.bytes().any(|byte| byte.is_ascii_digit())
}
pub(super) fn decode_entities(
    value: &str,
    mut remaining: usize,
    limit_reason: CmlRefusalReasonV1,
) -> Result<String> {
    let mut stream = Stream::from(value);
    let mut start = 0;
    let mut decoded = String::new();
    while !stream.at_end() {
        if stream.curr_byte().map_err(|_| CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InvalidXml,
        })? == b'&'
        {
            append_bounded(
                &mut decoded,
                &value[start..stream.pos()],
                &mut remaining,
                limit_reason,
            )?;
            match stream.consume_reference().map_err(|_| CmlDecoderErrorV1 {
                reason: CmlRefusalReasonV1::EntityForbidden,
            })? {
                Reference::Char(character) => {
                    append_character_bounded(
                        &mut decoded,
                        character,
                        &mut remaining,
                        limit_reason,
                    )?;
                }
                Reference::Entity(name)
                    if matches!(name, "amp" | "lt" | "gt" | "quot" | "apos") =>
                {
                    let character = match name {
                        "amp" => '&',
                        "lt" => '<',
                        "gt" => '>',
                        "quot" => '\"',
                        _ => '\'',
                    };
                    append_character_bounded(
                        &mut decoded,
                        character,
                        &mut remaining,
                        limit_reason,
                    )?;
                }
                Reference::Entity(_) => return refused(CmlRefusalReasonV1::EntityForbidden),
            }
            start = stream.pos();
        } else {
            let character = value[stream.pos()..]
                .chars()
                .next()
                .ok_or(CmlDecoderErrorV1 {
                    reason: CmlRefusalReasonV1::InvalidXml,
                })?;
            stream.advance(character.len_utf8());
        }
    }
    append_bounded(&mut decoded, &value[start..], &mut remaining, limit_reason)?;
    Ok(decoded)
}

pub(super) fn append_bounded(
    output: &mut String,
    value: &str,
    remaining: &mut usize,
    limit_reason: CmlRefusalReasonV1,
) -> Result<()> {
    if value.len() > *remaining {
        return refused(limit_reason);
    }
    output.push_str(value);
    *remaining -= value.len();
    Ok(())
}

pub(super) fn append_character_bounded(
    output: &mut String,
    value: char,
    remaining: &mut usize,
    limit_reason: CmlRefusalReasonV1,
) -> Result<()> {
    let byte_count = value.len_utf8();
    if byte_count > *remaining {
        return refused(limit_reason);
    }
    output.push(value);
    *remaining -= byte_count;
    Ok(())
}
