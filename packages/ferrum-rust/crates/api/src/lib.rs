//! Delivery boundary for Ferrum's command-line interface and frozen protocol.
//!
//! Owner crates expose chemistry, document, rendering, and geometry APIs
//! directly. This crate owns only transport, CLI presentation, and the
//! stateless operation protocol shared by the CLI and Python extension.

mod catalog_placement_v2;
mod cli;
mod document_interchange_import_v1;
mod interchange_import_v1;
mod plus_placement_gesture_v1;
mod presentation_vector_gesture_v1;
mod presentation_path_gesture_v1;
mod protocol;
#[cfg(feature = "python-binding")]
mod python_extension_binding_v1;
mod reaction_aggregate_v1;
// Render interaction capabilities are owned by ferrum-document-render.

pub use ferrum_document_render::{
    ReactionDefinitionDispositionV1, ReactionListObservationV1, ReactionMemberObservationV1,
    ReactionObservationV1, ReactionSelectionV1,
};
mod text_placement_gesture_v1;
mod transport;

pub use catalog_placement_v2::{
    ApiCatalogPlacementGestureV2, ApiCatalogPlacementPreparedV2, ApiCatalogPlacementPreviewV2,
    CatalogPlacementCategoryV2, CatalogPlacementErrorV2, CatalogPlacementRecoveryV2,
    CommittedCatalogPlacementV2, begin_api_catalog_placement_v2,
    cancel_api_catalog_placement_gesture_v2, commit_api_catalog_placement_v2,
    prepare_api_catalog_placement_v2, preview_api_catalog_placement_v2,
    release_api_catalog_placement_preview_v2,
};
pub use cli::{Cli, run};
pub use ferrum_document_render::{
    CommittedRenderInteractionTranslationV1, CommittedStructureDeletionV1,
    ReactionAuthoringChoiceAvailabilityV1, ReactionAuthoringChoiceKindV1,
    ReactionAuthoringChoiceV1, ReactionAuthoringChoicesV1, ReactionAuthoringExclusionReasonV1,
    ReactionAuthoringExclusionRecoveryV1, ReactionAuthoringExclusionV1, RenderInteractionAxisV1,
    RenderInteractionBoundsV1, RenderInteractionErrorV1, RenderInteractionExclusionReasonV1,
    RenderInteractionExclusionV1, RenderInteractionGridSnapPolicyV1, RenderInteractionModifierV1,
    RenderInteractionObservationV1, RenderInteractionQueryV1, RenderInteractionRootV1,
    RenderInteractionSelectionV1, RenderInteractionSessionV1, RenderInteractionSnapV1,
    RenderInteractionTranslationGestureV1, RenderInteractionTranslationPreviewV1,
    StructureInteractionObservationV1, StructureInteractionQueryV1,
    StructureInteractionSelectionV1, StructureInteractionTargetV1, StructureTargetKindV1,
};
pub use interchange_import_v1::{
    CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1, CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
    InterchangeCompressionPolicyV1, InterchangeDecoderKeyV1, InterchangeDirectionV1,
    InterchangeFormatDescriptorV1, InterchangeFormatRegistryV1, InterchangeImportLimitsV1,
    InterchangeImportRecoveryV1, InterchangeImportRefusalCategoryV1,
    InterchangeImportRefusalReasonV1, InterchangeImportRefusalV1, InterchangeSemanticLossPolicyV1,
    SDF_IMPORT_FORMAT_V1, SDF_IMPORT_PROFILE_V1,
};
pub use plus_placement_gesture_v1::{
    ApiPlusGestureV1, ApiPlusOverlayV1, ApiPlusPreviewV1, begin_api_plus_gesture_v1,
    commit_api_plus_gesture_v1, preview_api_plus_gesture_v1,
};
pub use presentation_vector_gesture_v1::{
    ApiPresentationVectorGestureV1, ApiPresentationVectorPreparedV1,
    ApiPresentationVectorPreviewV1, CommittedPresentationVectorV1,
    PresentationVectorGestureCategoryV1, PresentationVectorGestureErrorV1,
    PresentationVectorGestureRecoveryV1, PresentationVectorKindV1, PresentationVectorOverlayV1,
    begin_api_presentation_vector_gesture_v1, commit_api_presentation_vector_gesture_v1,
    prepare_api_presentation_vector_gesture_v1, preview_api_presentation_vector_gesture_v1,
};
pub use presentation_path_gesture_v1::{
    ApiPresentationPathGestureV1, ApiPresentationPathPreparedV1, ApiPresentationPathPreviewV1,
    CommittedPresentationPathV1, PresentationPathRenderCategoryV1, PresentationPathRenderErrorV1,
    PresentationPathRenderRecoveryV1, begin_api_presentation_path_gesture_v1,
    commit_api_presentation_path_gesture_v1, prepare_api_presentation_path_gesture_v1,
    preview_api_presentation_path_gesture_v1,
};
pub use protocol::{
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
    DocumentRenderArtifactRequestV1, DocumentRewriteRequestV1, DocumentSmartsQueryDocumentV1,
    DocumentSmartsQueryInputV1, DocumentSmartsQueryLimitsV1, DocumentSmartsQueryMoleculeSummaryV1,
    DocumentSmartsQueryRequestV1, DocumentSmartsQuerySummaryV1,
    DocumentSmartsQueryTraversalSummaryV1, DocumentValidateRequestV1, MAX_REQUEST_ID_UTF8_BYTES_V1,
    OPERATION_PROTOCOL_ERROR_SCHEMA_V1, OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
    OPERATION_PROTOCOL_REQUEST_UTF8_BYTES_V1, OPERATION_PROTOCOL_RESPONSE_SCHEMA_V1,
    OperationProtocolEnvelopeV1, OperationProtocolErrorCategoryV1,
    OperationProtocolErrorResponseV1, OperationProtocolInputErrorV1, OperationProtocolOperationV1,
    OperationProtocolOutcomeV1, OperationProtocolRequestV1, OperationProtocolResponseV1,
    PresentationVectorRefusalV1, ProtocolArtifactFormatV1, ProtocolCatalogFamilyV1,
    ProtocolCatalogPlacementCategoryV1, ProtocolCatalogPlacementRecoveryV1, ProtocolErrorSchemaV1,
    ProtocolOperationKindV1, ProtocolPresentationVectorGestureCategoryV1,
    ProtocolPresentationVectorGestureRecoveryV1, ProtocolReactionDefinitionDispositionV1,
    ProtocolReactionTranslationSnapV1, ProtocolRequestSchemaV1, ProtocolResourceLimitReasonV1,
    ProtocolResponseSchemaV1, ProtocolValidationLevelV1, ReactionBoundsSummaryV1,
    ReactionMemberSummaryV1, ReactionObservationRequestV1, ReactionObservationSummaryV1,
    ReactionObserveRequestV1, ReactionTranslateRequestV1, execute_operation_v1,
    generated_operation_protocol_schema_v1, operation_protocol_schema_v1,
};
#[cfg(feature = "python-binding")]
pub use python_extension_binding_v1::initialize_python_extension_v1;
pub use reaction_aggregate_v1::{
    ApiPreparedReactionLifecycleV1, ApiPreparedReactionTranslationV1, ApiPreparedReactionV1,
    ApiReactionGestureV1, ApiReactionLifecycleGestureV1, ApiReactionTranslationGestureV1,
    ApiReactionTranslationPreviewV1, CommittedReactionLifecycleV1, CommittedReactionV1,
    ReactionCreateRequestV1, ReactionGestureCategoryV1, ReactionGestureErrorV1,
    ReactionGestureRecoveryV1, ReactionMembershipPatchRequestV1,
    begin_api_reaction_definition_delete_v1, begin_api_reaction_gesture_v1,
    begin_api_reaction_membership_patch_v1, begin_api_reaction_translation_v1,
    commit_api_reaction_gesture_v1, commit_api_reaction_lifecycle_v1,
    commit_api_reaction_translation_v1, prepare_api_reaction_gesture_v1,
    prepare_api_reaction_lifecycle_v1, prepare_api_reaction_translation_v1,
    preview_api_reaction_translation_v1,
};
pub use text_placement_gesture_v1::{
    ApiTextPlacementDefaultsV1, ApiTextPlacementGestureV1, ApiTextPlacementPreviewV1,
    begin_api_text_placement_gesture_v1, commit_api_text_placement_gesture_v1,
    preview_api_text_placement_gesture_v1, text_placement_defaults_v1,
};
