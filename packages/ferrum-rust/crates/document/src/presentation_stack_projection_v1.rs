//! Direct-root presentation projection with explicit display provenance.

use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use super::presentation_arrow_projection_v1::arrow;
use super::presentation_plus_projection_v1::plus;
use super::presentation_polyline_projection_v1::{
    PolylineProjectionKindV1, RootStrokeDefaultsV1, polyline,
};
use super::presentation_shape_projection_v1::{box_shape, polygon};
use super::presentation_text_projection_v1::text;
use super::{
    ArrowProjectionV1, BoxShapeProjectionV1, BracketPairProjectionV1, DocumentObjectIdV1,
    DocumentSnapshot, PlusProjectionV1, Point3V1, PolygonProjectionV1, PositiveFiniteV1,
    ProjectionLocalObjectKeyV1, Rgb24V1, TextProjectionV1, TypedChild, TypedClass, TypedDocument,
};
const BUILTIN_LINE_COLOR: &str = "#000000";
const BUILTIN_LINE_WIDTH: f64 = 1.0;

/// Stable schema identifier for [`PresentationStackProjectionV1`].
pub const PRESENTATION_STACK_PROJECTION_SCHEMA_V1: &str = "ferrum-presentation-stack-v1";

/// Direct-root display targets from one revision-bound document snapshot.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationStackProjectionV1 {
    schema: &'static str,
    revision: u64,
    digest: [u8; 32],
    roots: Vec<PresentationRootProjectionV1>,
    bracket_pairs: Vec<BracketPairProjectionV1>,
    issues: Vec<PresentationProjectionIssueV1>,
}

impl<'de> Deserialize<'de> for PresentationStackProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationStackWireV1::deserialize(deserializer)?;
        if wire.schema != PRESENTATION_STACK_PROJECTION_SCHEMA_V1 {
            return Err(serde::de::Error::custom(
                "unknown presentation stack schema",
            ));
        }
        if !round_bracket_roots_match_pairs(&wire.roots, &wire.bracket_pairs) {
            return Err(serde::de::Error::custom(
                "round bracket roots do not match their projected pair members",
            ));
        }
        Ok(Self {
            schema: PRESENTATION_STACK_PROJECTION_SCHEMA_V1,
            revision: wire.revision,
            digest: wire.digest,
            roots: wire.roots,
            bracket_pairs: wire.bracket_pairs,
            issues: wire.issues,
        })
    }
}

impl PresentationStackProjectionV1 {
    pub(crate) fn from_snapshot(document: &TypedDocument, snapshot: &DocumentSnapshot) -> Self {
        let standard = RootStrokeDefaultsV1::from_document(document);
        let bracket_pairs = super::bracket_pair_projection_v1::bracket_pairs(document);
        let round_members = bracket_pairs
            .iter()
            .filter(|pair| pair.style() == super::BracketStyleV1::Round)
            .flat_map(|pair| pair.member_ids().iter().map(String::as_str))
            .collect::<BTreeSet<_>>();
        let mut roots = Vec::new();
        let mut issues = Vec::new();
        for child in document.root().typed_children() {
            match child.record().class() {
                TypedClass::CanvasArrow => {
                    if let Some(arrow) = arrow(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Arrow { arrow });
                    }
                }
                TypedClass::CanvasPlus => {
                    if let Some(plus) = plus(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Plus { plus });
                    }
                }
                TypedClass::CanvasText => {
                    if let Some(text) = text(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Text { text });
                    }
                }
                TypedClass::Polyline => {
                    let round_bracket_member = child
                        .record()
                        .attribute("id")
                        .is_some_and(|identifier| round_members.contains(identifier));
                    if let Some((kind, polyline)) =
                        polyline(child, standard, round_bracket_member, &mut issues)
                    {
                        roots.push(match kind {
                            PolylineProjectionKindV1::Ordinary => {
                                PresentationRootProjectionV1::Polyline { polyline }
                            }
                            PolylineProjectionKindV1::Wavy => {
                                PresentationRootProjectionV1::Wavy { polyline }
                            }
                            PolylineProjectionKindV1::RoundBracket => {
                                PresentationRootProjectionV1::RoundBracket { polyline }
                            }
                        });
                    }
                }
                TypedClass::Rectangle => {
                    if let Some(shape) = box_shape(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Rectangle { shape });
                    }
                }
                TypedClass::Square => {
                    if let Some(shape) = box_shape(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Square { shape });
                    }
                }
                TypedClass::Oval => {
                    if let Some(shape) = box_shape(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Oval { shape });
                    }
                }
                TypedClass::Circle => {
                    if let Some(shape) = box_shape(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Circle { shape });
                    }
                }
                TypedClass::Polygon => {
                    if let Some(polygon) = polygon(child, standard, &mut issues) {
                        roots.push(PresentationRootProjectionV1::Polygon { polygon });
                    }
                }
                TypedClass::Cdml
                | TypedClass::Info
                | TypedClass::Metadata
                | TypedClass::Standard
                | TypedClass::Paper
                | TypedClass::Viewport
                | TypedClass::Molecule
                | TypedClass::Reaction
                | TypedClass::ExternalData => {}
                _ => unreachable!("only root typed children occur here"),
            }
        }
        Self {
            schema: PRESENTATION_STACK_PROJECTION_SCHEMA_V1,
            revision: snapshot.revision(),
            digest: *snapshot.digest(),
            roots,
            bracket_pairs,
            issues,
        }
    }

    /// Return the closed schema identifier.
    #[must_use]
    pub fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the source snapshot revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Return the source snapshot digest.
    #[must_use]
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Return renderable supported roots in direct root source order.
    #[must_use]
    pub fn roots(&self) -> &[PresentationRootProjectionV1] {
        &self.roots
    }

    /// Return exact durable bracket relationships in left-root source order.
    #[must_use]
    pub fn bracket_pairs(&self) -> &[BracketPairProjectionV1] {
        &self.bracket_pairs
    }

    /// Return explicit unsupported or invalid-root issues in encounter order.
    #[must_use]
    pub fn issues(&self) -> &[PresentationProjectionIssueV1] {
        &self.issues
    }
}

fn round_bracket_roots_match_pairs(
    roots: &[PresentationRootProjectionV1],
    pairs: &[BracketPairProjectionV1],
) -> bool {
    let expected = pairs
        .iter()
        .filter(|pair| pair.style() == super::BracketStyleV1::Round)
        .flat_map(|pair| pair.member_ids().iter().map(String::as_str))
        .collect::<Vec<_>>();
    let expected_set = expected.iter().copied().collect::<BTreeSet<_>>();
    let actual = roots
        .iter()
        .filter_map(|root| match root {
            PresentationRootProjectionV1::RoundBracket { polyline } => {
                polyline.target().source_id()
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let actual_set = actual.iter().copied().collect::<BTreeSet<_>>();
    expected.len() == expected_set.len()
        && actual.len() == actual_set.len()
        && expected_set == actual_set
        && roots.iter().all(|root| {
            root.target().source_id().is_none_or(|identifier| {
                !expected_set.contains(identifier)
                    || matches!(root, PresentationRootProjectionV1::RoundBracket { .. })
            })
        })
}

/// One supported direct-root projection, discriminated for future root classes.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PresentationRootProjectionV1 {
    /// A supported normal non-spline CDML arrow with explicit display geometry.
    Arrow { arrow: ArrowProjectionV1 },
    /// A fixed-content plus sign with resolved source appearance facts.
    Plus { plus: PlusProjectionV1 },
    /// A free-form Text label with normalized authored formatting runs.
    Text { text: TextProjectionV1 },
    /// A non-spline CDML polyline with two or more ordered points.
    Polyline { polyline: PolylineProjectionV1 },
    /// A Wavy CDML polyline rendered from its exact authored point path.
    Wavy { polyline: PolylineProjectionV1 },
    /// One spline side of a structurally valid durable round bracket pair.
    RoundBracket { polyline: PolylineProjectionV1 },
    /// A CDML rectangle described by normalized finite source bounds.
    Rectangle { shape: BoxShapeProjectionV1 },
    /// A CDML square described by normalized finite source bounds.
    Square { shape: BoxShapeProjectionV1 },
    /// A CDML oval described by normalized finite source bounds.
    Oval { shape: BoxShapeProjectionV1 },
    /// A CDML circle described by normalized finite source bounds.
    Circle { shape: BoxShapeProjectionV1 },
    /// A CDML polygon with three or more ordered finite points.
    Polygon { polygon: PolygonProjectionV1 },
}

impl<'de> Deserialize<'de> for PresentationRootProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationRootWireV1::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl PresentationRootProjectionV1 {
    /// Return the target independently of the root-kind payload.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        match self {
            Self::Arrow { arrow } => arrow.target(),
            Self::Plus { plus } => plus.target(),
            Self::Text { text } => text.target(),
            Self::Polyline { polyline }
            | Self::Wavy { polyline }
            | Self::RoundBracket { polyline } => polyline.target(),
            Self::Rectangle { shape }
            | Self::Square { shape }
            | Self::Oval { shape }
            | Self::Circle { shape } => shape.target(),
            Self::Polygon { polygon } => polygon.target(),
        }
    }
}

/// One supported direct-root polyline-family path with its root kind kept separate.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PolylineProjectionV1 {
    target: PresentationTargetV1,
    path: PolylinePathV1,
    stroke: PresentationStrokeV1,
}

impl<'de> Deserialize<'de> for PolylineProjectionV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PolylineWireV1::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl PolylineProjectionV1 {
    pub(crate) fn new(
        target: PresentationTargetV1,
        path: PolylinePathV1,
        stroke: PresentationStrokeV1,
    ) -> Self {
        Self {
            target,
            path,
            stroke,
        }
    }

    /// Return the identity and direct-root ordering facts.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    /// Return the exact ordered source path.
    #[must_use]
    pub fn path(&self) -> &PolylinePathV1 {
        &self.path
    }

    /// Return fully resolved display stroke facts.
    #[must_use]
    pub fn stroke(&self) -> &PresentationStrokeV1 {
        &self.stroke
    }
}

/// A durable-or-local direct-root display target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PresentationTargetV1 {
    id: Option<DocumentObjectIdV1>,
    projection_key: ProjectionLocalObjectKeyV1,
    source_id: Option<String>,
    source_order: u32,
    record_kind: PresentationRecordKindV1,
}

impl<'de> Deserialize<'de> for PresentationTargetV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationTargetWireV1::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl PresentationTargetV1 {
    pub(crate) fn from_child(child: &TypedChild) -> Self {
        let record = child.record();
        Self {
            id: DocumentObjectIdV1::from_record(record),
            projection_key: ProjectionLocalObjectKeyV1::from_record(record),
            source_id: record.attribute("id").map(str::to_owned),
            source_order: child.position(),
            record_kind: PresentationRecordKindV1::from_class(record.class())
                .expect("only supported presentation roots receive targets"),
        }
    }

    /// Return the authored durable selector, when the root supplied an ID.
    #[must_use]
    pub fn id(&self) -> Option<&DocumentObjectIdV1> {
        self.id.as_ref()
    }

    /// Return a unique projection-local key that is never an operation target.
    #[must_use]
    pub fn projection_key(&self) -> &ProjectionLocalObjectKeyV1 {
        &self.projection_key
    }

    /// Return the literal authored source ID.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }

    /// Return the position among all direct root children.
    #[must_use]
    pub fn source_order(&self) -> u32 {
        self.source_order
    }

    /// Return the closed persistent record kind represented by this target.
    #[must_use]
    pub fn record_kind(&self) -> PresentationRecordKindV1 {
        self.record_kind
    }
}

/// Closed persistent record kinds supported by the V1 presentation stack.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationRecordKindV1 {
    Arrow,
    Plus,
    Text,
    Polyline,
    Rectangle,
    Square,
    Oval,
    Circle,
    Polygon,
}

impl PresentationRecordKindV1 {
    fn from_class(class: TypedClass) -> Option<Self> {
        match class {
            TypedClass::CanvasArrow => Some(Self::Arrow),
            TypedClass::CanvasPlus => Some(Self::Plus),
            TypedClass::CanvasText => Some(Self::Text),
            TypedClass::Polyline => Some(Self::Polyline),
            TypedClass::Rectangle => Some(Self::Rectangle),
            TypedClass::Square => Some(Self::Square),
            TypedClass::Oval => Some(Self::Oval),
            TypedClass::Circle => Some(Self::Circle),
            TypedClass::Polygon => Some(Self::Polygon),
            _ => None,
        }
    }

    fn class_name(self) -> &'static str {
        match self {
            Self::Arrow => "cdml/arrow",
            Self::Plus => "cdml/plus",
            Self::Text => "cdml/text",
            Self::Polyline => "cdml/polyline",
            Self::Rectangle => "cdml/rect",
            Self::Square => "cdml/square",
            Self::Oval => "cdml/oval",
            Self::Circle => "cdml/circle",
            Self::Polygon => "cdml/polygon",
        }
    }

    pub(crate) const fn local_name(self) -> &'static str {
        match self {
            Self::Arrow => "arrow",
            Self::Plus => "plus",
            Self::Text => "text",
            Self::Polyline => "polyline",
            Self::Rectangle => "rect",
            Self::Square => "square",
            Self::Oval => "oval",
            Self::Circle => "circle",
            Self::Polygon => "polygon",
        }
    }
}

/// Two or more ordered finite points whose root kind defines their interpolation.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PolylinePathV1 {
    points: Vec<Point3V1>,
}

impl<'de> Deserialize<'de> for PolylinePathV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PolylinePathWireV1::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl PolylinePathV1 {
    pub(crate) fn new(points: Vec<Point3V1>) -> Self {
        Self { points }
    }

    /// Return every authored point in source order.
    #[must_use]
    pub fn points(&self) -> &[Point3V1] {
        &self.points
    }
}

/// Resolved polyline stroke values with the source that supplied each fact.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PresentationStrokeV1 {
    color: Rgb24V1,
    color_provenance: PresentationFactProvenanceV1,
    width: PositiveFiniteV1,
    width_provenance: PresentationFactProvenanceV1,
}

impl<'de> Deserialize<'de> for PresentationStrokeV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = PresentationStrokeWireV1::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl PresentationStrokeV1 {
    pub(crate) fn new(
        color: Rgb24V1,
        color_provenance: PresentationFactProvenanceV1,
        width: PositiveFiniteV1,
        width_provenance: PresentationFactProvenanceV1,
    ) -> Self {
        Self {
            color,
            color_provenance,
            width,
            width_provenance,
        }
    }

    /// Return the validated display colour.
    #[must_use]
    pub fn color(&self) -> &Rgb24V1 {
        &self.color
    }

    /// Return how the display colour was selected.
    #[must_use]
    pub fn color_provenance(&self) -> PresentationFactProvenanceV1 {
        self.color_provenance
    }

    /// Return the validated display width.
    #[must_use]
    pub fn width(&self) -> PositiveFiniteV1 {
        self.width
    }

    /// Return how the display width was selected.
    #[must_use]
    pub fn width_provenance(&self) -> PresentationFactProvenanceV1 {
        self.width_provenance
    }
}

/// The explicit precedence source for one resolved display fact.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationFactProvenanceV1 {
    /// The direct-root polyline supplied the fact.
    Root,
    /// The first direct-root drawing standard supplied the fact.
    Standard,
    /// The closed V1 built-in supplied the fact.
    Builtin,
}

/// A deterministic non-rendering result for one direct-root target.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PresentationProjectionIssueV1 {
    target: PresentationTargetV1,
    code: PresentationProjectionIssueCodeV1,
    detail: String,
}

impl<'de> Deserialize<'de> for PresentationProjectionIssueV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        PresentationIssueWireV1::deserialize(deserializer).map(Into::into)
    }
}

impl PresentationProjectionIssueV1 {
    pub(crate) fn new(
        target: PresentationTargetV1,
        code: PresentationProjectionIssueCodeV1,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            target,
            code,
            detail: detail.into(),
        }
    }

    /// Return the display-only target associated with this issue.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        &self.target
    }

    /// Return the closed issue category.
    #[must_use]
    pub fn code(&self) -> PresentationProjectionIssueCodeV1 {
        self.code
    }

    /// Return deterministic source detail without a rendering fallback.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Closed direct-root projection failure categories.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationProjectionIssueCodeV1 {
    /// An arrow does not have a valid finite source path or active head segment.
    InvalidArrowGeometry,
    /// An arrow boolean or head-shape fact is invalid.
    InvalidArrowFact,
    /// The persistent arrow family is preserved but not rendered by this V1 slice.
    UnsupportedArrowType,
    /// Spline interpolation is preserved but not rendered by this V1 slice.
    UnsupportedArrowSpline,
    /// A plus sign has no single finite anchor point.
    InvalidPlusGeometry,
    /// A Text label has no single finite anchor point.
    InvalidTextGeometry,
    /// A Text label has malformed or unsupported formatted character data.
    InvalidTextContent,
    /// A plus font size, family, or colour fact is malformed.
    InvalidFontFact,
    /// The root has fewer than two valid ordered points.
    InvalidPolylineGeometry,
    /// A bound-based shape has missing or invalid finite corners.
    InvalidShapeGeometry,
    /// A polygon has fewer than three valid ordered points.
    InvalidPolygonGeometry,
    /// The root requests spline geometry, which this slice does not implement.
    UnsupportedSpline,
    /// The root is a specialized Wavy polyline with a separate render contract.
    UnsupportedPolylineStyle,
    /// A retained presentation fact was invalid and lower-precedence data was used.
    InvalidStrokeFact,
    /// A retained fill fact was invalid and lower-precedence data was used.
    InvalidFillFact,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationStackWireV1 {
    schema: String,
    revision: u64,
    digest: [u8; 32],
    roots: Vec<PresentationRootProjectionV1>,
    bracket_pairs: Vec<BracketPairProjectionV1>,
    issues: Vec<PresentationProjectionIssueV1>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum PresentationRootWireV1 {
    Arrow { arrow: ArrowProjectionV1 },
    Plus { plus: PlusProjectionV1 },
    Text { text: TextProjectionV1 },
    Polyline { polyline: PolylineProjectionV1 },
    Wavy { polyline: PolylineProjectionV1 },
    RoundBracket { polyline: PolylineProjectionV1 },
    Rectangle { shape: BoxShapeProjectionV1 },
    Square { shape: BoxShapeProjectionV1 },
    Oval { shape: BoxShapeProjectionV1 },
    Circle { shape: BoxShapeProjectionV1 },
    Polygon { polygon: PolygonProjectionV1 },
}

impl TryFrom<PresentationRootWireV1> for PresentationRootProjectionV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PresentationRootWireV1) -> Result<Self, Self::Error> {
        let (root, actual, expected) = match value {
            PresentationRootWireV1::Arrow { arrow } => {
                let actual = arrow.target().record_kind();
                (
                    Self::Arrow { arrow },
                    actual,
                    PresentationRecordKindV1::Arrow,
                )
            }
            PresentationRootWireV1::Plus { plus } => {
                let actual = plus.target().record_kind();
                (Self::Plus { plus }, actual, PresentationRecordKindV1::Plus)
            }
            PresentationRootWireV1::Text { text } => {
                let actual = text.target().record_kind();
                (Self::Text { text }, actual, PresentationRecordKindV1::Text)
            }
            PresentationRootWireV1::Polyline { polyline } => {
                let actual = polyline.target().record_kind();
                (
                    Self::Polyline { polyline },
                    actual,
                    PresentationRecordKindV1::Polyline,
                )
            }
            PresentationRootWireV1::Wavy { polyline } => {
                let actual = polyline.target().record_kind();
                (
                    Self::Wavy { polyline },
                    actual,
                    PresentationRecordKindV1::Polyline,
                )
            }
            PresentationRootWireV1::RoundBracket { polyline } => {
                let actual = polyline.target().record_kind();
                (
                    Self::RoundBracket { polyline },
                    actual,
                    PresentationRecordKindV1::Polyline,
                )
            }
            PresentationRootWireV1::Rectangle { shape } => {
                let actual = shape.target().record_kind();
                (
                    Self::Rectangle { shape },
                    actual,
                    PresentationRecordKindV1::Rectangle,
                )
            }
            PresentationRootWireV1::Square { shape } => {
                let actual = shape.target().record_kind();
                (
                    Self::Square { shape },
                    actual,
                    PresentationRecordKindV1::Square,
                )
            }
            PresentationRootWireV1::Oval { shape } => {
                let actual = shape.target().record_kind();
                (Self::Oval { shape }, actual, PresentationRecordKindV1::Oval)
            }
            PresentationRootWireV1::Circle { shape } => {
                let actual = shape.target().record_kind();
                (
                    Self::Circle { shape },
                    actual,
                    PresentationRecordKindV1::Circle,
                )
            }
            PresentationRootWireV1::Polygon { polygon } => {
                let actual = polygon.target().record_kind();
                (
                    Self::Polygon { polygon },
                    actual,
                    PresentationRecordKindV1::Polygon,
                )
            }
        };
        if actual != expected {
            return Err(serde::de::Error::custom(
                "presentation root kind does not match its target record kind",
            ));
        }
        Ok(root)
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolylineWireV1 {
    target: PresentationTargetV1,
    path: PolylinePathV1,
    stroke: PresentationStrokeV1,
}

impl TryFrom<PolylineWireV1> for PolylineProjectionV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PolylineWireV1) -> Result<Self, Self::Error> {
        Ok(Self {
            target: value.target,
            path: value.path,
            stroke: value.stroke,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationTargetWireV1 {
    id: Option<String>,
    projection_key: String,
    source_id: Option<String>,
    source_order: u32,
    record_kind: PresentationRecordKindV1,
}

impl TryFrom<PresentationTargetWireV1> for PresentationTargetV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PresentationTargetWireV1) -> Result<Self, Self::Error> {
        let id = value
            .id
            .map(DocumentObjectIdV1::parse)
            .transpose()
            .map_err(|error| serde::de::Error::custom(error.to_string()))?;
        let projection_key = ProjectionLocalObjectKeyV1::parse(value.projection_key)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation projection-local key"))?;
        if id.is_some() != value.source_id.is_some() {
            return Err(serde::de::Error::custom(
                "presentation target ID and source ID must be present together",
            ));
        }
        if let (Some(id), Some(source_id)) = (&id, &value.source_id) {
            let expected =
                DocumentObjectIdV1::from_class_source(value.record_kind.class_name(), source_id);
            if *id != expected {
                return Err(serde::de::Error::custom(
                    "presentation target ID does not match its record kind and source ID",
                ));
            }
        }
        Ok(Self {
            id,
            projection_key,
            source_id: value.source_id,
            source_order: value.source_order,
            record_kind: value.record_kind,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolylinePathWireV1 {
    points: Vec<PointWireV1>,
}

impl TryFrom<PolylinePathWireV1> for PolylinePathV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PolylinePathWireV1) -> Result<Self, Self::Error> {
        if value.points.len() < 2 {
            return Err(serde::de::Error::custom(
                "polyline path requires at least two finite points",
            ));
        }
        Ok(Self {
            points: value
                .points
                .iter()
                .map(PointWireV1::to_point)
                .collect::<Result<_, _>>()?,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PointWireV1 {
    x: f64,
    y: f64,
    z: f64,
}

impl PointWireV1 {
    fn to_point(&self) -> Result<Point3V1, serde::de::value::Error> {
        Point3V1::new(self.x, self.y, self.z)
            .map_err(|error| serde::de::Error::custom(error.to_string()))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PresentationStrokeWireV1 {
    color: String,
    color_provenance: PresentationFactProvenanceV1,
    width: f64,
    width_provenance: PresentationFactProvenanceV1,
}

impl TryFrom<PresentationStrokeWireV1> for PresentationStrokeV1 {
    type Error = serde::de::value::Error;

    fn try_from(value: PresentationStrokeWireV1) -> Result<Self, Self::Error> {
        let color = Rgb24V1::new(value.color)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation stroke colour"))?;
        let width = PositiveFiniteV1::new(value.width)
            .ok_or_else(|| serde::de::Error::custom("invalid presentation stroke width"))?;
        if value.color_provenance == PresentationFactProvenanceV1::Builtin
            && color.as_str() != BUILTIN_LINE_COLOR
        {
            return Err(serde::de::Error::custom(
                "built-in presentation stroke colour must use the closed V1 value",
            ));
        }
        if value.width_provenance == PresentationFactProvenanceV1::Builtin
            && width.value() != BUILTIN_LINE_WIDTH
        {
            return Err(serde::de::Error::custom(
                "built-in presentation stroke width must use the closed V1 value",
            ));
        }
        Ok(Self {
            color,
            color_provenance: value.color_provenance,
            width,
            width_provenance: value.width_provenance,
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PresentationIssueWireV1 {
    target: PresentationTargetV1,
    code: PresentationProjectionIssueCodeV1,
    detail: String,
}

impl From<PresentationIssueWireV1> for PresentationProjectionIssueV1 {
    fn from(value: PresentationIssueWireV1) -> Self {
        Self {
            target: value.target,
            code: value.code,
            detail: value.detail,
        }
    }
}
