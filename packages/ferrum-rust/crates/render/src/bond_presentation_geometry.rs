//! Renderer-neutral explicit geometry for nonstereochemical styled bond axes.
//!
//! The atom-bond lowerer owns label clipping and passes this module an already
//! visible finite axis.  This module owns only the deterministic expansion of
//! that axis, so every sink consumes the same explicit lines or cubic path.
//!
//! Styled axes are exactly equivariant under translation and proper rotation.
//! `Wavy` has no source-owned phase or handedness: under reflection it keeps
//! the same axis projection and cubic structure while reversing signed normal
//! displacement. That is the honest reflection contract for an undirected
//! CDXML presentation fact represented by one stroked curve.

use ferrum_geometry::Vector2;

use crate::{
    LineOp, PathOpV3, PositiveFinite, RenderIssueKind, RenderOp, RenderPoint, ScenePathCommandV3,
    ScenePathStrokeV3, VectorStrokeLineCapV1,
};

const DASH_LENGTH_FACTOR: f64 = 3.0;
const DASH_GAP_FACTOR: f64 = 3.0;
const WAVE_LENGTH_FACTOR: f64 = 12.0;
const WAVE_AMPLITUDE_FACTOR: f64 = 2.0;
const MAX_PRIMITIVES: usize = 4096;

/// Expand an already clipped axis into its bold presentation.
pub(crate) fn bold(axis: LineOp) -> Result<Vec<RenderOp>, RenderIssueKind> {
    let doubled_width = PositiveFinite::new(axis.width().get() * 2.0)
        .map_err(|error| unrenderable(format!("bold bond width is not representable: {error}")))?;
    let line = LineOp::new(
        axis.start(),
        axis.end(),
        doubled_width,
        axis.paint().clone(),
        axis.z(),
    )
    .map_err(|error| unrenderable(format!("bold bond is not renderable: {error}")))?;
    Ok(vec![RenderOp::Line(line)])
}

/// Expand an already clipped axis into explicit endpoint-serving dash lines.
///
/// Each emitted dash is exactly `3w` long. The first and last dashes meet the
/// already clipped axis, so both labels retain a visible chemical attachment.
/// Interior gaps are equal and no smaller than `3w`; any remainder is assigned
/// between dashes rather than converted into label-facing blank margins. A
/// visible axis that cannot carry a serving dash at both endpoints, or that
/// would require more than the bounded number of primitives, is unrenderable.
pub(crate) fn dashed(axis: LineOp) -> Result<Vec<RenderOp>, RenderIssueKind> {
    let frame = AxisFrame::from_line(&axis)?;
    let width = axis.width().get();
    let dash = DASH_LENGTH_FACTOR * width;
    let gap = DASH_GAP_FACTOR * width;
    let period = dash + gap;
    if !dash.is_finite() || !gap.is_finite() || !period.is_finite() {
        return Err(unrenderable("dashed bond period is not finite"));
    }
    if frame.length < 2.0 * dash + gap {
        return Err(unrenderable(
            "dashed bond visible axis cannot serve both endpoints with exact 3w dashes",
        ));
    }
    let requested_count = ((frame.length + gap) / period).floor();
    let count = bounded_primitive_count(requested_count, "dashed bond")?;
    if count < 2 {
        return Err(unrenderable(
            "dashed bond visible axis cannot serve both endpoints with exact 3w dashes",
        ));
    }
    let interior_gap = (frame.length - count as f64 * dash) / (count - 1) as f64;
    if !interior_gap.is_finite() || interior_gap < gap {
        return Err(unrenderable("dashed bond placement is not finite"));
    }
    let mut operations = Vec::with_capacity(count);
    for index in 0..count {
        let start_distance = index as f64 * (dash + interior_gap);
        let end_distance = start_distance + dash;
        let line = LineOp::new(
            frame.point_at(start_distance)?,
            frame.point_at(end_distance)?,
            axis.width(),
            axis.paint().clone(),
            axis.z()
                .checked_add(i32::try_from(index).expect("dash primitive cap fits i32"))
                .ok_or_else(|| unrenderable("dashed bond z-order overflows"))?,
        )
        .map_err(|error| unrenderable(format!("dashed bond is not renderable: {error}")))?;
        operations.push(RenderOp::Line(line));
    }
    Ok(operations)
}

/// Expand an already clipped axis into one explicit cubic sine-approximation path.
pub(crate) fn wavy(axis: LineOp) -> Result<Vec<RenderOp>, RenderIssueKind> {
    let frame = AxisFrame::from_line(&axis)?;
    let width = axis.width().get();
    let target_wavelength = WAVE_LENGTH_FACTOR * width;
    if !target_wavelength.is_finite() || target_wavelength <= 0.0 {
        return Err(unrenderable("wavy bond target wavelength is not finite"));
    }
    let requested_count = (frame.length / target_wavelength)
        .round_ties_even()
        .max(1.0);
    let count = bounded_primitive_count(requested_count, "wavy bond")?;
    let wavelength = frame.length / count as f64;
    let amplitude = (WAVE_AMPLITUDE_FACTOR * width).min(frame.length / 6.0);
    if !wavelength.is_finite() || wavelength <= 0.0 || !amplitude.is_finite() || amplitude <= 0.0 {
        return Err(unrenderable("wavy bond placement is not finite"));
    }
    let quarter = wavelength / 4.0;
    let slope = amplitude * std::f64::consts::TAU / wavelength;
    let mut commands = Vec::with_capacity(1 + count * 4);
    commands.push(ScenePathCommandV3::MoveTo(
        frame.point_at_with_normal(0.0, 0.0)?,
    ));
    for wave in 0..count {
        let wave_start = wave as f64 * wavelength;
        for quarter_index in 0..4 {
            let x0 = wave_start + quarter_index as f64 * quarter;
            let x1 = x0 + quarter;
            let (sine0, cosine0) = quarter_trigonometry(quarter_index);
            let (sine1, cosine1) = quarter_trigonometry((quarter_index + 1) % 4);
            let y0 = amplitude * sine0;
            let y1 = amplitude * sine1;
            let dy_dx0 = slope * cosine0;
            let dy_dx1 = slope * cosine1;
            commands.push(ScenePathCommandV3::CubicTo {
                control_1: frame
                    .point_at_with_normal(x0 + quarter / 3.0, y0 + dy_dx0 * quarter / 3.0)?,
                control_2: frame
                    .point_at_with_normal(x1 - quarter / 3.0, y1 - dy_dx1 * quarter / 3.0)?,
                end: frame.point_at_with_normal(x1, y1)?,
            });
        }
    }
    let stroke = ScenePathStrokeV3::new(axis.paint().clone(), axis.width())
        .with_line_cap(VectorStrokeLineCapV1::Round);
    let path = PathOpV3::new(commands, Some(stroke), None, axis.z())
        .map_err(|error| unrenderable(format!("wavy bond is not renderable: {error}")))?;
    Ok(vec![RenderOp::Path(path)])
}

/// Admit an exact geometry count without clamping a source-visible pattern.
///
/// Geometry builders return before constructing a local batch, and the atom-bond
/// owner converts this issue into an `UnrenderableTarget` with no partial batch.
fn bounded_primitive_count(
    requested_count: f64,
    presentation: &str,
) -> Result<usize, RenderIssueKind> {
    if !requested_count.is_finite() || requested_count < 1.0 {
        return Err(unrenderable(format!(
            "{presentation} primitive count is not representable"
        )));
    }
    if requested_count > MAX_PRIMITIVES as f64 {
        return Err(unrenderable(format!(
            "{presentation} requires {requested_count:.0} primitives, above the {MAX_PRIMITIVES} primitive cap"
        )));
    }
    Ok(requested_count as usize)
}

/// Return the exact sine and cosine at the four quarter-wave boundaries.
///
/// The finite lookup, rather than a transcendental evaluation, guarantees that
/// each completed whole wave lands exactly back on the clipped bond axis.
fn quarter_trigonometry(quarter: usize) -> (f64, f64) {
    match quarter {
        0 => (0.0, 1.0),
        1 => (1.0, 0.0),
        2 => (0.0, -1.0),
        3 => (-1.0, 0.0),
        _ => unreachable!("quarter-wave index is reduced modulo four"),
    }
}

struct AxisFrame {
    start: RenderPoint,
    direction: Vector2,
    normal: Vector2,
    length: f64,
}

impl AxisFrame {
    fn from_line(line: &LineOp) -> Result<Self, RenderIssueKind> {
        let delta_x = line.end().x() - line.start().x();
        let delta_y = line.end().y() - line.start().y();
        let length = delta_x.hypot(delta_y);
        if !length.is_finite() || length <= 0.0 {
            return Err(unrenderable(
                "styled bond axis has no positive finite length",
            ));
        }
        let direction = Vector2::new(delta_x / length, delta_y / length)
            .map_err(|error| unrenderable(format!("styled bond axis is invalid: {error}")))?;
        Ok(Self {
            start: line.start(),
            normal: direction.perpendicular_left(),
            direction,
            length,
        })
    }

    fn point_at(&self, distance: f64) -> Result<RenderPoint, RenderIssueKind> {
        self.point_at_with_normal(distance, 0.0)
    }

    fn point_at_with_normal(
        &self,
        distance: f64,
        normal_distance: f64,
    ) -> Result<RenderPoint, RenderIssueKind> {
        let x = self.start.x() + self.direction.x() * distance + self.normal.x() * normal_distance;
        let y = self.start.y() + self.direction.y() * distance + self.normal.y() * normal_distance;
        RenderPoint::new(x, y)
            .map_err(|error| unrenderable(format!("styled bond point is not finite: {error}")))
    }
}

fn unrenderable(reason: impl Into<String>) -> RenderIssueKind {
    RenderIssueKind::UnrenderableTarget {
        reason: reason.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RenderPaintV3, Rgb24};

    fn axis(length: f64) -> LineOp {
        LineOp::new(
            RenderPoint::new(0.0, 0.0).expect("start"),
            RenderPoint::new(length, 0.0).expect("end"),
            PositiveFinite::new(1.0).expect("width"),
            RenderPaintV3::authored_rgb24(Rgb24::new("112233").expect("paint")),
            10,
        )
        .expect("axis")
    }

    #[test]
    fn bold_retains_the_clipped_axis_and_doubles_only_width() {
        let operations = bold(axis(40.0)).expect("bold geometry");
        let RenderOp::Line(line) = &operations[0] else {
            panic!("bold uses one line")
        };
        assert_eq!(line.start().x(), 0.0);
        assert_eq!(line.end().x(), 40.0);
        assert_eq!(line.width().get(), 2.0);
        assert_eq!(line.z(), 10);
    }

    #[test]
    fn dashes_serve_both_clipped_endpoints_and_distribute_remainder_interiorly() {
        let operations = dashed(axis(24.0)).expect("dash geometry");
        let lines = operations
            .iter()
            .map(|operation| match operation {
                RenderOp::Line(line) => line,
                _ => panic!("dash lowering only emits lines"),
            })
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 4);
        assert_near(lines.first().expect("first").start().x(), 0.0);
        assert_near(lines.last().expect("last").end().x(), 24.0);
        for pair in lines.windows(2) {
            assert!(pair[0].end().x() < pair[1].start().x());
            assert!(pair[0].z() < pair[1].z());
            assert_near(pair[0].end().x() - pair[0].start().x(), 3.0);
            assert_near(pair[1].start().x() - pair[0].end().x(), 4.0);
        }
    }

    #[test]
    fn short_dash_axis_is_refused_instead_of_shrinking_the_exact_dash() {
        assert!(matches!(
            dashed(axis(8.999_999_999_999)),
            Err(RenderIssueKind::UnrenderableTarget { reason })
                if reason.contains("cannot serve both endpoints")
        ));
    }

    #[test]
    fn styled_geometry_is_translation_equivariant() {
        let translated = LineOp::new(
            RenderPoint::new(100.0, -50.0).expect("start"),
            RenderPoint::new(124.0, -50.0).expect("end"),
            PositiveFinite::new(1.0).expect("width"),
            RenderPaintV3::authored_rgb24(Rgb24::new("112233").expect("paint")),
            10,
        )
        .expect("axis");
        let original = dashed(axis(24.0)).expect("dashes");
        let shifted = dashed(translated).expect("shifted dashes");
        assert_eq!(original.len(), shifted.len());
        for (original, shifted) in original.iter().zip(&shifted) {
            let (RenderOp::Line(original), RenderOp::Line(shifted)) = (original, shifted) else {
                panic!("dashes remain lines")
            };
            assert_eq!(shifted.start().x() - original.start().x(), 100.0);
            assert_eq!(shifted.end().x() - original.end().x(), 100.0);
            assert_eq!(shifted.start().y() - original.start().y(), -50.0);
            assert_eq!(shifted.end().y() - original.end().y(), -50.0);
        }
    }

    #[test]
    fn styled_geometry_is_rotation_equivariant_and_reflection_reverses_unspecified_wave_phase() {
        let rotated = LineOp::new(
            RenderPoint::new(7.0, -11.0).expect("start"),
            RenderPoint::new(7.0, 13.0).expect("end"),
            PositiveFinite::new(1.0).expect("width"),
            RenderPaintV3::authored_rgb24(Rgb24::new("112233").expect("paint")),
            10,
        )
        .expect("axis");
        let transformed = LineOp::new(
            RenderPoint::new(7.0, -11.0).expect("start"),
            RenderPoint::new(7.0, -35.0).expect("end"),
            PositiveFinite::new(1.0).expect("width"),
            RenderPaintV3::authored_rgb24(Rgb24::new("112233").expect("paint")),
            10,
        )
        .expect("axis");
        let original_dashes = dashed(axis(24.0)).expect("dashes");
        let rotated_dashes = dashed(rotated.clone()).expect("rotated dashes");
        for (original, rotated) in original_dashes.iter().zip(&rotated_dashes) {
            let (RenderOp::Line(original), RenderOp::Line(rotated)) = (original, rotated) else {
                panic!("dashes remain lines")
            };
            assert_near(rotated.start().x(), 7.0);
            assert_near(rotated.end().x(), 7.0);
            assert_near(rotated.start().y(), -11.0 + original.start().x());
            assert_near(rotated.end().y(), -11.0 + original.end().x());
        }
        let transformed_dashes = dashed(transformed.clone()).expect("transformed dashes");
        assert_eq!(original_dashes.len(), transformed_dashes.len());
        for (original, transformed) in original_dashes.iter().zip(&transformed_dashes) {
            let (RenderOp::Line(original), RenderOp::Line(transformed)) = (original, transformed)
            else {
                panic!("dashes remain lines")
            };
            assert_near(transformed.start().x(), 7.0);
            assert_near(transformed.end().x(), 7.0);
            assert_near(transformed.start().y(), -11.0 - original.start().x());
            assert_near(transformed.end().y(), -11.0 - original.end().x());
        }

        let original_wave = wavy(axis(24.0)).expect("wave");
        let rotated_wave = wavy(rotated).expect("rotated wave");
        let (RenderOp::Path(original), RenderOp::Path(rotated)) =
            (&original_wave[0], &rotated_wave[0])
        else {
            panic!("waves remain paths")
        };
        for (original, rotated) in original.commands().iter().zip(rotated.commands()) {
            assert_rotated_command(*original, *rotated);
        }
        let transformed_wave = wavy(transformed).expect("transformed wave");
        let (RenderOp::Path(original), RenderOp::Path(transformed)) =
            (&original_wave[0], &transformed_wave[0])
        else {
            panic!("waves remain paths")
        };
        for (original, transformed) in original.commands().iter().zip(transformed.commands()) {
            assert_reflected_phase_command(*original, *transformed);
        }
    }

    #[test]
    fn styled_geometry_obeys_exact_count_thresholds_and_refuses_cap_breaches() {
        assert_eq!(dashed(axis(9.0)).expect("dashes").len(), 2);
        let exact_target = dashed(axis(24.0)).expect("dashes");
        let exact_target = exact_target
            .iter()
            .map(|operation| match operation {
                RenderOp::Line(line) => line,
                _ => panic!("dashes remain lines"),
            })
            .collect::<Vec<_>>();
        assert_eq!(exact_target.len(), 4);
        assert_near(exact_target[0].start().x(), 0.0);
        assert_near(exact_target.last().expect("last").end().x(), 24.0);
        for pair in exact_target.windows(2) {
            assert_near(pair[0].end().x() - pair[0].start().x(), 3.0);
            assert_near(pair[1].start().x() - pair[0].end().x(), 4.0);
        }
        assert_eq!(wave_command_count(17.999_999_999_999), 5);
        assert_eq!(wave_command_count(18.0), 9);
        assert_eq!(
            dashed(axis(24_573.0)).expect("dashes").len(),
            MAX_PRIMITIVES
        );
        assert_eq!(wave_command_count(49_152.0), 1 + 4 * MAX_PRIMITIVES);
        for result in [dashed(axis(24_579.0)), dashed(axis(1.0e100))] {
            assert!(matches!(
                result,
                Err(RenderIssueKind::UnrenderableTarget { reason }) if reason.contains("primitive")
            ));
        }
        for result in [wavy(axis(49_158.000_001)), wavy(axis(1.0e100))] {
            assert!(matches!(
                result,
                Err(RenderIssueKind::UnrenderableTarget { reason }) if reason.contains("primitive")
            ));
        }
    }

    #[test]
    fn wave_has_exact_endpoints_finite_cubics_and_expected_tangents() {
        let operations = wavy(axis(24.0)).expect("wave geometry");
        let RenderOp::Path(path) = &operations[0] else {
            panic!("wave uses one path")
        };
        assert!(path.fill().is_none());
        assert_eq!(
            path.stroke().expect("stroke").line_cap(),
            VectorStrokeLineCapV1::Round
        );
        let ScenePathCommandV3::MoveTo(start) = path.commands()[0] else {
            panic!("path begins at axis start")
        };
        assert_eq!(start.x(), 0.0);
        assert_eq!(start.y(), 0.0);
        let ScenePathCommandV3::CubicTo { control_1, .. } = path.commands()[1] else {
            panic!("wave begins with cubic")
        };
        assert!(control_1.y() > 0.0);
        let ScenePathCommandV3::CubicTo { end, .. } = path.commands().last().expect("last") else {
            panic!("wave ends with cubic")
        };
        assert_eq!(end.x(), 24.0);
        assert!(end.y().abs() < 1e-12);
        for command in path.commands() {
            match command {
                ScenePathCommandV3::MoveTo(point) | ScenePathCommandV3::LineTo(point) => {
                    assert!(point.x().is_finite() && point.y().is_finite());
                }
                ScenePathCommandV3::CubicTo {
                    control_1,
                    control_2,
                    end,
                } => {
                    for point in [control_1, control_2, end] {
                        assert!(point.x().is_finite() && point.y().is_finite());
                    }
                }
                ScenePathCommandV3::Close => panic!("wave is an open stroke"),
            }
        }
        let extrema = path
            .commands()
            .iter()
            .filter_map(|command| match command {
                ScenePathCommandV3::CubicTo { end, .. } if end.y() != 0.0 => Some(end.y()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(extrema, vec![2.0, -2.0, 2.0, -2.0]);
    }

    fn wave_command_count(length: f64) -> usize {
        let operations = wavy(axis(length)).expect("wave");
        let RenderOp::Path(path) = &operations[0] else {
            panic!("wave remains a path")
        };
        path.commands().len()
    }

    fn assert_near(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1.0e-12,
            "{actual} != {expected}"
        );
    }

    fn assert_reflected_phase_command(
        original: ScenePathCommandV3,
        transformed: ScenePathCommandV3,
    ) {
        match (original, transformed) {
            (ScenePathCommandV3::MoveTo(original), ScenePathCommandV3::MoveTo(transformed))
            | (ScenePathCommandV3::LineTo(original), ScenePathCommandV3::LineTo(transformed)) => {
                assert_reflected_phase_point(original, transformed);
            }
            (
                ScenePathCommandV3::CubicTo {
                    control_1: original_1,
                    control_2: original_2,
                    end: original_end,
                },
                ScenePathCommandV3::CubicTo {
                    control_1: transformed_1,
                    control_2: transformed_2,
                    end: transformed_end,
                },
            ) => {
                assert_reflected_phase_point(original_1, transformed_1);
                assert_reflected_phase_point(original_2, transformed_2);
                assert_reflected_phase_point(original_end, transformed_end);
            }
            _ => panic!("frame transform preserves path command structure"),
        }
    }

    fn assert_reflected_phase_point(original: RenderPoint, transformed: RenderPoint) {
        assert_near(transformed.x(), 7.0 + original.y());
        assert_near(transformed.y(), -11.0 - original.x());
    }

    fn assert_rotated_command(original: ScenePathCommandV3, rotated: ScenePathCommandV3) {
        match (original, rotated) {
            (ScenePathCommandV3::MoveTo(original), ScenePathCommandV3::MoveTo(rotated))
            | (ScenePathCommandV3::LineTo(original), ScenePathCommandV3::LineTo(rotated)) => {
                assert_rotated_point(original, rotated);
            }
            (
                ScenePathCommandV3::CubicTo {
                    control_1: original_1,
                    control_2: original_2,
                    end: original_end,
                },
                ScenePathCommandV3::CubicTo {
                    control_1: rotated_1,
                    control_2: rotated_2,
                    end: rotated_end,
                },
            ) => {
                assert_rotated_point(original_1, rotated_1);
                assert_rotated_point(original_2, rotated_2);
                assert_rotated_point(original_end, rotated_end);
            }
            _ => panic!("proper rotation preserves path command structure"),
        }
    }

    fn assert_rotated_point(original: RenderPoint, rotated: RenderPoint) {
        assert_near(rotated.x(), 7.0 - original.y());
        assert_near(rotated.y(), -11.0 + original.x());
    }
}
