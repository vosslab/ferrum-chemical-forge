//! Delivery boundary for Ferrum's command-line interface and frozen protocol.
//!
//! Owner crates expose chemistry, document, rendering, and geometry APIs
//! directly. This crate owns only transport, CLI presentation, and the
//! stateless operation protocol shared by the CLI and Python extension.

mod cli;
mod protocol;
mod transport;

pub use cli::{Cli, run};
pub use protocol::runtime::TrustedLibraryChemistryRuntimeV1;
pub use protocol::{
    ChemistryConvertInputV1, ChemistryConvertRequestV1, DocumentGenerateCoordinatesRequestV1,
    DocumentInspectRequestV1, DocumentRenderArtifactRequestV1, DocumentRewriteRequestV1,
    DocumentValidateRequestV1, MAX_REQUEST_ID_UTF8_BYTES_V1, OPERATION_PROTOCOL_ERROR_SCHEMA_V1,
    OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1, OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1, OperationProtocolErrorResponseV1,
    OperationProtocolInputErrorV1, OperationProtocolOperationV1, OperationProtocolOutcomeV1,
    OperationProtocolRequestV1, OperationProtocolResponseV1, ProtocolArtifactFormatV1,
    ProtocolErrorSchemaV1, ProtocolOperationKindV1, ProtocolRequestSchemaV1,
    ProtocolResponseSchemaV1, ProtocolValidationLevelV1, execute_operation_v1,
    execute_operation_with_runtime_v1, generated_operation_protocol_schema_v1,
    operation_protocol_schema_v1,
};
