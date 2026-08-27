//! Typed-CDML adapter for immutable bracket-pair projection values.

use std::collections::BTreeMap;

use ferrum_document_projection::{BracketPairProjectionV1, PresentationBracketStyleV1};

use super::presentation_polyline_projection_v1::parse_width;
use super::{
    CDML_NAMESPACE, PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass, TypedDocument, TypedRecord,
    UnrecognizedNode,
};

pub(crate) fn bracket_pairs(
    document: &TypedDocument,
) -> Result<Vec<BracketPairProjectionV1>, crate::ProjectionError> {
    let children = document
        .root()
        .typed_children()
        .iter()
        .filter(|child| child.record().class() == TypedClass::Polyline)
        .collect::<Vec<_>>();
    let standard = document
        .root()
        .typed_children()
        .iter()
        .find(|child| child.record().class() == TypedClass::Standard)
        .map(TypedChild::record);
    let mut by_pair = BTreeMap::<&str, Vec<&TypedChild>>::new();
    for child in &children {
        if let Some(pair_id) = child.record().attribute("bracket_pair") {
            by_pair.entry(pair_id).or_default().push(child);
        }
    }
    let mut pairs = Vec::new();
    for child in children {
        let record = child.record();
        if record.attribute("bracket_side") != Some("left") {
            continue;
        }
        let Some(pair_id) = record.attribute("bracket_pair") else {
            continue;
        };
        let Some(candidates) = by_pair.get(pair_id) else {
            continue;
        };
        let Some(pair) = observed_pair(pair_id, candidates, standard)? else {
            continue;
        };
        pairs.push(pair);
    }
    Ok(pairs)
}

fn observed_pair(
    pair_id: &str,
    candidates: &[&TypedChild],
    standard: Option<&TypedRecord>,
) -> Result<Option<BracketPairProjectionV1>, crate::ProjectionError> {
    if candidates.len() != 2 {
        return Ok(None);
    }
    let Some(left) = candidates
        .iter()
        .find(|child| child.record().attribute("bracket_side") == Some("left"))
        .map(|child| child.record())
    else {
        return Ok(None);
    };
    let Some(right) = candidates
        .iter()
        .find(|child| child.record().attribute("bracket_side") == Some("right"))
        .map(|child| child.record())
    else {
        return Ok(None);
    };
    let Some(left_id) = left.attribute("id") else {
        return Ok(None);
    };
    let Some(right_id) = right.attribute("id") else {
        return Ok(None);
    };
    if left_id != pair_id || left_id == right_id {
        return Ok(None);
    }
    if !valid_bracket_member(left) || !valid_bracket_member(right) {
        return Ok(None);
    }
    let Some(style) = shared_style(left, right) else {
        return Ok(None);
    };
    let Some(left_stroke) = resolved_stroke(left, standard) else {
        return Ok(None);
    };
    let Some(right_stroke) = resolved_stroke(right, standard) else {
        return Ok(None);
    };
    let left_object_id =
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(left)?;
    let right_object_id =
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(right)?;
    BracketPairProjectionV1::try_new(
        [left_object_id, right_object_id],
        style,
        (left_stroke.0 == right_stroke.0).then_some(left_stroke.0),
        (left_stroke.1 == right_stroke.1).then_some(left_stroke.1),
    )
    .map(Some)
    .map_err(|error| crate::ProjectionError::InvalidValue {
        context: format!("bracket pair {pair_id}"),
        field: "bracket pair projection",
        value: error.to_string(),
    })
}

pub(crate) fn valid_bracket_member(record: &TypedRecord) -> bool {
    let points = record.children_of(TypedClass::Point).collect::<Vec<_>>();
    points.len() == 4
        && record.attribute("style") != Some("wavy")
        && record
            .typed_children()
            .iter()
            .all(|child| child.record().class() == TypedClass::Point)
        && !has_unsupported_core_content(record)
        && points.into_iter().all(|source| {
            source.typed_children().is_empty()
                && !has_unsupported_core_content(source)
                && super::presentation_polyline_projection_v1::point(source).is_ok()
        })
}

fn has_unsupported_core_content(record: &TypedRecord) -> bool {
    record
        .unrecognized_children()
        .iter()
        .any(|child| match child.node() {
            UnrecognizedNode::Element { name, .. } => name.namespace() == CDML_NAMESPACE,
            UnrecognizedNode::Text(value) => !value.trim().is_empty(),
            UnrecognizedNode::Comment(_) | UnrecognizedNode::ProcessingInstruction { .. } => false,
        })
}

fn shared_style(left: &TypedRecord, right: &TypedRecord) -> Option<PresentationBracketStyleV1> {
    let left = left.attribute("spline")?;
    let right = right.attribute("spline")?;
    if matches!(left, "no" | "false" | "0") && matches!(right, "no" | "false" | "0") {
        Some(PresentationBracketStyleV1::Rectangular)
    } else if matches!(left, "yes" | "true" | "1") && matches!(right, "yes" | "true" | "1") {
        Some(PresentationBracketStyleV1::Round)
    } else {
        None
    }
}

fn resolved_stroke(
    record: &TypedRecord,
    standard: Option<&TypedRecord>,
) -> Option<(PositiveFiniteV1, Rgb24V1)> {
    let width = [(Some(record), "width"), (standard, "line_width")]
        .into_iter()
        .find_map(|(record, field)| record.and_then(|item| item.attribute(field)))
        .map_or_else(|| PositiveFiniteV1::new(1.0), parse_width)?;
    let color = [
        (Some(record), "line_color"),
        (Some(record), "color"),
        (standard, "line_color"),
    ]
    .into_iter()
    .find_map(|(record, field)| record.and_then(|item| item.attribute(field)))
    .map_or_else(|| Rgb24V1::new("#000000"), Rgb24V1::new)?;
    Some((width, color))
}
