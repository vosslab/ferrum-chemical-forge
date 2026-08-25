//! Atom-local label, number, and mark lowering.

use ferrum_core::RecordKind;

use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    BatchSpace, EllipseOp, FontFace, GlyphBounds, GlyphMetrics, LineOp, MaskOp, Paint,
    PositiveFinite, RenderBatch, RenderError, RenderIssueKind, RenderOp, RenderPoint, RenderTarget,
    TextOp, TextScript,
};
use ferrum_document_model::is_admitted_atom_symbol_v1;

use super::TargetVisibility;

#[derive(Clone, Debug, PartialEq)]
pub struct AtomLabelFontProfile {
    face: FontFace,
    size: PositiveFinite,
    pub(super) paint: Paint,
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
        if !is_admitted_atom_symbol_v1(&element) {
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
/// An atom with explicit source identity, finite position, and label facts.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomRenderTarget {
    pub(super) context: RenderPlanEntryContextV1,
    pub(super) position: RenderPoint,
    label: AtomLabelFacts,
    pub(super) visibility: TargetVisibility,
    pub(super) font: Option<AtomLabelFontProfile>,
    number_label: Option<AtomNumberLabelFacts>,
    marks: Vec<AtomMarkRenderFacts>,
}
impl AtomRenderTarget {
    /// Construct a valid atom target for this render slice.
    pub(crate) fn new(
        context: RenderPlanEntryContextV1,
        position: RenderPoint,
        label: AtomLabelFacts,
        visibility: TargetVisibility,
    ) -> Result<Self, RenderError> {
        if context.record_id().kind() != RecordKind::Atom {
            return Err(RenderError::InvalidRequest(
                "atom render target requires an atom RecordId".to_owned(),
            ));
        }
        Ok(Self {
            context,
            position,
            label,
            visibility,
            font: None,
            number_label: None,
            marks: Vec::new(),
        })
    }

    /// Return the durable target.
    #[must_use]
    pub fn target(&self) -> &RenderTarget {
        self.context.target()
    }

    pub(super) const fn context(&self) -> &RenderPlanEntryContextV1 {
        &self.context
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

pub(super) fn build_atom_batch<M: GlyphMetrics>(
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
    let batch = RenderBatch::from_context(
        atom.context.clone(),
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
            let (x, y) = perpendicular(mark.angle_degrees, spacing);
            for direction in [-1.0, 1.0] {
                push_filled_dot(
                    operations,
                    RenderPoint::new(
                        mark.origin.x() + x * direction,
                        mark.origin.y() + y * direction,
                    )?,
                    dot_radius,
                    mark.paint.clone(),
                    take_z(next_z)?,
                )?;
            }
        }
        AtomMarkRenderKind::Electronpair => {
            let (x, y) = perpendicular(mark.angle_degrees, radius);
            push_line(
                operations,
                RenderPoint::new(mark.origin.x() - x, mark.origin.y() - y)?,
                RenderPoint::new(mark.origin.x() + x, mark.origin.y() + y)?,
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
