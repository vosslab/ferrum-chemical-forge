//! Immutable presentation-stack values and construction invariants.

use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

use super::super::{
    ArrowProjectionV1, BoxShapeProjectionV1, BracketPairProjectionV1, PlusProjectionV1,
    PolygonProjectionV1, TextProjectionV1,
};
use super::stack_wire::{
    PolylinePathWireV1, PolylineWireV1, PresentationIssueWireV1, PresentationRootWireV1,
    PresentationStackWireV1, PresentationStrokeWireV1, PresentationTargetWireV1,
};
use crate::{DocumentObjectIdV1, Point3V1, PositiveFiniteV1, ProjectionLocalObjectKeyV1, Rgb24V1};

pub(super) const BUILTIN_LINE_COLOR: &str = "#000000";
pub(super) const BUILTIN_LINE_WIDTH: f64 = 1.0;

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
        Self::new(
            wire.revision,
            wire.digest,
            wire.roots,
            wire.bracket_pairs,
            wire.issues,
        )
        .map_err(serde::de::Error::custom)
    }
}

impl PresentationStackProjectionV1 {
    /// Construct one revision-bound immutable presentation stack.
    ///
    /// The stack owns every projected root, pair, and issue. Construction
    /// refuses a root/pair mismatch so all consumers observe one internally
    /// consistent direct-root presentation view.
    pub fn new(
        revision: u64,
        digest: [u8; 32],
        roots: Vec<PresentationRootProjectionV1>,
        bracket_pairs: Vec<BracketPairProjectionV1>,
        issues: Vec<PresentationProjectionIssueV1>,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        validate_root_identities(&roots)?;
        if !round_bracket_roots_match_pairs(&roots, &bracket_pairs) {
            return Err(PresentationStackProjectionV1Error::RoundBracketPairMismatch);
        }
        Ok(Self {
            schema: PRESENTATION_STACK_PROJECTION_SCHEMA_V1,
            revision,
            digest,
            roots,
            bracket_pairs,
            issues,
        })
    }

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

fn validate_root_identities(
    roots: &[PresentationRootProjectionV1],
) -> Result<(), PresentationStackProjectionV1Error> {
    let mut source_orders = BTreeSet::new();
    let mut projection_keys = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    let mut durable_ids = BTreeSet::new();
    for root in roots {
        validate_root_kind(root)?;
        let target = root.target();
        if !source_orders.insert(target.source_order()) {
            return Err(PresentationStackProjectionV1Error::DuplicateRootSourceOrder);
        }
        if !projection_keys.insert(target.projection_key().as_str()) {
            return Err(PresentationStackProjectionV1Error::DuplicateRootProjectionKey);
        }
        if let Some(source_id) = target.source_id()
            && !source_ids.insert(source_id)
        {
            return Err(PresentationStackProjectionV1Error::DuplicateRootSourceId);
        }
        if let Some(id) = target.id()
            && !durable_ids.insert(id.as_str())
        {
            return Err(PresentationStackProjectionV1Error::DuplicateRootDurableId);
        }
    }
    Ok(())
}

fn round_bracket_roots_match_pairs(
    roots: &[PresentationRootProjectionV1],
    pairs: &[BracketPairProjectionV1],
) -> bool {
    let expected = pairs
        .iter()
        .filter(|pair| pair.style() == super::super::PresentationBracketStyleV1::Round)
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

/// Closed refusal taxonomy for immutable presentation-stack construction.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PresentationStackProjectionV1Error {
    /// A direct-root target does not identify one consistent source record.
    #[error("presentation target identity is inconsistent")]
    InvalidTargetIdentity,
    /// A root payload does not agree with its target record kind.
    #[error("presentation root kind does not match its target record kind")]
    RootKindMismatch,
    /// A polyline requires at least two finite ordered points.
    #[error("presentation polyline path requires at least two finite points")]
    InvalidPolylinePath,
    /// A polygon requires at least three finite ordered points.
    #[error("presentation polygon path requires at least three finite points")]
    InvalidPolygonPath,
    /// Bounds must be finite with positive width and height.
    #[error("presentation bounds must be finite with positive width and height")]
    InvalidBounds,
    /// Stroke facts conflict with their declared provenance.
    #[error("presentation stroke facts conflict with their provenance")]
    InvalidStroke,
    /// Fill facts conflict with their declared provenance.
    #[error("presentation fill facts conflict with their provenance")]
    InvalidFill,
    /// Font facts conflict with their declared provenance.
    #[error("presentation font facts conflict with their provenance")]
    InvalidFont,
    /// Text runs are not normalized renderable content.
    #[error("presentation text runs are not normalized renderable content")]
    InvalidTextRuns,
    /// A durable bracket pair is internally inconsistent.
    #[error("presentation bracket pair durable identity is invalid")]
    InvalidBracketPair,
    /// Direct-root source order must be unique.
    #[error("presentation root source order is duplicated")]
    DuplicateRootSourceOrder,
    /// Direct-root local projection keys must be unique.
    #[error("presentation root projection-local key is duplicated")]
    DuplicateRootProjectionKey,
    /// Authored durable source IDs must be unique among roots.
    #[error("presentation root source ID is duplicated")]
    DuplicateRootSourceId,
    /// Durable object IDs must be unique among roots.
    #[error("presentation root durable ID is duplicated")]
    DuplicateRootDurableId,
    /// Round-bracket roots and retained bracket-pair members disagree.
    #[error("round bracket roots do not match their projected pair members")]
    RoundBracketPairMismatch,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PresentationRootProjectionV1 {
    /// A supported normal non-spline CDML arrow with semantic authored facts.
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
    /// Construct an Arrow root only for an Arrow target.
    pub fn arrow(arrow: ArrowProjectionV1) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Arrow { arrow }, PresentationRecordKindV1::Arrow)
    }

    /// Construct a Plus root only for a Plus target.
    pub fn plus(plus: PlusProjectionV1) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Plus { plus }, PresentationRecordKindV1::Plus)
    }

    /// Construct a Text root only for a Text target.
    pub fn text(text: TextProjectionV1) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Text { text }, PresentationRecordKindV1::Text)
    }

    /// Construct a normal polyline root only for a Polyline target.
    pub fn polyline(
        polyline: PolylineProjectionV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(
            Self::Polyline { polyline },
            PresentationRecordKindV1::Polyline,
        )
    }

    /// Construct a Wavy polyline root only for a Polyline target.
    pub fn wavy(
        polyline: PolylineProjectionV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Wavy { polyline }, PresentationRecordKindV1::Polyline)
    }

    /// Construct a RoundBracket root only for a Polyline target.
    pub fn round_bracket(
        polyline: PolylineProjectionV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(
            Self::RoundBracket { polyline },
            PresentationRecordKindV1::Polyline,
        )
    }

    /// Construct a rectangle root only for a Rectangle target.
    pub fn rectangle(
        shape: BoxShapeProjectionV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(
            Self::Rectangle { shape },
            PresentationRecordKindV1::Rectangle,
        )
    }

    /// Construct a square root only for a Square target.
    pub fn square(shape: BoxShapeProjectionV1) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Square { shape }, PresentationRecordKindV1::Square)
    }

    /// Construct an oval root only for an Oval target.
    pub fn oval(shape: BoxShapeProjectionV1) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Oval { shape }, PresentationRecordKindV1::Oval)
    }

    /// Construct a circle root only for a Circle target.
    pub fn circle(shape: BoxShapeProjectionV1) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Circle { shape }, PresentationRecordKindV1::Circle)
    }

    /// Construct a polygon root only for a Polygon target.
    pub fn polygon(
        polygon: PolygonProjectionV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        root_for_kind(Self::Polygon { polygon }, PresentationRecordKindV1::Polygon)
    }
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

fn root_for_kind(
    root: PresentationRootProjectionV1,
    expected: PresentationRecordKindV1,
) -> Result<PresentationRootProjectionV1, PresentationStackProjectionV1Error> {
    (root.target().record_kind() == expected)
        .then_some(root)
        .ok_or(PresentationStackProjectionV1Error::RootKindMismatch)
}

fn validate_root_kind(
    root: &PresentationRootProjectionV1,
) -> Result<(), PresentationStackProjectionV1Error> {
    let expected = match root {
        PresentationRootProjectionV1::Arrow { .. } => PresentationRecordKindV1::Arrow,
        PresentationRootProjectionV1::Plus { .. } => PresentationRecordKindV1::Plus,
        PresentationRootProjectionV1::Text { .. } => PresentationRecordKindV1::Text,
        PresentationRootProjectionV1::Polyline { .. }
        | PresentationRootProjectionV1::Wavy { .. }
        | PresentationRootProjectionV1::RoundBracket { .. } => PresentationRecordKindV1::Polyline,
        PresentationRootProjectionV1::Rectangle { .. } => PresentationRecordKindV1::Rectangle,
        PresentationRootProjectionV1::Square { .. } => PresentationRecordKindV1::Square,
        PresentationRootProjectionV1::Oval { .. } => PresentationRecordKindV1::Oval,
        PresentationRootProjectionV1::Circle { .. } => PresentationRecordKindV1::Circle,
        PresentationRootProjectionV1::Polygon { .. } => PresentationRecordKindV1::Polygon,
    };
    (root.target().record_kind() == expected)
        .then_some(())
        .ok_or(PresentationStackProjectionV1Error::RootKindMismatch)
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
    pub fn new(
        target: PresentationTargetV1,
        path: PolylinePathV1,
        stroke: PresentationStrokeV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        (target.record_kind() == PresentationRecordKindV1::Polyline)
            .then_some(Self {
                target,
                path,
                stroke,
            })
            .ok_or(PresentationStackProjectionV1Error::RootKindMismatch)
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
    /// Construct a target whose durable and authored identities agree.
    pub fn try_new(
        id: Option<DocumentObjectIdV1>,
        projection_key: ProjectionLocalObjectKeyV1,
        source_id: Option<String>,
        source_order: u32,
        record_kind: PresentationRecordKindV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if id.is_some() != source_id.is_some() {
            return Err(PresentationStackProjectionV1Error::InvalidTargetIdentity);
        }
        if let (Some(id), Some(source_id)) = (&id, &source_id) {
            let expected =
                DocumentObjectIdV1::from_class_source(record_kind.class_name(), source_id)
                    .map_err(|_| PresentationStackProjectionV1Error::InvalidTargetIdentity)?;
            if id != &expected {
                return Err(PresentationStackProjectionV1Error::InvalidTargetIdentity);
            }
        }
        Ok(Self {
            id,
            projection_key,
            source_id,
            source_order,
            record_kind,
        })
    }
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
    pub(super) fn class_name(self) -> &'static str {
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

    /// Return the exact CDML local name associated with this root kind.
    #[must_use]
    pub const fn local_name(self) -> &'static str {
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
    pub fn try_new(points: Vec<Point3V1>) -> Result<Self, PresentationStackProjectionV1Error> {
        (points.len() >= 2)
            .then_some(Self { points })
            .ok_or(PresentationStackProjectionV1Error::InvalidPolylinePath)
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
    pub fn new(
        color: Rgb24V1,
        color_provenance: PresentationFactProvenanceV1,
        width: PositiveFiniteV1,
        width_provenance: PresentationFactProvenanceV1,
    ) -> Result<Self, PresentationStackProjectionV1Error> {
        if (color_provenance == PresentationFactProvenanceV1::Builtin
            && color.as_str() != BUILTIN_LINE_COLOR)
            || (width_provenance == PresentationFactProvenanceV1::Builtin
                && width.value() != BUILTIN_LINE_WIDTH)
        {
            return Err(PresentationStackProjectionV1Error::InvalidStroke);
        }
        Ok(Self {
            color,
            color_provenance,
            width,
            width_provenance,
        })
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
    pub fn new(
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
    /// A Text or Plus requested a face outside the closed bundled resource set.
    UnsupportedTextFace,
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
