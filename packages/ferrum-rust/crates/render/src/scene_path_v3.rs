//! Neutral, checked scene-path facts for the V3 molecule render grammar.
//!
//! This module owns geometry and paint validation only.  Document-vector roots
//! retain their V1 ownership boundary and lower through the same private draw
//! stream; molecule depiction can therefore publish paths without depending on
//! document-vector records.

use crate::{
    PositiveFinite, RenderError, RenderPaintV3, RenderPoint, VectorFillRuleV1,
    VectorStrokeLineCapV1, VectorStrokeLineJoinV1,
};

/// One explicit V3 path stroke with no backend-selected defaults.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenePathStrokeV3 {
    paint: RenderPaintV3,
    width: PositiveFinite,
    line_cap: VectorStrokeLineCapV1,
}

impl ScenePathStrokeV3 {
    /// Construct an explicit stroke.
    #[must_use]
    pub const fn new(paint: RenderPaintV3, width: PositiveFinite) -> Self {
        Self {
            paint,
            width,
            line_cap: VectorStrokeLineCapV1::Butt,
        }
    }
    /// Attach the source-owned cap selected for this explicit path stroke.
    #[must_use]
    pub const fn with_line_cap(mut self, line_cap: VectorStrokeLineCapV1) -> Self {
        self.line_cap = line_cap;
        self
    }
    #[must_use]
    pub fn paint(&self) -> &RenderPaintV3 {
        &self.paint
    }
    #[must_use]
    pub const fn width(&self) -> PositiveFinite {
        self.width
    }
    #[must_use]
    pub const fn line_cap(&self) -> VectorStrokeLineCapV1 {
        self.line_cap
    }
    #[must_use]
    pub const fn line_join(&self) -> VectorStrokeLineJoinV1 {
        VectorStrokeLineJoinV1::v1()
    }
    #[must_use]
    pub const fn miter_limit(&self) -> f64 {
        VectorStrokeLineJoinV1::v1().miter_limit()
    }
}

/// One finite command in a backend-neutral V3 scene path.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "command", rename_all = "snake_case")]
pub enum ScenePathCommandV3 {
    MoveTo(RenderPoint),
    LineTo(RenderPoint),
    CubicTo {
        control_1: RenderPoint,
        control_2: RenderPoint,
        end: RenderPoint,
    },
    Close,
}

/// A checked scene path with explicit paint and deterministic z order.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct PathOpV3 {
    commands: Vec<ScenePathCommandV3>,
    stroke: Option<ScenePathStrokeV3>,
    fill: Option<RenderPaintV3>,
    z: i32,
}

impl PathOpV3 {
    /// Construct a nonempty path whose finite commands and paint are explicit.
    pub fn new(
        commands: Vec<ScenePathCommandV3>,
        stroke: Option<ScenePathStrokeV3>,
        fill: Option<RenderPaintV3>,
        z: i32,
    ) -> Result<Self, RenderError> {
        validate_path(&commands, fill.is_some())?;
        if stroke.is_none() && fill.is_none() {
            return Err(RenderError::InvalidRequest(
                "scene path requires an explicit stroke or fill".to_owned(),
            ));
        }
        Ok(Self {
            commands,
            stroke,
            fill,
            z,
        })
    }
    #[must_use]
    pub fn commands(&self) -> &[ScenePathCommandV3] {
        &self.commands
    }
    #[must_use]
    pub fn stroke(&self) -> Option<&ScenePathStrokeV3> {
        self.stroke.as_ref()
    }
    #[must_use]
    pub fn fill(&self) -> Option<&RenderPaintV3> {
        self.fill.as_ref()
    }
    #[must_use]
    pub const fn fill_rule(&self) -> Option<VectorFillRuleV1> {
        if self.fill.is_some() {
            Some(VectorFillRuleV1::v1())
        } else {
            None
        }
    }
    #[must_use]
    pub const fn z(&self) -> i32 {
        self.z
    }
}

impl<'de> serde::Deserialize<'de> for PathOpV3 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            commands: Vec<ScenePathCommandV3>,
            stroke: Option<ScenePathStrokeV3>,
            fill: Option<RenderPaintV3>,
            z: i32,
        }
        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.commands, wire.stroke, wire.fill, wire.z).map_err(serde::de::Error::custom)
    }
}

fn validate_path(commands: &[ScenePathCommandV3], filled: bool) -> Result<(), RenderError> {
    let Some(ScenePathCommandV3::MoveTo(_)) = commands.first() else {
        return Err(RenderError::InvalidRequest(
            "scene path must begin with MoveTo".to_owned(),
        ));
    };
    let mut drawable = false;
    let mut closed = false;
    let mut current = false;
    let mut any = false;
    for command in commands {
        match command {
            ScenePathCommandV3::MoveTo(_) => {
                if current && !drawable {
                    return Err(RenderError::InvalidRequest(
                        "scene path cannot contain an empty subpath".to_owned(),
                    ));
                }
                if filled && current && !closed {
                    return Err(RenderError::InvalidRequest(
                        "filled scene path requires every subpath to close".to_owned(),
                    ));
                }
                drawable = false;
                closed = false;
                current = true;
            }
            ScenePathCommandV3::LineTo(_) | ScenePathCommandV3::CubicTo { .. } => {
                if closed {
                    return Err(RenderError::InvalidRequest(
                        "scene path cannot draw after Close without MoveTo".to_owned(),
                    ));
                }
                drawable = true;
                any = true;
            }
            ScenePathCommandV3::Close => {
                if !drawable || closed {
                    return Err(RenderError::InvalidRequest(
                        "scene path can close only one drawable subpath".to_owned(),
                    ));
                }
                closed = true;
            }
        }
    }
    if !any {
        return Err(RenderError::InvalidRequest(
            "scene path requires a drawable subpath".to_owned(),
        ));
    }
    if current && !drawable {
        return Err(RenderError::InvalidRequest(
            "scene path cannot end with an empty subpath".to_owned(),
        ));
    }
    if filled && !closed {
        return Err(RenderError::InvalidRequest(
            "filled scene path requires every subpath to close".to_owned(),
        ));
    }
    Ok(())
}
