//! Exact durable relationship facts for paired top-level bracket polylines.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use super::presentation_polyline_projection_v1::parse_width;
use super::{
    BracketStyleV1, CDML_NAMESPACE, PositiveFiniteV1, Rgb24V1, TypedChild, TypedClass,
    TypedDocument, TypedRecord, UnrecognizedNode,
};

/// One structurally valid durable bracket pair.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BracketPairProjectionV1 {
    pair_id: String,
    member_ids: [String; 2],
    style: BracketStyleV1,
    line_width: Option<PositiveFiniteV1>,
    line_color: Option<Rgb24V1>,
}

impl<'de> Deserialize<'de> for BracketPairProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BracketPairWireV1::deserialize(deserializer)?
            .try_into()
            .map_err(serde::de::Error::custom)
    }
}

impl BracketPairProjectionV1 {
    /// Return the left member's durable source ID, which is also the pair ID.
    #[must_use]
    pub fn pair_id(&self) -> &str {
        &self.pair_id
    }

    /// Return left and right durable source IDs in side order.
    #[must_use]
    pub fn member_ids(&self) -> &[String; 2] {
        &self.member_ids
    }

    /// Return the exact shared spline family.
    #[must_use]
    pub fn style(&self) -> BracketStyleV1 {
        self.style
    }

    /// Return the common resolved width, or `None` when the two sides differ.
    #[must_use]
    pub fn line_width(&self) -> Option<PositiveFiniteV1> {
        self.line_width
    }

    /// Return the common resolved colour, or `None` when the two sides differ.
    #[must_use]
    pub fn line_color(&self) -> Option<&Rgb24V1> {
        self.line_color.as_ref()
    }
}

pub(crate) fn bracket_pairs(document: &TypedDocument) -> Vec<BracketPairProjectionV1> {
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
        let Some(pair) = observed_pair(pair_id, candidates, standard) else {
            continue;
        };
        pairs.push(pair);
    }
    pairs
}

fn observed_pair(
    pair_id: &str,
    candidates: &[&TypedChild],
    standard: Option<&TypedRecord>,
) -> Option<BracketPairProjectionV1> {
    if candidates.len() != 2 {
        return None;
    }
    let left = candidates
        .iter()
        .find(|child| child.record().attribute("bracket_side") == Some("left"))?
        .record();
    let right = candidates
        .iter()
        .find(|child| child.record().attribute("bracket_side") == Some("right"))?
        .record();
    let left_id = left.attribute("id")?;
    let right_id = right.attribute("id")?;
    if left_id != pair_id || left_id == right_id {
        return None;
    }
    if !valid_bracket_member(left) || !valid_bracket_member(right) {
        return None;
    }
    let style = shared_style(left, right)?;
    let left_stroke = resolved_stroke(left, standard)?;
    let right_stroke = resolved_stroke(right, standard)?;
    Some(BracketPairProjectionV1 {
        pair_id: pair_id.to_owned(),
        member_ids: [left_id.to_owned(), right_id.to_owned()],
        style,
        line_width: (left_stroke.0 == right_stroke.0).then_some(left_stroke.0),
        line_color: (left_stroke.1 == right_stroke.1).then_some(left_stroke.1),
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

fn shared_style(left: &TypedRecord, right: &TypedRecord) -> Option<BracketStyleV1> {
    let left = left.attribute("spline")?;
    let right = right.attribute("spline")?;
    if matches!(left, "no" | "false" | "0") && matches!(right, "no" | "false" | "0") {
        Some(BracketStyleV1::Rectangular)
    } else if matches!(left, "yes" | "true" | "1") && matches!(right, "yes" | "true" | "1") {
        Some(BracketStyleV1::Round)
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BracketPairWireV1 {
    pair_id: String,
    member_ids: [String; 2],
    style: BracketStyleV1,
    line_width: Option<f64>,
    line_color: Option<String>,
}

impl TryFrom<BracketPairWireV1> for BracketPairProjectionV1 {
    type Error = &'static str;

    fn try_from(value: BracketPairWireV1) -> Result<Self, Self::Error> {
        if value.pair_id.trim().is_empty()
            || value.member_ids[0] != value.pair_id
            || value.member_ids[1].trim().is_empty()
            || value.member_ids[0] == value.member_ids[1]
        {
            return Err("invalid bracket pair durable identity");
        }
        let line_width = match value.line_width {
            Some(width) => {
                Some(PositiveFiniteV1::new(width).ok_or("invalid bracket pair common width")?)
            }
            None => None,
        };
        let line_color = match value.line_color {
            Some(color) => Some(Rgb24V1::new(color).ok_or("invalid bracket pair common colour")?),
            None => None,
        };
        Ok(Self {
            pair_id: value.pair_id,
            member_ids: value.member_ids,
            style: value.style,
            line_width,
            line_color,
        })
    }
}
