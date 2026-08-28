//! Closed, ownership-first wire grammar for immutable render plans.

use serde::{Deserialize, Serialize};

use crate::{RenderError, RenderPaintV3};

const SCHEMA_V4: &str = "ferrum-render-plan-v4";

/// The only schema accepted by the active native render-plan slice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderSchemaVersion {
    /// Declarative Ferrum render-plan grammar with typed batch content.
    V4,
}

impl Serialize for RenderSchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(SCHEMA_V4)
    }
}

impl<'de> Deserialize<'de> for RenderSchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == SCHEMA_V4 {
            Ok(Self::V4)
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
    paint: RenderPaintV3,
    z: i32,
}

impl TextOp {
    /// Construct a fully specified text operation.
    pub(crate) fn new(
        origin: RenderPoint,
        runs: Vec<TextRun>,
        face: FontFace,
        size: PositiveFinite,
        paint: RenderPaintV3,
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
    pub fn paint(&self) -> &RenderPaintV3 {
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
            paint: RenderPaintV3,
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
    paint: RenderPaintV3,
    z: i32,
}

/// An explicit atom-local rectangular label mask.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MaskOp {
    origin: RenderPoint,
    width: PositiveFinite,
    height: PositiveFinite,
    paint: RenderPaintV3,
    z: i32,
}

impl MaskOp {
    /// Construct a fully specified opaque label mask.
    pub fn new(
        origin: RenderPoint,
        width: PositiveFinite,
        height: PositiveFinite,
        paint: RenderPaintV3,
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
    pub fn paint(&self) -> &RenderPaintV3 {
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
            paint: RenderPaintV3,
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
        paint: RenderPaintV3,
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
    pub fn paint(&self) -> &RenderPaintV3 {
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
            paint: RenderPaintV3,
            z: i32,
        }
        let wire = WireLineOp::deserialize(deserializer)?;
        Self::new(wire.start, wire.end, wire.width, wire.paint, wire.z)
            .map_err(serde::de::Error::custom)
    }
}

/// The closed generic operation grammar used inside a typed V4 batch.
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
    /// An explicit filled and/or stroked scene path.
    Path(crate::PathOpV3),
    /// A stored E/Z carrier accent linked to its central double-bond provenance.
    DoubleBondCarrierMark(crate::DoubleBondCarrierMarkOp),
}
