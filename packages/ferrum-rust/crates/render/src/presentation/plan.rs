//! Pure renderer planning for immutable direct-root presentation stacks.
//!
//! This module owns the display plan and its painted bounds. It accepts a
//! frozen lower projection only; session state, CDML, mutation candidates, and
//! admission receipts remain outside `ferrum-render`.

use crate::{
    DocumentPlusRenderV1, DocumentTextRenderV1, DocumentVectorOpV1, DocumentVectorRootV1,
    FerrumFontEnvironmentV1, PathCommandV1, RenderError, RenderPoint, VerifiedTelexGlyphMetrics,
};
use ferrum_document_projection::{
    PlusProjectionV1, Point3V1, PositiveFiniteV1, PresentationArrowPreviewRequestV1,
    PresentationFactProvenanceV1, PresentationFillV1, PresentationFontFaceV1, PresentationFontV1,
    PresentationRecordKindV1, PresentationRootProjectionV1, PresentationStackProjectionV1,
    PresentationTargetV1, ProjectionLocalObjectKeyV1, Rgb24V1,
};

/// Closed schema identifier for renderer-owned presentation delivery plans.
pub const PRESENTATION_RENDER_PLAN_SCHEMA_V1: &str = "ferrum-presentation-render-plan-v1";

/// Finite scene-space bounds calculated from renderer-issued operations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PresentationRenderBoundsV1 {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl PresentationRenderBoundsV1 {
    fn new(left: f64, top: f64, right: f64, bottom: f64) -> Result<Self, RenderError> {
        if ![left, top, right, bottom].into_iter().all(f64::is_finite)
            || left > right
            || top > bottom
        {
            return Err(RenderError::InvalidRequest(
                "presentation render bounds must be finite and ordered".to_owned(),
            ));
        }
        Ok(Self {
            left,
            top,
            right,
            bottom,
        })
    }

    /// Return the left painted scene coordinate.
    #[must_use]
    pub const fn left(self) -> f64 {
        self.left
    }

    /// Return the top painted scene coordinate.
    #[must_use]
    pub const fn top(self) -> f64 {
        self.top
    }

    /// Return the right painted scene coordinate.
    #[must_use]
    pub const fn right(self) -> f64 {
        self.right
    }

    /// Return the bottom painted scene coordinate.
    #[must_use]
    pub const fn bottom(self) -> f64 {
        self.bottom
    }
}

/// One target-preserving renderer outcome in direct-root paint order.
#[derive(Clone, Debug, PartialEq)]
pub enum PresentationRenderRootV1 {
    /// Renderer-neutral vector operations for a geometric root.
    Vector {
        target: PresentationTargetV1,
        vector: DocumentVectorRootV1,
        bounds: PresentationRenderBoundsV1,
    },
    /// Verified Telex operations for one plus root.
    Plus {
        render: DocumentPlusRenderV1,
        bounds: PresentationRenderBoundsV1,
    },
    /// Verified Telex operations for one Text root.
    Text {
        render: DocumentTextRenderV1,
        bounds: PresentationRenderBoundsV1,
    },
}

impl PresentationRenderRootV1 {
    /// Return the target whose source order owns this issued render root.
    #[must_use]
    pub fn target(&self) -> &PresentationTargetV1 {
        match self {
            Self::Vector { target, .. } => target,
            Self::Plus { render, .. } => render.target(),
            Self::Text { render, .. } => render.target(),
        }
    }

    /// Return renderer-calculated painted bounds for this root.
    #[must_use]
    pub const fn bounds(&self) -> PresentationRenderBoundsV1 {
        match self {
            Self::Vector { bounds, .. } | Self::Plus { bounds, .. } | Self::Text { bounds, .. } => {
                *bounds
            }
        }
    }

    /// Return geometric operations when this root is vector-backed.
    #[must_use]
    pub fn vector(&self) -> Option<&DocumentVectorRootV1> {
        match self {
            Self::Vector { vector, .. } => Some(vector),
            Self::Plus { .. } | Self::Text { .. } => None,
        }
    }
}

/// A complete immutable renderer plan for one immutable presentation stack.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationRenderPlanV1 {
    revision: u64,
    digest: [u8; 32],
    roots: Vec<PresentationRenderRootV1>,
}

impl PresentationRenderPlanV1 {
    fn new(
        revision: u64,
        digest: [u8; 32],
        roots: Vec<PresentationRenderRootV1>,
    ) -> Result<Self, RenderError> {
        if roots
            .windows(2)
            .any(|pair| pair[0].target().source_order() >= pair[1].target().source_order())
        {
            return Err(RenderError::InvalidRequest(
                "presentation render roots must use strictly increasing source order".to_owned(),
            ));
        }
        Ok(Self {
            revision,
            digest,
            roots,
        })
    }

    /// Return the fixed renderer-owned delivery schema.
    ///
    /// Every
    /// construction route through this private constructor publishes the same
    /// fixed delivery grammar.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        PRESENTATION_RENDER_PLAN_SCHEMA_V1
    }

    /// Return the immutable source revision that this plan renders.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Return the immutable source digest that this plan renders.
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Return target-preserving roots in direct-root source order.
    #[must_use]
    pub fn roots(&self) -> &[PresentationRenderRootV1] {
        &self.roots
    }
}

/// Render one frozen presentation stack into target-preserving renderer output.
///
/// The result is a pure owned plan. It neither reads retained document state
/// nor grants authority to mutate, publish, or admit a document candidate.
pub fn render_presentation_stack_v1(
    stack: &PresentationStackProjectionV1,
) -> Result<PresentationRenderPlanV1, RenderError> {
    let environment = FerrumFontEnvironmentV1::load()?;
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)?;
    let mut roots = Vec::new();
    roots
        .try_reserve(stack.roots().len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for root in stack.roots() {
        roots.push(render_root(root, &metrics)?);
    }
    PresentationRenderPlanV1::new(stack.revision(), *stack.digest(), roots)
}

/// Lower one semantic arrow preview through the same vector primitives as a
/// committed presentation root.
///
/// The plan uses synthetic provenance because the caller's opaque gesture owns
/// revision and digest fencing; this pure renderer value owns only paint
/// operations and renderer-calculated bounds.
pub fn lower_arrow_preview_v1(
    request: &PresentationArrowPreviewRequestV1,
) -> Result<PresentationRenderPlanV1, RenderError> {
    let vector = super::vector::lower_arrow_projection_v1(request.arrow())?;
    let bounds = vector_bounds(&vector)?;
    PresentationRenderPlanV1::new(
        0,
        [0; 32],
        vec![PresentationRenderRootV1::Vector {
            target: request.arrow().target().clone(),
            vector,
            bounds,
        }],
    )
}

/// Lower one identifier-free standard Plus preview through the ordinary
/// verified-Telex presentation path.
///
/// The returned plan contains a synthetic local target only because the shared
/// presentation-plan grammar requires one. It has neither a durable nor a
/// source identifier, and it carries no session, mutation, or transition
/// authority.
pub fn lower_standard_plus_preview_v1(
    anchor: RenderPoint,
) -> Result<PresentationRenderPlanV1, RenderError> {
    let environment = FerrumFontEnvironmentV1::load()?;
    let metrics = VerifiedTelexGlyphMetrics::new(&environment)?;
    let plus = standard_plus_projection(anchor);
    let render = DocumentPlusRenderV1::from_projection(&plus, &metrics)?;
    let bounds = text_bounds(render.anchor(), render.bounds())?;
    PresentationRenderPlanV1::new(
        0,
        [0; 32],
        vec![PresentationRenderRootV1::Plus { render, bounds }],
    )
}

fn standard_plus_projection(anchor: RenderPoint) -> PlusProjectionV1 {
    let target = PresentationTargetV1::try_new(
        None,
        ProjectionLocalObjectKeyV1::from_path_components(&[0])
            .expect("preview target has a nonempty local path"),
        None,
        0,
        PresentationRecordKindV1::Plus,
    )
    .expect("synthetic preview target has coherent local identity");
    let font = PresentationFontV1::try_new(
        PresentationFontFaceV1::TelexRegularV1,
        PresentationFactProvenanceV1::Builtin,
        PositiveFiniteV1::new(14.0).expect("built-in Plus font size is positive"),
        PresentationFactProvenanceV1::Builtin,
        Rgb24V1::new("#000000").expect("built-in Plus colour is valid"),
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("built-in Plus font facts are coherent");
    let background = PresentationFillV1::try_new(None, PresentationFactProvenanceV1::Builtin)
        .expect("built-in Plus background facts are coherent");
    PlusProjectionV1::try_new(
        target,
        Point3V1::new(anchor.x(), anchor.y(), 0.0).expect("render point is finite"),
        font,
        background,
    )
    .expect("synthetic Plus projection is coherent")
}

fn render_root(
    root: &PresentationRootProjectionV1,
    metrics: &VerifiedTelexGlyphMetrics,
) -> Result<PresentationRenderRootV1, RenderError> {
    match root {
        PresentationRootProjectionV1::Plus { plus } => {
            let render = DocumentPlusRenderV1::from_projection(plus, metrics)?;
            let bounds = text_bounds(render.anchor(), render.bounds())?;
            Ok(PresentationRenderRootV1::Plus { render, bounds })
        }
        PresentationRootProjectionV1::Text { text } => {
            let render = DocumentTextRenderV1::from_projection(text, metrics)?;
            let bounds = text_bounds(render.anchor(), render.bounds())?;
            Ok(PresentationRenderRootV1::Text { render, bounds })
        }
        _ => {
            let vector = super::vector::lower_presentation_vector_v1(root)?;
            let bounds = vector_bounds(&vector)?;
            Ok(PresentationRenderRootV1::Vector {
                target: root.target().clone(),
                vector,
                bounds,
            })
        }
    }
}

fn text_bounds(
    anchor: RenderPoint,
    bounds: crate::PresentationTextBoundsV1,
) -> Result<PresentationRenderBoundsV1, RenderError> {
    PresentationRenderBoundsV1::new(
        checked(anchor.x() + bounds.left())?,
        checked(anchor.y() + bounds.top())?,
        checked(anchor.x() + bounds.right())?,
        checked(anchor.y() + bounds.bottom())?,
    )
}

fn vector_bounds(vector: &DocumentVectorRootV1) -> Result<PresentationRenderBoundsV1, RenderError> {
    let mut bounds = BoundsAccumulator::default();
    for operation in vector.operations() {
        operation_bounds(operation, &mut bounds)?;
    }
    bounds.finish()
}

fn operation_bounds(
    operation: &DocumentVectorOpV1,
    bounds: &mut BoundsAccumulator,
) -> Result<(), RenderError> {
    match operation {
        DocumentVectorOpV1::Path {
            commands, stroke, ..
        } => {
            path_bounds(commands, bounds)?;
            if let Some(stroke) = stroke {
                bounds.expand(checked(stroke.width().get() * stroke.miter_limit() / 2.0)?)?;
            }
        }
        DocumentVectorOpV1::Ellipse {
            center,
            radius_x,
            radius_y,
            stroke,
            ..
        } => {
            let extension = stroke
                .as_ref()
                .map(|value| value.width().get() / 2.0)
                .unwrap_or(0.0);
            bounds.include_rect(
                checked(center.x() - radius_x.get() - extension)?,
                checked(center.y() - radius_y.get() - extension)?,
                checked(center.x() + radius_x.get() + extension)?,
                checked(center.y() + radius_y.get() + extension)?,
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document_projection::{
        ArrowHeadShapeV1, ArrowProjectionKindV1, CurvedTerminalArrowKindV1, Point3V1,
        PositiveFiniteV1, PresentationFactProvenanceV1, PresentationStrokeV1, Rgb24V1,
    };

    fn point(x: f64, y: f64) -> Point3V1 {
        Point3V1::new(x, y, 0.0).expect("finite preview point")
    }

    fn stroke() -> PresentationStrokeV1 {
        PresentationStrokeV1::new(
            Rgb24V1::new("#000000").expect("closed test color"),
            PresentationFactProvenanceV1::Builtin,
            PositiveFiniteV1::new(1.0).expect("positive test width"),
            PresentationFactProvenanceV1::Builtin,
        )
        .expect("closed test stroke")
    }

    #[test]
    fn arrow_preview_matches_committed_vector_operations_and_bounds_for_each_family() {
        let cases = [
            (
                vec![point(0.0, 0.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::Normal {
                    head_shape: ArrowHeadShapeV1::default_authored(),
                    start_head: false,
                    end_head: true,
                },
            ),
            (
                vec![point(0.0, 0.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::Equilibrium,
            ),
            (
                vec![point(0.0, 0.0), point(20.0, 12.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::CurvedTerminal {
                    terminal_kind: CurvedTerminalArrowKindV1::Electron,
                },
            ),
            (
                vec![point(0.0, 0.0), point(20.0, 12.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::CurvedTerminal {
                    terminal_kind: CurvedTerminalArrowKindV1::Retro,
                },
            ),
            (
                vec![point(0.0, 0.0), point(20.0, 12.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::CurvedTerminal {
                    terminal_kind: CurvedTerminalArrowKindV1::Normal,
                },
            ),
            (
                vec![point(0.0, 0.0), point(20.0, 12.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::CurvedEquilibrium,
            ),
        ];

        for (points, kind) in cases {
            let request = PresentationArrowPreviewRequestV1::new(points, kind, stroke())
                .expect("closed semantic preview request");
            let preview = lower_arrow_preview_v1(&request).expect("preview plan");
            let committed = super::super::vector::lower_presentation_vector_v1(
                &PresentationRootProjectionV1::Arrow {
                    arrow: request.arrow().clone(),
                },
            )
            .expect("committed vector root");
            let root = preview.roots().first().expect("one preview root");

            assert_eq!(root.vector(), Some(&committed));
            assert_eq!(
                root.bounds(),
                vector_bounds(&committed).expect("committed bounds")
            );
        }
    }

    #[test]
    fn terminal_arrow_families_share_renderer_visual_operations_and_bounds() {
        let families = [
            CurvedTerminalArrowKindV1::Electron,
            CurvedTerminalArrowKindV1::Retro,
            CurvedTerminalArrowKindV1::Normal,
        ];
        let mut visuals = families.into_iter().map(|terminal_kind| {
            let request = PresentationArrowPreviewRequestV1::new(
                vec![point(0.0, 0.0), point(20.0, 12.0), point(60.0, 0.0)],
                ArrowProjectionKindV1::CurvedTerminal { terminal_kind },
                stroke(),
            )
            .expect("closed terminal-arrow preview request");
            let plan = lower_arrow_preview_v1(&request).expect("terminal-arrow preview plan");
            let root = plan.roots().first().expect("one terminal-arrow root");
            (root.vector().cloned(), root.bounds())
        });
        let expected = visuals.next().expect("one terminal-arrow family");

        for visual in visuals {
            assert_eq!(visual, expected);
        }
    }
}

fn path_bounds(
    commands: &[PathCommandV1],
    bounds: &mut BoundsAccumulator,
) -> Result<(), RenderError> {
    let mut current = None;
    for command in commands {
        match *command {
            PathCommandV1::MoveTo(point) => {
                bounds.include(point)?;
                current = Some(point);
            }
            PathCommandV1::LineTo(point) => {
                bounds.include(point)?;
                current = Some(point);
            }
            PathCommandV1::CubicTo {
                control_1,
                control_2,
                end,
            } => {
                let start = current.ok_or_else(|| {
                    RenderError::InvalidRequest("cubic path lost its current point".to_owned())
                })?;
                cubic_bounds(start, control_1, control_2, end, bounds)?;
                current = Some(end);
            }
            PathCommandV1::Close => {}
        }
    }
    Ok(())
}

fn cubic_bounds(
    start: RenderPoint,
    control_1: RenderPoint,
    control_2: RenderPoint,
    end: RenderPoint,
    bounds: &mut BoundsAccumulator,
) -> Result<(), RenderError> {
    for point in [start, end] {
        bounds.include(point)?;
    }
    let mut parameters = cubic_extrema(start.x(), control_1.x(), control_2.x(), end.x());
    parameters.extend(cubic_extrema(
        start.y(),
        control_1.y(),
        control_2.y(),
        end.y(),
    ));
    for parameter in parameters {
        if (0.0..=1.0).contains(&parameter) {
            bounds.include(RenderPoint::new(
                cubic_value(start.x(), control_1.x(), control_2.x(), end.x(), parameter),
                cubic_value(start.y(), control_1.y(), control_2.y(), end.y(), parameter),
            )?)?;
        }
    }
    Ok(())
}

fn cubic_extrema(start: f64, control_1: f64, control_2: f64, end: f64) -> Vec<f64> {
    let a = -start + 3.0 * control_1 - 3.0 * control_2 + end;
    let b = 3.0 * start - 6.0 * control_1 + 3.0 * control_2;
    let c = -3.0 * start + 3.0 * control_1;
    if a.abs() <= f64::EPSILON {
        return (b.abs() > f64::EPSILON)
            .then_some(-c / (2.0 * b))
            .into_iter()
            .collect();
    }
    let discriminant = 4.0 * b * b - 12.0 * a * c;
    if discriminant < 0.0 {
        return Vec::new();
    }
    let root = discriminant.sqrt();
    vec![(-2.0 * b + root) / (6.0 * a), (-2.0 * b - root) / (6.0 * a)]
}

fn cubic_value(start: f64, control_1: f64, control_2: f64, end: f64, t: f64) -> f64 {
    let inverse = 1.0 - t;
    inverse.powi(3) * start
        + 3.0 * inverse.powi(2) * t * control_1
        + 3.0 * inverse * t.powi(2) * control_2
        + t.powi(3) * end
}

#[derive(Default)]
struct BoundsAccumulator {
    left: Option<f64>,
    top: Option<f64>,
    right: Option<f64>,
    bottom: Option<f64>,
}

impl BoundsAccumulator {
    fn include(&mut self, point: RenderPoint) -> Result<(), RenderError> {
        self.include_rect(point.x(), point.y(), point.x(), point.y())
    }

    fn include_rect(
        &mut self,
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
    ) -> Result<(), RenderError> {
        PresentationRenderBoundsV1::new(left, top, right, bottom)?;
        self.left = Some(self.left.map_or(left, |value| value.min(left)));
        self.top = Some(self.top.map_or(top, |value| value.min(top)));
        self.right = Some(self.right.map_or(right, |value| value.max(right)));
        self.bottom = Some(self.bottom.map_or(bottom, |value| value.max(bottom)));
        Ok(())
    }

    fn expand(&mut self, extension: f64) -> Result<(), RenderError> {
        let (Some(left), Some(top), Some(right), Some(bottom)) =
            (self.left, self.top, self.right, self.bottom)
        else {
            return Err(RenderError::InvalidRequest(
                "stroked presentation operation has no geometry".to_owned(),
            ));
        };
        self.left = Some(checked(left - extension)?);
        self.top = Some(checked(top - extension)?);
        self.right = Some(checked(right + extension)?);
        self.bottom = Some(checked(bottom + extension)?);
        Ok(())
    }

    fn finish(self) -> Result<PresentationRenderBoundsV1, RenderError> {
        let (Some(left), Some(top), Some(right), Some(bottom)) =
            (self.left, self.top, self.right, self.bottom)
        else {
            return Err(RenderError::InvalidRequest(
                "presentation vector root has no renderer geometry".to_owned(),
            ));
        };
        PresentationRenderBoundsV1::new(left, top, right, bottom)
    }
}

fn checked(value: f64) -> Result<f64, RenderError> {
    value.is_finite().then_some(value).ok_or_else(|| {
        RenderError::InvalidRequest("presentation render bounds are not finite".to_owned())
    })
}
