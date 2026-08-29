//! Private final-bond-ink versus atom-label-envelope collision geometry.
//!
//! This module deliberately consumes only already-closed bond operations. It
//! never reconstructs a source axis or asks a presentation consumer for pixels.

use std::collections::HashMap;

use ferrum_core::RecordId;

use crate::glyph_metrics::GlyphBounds;
use crate::{
    BondRenderBatchV1, BondRenderOpV1, LineOp, PathOpV3, PositiveFinite, RenderPoint,
    ScenePathCommandV3, VectorStrokeLineCapV1,
};

/// All numerical policy for the tiny exact-operation kernel lives here.
struct CollisionTolerance;

impl CollisionTolerance {
    /// Conservative cubic flatness bound in scene units. Any uncertain contact
    /// is a conflict, while ordinary strictly separated near-misses remain
    /// separable by many orders of magnitude.
    const CUBIC_FLATNESS: f64 = 1.0e-7;
    const MAX_CUBIC_SUBDIVISIONS: u8 = 24;
    const ORIENTATION_RELATIVE_EPSILON: f64 = 32.0 * f64::EPSILON;
}

#[derive(Clone, Copy)]
pub(super) struct LabelInkEnvelope {
    rectangle: Rectangle,
}

impl LabelInkEnvelope {
    pub(super) fn from_local_bounds(
        bounds: GlyphBounds,
        anchor: RenderPoint,
        clearance: PositiveFinite,
    ) -> Result<Self, String> {
        let amount = clearance.get();
        let rectangle = Rectangle::new(
            bounds.min_x() + anchor.x() - amount,
            bounds.min_y() + anchor.y() - amount,
            bounds.max_x() + anchor.x() + amount,
            bounds.max_y() + anchor.y() + amount,
        )?;
        Ok(Self { rectangle })
    }
}

/// Return whether any complete painted primitive conflicts with another atom.
pub(super) fn batch_intersects_non_endpoint_label(
    batch: &BondRenderBatchV1,
    labels: &HashMap<RecordId, LabelInkEnvelope>,
    first_endpoint: &RecordId,
    second_endpoint: &RecordId,
) -> Result<bool, String> {
    for (record_id, envelope) in labels {
        if record_id == first_endpoint || record_id == second_endpoint {
            continue;
        }
        for operation in batch.operations() {
            if operation_intersects_rectangle(operation, envelope.rectangle)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn operation_intersects_rectangle(
    operation: &BondRenderOpV1,
    rectangle: Rectangle,
) -> Result<bool, String> {
    match operation {
        BondRenderOpV1::Line(line) => stroked_line_intersects_rectangle(line, rectangle, false),
        BondRenderOpV1::Path(path) => path_intersects_rectangle(path, rectangle),
        BondRenderOpV1::DoubleBondCarrierMark(mark) => {
            stroked_line_intersects_rectangle(&mark.accent_line(), rectangle, false)
        }
    }
}

fn stroked_line_intersects_rectangle(
    line: &LineOp,
    rectangle: Rectangle,
    round_caps: bool,
) -> Result<bool, String> {
    let start = Point::from(line.start());
    let end = Point::from(line.end());
    stroked_segment_intersects_rectangle(
        start,
        end,
        line.width().get() / 2.0,
        round_caps,
        rectangle,
    )
}

fn path_intersects_rectangle(path: &PathOpV3, rectangle: Rectangle) -> Result<bool, String> {
    let paths = flatten_path(path)?;
    if let Some(stroke) = path.stroke() {
        let radius = stroke.width().get() / 2.0 + CollisionTolerance::CUBIC_FLATNESS;
        let round_caps = stroke.line_cap() == VectorStrokeLineCapV1::Round;
        for subpath in &paths {
            if stroked_polyline_intersects_rectangle(
                subpath,
                radius,
                round_caps,
                stroke.miter_limit(),
                rectangle,
            )? {
                return Ok(true);
            }
        }
    }
    Ok(path.fill().is_some() && filled_paths_intersect_rectangle(&paths, rectangle))
}

fn stroked_polyline_intersects_rectangle(
    subpath: &FlattenedPath,
    radius: f64,
    round_caps: bool,
    miter_limit: f64,
    rectangle: Rectangle,
) -> Result<bool, String> {
    for segment in subpath.points.windows(2) {
        if stroked_segment_intersects_rectangle(segment[0], segment[1], radius, false, rectangle)? {
            return Ok(true);
        }
    }
    let endpoint_count = subpath.points.len() - usize::from(subpath.closed);
    let joint_points = if subpath.closed {
        &subpath.points[..endpoint_count]
    } else {
        &subpath.points[1..endpoint_count - 1]
    };
    if joint_points
        .iter()
        .any(|point| rectangle.distance_to_point(*point) <= radius * miter_limit)
    {
        return Ok(true);
    }
    Ok(!subpath.closed
        && round_caps
        && (rectangle.distance_to_point(subpath.points[0]) <= radius
            || rectangle.distance_to_point(subpath.points[endpoint_count - 1]) <= radius))
}

fn flatten_path(path: &PathOpV3) -> Result<Vec<FlattenedPath>, String> {
    let mut paths = Vec::new();
    let mut current_path = Vec::new();
    let mut current = None;
    let mut start = None;
    let mut closed = false;
    for command in path.commands() {
        match *command {
            ScenePathCommandV3::MoveTo(point) => {
                if !current_path.is_empty() {
                    paths.push(FlattenedPath {
                        points: std::mem::take(&mut current_path),
                        closed,
                    });
                }
                let point = Point::from(point);
                current_path.push(point);
                current = Some(point);
                start = Some(point);
                closed = false;
            }
            ScenePathCommandV3::LineTo(point) => {
                let point = Point::from(point);
                current_path.push(point);
                current = Some(point);
            }
            ScenePathCommandV3::CubicTo {
                control_1,
                control_2,
                end,
            } => {
                let start_point = current.expect("validated path command has a current point");
                flatten_cubic(
                    start_point,
                    Point::from(control_1),
                    Point::from(control_2),
                    Point::from(end),
                    0,
                    &mut current_path,
                )?;
                current = Some(Point::from(end));
            }
            ScenePathCommandV3::Close => {
                let first = start.expect("validated closed path has a start point");
                if current != Some(first) {
                    current_path.push(first);
                }
                current = Some(first);
                closed = true;
            }
        }
    }
    if !current_path.is_empty() {
        paths.push(FlattenedPath {
            points: current_path,
            closed,
        });
    }
    Ok(paths)
}

fn flatten_cubic(
    first: Point,
    control_1: Point,
    control_2: Point,
    final_point: Point,
    depth: u8,
    output: &mut Vec<Point>,
) -> Result<(), String> {
    let flatness = cubic_flatness(first, control_1, control_2, final_point);
    if !flatness.is_finite() {
        return Err("uncertain final bond cubic flattening".to_owned());
    }
    if flatness <= CollisionTolerance::CUBIC_FLATNESS {
        output.push(final_point);
        return Ok(());
    }
    if depth >= CollisionTolerance::MAX_CUBIC_SUBDIVISIONS {
        return Err("uncertain final bond cubic flattening".to_owned());
    }
    let first_control = first.midpoint(control_1);
    let controls = control_1.midpoint(control_2);
    let final_control = control_2.midpoint(final_point);
    let left_final = first_control.midpoint(controls);
    let right_first = controls.midpoint(final_control);
    let middle = left_final.midpoint(right_first);
    flatten_cubic(first, first_control, left_final, middle, depth + 1, output)?;
    flatten_cubic(
        middle,
        right_first,
        final_control,
        final_point,
        depth + 1,
        output,
    )?;
    Ok(())
}

fn cubic_flatness(first: Point, control_1: Point, control_2: Point, final_point: Point) -> f64 {
    point_line_distance(control_1, first, final_point).max(point_line_distance(
        control_2,
        first,
        final_point,
    ))
}

fn point_line_distance(point: Point, first: Point, final_point: Point) -> f64 {
    let dx = final_point.x - first.x;
    let dy = final_point.y - first.y;
    let length = dx.hypot(dy);
    if length <= coordinate_tolerance(first, final_point, point) {
        point.distance(first)
    } else {
        ((point.x - first.x) * dy - (point.y - first.y) * dx).abs() / length
    }
}

fn stroked_segment_intersects_rectangle(
    first: Point,
    final_point: Point,
    radius: f64,
    round_caps: bool,
    rectangle: Rectangle,
) -> Result<bool, String> {
    if !radius.is_finite() || radius < 0.0 {
        return Ok(true);
    }
    let dx = final_point.x - first.x;
    let dy = final_point.y - first.y;
    let length = dx.hypot(dy);
    if !length.is_finite() {
        return Ok(true);
    }
    if first == final_point {
        return Ok(round_caps && rectangle.distance_to_point(first) <= radius);
    }
    let normal = Point::new(-(dy / length) * radius, (dx / length) * radius);
    if !normal.x.is_finite() || !normal.y.is_finite() {
        return Ok(true);
    }
    let stroke = [
        first.add(normal),
        final_point.add(normal),
        final_point.subtract(normal),
        first.subtract(normal),
    ];
    Ok(polygon_intersects_rectangle(&stroke, rectangle)
        || (round_caps
            && (rectangle.distance_to_point(first) <= radius
                || rectangle.distance_to_point(final_point) <= radius)))
}

fn filled_paths_intersect_rectangle(paths: &[FlattenedPath], rectangle: Rectangle) -> bool {
    if paths.iter().any(|path| {
        path.points.len() >= 4
            && (path.points.iter().any(|point| rectangle.contains(*point))
                || polygon_edges(&path.points).any(|(first, final_point)| {
                    rectangle_edges(rectangle)
                        .iter()
                        .any(|side| segments_intersect(first, final_point, side.0, side.1))
                }))
    }) {
        return true;
    }
    rectangle.corners().iter().any(|corner| {
        paths.iter().fold(false, |inside, path| {
            inside ^ point_in_polygon(*corner, &path.points)
        })
    })
}

fn polygon_intersects_rectangle(polygon: &[Point], rectangle: Rectangle) -> bool {
    if polygon.iter().any(|point| rectangle.contains(*point)) {
        return true;
    }
    let corners = rectangle.corners();
    if corners
        .iter()
        .any(|corner| point_in_polygon(*corner, polygon))
    {
        return true;
    }
    polygon_edges(polygon).any(|(first, final_point)| {
        rectangle_edges(rectangle)
            .iter()
            .any(|side| segments_intersect(first, final_point, side.0, side.1))
    })
}

fn point_in_polygon(point: Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    for (first, final_point) in polygon_edges(polygon) {
        if point_on_segment(point, first, final_point) {
            return true;
        }
        let crosses = (first.y > point.y) != (final_point.y > point.y);
        if crosses {
            let x = (final_point.x - first.x) * (point.y - first.y) / (final_point.y - first.y)
                + first.x;
            if x >= point.x {
                inside = !inside;
            }
        }
    }
    inside
}

fn segments_intersect(first: Point, second: Point, third: Point, fourth: Point) -> bool {
    let first_orientation = orientation(first, second, third);
    let second_orientation = orientation(first, second, fourth);
    let third_orientation = orientation(third, fourth, first);
    let fourth_orientation = orientation(third, fourth, second);
    if orientation_is_uncertain(first, second, third, first_orientation)
        && point_on_segment(third, first, second)
    {
        return true;
    }
    if orientation_is_uncertain(first, second, fourth, second_orientation)
        && point_on_segment(fourth, first, second)
    {
        return true;
    }
    if orientation_is_uncertain(third, fourth, first, third_orientation)
        && point_on_segment(first, third, fourth)
    {
        return true;
    }
    if orientation_is_uncertain(third, fourth, second, fourth_orientation)
        && point_on_segment(second, third, fourth)
    {
        return true;
    }
    (first_orientation > 0.0) != (second_orientation > 0.0)
        && (third_orientation > 0.0) != (fourth_orientation > 0.0)
}

fn orientation(first: Point, second: Point, point: Point) -> f64 {
    let scale = coordinate_scale(first, second, point);
    let first_x = first.x / scale;
    let first_y = first.y / scale;
    let second_x = second.x / scale;
    let second_y = second.y / scale;
    let point_x = point.x / scale;
    let point_y = point.y / scale;
    (second_x - first_x) * (point_y - first_y) - (second_y - first_y) * (point_x - first_x)
}

fn point_on_segment(point: Point, first: Point, final_point: Point) -> bool {
    orientation_is_uncertain(
        first,
        final_point,
        point,
        orientation(first, final_point, point),
    ) && point_within_segment_bounds(point, first, final_point)
}

fn point_within_segment_bounds(point: Point, first: Point, final_point: Point) -> bool {
    let tolerance = coordinate_tolerance(first, final_point, point);
    point.x >= first.x.min(final_point.x) - tolerance
        && point.x <= first.x.max(final_point.x) + tolerance
        && point.y >= first.y.min(final_point.y) - tolerance
        && point.y <= first.y.max(final_point.y) + tolerance
}

fn orientation_is_uncertain(_first: Point, _second: Point, _point: Point, value: f64) -> bool {
    !value.is_finite() || value.abs() <= CollisionTolerance::ORIENTATION_RELATIVE_EPSILON
}

fn coordinate_tolerance(first: Point, second: Point, point: Point) -> f64 {
    coordinate_scale(first, second, point) * CollisionTolerance::ORIENTATION_RELATIVE_EPSILON
}

fn coordinate_scale(first: Point, second: Point, point: Point) -> f64 {
    first
        .x
        .abs()
        .max(first.y.abs())
        .max(second.x.abs())
        .max(second.y.abs())
        .max(point.x.abs())
        .max(point.y.abs())
        .max(1.0)
}

fn polygon_edges(polygon: &[Point]) -> impl Iterator<Item = (Point, Point)> + '_ {
    polygon
        .iter()
        .copied()
        .zip(polygon.iter().copied().cycle().skip(1))
        .take(polygon.len())
}

fn rectangle_edges(rectangle: Rectangle) -> [(Point, Point); 4] {
    let [bottom_left, bottom_right, top_right, top_left] = rectangle.corners();
    [
        (bottom_left, bottom_right),
        (bottom_right, top_right),
        (top_right, top_left),
        (top_left, bottom_left),
    ]
}

#[derive(Clone, Copy)]
struct Rectangle {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl Rectangle {
    fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Self, String> {
        if !min_x.is_finite()
            || !min_y.is_finite()
            || !max_x.is_finite()
            || !max_y.is_finite()
            || min_x > max_x
            || min_y > max_y
        {
            return Err("atom label exclusion envelope is not finite".to_owned());
        }
        Ok(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    fn contains(self, point: Point) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }

    fn distance_to_point(self, point: Point) -> f64 {
        let dx = if point.x < self.min_x {
            self.min_x - point.x
        } else if point.x > self.max_x {
            point.x - self.max_x
        } else {
            0.0
        };
        let dy = if point.y < self.min_y {
            self.min_y - point.y
        } else if point.y > self.max_y {
            point.y - self.max_y
        } else {
            0.0
        };
        dx.hypot(dy)
    }

    fn corners(self) -> [Point; 4] {
        [
            Point::new(self.min_x, self.min_y),
            Point::new(self.max_x, self.min_y),
            Point::new(self.max_x, self.max_y),
            Point::new(self.min_x, self.max_y),
        ]
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Point {
    x: f64,
    y: f64,
}

struct FlattenedPath {
    points: Vec<Point>,
    closed: bool,
}

impl Point {
    const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn midpoint(self, other: Self) -> Self {
        Self::new(self.x / 2.0 + other.x / 2.0, self.y / 2.0 + other.y / 2.0)
    }

    fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y)
    }

    fn subtract(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

impl From<RenderPoint> for Point {
    fn from(point: RenderPoint) -> Self {
        Self::new(point.x(), point.y())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BondRenderOpV1, PathOpV3, RenderPaintV3, Rgb24, ScenePathStrokeV3};

    fn point(x: f64, y: f64) -> RenderPoint {
        RenderPoint::new(x, y).expect("finite test point")
    }

    fn size(value: f64) -> PositiveFinite {
        PositiveFinite::new(value).expect("positive test extent")
    }

    fn paint() -> RenderPaintV3 {
        RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("rgb"))
    }

    fn rectangle() -> Rectangle {
        Rectangle::new(0.0, 0.0, 2.0, 2.0).expect("rectangle")
    }

    #[test]
    fn horizontal_vertical_diagonal_touching_and_near_miss_are_classified_from_final_ink() {
        for (start, end) in [
            (point(-3.0, 1.0), point(3.0, 1.0)),
            (point(1.0, -3.0), point(1.0, 3.0)),
            (point(-2.0, -2.0), point(3.0, 3.0)),
        ] {
            let line = LineOp::new(start, end, size(1.0), paint(), 1).expect("line");
            assert!(stroked_line_intersects_rectangle(&line, rectangle(), false).expect("line"));
        }
        let touching = LineOp::new(point(-3.0, 2.5), point(3.0, 2.5), size(1.0), paint(), 1)
            .expect("touching line");
        let near_miss = LineOp::new(
            point(-3.0, 2.500_001),
            point(3.0, 2.500_001),
            size(1.0),
            paint(),
            1,
        )
        .expect("separated line");
        assert!(
            stroked_line_intersects_rectangle(&touching, rectangle(), false)
                .expect("touching line")
        );
        assert!(
            !stroked_line_intersects_rectangle(&near_miss, rectangle(), false)
                .expect("near-miss line")
        );
    }

    #[test]
    fn filled_wedge_and_round_cubic_path_are_checked_as_painted_geometry() {
        let filled = PathOpV3::new(
            vec![
                ScenePathCommandV3::MoveTo(point(-2.0, 1.0)),
                ScenePathCommandV3::LineTo(point(3.0, 0.0)),
                ScenePathCommandV3::LineTo(point(3.0, 2.0)),
                ScenePathCommandV3::Close,
            ],
            None,
            Some(paint()),
            1,
        )
        .expect("filled wedge");
        let cubic = PathOpV3::new(
            vec![
                ScenePathCommandV3::MoveTo(point(-2.0, -1.0)),
                ScenePathCommandV3::CubicTo {
                    control_1: point(-1.0, 4.0),
                    control_2: point(3.0, 4.0),
                    end: point(4.0, -1.0),
                },
            ],
            Some(
                ScenePathStrokeV3::new(paint(), size(1.0))
                    .with_line_cap(VectorStrokeLineCapV1::Round),
            ),
            None,
            2,
        )
        .expect("cubic path");
        assert!(path_intersects_rectangle(&filled, rectangle()).expect("filled path"));
        assert!(path_intersects_rectangle(&cubic, rectangle()).expect("cubic path"));
        let axis = crate::BondAttachmentAxisV1::new(point(-2.0, -1.0), point(4.0, -1.0))
            .expect("attachment axis");
        let batch = BondRenderBatchV1::new(
            axis,
            vec![BondRenderOpV1::Path(filled), BondRenderOpV1::Path(cubic)],
        )
        .expect("closed batch");
        assert_eq!(batch.operations().len(), 2);
    }

    #[test]
    fn polygon_closing_edge_and_collinearity_boundaries_are_not_skipped_or_overclaimed() {
        let polygon = [
            Point::new(-1.0, 2.0),
            Point::new(-2.0, 3.0),
            Point::new(3.0, 3.0),
            Point::new(2.0, -1.0),
        ];
        assert!(polygon_intersects_rectangle(&polygon, rectangle()));
        assert!(!point_on_segment(
            Point::new(1.0, 0.5),
            Point::new(0.0, 0.0),
            Point::new(2.0, 2.0),
        ));
    }

    #[test]
    fn cubic_limit_and_even_odd_hole_have_closed_outcomes() {
        let mut output = vec![Point::new(0.0, 0.0)];
        assert!(
            flatten_cubic(
                Point::new(0.0, 0.0),
                Point::new(0.0, 1.0e12),
                Point::new(1.0, 1.0e12),
                Point::new(1.0, 0.0),
                CollisionTolerance::MAX_CUBIC_SUBDIVISIONS,
                &mut output,
            )
            .is_err()
        );
        let filled = PathOpV3::new(
            vec![
                ScenePathCommandV3::MoveTo(point(-5.0, -5.0)),
                ScenePathCommandV3::LineTo(point(5.0, -5.0)),
                ScenePathCommandV3::LineTo(point(5.0, 5.0)),
                ScenePathCommandV3::LineTo(point(-5.0, 5.0)),
                ScenePathCommandV3::Close,
                ScenePathCommandV3::MoveTo(point(-2.0, -2.0)),
                ScenePathCommandV3::LineTo(point(-2.0, 4.0)),
                ScenePathCommandV3::LineTo(point(4.0, 4.0)),
                ScenePathCommandV3::LineTo(point(4.0, -2.0)),
                ScenePathCommandV3::Close,
            ],
            None,
            Some(paint()),
            1,
        )
        .expect("even-odd path");
        assert!(!path_intersects_rectangle(&filled, rectangle()).expect("hole"));
    }

    #[test]
    fn zero_length_strokes_are_empty_or_round_disks_without_remote_refusal() {
        let far = Point::new(100.0, 100.0);
        assert!(
            !stroked_segment_intersects_rectangle(far, far, 1.0, false, rectangle()).expect("butt")
        );
        assert!(
            !stroked_segment_intersects_rectangle(far, far, 1.0, true, rectangle())
                .expect("round far")
        );
        let near = Point::new(1.0, 1.0);
        assert!(
            stroked_segment_intersects_rectangle(near, near, 1.0, true, rectangle())
                .expect("round near")
        );
    }

    #[test]
    fn contained_fill_and_extreme_finite_segments_have_explicit_outcomes() {
        let island = FlattenedPath {
            points: vec![
                Point::new(0.5, 0.5),
                Point::new(1.5, 0.5),
                Point::new(1.5, 1.5),
                Point::new(0.5, 1.5),
                Point::new(0.5, 0.5),
            ],
            closed: true,
        };
        assert!(filled_paths_intersect_rectangle(&[island], rectangle()));
        let far = Point::new(f64::MAX, f64::MAX);
        assert!(
            !stroked_segment_intersects_rectangle(far, far, 1.0, false, rectangle())
                .expect("far finite butt")
        );
        assert!(
            stroked_segment_intersects_rectangle(
                Point::new(-f64::MAX, 0.0),
                Point::new(f64::MAX, 0.0),
                1.0,
                false,
                rectangle(),
            )
            .expect("extreme crossing")
        );
    }
}
