//! Document-session chemistry operations and document-to-chemistry mappings.
//!
//! Chemistry codecs and native adapters remain in `ferrum-chemistry`; this module
//! owns the CDML/session side of their boundary.

mod clean_geometry_v1;
mod complete_graph_molecule_insertion_v1;
mod document_atom_oxidation_observation_v1;
mod document_bond_capacity_v1;
mod document_chemistry_limits_v1;
mod document_molecule_composition_graph_v1;
mod document_molecule_graph_v1;
mod document_molecule_inchi_v1;
mod document_molecule_information_v1;
mod document_molecule_inspection_v1;
mod document_molecule_molblock_v1;
mod document_molecule_sdf_v1;
mod document_molecule_smiles_v1;
mod document_molecules_sdf_v2;
mod inchi_molecule_insertion_v1;
mod interchange_record_insertion_v1;
mod molblock_molecule_insertion_v1;
mod molblock_source_v1;
mod molecule_coordinate_generation_v1;
mod ordinary_attachment_capacity_v1;
mod peptide_template_molecule_insertion_v1;
mod sdf_source_v1;
mod smiles_molecule_insertion_v1;

#[cfg(test)]
mod document_bond_capacity_v1_tests;
#[cfg(test)]
mod document_molecule_information_v1_tests;
#[cfg(test)]
mod document_molecule_inspection_v1_tests;

pub use clean_geometry_v1::{CleanGeometryBuildError, build_clean_geometry_update_v1};
pub use complete_graph_molecule_insertion_v1::{
    CompleteGraphMoleculeInsertionError,
    build_complete_graph_molecule_insertion_from_validated_facts_v1,
    build_complete_graph_molecule_insertion_v1, validate_supported_complete_graph_facts_v1,
    validate_supported_peptide_template_complete_graph_facts_v1,
};
pub(crate) use document_atom_oxidation_observation_v1::observe_current_document_atom_oxidation_v1;
pub use document_atom_oxidation_observation_v1::{
    DocumentAtomOxidationObservationRequestV1, DocumentAtomOxidationObservationV1,
    DocumentAtomOxidationRefusalV1, DocumentAtomOxidationResourceV1, DocumentAtomOxidationResultV1,
    DocumentAtomOxidationUnavailableReasonV1,
};
pub(crate) use document_bond_capacity_v1::evaluate_document_molecule_neutral_capacity_v1;
pub use document_bond_capacity_v1::{
    DOCUMENT_BOND_CAPACITY_SCHEMA_V1, DocumentBondCapacityErrorV1,
    DocumentBondCapacityNotCheckedReasonV1, DocumentBondCapacityOutcomeV1,
    DocumentBondCapacityRecordV1, DocumentBondCapacityRequestErrorV1,
    DocumentBondCapacityRequestV1, DocumentBondCapacitySourceV1, DocumentBondCapacityV1,
    inspect_document_bond_capacity_v1,
};
pub use document_molecule_composition_graph_v1::{
    DocumentMoleculeCompositionGraphErrorV1, document_molecule_composition_graph_v1,
};
pub use document_molecule_graph_v1::{
    DocumentMoleculeGraphError, DocumentMoleculeGraphV1, document_molecule_coordinate_graph_v1,
    document_molecule_graph_v1,
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
    build_document_molecule_inspection_v1, direct_projection_molecule_v1,
    inspect_document_molecule_v1, verify_molecule_observation_v1,
};
pub use document_molecule_molblock_v1::{
    DOCUMENT_MOLECULE_MOLBLOCK_PROFILE_V1, DOCUMENT_MOLECULE_MOLBLOCK_SCHEMA_V1,
    DocumentMoleculeMolblockErrorV1, DocumentMoleculeMolblockRequestV1, DocumentMoleculeMolblockV1,
    PreparedDocumentMoleculeMolblockV1, export_prepared_document_molecule_molblock_v1,
    prepare_document_molecule_molblock_v1,
};
pub use document_molecule_sdf_v1::{
    DOCUMENT_MOLECULE_SDF_PROFILE_V1, DOCUMENT_MOLECULE_SDF_SCHEMA_V1, DocumentMoleculeSdfErrorV1,
    DocumentMoleculeSdfRequestV1, DocumentMoleculeSdfV1, PreparedDocumentMoleculeSdfV1,
    export_prepared_document_molecule_sdf_v1, prepare_document_molecule_sdf_v1,
};
pub use document_molecule_smiles_v1::{
    DOCUMENT_MOLECULE_SMILES_PROFILE_V1, DOCUMENT_MOLECULE_SMILES_SCHEMA_V1,
    DocumentMoleculeSmilesErrorV1, DocumentMoleculeSmilesRequestV1, DocumentMoleculeSmilesV1,
    PreparedDocumentMoleculeSmilesV1, export_prepared_document_molecule_smiles_v1,
    prepare_document_molecule_smiles_v1,
};
pub use document_molecules_sdf_v2::{
    DOCUMENT_MOLECULES_SDF_PROFILE_V2, DOCUMENT_MOLECULES_SDF_SCHEMA_V2,
    DocumentMoleculesSdfErrorV2, DocumentMoleculesSdfRecordV2, DocumentMoleculesSdfRequestErrorV2,
    DocumentMoleculesSdfRequestV2, DocumentMoleculesSdfV2, PreparedDocumentMoleculesSdfV2,
    export_prepared_document_molecules_sdf_v2, prepare_document_molecules_sdf_from_source_ids_v2,
    prepare_document_molecules_sdf_v2,
};
pub use inchi_molecule_insertion_v1::{InchiMoleculeBuildError, build_inchi_molecule_insertion_v1};
pub use interchange_record_insertion_v1::{
    InterchangeRecordBuildErrorV1, build_interchange_record_batch_insertion_v1,
};
pub use molblock_molecule_insertion_v1::{
    MolblockMoleculeBuildError, build_molblock_molecule_insertion_v1,
};
pub use molblock_source_v1::{MolblockSourceErrorV1, read_molblock_file_v1};
pub use molecule_coordinate_generation_v1::{
    MoleculeCoordinateBuildError, build_molecule_coordinate_update_v1,
};
pub(crate) use ordinary_attachment_capacity_v1::{
    DocumentOrdinaryAttachmentAvailabilityV1, DocumentOrdinaryAttachmentReasonV1,
    OrdinaryAttachmentCandidateWitnessV1, admit_candidate_ordinary_attachment_capacity_v1,
};
pub use peptide_template_molecule_insertion_v1::{
    PeptideTemplateMoleculeBuildErrorV1, build_native_template_insertion_with_engine,
    build_supported_peptide_template_molecule_insertion_v1,
};
pub use sdf_source_v1::{SdfSourceErrorV1, read_sdf_file_v1};
pub use smiles_molecule_insertion_v1::{
    SmilesMoleculeBuildError, SmilesMoleculeInsertionError, build_smiles_molecule_insertion_v1,
    prepare_smiles_molecule_v1,
};
