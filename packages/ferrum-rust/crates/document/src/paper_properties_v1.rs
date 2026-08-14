//! Closed paper observation and mutation intent for authoritative CDML sessions.

use std::collections::HashSet;

use serde::Serialize;
use thiserror::Error;

use super::{
    DocumentSnapshot, PaperDimensionsMmV1, TypedClass, TypedDocument, TypedRecord, paper_size_v1,
};

const SCENE_POINTS_PER_MILLIMETRE: f64 = 72.0 / 25.4;

/// Stable schema identifier for one paper-layout projection.
pub const PAPER_LAYOUT_PROJECTION_SCHEMA_V1: &str = "ferrum-document-paper-layout-v1";

/// One supported paper orientation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperOrientationV1 {
    /// Portrait page orientation.
    Portrait,
    /// Landscape page orientation.
    Landscape,
}

/// Compatibility problem that required the physical page to use the built-in fallback.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperPageIssueV1 {
    /// The direct paper type is absent or outside the recognized catalog.
    UnsupportedType,
    /// The direct paper orientation is absent or unsupported.
    UnsupportedOrientation,
    /// A custom page is missing positive finite dimensions representable in scene units.
    InvalidCustomDimensions,
}

/// Physical page dimensions and its backend-issued scene rectangle.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PaperPageV1 {
    width_mm: f64,
    height_mm: f64,
    scene_left: f64,
    scene_top: f64,
    scene_right: f64,
    scene_bottom: f64,
    issue: Option<PaperPageIssueV1>,
}

impl PaperPageV1 {
    fn from_dimensions(
        dimensions: PaperDimensionsMmV1,
        orientation: PaperOrientationV1,
        issue: Option<PaperPageIssueV1>,
    ) -> Option<Self> {
        let (width_mm, height_mm) = match orientation {
            PaperOrientationV1::Portrait => (dimensions.width(), dimensions.height()),
            PaperOrientationV1::Landscape => (dimensions.height(), dimensions.width()),
        };
        let scene_right = width_mm * SCENE_POINTS_PER_MILLIMETRE;
        let scene_bottom = height_mm * SCENE_POINTS_PER_MILLIMETRE;
        if !scene_right.is_finite() || !scene_bottom.is_finite() {
            return None;
        }
        Some(Self {
            width_mm,
            height_mm,
            scene_left: 0.0,
            scene_top: 0.0,
            scene_right,
            scene_bottom,
            issue,
        })
    }

    /// Return the oriented physical page width in millimetres.
    #[must_use]
    pub const fn width_mm(self) -> f64 {
        self.width_mm
    }

    /// Return the oriented physical page height in millimetres.
    #[must_use]
    pub const fn height_mm(self) -> f64 {
        self.height_mm
    }

    /// Return the page's left scene coordinate.
    #[must_use]
    pub const fn scene_left(self) -> f64 {
        self.scene_left
    }

    /// Return the page's top scene coordinate.
    #[must_use]
    pub const fn scene_top(self) -> f64 {
        self.scene_top
    }

    /// Return the page's right scene coordinate.
    #[must_use]
    pub const fn scene_right(self) -> f64 {
        self.scene_right
    }

    /// Return the page's bottom scene coordinate.
    #[must_use]
    pub const fn scene_bottom(self) -> f64 {
        self.scene_bottom
    }

    /// Return why compatibility fallback geometry was used, when applicable.
    #[must_use]
    pub const fn issue(self) -> Option<PaperPageIssueV1> {
        self.issue
    }
}

impl PaperOrientationV1 {
    /// Parse one exact CDML orientation token.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "portrait" => Some(Self::Portrait),
            "landscape" => Some(Self::Landscape),
            _ => None,
        }
    }

    /// Return the exact CDML orientation token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Portrait => "portrait",
            Self::Landscape => "landscape",
        }
    }
}

/// Recognized attributes copied from the first direct core paper record.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PaperAttributesV1 {
    id: Option<String>,
    #[serde(rename = "type")]
    type_name: Option<String>,
    orientation: Option<String>,
    crop_svg: Option<String>,
    crop_margin: Option<String>,
    use_real_minus: Option<String>,
    replace_minus: Option<String>,
    size_x: Option<String>,
    size_y: Option<String>,
}

impl PaperAttributesV1 {
    fn from_record(record: &TypedRecord) -> Self {
        Self {
            id: copied(record, "id"),
            type_name: copied(record, "type"),
            orientation: copied(record, "orientation"),
            crop_svg: copied(record, "crop_svg"),
            crop_margin: copied(record, "crop_margin"),
            use_real_minus: copied(record, "use_real_minus"),
            replace_minus: copied(record, "replace_minus"),
            size_x: copied(record, "size_x"),
            size_y: copied(record, "size_y"),
        }
    }

    fn from_defaults(type_name: &str, orientation: PaperOrientationV1) -> Self {
        Self {
            type_name: Some(type_name.to_owned()),
            orientation: Some(orientation.as_str().to_owned()),
            ..Self::default()
        }
    }

    /// Return the authored persistent ID, when present.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
    /// Return the authored paper type token, when present.
    #[must_use]
    pub fn type_name(&self) -> Option<&str> {
        self.type_name.as_deref()
    }
    /// Return the authored orientation token, when present.
    #[must_use]
    pub fn orientation(&self) -> Option<&str> {
        self.orientation.as_deref()
    }
    /// Return the authored crop-SVG token, when present.
    #[must_use]
    pub fn crop_svg(&self) -> Option<&str> {
        self.crop_svg.as_deref()
    }
    /// Return the authored crop-margin token, when present.
    #[must_use]
    pub fn crop_margin(&self) -> Option<&str> {
        self.crop_margin.as_deref()
    }
    /// Return the authored real-minus token, when present.
    #[must_use]
    pub fn use_real_minus(&self) -> Option<&str> {
        self.use_real_minus.as_deref()
    }
    /// Return the authored replace-minus token, when present.
    #[must_use]
    pub fn replace_minus(&self) -> Option<&str> {
        self.replace_minus.as_deref()
    }
    /// Return the authored custom width token, when present.
    #[must_use]
    pub fn size_x(&self) -> Option<&str> {
        self.size_x.as_deref()
    }
    /// Return the authored custom height token, when present.
    #[must_use]
    pub fn size_y(&self) -> Option<&str> {
        self.size_y.as_deref()
    }
}

/// Recognized attributes copied from the first direct core viewport record.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct ViewportAttributesV1 {
    id: Option<String>,
    viewport: Option<String>,
}

impl ViewportAttributesV1 {
    fn from_record(record: &TypedRecord) -> Self {
        Self {
            id: copied(record, "id"),
            viewport: copied(record, "viewport"),
        }
    }
    /// Return the authored viewport ID, when present.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }
    /// Return the authored viewport rectangle token, when present.
    #[must_use]
    pub fn viewport(&self) -> Option<&str> {
        self.viewport.as_deref()
    }
}

/// Revision- and digest-bound paper and viewport observation.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PaperLayoutProjectionV1 {
    schema: &'static str,
    revision: u64,
    digest: [u8; 32],
    paper_present: bool,
    paper_attributes: PaperAttributesV1,
    effective_paper_attributes: PaperAttributesV1,
    viewport_attributes: ViewportAttributesV1,
    default_type: String,
    default_orientation: PaperOrientationV1,
    page: PaperPageV1,
}

impl PaperLayoutProjectionV1 {
    pub(crate) fn from_snapshot(document: &TypedDocument, snapshot: &DocumentSnapshot) -> Self {
        let paper = first_direct(document, TypedClass::Paper);
        let viewport = first_direct(document, TypedClass::Viewport);
        let (default_type, default_orientation) = document.paper_defaults_v1();
        let paper_attributes = paper
            .map(PaperAttributesV1::from_record)
            .unwrap_or_default();
        let effective_paper_attributes = if paper.is_some() {
            paper_attributes.clone()
        } else {
            PaperAttributesV1::from_defaults(&default_type, default_orientation)
        };
        let page = resolve_page(&effective_paper_attributes);
        Self {
            schema: PAPER_LAYOUT_PROJECTION_SCHEMA_V1,
            revision: snapshot.revision(),
            digest: *snapshot.digest(),
            paper_present: paper.is_some(),
            paper_attributes,
            effective_paper_attributes,
            viewport_attributes: viewport
                .map(ViewportAttributesV1::from_record)
                .unwrap_or_default(),
            default_type,
            default_orientation,
            page,
        }
    }

    /// Return the closed wire schema identifier.
    #[must_use]
    pub fn schema(&self) -> &'static str {
        self.schema
    }
    /// Return the owning document revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Return the owning document digest.
    #[must_use]
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    /// Return whether a direct core paper record exists.
    #[must_use]
    pub fn paper_present(&self) -> bool {
        self.paper_present
    }
    /// Return exact authored recognized paper fields.
    #[must_use]
    pub fn paper_attributes(&self) -> &PaperAttributesV1 {
        &self.paper_attributes
    }
    /// Return authored paper fields, or standard-backed creation defaults when absent.
    #[must_use]
    pub fn effective_paper_attributes(&self) -> &PaperAttributesV1 {
        &self.effective_paper_attributes
    }
    /// Return exact authored recognized viewport fields.
    #[must_use]
    pub fn viewport_attributes(&self) -> &ViewportAttributesV1 {
        &self.viewport_attributes
    }
    /// Return the valid named paper type used when creating a paper record.
    #[must_use]
    pub fn default_type(&self) -> &str {
        &self.default_type
    }
    /// Return the valid orientation used when creating a paper record.
    #[must_use]
    pub fn default_orientation(&self) -> PaperOrientationV1 {
        self.default_orientation
    }
    /// Return the backend-issued physical page and scene rectangle.
    #[must_use]
    pub const fn page(&self) -> PaperPageV1 {
        self.page
    }
}

/// One supported paper property change.
#[derive(Clone, Debug, PartialEq)]
pub enum PaperPropertyChangeV1 {
    /// Replace the exact recognized paper type.
    Type(String),
    /// Replace page orientation.
    Orientation(PaperOrientationV1),
    /// Replace the SVG crop flag.
    CropSvg(bool),
    /// Replace the nonnegative SVG crop margin.
    CropMargin(u64),
    /// Replace the real-minus flag.
    UseRealMinus(bool),
    /// Replace the SVG hyphen-replacement flag.
    ReplaceMinus(bool),
    /// Replace custom paper dimensions.
    Dimensions(PaperDimensionsMmV1),
}

impl PaperPropertyChangeV1 {
    fn kind(&self) -> PaperPropertyKindV1 {
        match self {
            Self::Type(_) => PaperPropertyKindV1::Type,
            Self::Orientation(_) => PaperPropertyKindV1::Orientation,
            Self::CropSvg(_) => PaperPropertyKindV1::CropSvg,
            Self::CropMargin(_) => PaperPropertyKindV1::CropMargin,
            Self::UseRealMinus(_) => PaperPropertyKindV1::UseRealMinus,
            Self::ReplaceMinus(_) => PaperPropertyKindV1::ReplaceMinus,
            Self::Dimensions(_) => PaperPropertyKindV1::Dimensions,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PaperPropertyKindV1 {
    Type,
    Orientation,
    CropSvg,
    CropMargin,
    UseRealMinus,
    ReplaceMinus,
    Dimensions,
}

/// One validated unique-field paper-properties patch.
#[derive(Clone, Debug, PartialEq)]
pub struct PaperPropertiesPatchV1 {
    changes: Vec<PaperPropertyChangeV1>,
}

impl PaperPropertiesPatchV1 {
    /// Validate one complete paper edit intent without reading a document.
    pub fn new(changes: Vec<PaperPropertyChangeV1>) -> Result<Self, PaperPropertiesPatchV1Error> {
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            if !kinds.insert(change.kind()) {
                return Err(PaperPropertiesPatchV1Error::DuplicateChange);
            }
            if let PaperPropertyChangeV1::Type(value) = change
                && paper_size_v1(value).is_none()
            {
                return Err(PaperPropertiesPatchV1Error::UnsupportedType);
            }
        }
        let authored_type = changes.iter().find_map(|change| match change {
            PaperPropertyChangeV1::Type(value) => Some(value.as_str()),
            _ => None,
        });
        let has_dimensions = kinds.contains(&PaperPropertyKindV1::Dimensions);
        if authored_type == Some("custom") && !has_dimensions {
            return Err(PaperPropertiesPatchV1Error::CustomRequiresDimensions);
        }
        if authored_type.is_some_and(|value| value != "custom") && has_dimensions {
            return Err(PaperPropertiesPatchV1Error::DimensionsRequireCustom);
        }
        Ok(Self { changes })
    }

    /// Return unique validated changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[PaperPropertyChangeV1] {
        &self.changes
    }
}

/// Invalid paper-properties intent rejected before document mutation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PaperPropertiesPatchV1Error {
    /// A property occurred more than once.
    #[error("paper property change is duplicated")]
    DuplicateChange,
    /// The requested paper type is not in the closed catalog.
    #[error("paper type is unsupported")]
    UnsupportedType,
    /// Custom paper needs an explicit dimension pair.
    #[error("custom paper type requires dimensions")]
    CustomRequiresDimensions,
    /// Dimensions cannot accompany a named paper type.
    #[error("paper dimensions apply only to custom paper")]
    DimensionsRequireCustom,
}

fn copied(record: &TypedRecord, name: &str) -> Option<String> {
    record.attribute(name).map(str::to_owned)
}

fn first_direct(document: &TypedDocument, class: TypedClass) -> Option<&TypedRecord> {
    document
        .root()
        .typed_children()
        .iter()
        .map(|child| child.record())
        .find(|record| record.class() == class)
}

fn resolve_page(attributes: &PaperAttributesV1) -> PaperPageV1 {
    let Some(type_name) = attributes.type_name() else {
        return fallback_page(PaperPageIssueV1::UnsupportedType);
    };
    let Some(paper_size) = paper_size_v1(type_name) else {
        return fallback_page(PaperPageIssueV1::UnsupportedType);
    };
    let Some(orientation) = attributes.orientation().and_then(PaperOrientationV1::parse) else {
        return fallback_page(PaperPageIssueV1::UnsupportedOrientation);
    };
    let dimensions = if paper_size.name() == "custom" {
        let Some(dimensions) = custom_dimensions(attributes) else {
            return fallback_page(PaperPageIssueV1::InvalidCustomDimensions);
        };
        dimensions
    } else {
        paper_size
            .dimensions()
            .expect("a recognized non-custom paper size has fixed dimensions")
    };
    PaperPageV1::from_dimensions(dimensions, orientation, None)
        .unwrap_or_else(|| fallback_page(PaperPageIssueV1::InvalidCustomDimensions))
}

fn custom_dimensions(attributes: &PaperAttributesV1) -> Option<PaperDimensionsMmV1> {
    let width = attributes.size_x()?.parse().ok()?;
    let height = attributes.size_y()?.parse().ok()?;
    PaperDimensionsMmV1::try_new(width, height).ok()
}

fn fallback_page(issue: PaperPageIssueV1) -> PaperPageV1 {
    let dimensions = paper_size_v1("A4")
        .and_then(|paper| paper.dimensions())
        .expect("the closed paper catalog contains fixed A4 dimensions");
    PaperPageV1::from_dimensions(dimensions, PaperOrientationV1::Portrait, Some(issue))
        .expect("closed A4 dimensions are finite in scene units")
}
