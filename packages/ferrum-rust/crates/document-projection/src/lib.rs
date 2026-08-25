//! Immutable typed vocabulary shared by document projection and rendering.
//!
//! This crate carries values only. Retained CDML traversal, session state,
//! history, fences, generated IDs, and renderer planning remain in their
//! owning crates.

mod document;
mod geometry;
mod identity;
mod issue;
mod molecule;
mod paper;
mod presentation;
mod style;

pub use document::{
    DOCUMENT_PROJECTION_SCHEMA_V1, DocumentProjectionProvenanceV1, DocumentProjectionV1,
    DocumentProjectionV1Error,
};
pub use geometry::{NonZeroFiniteV1, Point3V1, ProjectionError};
pub use identity::{
    DocumentObjectIdV1, DocumentObjectIdV1Error, ProjectionLocalObjectKeyV1,
    ProjectionLocalObjectKeyV1Error,
};
pub use issue::{ProjectionIssueCodeV1, ProjectionIssueV1, ProjectionIssueV1Error};
pub use molecule::{
    AtomMarkKindV1, AtomMarkProjectionV1, AtomProjectionV1, BondEndpointKindV1, BondEndpointV1,
    BondProjectionV1, CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, CompactGroupProjectionV1,
    CompactGroupV1, CompactGroupV1Error, DocumentHaworthPositionV1,
    DoubleBondCarrierMarkProjectionV1, DoubleBondCarrierMarkProjectionV1Error,
    DoubleBondCarrierMarkV1, MoleculeProjectionV1, MoleculeProjectionV1Error,
};
pub use paper::{
    PAPER_LAYOUT_PROJECTION_SCHEMA_V1, PaperAttributesV1, PaperLayoutProjectionV1,
    PaperOrientationV1, PaperPageIssueV1, PaperPageV1, ViewportAttributesV1,
};
pub use presentation::{
    ArrowHeadShapeV1, ArrowPathV1, ArrowProjectionKindV1, ArrowProjectionV1,
    ArrowProjectionV1Error, BoxShapeProjectionV1, BracketPairProjectionV1,
    CurvedTerminalArrowKindV1, PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PlusProjectionV1,
    PolygonPathV1, PolygonProjectionV1, PolylinePathV1, PolylineProjectionV1,
    PresentationArrowPreviewRequestV1, PresentationBoundsV1, PresentationBracketStyleV1,
    PresentationFactProvenanceV1, PresentationFillV1, PresentationFontFaceV1, PresentationFontV1,
    PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1, PresentationRecordKindV1,
    PresentationRootProjectionV1, PresentationStackProjectionV1,
    PresentationStackProjectionV1Error, PresentationStrokeV1, PresentationTargetV1,
    PresentationTextFontV1, PresentationTextRunV1, PresentationTextStyleV1, TextProjectionV1,
};
pub use style::{
    DrawingStandardV1, FontFactsV1, PositiveFiniteV1, PresentationLengthUnitV1,
    PresentationLengthV1, Rgb24V1, RichTextV1, TransparentOrRgb24V1, VisibilityV1,
};
