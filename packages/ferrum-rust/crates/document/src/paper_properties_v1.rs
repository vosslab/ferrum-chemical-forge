//! Closed paper observation and mutation intent for authoritative CDML sessions.

use std::collections::HashSet;

use thiserror::Error;

use ferrum_document_projection::{
    PaperAttributesV1, PaperLayoutProjectionV1, PaperOrientationV1, PaperPageIssueV1, PaperPageV1,
    PositiveFiniteV1, ViewportAttributesV1,
};

use super::{
    DocumentSnapshot, PaperDimensionsMmV1, TypedClass, TypedDocument, TypedRecord, paper_size_v1,
};

pub(crate) fn paper_layout_from_snapshot(
    document: &TypedDocument,
    snapshot: &DocumentSnapshot,
) -> PaperLayoutProjectionV1 {
    let paper = first_direct(document, TypedClass::Paper);
    let viewport = first_direct(document, TypedClass::Viewport);
    let (default_type, default_orientation) = document.paper_defaults_v1();
    let paper_attributes = paper.map(paper_attributes_from_record).unwrap_or_default();
    let effective_paper_attributes = if paper.is_some() {
        paper_attributes.clone()
    } else {
        paper_attributes_from_defaults(&default_type, default_orientation)
    };
    PaperLayoutProjectionV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        paper.is_some(),
        paper_attributes,
        effective_paper_attributes.clone(),
        viewport
            .map(viewport_attributes_from_record)
            .unwrap_or_default(),
        default_type,
        default_orientation,
        resolve_page(&effective_paper_attributes),
    )
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

fn paper_attributes_from_record(record: &TypedRecord) -> PaperAttributesV1 {
    PaperAttributesV1::new(
        copied(record, "id"),
        copied(record, "type"),
        copied(record, "orientation"),
        copied(record, "crop_svg"),
        copied(record, "crop_margin"),
        copied(record, "use_real_minus"),
        copied(record, "replace_minus"),
        copied(record, "size_x"),
        copied(record, "size_y"),
    )
}

fn paper_attributes_from_defaults(
    type_name: &str,
    orientation: PaperOrientationV1,
) -> PaperAttributesV1 {
    PaperAttributesV1::new(
        None,
        Some(type_name.to_owned()),
        Some(orientation.as_str().to_owned()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
}

fn viewport_attributes_from_record(record: &TypedRecord) -> ViewportAttributesV1 {
    ViewportAttributesV1::new(copied(record, "id"), copied(record, "viewport"))
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
    let Some(width_mm) = PositiveFiniteV1::new(dimensions.width()) else {
        return fallback_page(PaperPageIssueV1::InvalidCustomDimensions);
    };
    let Some(height_mm) = PositiveFiniteV1::new(dimensions.height()) else {
        return fallback_page(PaperPageIssueV1::InvalidCustomDimensions);
    };
    PaperPageV1::from_resolved_dimensions(width_mm, height_mm, orientation, None)
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
    let width_mm = PositiveFiniteV1::new(dimensions.width())
        .expect("closed A4 dimensions have positive finite width");
    let height_mm = PositiveFiniteV1::new(dimensions.height())
        .expect("closed A4 dimensions have positive finite height");
    PaperPageV1::from_resolved_dimensions(
        width_mm,
        height_mm,
        PaperOrientationV1::Portrait,
        Some(issue),
    )
    .expect("closed A4 dimensions are finite in scene units")
}
