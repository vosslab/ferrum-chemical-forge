//! Atom-label and normal single-, double-, and triple-bond render-plan generation.

use std::collections::{HashMap, HashSet};

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::{Point2, Vector2};

use crate::{
    BatchSpace, EllipseOp, FontFace, GlyphBounds, GlyphMetrics, LineOp, MaskOp, MoleculeRenderPlan,
    Paint, PositiveFinite, RenderBatch, RenderError, RenderIssue, RenderIssueKind, RenderOp,
    RenderPoint, RenderProvenance, RenderTarget, TextOp, TextScript,
};

/// Complete atom-label presentation facts required by this render slice.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomLabelFontProfile {
    face: FontFace,
    size: PositiveFinite,
    paint: Paint,
    label_mask: Option<Paint>,
}

impl AtomLabelFontProfile {
    /// Construct an exact label presentation profile without renderer defaults.
    #[must_use]
    pub const fn new(face: FontFace, size: PositiveFinite, paint: Paint) -> Self {
        Self {
            face,
            size,
            paint,
            label_mask: None,
        }
    }

    /// Return the exact requested face.
    #[must_use]
    pub const fn face(&self) -> &FontFace {
        &self.face
    }

    /// Return the exact requested text size.
    #[must_use]
    pub const fn size(&self) -> PositiveFinite {
        self.size
    }

    /// Return the exact requested label paint.
    #[must_use]
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }

    /// Attach an exact opaque mask; absence means transparent with no mask operation.
    #[must_use]
    pub fn with_label_mask(mut self, paint: Paint) -> Self {
        self.label_mask = Some(paint);
        self
    }
}

/// Source facts that produce a structured atom label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtomLabelFacts {
    element: String,
    formal_charge: i8,
    explicit_hydrogens: u8,
}

/// Explicit presentation facts for one visible persistent atom number.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomNumberLabelFacts {
    number: u64,
    origin: RenderPoint,
    font: AtomLabelFontProfile,
}

/// Closed semantic category for one atom-attached mark.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomMarkRenderKind {
    Plus,
    Minus,
    Radical,
    Biradical,
    Electronpair,
    DottedElectronpair,
    PzOrbital,
}

/// Explicit atom-local geometry and paint for one persistent mark.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomMarkRenderFacts {
    kind: AtomMarkRenderKind,
    origin: RenderPoint,
    angle_degrees: f64,
    size: PositiveFinite,
    draw_circle: bool,
    line_width: PositiveFinite,
    paint: Paint,
}

impl AtomMarkRenderFacts {
    /// Construct one complete mark without renderer defaults.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: AtomMarkRenderKind,
        origin: RenderPoint,
        angle_degrees: f64,
        size: PositiveFinite,
        draw_circle: bool,
        line_width: PositiveFinite,
        paint: Paint,
    ) -> Result<Self, RenderError> {
        if !angle_degrees.is_finite() {
            return Err(RenderError::InvalidRequest(
                "atom mark angle must be finite".to_owned(),
            ));
        }
        Ok(Self {
            kind,
            origin,
            angle_degrees,
            size,
            draw_circle,
            line_width,
            paint,
        })
    }
}

impl AtomNumberLabelFacts {
    /// Construct one positive decimal annotation with explicit geometry and paint.
    pub fn new(
        number: u64,
        origin: RenderPoint,
        font: AtomLabelFontProfile,
    ) -> Result<Self, RenderError> {
        if number == 0 {
            return Err(RenderError::InvalidRequest(
                "atom number must be a positive integer".to_owned(),
            ));
        }
        Ok(Self {
            number,
            origin,
            font,
        })
    }
}

impl AtomLabelFacts {
    /// Construct validated atom-label facts.
    pub fn new(
        element: impl Into<String>,
        formal_charge: i8,
        explicit_hydrogens: u8,
    ) -> Result<Self, RenderError> {
        let element = element.into();
        let mut characters = element.chars();
        let Some(first) = characters.next() else {
            return Err(RenderError::InvalidRequest(
                "atom element must not be blank".to_owned(),
            ));
        };
        if !first.is_ascii_uppercase()
            || characters.clone().count() > 2
            || !characters.all(|character| character.is_ascii_lowercase())
        {
            return Err(RenderError::InvalidRequest(
                "atom element must use one uppercase letter followed by at most two lowercase letters"
                    .to_owned(),
            ));
        }
        Ok(Self {
            element,
            formal_charge,
            explicit_hydrogens,
        })
    }

    /// Return the declared element symbol.
    #[must_use]
    pub fn element(&self) -> &str {
        &self.element
    }

    /// Return the formal charge supplied by the source model.
    #[must_use]
    pub const fn formal_charge(&self) -> i8 {
        self.formal_charge
    }

    /// Return the source model's explicit hydrogen count.
    #[must_use]
    pub const fn explicit_hydrogens(&self) -> u8 {
        self.explicit_hydrogens
    }

    /// Return ordered source text segments before a metric provider lays them out.
    pub(crate) fn text_pieces(&self) -> Vec<(String, TextScript)> {
        let mut runs = vec![(self.element.clone(), TextScript::Baseline)];
        if self.explicit_hydrogens > 0 {
            runs.push(("H".to_owned(), TextScript::Baseline));
            if self.explicit_hydrogens > 1 {
                runs.push((self.explicit_hydrogens.to_string(), TextScript::Subscript));
            }
        }
        if self.formal_charge != 0 {
            let sign = if self.formal_charge > 0 { '+' } else { '-' };
            let magnitude = self.formal_charge.unsigned_abs();
            let charge = if magnitude == 1 {
                sign.to_string()
            } else {
                format!("{magnitude}{sign}")
            };
            runs.push((charge, TextScript::Superscript));
        }
        runs
    }
}

/// Deliberate visibility supplied by the authoritative source projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetVisibility {
    /// The target is part of this visible render projection.
    Visible,
    /// The target must remain absent with an explanatory non-fatal diagnostic.
    Hidden { reason: String },
}

impl TargetVisibility {
    fn issue(&self, noun: &str) -> Option<RenderIssueKind> {
        match self {
            Self::Visible => None,
            Self::Hidden { reason } if !reason.trim().is_empty() => {
                Some(RenderIssueKind::UnsupportedFeature {
                    feature: format!("invisible {noun}: {reason}"),
                })
            }
            Self::Hidden { .. } => Some(RenderIssueKind::UnsupportedFeature {
                feature: format!("invisible {noun}"),
            }),
        }
    }
}

/// Bond style carried by the source projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BondStyle {
    /// The sole supported bond style in this vertical slice.
    NormalSingle,
    /// A parallel double bond.
    Double,
    /// A parallel triple bond.
    Triple,
    /// An aromatic bond.
    Aromatic,
    /// A solid stereochemical wedge.
    SolidWedge,
    /// A hashed stereochemical wedge.
    HashedWedge,
    /// A dashed bond.
    Dashed,
    /// An exact source depiction that V1 intentionally cannot lower.
    Unsupported { detail: String },
}

impl BondStyle {
    fn unsupported_name(&self) -> Option<&str> {
        match self {
            Self::NormalSingle | Self::Double | Self::Triple => None,
            Self::Aromatic => Some("aromatic bond"),
            Self::SolidWedge => Some("solid wedge bond"),
            Self::HashedWedge => Some("hashed wedge bond"),
            Self::Dashed => Some("dashed bond"),
            Self::Unsupported { detail } => Some(detail.as_str()),
        }
    }
}

/// An atom with explicit source identity, finite position, and label facts.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomRenderTarget {
    target: RenderTarget,
    position: RenderPoint,
    label: AtomLabelFacts,
    visibility: TargetVisibility,
    font: Option<AtomLabelFontProfile>,
    number_label: Option<AtomNumberLabelFacts>,
    marks: Vec<AtomMarkRenderFacts>,
}

impl AtomRenderTarget {
    /// Construct a valid atom target for this render slice.
    pub fn new(
        target: RenderTarget,
        position: RenderPoint,
        label: AtomLabelFacts,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if target.record_id().kind() != RecordKind::Atom {
            return Err(RenderError::InvalidRequest(
                "atom render target requires an atom RecordId".to_owned(),
            ));
        }
        Ok(Self {
            target,
            position,
            label,
            visibility,
            font: None,
            number_label: None,
            marks: Vec::new(),
        })
    }

    /// Return the durable target and source order.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }

    /// Attach source-resolved presentation facts for this target only.
    #[must_use]
    pub fn with_font_profile(mut self, font: AtomLabelFontProfile) -> Self {
        self.font = Some(font);
        self
    }

    /// Attach one fully resolved visible atom-number annotation.
    #[must_use]
    pub fn with_number_label(mut self, number_label: AtomNumberLabelFacts) -> Self {
        self.number_label = Some(number_label);
        self
    }

    /// Attach complete persistent atom-mark facts in source order.
    #[must_use]
    pub fn with_marks(mut self, marks: Vec<AtomMarkRenderFacts>) -> Self {
        self.marks = marks;
        self
    }
}

/// A bond with explicit endpoint atom identities and source style facts.
#[derive(Clone, Debug, PartialEq)]
pub struct BondRenderTarget {
    target: RenderTarget,
    first_atom: RecordId,
    second_atom: RecordId,
    style: BondStyle,
    visibility: TargetVisibility,
    appearance: Option<BondLineAppearance>,
}

#[derive(Clone, Debug, PartialEq)]
struct BondLineAppearance {
    stroke_width: PositiveFinite,
    lane_spacing: PositiveFinite,
    paint: Paint,
}

impl BondRenderTarget {
    /// Construct a valid bond target for this render slice.
    pub fn new(
        target: RenderTarget,
        first_atom: RecordId,
        second_atom: RecordId,
        style: BondStyle,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if target.record_id().kind() != RecordKind::Bond {
            return Err(RenderError::InvalidRequest(
                "bond render target requires a bond RecordId".to_owned(),
            ));
        }
        if first_atom.kind() != RecordKind::Atom || second_atom.kind() != RecordKind::Atom {
            return Err(RenderError::InvalidRequest(
                "bond endpoints require atom RecordIds".to_owned(),
            ));
        }
        Ok(Self {
            target,
            first_atom,
            second_atom,
            style,
            visibility,
            appearance: None,
        })
    }

    /// Return the durable target and source order.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        &self.target
    }

    /// Attach source-resolved stroke and parallel-lane facts for this bond only.
    #[must_use]
    pub fn with_appearance(
        mut self,
        stroke_width: PositiveFinite,
        lane_spacing: PositiveFinite,
        paint: Paint,
    ) -> Self {
        self.appearance = Some(BondLineAppearance {
            stroke_width,
            lane_spacing,
            paint,
        });
        self
    }
}

/// A complete, order-explicit request for atom labels and normal covalent bonds.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomBondRenderRequest {
    provenance: RenderProvenance,
    atoms: Vec<AtomRenderTarget>,
    bonds: Vec<BondRenderTarget>,
    font: AtomLabelFontProfile,
    line_width: PositiveFinite,
    bond_lane_spacing: PositiveFinite,
    line_paint: Paint,
}

impl AtomBondRenderRequest {
    /// Construct a request whose target identities and source orders are unique.
    pub fn new(
        provenance: RenderProvenance,
        atoms: Vec<AtomRenderTarget>,
        bonds: Vec<BondRenderTarget>,
        font: AtomLabelFontProfile,
        line_width: PositiveFinite,
        bond_lane_spacing: PositiveFinite,
        line_paint: Paint,
    ) -> Result<Self, RenderError> {
        let mut identifiers = HashSet::new();
        let mut source_orders = HashSet::new();
        for target in atoms
            .iter()
            .map(AtomRenderTarget::target)
            .chain(bonds.iter().map(BondRenderTarget::target))
        {
            if !identifiers.insert(target.record_id().clone()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate targets".to_owned(),
                ));
            }
            if !source_orders.insert(target.source_order()) {
                return Err(RenderError::InvalidRequest(
                    "atom-bond request has duplicate source order".to_owned(),
                ));
            }
        }
        Ok(Self {
            provenance,
            atoms,
            bonds,
            font,
            line_width,
            bond_lane_spacing,
            line_paint,
        })
    }
}

/// Build the total ordered batch-or-issue partition for this render slice.
pub fn build_atom_bond_plan<M: GlyphMetrics>(
    request: &AtomBondRenderRequest,
    metrics: &M,
) -> Result<MoleculeRenderPlan, RenderError> {
    let mut batches = Vec::new();
    let mut issues = Vec::new();
    let mut atoms = HashMap::new();

    for atom in &request.atoms {
        let target = atom.target.clone();
        let outcome = atom.visibility.issue("atom target").map_or_else(
            || build_atom_batch(atom, atom.font.as_ref().unwrap_or(&request.font), metrics),
            |kind| Ok(Err(kind)),
        );
        match outcome? {
            Ok((batch, bounds)) => {
                atoms.insert(
                    target.record_id().clone(),
                    AtomGeometry {
                        position: render_point_to_geometry(atom.position)?,
                        bounds,
                    },
                );
                batches.push(batch);
            }
            Err(kind) => issues.push(RenderIssue::new(target, kind)?),
        }
    }

    for bond in &request.bonds {
        let target = bond.target.clone();
        let outcome = if let Some(kind) = bond.visibility.issue("bond target") {
            Err(kind)
        } else if let Some(style) = bond.style.unsupported_name() {
            Err(RenderIssueKind::UnsupportedFeature {
                feature: style.to_owned(),
            })
        } else {
            let (stroke_width, lane_spacing, paint) = bond.appearance.as_ref().map_or_else(
                || {
                    (
                        request.line_width,
                        request.bond_lane_spacing,
                        request.line_paint.clone(),
                    )
                },
                |appearance| {
                    (
                        appearance.stroke_width,
                        appearance.lane_spacing,
                        appearance.paint.clone(),
                    )
                },
            );
            build_bond_batch(bond, &atoms, stroke_width, lane_spacing, paint)
        };
        match outcome {
            Ok(batch) => batches.push(batch),
            Err(kind) => issues.push(RenderIssue::new(target, kind)?),
        }
    }

    batches.sort_by_key(|batch| batch.target().source_order());
    issues.sort_by_key(|issue| issue.target().source_order());
    MoleculeRenderPlan::new(request.provenance, batches, issues)
}

struct AtomGeometry {
    position: Point2,
    bounds: GlyphBounds,
}

fn build_atom_batch<M: GlyphMetrics>(
    atom: &AtomRenderTarget,
    font: &AtomLabelFontProfile,
    metrics: &M,
) -> Result<Result<(RenderBatch, GlyphBounds), RenderIssueKind>, RenderError> {
    let layout = match metrics.layout_atom_label(&atom.label, font) {
        Ok(layout) => layout,
        Err(error) => {
            return Ok(Err(RenderIssueKind::UnrenderableTarget {
                reason: format!("atom label metrics unavailable: {error}"),
            }));
        }
    };
    let operation = TextOp::new(
        RenderPoint::new(0.0, 0.0)?,
        layout.runs().to_vec(),
        font.face.clone(),
        font.size,
        font.paint.clone(),
        30,
    )?;
    let mut operations = Vec::new();
    if let Some(paint) = font.label_mask.clone() {
        let width = PositiveFinite::new(layout.bounds().max_x() - layout.bounds().min_x())?;
        let height = PositiveFinite::new(layout.bounds().max_y() - layout.bounds().min_y())?;
        operations.push(RenderOp::Mask(MaskOp::new(
            RenderPoint::new(layout.bounds().min_x(), layout.bounds().min_y())?,
            width,
            height,
            paint,
            20,
        )?));
    }
    operations.push(RenderOp::Text(operation));
    if let Some(number) = &atom.number_label {
        let run = metrics.layout_atom_number(number.number, &number.font)?;
        operations.push(RenderOp::Text(TextOp::new(
            number.origin,
            vec![run],
            number.font.face.clone(),
            number.font.size,
            number.font.paint.clone(),
            40,
        )?));
    }
    let mut next_mark_z = 50;
    for mark in &atom.marks {
        append_mark_operations(mark, &mut operations, &mut next_mark_z)?;
    }
    let batch = RenderBatch::new(
        atom.target.clone(),
        BatchSpace::AtomLocal {
            anchor: atom.position,
        },
        operations,
    )?;
    Ok(Ok((batch, layout.bounds())))
}

fn append_mark_operations(
    mark: &AtomMarkRenderFacts,
    operations: &mut Vec<RenderOp>,
    next_z: &mut i32,
) -> Result<(), RenderError> {
    let radius = mark.size.get() / 2.0;
    match mark.kind {
        AtomMarkRenderKind::Plus | AtomMarkRenderKind::Minus => {
            if mark.draw_circle {
                operations.push(RenderOp::Ellipse(EllipseOp::new(
                    mark.origin,
                    PositiveFinite::new(radius)?,
                    PositiveFinite::new(radius)?,
                    0.0,
                    Some(mark.line_width),
                    Some(mark.paint.clone()),
                    None,
                    take_z(next_z)?,
                )?));
            }
            let half = radius * 0.6;
            push_line(
                operations,
                RenderPoint::new(mark.origin.x() - half, mark.origin.y())?,
                RenderPoint::new(mark.origin.x() + half, mark.origin.y())?,
                mark.line_width,
                mark.paint.clone(),
                take_z(next_z)?,
            )?;
            if mark.kind == AtomMarkRenderKind::Plus {
                push_line(
                    operations,
                    RenderPoint::new(mark.origin.x(), mark.origin.y() - half)?,
                    RenderPoint::new(mark.origin.x(), mark.origin.y() + half)?,
                    mark.line_width,
                    mark.paint.clone(),
                    take_z(next_z)?,
                )?;
            }
        }
        AtomMarkRenderKind::Radical => push_filled_dot(
            operations,
            mark.origin,
            radius,
            mark.paint.clone(),
            take_z(next_z)?,
        )?,
        AtomMarkRenderKind::Biradical | AtomMarkRenderKind::DottedElectronpair => {
            let dot_radius = (radius * 0.3).max(1.0);
            let spacing = (radius * 0.6).max(dot_radius);
            let (perpendicular_x, perpendicular_y) = perpendicular(mark.angle_degrees, spacing);
            for direction in [-1.0, 1.0] {
                push_filled_dot(
                    operations,
                    RenderPoint::new(
                        mark.origin.x() + perpendicular_x * direction,
                        mark.origin.y() + perpendicular_y * direction,
                    )?,
                    dot_radius,
                    mark.paint.clone(),
                    take_z(next_z)?,
                )?;
            }
        }
        AtomMarkRenderKind::Electronpair => {
            let (perpendicular_x, perpendicular_y) = perpendicular(mark.angle_degrees, radius);
            push_line(
                operations,
                RenderPoint::new(
                    mark.origin.x() - perpendicular_x,
                    mark.origin.y() - perpendicular_y,
                )?,
                RenderPoint::new(
                    mark.origin.x() + perpendicular_x,
                    mark.origin.y() + perpendicular_y,
                )?,
                mark.line_width,
                mark.paint.clone(),
                take_z(next_z)?,
            )?;
        }
        AtomMarkRenderKind::PzOrbital => {
            let lobe_width = radius * 0.45;
            let lobe_height = radius * 0.65;
            let center_offset = radius * 0.38;
            let radians = mark.angle_degrees.to_radians();
            for direction in [-1.0, 1.0] {
                let local_y = center_offset * direction;
                let center = RenderPoint::new(
                    mark.origin.x() - local_y * radians.sin(),
                    mark.origin.y() + local_y * radians.cos(),
                )?;
                operations.push(RenderOp::Ellipse(EllipseOp::new(
                    center,
                    PositiveFinite::new(lobe_width)?,
                    PositiveFinite::new(lobe_height)?,
                    mark.angle_degrees,
                    Some(mark.line_width),
                    Some(mark.paint.clone()),
                    None,
                    take_z(next_z)?,
                )?));
            }
        }
    }
    Ok(())
}

fn perpendicular(angle_degrees: f64, length: f64) -> (f64, f64) {
    let radians = angle_degrees.to_radians();
    (-radians.sin() * length, radians.cos() * length)
}

fn push_filled_dot(
    operations: &mut Vec<RenderOp>,
    center: RenderPoint,
    radius: f64,
    paint: Paint,
    z: i32,
) -> Result<(), RenderError> {
    operations.push(RenderOp::Ellipse(EllipseOp::new(
        center,
        PositiveFinite::new(radius)?,
        PositiveFinite::new(radius)?,
        0.0,
        None,
        None,
        Some(paint),
        z,
    )?));
    Ok(())
}

fn push_line(
    operations: &mut Vec<RenderOp>,
    start: RenderPoint,
    end: RenderPoint,
    width: PositiveFinite,
    paint: Paint,
    z: i32,
) -> Result<(), RenderError> {
    operations.push(RenderOp::Line(LineOp::new(start, end, width, paint, z)?));
    Ok(())
}

fn take_z(next_z: &mut i32) -> Result<i32, RenderError> {
    let current = *next_z;
    *next_z = next_z.checked_add(1).ok_or_else(|| {
        RenderError::InvalidRequest("atom mark operation z-order is exhausted".to_owned())
    })?;
    Ok(current)
}

fn build_bond_batch(
    bond: &BondRenderTarget,
    atoms: &HashMap<RecordId, AtomGeometry>,
    stroke_width: PositiveFinite,
    lane_spacing: PositiveFinite,
    paint: Paint,
) -> Result<RenderBatch, RenderIssueKind> {
    let Some(first) = atoms.get(&bond.first_atom) else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "first bond endpoint has no renderable visible atom label".to_owned(),
        });
    };
    let Some(second) = atoms.get(&bond.second_atom) else {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "second bond endpoint has no renderable visible atom label".to_owned(),
        });
    };
    let vector = second.position - first.position;
    let length = vector.length();
    if !length.is_finite() || length == 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "bond endpoints are coincident or not representable".to_owned(),
        });
    }
    let direction = Vector2::new(vector.x() / length, vector.y() / length).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("bond direction is not representable: {error}"),
        }
    })?;
    // Retain the established CDML depiction convention: `bond_width` is the
    // centered double-lane separation, while triple outer lanes use 70% of it.
    const TRIPLE_OUTER_LANE_FACTOR: f64 = 0.7;
    let offsets: &[f64] = match &bond.style {
        BondStyle::NormalSingle => &[0.0],
        BondStyle::Double => &[-0.5, 0.5],
        BondStyle::Triple => &[-TRIPLE_OUTER_LANE_FACTOR, 0.0, TRIPLE_OUTER_LANE_FACTOR],
        _ => unreachable!("unsupported styles are excluded before bond geometry"),
    };
    let perpendicular = direction.perpendicular_left();
    let line_context = BondLineContext {
        first,
        second,
        direction,
        perpendicular,
        length,
    };
    let mut operations = Vec::with_capacity(offsets.len());
    for (index, factor) in offsets.iter().enumerate() {
        let offset = lane_spacing.get() * *factor;
        if !offset.is_finite() {
            return Err(RenderIssueKind::UnrenderableTarget {
                reason: "bond line spacing is not representable".to_owned(),
            });
        }
        let line = build_bond_line(
            &line_context,
            offset,
            stroke_width,
            paint.clone(),
            10 + i32::try_from(index).expect("bond line count fits i32"),
        )?;
        operations.push(RenderOp::Line(line));
    }
    RenderBatch::new(bond.target.clone(), BatchSpace::Scene, operations).map_err(|error| {
        RenderIssueKind::UnrenderableTarget {
            reason: format!("bond batch is not renderable: {error}"),
        }
    })
}

struct BondLineContext<'a> {
    first: &'a AtomGeometry,
    second: &'a AtomGeometry,
    direction: Vector2,
    perpendicular: Vector2,
    length: f64,
}

fn build_bond_line(
    context: &BondLineContext<'_>,
    offset: f64,
    width: PositiveFinite,
    paint: Paint,
    z: i32,
) -> Result<LineOp, RenderIssueKind> {
    let local_offset = Vector2::new(
        context.perpendicular.x() * offset,
        context.perpendicular.y() * offset,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond line offset is not representable: {error}"),
    })?;
    let reverse = negated(context.direction)?;
    let first_clip = clip_distance(context.first.bounds, context.direction, local_offset)?;
    let second_clip = clip_distance(context.second.bounds, reverse, local_offset)?;
    let remaining_length = context.length - first_clip - second_clip;
    if !remaining_length.is_finite() || remaining_length <= 0.0 {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "label clipping leaves no positive visible bond segment".to_owned(),
        });
    }
    let start = context
        .first
        .position
        .offset(context.perpendicular, offset)
        .and_then(|point| point.offset(context.direction, first_clip))
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond start is not representable: {error}"),
        })?;
    let end = context
        .second
        .position
        .offset(context.perpendicular, offset)
        .and_then(|point| point.offset(reverse, second_clip))
        .map_err(|error| RenderIssueKind::UnrenderableTarget {
            reason: format!("bond end is not representable: {error}"),
        })?;
    LineOp::new(
        geometry_to_render_point(start)?,
        geometry_to_render_point(end)?,
        width,
        paint,
        z,
    )
    .map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("clipped bond is not renderable: {error}"),
    })
}

fn clip_distance(
    bounds: GlyphBounds,
    direction: Vector2,
    origin: Vector2,
) -> Result<f64, RenderIssueKind> {
    let x = ray_slab(bounds.min_x(), bounds.max_x(), origin.x(), direction.x());
    let y = ray_slab(bounds.min_y(), bounds.max_y(), origin.y(), direction.y());
    let near = x.0.max(y.0);
    let far = x.1.min(y.1);
    if far < near || far < 0.0 {
        return Ok(0.0);
    }
    let distance = far.max(0.0);
    if !distance.is_finite() {
        return Err(RenderIssueKind::UnrenderableTarget {
            reason: "glyph clipping distance is not finite".to_owned(),
        });
    }
    Ok(distance)
}

fn ray_slab(minimum: f64, maximum: f64, origin: f64, direction: f64) -> (f64, f64) {
    if direction == 0.0 {
        return if origin < minimum || origin > maximum {
            (f64::INFINITY, f64::NEG_INFINITY)
        } else {
            (f64::NEG_INFINITY, f64::INFINITY)
        };
    }
    let first = (minimum - origin) / direction;
    let second = (maximum - origin) / direction;
    (first.min(second), first.max(second))
}

fn negated(vector: Vector2) -> Result<Vector2, RenderIssueKind> {
    Vector2::new(-vector.x(), -vector.y()).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("bond direction is not representable: {error}"),
    })
}

fn render_point_to_geometry(point: RenderPoint) -> Result<Point2, RenderError> {
    Point2::new(point.x(), point.y()).map_err(|error| {
        RenderError::InvalidRequest(format!("render point is invalid geometry: {error}"))
    })
}

fn geometry_to_render_point(point: Point2) -> Result<RenderPoint, RenderIssueKind> {
    RenderPoint::new(point.x(), point.y()).map_err(|error| RenderIssueKind::UnrenderableTarget {
        reason: format!("derived bond point is not finite: {error}"),
    })
}
