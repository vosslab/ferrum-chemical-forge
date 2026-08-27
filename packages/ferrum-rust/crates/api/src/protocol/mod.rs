//! Closed, stateless JSON operation protocol V1.
//!
//! Public DTOs and schema generation stay separate from execution so the
//! portable wire contract remains easy to audit independently of adapter use.

mod document_atom_oxidation_v1;
mod document_compact_group_attachment_v1;
mod document_compact_group_materialization_v1;
mod document_hydrogen_materialization_v1;
mod document_request_parse_v1;
pub(crate) mod document_smarts_snapshot_v1;
mod dto;
pub use dto::InspectGraphNormalizationV1;
mod execution;
mod frozen_document_snapshot_v1;
#[cfg(feature = "python-binding")]
pub(crate) mod live_document_operation_v1;
mod molecule_diagnostics_core_v1;
mod molecule_report_core_v1;
mod molecule_report_diagnostics_v1;
pub(crate) mod runtime;
mod schema;
pub(crate) mod smarts_query_core_v1;

pub use dto::{
    CatalogCategorySummaryV1, CatalogEntrySummaryV1, CatalogInsertRequestV1, CatalogListRequestV1,
    CatalogPlacementRefusalV1, CatalogProvenanceSummaryV1, ChemistryConvertInputV1,
    ChemistryConvertRequestV1, CompactGroupAttachmentRefusalV1,
    CompactGroupMaterializationRefusalV1, DocumentAtomOxidationObservationOutcomeV1,
    DocumentAtomOxidationObservationV1, DocumentAtomOxidationObserveRequestV1,
    DocumentAtomOxidationUnavailableReasonV1, DocumentCompactGroupAttachmentRequestV1,
    DocumentCompactGroupAttachmentResultV1, DocumentCompactGroupMaterializationRequestV1,
    DocumentCompactGroupMaterializationResultV1, DocumentGenerateCoordinatesRequestV1,
    DocumentInspectRequestV1, DocumentInterchangeImportLossReportV1,
    DocumentInterchangeImportSummaryV1, DocumentInterchangeLossCategoryV1,
    DocumentInterchangeProvenanceV1, DocumentInterchangeSourceKindV1,
    DocumentMoleculeDiagnosticRecordSummaryV1, DocumentMoleculeDiagnosticsRequestV1,
    DocumentMoleculeDiagnosticsSnapshotV1, DocumentMoleculeDiagnosticsSummaryV1,
    DocumentMoleculeHydrogenMaterializationOutcomeV1,
    DocumentMoleculeHydrogenMaterializationRequestV1,
    DocumentMoleculeHydrogenMaterializationResultV1,
    DocumentMoleculeHydrogenMaterializationUnavailableReasonV1,
    DocumentMoleculeInterchangeImportRequestV1,
    DocumentMoleculeReportAggregateOmissionReasonSummaryV1,
    DocumentMoleculeReportAggregateOutcomeSummaryV1,
    DocumentMoleculeReportCompositionElementSummaryV1, DocumentMoleculeReportCompositionSummaryV1,
    DocumentMoleculeReportDirectedBondDepictionSummaryV1,
    DocumentMoleculeReportDirectedBondPresentationSummaryV1,
    DocumentMoleculeReportDoubleBondCarrierMarkKindSummaryV1,
    DocumentMoleculeReportDoubleBondCarrierMarkSummaryV1,
    DocumentMoleculeReportDoubleBondConfigurationSummaryV1,
    DocumentMoleculeReportDoubleBondStereoSummaryV1, DocumentMoleculeReportElementCountSummaryV1,
    DocumentMoleculeReportFindingCodeSummaryV1, DocumentMoleculeReportFindingLocationSummaryV1,
    DocumentMoleculeReportFindingRecoverySummaryV1, DocumentMoleculeReportFindingSeveritySummaryV1,
    DocumentMoleculeReportFindingSubjectSummaryV1, DocumentMoleculeReportFindingSummaryV1,
    DocumentMoleculeReportRecordSummaryV1, DocumentMoleculeReportRequestV1,
    DocumentMoleculeReportSnapshotV1, DocumentMoleculeReportStereoDepictionSummaryV1,
    DocumentMoleculeReportStereoLigandSummaryV1, DocumentMoleculeReportStereoSemanticsSummaryV1,
    DocumentMoleculeReportSummaryV1, DocumentMoleculeReportTetrahedralParitySummaryV1,
    DocumentMoleculeReportTetrahedralStereoSummaryV1, DocumentRenderArtifactRequestV1,
    DocumentRequestFenceV1, DocumentRewriteRequestV1, DocumentSmartsQueryInputV1,
    DocumentSmartsQueryLimitsV1, DocumentSmartsQueryMoleculeSummaryV1,
    DocumentSmartsQueryRequestV1, DocumentSmartsQuerySummaryV1,
    DocumentSmartsQueryTraversalSummaryV1, DocumentSnapshotRequestV1, DocumentValidateRequestV1,
    InspectGraphFactCoverageStatusV1, InspectGraphFactCoverageV1, InspectInterchangeGraphInputV1,
    InspectInterchangeGraphRecordSummaryV1, InspectInterchangeGraphRequestV1,
    InspectInterchangeGraphSummaryV1, MAX_REQUEST_ID_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_ERROR_SCHEMA_V1, OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
    OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1, OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1,
    OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1, OperationProtocolEnvelopeV1,
    OperationProtocolErrorCategoryV1, OperationProtocolErrorResponseV1, OperationProtocolErrorV1,
    OperationProtocolInputErrorV1, OperationProtocolOperationV1, OperationProtocolOutcomeV1,
    OperationProtocolRequestV1, OperationProtocolResponseV1, PresentationAuthorDirectBondOutcomeV1,
    PresentationAuthorPointV1, PresentationAuthorRefusalV1, PresentationAuthorRequestV1,
    PresentationAuthoringKindV1, PresentationAuthoringRequestV1, ProtocolArtifactFormatV1,
    ProtocolCatalogFamilyV1, ProtocolCatalogPlacementCategoryV1,
    ProtocolCatalogPlacementRecoveryV1, ProtocolCompactGroupAttachmentCategoryV1,
    ProtocolCompactGroupAttachmentRecoveryV1, ProtocolCompactGroupMaterializationCategoryV1,
    ProtocolCompactGroupMaterializationRecoveryV1, ProtocolCurvedTerminalArrowKindV1,
    ProtocolDirectBondEndpointV1, ProtocolDirectBondOrderV1, ProtocolDirectBondPresentationV1,
    ProtocolDirectBondSnapV1, ProtocolErrorSchemaV1, ProtocolOperationKindV1,
    ProtocolPresentationAuthorCategoryV1, ProtocolPresentationAuthorRecoveryV1,
    ProtocolPresentationPathKindV1, ProtocolPresentationVectorAppearancePolicyV1,
    ProtocolPresentationVectorKindV1, ProtocolReactionDefinitionDispositionV1,
    ProtocolRequestSchemaV1, ProtocolResourceLimitReasonV1, ProtocolResponseSchemaV1,
    ProtocolValidationLevelV1, ReactionMemberSummaryV1, ReactionObservationRequestV1,
    ReactionObservationSummaryV1, ReactionObserveRequestV1, SourceFactV1,
};
#[cfg(feature = "python-binding")]
pub(crate) use execution::execute_admitted_operation_v1;
pub use execution::execute_operation_v1;
#[cfg(test)]
pub(crate) use execution::execute_operation_with_runtime_and_smarts_response_limit_for_test;
pub(crate) use execution::{
    MINIMUM_RESPONSE_SIZE_EXCEEDED_ENVELOPE_BYTES_V1, canonical_protocol_envelope_json_v1,
    execute_operation_with_runtime_v1, interchange_import_refusal_envelope_v1,
    interchange_import_success_envelope_v1, response_size_exceeded_envelope_v1,
};
pub use schema::{generated_operation_protocol_schema_v1, operation_protocol_schema_v1};
