//! Checked generic vector operations for one already-authoritative document root.
//!
//! These types preserve scene-space geometry and explicit paint without learning
//! about CDML, document records, Qt items, or a renderer backend.

use crate::{Paint, PositiveFinite, RenderError, RenderPoint};

/// Fixed V1 cap semantics for a vector operation with a stroke.
///
/// The current document projection supplies no cap alternatives, so V1 fixes this
/// value instead of leaving it to a backend default. A later source-backed choice
/// requires a new validated vector grammar revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorStrokeLineCapV1 {
    /// End an open stroke exactly at its issued endpoint.
    Butt,
}

impl VectorStrokeLineCapV1 {
    /// Return the backend-neutral V1 cap selected for every issued stroke.
    #[must_use]
    pub const fn v1() -> Self {
        Self::Butt
    }

    /// Return the matching SVG keyword.
    #[must_use]
    pub const fn svg_keyword(self) -> &'static str {
        match self {
            Self::Butt => "butt",
        }
    }
}

/// Fixed V1 join semantics for a vector operation with a stroke.
///
/// The current document projection supplies no join alternatives, so V1 fixes this
/// value instead of leaving it to a backend default. A later source-backed choice
/// requires a new validated vector grammar revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorStrokeLineJoinV1 {
    /// Join adjacent stroked segments with a miter.
    Miter,
}

impl VectorStrokeLineJoinV1 {
    /// Return the backend-neutral V1 join selected for every issued stroke.
    #[must_use]
    pub const fn v1() -> Self {
        Self::Miter
    }

    /// Return the matching SVG keyword.
    #[must_use]
    pub const fn svg_keyword(self) -> &'static str {
        match self {
            Self::Miter => "miter",
        }
    }

    /// Return the fixed V1 miter ratio before a sink bevels an acute join.
    ///
    /// This makes the SVG-compatible V1 profile material instead of relying on
    /// the initial state of an individual drawing backend.
    #[must_use]
    pub const fn miter_limit(self) -> f64 {
        match self {
            Self::Miter => 4.0,
        }
    }
}

/// Fixed V1 fill semantics for a filled closed vector path.
///
/// The current document projection supplies no fill-rule alternatives, so V1 fixes
/// this value instead of leaving it to a backend default. Ellipses are one closed
/// primitive and therefore have no fill-rule choice in this grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VectorFillRuleV1 {
    /// Fill regions by even-odd parity across all closed subpaths.
    EvenOdd,
}

impl VectorFillRuleV1 {
    /// Return the backend-neutral V1 rule selected for every filled path.
    #[must_use]
    pub const fn v1() -> Self {
        Self::EvenOdd
    }

    /// Return the matching SVG keyword.
    #[must_use]
    pub const fn svg_keyword(self) -> &'static str {
        match self {
            Self::EvenOdd => "evenodd",
        }
    }
}

/// One explicit stroke with its required positive width.
#[derive(Clone, Debug, PartialEq)]
pub struct StrokeV1 {
    paint: Paint,
    width: PositiveFinite,
}

impl StrokeV1 {
    /// Construct a fully specified stroke without toolkit defaults.
    #[must_use]
    pub const fn new(paint: Paint, width: PositiveFinite) -> Self {
        Self { paint, width }
    }

    /// Return the explicit stroke paint.
    #[must_use]
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }

    /// Return the explicit positive stroke width.
    #[must_use]
    pub const fn width(&self) -> PositiveFinite {
        self.width
    }

    /// Return the fixed V1 cap for this explicit stroke.
    #[must_use]
    pub const fn line_cap(&self) -> VectorStrokeLineCapV1 {
        VectorStrokeLineCapV1::v1()
    }

    /// Return the fixed V1 join for this explicit stroke.
    #[must_use]
    pub const fn line_join(&self) -> VectorStrokeLineJoinV1 {
        VectorStrokeLineJoinV1::v1()
    }

    /// Return the fixed V1 miter ratio before a sink bevels an acute join.
    #[must_use]
    pub const fn miter_limit(&self) -> f64 {
        self.line_join().miter_limit()
    }
}

/// One command in a scene-space vector path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommandV1 {
    /// Begin a new subpath at an exact scene point.
    MoveTo(RenderPoint),
    /// Add an exact straight segment to the current subpath.
    LineTo(RenderPoint),
    /// Add an exact cubic Bezier segment to the current subpath.
    CubicTo {
        /// First exact cubic control point.
        control_1: RenderPoint,
        /// Second exact cubic control point.
        control_2: RenderPoint,
        /// Exact cubic endpoint.
        end: RenderPoint,
    },
    /// Close the current drawable subpath exactly once.
    Close,
}

/// Renderer-neutral vector paint for one direct document root.
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentVectorOpV1 {
    /// A validated path with explicit outline and/or fill paint.
    Path {
        commands: Vec<PathCommandV1>,
        stroke: Option<StrokeV1>,
        fill: Option<Paint>,
    },
    /// A validated axis-aligned ellipse with explicit outline and/or fill paint.
    Ellipse {
        center: RenderPoint,
        radius_x: PositiveFinite,
        radius_y: PositiveFinite,
        stroke: Option<StrokeV1>,
        fill: Option<Paint>,
    },
}

impl DocumentVectorOpV1 {
    /// Construct a checked path with the fixed V1 butt, 4.0-miter, and even-odd profile.
    pub fn path(
        commands: Vec<PathCommandV1>,
        stroke: Option<StrokeV1>,
        fill: Option<Paint>,
    ) -> Result<Self, RenderError> {
        validate_path(&commands, fill.is_some())?;
        validate_paint(&stroke, &fill)?;
        Ok(Self::Path {
            commands,
            stroke,
            fill,
        })
    }

    /// Construct a checked, positive-radius ellipse with explicit paint.
    pub fn ellipse(
        center: RenderPoint,
        radius_x: PositiveFinite,
        radius_y: PositiveFinite,
        stroke: Option<StrokeV1>,
        fill: Option<Paint>,
    ) -> Result<Self, RenderError> {
        validate_paint(&stroke, &fill)?;
        Ok(Self::Ellipse {
            center,
            radius_x,
            radius_y,
            stroke,
            fill,
        })
    }

    /// Return the ordered exact commands when this operation is a path.
    #[must_use]
    pub fn commands(&self) -> Option<&[PathCommandV1]> {
        match self {
            Self::Path { commands, .. } => Some(commands),
            Self::Ellipse { .. } => None,
        }
    }

    /// Return the explicit stroke when this operation has one.
    #[must_use]
    pub fn stroke(&self) -> Option<&StrokeV1> {
        match self {
            Self::Path { stroke, .. } | Self::Ellipse { stroke, .. } => stroke.as_ref(),
        }
    }

    /// Return the explicit fill when this operation has one.
    #[must_use]
    pub fn fill(&self) -> Option<&Paint> {
        match self {
            Self::Path { fill, .. } | Self::Ellipse { fill, .. } => fill.as_ref(),
        }
    }

    /// Return the fixed V1 cap when this operation has a stroke.
    #[must_use]
    pub const fn stroke_line_cap(&self) -> Option<VectorStrokeLineCapV1> {
        match self {
            Self::Path { stroke, .. } | Self::Ellipse { stroke, .. } if stroke.is_some() => {
                Some(VectorStrokeLineCapV1::v1())
            }
            Self::Path { .. } | Self::Ellipse { .. } => None,
        }
    }

    /// Return the fixed V1 join when this operation has a stroke.
    #[must_use]
    pub const fn stroke_line_join(&self) -> Option<VectorStrokeLineJoinV1> {
        match self {
            Self::Path { stroke, .. } | Self::Ellipse { stroke, .. } if stroke.is_some() => {
                Some(VectorStrokeLineJoinV1::v1())
            }
            Self::Path { .. } | Self::Ellipse { .. } => None,
        }
    }

    /// Return the fixed V1 miter ratio when this operation has a stroke.
    #[must_use]
    pub const fn stroke_miter_limit(&self) -> Option<f64> {
        match self {
            Self::Path { stroke, .. } | Self::Ellipse { stroke, .. } if stroke.is_some() => {
                Some(VectorStrokeLineJoinV1::v1().miter_limit())
            }
            Self::Path { .. } | Self::Ellipse { .. } => None,
        }
    }

    /// Return the fixed V1 fill rule for a filled closed path.
    ///
    /// Ellipses are single closed primitives, so their fill needs no rule choice.
    #[must_use]
    pub const fn fill_rule(&self) -> Option<VectorFillRuleV1> {
        match self {
            Self::Path { fill, .. } if fill.is_some() => Some(VectorFillRuleV1::v1()),
            Self::Path { .. } | Self::Ellipse { .. } => None,
        }
    }

    /// Return exact ellipse geometry, if this is an ellipse operation.
    #[must_use]
    pub fn ellipse_geometry(&self) -> Option<(RenderPoint, PositiveFinite, PositiveFinite)> {
        match self {
            Self::Ellipse {
                center,
                radius_x,
                radius_y,
                ..
            } => Some((*center, *radius_x, *radius_y)),
            Self::Path { .. } => None,
        }
    }
}

/// Ordered validated vector paints for one direct root.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentVectorRootV1 {
    operations: Vec<DocumentVectorOpV1>,
}

impl DocumentVectorRootV1 {
    /// Construct a nonempty root-local operation list in exact paint order.
    pub fn new(operations: Vec<DocumentVectorOpV1>) -> Result<Self, RenderError> {
        if operations.is_empty() {
            return Err(RenderError::InvalidRequest(
                "document vector root must contain at least one operation".to_owned(),
            ));
        }
        Ok(Self { operations })
    }

    /// Return vector operations in their required root-local paint order.
    #[must_use]
    pub fn operations(&self) -> &[DocumentVectorOpV1] {
        &self.operations
    }
}

fn validate_paint(stroke: &Option<StrokeV1>, fill: &Option<Paint>) -> Result<(), RenderError> {
    if stroke.is_none() && fill.is_none() {
        return Err(RenderError::InvalidRequest(
            "document vector operation requires an explicit stroke or fill".to_owned(),
        ));
    }
    Ok(())
}

fn validate_path(commands: &[PathCommandV1], filled: bool) -> Result<(), RenderError> {
    let Some(PathCommandV1::MoveTo(_)) = commands.first() else {
        return Err(RenderError::InvalidRequest(
            "document vector path must begin with MoveTo".to_owned(),
        ));
    };

    let mut drawable = false;
    let mut closed = false;
    let mut has_drawable_subpath = false;
    let mut has_current_subpath = false;
    for command in commands {
        match command {
            PathCommandV1::MoveTo(_) => {
                if !closed && drawable && filled {
                    return Err(RenderError::InvalidRequest(
                        "filled document vector path requires every subpath to close".to_owned(),
                    ));
                }
                if has_current_subpath && !drawable {
                    return Err(RenderError::InvalidRequest(
                        "document vector path cannot contain an empty subpath".to_owned(),
                    ));
                }
                drawable = false;
                closed = false;
                has_current_subpath = true;
            }
            PathCommandV1::LineTo(_) | PathCommandV1::CubicTo { .. } => {
                if closed {
                    return Err(RenderError::InvalidRequest(
                        "document vector path cannot draw after Close without MoveTo".to_owned(),
                    ));
                }
                drawable = true;
                has_drawable_subpath = true;
            }
            PathCommandV1::Close => {
                if !drawable || closed {
                    return Err(RenderError::InvalidRequest(
                        "document vector path can close only one drawable subpath".to_owned(),
                    ));
                }
                closed = true;
            }
        }
    }
    if !has_drawable_subpath {
        return Err(RenderError::InvalidRequest(
            "document vector path requires a drawable subpath".to_owned(),
        ));
    }
    if !closed && !drawable {
        return Err(RenderError::InvalidRequest(
            "document vector path cannot end with an empty subpath".to_owned(),
        ));
    }
    if filled && !closed {
        return Err(RenderError::InvalidRequest(
            "filled document vector path requires every subpath to close".to_owned(),
        ));
    }
    Ok(())
}
