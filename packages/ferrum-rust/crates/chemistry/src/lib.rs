//! Stable, owned chemistry values at Ferrum's engine boundary.
//!
//! This crate intentionally knows neither CDML nor a native toolkit.  An engine
//! receives a validated [`MolGraph`] and returns a new one, keeping callers free
//! of foreign handles, borrowed buffers, and toolkit-specific representations.

mod adapter;
mod adapter_contract;
mod cdxml_decoder;
mod cml;
mod codec;
mod composition;
mod element;
mod engine;
mod interchange;
mod interchange_sdf;
mod model;
mod native_engine;
mod oxidation_state_v1;
mod sdf;

pub use crate::adapter::{ExplicitAdapterError, load_explicit_adapter};
pub use crate::cdxml_decoder::{
    CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1, CdxmlBondPresentationV1,
    CdxmlDecodedDocumentV1, CdxmlDecodedRecordV1, CdxmlDecoderErrorV1, CdxmlLossCategoryV1,
    CdxmlRefusalReasonV1, decode_cdxml_bytes_v1,
};
pub use crate::cml::{
    CmlDecodedDocumentV1, CmlDecodedRecordV1, CmlDecoderErrorV1, CmlEncoderErrorV1,
    CmlEncoderRefusalReasonV1, CmlRefusalReasonV1, CmlSourceAtomV1, CmlSourceBondV1,
    decode_cml_bytes_v1, encode_cml_decoded_document_v1, encode_cml_interchange_records_v1,
};
pub use crate::codec::{
    CanonicalSmilesError, INCHI_INSPECTION_SCHEMA_V1, InchiExportError, InchiInspectionError,
    InchiInspectionV1, MOLBLOCK_INSPECTION_SCHEMA_V1, MolblockExportError, MolblockInspectionError,
    MolblockInspectionV1, MoleculeInspectionFactsV1, SDF_INSPECTION_SCHEMA_V1,
    SMILES_INSPECTION_SCHEMA_V1, SdfExportError, SdfInspectionError, SdfInspectionV1,
    SdfPropertyInspectionV1, SdfRecordInspectionV1, SmartsExportError, SmilesAtomInspectionV1,
    SmilesBondInspectionV1, SmilesInspectionError, SmilesInspectionV1, SmilesPointInspectionV1,
    canonical_smiles_from_smiles, inchi_from_smiles, inspect_inchi, inspect_molblock, inspect_sdf,
    inspect_smiles, molblock_from_smiles, molecule_inspection_facts, sdf_from_smiles,
    smarts_from_smiles,
};
pub use crate::interchange::{
    CDXML_SIMPLE_MOLECULE_IMPORT_FORMAT_ID_V1, CDXML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1,
    CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1, INTERCHANGE_MAX_TEXT_BYTES_V1,
    InterchangeCodecErrorV1, InterchangeFormatV1, InterchangePropertyV1, InterchangeRecordV1,
    decode_non_cdml_interchange_v1, encode_non_cdml_interchange_v1,
};
pub use crate::interchange_sdf::interchange_record_from_sdf_v1;

pub use crate::composition::{
    CompositionAggregationError, CompositionBuildError, CompositionElementKey, ElementCount,
    ElementMassPercentage, MoleculeComposition, MoleculeCompositionEntry,
};
pub use crate::engine::{
    ChemEngine, ChemistryError, InchiMode, KekulizeOptions, KekulizeOptionsError, MolblockVersion,
    NativeTextOutputLimit, NativeTextOutputLimitError, SmartsMatchOptions, SmartsMatchOptionsError,
    SmartsMatchResult, SmartsMatchUnavailableReason, UnavailableChemEngine,
};
pub use crate::model::{
    AtomChirality, AtomicNumber, BondDirection, BondOrder, BondStereo, Coordinates, MolAtom,
    MolBond, MolBondDirectionError, MolGraph, MolGraphError, Point2, SmilesMolecule,
};
pub use crate::native_engine::{
    INCHI_MAX_INPUT_BYTES, MOLBLOCK_MAX_INPUT_BYTES, NATIVE_SMILES_MAX_INPUT_BYTES,
    NATIVE_SMILES_MAX_OUTPUT_BYTES, NativeChemEngine, SDF_MAX_INPUT_BYTES, validate_inchi_input,
    validate_molblock_input, validate_molblock_title, validate_sdf_input, validate_smiles_input,
};
pub use crate::ordinary_attachment_capacity_v1::{
    OrdinaryAttachmentAnchorV1, OrdinaryAttachmentBondOrderV1,
    OrdinaryAttachmentCapacityAdmissionV1, OrdinaryAttachmentCapacityOutcomeV1,
    OrdinaryAttachmentCapacityReasonV1, OrdinaryAttachmentCapacityRecoveryV1,
    OrdinaryAttachmentProfileV1, admit_ordinary_attachment_capacity_v1,
};
pub use crate::oxidation_state_v1::{
    OXIDATION_STATE_CONVENTION_V1, OxidationStateErrorV1, OxidationStateObservationV1,
    OxidationStateResourceV1, OxidationStateRootAdmissionV1, OxidationStateUnavailableReasonV1,
    admit_oxidation_state_root_v1, observe_admitted_oxidation_state_v1, observe_oxidation_state_v1,
};
pub use crate::sdf::{ImportedSdfRecord, SdfError, SdfProperty, SdfRecord, compose_sdf_record};

pub use crate::adapter_contract::{ADAPTER_ABI_VERSION, NATIVE_SDF_MAX_RECORDS};
pub(crate) use crate::adapter_contract::{
    FERRUM_CHEM_ALL_KNOWN_CAPABILITIES, FERRUM_CHEM_CALL_ALLOCATION_FAILURE,
    FERRUM_CHEM_CAPABILITY_COMPOSITION, FERRUM_CHEM_CAPABILITY_GENERATE_2D,
    FERRUM_CHEM_CAPABILITY_INCHI, FERRUM_CHEM_CAPABILITY_KEKULIZE, FERRUM_CHEM_CAPABILITY_MOLFILE,
    FERRUM_CHEM_CAPABILITY_MOLFILE_READ, FERRUM_CHEM_CAPABILITY_MOLFILE_TITLE,
    FERRUM_CHEM_CAPABILITY_SDF_READ, FERRUM_CHEM_CAPABILITY_SDF_WRITE,
    FERRUM_CHEM_CAPABILITY_SMARTS, FERRUM_CHEM_CAPABILITY_SMARTS_MATCH,
    FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE, FERRUM_CHEM_CAPABILITY_SMILES_WRITE,
    FERRUM_CHEM_COMPOSITION_ENTRY_BYTES, FERRUM_CHEM_COMPOSITION_FLAGS_NONE,
    FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES, FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES,
    FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES, FERRUM_CHEM_COMPOSITION_WIRE_VERSION,
    FERRUM_CHEM_COORDINATE_BYTES, FERRUM_CHEM_GRAPH_ATOM_BYTES, FERRUM_CHEM_GRAPH_BOND_BYTES,
    FERRUM_CHEM_GRAPH_FLAGS_NONE, FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_GRAPH_WIRE_VERSION, FERRUM_CHEM_INCHI_FLAGS_NONE, FERRUM_CHEM_INCHI_KEY_BYTES,
    FERRUM_CHEM_INCHI_MAX_BYTES, FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN,
    FERRUM_CHEM_INCHI_MODE_STANDARD, FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_INCHI_WIRE_VERSION, FERRUM_CHEM_KEKULIZE_ATOM_BYTES,
    FERRUM_CHEM_KEKULIZE_BOND_BYTES, FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE, FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE,
    FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED, FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS,
    FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE, FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE,
    FERRUM_CHEM_KEKULIZE_MAX_ATOMS, FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS,
    FERRUM_CHEM_KEKULIZE_MAX_BONDS, FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES,
    FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL, FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS,
    FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES, FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_KEKULIZE_WIRE_VERSION, FERRUM_CHEM_MAX_RESPONSE_BYTES,
    FERRUM_CHEM_MOLBLOCK_FLAGS_NONE, FERRUM_CHEM_MOLBLOCK_FORMAT_V2000,
    FERRUM_CHEM_MOLBLOCK_FORMAT_V3000, FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_MOLBLOCK_WIRE_VERSION, FERRUM_CHEM_MOLECULE_ATOM_BYTES,
    FERRUM_CHEM_MOLECULE_BOND_BYTES, FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_RESULT_DEPICTION_FAILURE, FERRUM_CHEM_RESULT_INTERNAL_FAILURE,
    FERRUM_CHEM_RESULT_INVALID_MOLECULE, FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
    FERRUM_CHEM_RESULT_OK, FERRUM_CHEM_RESULT_RESOURCE_LIMIT,
    FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE, FERRUM_CHEM_SDF_FLAGS_NONE,
    FERRUM_CHEM_SDF_MAX_PROPERTIES, FERRUM_CHEM_SDF_MAX_RECORDS,
    FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES, FERRUM_CHEM_SDF_RECORD_HEADER_BYTES,
    FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES, FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_SDF_WIRE_VERSION, FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED,
    FERRUM_CHEM_SMARTS_MATCH_MAX_MATRIX_CELLS, FERRUM_CHEM_SMARTS_MATCH_MAX_QUERY_BYTES,
    FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS, FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES,
    FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES, FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE, FERRUM_CHEM_SMARTS_MATCH_STATUS_OK,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET, FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION,
    FERRUM_CHEM_SMILES_MAX_BYTES, FERRUM_CHEM_SMILES_WRITE_MAX_BYTES, FERRUM_CHEM_TEXT_FLAGS_NONE,
    FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES, FERRUM_CHEM_TEXT_WIRE_VERSION,
    FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES, FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION,
};
mod ordinary_attachment_capacity_v1;
