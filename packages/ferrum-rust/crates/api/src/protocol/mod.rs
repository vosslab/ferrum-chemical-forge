//! Closed, stateless JSON operation protocol V1.
//!
//! Public DTOs and schema generation stay separate from execution so the
//! portable wire contract remains easy to audit independently of adapter use.

mod dto;
mod execution;
pub(crate) mod runtime;
mod schema;

pub use dto::{
    ChemistryConvertInputV1, ChemistryConvertRequestV1, DocumentGenerateCoordinatesRequestV1,
    DocumentInspectRequestV1, DocumentRenderArtifactRequestV1, DocumentRewriteRequestV1,
    DocumentValidateRequestV1, MAX_REQUEST_ID_UTF8_BYTES_V1, OPERATION_PROTOCOL_ERROR_SCHEMA_V1,
    OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1, OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1, OperationProtocolErrorResponseV1,
    OperationProtocolInputErrorV1, OperationProtocolOperationV1, OperationProtocolOutcomeV1,
    OperationProtocolRequestV1, OperationProtocolResponseV1, ProtocolArtifactFormatV1,
    ProtocolErrorSchemaV1, ProtocolOperationKindV1, ProtocolRequestSchemaV1,
    ProtocolResponseSchemaV1, ProtocolValidationLevelV1,
};
pub use execution::{execute_operation_v1, execute_operation_with_runtime_v1};
pub use schema::{generated_operation_protocol_schema_v1, operation_protocol_schema_v1};
