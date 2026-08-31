//! Closed V4 batch content and immutable molecule render-plan wire grammar.

use std::collections::HashSet;

use ferrum_core::{RecordId, RecordKind};
use serde::{Deserialize, Serialize};

use crate::glyph_metrics::GlyphBounds;
use crate::{
    DoubleBondCarrierMarkOp, EllipseOp, LineOp, MaskOp, PathOpV3, PositiveFinite, RenderError,
    RenderIssue, RenderOp, RenderPoint, RenderProvenance, RenderRevision, RenderSchemaVersion,
    RenderTarget, TextOp, TextScript,
};

/// A finite, nonempty V4 wire rectangle in atom-local scene units.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InkBoundsV1 {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl InkBoundsV1 {
    /// Construct canonical finite visible-ink bounds.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Self, RenderError> {
        let glyph = GlyphBounds::new(min_x, min_y, max_x, max_y)?;
        Ok(Self::from_glyph_bounds(glyph))
    }
    /// Convert exact internal glyph bounds at the render-wire boundary.
    #[must_use]
    pub(crate) fn from_glyph_bounds(bounds: GlyphBounds) -> Self {
        Self {
            min_x: canon(bounds.min_x()),
            min_y: canon(bounds.min_y()),
            max_x: canon(bounds.max_x()),
            max_y: canon(bounds.max_y()),
        }
    }
    #[must_use]
    pub const fn min_x(self) -> f64 {
        self.min_x
    }
    #[must_use]
    pub const fn min_y(self) -> f64 {
        self.min_y
    }
    #[must_use]
    pub const fn max_x(self) -> f64 {
        self.max_x
    }
    #[must_use]
    pub const fn max_y(self) -> f64 {
        self.max_y
    }
    #[must_use]
    pub fn contains(self, other: Self) -> bool {
        self.min_x <= other.min_x
            && self.min_y <= other.min_y
            && self.max_x >= other.max_x
            && self.max_y >= other.max_y
    }
    pub fn center(self) -> Result<RenderPoint, RenderError> {
        RenderPoint::new(
            canon((self.min_x + self.max_x) / 2.0),
            canon((self.min_y + self.max_y) / 2.0),
        )
    }
}

impl<'de> Deserialize<'de> for InkBoundsV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            min_x: f64,
            min_y: f64,
            max_x: f64,
            max_y: f64,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.min_x, wire.min_y, wire.max_x, wire.max_y).map_err(serde::de::Error::custom)
    }
}

/// Frozen atom-label facts that atom-bond clipping and Qt must not infer.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AtomLabelRenderV1 {
    mask: Option<MaskOp>,
    text: TextOp,
    core_element_run_index: u32,
    bond_ink_clearance: PositiveFinite,
    full_ink_bounds: InkBoundsV1,
    core_element_ink_bounds: InkBoundsV1,
}

/// One borrowed replay operation derived from its sole typed batch content.
///
/// This private sink-facing view intentionally owns no parallel operation
/// vector. Public consumers that need owned values use `RenderBatchV4::operations`.
pub(crate) enum RenderOperationRef<'a> {
    Line(&'a LineOp),
    Mask(&'a MaskOp),
    Ellipse(&'a EllipseOp),
    Path(&'a PathOpV3),
    DoubleBondCarrierMark(&'a DoubleBondCarrierMarkOp),
    Text(&'a TextOp),
}

impl AtomLabelRenderV1 {
    /// Construct a verified label from the exact Rust-issued layout result.
    pub fn new(
        mask: Option<MaskOp>,
        text: TextOp,
        core_element_run_index: u32,
        bond_ink_clearance: PositiveFinite,
        full_ink_bounds: InkBoundsV1,
        core_element_ink_bounds: InkBoundsV1,
    ) -> Result<Self, RenderError> {
        let index = usize::try_from(core_element_run_index).map_err(|_| {
            RenderError::InvalidRequest("atom-label core run index is not addressable".to_owned())
        })?;
        let run = text.runs().get(index).ok_or_else(|| {
            RenderError::InvalidRequest(
                "atom-label core run index is outside label text".to_owned(),
            )
        })?;
        if run.script() != TextScript::Baseline {
            return Err(RenderError::InvalidRequest(
                "atom-label core run must use baseline script".to_owned(),
            ));
        }
        if !full_ink_bounds.contains(core_element_ink_bounds) {
            return Err(RenderError::InvalidRequest(
                "atom-label core bounds must lie within full label bounds".to_owned(),
            ));
        }
        let environment = crate::FerrumFontEnvironmentV1::load()?;
        let metrics = crate::VerifiedTelexGlyphMetrics::new(&environment)?;
        if InkBoundsV1::from_glyph_bounds(metrics.v1_atom_label_ink_bounds(&text, index)?)
            != full_ink_bounds
        {
            return Err(RenderError::InvalidRequest(
                "atom-label full bounds must equal exact Telex label ink".to_owned(),
            ));
        }
        if InkBoundsV1::from_glyph_bounds(metrics.v1_centered_core_run_ink_bounds(&text, index)?)
            != core_element_ink_bounds
        {
            return Err(RenderError::InvalidRequest(
                "atom-label core bounds must equal indexed Telex run ink".to_owned(),
            ));
        }
        let center = core_element_ink_bounds.center()?;
        if center.x() != 0.0 || center.y() != 0.0 {
            return Err(RenderError::InvalidRequest(
                "atom-label core bounds center must equal local origin".to_owned(),
            ));
        }
        if let Some(mask) = &mask
            && mask.z() >= text.z()
        {
            return Err(RenderError::InvalidRequest(
                "atom-label mask must paint before label text".to_owned(),
            ));
        }
        Ok(Self {
            mask,
            text,
            core_element_run_index,
            bond_ink_clearance,
            full_ink_bounds,
            core_element_ink_bounds,
        })
    }
    #[must_use]
    pub fn mask(&self) -> Option<&MaskOp> {
        self.mask.as_ref()
    }
    #[must_use]
    pub fn text(&self) -> &TextOp {
        &self.text
    }
    #[must_use]
    pub const fn core_element_run_index(&self) -> u32 {
        self.core_element_run_index
    }
    /// Return the renderer-issued positive gap around full label ink.
    #[must_use]
    pub const fn bond_ink_clearance(&self) -> PositiveFinite {
        self.bond_ink_clearance
    }
    #[must_use]
    pub const fn full_ink_bounds(&self) -> InkBoundsV1 {
        self.full_ink_bounds
    }
    #[must_use]
    pub const fn core_element_ink_bounds(&self) -> InkBoundsV1 {
        self.core_element_ink_bounds
    }
}

impl<'de> Deserialize<'de> for AtomLabelRenderV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            mask: Option<MaskOp>,
            text: TextOp,
            core_element_run_index: u32,
            bond_ink_clearance: PositiveFinite,
            full_ink_bounds: InkBoundsV1,
            core_element_ink_bounds: InkBoundsV1,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(
            wire.mask,
            wire.text,
            wire.core_element_run_index,
            wire.bond_ink_clearance,
            wire.full_ink_bounds,
            wire.core_element_ink_bounds,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// An atom-local operation that follows the mandatory semantic label.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "kind",
    content = "operation",
    rename_all = "snake_case"
)]
pub enum AtomDecorationRenderOpV1 {
    Text(TextOp),
    Line(LineOp),
    Ellipse(EllipseOp),
}
impl AtomDecorationRenderOpV1 {
    fn z(&self) -> i32 {
        match self {
            Self::Text(op) => op.z(),
            Self::Line(op) => op.z(),
            Self::Ellipse(op) => op.z(),
        }
    }
    fn as_render_op(&self) -> RenderOp {
        match self {
            Self::Text(op) => RenderOp::Text(op.clone()),
            Self::Line(op) => RenderOp::Line(op.clone()),
            Self::Ellipse(op) => RenderOp::Ellipse(op.clone()),
        }
    }
}

/// Exactly one atom-local label plus optional decorations.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AtomRenderBatchV1 {
    atom_local_anchor: RenderPoint,
    label: AtomLabelRenderV1,
    decorations: Vec<AtomDecorationRenderOpV1>,
}
impl AtomRenderBatchV1 {
    pub fn new(
        atom_local_anchor: RenderPoint,
        label: AtomLabelRenderV1,
        decorations: Vec<AtomDecorationRenderOpV1>,
    ) -> Result<Self, RenderError> {
        let mut previous = label.text().z();
        for decoration in &decorations {
            if decoration.z() <= previous {
                return Err(RenderError::InvalidRequest(
                    "atom label and decorations require strictly increasing z".to_owned(),
                ));
            }
            previous = decoration.z();
        }
        Ok(Self {
            atom_local_anchor,
            label,
            decorations,
        })
    }
    #[must_use]
    pub const fn atom_local_anchor(&self) -> RenderPoint {
        self.atom_local_anchor
    }
    #[must_use]
    pub fn label(&self) -> &AtomLabelRenderV1 {
        &self.label
    }
    #[must_use]
    pub fn decorations(&self) -> &[AtomDecorationRenderOpV1] {
        &self.decorations
    }
    fn operations(&self) -> Vec<RenderOp> {
        self.label
            .mask()
            .into_iter()
            .cloned()
            .map(RenderOp::Mask)
            .chain(std::iter::once(RenderOp::Text(self.label.text().clone())))
            .chain(
                self.decorations
                    .iter()
                    .map(AtomDecorationRenderOpV1::as_render_op),
            )
            .collect()
    }
}
impl<'de> Deserialize<'de> for AtomRenderBatchV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            atom_local_anchor: RenderPoint,
            label: AtomLabelRenderV1,
            decorations: Vec<AtomDecorationRenderOpV1>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.atom_local_anchor, wire.label, wire.decorations)
            .map_err(serde::de::Error::custom)
    }
}

/// A generic atom-local compact-group operation with no atom-label semantics.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "kind",
    content = "operation",
    rename_all = "snake_case"
)]
pub enum CompactGroupRenderOpV1 {
    Text(TextOp),
    Line(LineOp),
    Ellipse(EllipseOp),
}
impl CompactGroupRenderOpV1 {
    fn z(&self) -> i32 {
        match self {
            Self::Text(op) => op.z(),
            Self::Line(op) => op.z(),
            Self::Ellipse(op) => op.z(),
        }
    }
    fn as_render_op(&self) -> RenderOp {
        match self {
            Self::Text(op) => RenderOp::Text(op.clone()),
            Self::Line(op) => RenderOp::Line(op.clone()),
            Self::Ellipse(op) => RenderOp::Ellipse(op.clone()),
        }
    }
}
/// Atom-local compact-group content.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompactGroupRenderBatchV1 {
    atom_local_anchor: RenderPoint,
    operations: Vec<CompactGroupRenderOpV1>,
}
impl CompactGroupRenderBatchV1 {
    pub fn new(
        atom_local_anchor: RenderPoint,
        operations: Vec<CompactGroupRenderOpV1>,
    ) -> Result<Self, RenderError> {
        validate_nonempty_z(&operations, CompactGroupRenderOpV1::z)?;
        Ok(Self {
            atom_local_anchor,
            operations,
        })
    }
    #[must_use]
    pub const fn atom_local_anchor(&self) -> RenderPoint {
        self.atom_local_anchor
    }
    #[must_use]
    pub fn operations(&self) -> &[CompactGroupRenderOpV1] {
        &self.operations
    }
    fn render_operations(&self) -> Vec<RenderOp> {
        self.operations
            .iter()
            .map(CompactGroupRenderOpV1::as_render_op)
            .collect()
    }
}
impl<'de> Deserialize<'de> for CompactGroupRenderBatchV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            atom_local_anchor: RenderPoint,
            operations: Vec<CompactGroupRenderOpV1>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.atom_local_anchor, wire.operations).map_err(serde::de::Error::custom)
    }
}

/// A scene-space bond operation with no text or atom-label facts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "kind",
    content = "operation",
    rename_all = "snake_case"
)]
pub enum BondRenderOpV1 {
    Line(LineOp),
    Path(PathOpV3),
    DoubleBondCarrierMark(DoubleBondCarrierMarkOp),
}
impl BondRenderOpV1 {
    fn z(&self) -> i32 {
        match self {
            Self::Line(op) => op.z(),
            Self::Path(op) => op.z(),
            Self::DoubleBondCarrierMark(op) => op.z(),
        }
    }
    fn as_render_op(&self) -> RenderOp {
        match self {
            Self::Line(op) => RenderOp::Line(op.clone()),
            Self::Path(op) => RenderOp::Path(op.clone()),
            Self::DoubleBondCarrierMark(op) => RenderOp::DoubleBondCarrierMark(op.clone()),
        }
    }
}
/// Structural scene-space attachment for one bond before visible-ink clipping.
///
/// This axis is semantic geometry: renderers validate and transport it, but do
/// not paint or hit-test it. `operations` owns the separately clipped ink.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BondAttachmentAxisV1 {
    start: RenderPoint,
    end: RenderPoint,
}

impl BondAttachmentAxisV1 {
    /// Construct a finite, nonzero structural bond axis.
    pub fn new(start: RenderPoint, end: RenderPoint) -> Result<Self, RenderError> {
        if start == end {
            return Err(RenderError::InvalidRequest(
                "bond attachment axis endpoints are coincident".to_owned(),
            ));
        }
        Ok(Self { start, end })
    }

    /// Return the first issued structural connection point.
    #[must_use]
    pub const fn start(self) -> RenderPoint {
        self.start
    }

    /// Return the second issued structural connection point.
    #[must_use]
    pub const fn end(self) -> RenderPoint {
        self.end
    }
}

impl<'de> Deserialize<'de> for BondAttachmentAxisV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            start: RenderPoint,
            end: RenderPoint,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.start, wire.end).map_err(serde::de::Error::custom)
    }
}

/// Scene-space bond content.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BondRenderBatchV1 {
    attachment_axis: BondAttachmentAxisV1,
    operations: Vec<BondRenderOpV1>,
}
impl BondRenderBatchV1 {
    pub fn new(
        attachment_axis: BondAttachmentAxisV1,
        operations: Vec<BondRenderOpV1>,
    ) -> Result<Self, RenderError> {
        validate_nonempty_z(&operations, BondRenderOpV1::z)?;
        Ok(Self {
            attachment_axis,
            operations,
        })
    }
    /// Return the issued structural axis before any visible-ink clipping.
    #[must_use]
    pub const fn attachment_axis(&self) -> BondAttachmentAxisV1 {
        self.attachment_axis
    }
    #[must_use]
    pub fn operations(&self) -> &[BondRenderOpV1] {
        &self.operations
    }
    fn render_operations(&self) -> Vec<RenderOp> {
        self.operations
            .iter()
            .map(BondRenderOpV1::as_render_op)
            .collect()
    }
    /// Convert private bond lowering output before it reaches the V4 contract.
    pub(crate) fn from_render_operations(
        attachment_axis: BondAttachmentAxisV1,
        operations: Vec<RenderOp>,
    ) -> Result<Self, RenderError> {
        let operations = operations
            .into_iter()
            .map(|operation| match operation {
                RenderOp::Line(operation) => Ok(BondRenderOpV1::Line(operation)),
                RenderOp::Path(operation) => Ok(BondRenderOpV1::Path(operation)),
                RenderOp::DoubleBondCarrierMark(operation) => {
                    Ok(BondRenderOpV1::DoubleBondCarrierMark(operation))
                }
                _ => Err(RenderError::InvalidRequest(
                    "bond content cannot carry atom-local operations".to_owned(),
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(attachment_axis, operations)
    }
}
impl<'de> Deserialize<'de> for BondRenderBatchV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            attachment_axis: BondAttachmentAxisV1,
            operations: Vec<BondRenderOpV1>,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.attachment_axis, wire.operations).map_err(serde::de::Error::custom)
    }
}

/// The only legal semantic content for a V4 batch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(
    deny_unknown_fields,
    tag = "kind",
    content = "content",
    rename_all = "snake_case"
)]
pub enum RenderBatchContentV4 {
    Atom(Box<AtomRenderBatchV1>),
    CompactGroup(CompactGroupRenderBatchV1),
    Bond(BondRenderBatchV1),
}

/// Replay-only coordinate interpretation derived from typed V4 content.
///
/// It is not serialized as an independent batch field: V4 content owns the
/// coordinate space, so no wire payload can contradict its semantic variant.
#[derive(Clone, Debug, PartialEq)]
pub enum BatchSpace {
    /// Atom and compact-group content is translated from its local anchor.
    AtomLocal { anchor: RenderPoint },
    /// Bond content is already in Ferrum scene coordinates.
    Scene,
}

/// Rust-selected display tier for complete target geometry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderDisplayLayerV1 {
    Ordinary,
    HaworthFrontStroke,
    HaworthFrontWedge,
}
impl RenderDisplayLayerV1 {
    #[must_use]
    pub const fn z_tier(self) -> i32 {
        match self {
            Self::Ordinary => 0,
            Self::HaworthFrontStroke => 1,
            Self::HaworthFrontWedge => 2,
        }
    }
}

/// Immutable target-specific V4 content with a derived operation replay view.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderBatchV4 {
    target: RenderTarget,
    paint_order: u32,
    display_layer: RenderDisplayLayerV1,
    content: RenderBatchContentV4,
}
impl RenderBatchV4 {
    #[cfg(test)]
    pub(crate) fn test_atom_label_from_facts(
        mask: Option<MaskOp>,
        facts: crate::AtomLabelFacts,
        font: crate::AtomLabelFontProfile,
        z: i32,
    ) -> Result<AtomLabelRenderV1, RenderError> {
        let environment = crate::FerrumFontEnvironmentV1::load()?;
        let metrics = crate::VerifiedTelexGlyphMetrics::new(&environment)?;
        let layout =
            crate::glyph_metrics::GlyphMetrics::layout_atom_label(&metrics, &facts, &font)?;
        let normalized = TextOp::new(
            RenderPoint::new(0.0, 0.0)?,
            layout.runs().to_vec(),
            font.face().clone(),
            font.size(),
            font.paint().clone(),
            z,
        )?;
        let core_index = layout.core_element_run_index();
        let core = InkBoundsV1::from_glyph_bounds(
            metrics.v1_centered_core_run_ink_bounds(&normalized, core_index as usize)?,
        );
        let full = InkBoundsV1::from_glyph_bounds(
            metrics.v1_atom_label_ink_bounds(&normalized, core_index as usize)?,
        );
        let bond_ink_clearance =
            crate::atom_bond::bond::NormalBondEndpointClipPolicy::label_clearance_for_font(
                font.size(),
            )
            .map_err(|issue| {
                RenderError::InvalidRequest(format!("test atom label clearance failed: {issue:?}"))
            })?
            .gap();
        AtomLabelRenderV1::new(mask, normalized, core_index, bond_ink_clearance, full, core)
    }

    #[cfg(test)]
    pub(crate) fn test_atom_target(
        target: RenderTarget,
        paint_order: u32,
        content: AtomRenderBatchV1,
    ) -> Self {
        Self {
            target,
            paint_order,
            display_layer: RenderDisplayLayerV1::Ordinary,
            content: RenderBatchContentV4::Atom(Box::new(content)),
        }
    }

    #[cfg(test)]
    pub(crate) fn test_compact_group_target(
        target: RenderTarget,
        paint_order: u32,
        content: CompactGroupRenderBatchV1,
    ) -> Self {
        Self {
            target,
            paint_order,
            display_layer: RenderDisplayLayerV1::Ordinary,
            content: RenderBatchContentV4::CompactGroup(content),
        }
    }
    pub(crate) fn atom(
        context: crate::render_target::RenderPlanEntryContextV1,
        content: AtomRenderBatchV1,
    ) -> Result<Self, RenderError> {
        Self::from_typed_context(context, RenderBatchContentV4::Atom(Box::new(content)))
    }
    pub(crate) fn compact_group(
        context: crate::render_target::RenderPlanEntryContextV1,
        content: CompactGroupRenderBatchV1,
    ) -> Result<Self, RenderError> {
        Self::from_typed_context(context, RenderBatchContentV4::CompactGroup(content))
    }
    pub(crate) fn bond(
        context: crate::render_target::RenderPlanEntryContextV1,
        content: BondRenderBatchV1,
    ) -> Result<Self, RenderError> {
        Self::from_typed_context(context, RenderBatchContentV4::Bond(content))
    }
    /// Construct a bond batch from a renderer-owned fixture or detached preview.
    pub(crate) fn bond_target(
        target: RenderTarget,
        paint_order: u32,
        content: BondRenderBatchV1,
    ) -> Self {
        Self {
            target,
            paint_order,
            display_layer: RenderDisplayLayerV1::Ordinary,
            content: RenderBatchContentV4::Bond(content),
        }
    }
    fn from_typed_context(
        context: crate::render_target::RenderPlanEntryContextV1,
        content: RenderBatchContentV4,
    ) -> Result<Self, RenderError> {
        let valid = matches!(
            (&content, context.record_id().kind()),
            (RenderBatchContentV4::Atom(_), RecordKind::Atom)
                | (RenderBatchContentV4::CompactGroup(_), RecordKind::Group)
                | (RenderBatchContentV4::Bond(_), RecordKind::Bond)
        );
        if !valid {
            return Err(RenderError::InvalidRequest(
                "render batch content kind must match source record kind".to_owned(),
            ));
        }
        Ok(Self {
            target: context.target().clone(),
            paint_order: context.paint_order(),
            display_layer: RenderDisplayLayerV1::Ordinary,
            content,
        })
    }
    #[must_use]
    pub fn with_display_layer(mut self, display_layer: RenderDisplayLayerV1) -> Self {
        self.display_layer = display_layer;
        self
    }
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }
    #[must_use]
    pub const fn paint_order(&self) -> u32 {
        self.paint_order
    }
    #[must_use]
    pub const fn display_layer(&self) -> RenderDisplayLayerV1 {
        self.display_layer
    }
    #[must_use]
    pub fn content(&self) -> &RenderBatchContentV4 {
        &self.content
    }
    /// Return the replay space that is mechanically determined by content.
    #[must_use]
    pub fn coordinate_space(&self) -> BatchSpace {
        match &self.content {
            RenderBatchContentV4::Atom(content) => BatchSpace::AtomLocal {
                anchor: content.atom_local_anchor(),
            },
            RenderBatchContentV4::CompactGroup(content) => BatchSpace::AtomLocal {
                anchor: content.atom_local_anchor(),
            },
            RenderBatchContentV4::Bond(_) => BatchSpace::Scene,
        }
    }
    /// Derive the legacy-neutral replay sequence from the sole typed content source.
    #[must_use]
    pub fn operations(&self) -> Vec<RenderOp> {
        match &self.content {
            RenderBatchContentV4::Atom(content) => content.operations(),
            RenderBatchContentV4::CompactGroup(content) => content.render_operations(),
            RenderBatchContentV4::Bond(content) => content.render_operations(),
        }
    }

    /// Visit the immutable replay sequence without allocating cloned operations.
    ///
    /// The order is derived from the one closed content variant: atom masks,
    /// labels, and decorations; then compact-group or bond operations.
    pub(crate) fn visit_operations<E>(
        &self,
        mut visitor: impl FnMut(RenderOperationRef<'_>) -> Result<(), E>,
    ) -> Result<(), E> {
        match &self.content {
            RenderBatchContentV4::Atom(content) => {
                if let Some(mask) = content.label().mask() {
                    visitor(RenderOperationRef::Mask(mask))?;
                }
                visitor(RenderOperationRef::Text(content.label().text()))?;
                for decoration in content.decorations() {
                    match decoration {
                        AtomDecorationRenderOpV1::Text(operation) => {
                            visitor(RenderOperationRef::Text(operation))?;
                        }
                        AtomDecorationRenderOpV1::Line(operation) => {
                            visitor(RenderOperationRef::Line(operation))?;
                        }
                        AtomDecorationRenderOpV1::Ellipse(operation) => {
                            visitor(RenderOperationRef::Ellipse(operation))?;
                        }
                    }
                }
            }
            RenderBatchContentV4::CompactGroup(content) => {
                for operation in &content.operations {
                    match operation {
                        CompactGroupRenderOpV1::Text(operation) => {
                            visitor(RenderOperationRef::Text(operation))?;
                        }
                        CompactGroupRenderOpV1::Line(operation) => {
                            visitor(RenderOperationRef::Line(operation))?;
                        }
                        CompactGroupRenderOpV1::Ellipse(operation) => {
                            visitor(RenderOperationRef::Ellipse(operation))?;
                        }
                    }
                }
            }
            RenderBatchContentV4::Bond(content) => {
                for operation in &content.operations {
                    match operation {
                        BondRenderOpV1::Line(operation) => {
                            visitor(RenderOperationRef::Line(operation))?;
                        }
                        BondRenderOpV1::Path(operation) => {
                            visitor(RenderOperationRef::Path(operation))?;
                        }
                        BondRenderOpV1::DoubleBondCarrierMark(operation) => {
                            visitor(RenderOperationRef::DoubleBondCarrierMark(operation))?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
impl<'de> Deserialize<'de> for RenderBatchV4 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            target: RenderTarget,
            paint_order: u32,
            display_layer: RenderDisplayLayerV1,
            content: RenderBatchContentV4,
        }
        let wire = Wire::deserialize(deserializer)?;
        let kind = match &wire.content {
            RenderBatchContentV4::Atom(_) => RecordKind::Atom,
            RenderBatchContentV4::CompactGroup(_) => RecordKind::Group,
            RenderBatchContentV4::Bond(_) => RecordKind::Bond,
        };
        let context = crate::render_target::RenderPlanEntryContextV1::new(
            wire.target,
            RecordId::new(
                kind,
                ferrum_core::Identifier::new("wire").map_err(serde::de::Error::custom)?,
            )
            .map_err(serde::de::Error::custom)?,
            wire.paint_order,
            None,
        );
        Self::from_typed_context(context, wire.content)
            .map(|batch| batch.with_display_layer(wire.display_layer))
            .map_err(serde::de::Error::custom)
    }
}

/// A complete immutable V4 response from one document-projection revision.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoleculeRenderPlanV4 {
    schema: RenderSchemaVersion,
    provenance: RenderProvenance,
    batches: Vec<RenderBatchV4>,
    issues: Vec<RenderIssue>,
}
impl MoleculeRenderPlanV4 {
    pub fn new(
        provenance: RenderProvenance,
        batches: Vec<RenderBatchV4>,
        issues: Vec<RenderIssue>,
    ) -> Result<Self, RenderError> {
        validate_plan(&batches, &issues)?;
        Ok(Self {
            schema: RenderSchemaVersion::V4,
            provenance,
            batches,
            issues,
        })
    }
    #[must_use]
    pub const fn schema(&self) -> RenderSchemaVersion {
        self.schema
    }
    #[must_use]
    pub const fn revision(&self) -> RenderRevision {
        self.provenance.revision()
    }
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    #[must_use]
    pub fn batches(&self) -> &[RenderBatchV4] {
        &self.batches
    }
    #[must_use]
    pub fn issues(&self) -> &[RenderIssue] {
        &self.issues
    }
    pub fn to_canonical_json(&self) -> Result<String, RenderError> {
        serde_json::to_string(self).map_err(|error| RenderError::Serialization(error.to_string()))
    }
    pub fn from_json(input: &str) -> Result<Self, RenderError> {
        serde_json::from_str(input).map_err(|error| RenderError::InvalidJson(error.to_string()))
    }
}
impl<'de> Deserialize<'de> for MoleculeRenderPlanV4 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            schema: RenderSchemaVersion,
            provenance: RenderProvenance,
            batches: Vec<RenderBatchV4>,
            issues: Vec<RenderIssue>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.schema != RenderSchemaVersion::V4 {
            return Err(serde::de::Error::custom("unsupported render-plan schema"));
        }
        Self::new(wire.provenance, wire.batches, wire.issues).map_err(serde::de::Error::custom)
    }
}

fn validate_nonempty_z<T>(operations: &[T], z: impl Fn(&T) -> i32) -> Result<(), RenderError> {
    if operations.is_empty() {
        return Err(RenderError::InvalidRequest(
            "render batch requires operations".to_owned(),
        ));
    }
    if operations.windows(2).any(|pair| z(&pair[0]) >= z(&pair[1])) {
        return Err(RenderError::InvalidRequest(
            "render batch operations must have strictly increasing z".to_owned(),
        ));
    }
    Ok(())
}
fn validate_plan(batches: &[RenderBatchV4], issues: &[RenderIssue]) -> Result<(), RenderError> {
    let mut targets = HashSet::new();
    let mut orders = HashSet::new();
    let mut previous = None;
    for batch in batches {
        if !targets.insert(batch.target().document_object_id().clone())
            || !orders.insert(batch.paint_order())
        {
            return Err(RenderError::InvalidRequest(
                "render plan has duplicate target or paint order".to_owned(),
            ));
        }
        if previous.is_some_and(|value| batch.paint_order() <= value) {
            return Err(RenderError::InvalidRequest(
                "render plan batches must have strictly increasing paint order".to_owned(),
            ));
        }
        previous = Some(batch.paint_order());
    }
    previous = None;
    for issue in issues {
        issue.validate()?;
        if !targets.insert(issue.target().document_object_id().clone())
            || !orders.insert(issue.paint_order())
        {
            return Err(RenderError::InvalidRequest(
                "render plan has duplicate target or paint order".to_owned(),
            ));
        }
        if previous.is_some_and(|value| issue.paint_order() <= value) {
            return Err(RenderError::InvalidRequest(
                "render plan issues must have strictly increasing paint order".to_owned(),
            ));
        }
        previous = Some(issue.paint_order());
    }
    Ok(())
}
fn canon(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}
