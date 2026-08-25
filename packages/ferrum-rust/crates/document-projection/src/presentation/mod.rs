//! Immutable presentation projection value family.

mod arrow;
mod bracket;
mod font_face;
mod plus;
mod shape;
mod stack;
mod text;

use serde::{Deserialize, Serialize};

pub use arrow::{
    ArrowHeadShapeV1, ArrowPathV1, ArrowProjectionKindV1, ArrowProjectionV1,
    ArrowProjectionV1Error, CurvedTerminalArrowKindV1, PresentationArrowPreviewRequestV1,
};
pub use bracket::BracketPairProjectionV1;
pub use font_face::PresentationFontFaceV1;
pub use plus::{PlusProjectionV1, PresentationFontV1};
pub use shape::{
    BoxShapeProjectionV1, PolygonPathV1, PolygonProjectionV1, PresentationBoundsV1,
    PresentationFillV1,
};
pub use stack::{
    PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PolylinePathV1, PolylineProjectionV1,
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationRecordKindV1, PresentationRootProjectionV1, PresentationStackEntryV1,
    PresentationStackProjectionV1, PresentationStackProjectionV1Error, PresentationStrokeV1,
    PresentationTargetV1,
};
pub use text::{
    PresentationTextFontV1, PresentationTextRunV1, PresentationTextStyleV1, TextProjectionV1,
};

/// Persistent bracket geometry family carried by a bracket-pair projection.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationBracketStyleV1 {
    /// Four connected straight segments on each side.
    Rectangular,
    /// Four control points on each spline side.
    Round,
}
