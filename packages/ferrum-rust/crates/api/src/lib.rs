//! Public integration surface and command-line application for Ferrum clients.

mod canonical_smiles;
mod cdml;
mod cdsvg;
mod clean_geometry_v1;
mod cli;
mod complete_graph_molecule_insertion_v1;
mod depiction_profile_v1;
mod direct_haworth_document_composition_v1;
mod direct_haworth_reobservation_composition_v1;
mod direct_haworth_smiles_v1;
mod document_clipboard_cut_v1;
mod document_clipboard_paste_v1;
mod document_complete_render_plan_v1;
mod document_ingress_v1;
mod document_linear_form_v1;
mod document_molecule_composition_graph_v1;
mod document_molecule_graph_v1;
mod document_molecule_inchi_publication_v1;
mod document_molecule_inchi_v1;
mod document_molecule_information_v1;
mod document_molecule_inspection_v1;
mod document_molecule_molblock_publication_v1;
mod document_molecule_molblock_v1;
mod document_molecule_name_v1;
mod document_molecule_sdf_publication_v1;
mod document_molecule_sdf_v1;
mod document_molecule_smiles_publication_v1;
mod document_molecule_smiles_v1;
mod document_pdf_artifact_v1;
mod document_png_artifact_v1;
mod document_render_cli;
mod document_render_plan_v1;
mod document_selection_svg_v1;
mod document_svg_artifact_v1;
mod errors;
mod explicit_adapter;
mod haworth_observation_v1;
mod inchi_codec;
mod inchi_molecule_insertion_v1;
mod local_document_profile_v1;
mod molblock_export;
mod molblock_inspection;
mod molblock_molecule_insertion_v1;
mod molblock_source_v1;
mod molecule_coordinate_cli;
mod molecule_coordinate_generation_v1;
mod peptide_sequence_inspection_v1;
mod peptide_template_molecule_insertion_v1;
mod presentation_plus_render_v1;
mod presentation_text_render_v1;
mod presentation_vector_lowering_v1;
mod render_observation_cli;
mod render_observation_v1;
mod reports;
mod sdf_export;
mod sdf_inspection;
mod sdf_molecule_insertion_v1;
mod sdf_source_v1;
mod smarts_export;
mod smiles_inspection;
mod smiles_molecule_insertion_v1;
mod streams;
mod telex_resource;

#[cfg(test)]
mod cdsvg_tests;

#[cfg(test)]
mod clean_geometry_v1_tests;

#[cfg(test)]
mod complete_graph_molecule_insertion_v1_tests;

#[cfg(test)]
mod depiction_profile_v1_tests;

#[cfg(test)]
mod document_molecule_inchi_v1_tests;

#[cfg(test)]
mod document_molecule_information_v1_tests;

#[cfg(test)]
mod document_molecule_inspection_v1_tests;

#[cfg(test)]
mod document_molecule_molblock_v1_tests;
#[cfg(test)]
mod document_molecule_sdf_v1_tests;

#[cfg(test)]
mod document_molecule_name_v1_tests;

#[cfg(test)]
mod document_linear_form_v1_tests;

#[cfg(test)]
mod document_native_artifact_v1_tests;

#[cfg(test)]
mod document_molecule_smiles_v1_tests;

#[cfg(test)]
mod document_render_plan_v1_tests;

#[cfg(test)]
mod document_selection_svg_v1_tests;

#[cfg(test)]
mod document_svg_artifact_v1_tests;

#[cfg(test)]
mod direct_haworth_document_composition_v1_tests;

#[cfg(test)]
mod direct_haworth_smiles_v1_tests;

#[cfg(test)]
mod haworth_observation_v1_tests;

#[cfg(test)]
mod render_observation_v1_tests;

#[cfg(test)]
mod render_observation_cli_tests;

#[cfg(test)]
mod smiles_molecule_insertion_v1_tests;

#[cfg(test)]
mod molecule_coordinate_generation_v1_tests;

#[cfg(test)]
mod molblock_molecule_insertion_v1_tests;

#[cfg(test)]
mod peptide_sequence_inspection_v1_tests;

#[cfg(test)]
mod peptide_template_molecule_insertion_v1_tests;

pub use canonical_smiles::{CanonicalSmilesError, canonical_smiles_from_smiles};
pub use cdml::{inspect_cdml, rewrite_cdml, validate_cdml, verify_cdml_rewrite};
pub use cdsvg::extract_cdsvg;
pub use clean_geometry_v1::{CleanGeometryBuildError, build_clean_geometry_update_v1};
pub use cli::Cli;
pub use cli::run;
pub use complete_graph_molecule_insertion_v1::{
    CompleteGraphMoleculeInsertionError, build_complete_graph_molecule_insertion_v1,
};
pub(crate) use depiction_profile_v1::render_document_projection_v1;
pub use depiction_profile_v1::{
    DEPICTION_PROFILE_SCHEMA_V1, DEPICTION_RESOLUTION_SCHEMA_V1, DepictionError,
    DepictionIssueCodeV1, DepictionIssueV1, DepictionProfileV1, DepictionSuppressionV1,
    HydrogenVisibilityV1,
};
pub use direct_haworth_document_composition_v1::{
    DirectHaworthDocumentCompositionErrorV1, compose_committed_direct_haworth_document_v1,
};
pub use direct_haworth_reobservation_composition_v1::compose_reobserved_direct_haworth_document_v1;
pub use direct_haworth_smiles_v1::{
    DirectHaworthFromSmilesBuildErrorV1, PreparedDirectHaworthFromSmilesV1,
    build_direct_haworth_from_smiles_v1,
};
pub use document_clipboard_cut_v1::{
    DocumentClipboardCutApplyErrorV1, apply_clipboard_cut_v1, prepare_clipboard_cut_v1,
};
pub use document_clipboard_paste_v1::{
    DOCUMENT_CLIPBOARD_PASTE_PROFILE_V1, DOCUMENT_CLIPBOARD_PASTE_TRANSLATION_V1,
    DocumentClipboardPasteApplyErrorV1, apply_clipboard_paste_v1,
    document_clipboard_paste_budget_v1, prepare_clipboard_paste_v1,
};
pub use document_complete_render_plan_v1::{
    CompleteDocumentRenderPlanErrorV1, compose_complete_document_render_plan_v1,
};
pub use document_ingress_v1::{
    AdmittedDocumentFileV1, CdmlIngressBudgetV1, CdmlIngressErrorV1, CdsvgIngressBudgetV1,
    DocumentIngressErrorV1, DocumentIngressFormatV1, DocumentIngressOriginV1, SourcePolicyErrorV1,
    load_document_file_for_publication_with_budget, load_document_file_with_budget,
    load_document_reader_with_budget, load_document_utf8_bytes_with_budget,
};
pub use document_linear_form_v1::{
    DocumentLinearFormErrorV1, DocumentLinearFormRequestV1, DocumentLinearFormResultV1,
    convert_document_linear_form_v1,
};
pub use document_molecule_composition_graph_v1::DocumentMoleculeCompositionGraphErrorV1;
pub use document_molecule_graph_v1::DocumentMoleculeGraphError;
pub use document_molecule_inchi_publication_v1::{
    DocumentMoleculeInchiPublicationErrorV1, publish_document_molecule_inchi_v1,
};
pub use document_molecule_inchi_v1::{
    DocumentMoleculeInchiError, DocumentMoleculeInchiV1, PreparedDocumentMoleculeInchiV1,
    export_document_molecule_inchi_v1, export_prepared_document_molecule_inchi_receipt_v1,
    export_prepared_document_molecule_inchi_v1, prepare_document_molecule_inchi_v1,
};
pub use document_molecule_information_v1::{
    DOCUMENT_MOLECULE_INFORMATION_SCHEMA_V1, DocumentMoleculeInformationErrorV1,
    DocumentMoleculeInformationRecordV1, DocumentMoleculeInformationRequestErrorV1,
    DocumentMoleculeInformationRequestV1, DocumentMoleculeInformationV1,
    PreparedDocumentMoleculeInformationV1, execute_prepared_document_molecule_information_v1,
    prepare_document_molecule_information_v1,
};
pub use document_molecule_inspection_v1::{
    DOCUMENT_MOLECULE_INSPECTION_SCHEMA_V1, DocumentMoleculeBoundsV1,
    DocumentMoleculeElementCountV1, DocumentMoleculeInspectionErrorV1,
    DocumentMoleculeInspectionRequestV1, DocumentMoleculeInspectionV1,
    inspect_document_molecule_v1,
};
pub use document_molecule_molblock_publication_v1::{
    DocumentMoleculeMolblockPublicationErrorV1, publish_document_molecule_molblock_v1,
};
pub use document_molecule_molblock_v1::{
    DOCUMENT_MOLECULE_MOLBLOCK_PROFILE_V1, DOCUMENT_MOLECULE_MOLBLOCK_SCHEMA_V1,
    DocumentMoleculeMolblockErrorV1, DocumentMoleculeMolblockRequestV1, DocumentMoleculeMolblockV1,
    PreparedDocumentMoleculeMolblockV1, export_prepared_document_molecule_molblock_v1,
    prepare_document_molecule_molblock_v1,
};
pub use document_molecule_name_v1::{
    DocumentMoleculeNameErrorV1, DocumentMoleculeNameRequestV1, set_document_molecule_name_v1,
};
pub use document_molecule_sdf_publication_v1::{
    DocumentMoleculeSdfPublicationErrorV1, publish_document_molecule_sdf_v1,
};
pub use document_molecule_sdf_v1::{
    DOCUMENT_MOLECULE_SDF_PROFILE_V1, DOCUMENT_MOLECULE_SDF_SCHEMA_V1, DocumentMoleculeSdfErrorV1,
    DocumentMoleculeSdfRequestV1, DocumentMoleculeSdfV1, PreparedDocumentMoleculeSdfV1,
    export_prepared_document_molecule_sdf_v1, prepare_document_molecule_sdf_v1,
};
pub use document_molecule_smiles_publication_v1::{
    DocumentMoleculeSmilesPublicationErrorV1, publish_document_molecule_smiles_v1,
};
pub use document_molecule_smiles_v1::{
    DOCUMENT_MOLECULE_SMILES_PROFILE_V1, DOCUMENT_MOLECULE_SMILES_SCHEMA_V1,
    DocumentMoleculeSmilesErrorV1, DocumentMoleculeSmilesRequestV1, DocumentMoleculeSmilesV1,
    PreparedDocumentMoleculeSmilesV1, export_prepared_document_molecule_smiles_v1,
    prepare_document_molecule_smiles_v1,
};
pub use document_pdf_artifact_v1::{DocumentPdfArtifactErrorV1, render_document_session_to_pdf_v1};
pub use document_png_artifact_v1::{DocumentPngArtifactErrorV1, render_document_session_to_png_v1};
pub use document_render_plan_v1::{
    DocumentRenderPlanCompositionError, compose_document_render_plan_v1,
};
pub use document_selection_svg_v1::{
    DOCUMENT_SELECTION_SVG_SCHEMA_V1, DocumentSelectionSvgErrorV1, DocumentSelectionSvgRootV1,
    DocumentSelectionSvgV1, DocumentSvgSelectionV1, render_document_selection_to_svg_v1,
};
pub use document_svg_artifact_v1::{DocumentSvgArtifactErrorV1, render_document_session_to_svg_v1};
pub use errors::{CdmlError, CdsvgError, CliError};
pub use explicit_adapter::ExplicitAdapterError;
pub use ferrum_core::{RecordId, RecordKind, RecordOrigin};
pub use ferrum_document::{
    DOCUMENT_CLIPBOARD_CUT_SCHEMA_V1, DocumentClipboardCutErrorV1, DocumentClipboardCutPlanV1,
};
pub use ferrum_document::{
    DOCUMENT_CLIPBOARD_FRAGMENT_SCHEMA_V1, DocumentClipboardFragmentErrorV1,
    DocumentClipboardFragmentKindV1, DocumentClipboardFragmentV1, DocumentClipboardSelectionV1,
    extract_document_clipboard_fragment_v1,
};
pub use ferrum_document::{
    DOCUMENT_CLIPBOARD_PASTE_SCHEMA_V1, DocumentClipboardPasteErrorV1,
    DocumentClipboardPastePlanV1, DocumentClipboardPasteResultV1, DocumentClipboardPasteRootV1,
    DocumentClipboardPastedRootV1,
};
pub use ferrum_geometry::{MoleculePlacementV1, Point2 as MoleculePlacementPointV1};
pub use ferrum_render::{
    BatchSpace, DocumentRenderIdentityV1, FerrumFontEnvironmentV1, FerrumFontId, GlyphPlacement,
    LineOp, MaskOp, MoleculeRenderPlan, PresentationGlyphRun, PresentationTextOp,
    PresentationTextSourceRun, RenderBatch, RenderIssue, RenderIssueKind, RenderOp, RenderPoint,
    RenderTarget, SvgOutputBudgetV1, TextOp, TextRun, TextScript,
};
pub use haworth_observation_v1::{
    DocumentHaworthObservationErrorV1, DocumentHaworthObservationRequestV1,
    DocumentHaworthObservationV1, DocumentHaworthRequestErrorV1, DocumentHaworthRootV1,
    HaworthTemplateBoundsV1, observe_document_haworth_v1,
};
pub use inchi_codec::{
    INCHI_INSPECTION_SCHEMA_V1, InchiExportError, InchiInspectionError, InchiInspectionV1,
    inchi_from_smiles, inspect_inchi,
};
pub use inchi_molecule_insertion_v1::{InchiMoleculeBuildError, build_inchi_molecule_insertion_v1};
pub use local_document_profile_v1::{
    LOCAL_CDML_INGRESS_PROFILE_V1, LOCAL_PDF_COMPLETED_BYTES_V1, LOCAL_PDF_DRAW_PATH_COMMANDS_V1,
    LOCAL_PDF_PLAN_ITEMS_V1, LOCAL_PDF_RENDER_PROFILE_V1, LOCAL_PNG_ENCODED_BYTES_V1,
    LOCAL_PNG_RAW_RGBA_BYTES_V1, LOCAL_PNG_RENDER_PROFILE_V1, LOCAL_SVG_COMPLETED_BYTES_V1,
    load_local_cdml_file_v1, local_cdml_ingress_format_v1, local_pdf_render_request_v1,
    local_png_render_request_v1,
};
pub use molblock_export::{MolblockExportError, molblock_from_smiles};
pub use molblock_inspection::{
    MOLBLOCK_INSPECTION_SCHEMA_V1, MolblockInspectionError, MolblockInspectionV1, inspect_molblock,
};
pub use molblock_molecule_insertion_v1::{
    MolblockMoleculeBuildError, build_molblock_molecule_insertion_v1,
};
pub use molblock_source_v1::{MolblockSourceErrorV1, read_molblock_file_v1};
pub use molecule_coordinate_generation_v1::{
    MoleculeCoordinateBuildError, build_molecule_coordinate_update_v1,
};
pub use peptide_sequence_inspection_v1::{
    PEPTIDE_SEQUENCE_INSPECTION_SCHEMA_V1, PeptideResidueInspectionV1,
    PeptideSequenceInspectionErrorV1, PeptideSequenceInspectionV1, inspect_peptide_sequence_v1,
};
#[cfg(test)]
pub(crate) use peptide_template_molecule_insertion_v1::build_native_template_insertion_with_engine;
pub use peptide_template_molecule_insertion_v1::{
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_FIXED_OUTPUT_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_ADDITIONAL_RESIDUE_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_MAX_SUBMITTED_BYTES_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_PROFILE_V1,
    NATIVE_PEPTIDE_TEMPLATE_INSERTION_SUPPORTED_ALPHABET_V1, PeptideTemplateInsertionErrorV1,
    PeptideTemplateMoleculeBuildErrorV1, SupportedPeptideTemplateRequestV1,
    build_supported_peptide_template_molecule_insertion_v1,
    compile_supported_peptide_template_request_v1,
};
pub use presentation_plus_render_v1::{DocumentPlusRenderV1, PresentationTextBoundsV1};
pub use presentation_text_render_v1::DocumentTextRenderV1;
pub use render_observation_cli::RenderObservationCliError;
pub use render_observation_v1::{
    DocumentMoleculeRenderPlanV1, MoleculeRenderRootV1, RENDER_OBSERVATION_SCHEMA_V1,
    RenderDocumentProvenanceV1, RenderObservationError, RenderObservationV1,
    RenderObservationWireV1, observe_render_v1,
};
pub use reports::{CdmlInspection, CdmlValidation, RewriteCheck};
pub use sdf_export::{SdfExportError, sdf_from_smiles};
pub use sdf_inspection::{
    SDF_INSPECTION_SCHEMA_V1, SdfInspectionError, SdfInspectionV1, SdfPropertyInspectionV1,
    SdfRecordInspectionV1, inspect_sdf,
};
pub use sdf_molecule_insertion_v1::{SdfMoleculeBuildError, build_sdf_record_batch_insertion_v1};
pub use sdf_source_v1::{SdfSourceErrorV1, read_sdf_file_v1};
pub use smarts_export::{SmartsExportError, smarts_from_smiles};
pub use smiles_inspection::{
    SMILES_INSPECTION_SCHEMA_V1, SmilesAtomInspectionV1, SmilesBondInspectionV1,
    SmilesInspectionError, SmilesInspectionV1, SmilesPointInspectionV1, inspect_smiles,
};
pub use smiles_molecule_insertion_v1::{
    SmilesMoleculeBuildError, SmilesMoleculeInsertionError, build_smiles_molecule_insertion_v1,
    prepare_smiles_molecule_v1,
};
pub use telex_resource::{VerifiedTelexRegularV1, verified_telex_regular_v1};
