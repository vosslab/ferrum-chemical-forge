//! Closed durable-root grammar for top-level document transforms.

use std::collections::HashSet;

use thiserror::Error;

use super::PersistentId;

/// Direct-root record kinds supported by the transform boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TopLevelRootKindV1 {
    Molecule,
    Arrow,
    Plus,
    Text,
    Rectangle,
    Square,
    Oval,
    Circle,
    Polygon,
    Polyline,
}

impl TopLevelRootKindV1 {
    pub(crate) const fn local_name(self) -> &'static str {
        match self {
            Self::Molecule => "molecule",
            Self::Arrow => "arrow",
            Self::Plus => "plus",
            Self::Text => "text",
            Self::Rectangle => "rect",
            Self::Square => "square",
            Self::Oval => "oval",
            Self::Circle => "circle",
            Self::Polygon => "polygon",
            Self::Polyline => "polyline",
        }
    }
}

/// One exact-kind durable direct-root selector.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopLevelRootSelectorV1 {
    root_id: PersistentId,
    kind: TopLevelRootKindV1,
}

impl TopLevelRootSelectorV1 {
    pub fn new(
        root_id: impl Into<String>,
        kind: TopLevelRootKindV1,
    ) -> Result<Self, TopLevelTransformV1Error> {
        let root_id = PersistentId::new(root_id.into())
            .map_err(|_| TopLevelTransformV1Error::InvalidRootId)?;
        Ok(Self { root_id, kind })
    }

    #[must_use]
    pub fn root_id(&self) -> &PersistentId {
        &self.root_id
    }

    #[must_use]
    pub const fn kind(&self) -> TopLevelRootKindV1 {
        self.kind
    }
}

/// Private common transforms over complete durable direct roots.
///
/// The public session-operation boundary names layout and interaction
/// translation separately. This representation is only the shared lowering
/// detail beneath those semantic requests.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TopLevelTransformModeV1 {
    Translate { dx: f64, dy: f64 },
    AlignTop,
    AlignBottom,
    AlignLeft,
    AlignRight,
    AlignCenterX,
    AlignCenterY,
    Scale { scale_x: f64, scale_y: f64 },
    MirrorVertical,
    MirrorHorizontal,
}

/// Private common validated intent for one atomic top-level transform.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TopLevelTransformV1 {
    targets: Vec<TopLevelRootSelectorV1>,
    transform: TopLevelTransformModeV1,
}

impl TopLevelTransformV1 {
    pub(crate) fn new(
        targets: Vec<TopLevelRootSelectorV1>,
        transform: TopLevelTransformModeV1,
    ) -> Result<Self, TopLevelTransformV1Error> {
        if targets.is_empty() {
            return Err(TopLevelTransformV1Error::EmptyTargets);
        }
        let mut identifiers = HashSet::with_capacity(targets.len());
        if targets
            .iter()
            .any(|target| !identifiers.insert(target.root_id().clone()))
        {
            return Err(TopLevelTransformV1Error::DuplicateTarget);
        }
        if matches!(
            transform,
            TopLevelTransformModeV1::AlignTop
                | TopLevelTransformModeV1::AlignBottom
                | TopLevelTransformModeV1::AlignLeft
                | TopLevelTransformModeV1::AlignRight
                | TopLevelTransformModeV1::AlignCenterX
                | TopLevelTransformModeV1::AlignCenterY
        ) && targets.len() < 2
        {
            return Err(TopLevelTransformV1Error::AlignmentNeedsTwoTargets);
        }
        if let TopLevelTransformModeV1::Translate { dx, dy } = transform
            && (!dx.is_finite() || !dy.is_finite())
        {
            return Err(TopLevelTransformV1Error::NonFiniteTranslation);
        }
        if let TopLevelTransformModeV1::Scale { scale_x, scale_y } = transform
            && (!scale_x.is_finite() || !scale_y.is_finite() || scale_x <= 0.0 || scale_y <= 0.0)
        {
            return Err(TopLevelTransformV1Error::InvalidScaleFactors);
        }
        Ok(Self { targets, transform })
    }

    #[must_use]
    pub(crate) fn targets(&self) -> &[TopLevelRootSelectorV1] {
        &self.targets
    }

    #[must_use]
    pub(crate) const fn transform(&self) -> TopLevelTransformModeV1 {
        self.transform
    }
}

/// Closed capability-free layout transforms over complete durable direct roots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TopLevelRootLayoutTransformModeV1 {
    AlignTop,
    AlignBottom,
    AlignLeft,
    AlignRight,
    AlignCenterX,
    AlignCenterY,
    Scale { scale_x: f64, scale_y: f64 },
    MirrorVertical,
    MirrorHorizontal,
}

impl From<TopLevelRootLayoutTransformModeV1> for TopLevelTransformModeV1 {
    fn from(value: TopLevelRootLayoutTransformModeV1) -> Self {
        match value {
            TopLevelRootLayoutTransformModeV1::AlignTop => Self::AlignTop,
            TopLevelRootLayoutTransformModeV1::AlignBottom => Self::AlignBottom,
            TopLevelRootLayoutTransformModeV1::AlignLeft => Self::AlignLeft,
            TopLevelRootLayoutTransformModeV1::AlignRight => Self::AlignRight,
            TopLevelRootLayoutTransformModeV1::AlignCenterX => Self::AlignCenterX,
            TopLevelRootLayoutTransformModeV1::AlignCenterY => Self::AlignCenterY,
            TopLevelRootLayoutTransformModeV1::Scale { scale_x, scale_y } => {
                Self::Scale { scale_x, scale_y }
            }
            TopLevelRootLayoutTransformModeV1::MirrorVertical => Self::MirrorVertical,
            TopLevelRootLayoutTransformModeV1::MirrorHorizontal => Self::MirrorHorizontal,
        }
    }
}

/// Complete validated direct document-edit request for one root layout transform.
#[derive(Clone, Debug, PartialEq)]
pub struct TopLevelRootLayoutTransformV1 {
    transform: TopLevelTransformV1,
}

impl TopLevelRootLayoutTransformV1 {
    pub fn new(
        targets: Vec<TopLevelRootSelectorV1>,
        mode: TopLevelRootLayoutTransformModeV1,
    ) -> Result<Self, TopLevelTransformV1Error> {
        Ok(Self {
            transform: TopLevelTransformV1::new(targets, mode.into())?,
        })
    }

    #[must_use]
    pub fn targets(&self) -> &[TopLevelRootSelectorV1] {
        self.transform.targets()
    }

    #[must_use]
    pub fn mode(&self) -> TopLevelRootLayoutTransformModeV1 {
        match self.transform.transform() {
            TopLevelTransformModeV1::AlignTop => TopLevelRootLayoutTransformModeV1::AlignTop,
            TopLevelTransformModeV1::AlignBottom => TopLevelRootLayoutTransformModeV1::AlignBottom,
            TopLevelTransformModeV1::AlignLeft => TopLevelRootLayoutTransformModeV1::AlignLeft,
            TopLevelTransformModeV1::AlignRight => TopLevelRootLayoutTransformModeV1::AlignRight,
            TopLevelTransformModeV1::AlignCenterX => {
                TopLevelRootLayoutTransformModeV1::AlignCenterX
            }
            TopLevelTransformModeV1::AlignCenterY => {
                TopLevelRootLayoutTransformModeV1::AlignCenterY
            }
            TopLevelTransformModeV1::Scale { scale_x, scale_y } => {
                TopLevelRootLayoutTransformModeV1::Scale { scale_x, scale_y }
            }
            TopLevelTransformModeV1::MirrorVertical => {
                TopLevelRootLayoutTransformModeV1::MirrorVertical
            }
            TopLevelTransformModeV1::MirrorHorizontal => {
                TopLevelRootLayoutTransformModeV1::MirrorHorizontal
            }
            TopLevelTransformModeV1::Translate { .. } => {
                unreachable!("layout transforms never contain translations")
            }
        }
    }

    pub(crate) fn common_transform(&self) -> &TopLevelTransformV1 {
        &self.transform
    }
}

/// Complete validated interaction-derived durable root translation request.
#[derive(Clone, Debug, PartialEq)]
pub struct TopLevelRootTranslationV1 {
    transform: TopLevelTransformV1,
}

impl TopLevelRootTranslationV1 {
    pub fn new(
        targets: Vec<TopLevelRootSelectorV1>,
        dx: f64,
        dy: f64,
    ) -> Result<Self, TopLevelTransformV1Error> {
        Ok(Self {
            transform: TopLevelTransformV1::new(
                targets,
                TopLevelTransformModeV1::Translate { dx, dy },
            )?,
        })
    }

    #[must_use]
    pub fn targets(&self) -> &[TopLevelRootSelectorV1] {
        self.transform.targets()
    }

    #[must_use]
    pub fn dx(&self) -> f64 {
        match self.transform.transform() {
            TopLevelTransformModeV1::Translate { dx, .. } => dx,
            _ => unreachable!("translation requests always contain a translation"),
        }
    }

    #[must_use]
    pub fn dy(&self) -> f64 {
        match self.transform.transform() {
            TopLevelTransformModeV1::Translate { dy, .. } => dy,
            _ => unreachable!("translation requests always contain a translation"),
        }
    }

    pub(crate) fn common_transform(&self) -> &TopLevelTransformV1 {
        &self.transform
    }
}

/// Invalid transform intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TopLevelTransformV1Error {
    #[error("top-level transform requires a valid persistent root ID")]
    InvalidRootId,
    #[error("top-level transform requires at least one root")]
    EmptyTargets,
    #[error("top-level transform roots must be unique")]
    DuplicateTarget,
    #[error("top-level alignment requires at least two roots")]
    AlignmentNeedsTwoTargets,
    #[error("top-level translation requires finite point values")]
    NonFiniteTranslation,
    #[error("top-level scale factors must be finite and greater than zero")]
    InvalidScaleFactors,
}
