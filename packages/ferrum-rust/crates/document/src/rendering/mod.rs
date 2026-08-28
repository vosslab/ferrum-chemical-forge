//! Session-owned render observation, complete-plan, and artifact boundaries.

mod artifact_v1;
mod complete_plan_v1;
mod native_artifact_v1;
mod observation;
mod selection_svg_v1;

pub use artifact_v1::{
    DocumentPdfArtifactErrorV1, DocumentPngArtifactErrorV1, DocumentSvgArtifactErrorV1,
    render_document_session_to_pdf_v1, render_document_session_to_png_v1,
    render_document_session_to_svg_v1,
};
pub use complete_plan_v1::{
    CompleteDocumentRenderPlanErrorV1, compose_complete_document_render_plan_v1,
};
pub use native_artifact_v1::{
    DocumentNativeArtifactErrorV1, DocumentNativeArtifactProfileV1,
    PreparedDocumentNativeArtifactV1, prepare_document_native_artifact_v1,
    publish_prepared_document_native_artifact_v1,
};
pub use observation::{
    DOCUMENT_RENDER_OBSERVATION_SCHEMA_V2, DocumentRenderObservationErrorV1,
    DocumentRenderObservationV2, DocumentRenderObservationWireV2,
    derive_document_render_observation_from_accepted_operation_v2, observe_document_render_v2,
};
pub use selection_svg_v1::{
    DOCUMENT_SELECTION_SVG_SCHEMA_V1, DocumentSelectionSvgErrorV1, DocumentSelectionSvgRootV1,
    DocumentSelectionSvgV1, DocumentSvgSelectionV1, render_document_selection_to_svg_v1,
};
