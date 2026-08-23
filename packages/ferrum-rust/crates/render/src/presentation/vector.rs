//! Lower already-verified presentation projections to renderer-neutral vector roots.
//!
//! This boundary consumes only one frozen document projection.  It neither rereads
//! CDML nor derives replacement geometry from a toolkit-specific representation.

use crate::{
    DocumentVectorOpV1, DocumentVectorRootV1, Paint, PathCommandV1, PositiveFinite, RenderError,
    RenderPoint, Rgb24, StrokeV1,
};
use ferrum_document::{
    ArrowDisplayGeometryV1, ArrowProjectionV1, BoxShapeProjectionV1, Point3V1, PolygonProjectionV1,
    PolylineProjectionV1, PresentationFillV1, PresentationRootProjectionV1, PresentationStrokeV1,
};

/// Lower one retained non-text presentation root without changing its issued geometry.
pub fn lower_presentation_vector_v1(
    root: &PresentationRootProjectionV1,
) -> Result<DocumentVectorRootV1, RenderError> {
    match root {
        PresentationRootProjectionV1::Arrow { arrow } => arrow_root(arrow),
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

fn arrow_root(arrow: &ArrowProjectionV1) -> Result<DocumentVectorRootV1, RenderError> {
    let stroke = stroke(arrow.stroke())?;
    match arrow.geometry() {
        ArrowDisplayGeometryV1::Normal {
            axis_path, heads, ..
        } => arrow_geometry_root(&stroke, std::slice::from_ref(axis_path), heads),
        ArrowDisplayGeometryV1::Equilibrium { axes, heads } => {
            arrow_geometry_root(&stroke, axes, heads)
        }
        ArrowDisplayGeometryV1::CurvedEquilibrium { axes, heads, .. } => {
            cubic_arrow_geometry_root(&stroke, axes, heads, "curved equilibrium")
        }
        ArrowDisplayGeometryV1::CurvedTerminal {
            axis_path, head, ..
        } => cubic_arrow_geometry_root(
            &stroke,
            std::slice::from_ref(axis_path),
            std::slice::from_ref(head),
            "curved terminal",
        ),
    }
}

fn cubic_arrow_geometry_root(
    stroke: &StrokeV1,
    axes: &[ferrum_document::ArrowPathV1],
    heads: &[ferrum_document::ArrowHeadV1],
    arrow_family: &str,
) -> Result<DocumentVectorRootV1, RenderError> {
    let mut operations = Vec::new();
    operations
        .try_reserve(axes.len() + usize::from(!heads.is_empty()))
        .map_err(|_| RenderError::ResourceExhausted)?;
    for axis in axes {
        let [start, control_1, control_2, end] = axis.points() else {
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
            closed_points(&mut commands, head.points())?;
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
    axes: &[ferrum_document::ArrowPathV1],
    heads: &[ferrum_document::ArrowHeadV1],
) -> Result<DocumentVectorRootV1, RenderError> {
    let mut operations = Vec::new();
    operations
        .try_reserve(axes.len() + usize::from(!heads.is_empty()))
        .map_err(|_| RenderError::ResourceExhausted)?;
    for axis in axes {
        operations.push(DocumentVectorOpV1::path(
            open_path(axis.points())?,
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
            closed_points(&mut commands, head.points())?;
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
