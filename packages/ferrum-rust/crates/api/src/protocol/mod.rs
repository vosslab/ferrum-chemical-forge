//! Closed, stateless JSON operation protocol V1.
//!
//! Public DTOs and schema generation stay separate from execution so the
//! portable wire contract remains easy to audit independently of adapter use.

pub(crate) mod document_smarts_snapshot_v1;
mod dto;
mod execution;
mod molecule_report_core_v1;
pub(crate) mod runtime;
mod schema;
pub(crate) mod smarts_query_core_v1;

pub use dto::{
    CatalogCategorySummaryV1, CatalogEntrySummaryV1, CatalogInsertRequestV1, CatalogListRequestV1,
    CatalogPlacementRefusalV1, CatalogProvenanceSummaryV1, ChemistryConvertInputV1,
    ChemistryConvertRequestV1, DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1,
    DocumentGenerateCoordinatesRequestV1, DocumentInspectRequestV1,
    DocumentInterchangeImportLossReportV1, DocumentInterchangeImportSummaryV1,
    DocumentInterchangeLossCategoryV1, DocumentInterchangeProvenanceV1,
    DocumentInterchangeSourceKindV1, DocumentMoleculeInterchangeImportRequestV1,
    DocumentMoleculeReportAggregateOmissionReasonSummaryV1,
    DocumentMoleculeReportAggregateOutcomeSummaryV1,
    DocumentMoleculeReportCompositionElementSummaryV1, DocumentMoleculeReportCompositionSummaryV1,
    DocumentMoleculeReportElementCountSummaryV1, DocumentMoleculeReportRecordSummaryV1,
    DocumentMoleculeReportRequestV1, DocumentMoleculeReportSummaryV1,
    DocumentRenderArtifactRequestV1, DocumentRequestFenceV1, DocumentRewriteRequestV1,
    DocumentSmartsQueryDocumentV1, DocumentSmartsQueryInputV1, DocumentSmartsQueryLimitsV1,
    DocumentSmartsQueryMoleculeSummaryV1, DocumentSmartsQueryRequestV1,
    DocumentSmartsQuerySummaryV1, DocumentSmartsQueryTraversalSummaryV1, DocumentValidateRequestV1,
    MAX_REQUEST_ID_UTF8_BYTES_V1, OPERATION_PROTOCOL_ERROR_SCHEMA_V1,
    OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1, OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1, OperationProtocolErrorResponseV1,
    OperationProtocolInputErrorV1, OperationProtocolOperationV1, OperationProtocolOutcomeV1,
    OperationProtocolRequestV1, OperationProtocolResponseV1, PresentationAuthorDirectBondOutcomeV1,
    PresentationAuthorPointV1, PresentationAuthorRefusalV1, PresentationAuthorRequestV1,
    PresentationAuthoringKindV1, PresentationAuthoringRequestV1, ProtocolArtifactFormatV1,
    ProtocolCatalogFamilyV1, ProtocolCatalogPlacementCategoryV1,
    ProtocolCatalogPlacementRecoveryV1, ProtocolCurvedTerminalArrowKindV1,
    ProtocolDirectBondEndpointV1, ProtocolDirectBondOrderV1, ProtocolDirectBondPresentationV1,
    ProtocolDirectBondSnapV1, ProtocolErrorSchemaV1, ProtocolOperationKindV1,
    ProtocolPresentationAuthorCategoryV1, ProtocolPresentationAuthorRecoveryV1,
    ProtocolPresentationPathKindV1, ProtocolPresentationVectorAppearancePolicyV1,
    ProtocolPresentationVectorKindV1, ProtocolReactionDefinitionDispositionV1,
    ProtocolReactionTranslationSnapV1, ProtocolRequestSchemaV1, ProtocolResourceLimitReasonV1,
    ProtocolResponseSchemaV1, ProtocolValidationLevelV1, ReactionBoundsSummaryV1,
    ReactionMemberSummaryV1, ReactionObservationRequestV1, ReactionObservationSummaryV1,
    ReactionObserveRequestV1, ReactionTranslateRequestV1,
};
pub use execution::execute_operation_v1;
#[cfg(test)]
pub(crate) use execution::execute_operation_with_runtime_and_smarts_response_limit_for_test;
pub(crate) use execution::{
    canonical_protocol_envelope_json_v1, execute_operation_with_runtime_v1,
};
pub use schema::{generated_operation_protocol_schema_v1, operation_protocol_schema_v1};
