//! Delivery boundary for Ferrum's command-line interface and frozen protocol.
//!
//! Owner crates expose chemistry, document, rendering, and geometry APIs
//! directly. This crate owns only transport, CLI presentation, and the
//! stateless operation protocol shared by the CLI and Python extension.

mod cli;
mod document_interchange_import_v1;
mod interchange_capability_catalog_v1;
mod interchange_capability_v1;
mod interchange_import_v1;
mod interchange_output_v1;
mod local_document_open_catalog_v2;
mod presentation_path_gesture_v1;
mod presentation_vector_gesture_v1;
mod protocol;
#[cfg(feature = "python-binding")]
mod python_extension_binding_v1;
// Render interaction capabilities are owned by ferrum-document-render.

pub use ferrum_document::{
    DocumentCreateReactionCommandV1, DocumentDeleteReactionCommandV1,
    DocumentReactionAuthoringCommandKindV1, DocumentReactionListDispositionV1,
    DocumentReactionListObservationV1, DocumentReactionListReactionV1,
    DocumentReactionMemberObservationV1, DocumentReactionMemberSelectionV1,
    DocumentReactionMemberTargetsV1, DocumentReactionSelectionObservationV1,
    DocumentReplaceReactionMembersCommandV1, ReactionAuthoringCommandRefusalV1,
    ReactionMemberSelectionRefusalV1,
};
mod transport;

pub use cli::{Cli, run};
pub use ferrum_document_render::{
    CommittedRenderInteractionTranslationV1, CommittedStructureDeletionV1,
    ReactionAuthoringChoiceAvailabilityV1, ReactionAuthoringChoiceKindV1,
    ReactionAuthoringChoiceV1, ReactionAuthoringExclusionReasonV1,
    ReactionAuthoringExclusionRecoveryV1, ReactionAuthoringExclusionV1,
    ReactionAuthoringObservationV1, RenderInteractionAxisV1, RenderInteractionBoundsV1,
    RenderInteractionErrorV1, RenderInteractionExclusionReasonV1, RenderInteractionExclusionV1,
    RenderInteractionGridSnapPolicyV1, RenderInteractionModifierV1, RenderInteractionObservationV1,
    RenderInteractionQueryV1, RenderInteractionRootV1, RenderInteractionSelectionV1,
    RenderInteractionSessionV1, RenderInteractionSnapV1, RenderInteractionTranslationGestureV1,
    RenderInteractionTranslationPreviewV1, StructureInteractionObservationV1,
    StructureInteractionQueryV1, StructureInteractionSelectionV1, StructureInteractionTargetV1,
    StructureTargetKindV1,
};
pub use interchange_capability_catalog_v1::{
    INTERCHANGE_CAPABILITY_CATALOG_SCHEMA_V1, InterchangeCapabilityCatalogErrorV1,
    InterchangeCapabilityCatalogV1, InterchangeCapabilityInputV1, InterchangeCapabilityOutputV1,
    InterchangeCapabilityV1,
};
pub use interchange_capability_v1::{
    ConversionInputCapabilityV1, InterchangeCapabilityResolverV1,
    InterchangeGraphInspectionProfileV1, NativeConversionInputDescriptorV1,
};
pub use interchange_import_v1::{
    CDXML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1, CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1, CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    ConversionExecutionProfileV1, ConversionInputProfileV1, InterchangeCompressionPolicyV1,
    InterchangeDecoderKeyV1, InterchangeFormatDescriptorV1, InterchangeFormatRegistryV1,
    InterchangeImportLimitsV1, InterchangeImportRecoveryV1, InterchangeImportRefusalCategoryV1,
    InterchangeImportRefusalReasonV1, InterchangeImportRefusalV1, InterchangeOperationRefusalV1,
    InterchangeOperationV1, InterchangeRuntimeRequirementV1, InterchangeSemanticLossPolicyV1,
    SDF_IMPORT_FORMAT_V1, SDF_IMPORT_PROFILE_V1,
};
pub use interchange_output_v1::{
    CML_SIMPLE_MOLECULE_OUTPUT_FORMAT_V1, CML_SIMPLE_MOLECULE_OUTPUT_PROFILE_V1,
    ConversionOutputDescriptorV1, ConversionOutputRegistryRefusalV1, ConversionOutputRegistryV1,
    ConversionOutputTargetV1,
};
pub use local_document_open_catalog_v2::{
    LocalDocumentOpenCatalogErrorV2, LocalDocumentOpenCatalogV2, LocalDocumentOpenDescriptorV2,
    LocalDocumentOpenDispositionV2, LocalDocumentOpenRouteV2,
};
pub use presentation_path_gesture_v1::{
    ApiPresentationPathGestureV1, ApiPresentationPathOverlayV1, PresentationPathProgressV1,
    PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1,
    PresentationPathRenderRecoveryV1, add_api_presentation_path_gesture_point_v1,
    begin_api_presentation_path_gesture_v1, cancel_api_presentation_path_gesture_v1,
    preview_incremental_api_presentation_path_gesture_v1,
    resolve_incremental_api_presentation_path_gesture_v1,
};
pub use presentation_vector_gesture_v1::{
    ApiPresentationVectorGestureV1, ApiPresentationVectorPreviewV1,
    PresentationVectorGestureCategoryV1, PresentationVectorGestureErrorV1,
    PresentationVectorGestureRecoveryV1, PresentationVectorKindV1, PresentationVectorOverlayV1,
    begin_api_presentation_vector_gesture_v1, preview_api_presentation_vector_gesture_v1,
    resolve_api_presentation_vector_gesture_v1,
};
#[cfg(feature = "python-binding")]
pub(crate) use protocol::execute_admitted_operation_v1;
pub use protocol::{
    CatalogCategorySummaryV1, CatalogEntrySummaryV1, CatalogInsertRequestV1, CatalogListRequestV1,
    CatalogPlacementRefusalV1, CatalogProvenanceSummaryV1, ChemistryConvertInputV1,
    ChemistryConvertRequestV1, DocumentAtomOxidationObservationOutcomeV1,
    DocumentAtomOxidationObservationV1, DocumentAtomOxidationObserveRequestV1,
    DocumentAtomOxidationUnavailableReasonV1, DocumentGenerateCoordinatesRequestV1,
    DocumentInspectRequestV1, DocumentInterchangeImportLossReportV1,
    DocumentInterchangeImportSummaryV1, DocumentInterchangeLossCategoryV1,
    DocumentInterchangeProvenanceV1, DocumentInterchangeSourceKindV1,
    DocumentMoleculeDiagnosticRecordSummaryV1, DocumentMoleculeDiagnosticsRequestV1,
    DocumentMoleculeDiagnosticsSnapshotV1, DocumentMoleculeDiagnosticsSummaryV1,
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
    DocumentMoleculeReportIdentifierUnavailableReasonSummaryV1,
    DocumentMoleculeReportIdentifiersSummaryV1, DocumentMoleculeReportRecordSummaryV1,
    DocumentMoleculeReportRequestV1, DocumentMoleculeReportSnapshotV1,
    DocumentMoleculeReportStereoDepictionSummaryV1, DocumentMoleculeReportStereoLigandSummaryV1,
    DocumentMoleculeReportStereoSemanticsSummaryV1, DocumentMoleculeReportSummaryV1,
    DocumentMoleculeReportTetrahedralParitySummaryV1,
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
    ProtocolRequestSchemaV1, ProtocolResourceLimitReasonV1, ProtocolResponseSchemaV1,
    ProtocolValidationLevelV1, ReactionMemberSummaryV1, ReactionObservationRequestV1,
    ReactionObservationSummaryV1, ReactionObserveRequestV1, SourceFactV1, execute_operation_v1,
    generated_operation_protocol_schema_v1, operation_protocol_schema_v1,
};
#[cfg(feature = "python-binding")]
pub use python_extension_binding_v1::initialize_python_extension_v1;
