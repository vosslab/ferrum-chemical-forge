//! Closed protocol error DTOs for the stateless JSON operation protocol V1.

use schemars::JsonSchema;

use serde::Serialize;

#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolErrorResponseV1 {
    /// Exact V1 error schema identifier.
    pub schema: ProtocolErrorSchemaV1,
    /// Present only after the request envelope admitted this opaque value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Stable discriminator and diagnostic detail.
    pub error: OperationProtocolErrorV1,
}

/// Closed V1 error schema identifiers.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolErrorSchemaV1 {
    /// `ferrum-operation-error-v1`.
    #[serde(rename = "ferrum-operation-error-v1")]
    V1,
}

/// Stable protocol error facts; clients must not discriminate on `message`.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationProtocolErrorV1 {
    /// Stable machine-readable failure category.
    pub category: OperationProtocolErrorCategoryV1,
    /// Decoded operation kind when one was available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<ProtocolOperationKindV1>,
    /// Human-readable diagnostic detail.
    pub message: String,
    /// Closed reason for a resource-limit refusal when one is safe to expose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_limit_reason: Option<ProtocolResourceLimitReasonV1>,
    /// Closed vector gesture recovery facts when this operation refused one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_vector_refusal: Option<PresentationVectorRefusalV1>,
    /// Closed catalog-placement recovery facts when this operation refused one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_placement_refusal: Option<CatalogPlacementRefusalV1>,
    /// Closed reaction authoring recovery facts when this operation refused one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reaction_refusal: Option<ReactionRefusalV1>,
}

/// Closed public reasons for protocol-wide resource-limit refusals.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolResourceLimitReasonV1 {
    /// The exact canonical SMARTS result envelope exceeded its public budget.
    ResponseSizeExceeded,
}

/// Typed refusal facts for `presentation.vector.create.v1`.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationVectorRefusalV1 {
    pub category: ProtocolPresentationVectorGestureCategoryV1,
    pub recovery: ProtocolPresentationVectorGestureRecoveryV1,
}

/// Typed refusal facts for `catalog.insert.v1`.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogPlacementRefusalV1 {
    pub category: ProtocolCatalogPlacementCategoryV1,
    pub recovery: ProtocolCatalogPlacementRecoveryV1,
}

/// Typed refusal facts for `reaction.create.v1`.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReactionRefusalV1 {
    pub category: ProtocolReactionRefusalCategoryV1,
    pub recovery: ProtocolReactionRefusalRecoveryV1,
}

#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolReactionRefusalCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    ReplayedGesture,
    InvalidRequest,
    MissingTarget,
    WrongTargetKind,
    DuplicateTarget,
    CrossReactionReuse,
    UnrenderableDocument,
    RenderPreparation,
    SessionConflict,
    MissingReaction,
    LegacyDefinitionNotEditable,
    MembershipChanged,
    RendererExclusion,
}

#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolReactionRefusalRecoveryV1 {
    RefreshAndRestart,
    CorrectSelectors,
    ChooseRenderableMembers,
    RepairLegacyDefinition,
}

#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCatalogPlacementCategoryV1 {
    UnknownKey,
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    RenderPreparation,
    SessionConflict,
}

#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCatalogPlacementRecoveryV1 {
    ChooseCatalogEntry,
    RefreshAndRestart,
    DocumentUnchanged,
}

/// Closed presentation-vector refusal categories.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolPresentationVectorGestureCategoryV1 {
    StaleSnapshot,
    ForeignSession,
    MismatchedPreview,
    ReplayedGesture,
    InvalidPoint,
    DegenerateGeometry,
    UnsupportedKind,
    UnrenderableStandard,
    RenderPreparation,
    SessionConflict,
    ResourceExhausted,
}

/// Closed recovery instructions for presentation-vector refusals.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolPresentationVectorGestureRecoveryV1 {
    DocumentUnchanged,
    RefreshAndRestart,
    ChangeGeometry,
    ChooseSupportedAppearance,
    ReduceRequest,
}

/// Stable V1 error categories.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
pub enum OperationProtocolErrorCategoryV1 {
    /// The JSON envelope or closed operation payload was invalid.
    #[serde(rename = "invalid_request")]
    InvalidRequest,
    /// The envelope named a schema identifier not supported by V1.
    #[serde(rename = "unsupported_protocol_version")]
    UnsupportedProtocolVersion,
    /// Bounded uncompressed CDML admission failed.
    #[serde(rename = "document_admission_failed")]
    DocumentAdmissionFailed,
    /// A semantically required document operation could not proceed.
    #[serde(rename = "document_invalid")]
    DocumentInvalid,
    /// The document cannot produce a complete artifact for the requested profile.
    #[serde(rename = "render_unsupported")]
    RenderUnsupported,
    /// A native artifact backend could not finish rendering.
    #[serde(rename = "render_failed")]
    RenderFailed,
    /// The caller supplied no usable out-of-band chemistry capability.
    #[serde(rename = "chemistry_unavailable")]
    ChemistryUnavailable,
    /// Molecular interchange was malformed, bounded out, or could not complete.
    #[serde(rename = "conversion_failed")]
    ConversionFailed,
    /// The requested interchange target cannot exactly represent the input records.
    #[serde(rename = "conversion_unsupported")]
    ConversionUnsupported,
    /// Coordinate generation could not complete for the whole document snapshot.
    #[serde(rename = "coordinate_generation_failed")]
    CoordinateGenerationFailed,
    /// Existing resource policy or derived base64 completion limits refused work.
    #[serde(rename = "resource_limit")]
    ResourceLimit,
    /// An unexpected owned-value executor failure occurred.
    #[serde(rename = "internal_failure")]
    InternalFailure,
}

/// Stable names for decoded operation kinds in error envelopes.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
pub enum ProtocolOperationKindV1 {
    /// `document.inspect`.
    #[serde(rename = "document.inspect")]
    Inspect,
    /// `document.validate`.
    #[serde(rename = "document.validate")]
    Validate,
    /// `document.rewrite`.
    #[serde(rename = "document.rewrite")]
    Rewrite,
    /// `document.render_artifact`.
    #[serde(rename = "document.render_artifact")]
    RenderArtifact,
    /// `chemistry.convert`.
    #[serde(rename = "chemistry.convert")]
    ChemistryConvert,
    /// `document.generate_coordinates`.
    #[serde(rename = "document.generate_coordinates")]
    GenerateCoordinates,
    /// `presentation.vector.create.v1`.
    #[serde(rename = "presentation.vector.create.v1")]
    PresentationVectorCreate,
    /// `catalog.list.v1`.
    #[serde(rename = "catalog.list.v1")]
    CatalogList,
    /// `catalog.insert.v1`.
    #[serde(rename = "catalog.insert.v1")]
    CatalogInsert,
    #[serde(rename = "reaction.create.v1")]
    ReactionCreate,
    #[serde(rename = "reaction.list.v1")]
    ReactionList,
    #[serde(rename = "reaction.observe.v1")]
    ReactionObserve,
    #[serde(rename = "reaction.select.v1")]
    ReactionSelect,
    #[serde(rename = "reaction.patch-membership.v1")]
    ReactionPatchMembership,
    #[serde(rename = "reaction.delete-definition.v1")]
    ReactionDeleteDefinition,
    #[serde(rename = "reaction.translate.v1")]
    ReactionTranslate,
    /// `document.molecule.report.v1`.
    #[serde(rename = "document.molecule.report.v1")]
    DocumentMoleculeReport,
    /// `document.molecule.smarts.query.v1`.
    #[serde(rename = "document.molecule.smarts.query.v1")]
    DocumentSmartsQuery,
    /// `document.molecule.interchange.import.v1`.
    #[serde(rename = "document.molecule.interchange.import.v1")]
    DocumentMoleculeInterchangeImport,
}
