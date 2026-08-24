//! Lower already-verified presentation projections to renderer-neutral vector roots.
//!
//! This boundary consumes only one frozen document projection.  It neither rereads
//! CDML nor derives replacement geometry from a toolkit-specific representation.

use crate::{
    DocumentVectorOpV1, DocumentVectorRootV1, Paint, PathCommandV1, PositiveFinite, RenderError,
    RenderPoint, Rgb24, StrokeV1,
};
use ferrum_document_projection::{
    ArrowHeadShapeV1, ArrowPathV1, ArrowProjectionKindV1, ArrowProjectionV1, BoxShapeProjectionV1,
    CurvedTerminalArrowKindV1, Point3V1, PolygonProjectionV1, PolylineProjectionV1,
    PresentationFillV1, PresentationRootProjectionV1, PresentationStrokeV1,
};

/// Lower one semantic non-text presentation root into renderer-issued geometry.
pub fn lower_presentation_vector_v1(
    root: &PresentationRootProjectionV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    match root {
        PresentationRootProjectionV1::Arrow { arrow } => lower_arrow_projection_v1(arrow),
        PresentationRootProjectionV1::Polyline { polyline }
        | PresentationRootProjectionV1::Wavy { polyline } => polyline_root(polyline),
        PresentationRootProjectionV1::RoundBracket { polyline } => round_bracket_root(polyline),
        PresentationRootProjectionV1::Rectangle { shape }
        | PresentationRootProjectionV1::Square { shape } => rectangle_root(shape),
        PresentationRootProjectionV1::Oval { shape }
        | PresentationRootProjectionV1::Circle { shape } => ellipse_root(shape),
        PresentationRootProjectionV1::Polygon { polygon } => polygon_root(polygon),
        PresentationRootProjectionV1::Plus { .. } | PresentationRootProjectionV1::Text { .. } => {
            Err(RenderError::InvalidRequest(
                "text presentation root cannot be lowered as a vector".to_owned(),
            ))
        }
    }
}

pub(crate) fn lower_arrow_projection_v1(
    arrow: &ArrowProjectionV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    let stroke = stroke(arrow.stroke())?;
    match arrow.kind() {
        ArrowProjectionKindV1::Normal {
            head_shape,
            start_head,
            end_head,
        } => normal_arrow_root(
            &stroke,
            arrow.source_path(),
            *head_shape,
            *start_head,
            *end_head,
        ),
        ArrowProjectionKindV1::Equilibrium => equilibrium_arrow_root(&stroke, arrow.source_path()),
        ArrowProjectionKindV1::CurvedEquilibrium => {
            curved_equilibrium_arrow_root(&stroke, arrow.source_path())
        }
        // These authoring families remain semantically distinct in CDML, but
        // intentionally share one terminal shaft-and-head visual policy.
        ArrowProjectionKindV1::CurvedTerminal {
            terminal_kind:
                CurvedTerminalArrowKindV1::Electron
                | CurvedTerminalArrowKindV1::Retro
                | CurvedTerminalArrowKindV1::Normal,
        } => curved_terminal_arrow_root(&stroke, arrow.source_path()),
    }
}

const EQUILIBRIUM_HALF_SPACING_PT_V1: f64 = 4.0;
const EQUILIBRIUM_HEAD_LINE_INSET_PT_V1: f64 = 8.0;
const EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1: f64 = 10.0;
const EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1: f64 = 3.0;
const EQUILIBRIUM_MINIMUM_LENGTH_PT_V1: f64 = 20.0;
const NORMAL_ARROW_VISIBLE_SHAFT_FRACTION_V1: f64 = 0.5;

fn normal_arrow_root(
    stroke: &StrokeV1,
    path: &ArrowPathV1,
    shape: ArrowHeadShapeV1,
    start_head: bool,
    end_head: bool,
) -> Result<DocumentVectorRootV1, RenderError> {
    let [start, end] = path.points() else {
        return invalid_arrow("normal arrow requires two source points");
    };
    let (ux, uy, length) = unit(*start, *end, "normal arrow")?;
    let shape = normal_arrow_shape_for_span(shape, start_head, end_head, length)?;
    let axis_start = offset(
        *start,
        ux * if start_head { shape.line_inset() } else { 0.0 },
        uy * if start_head { shape.line_inset() } else { 0.0 },
    )?;
    let axis_end = offset(
        *end,
        -ux * if end_head { shape.line_inset() } else { 0.0 },
        -uy * if end_head { shape.line_inset() } else { 0.0 },
    )?;
    let mut heads = Vec::with_capacity(usize::from(start_head) + usize::from(end_head));
    if start_head {
        heads.push(head(*start, -ux, -uy, shape)?);
    }
    if end_head {
        heads.push(head(*end, ux, uy, shape)?);
    }
    arrow_geometry_root(stroke, &[vec![axis_start, axis_end]], &heads)
}

fn normal_arrow_shape_for_span(
    shape: ArrowHeadShapeV1,
    start_head: bool,
    end_head: bool,
    source_span: f64,
) -> Result<ArrowHeadShapeV1, RenderError> {
    let head_count = f64::from(start_head as u8 + end_head as u8);
    let authored_inset = shape.line_inset() * head_count;
    if head_count == 0.0 || source_span > authored_inset {
        return Ok(shape);
    }
    // Reserve the configured fraction of a short source span for a visible
    // shaft after the renderer scales head insets and dimensions together.
    let scale = source_span * (1.0 - NORMAL_ARROW_VISIBLE_SHAFT_FRACTION_V1) / authored_inset;
    ArrowHeadShapeV1::new(
        shape.line_inset() * scale,
        shape.total_length() * scale,
        shape.half_width() * scale,
    )
    .ok_or_else(|| {
        RenderError::InvalidRequest(
            "normal arrow source span cannot produce finite scaled head geometry".to_owned(),
        )
    })
}

fn equilibrium_arrow_root(
    stroke: &StrokeV1,
    path: &ArrowPathV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    let [start, end] = path.points() else {
        return invalid_arrow("equilibrium arrow requires two source points");
    };
    let (ux, uy, length) = unit(*start, *end, "equilibrium arrow")?;
    if length < EQUILIBRIUM_MINIMUM_LENGTH_PT_V1 {
        return invalid_arrow("equilibrium arrow source span is below its fixed geometry minimum");
    }
    let (px, py) = (-uy, ux);
    let lower_start = offset(
        *start,
        -px * EQUILIBRIUM_HALF_SPACING_PT_V1,
        -py * EQUILIBRIUM_HALF_SPACING_PT_V1,
    )?;
    let lower_end = offset(
        *end,
        -px * EQUILIBRIUM_HALF_SPACING_PT_V1,
        -py * EQUILIBRIUM_HALF_SPACING_PT_V1,
    )?;
    let upper_start = offset(
        *start,
        px * EQUILIBRIUM_HALF_SPACING_PT_V1,
        py * EQUILIBRIUM_HALF_SPACING_PT_V1,
    )?;
    let upper_end = offset(
        *end,
        px * EQUILIBRIUM_HALF_SPACING_PT_V1,
        py * EQUILIBRIUM_HALF_SPACING_PT_V1,
    )?;
    let lower_axis_start = offset(
        lower_start,
        ux * EQUILIBRIUM_HEAD_LINE_INSET_PT_V1,
        uy * EQUILIBRIUM_HEAD_LINE_INSET_PT_V1,
    )?;
    let upper_axis_end = offset(
        upper_end,
        -ux * EQUILIBRIUM_HEAD_LINE_INSET_PT_V1,
        -uy * EQUILIBRIUM_HEAD_LINE_INSET_PT_V1,
    )?;
    let shape = ArrowHeadShapeV1::new(
        EQUILIBRIUM_HEAD_LINE_INSET_PT_V1,
        EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
        EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
    )
    .expect("fixed equilibrium head dimensions are valid");
    arrow_geometry_root(
        stroke,
        &[
            vec![lower_axis_start, lower_end],
            vec![upper_start, upper_axis_end],
        ],
        &[
            head(lower_start, -ux, -uy, shape)?,
            head(upper_end, ux, uy, shape)?,
        ],
    )
}

fn curved_terminal_arrow_root(
    stroke: &StrokeV1,
    path: &ArrowPathV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    let [start, control, end] = path.points() else {
        return invalid_arrow("curved terminal arrow requires three source points");
    };
    let (ux, uy, _) = unit(*control, *end, "curved terminal arrow terminal tangent")?;
    let shape = ArrowHeadShapeV1::new(8.0, 10.0, 3.0)
        .expect("fixed curved-terminal head dimensions are valid");
    cubic_arrow_geometry_root(
        stroke,
        &[quadratic_cubic(*start, *control, *end)?],
        &[head(*end, ux, uy, shape)?],
        "curved terminal",
    )
}

fn curved_equilibrium_arrow_root(
    stroke: &StrokeV1,
    path: &ArrowPathV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    let [start, control, end] = path.points() else {
        return invalid_arrow("curved equilibrium arrow requires three source points");
    };
    let (ux, uy, length) = unit(*start, *end, "curved equilibrium arrow")?;
    if length < EQUILIBRIUM_MINIMUM_LENGTH_PT_V1 {
        return invalid_arrow(
            "curved equilibrium arrow source span is below its fixed geometry minimum",
        );
    }
    let start_tangent = unit(*start, *control, "curved equilibrium arrow start tangent")?;
    let end_tangent = unit(*control, *end, "curved equilibrium arrow end tangent")?;
    if start_tangent.0 * ux + start_tangent.1 * uy < std::f64::consts::FRAC_1_SQRT_2
        || end_tangent.0 * ux + end_tangent.1 * uy < std::f64::consts::FRAC_1_SQRT_2
    {
        return invalid_arrow(
            "curved equilibrium arrow endpoint tangents must stay forward and within 45 degrees of the chord",
        );
    }
    let (nx, ny) = (-uy, ux);
    let lower = [
        offset(
            *start,
            -nx * EQUILIBRIUM_HALF_SPACING_PT_V1,
            -ny * EQUILIBRIUM_HALF_SPACING_PT_V1,
        )?,
        offset(
            *control,
            -nx * EQUILIBRIUM_HALF_SPACING_PT_V1,
            -ny * EQUILIBRIUM_HALF_SPACING_PT_V1,
        )?,
        offset(
            *end,
            -nx * EQUILIBRIUM_HALF_SPACING_PT_V1,
            -ny * EQUILIBRIUM_HALF_SPACING_PT_V1,
        )?,
    ];
    let upper = [
        offset(
            *start,
            nx * EQUILIBRIUM_HALF_SPACING_PT_V1,
            ny * EQUILIBRIUM_HALF_SPACING_PT_V1,
        )?,
        offset(
            *control,
            nx * EQUILIBRIUM_HALF_SPACING_PT_V1,
            ny * EQUILIBRIUM_HALF_SPACING_PT_V1,
        )?,
        offset(
            *end,
            nx * EQUILIBRIUM_HALF_SPACING_PT_V1,
            ny * EQUILIBRIUM_HALF_SPACING_PT_V1,
        )?,
    ];
    let shape = ArrowHeadShapeV1::new(
        EQUILIBRIUM_HEAD_LINE_INSET_PT_V1,
        EQUILIBRIUM_HEAD_TOTAL_LENGTH_PT_V1,
        EQUILIBRIUM_HEAD_HALF_WIDTH_PT_V1,
    )
    .expect("fixed equilibrium head dimensions are valid");
    cubic_arrow_geometry_root(
        stroke,
        &[
            quadratic_cubic(lower[0], lower[1], lower[2])?,
            quadratic_cubic(upper[0], upper[1], upper[2])?,
        ],
        &[
            head(lower[0], -start_tangent.0, -start_tangent.1, shape)?,
            head(upper[2], end_tangent.0, end_tangent.1, shape)?,
        ],
        "curved equilibrium",
    )
}

fn unit(start: Point3V1, end: Point3V1, label: &str) -> Result<(f64, f64, f64), RenderError> {
    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let length = dx.hypot(dy);
    (length.is_finite() && length > 0.0)
        .then_some((dx / length, dy / length, length))
        .ok_or_else(|| {
            RenderError::InvalidRequest(format!("{label} requires a noncollapsed finite span"))
        })
}
fn offset(point: Point3V1, x: f64, y: f64) -> Result<Point3V1, RenderError> {
    Point3V1::new(point.x() + x, point.y() + y, point.z()).map_err(|_| {
        RenderError::InvalidRequest("renderer-derived arrow geometry is not finite".to_owned())
    })
}
fn quadratic_cubic(
    start: Point3V1,
    control: Point3V1,
    end: Point3V1,
) -> Result<Vec<Point3V1>, RenderError> {
    Ok(vec![
        start,
        offset(
            start,
            (control.x() - start.x()) * (2.0 / 3.0),
            (control.y() - start.y()) * (2.0 / 3.0),
        )?,
        offset(
            end,
            (control.x() - end.x()) * (2.0 / 3.0),
            (control.y() - end.y()) * (2.0 / 3.0),
        )?,
        end,
    ])
}
fn head(
    tip: Point3V1,
    ux: f64,
    uy: f64,
    shape: ArrowHeadShapeV1,
) -> Result<Vec<Point3V1>, RenderError> {
    let (px, py) = (-uy, ux);
    Ok(vec![
        tip,
        offset(
            tip,
            -ux * shape.total_length() + px * shape.half_width(),
            -uy * shape.total_length() + py * shape.half_width(),
        )?,
        offset(tip, -ux * shape.line_inset(), -uy * shape.line_inset())?,
        offset(
            tip,
            -ux * shape.total_length() - px * shape.half_width(),
            -uy * shape.total_length() - py * shape.half_width(),
        )?,
    ])
}
fn invalid_arrow<T>(detail: &str) -> Result<T, RenderError> {
    Err(RenderError::InvalidRequest(detail.to_owned()))
}

fn cubic_arrow_geometry_root(
    stroke: &StrokeV1,
    axes: &[Vec<Point3V1>],
    heads: &[Vec<Point3V1>],
    arrow_family: &str,
) -> Result<DocumentVectorRootV1, RenderError> {
    let mut operations = Vec::new();
    operations
        .try_reserve(axes.len() + usize::from(!heads.is_empty()))
        .map_err(|_| RenderError::ResourceExhausted)?;
    for axis in axes {
        let [start, control_1, control_2, end] = axis.as_slice() else {
            return Err(RenderError::InvalidRequest(format!(
                "{arrow_family} arrow axis must contain one cubic segment"
            )));
        };
        operations.push(DocumentVectorOpV1::path(
            vec![
                PathCommandV1::MoveTo(point(*start)?),
                PathCommandV1::CubicTo {
                    control_1: point(*control_1)?,
                    control_2: point(*control_2)?,
                    end: point(*end)?,
                },
            ],
            Some(stroke.clone()),
            None,
        )?);
    }
    if !heads.is_empty() {
        let mut commands = Vec::new();
        commands
            .try_reserve(
                heads
                    .len()
                    .checked_mul(5)
                    .ok_or(RenderError::ResourceExhausted)?,
            )
            .map_err(|_| RenderError::ResourceExhausted)?;
        for head in heads {
            closed_points(&mut commands, head)?;
        }
        operations.push(DocumentVectorOpV1::path(
            commands,
            None,
            Some(stroke.paint().clone()),
        )?);
    }
    DocumentVectorRootV1::new(operations)
}

fn arrow_geometry_root(
    stroke: &StrokeV1,
    axes: &[Vec<Point3V1>],
    heads: &[Vec<Point3V1>],
) -> Result<DocumentVectorRootV1, RenderError> {
    let mut operations = Vec::new();
    operations
        .try_reserve(axes.len() + usize::from(!heads.is_empty()))
        .map_err(|_| RenderError::ResourceExhausted)?;
    for axis in axes {
        operations.push(DocumentVectorOpV1::path(
            open_path(axis)?,
            Some(stroke.clone()),
            None,
        )?);
    }

    if !heads.is_empty() {
        let mut commands = Vec::new();
        let command_count = heads
            .len()
            .checked_mul(5)
            .ok_or(RenderError::ResourceExhausted)?;
        commands
            .try_reserve(command_count)
            .map_err(|_| RenderError::ResourceExhausted)?;
        for head in heads {
            closed_points(&mut commands, head)?;
        }
        operations.push(DocumentVectorOpV1::path(
            commands,
            None,
            Some(stroke.paint().clone()),
        )?);
    }
    DocumentVectorRootV1::new(operations)
}

fn polyline_root(polyline: &PolylineProjectionV1) -> Result<DocumentVectorRootV1, RenderError> {
    DocumentVectorRootV1::new(vec![DocumentVectorOpV1::path(
        open_path(polyline.path().points())?,
        Some(stroke(polyline.stroke())?),
        None,
    )?])
}

fn round_bracket_root(
    polyline: &PolylineProjectionV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    let [start, control_1, control_2, end] = polyline.path().points() else {
        return Err(RenderError::InvalidRequest(
            "round bracket projection requires exactly four issued points".to_owned(),
        ));
    };
    DocumentVectorRootV1::new(vec![DocumentVectorOpV1::path(
        vec![
            PathCommandV1::MoveTo(point(*start)?),
            PathCommandV1::CubicTo {
                control_1: point(*control_1)?,
                control_2: point(*control_2)?,
                end: point(*end)?,
            },
        ],
        Some(stroke(polyline.stroke())?),
        None,
    )?])
}

fn rectangle_root(shape: &BoxShapeProjectionV1) -> Result<DocumentVectorRootV1, RenderError> {
    let bounds = shape.bounds();
    let commands = vec![
        PathCommandV1::MoveTo(RenderPoint::new(bounds.left(), bounds.top())?),
        PathCommandV1::LineTo(RenderPoint::new(bounds.right(), bounds.top())?),
        PathCommandV1::LineTo(RenderPoint::new(bounds.right(), bounds.bottom())?),
        PathCommandV1::LineTo(RenderPoint::new(bounds.left(), bounds.bottom())?),
        PathCommandV1::Close,
    ];
    DocumentVectorRootV1::new(vec![DocumentVectorOpV1::path(
        commands,
        Some(stroke(shape.stroke())?),
        fill(shape.fill())?,
    )?])
}

fn ellipse_root(shape: &BoxShapeProjectionV1) -> Result<DocumentVectorRootV1, RenderError> {
    let bounds = shape.bounds();
    let width = bounds.right() - bounds.left();
    let height = bounds.bottom() - bounds.top();
    let radius_x = PositiveFinite::new(width / 2.0)?;
    let radius_y = PositiveFinite::new(height / 2.0)?;
    let center = RenderPoint::new(bounds.left() + width / 2.0, bounds.top() + height / 2.0)?;
    DocumentVectorRootV1::new(vec![DocumentVectorOpV1::ellipse(
        center,
        radius_x,
        radius_y,
        Some(stroke(shape.stroke())?),
        fill(shape.fill())?,
    )?])
}

fn polygon_root(polygon: &PolygonProjectionV1) -> Result<DocumentVectorRootV1, RenderError> {
    let mut commands = Vec::new();
    commands
        .try_reserve(polygon.path().points().len() + 1)
        .map_err(|_| RenderError::ResourceExhausted)?;
    closed_points(&mut commands, polygon.path().points())?;
    DocumentVectorRootV1::new(vec![DocumentVectorOpV1::path(
        commands,
        Some(stroke(polygon.stroke())?),
        fill(polygon.fill())?,
    )?])
}

fn open_path(points: &[Point3V1]) -> Result<Vec<PathCommandV1>, RenderError> {
    let Some((first, rest)) = points.split_first() else {
        return Err(RenderError::InvalidRequest(
            "presentation path requires at least one issued point".to_owned(),
        ));
    };
    let mut commands = Vec::new();
    commands
        .try_reserve(points.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    commands.push(PathCommandV1::MoveTo(point(*first)?));
    for source in rest {
        commands.push(PathCommandV1::LineTo(point(*source)?));
    }
    Ok(commands)
}

fn closed_points(
    commands: &mut Vec<PathCommandV1>,
    points: &[Point3V1],
) -> Result<(), RenderError> {
    let Some((first, rest)) = points.split_first() else {
        return Err(RenderError::InvalidRequest(
            "closed presentation path requires issued points".to_owned(),
        ));
    };
    commands.push(PathCommandV1::MoveTo(point(*first)?));
    for source in rest {
        commands.push(PathCommandV1::LineTo(point(*source)?));
    }
    commands.push(PathCommandV1::Close);
    Ok(())
}

fn stroke(source: &PresentationStrokeV1) -> Result<StrokeV1, RenderError> {
    Ok(StrokeV1::new(
        paint(source.color().as_str())?,
        PositiveFinite::new(source.width().value())?,
    ))
}

fn fill(source: &PresentationFillV1) -> Result<Option<Paint>, RenderError> {
    source
        .color()
        .map(|color| paint(color.as_str()))
        .transpose()
}

fn paint(source: &str) -> Result<Paint, RenderError> {
    let Some(rgb) = source.strip_prefix('#') else {
        return Err(RenderError::InvalidRequest(
            "presentation RGB colour must use the canonical #rrggbb spelling".to_owned(),
        ));
    };
    Ok(Paint::rgb24(Rgb24::new(rgb.to_owned())?))
}

fn point(source: Point3V1) -> Result<RenderPoint, RenderError> {
    RenderPoint::new(source.x(), source.y())
}
