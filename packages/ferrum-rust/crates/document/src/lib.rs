//! CDML document storage, typed recognition, and session services.
//!
//! The crate retains one authoritative XML tree, offers a typed CDML view, and exposes
//! revision-bound transactions, observations, and safe publication. The Rust
//! session is the sole mutable state authority; frontend clients receive immutable
//! values derived from accepted revisions. Its public API is intentionally
//! independent from the private module tree.

mod arrow_properties_patch_v1;
pub mod artifact_publication_v1;
mod atom_mark_projection;
mod atom_mark_v1;
mod atom_projection_v1;
mod atom_properties_patch_v1;
mod atom_rotation_v1;
#[allow(dead_code)] // WP-A1 private topology is consumed by the later session capability.
mod attached_cyclohexane_v1;
mod authored_text_v1;
mod authoring_capability_v1;
mod bond_presentation_v1;
mod bond_properties_patch_v1;
mod bracket_insertion_v1;
mod bracket_pair_projection_v1;
mod bracket_properties_patch_v1;
mod cdml_facade_v1;
mod cdml_namespace_v1;
mod cdsvg;
mod cdsvg_facade_v1;
mod chemistry;
mod clean_geometry_update_v1;
mod clipboard_cut_v1;
mod clipboard_fragment_v1;
mod clipboard_paste_v1;
mod core_projection;
mod curved_equilibrium_arrow_geometry_v1;
mod direct_bond_mutation;
mod direct_bond_primitives_v1;
mod direct_cdml_semantic_index_v1;
mod direct_haworth_insertion_v1;
mod direct_haworth_reobservation_v1;
mod document_explicit_fragment_api_v1;
mod document_ingress_v1;
mod document_smarts_snapshot_v1;
mod drawing_standard_patch_v1;
mod equilibrium_arrow_geometry_v1;
mod explicit_fragment_v1;
mod generated_ids;
mod geometric_properties_patch_v1;
mod geometry_repair_v1;
mod identity_index;
mod interchange;
mod linear_form_convert_v1;
mod local_document_profile_v1;
mod molecule_coordinate_batch_update_v1;
mod molecule_coordinate_update_v1;
mod molecule_insertion_v1;
mod operations;
mod paper_properties_v1;
mod paper_size_v1;
mod plus_properties_patch_v1;
mod presentation_arrow_projection_v1;
mod presentation_creation_gesture_v1;
mod presentation_path_gesture_v1;
mod presentation_plus_projection_v1;
mod presentation_polyline_projection_v1;
mod presentation_root_deletion_v1;
mod presentation_shape_projection_v1;
mod presentation_stack_projection_v1;
mod presentation_stack_reorder_v1;
mod presentation_text_projection_v1;
mod presentation_v1;
mod projection_identity_v1;
mod projection_v1;
mod publication;
mod reaction_reference_graph_v1;
mod regular_ring_insertion_v1;
#[cfg(test)]
mod regular_ring_insertion_v1_tests;

mod interchange_record_insertion_v1;
mod interchange_record_metadata_v1;
mod render_document_model_v1;
mod reports_v1;
#[doc(hidden)]
pub mod session;
mod session_history;
mod session_observation;
mod session_operation;
mod session_state;
mod standalone_haworth_insertion_v1;
mod straighten_depiction_update_v1;
mod text_placement_gesture_v1;
mod text_properties_patch_v1;
mod top_level_transform_v1;
mod top_level_translation_anchor_v1;
mod typed;
mod typed_arrow_insertion;
mod typed_arrow_properties;
mod typed_atom_mark;
mod typed_atom_number;
mod typed_atom_position;
mod typed_atom_properties;
mod typed_atom_rotation;
mod typed_attached_cyclohexane;
mod typed_bond_insertion;
mod typed_bond_order;
mod typed_bond_properties;
mod typed_bracket_insertion;
mod typed_bracket_properties;
mod typed_class;
mod typed_coordinate;
mod typed_diagnostic;
mod typed_document_error;
mod typed_drawing_standard;
mod typed_geometric_properties;
mod typed_geometry_repair;
mod typed_linear_form_metadata;
mod typed_molecule_insertion;
mod typed_molecule_name;
mod typed_molecule_positions;
mod typed_object_resolution;
mod typed_paper_properties;
mod typed_plus_insertion;
mod typed_plus_properties;
mod typed_presentation_root_deletion;
mod typed_presentation_stack_reorder;
mod typed_record_deletion;
mod typed_schema;
mod typed_text_insertion_v1;
mod typed_text_properties;
mod typed_top_level_transform;
mod typed_wavy_insertion;
mod typed_wavy_properties;
mod user_template_v1;
mod wavy_insertion_v1;
mod wavy_properties_patch_v1;
mod xml_input_budget_v1;

pub use arrow_properties_patch_v1::{
    ArrowLineWidthV1, ArrowPropertiesPatchV1, ArrowPropertiesPatchV1Error, ArrowPropertyChangeV1,
};
pub use atom_mark_v1::{AtomMarkActionV1, AtomMarkKindV1, AtomMarkProjectionV1};
pub use atom_projection_v1::AtomProjectionV1;
pub use atom_properties_patch_v1::{
    AtomPropertiesPatchV1, AtomPropertiesPatchV1Error, AtomPropertyChangeV1,
};
pub use atom_rotation_v1::{AtomRotationTargetV1, AtomRotationV1, AtomRotationV1Error};
pub use attached_cyclohexane_v1::{AttachedCyclohexaneErrorV1, AttachedCyclohexaneReleaseV1};
pub use authored_text_v1::{
    AuthoredTextRunV1, AuthoredTextStyleV1, normalize_authored_text_runs_v1,
};
pub use authoring_capability_v1::{
    AuthoringCapabilityAccessErrorV1, AuthoringCapabilityClaimV1, AuthoringCapabilityIssuerV1,
    AuthoringCapabilityV1,
};
pub use bond_presentation_v1::DocumentBondPresentationV1;
pub use bond_properties_patch_v1::{
    BondPropertiesPatchV1, BondPropertiesPatchV1Error, BondPropertyChangeV1, DocumentBondStyleV1,
    NonZeroFiniteV1,
};
pub use bracket_insertion_v1::{BracketInsertionV1, BracketInsertionV1Error, BracketStyleV1};
pub use bracket_pair_projection_v1::BracketPairProjectionV1;
pub use bracket_properties_patch_v1::{
    BracketPropertiesPatchV1, BracketPropertiesPatchV1Error, BracketPropertyChangeV1,
};
pub use cdml_facade_v1::{
    CdmlError, inspect_cdml, rewrite_cdml, validate_cdml, verify_cdml_rewrite,
};
pub use cdsvg::{
    CdsvgExtractionError, CdsvgInputMeasurementV1, extract_cdml_from_svg,
    extract_cdml_from_svg_with_budget, measure_cdsvg_input_v1,
};
pub use cdsvg_facade_v1::{CdsvgError, extract_cdsvg};
pub use chemistry::*;
pub use clean_geometry_update_v1::{
    CleanGeometryMoleculeV1, CleanGeometryUpdateV1, CleanGeometryUpdateV1Error,
};
pub use clipboard_cut_v1::{
    DOCUMENT_CLIPBOARD_CUT_SCHEMA_V1, DocumentClipboardCutErrorV1, DocumentClipboardCutPlanV1,
    prepare_document_clipboard_cut_v1,
};
pub use clipboard_fragment_v1::{
    DOCUMENT_CLIPBOARD_FRAGMENT_SCHEMA_V1, DocumentClipboardFragmentErrorV1,
    DocumentClipboardFragmentKindV1, DocumentClipboardFragmentV1, DocumentClipboardSelectionV1,
    extract_document_clipboard_fragment_v1,
};
pub use clipboard_paste_v1::{
    DOCUMENT_CLIPBOARD_PASTE_SCHEMA_V1, DocumentClipboardPasteErrorV1,
    DocumentClipboardPastePlanV1, DocumentClipboardPasteRootV1, DocumentClipboardPastedRootV1,
    prepare_document_clipboard_paste_v1,
};
pub use core_projection::{CoreProjection, CoreProjectionError};
pub use curved_equilibrium_arrow_geometry_v1::{
    CurvedEquilibriumArrowEndHeadLaneV1, CurvedEquilibriumArrowGeometryErrorV1,
    CurvedEquilibriumArrowGeometryV1, CurvedEquilibriumArrowStartHeadLaneV1,
    curved_equilibrium_arrow_geometry_v1,
};
pub use direct_bond_mutation::{DirectBondEndpointIntent, DirectBondMutationCandidate};
pub use direct_bond_primitives_v1::{
    DirectBondAdmissionRefusalV1, DirectBondCommitErrorV1, DirectBondGestureErrorV1,
    DirectBondPoint2V1, DirectBondSnapPolicyV1, DocumentFenceV1,
};
pub use direct_cdml_semantic_index_v1::{
    DirectCdmlRootKindV1, DirectCdmlRootV1, DirectCdmlSemanticErrorV1, DirectCdmlSemanticIndexV1,
    DirectReactionMemberV1, DirectReactionRoleV1, ReactionDefinitionDiagnosticV1,
    ReactionDefinitionV1, append_direct_cdml_reaction_v1,
    delete_direct_cdml_reaction_definition_v1, inspect_direct_reactions_v1,
    replace_direct_cdml_reaction_members_v1,
};
pub use direct_haworth_insertion_v1::{
    CommittedDirectHaworthBondFactV1, DocumentDirectHaworthBondRoleV1,
    DocumentDirectHaworthBondTokenV1,
};
pub use direct_haworth_reobservation_v1::{
    DirectHaworthReobservationErrorV1, ReobservedDirectHaworthBondFactV1, ReobservedDirectHaworthV1,
};
pub use document_explicit_fragment_api_v1::{
    DOCUMENT_EXPLICIT_FRAGMENT_SCHEMA_V1, DocumentExplicitFragmentApiErrorV1,
    DocumentExplicitFragmentCreateResultV1, DocumentExplicitFragmentObservationReceiptV1,
    DocumentExplicitFragmentRequestV1, create_document_explicit_fragment_v1,
    inspect_document_explicit_fragments_v1,
};
pub use document_ingress_v1::{
    AdmittedDocumentFileV1, CdmlIngressBudgetV1, CdmlIngressErrorV1, CdsvgIngressBudgetV1,
    DocumentIngressErrorV1, DocumentIngressFormatV1, DocumentIngressOriginV1, SourcePolicyErrorV1,
    load_document_file_for_publication_with_budget, load_document_file_with_budget,
    load_document_reader_with_budget, load_document_utf8_bytes_with_budget,
    read_regular_file_with_origin_with_budget,
};
pub use document_smarts_snapshot_v1::{
    DocumentSmartsSnapshotErrorV1, DocumentSmartsTargetV1, PreparedDocumentSmartsSnapshotV1,
};
pub use drawing_standard_patch_v1::{
    DrawingStandardPatchV1, DrawingStandardPatchV1Error, DrawingStandardPropertyChangeV1,
    MAX_DRAWING_STANDARD_FONT_FAMILY_BYTES_V1, MAX_DRAWING_STANDARD_FONT_SIZE_V1,
    MAX_DRAWING_STANDARD_WIDTH_V1, MIN_DRAWING_STANDARD_FONT_SIZE_V1,
};
pub use explicit_fragment_v1::{
    DocumentExplicitFragmentErrorV1, DocumentExplicitFragmentObservationV1,
    DocumentExplicitFragmentRecordV1, observe_explicit_fragments_v1,
};
pub use geometric_properties_patch_v1::{
    GeometricLineWidthV1, GeometricPropertiesPatchV1, GeometricPropertiesPatchV1Error,
    GeometricPropertyChangeV1,
};
pub use geometry_repair_v1::{GeometryRepairKindV1, GeometryRepairV1, GeometryRepairV1Error};
pub use identity_index::{
    CompleteDocumentIdentityFactsV1, DocumentIdentityError, DocumentRecord, ElementPath,
    IndexedDocument, IndexedDocumentError, PersistentId, ResolvedId, SourceOrder, XmlDocument,
    XmlSerializationError,
};
pub use interchange::{
    INTERCHANGE_MAX_TEXT_BYTES_V1, InterchangeCodecErrorV1, InterchangeFormatV1,
    InterchangePropertyV1, InterchangeRecordV1, decode_interchange_v1, encode_interchange_v1,
};
pub use interchange_record_insertion_v1::{
    INTERCHANGE_IMPORT_NAMESPACE_V1, InterchangePropertyInsertionV1,
    InterchangeRecordBatchInsertionV1, InterchangeRecordInsertionV1,
    InterchangeRecordInsertionV1Error,
};
pub use interchange_record_metadata_v1::{
    InterchangePropertyMetadataV1, InterchangeRecordMetadataErrorV1, InterchangeRecordMetadataV1,
    observe_interchange_record_metadata_v1,
};
pub use local_document_profile_v1::{
    LOCAL_CDML_INGRESS_PROFILE_V1, LOCAL_CDML_SOURCE_UTF8_BYTES_V1,
    LOCAL_DECODED_CDSVG_INGRESS_PROFILE_V1, load_local_cdml_file_v1, local_cdml_ingress_format_v1,
    local_decoded_cdsvg_ingress_format_v1, prepare_local_cdml_file_v1,
    prepare_local_cdml_file_with_origin_v1, prepare_local_decoded_cdsvg_file_v1,
    prepare_local_decoded_cdsvg_file_with_origin_v1,
};
pub use molecule_coordinate_batch_update_v1::{
    MoleculeCoordinateBatchUpdateV1, MoleculeCoordinateBatchUpdateV1Error,
};
pub use molecule_coordinate_update_v1::{
    MoleculeCoordinateUpdateV1, MoleculeCoordinateUpdateV1Error,
};
pub use molecule_insertion_v1::{
    DocumentBondOrderV1, MoleculeInsertionAtomV1, MoleculeInsertionBondOrderV1,
    MoleculeInsertionBondV1, MoleculeInsertionV1, MoleculeInsertionV1Error,
};
pub use operations::{
    DOCUMENT_CLIPBOARD_PASTE_PROFILE_V1, DOCUMENT_CLIPBOARD_PASTE_TRANSLATION_V1,
    DOCUMENT_USER_TEMPLATE_PROFILE_V1, DocumentClipboardCutApplyErrorV1,
    DocumentClipboardPasteApplyErrorV1, DocumentLinearFormErrorV1, DocumentLinearFormRequestV1,
    DocumentLinearFormResultV1, DocumentMoleculeNameErrorV1, DocumentMoleculeNameRequestV1,
    DocumentUserTemplateApplyErrorV1, apply_clipboard_cut_v1, apply_clipboard_paste_v1,
    apply_user_template_v1, convert_document_linear_form_v1, document_clipboard_paste_budget_v1,
    document_operation_budget_v1, document_user_template_budget_v1,
    observe_top_level_translation_anchor_v1, prepare_clipboard_cut_v1, prepare_clipboard_paste_v1,
    prepare_user_template_v1, set_document_molecule_name_v1,
};
pub use paper_properties_v1::{
    PAPER_LAYOUT_PROJECTION_SCHEMA_V1, PaperAttributesV1, PaperLayoutProjectionV1,
    PaperOrientationV1, PaperPageIssueV1, PaperPageV1, PaperPropertiesPatchV1,
    PaperPropertiesPatchV1Error, PaperPropertyChangeV1, ViewportAttributesV1,
};
pub use paper_size_v1::{
    PaperDimensionsMmV1, PaperDimensionsMmV1Error, PaperSizeV1, paper_size_catalog_v1,
    paper_size_v1,
};
pub use plus_properties_patch_v1::{
    MAX_PLUS_FONT_SIZE_V1, MIN_PLUS_FONT_SIZE_V1, PlusPropertiesPatchV1,
    PlusPropertiesPatchV1Error, PlusPropertyChangeV1,
};
pub use presentation_arrow_projection_v1::{
    ArrowDisplayGeometryV1, ArrowHeadPositionV1, ArrowHeadShapeV1, ArrowHeadV1, ArrowPathV1,
    ArrowProjectionV1, CurvedTerminalArrowDisplayKindV1, CurvedTerminalArrowGeometryV1,
    CurvedTerminalArrowKindV1, curved_terminal_arrow_geometry_v1,
};
pub use presentation_creation_gesture_v1::{
    ArrowGestureStyleV1, CommittedPresentationGestureV1, PresentationCreationGestureV1,
    PresentationCreationPreviewV1, PresentationGestureCategoryV1, PresentationGestureErrorV1,
    PresentationGestureKindV1, PresentationGestureOverlayGeometryV1, PresentationGestureOverlayV1,
    PresentationGesturePoint2V1, PresentationGestureRecoveryV1, PresentationGestureSnapPolicyV1,
    PresentationGestureStyleV1,
};
pub use presentation_path_gesture_v1::{
    PRESENTATION_PATH_MAXIMUM_EXTENT_PT_V1, PRESENTATION_PATH_MAXIMUM_POINTS_V1,
    PresentationPathGestureCategoryV1, PresentationPathGestureErrorV1,
    PresentationPathGestureRecoveryV1, PresentationPathGestureV1, PresentationPathKindV1,
};
pub use presentation_plus_projection_v1::{PlusProjectionV1, PresentationFontV1};
pub use presentation_root_deletion_v1::{
    PresentationRootDeletionSetV1, PresentationRootDeletionSetV1Error, PresentationRootDeletionV1,
    PresentationRootDeletionV1Error, PresentationRootSelectorV1, PresentationRootSelectorV1Error,
};
pub use presentation_shape_projection_v1::{
    BoxShapeProjectionV1, PolygonPathV1, PolygonProjectionV1, PresentationBoundsV1,
    PresentationFillV1,
};
pub use presentation_stack_projection_v1::{
    PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PolylinePathV1, PolylineProjectionV1,
    PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationRecordKindV1, PresentationRootProjectionV1, PresentationStackProjectionV1,
    PresentationStrokeV1, PresentationTargetV1,
};
pub use presentation_stack_reorder_v1::{
    PresentationStackOrderV1, PresentationStackReorderV1, PresentationStackReorderV1Error,
};
pub use presentation_text_projection_v1::{
    PresentationTextFontV1, PresentationTextRunV1, PresentationTextStyleV1, TextProjectionV1,
};
pub use presentation_v1::{
    DrawingStandardV1, FontFactsV1, PositiveFiniteV1, PresentationLengthUnitV1,
    PresentationLengthV1, Rgb24V1, RichTextV1, TransparentOrRgb24V1, VisibilityV1,
};
pub use projection_identity_v1::{
    DocumentObjectIdV1, DocumentObjectIdV1Error, ProjectionLocalObjectKeyV1,
};
pub use projection_v1::{
    BondEndpointKindV1, BondEndpointV1, BondProjectionV1, DOCUMENT_PROJECTION_SCHEMA_V1,
    DocumentHaworthPositionV1, DocumentProjectionV1, MoleculeProjectionV1, Point3V1,
    ProjectionError, ProjectionIssueCodeV1, ProjectionIssueV1,
};
pub use publication::{
    DocumentMoleculeInchiPublicationErrorV1, DocumentMoleculeMolblockPublicationErrorV1,
    DocumentMoleculeSdfPublicationErrorV1, DocumentMoleculeSmilesPublicationErrorV1,
    DocumentMoleculesSdfPublicationErrorV2, PublicationDurability,
    publish_document_molecule_inchi_v1, publish_document_molecule_molblock_v1,
    publish_document_molecule_sdf_v1, publish_document_molecule_smiles_v1,
    publish_document_molecules_sdf_v2,
};
pub use regular_ring_insertion_v1::{
    DetachedRegularRingInsertionV1, RegularRingInsertionErrorV1, RegularRingOrientationV1,
    RegularRingSizeV1,
};
pub use render_document_model_v1::{
    RenderDocumentModelConversionErrorV1, render_document_model_from_observation_v1,
};
pub use reports_v1::{CdmlInspection, CdmlValidation, MoleculeInspection, RewriteCheck};
pub use session::{
    AttachedCyclohexaneSessionErrorV1, CommittedDirectHaworthResultV1, CommittedDirectHaworthV1,
    DocumentClipboardPasteResultV1, DocumentSession, DocumentSessionError, DocumentSnapshot,
    DocumentUserTemplateResultV1, PendingAttachedCyclohexaneV1, PendingCreateAtom,
    PendingCreateBond, PendingCreateBondedAtom, PendingCreateBracket,
    PendingCreateInterchangeBatchV1, PendingCreateMolecule, PendingCreateMoleculeBatchV1,
    PendingCreateWavy, PendingDeleteStructureV1, PendingDirectHaworthV1,
    PendingLinearFormConvertV1, PendingStandaloneHaworthV1, PreparedLinearFormConvertResultV1,
    Publication, SaveOutcome,
};
pub use session::{
    PendingCreatePresentationV1, PresentationAppearanceV1, PresentationCreateErrorV1,
    PresentationCreateRequestV1, PresentationVectorCreateKindV1,
};
pub use session_observation::SessionDocumentObservationV1;
pub use session_operation::{
    SessionOperation, SessionOperationError, SessionOperationResultV1, SessionOperationV1,
};
pub use straighten_depiction_update_v1::{
    PreparedStraightenDepictionsV1, StraightenDepictionUpdateV1Error,
    StraightenedDepictionMoleculeV1,
};
pub use text_placement_gesture_v1::{
    CommittedTextPlacementV1, TextPlacementContentV1, TextPlacementErrorCategoryV1,
    TextPlacementErrorV1, TextPlacementGestureV1, TextPlacementPreviewV1, TextPlacementRecoveryV1,
};
pub use text_properties_patch_v1::{
    MAX_TEXT_FONT_SIZE_V1, MIN_TEXT_FONT_SIZE_V1, TextEditRunV1, TextEditStyleV1,
    TextPropertiesPatchV1, TextPropertiesPatchV1Error, TextPropertyChangeV1,
};
pub use top_level_transform_v1::{
    TopLevelRootKindV1, TopLevelRootSelectorV1, TopLevelTransformModeV1, TopLevelTransformV1,
    TopLevelTransformV1Error,
};
pub use top_level_translation_anchor_v1::TopLevelTranslationAnchorV1;
pub use typed::{
    ExpandedName, NamespaceBinding, TypedChild, TypedClass, TypedDocument, TypedRecord, TypedText,
    UnknownAttribute, UnrecognizedChild, UnrecognizedNode,
};
pub use typed_diagnostic::{TypedDiagnostic, TypedDiagnosticKind};
pub use typed_document_error::TypedDocumentError;
pub use typed_record_deletion::{StructureDeletionComponentV1, StructureDeletionReceiptV1};
pub use user_template_v1::{
    DOCUMENT_USER_TEMPLATE_SCHEMA_V1, DocumentUserTemplateErrorV1,
    DocumentUserTemplateInsertedMoleculeV1, DocumentUserTemplatePlanV1,
    prepare_document_user_template_v1,
};
pub use wavy_insertion_v1::{
    WAVY_MAX_AMPLITUDE_V1, WAVY_MAX_SEGMENTS_V1, WAVY_SEGMENT_LENGTH_V1, WavyInsertionV1,
    WavyInsertionV1Error,
};
pub use wavy_properties_patch_v1::{
    WavyPropertiesPatchV1, WavyPropertiesPatchV1Error, WavyPropertyChangeV1,
};
pub use xml_input_budget_v1::{
    XmlBudgetError, XmlInputBudgetV1, XmlInputError, XmlInputMeasurementV1, measure_xml_input_v1,
};

pub(crate) use cdml_namespace_v1::{
    CDML_NAMESPACE, ferrum_cdml_element_name, is_ferrum_cdml_name, is_ferrum_cdml_root,
};
pub(crate) use identity_index::element_name;

#[cfg(test)]
mod compatibility_tests;

#[cfg(test)]
mod cdsvg_tests;

#[cfg(test)]
mod clipboard_fragment_v1_tests;

#[cfg(test)]
mod clipboard_cut_v1_tests;

#[cfg(test)]
mod clipboard_paste_v1_tests;

#[cfg(test)]
mod user_template_v1_tests;

#[cfg(test)]
mod identity_index_tests;

#[cfg(test)]
mod session_tests;

#[cfg(test)]
mod artifact_publication_v1_tests;
#[cfg(test)]
mod publication_tests;

#[cfg(test)]
mod projection_v1_tests;

#[cfg(test)]
mod presentation_stack_projection_v1_tests;

#[cfg(test)]
mod session_semantics_tests;

#[cfg(test)]
mod structural_deletion_tests;

#[cfg(test)]
mod interchange_record_insertion_v1_tests;
#[cfg(test)]
mod molecule_coordinate_batch_update_v1_tests;
#[cfg(test)]
mod molecule_insertion_v1_tests;

#[cfg(test)]
mod interchange_record_metadata_v1_tests;

#[cfg(test)]
mod typed_tests;

#[cfg(test)]
mod budgeted_document_construction_tests;

#[cfg(test)]
mod paper_size_v1_tests;
