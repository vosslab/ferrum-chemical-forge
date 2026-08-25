//! Immutable paper-layout projection values.

use serde::Serialize;

use crate::PositiveFiniteV1;

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
    /// Construct one page from already-resolved physical dimensions.
    #[must_use]
    pub fn from_resolved_dimensions(
        width_mm: PositiveFiniteV1,
        height_mm: PositiveFiniteV1,
        orientation: PaperOrientationV1,
        issue: Option<PaperPageIssueV1>,
    ) -> Option<Self> {
        let width_mm = width_mm.value();
        let height_mm = height_mm.value();
        let (width_mm, height_mm) = match orientation {
            PaperOrientationV1::Portrait => (width_mm, height_mm),
            PaperOrientationV1::Landscape => (height_mm, width_mm),
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

/// Raw recognized tokens copied from one direct core paper record.
///
/// This input deliberately groups the complete authored paper appearance before
/// it becomes the closed [`PaperAttributesV1`] wire value.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PaperAttributeTokensV1 {
    pub id: Option<String>,
    pub type_name: Option<String>,
    pub orientation: Option<String>,
    pub crop_svg: Option<String>,
    pub crop_margin: Option<String>,
    pub use_real_minus: Option<String>,
    pub replace_minus: Option<String>,
    pub size_x: Option<String>,
    pub size_y: Option<String>,
}

impl PaperAttributesV1 {
    /// Construct closed paper attributes from one copied paper record.
    #[must_use]
    pub fn from_tokens(tokens: PaperAttributeTokensV1) -> Self {
        Self {
            id: tokens.id,
            type_name: tokens.type_name,
            orientation: tokens.orientation,
            crop_svg: tokens.crop_svg,
            crop_margin: tokens.crop_margin,
            use_real_minus: tokens.use_real_minus,
            replace_minus: tokens.replace_minus,
            size_x: tokens.size_x,
            size_y: tokens.size_y,
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
    /// Construct copied recognized viewport attributes.
    #[must_use]
    pub fn new(id: Option<String>, viewport: Option<String>) -> Self {
        Self { id, viewport }
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

/// Document-resolved paper layout facts for one immutable projection.
///
/// These values are already typed or closed values. The projection retains its
/// own serialized fields so this construction input never changes the V1 wire
/// schema.
#[derive(Clone, Debug, PartialEq)]
pub struct PaperLayoutFactsV1 {
    pub paper_present: bool,
    pub paper_attributes: PaperAttributesV1,
    pub effective_paper_attributes: PaperAttributesV1,
    pub viewport_attributes: ViewportAttributesV1,
    pub default_type: String,
    pub default_orientation: PaperOrientationV1,
    pub page: PaperPageV1,
}

impl PaperLayoutProjectionV1 {
    /// Construct a projection from document-resolved paper facts.
    #[must_use]
    pub fn new(revision: u64, digest: [u8; 32], facts: PaperLayoutFactsV1) -> Self {
        Self {
            schema: PAPER_LAYOUT_PROJECTION_SCHEMA_V1,
            revision,
            digest,
            paper_present: facts.paper_present,
            paper_attributes: facts.paper_attributes,
            effective_paper_attributes: facts.effective_paper_attributes,
            viewport_attributes: facts.viewport_attributes,
            default_type: facts.default_type,
            default_orientation: facts.default_orientation,
            page: facts.page,
        }
    }

    /// Return the closed wire schema identifier.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the owning document revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Return the owning document digest.
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    /// Return whether a direct core paper record exists.
    #[must_use]
    pub const fn paper_present(&self) -> bool {
        self.paper_present
    }

    /// Return exact authored recognized paper fields.
    #[must_use]
    pub const fn paper_attributes(&self) -> &PaperAttributesV1 {
        &self.paper_attributes
    }

    /// Return authored paper fields, or standard-backed creation defaults when absent.
    #[must_use]
    pub const fn effective_paper_attributes(&self) -> &PaperAttributesV1 {
        &self.effective_paper_attributes
    }

    /// Return exact authored recognized viewport fields.
    #[must_use]
    pub const fn viewport_attributes(&self) -> &ViewportAttributesV1 {
        &self.viewport_attributes
    }

    /// Return the valid named paper type used when creating a paper record.
    #[must_use]
    pub fn default_type(&self) -> &str {
        &self.default_type
    }

    /// Return the valid orientation used when creating a paper record.
    #[must_use]
    pub const fn default_orientation(&self) -> PaperOrientationV1 {
        self.default_orientation
    }

    /// Return the backend-issued physical page and scene rectangle.
    #[must_use]
    pub const fn page(&self) -> PaperPageV1 {
        self.page
    }
}

#[cfg(test)]
mod tests {
    use super::{PaperOrientationV1, PaperPageIssueV1, PaperPageV1, PositiveFiniteV1};

    #[test]
    fn resolved_page_preserves_orientation_and_scene_dimensions() {
        let page = PaperPageV1::from_resolved_dimensions(
            PositiveFiniteV1::new(210.0).expect("positive finite width is valid"),
            PositiveFiniteV1::new(297.0).expect("positive finite height is valid"),
            PaperOrientationV1::Landscape,
            Some(PaperPageIssueV1::UnsupportedType),
        )
        .expect("finite dimensions must form a page");

        assert_eq!((page.width_mm(), page.height_mm()), (297.0, 210.0));
        assert_eq!(page.issue(), Some(PaperPageIssueV1::UnsupportedType));
        assert_eq!(
            PaperOrientationV1::parse(PaperOrientationV1::Portrait.as_str()),
            Some(PaperOrientationV1::Portrait)
        );
    }
}
