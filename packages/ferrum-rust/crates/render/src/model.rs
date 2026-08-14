//! Closed, ownership-first wire grammar for immutable render plans.

use std::collections::HashSet;

use ferrum_core::{RecordId, RecordKind};
use serde::{Deserialize, Serialize};

use crate::{RenderError, RenderIssue};

const SCHEMA_V1: &str = "ferrum-render-plan-v1";

/// The only schema accepted by this initial native render-plan slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderSchemaVersion {
    /// Initial declarative Ferrum render-plan grammar.
    V1,
}

impl Serialize for RenderSchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(SCHEMA_V1)
    }
}

impl<'de> Deserialize<'de> for RenderSchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == SCHEMA_V1 {
            Ok(Self::V1)
        } else {
            Err(serde::de::Error::custom("unknown render-plan schema"))
        }
    }
}

/// An explicit document projection revision, including the initial session revision zero.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct RenderRevision(u64);

impl RenderRevision {
    /// Construct a revision supplied by the authoritative document session.
    pub const fn new(value: u64) -> Result<Self, RenderError> {
        Ok(Self(value))
    }

    /// Return the durable projection revision.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for RenderRevision {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Exact document identity shared by every plan produced from one observation.
///
/// A revision alone cannot identify document content after independent sessions
/// evolve. V1 therefore carries the authoritative structural digest beside it
/// through construction and the strict wire grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderProvenance {
    revision: RenderRevision,
    digest: [u8; 32],
}

impl RenderProvenance {
    /// Construct the exact document identity for one render-plan result.
    #[must_use]
    pub const fn new(revision: RenderRevision, digest: [u8; 32]) -> Self {
        Self { revision, digest }
    }

    /// Return the exact document revision, including valid initial revision zero.
    #[must_use]
    pub const fn revision(self) -> RenderRevision {
        self.revision
    }

    /// Return the exact structural document digest.
    #[must_use]
    pub const fn digest(self) -> [u8; 32] {
        self.digest
    }
}

/// A finite point in the explicit render-plan coordinate system.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderPoint {
    x: f64,
    y: f64,
}

impl RenderPoint {
    /// Construct a point whose coordinates are finite.
    pub fn new(x: f64, y: f64) -> Result<Self, RenderError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(RenderError::InvalidRequest(
                "render coordinates must be finite".to_owned(),
            ));
        }
        Ok(Self {
            x: canonical_wire_float(x),
            y: canonical_wire_float(y),
        })
    }

    /// Return the horizontal coordinate.
    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }

    /// Return the vertical coordinate.
    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
}

impl<'de> Deserialize<'de> for RenderPoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WirePoint {
            x: f64,
            y: f64,
        }
        let wire = WirePoint::deserialize(deserializer)?;
        Self::new(wire.x, wire.y).map_err(serde::de::Error::custom)
    }
}

/// Normalize a validated finite scalar for stable JSON serialization.
///
/// Rust considers signed zeroes equal while JSON preserves their spelling. The
/// render wire grammar represents either zero as positive zero so equal plans
/// always have identical canonical JSON bytes.
fn canonical_wire_float(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// A finite strictly positive presentation extent.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct PositiveFinite(f64);

impl PositiveFinite {
    /// Construct a finite positive value.
    pub fn new(value: f64) -> Result<Self, RenderError> {
        if !value.is_finite() || value <= 0.0 {
            return Err(RenderError::InvalidRequest(
                "render extent must be finite and positive".to_owned(),
            ));
        }
        Ok(Self(canonical_wire_float(value)))
    }

    /// Return the validated extent.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for PositiveFinite {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(f64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// A lowercase six-digit RGB color with no toolkit-specific interpretation.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Rgb24(String);

impl Rgb24 {
    /// Construct a lowercase six-digit RGB value such as `"cc3366"`.
    pub fn new(value: impl Into<String>) -> Result<Self, RenderError> {
        let value = value.into();
        if value.len() != 6
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(RenderError::InvalidRequest(
                "RGB color must contain exactly six lowercase hexadecimal digits".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    /// Return the canonical RGB text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Rgb24 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// An explicit RGB paint with no toolkit or document-semantic fallback.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Paint(Rgb24);

impl Paint {
    /// Construct an explicit paint from a validated RGB value.
    #[must_use]
    pub const fn rgb24(value: Rgb24) -> Self {
        Self(value)
    }

    /// Return the exact RGB value a renderer must paint.
    #[must_use]
    pub fn color(&self) -> &Rgb24 {
        &self.0
    }
}

/// The immutable identifier of the only face accepted by the V1 render grammar.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct FontFace(String);

impl FontFace {
    /// Create the verified bundled Telex Regular face identifier.
    #[must_use]
    pub fn telex_regular() -> Self {
        Self("ferrum-telex-regular-v1".to_owned())
    }

    /// Validate the immutable Telex resource identifier without a fallback.
    pub fn new(value: impl Into<String>) -> Result<Self, RenderError> {
        let value = value.into();
        if value != "ferrum-telex-regular-v1" {
            return Err(RenderError::InvalidRequest(
                "V1 requires the ferrum-telex-regular-v1 font resource".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    /// Return the stable immutable font resource identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for FontFace {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

use crate::{GlyphPlacement, TextScript};

/// One nonempty semantic portion of a text operation.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextRun {
    text: String,
    script: TextScript,
    origin: RenderPoint,
    glyphs: Vec<GlyphPlacement>,
    scale: PositiveFinite,
}

impl TextRun {
    /// Construct a nonempty label run with fully explicit local layout.
    pub(crate) fn new(
        text: impl Into<String>,
        script: TextScript,
        origin: RenderPoint,
        glyphs: Vec<GlyphPlacement>,
        scale: PositiveFinite,
    ) -> Result<Self, RenderError> {
        let text = text.into();
        if !is_meaningful_text(&text) {
            return Err(RenderError::InvalidRequest(
                "text run must contain visible text and no control characters".to_owned(),
            ));
        }
        if glyphs.len() != text.chars().count() {
            return Err(RenderError::InvalidRequest(
                "text run requires exactly one verified glyph placement per Unicode scalar"
                    .to_owned(),
            ));
        }
        Ok(Self {
            text,
            script,
            origin,
            glyphs,
            scale,
        })
    }

    /// Return the exact run text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Return the semantic script placement.
    #[must_use]
    pub const fn script(&self) -> TextScript {
        self.script
    }
    /// Return the explicit baseline origin relative to the text operation origin.
    #[must_use]
    pub const fn origin(&self) -> RenderPoint {
        self.origin
    }
    /// Return exact Telex glyph IDs and run-local origins in scalar order.
    ///
    /// A consumer must neither shape nor advance this sequence.  It draws each
    /// supplied glyph ID at `TextOp.origin + TextRun.origin + glyph.origin`.
    #[must_use]
    pub fn glyphs(&self) -> &[GlyphPlacement] {
        &self.glyphs
    }
    /// Return the explicit multiplier for the operation's exact font size.
    #[must_use]
    pub const fn scale(&self) -> PositiveFinite {
        self.scale
    }
}

/// Return whether text can be represented safely and deliberately by V1.
///
/// The initial grammar has no multiline or invisible-text layout semantics, so
/// NUL, every Unicode control character, and whitespace-only placeholders are
/// rejected. Chemical labels and ordinary font names retain their exact text.
fn is_meaningful_text(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}

/// An atom-label draw operation with explicit presentation facts.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TextOp {
    origin: RenderPoint,
    runs: Vec<TextRun>,
    face: FontFace,
    size: PositiveFinite,
    paint: Paint,
    z: i32,
}

impl TextOp {
    /// Construct a fully specified text operation.
    pub(crate) fn new(
        origin: RenderPoint,
        runs: Vec<TextRun>,
        face: FontFace,
        size: PositiveFinite,
        paint: Paint,
        z: i32,
    ) -> Result<Self, RenderError> {
        if runs.is_empty() {
            return Err(RenderError::InvalidRequest(
                "text operation requires at least one run".to_owned(),
            ));
        }
        let environment = crate::FerrumFontEnvironmentV1::load()?;
        let metrics = crate::VerifiedTelexGlyphMetrics::new(&environment)?;
        for run in &runs {
            metrics.validate_v1_run(run.text(), run.script(), size, run.scale(), run.glyphs())?;
        }
        Ok(Self {
            origin,
            runs,
            face,
            size,
            paint,
            z,
        })
    }

    /// Return the immutable text origin.
    #[must_use]
    pub const fn origin(&self) -> RenderPoint {
        self.origin
    }
    /// Return structured text runs in paint order.
    #[must_use]
    pub fn runs(&self) -> &[TextRun] {
        &self.runs
    }
    /// Return the exact requested font face.
    #[must_use]
    pub fn face(&self) -> &FontFace {
        &self.face
    }
    /// Return the validated font size.
    #[must_use]
    pub const fn size(&self) -> PositiveFinite {
        self.size
    }
    /// Return the explicit paint.
    #[must_use]
    pub fn paint(&self) -> &Paint {
        &self.paint
    }
    /// Return deterministic z-order within the batch.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

impl<'de> Deserialize<'de> for TextOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireTextRun {
            text: String,
            script: TextScript,
            origin: RenderPoint,
            glyphs: Vec<GlyphPlacement>,
            scale: PositiveFinite,
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireTextOp {
            origin: RenderPoint,
            runs: Vec<WireTextRun>,
            face: FontFace,
            size: PositiveFinite,
            paint: Paint,
            z: i32,
        }
        let wire = WireTextOp::deserialize(deserializer)?;
        let runs = wire
            .runs
            .into_iter()
            .map(|run| TextRun::new(run.text, run.script, run.origin, run.glyphs, run.scale))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)?;
        Self::new(wire.origin, runs, wire.face, wire.size, wire.paint, wire.z)
            .map_err(serde::de::Error::custom)
    }
}

/// A clipped normal-line draw operation in scene space.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LineOp {
    start: RenderPoint,
    end: RenderPoint,
    width: PositiveFinite,
    paint: Paint,
    z: i32,
}

/// An explicit atom-local rectangular label mask.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaskOp {
    origin: RenderPoint,
    width: PositiveFinite,
    height: PositiveFinite,
    paint: Paint,
    z: i32,
}

impl MaskOp {
    /// Construct a fully specified opaque label mask.
    pub fn new(
        origin: RenderPoint,
        width: PositiveFinite,
        height: PositiveFinite,
        paint: Paint,
        z: i32,
    ) -> Result<Self, RenderError> {
        Ok(Self {
            origin,
            width,
            height,
            paint,
            z,
        })
    }

    /// Return the lower-left atom-local origin.
    #[must_use]
    pub const fn origin(&self) -> RenderPoint {
        self.origin
    }
    /// Return explicit mask width.
    #[must_use]
    pub const fn width(&self) -> PositiveFinite {
        self.width
    }
    /// Return explicit mask height.
    #[must_use]
    pub const fn height(&self) -> PositiveFinite {
        self.height
    }
    /// Return explicit mask paint.
    #[must_use]
    pub fn paint(&self) -> &Paint {
        &self.paint
    }
    /// Return fixed molecule-plane z-order.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

impl<'de> Deserialize<'de> for MaskOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireMaskOp {
            origin: RenderPoint,
            width: PositiveFinite,
            height: PositiveFinite,
            paint: Paint,
            z: i32,
        }
        let wire = WireMaskOp::deserialize(deserializer)?;
        Self::new(wire.origin, wire.width, wire.height, wire.paint, wire.z)
            .map_err(serde::de::Error::custom)
    }
}

impl LineOp {
    /// Construct a nondegenerate, finite line operation.
    pub fn new(
        start: RenderPoint,
        end: RenderPoint,
        width: PositiveFinite,
        paint: Paint,
        z: i32,
    ) -> Result<Self, RenderError> {
        if start == end {
            return Err(RenderError::InvalidRequest(
                "line operation endpoints must differ".to_owned(),
            ));
        }
        Ok(Self {
            start,
            end,
            width,
            paint,
            z,
        })
    }

    /// Return the accepted clipped start point.
    #[must_use]
    pub const fn start(&self) -> RenderPoint {
        self.start
    }
    /// Return the accepted clipped end point.
    #[must_use]
    pub const fn end(&self) -> RenderPoint {
        self.end
    }
    /// Return the validated line width.
    #[must_use]
    pub const fn width(&self) -> PositiveFinite {
        self.width
    }
    /// Return the explicit paint.
    #[must_use]
    pub fn paint(&self) -> &Paint {
        &self.paint
    }
    /// Return deterministic z-order within the batch.
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

impl<'de> Deserialize<'de> for LineOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireLineOp {
            start: RenderPoint,
            end: RenderPoint,
            width: PositiveFinite,
            paint: Paint,
            z: i32,
        }
        let wire = WireLineOp::deserialize(deserializer)?;
        Self::new(wire.start, wire.end, wire.width, wire.paint, wire.z)
            .map_err(serde::de::Error::custom)
    }
}

/// The initial closed operation grammar; unsupported styles have no variant yet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "kind",
    content = "operation",
    rename_all = "snake_case"
)]
pub enum RenderOp {
    /// A structured atom label.
    Text(TextOp),
    /// A clipped normal bond.
    Line(LineOp),
    /// An explicit opaque atom-label mask.
    Mask(MaskOp),
    /// An explicit atom-local outlined or filled ellipse.
    Ellipse(crate::EllipseOp),
}

impl RenderOp {
    fn z(&self) -> i32 {
        match self {
            Self::Text(operation) => operation.z(),
            Self::Line(operation) => operation.z(),
            Self::Mask(operation) => operation.z(),
            Self::Ellipse(operation) => operation.z(),
        }
    }
}

/// A durable target identity and stable projection order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderTarget {
    record_id: RecordId,
    source_order: u32,
}

impl RenderTarget {
    /// Construct a projection target. The identity remains Ferrum-owned.
    #[must_use]
    pub fn new(record_id: RecordId, source_order: u32) -> Self {
        Self {
            record_id,
            source_order,
        }
    }
    /// Return the stable source record identity.
    #[must_use]
    pub fn record_id(&self) -> &RecordId {
        &self.record_id
    }
    /// Return the deterministic document projection order.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
}

/// The coordinate space in which a batch is interpreted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, tag = "kind", rename_all = "snake_case")]
pub enum BatchSpace {
    /// Text operations move with a finite atom anchor.
    AtomLocal { anchor: RenderPoint },
    /// Bond operations use accepted document-scene coordinates directly.
    Scene,
}

/// An immutable target-specific operation batch.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderBatch {
    target: RenderTarget,
    coordinate_space: BatchSpace,
    operations: Vec<RenderOp>,
}

impl RenderBatch {
    /// Construct one complete, type-safe atom or bond batch.
    pub fn new(
        target: RenderTarget,
        coordinate_space: BatchSpace,
        operations: Vec<RenderOp>,
    ) -> Result<Self, RenderError> {
        if operations.is_empty() {
            return Err(RenderError::InvalidRequest(
                "render batch must not be empty".to_owned(),
            ));
        }
        if operations.windows(2).any(|pair| pair[0].z() >= pair[1].z()) {
            return Err(RenderError::InvalidRequest(
                "render batch operations must have strictly increasing z".to_owned(),
            ));
        }
        match (&coordinate_space, target.record_id.kind()) {
            (BatchSpace::AtomLocal { .. }, RecordKind::Atom)
                if operations.iter().all(|op| {
                    matches!(
                        op,
                        RenderOp::Text(_)
                            | RenderOp::Mask(_)
                            | RenderOp::Line(_)
                            | RenderOp::Ellipse(_)
                    )
                }) => {}
            (BatchSpace::Scene, RecordKind::Bond)
                if operations.iter().all(|op| matches!(op, RenderOp::Line(_))) => {}
            (BatchSpace::AtomLocal { .. }, _) => {
                return Err(RenderError::InvalidRequest(
                    "atom-local batch requires an atom target and annotation operations".to_owned(),
                ));
            }
            (BatchSpace::Scene, _) => {
                return Err(RenderError::InvalidRequest(
                    "scene batch requires a bond target and line operations".to_owned(),
                ));
            }
        }
        Ok(Self {
            target,
            coordinate_space,
            operations,
        })
    }

    /// Return the durable target.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }
    /// Return the explicit coordinate interpretation.
    #[must_use]
    pub fn coordinate_space(&self) -> &BatchSpace {
        &self.coordinate_space
    }
    /// Return immutable operation data.
    #[must_use]
    pub fn operations(&self) -> &[RenderOp] {
        &self.operations
    }
}

impl<'de> Deserialize<'de> for RenderBatch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireBatch {
            target: RenderTarget,
            coordinate_space: BatchSpace,
            operations: Vec<RenderOp>,
        }
        let wire = WireBatch::deserialize(deserializer)?;
        Self::new(wire.target, wire.coordinate_space, wire.operations)
            .map_err(serde::de::Error::custom)
    }
}

/// A complete immutable response from one document-projection revision.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoleculeRenderPlan {
    schema: RenderSchemaVersion,
    provenance: RenderProvenance,
    batches: Vec<RenderBatch>,
    issues: Vec<RenderIssue>,
}

impl MoleculeRenderPlan {
    /// Construct a plan with one ordered outcome for every supplied target.
    ///
    /// A target appears exactly once as either a complete batch or an exclusion
    /// issue. Every source order is unique across both outcome lists; each list
    /// is strictly source ordered, allowing consumers to merge them without
    /// inventing a tie breaker.
    pub fn new(
        provenance: RenderProvenance,
        batches: Vec<RenderBatch>,
        issues: Vec<RenderIssue>,
    ) -> Result<Self, RenderError> {
        let mut targets = HashSet::new();
        let mut source_orders = HashSet::new();
        let mut previous_batch_source_order = None;
        for batch in &batches {
            if !targets.insert(batch.target.record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "render plan has duplicate batch targets".to_owned(),
                ));
            }
            if !source_orders.insert(batch.target.source_order()) {
                return Err(RenderError::InvalidRequest(
                    "render plan has duplicate target source order".to_owned(),
                ));
            }
            if let Some(previous) = previous_batch_source_order
                && batch.target.source_order() <= previous
            {
                return Err(RenderError::InvalidRequest(
                    "render plan batches must have strictly increasing source order".to_owned(),
                ));
            }
            previous_batch_source_order = Some(batch.target.source_order());
        }
        let mut previous_issue_source_order = None;
        for issue in &issues {
            issue.validate()?;
            let target = issue.target();
            if !targets.insert(target.record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "render plan target cannot have both a batch and an issue".to_owned(),
                ));
            }
            if !source_orders.insert(target.source_order()) {
                return Err(RenderError::InvalidRequest(
                    "render plan has duplicate target source order".to_owned(),
                ));
            }
            if let Some(previous) = previous_issue_source_order
                && target.source_order() <= previous
            {
                return Err(RenderError::InvalidRequest(
                    "render plan issues must have strictly increasing source order".to_owned(),
                ));
            }
            previous_issue_source_order = Some(target.source_order());
        }
        Ok(Self {
            schema: RenderSchemaVersion::V1,
            provenance,
            batches,
            issues,
        })
    }

    /// Return the accepted schema marker.
    #[must_use]
    pub const fn schema(&self) -> RenderSchemaVersion {
        self.schema
    }
    /// Return the exact source projection revision.
    #[must_use]
    pub const fn revision(&self) -> RenderRevision {
        self.provenance.revision()
    }
    /// Return the exact document revision and digest that produced this plan.
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    /// Return immutable target batches in source order.
    #[must_use]
    pub fn batches(&self) -> &[RenderBatch] {
        &self.batches
    }
    /// Return non-fatal excluded-target diagnostics.
    #[must_use]
    pub fn issues(&self) -> &[RenderIssue] {
        &self.issues
    }
    /// Serialize deterministic canonical JSON for the closed grammar.
    pub fn to_canonical_json(&self) -> Result<String, RenderError> {
        serde_json::to_string(self).map_err(|error| RenderError::Serialization(error.to_string()))
    }
    /// Parse and validate the exact current grammar with no compatibility aliases.
    pub fn from_json(input: &str) -> Result<Self, RenderError> {
        serde_json::from_str(input).map_err(|error| RenderError::InvalidJson(error.to_string()))
    }
}

impl<'de> Deserialize<'de> for MoleculeRenderPlan {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WirePlan {
            schema: RenderSchemaVersion,
            provenance: RenderProvenance,
            batches: Vec<RenderBatch>,
            issues: Vec<RenderIssue>,
        }
        let wire = WirePlan::deserialize(deserializer)?;
        if wire.schema != RenderSchemaVersion::V1 {
            return Err(serde::de::Error::custom("unsupported render-plan schema"));
        }
        Self::new(wire.provenance, wire.batches, wire.issues).map_err(serde::de::Error::custom)
    }
}
