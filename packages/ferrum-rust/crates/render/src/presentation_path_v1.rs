//! Toolkit-neutral lowering for authored presentation paths.
//!
//! Authoring retains its exact control points. This module only issues the
//! derived, cubic-only command stream that a renderer can replay.

use thiserror::Error;

use crate::{PathCommandV1, RenderPoint};

/// The authored interpolation policy selected for a presentation path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathKindV1 {
    /// Join each authored point with a straight segment.
    Polyline,
    /// Apply Ferrum's authored-control spline interpolation policy.
    AuthoredSpline,
}

/// A frozen, toolkit-neutral presentation path.
#[derive(Clone, Debug, PartialEq)]
pub struct PresentationPathV1 {
    kind: PathKindV1,
    commands: Vec<PathCommandV1>,
}

impl PresentationPathV1 {
    /// Return the explicit interpolation policy used to lower this path.
    #[must_use]
    pub const fn kind(&self) -> PathKindV1 {
        self.kind
    }

    /// Return commands in the shared MoveTo/LineTo/CubicTo grammar.
    #[must_use]
    pub fn commands(&self) -> &[PathCommandV1] {
        &self.commands
    }
}

/// Failure while lowering an authored presentation path.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PresentationPathErrorV1 {
    /// A finite authored input produced an unrepresentable derived point.
    #[error("authored presentation path produced non-finite derived geometry")]
    NonFiniteDerivedGeometry,
}

/// Lower one authored control path into replayable presentation commands.
///
/// `RenderPoint` validates each authored point at construction. The output
/// copies endpoints exactly, emits straight segments for a polyline, and uses
/// cubic commands for every spline segment. A one-control quadratic is
/// elevated exactly; a three-or-more-control spline follows the established
/// midpoint-chain quadratic policy before applying that elevation.
pub fn lower_authored_control_path_v1(
    kind: PathKindV1,
    start: RenderPoint,
    controls: &[RenderPoint],
    end: RenderPoint,
) -> Result<PresentationPathV1, PresentationPathErrorV1> {
    let mut commands = Vec::with_capacity(controls.len().saturating_add(2));
    commands.push(PathCommandV1::MoveTo(start));

    match kind {
        PathKindV1::Polyline => {
            commands.extend(controls.iter().copied().map(PathCommandV1::LineTo));
            commands.push(PathCommandV1::LineTo(end));
        }
        PathKindV1::AuthoredSpline => match controls {
            [] => commands.push(PathCommandV1::LineTo(end)),
            [control] => append_elevated_quadratic(&mut commands, start, *control, end)?,
            [control_1, control_2] => commands.push(PathCommandV1::CubicTo {
                control_1: *control_1,
                control_2: *control_2,
                end,
            }),
            _ => {
                let mut current = start;
                for pair in controls.windows(2) {
                    let midpoint = midpoint(pair[0], pair[1])?;
                    append_elevated_quadratic(&mut commands, current, pair[0], midpoint)?;
                    current = midpoint;
                }
                append_elevated_quadratic(
                    &mut commands,
                    current,
                    controls[controls.len() - 1],
                    end,
                )?;
            }
        },
    }

    Ok(PresentationPathV1 { kind, commands })
}

fn append_elevated_quadratic(
    commands: &mut Vec<PathCommandV1>,
    start: RenderPoint,
    control: RenderPoint,
    end: RenderPoint,
) -> Result<(), PresentationPathErrorV1> {
    commands.push(PathCommandV1::CubicTo {
        control_1: weighted_point(start, 1.0 / 3.0, control, 2.0 / 3.0)?,
        control_2: weighted_point(end, 1.0 / 3.0, control, 2.0 / 3.0)?,
        end,
    });
    Ok(())
}

fn midpoint(left: RenderPoint, right: RenderPoint) -> Result<RenderPoint, PresentationPathErrorV1> {
    weighted_point(left, 0.5, right, 0.5)
}

fn weighted_point(
    left: RenderPoint,
    left_weight: f64,
    right: RenderPoint,
    right_weight: f64,
) -> Result<RenderPoint, PresentationPathErrorV1> {
    let x = left.x() * left_weight + right.x() * right_weight;
    let y = left.y() * left_weight + right.y() * right_weight;
    RenderPoint::new(x, y).map_err(|_| PresentationPathErrorV1::NonFiniteDerivedGeometry)
}
