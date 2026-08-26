use pyo3::prelude::*;

use super::atom_mark_binding::{PyAtomMarkActionV1, PyAtomMarkKindV1};
use super::atom_properties_binding::PyDocumentAtomPropertyChangeV1;
use super::bond_properties_binding::{PyDocumentBondPropertyChangeV1, PyDocumentBondStyleV1};
use super::bracket_binding::{
    PyBracketPairProjectionV1, PyDocumentBracketBoundsV1, PyDocumentBracketStyleV1,
    PyPreparedBracketInsertion,
};
pub(crate) use super::document_error_binding::operation_validation_error;
pub(crate) use super::document_error_binding::{
    DocumentError, DocumentInputError, DocumentLoadError, DocumentSerializationError, FerrumError,
    HistoryUnavailableError, InvalidAtomElementError, InvalidDestinationError,
    InvalidDocumentObjectIdError, OperationValidationError, PreparedOperationConsumedError,
    PreparedOperationError, PreparedOperationForeignSessionError, ProjectionError,
    PublicationError, PublicationNotStartedError, PublicationPossiblyCompletedError,
    RevisionConflictError, RevisionExhaustedError, UnknownDocumentObjectError, document_result,
    map_document_error, projection_error,
};
use super::document_operation_binding::PyDocumentOperationV1;
pub(crate) use super::document_session_binding::*;
use super::presentation_root_binding::PyPresentationRootProjectionV1;
use super::projection_binding::{
    PyArrowHeadShapeV1, PyArrowPathV1, PyArrowProjectionKindV1, PyArrowProjectionV1,
    PyAtomMarkProjectionV1, PyAtomProjectionV1, PyBondEndpointV1, PyBondProjectionV1,
    PyBoxShapeProjectionV1, PyCompactGroupProjectionV1, PyDocumentDirectRootV1,
    PyDocumentHaworthPositionV1, PyDocumentProjectionV1, PyFontFactsV1, PyMoleculeProjectionV1,
    PyPlusProjectionV1, PyPoint3V1, PyPolygonPathV1, PyPolygonProjectionV1, PyPolylinePathV1,
    PyPolylineProjectionV1, PyPresentationBoundsV1, PyPresentationFillV1, PyPresentationFontV1,
    PyPresentationProjectionIssueV1, PyPresentationStackProjectionV1, PyPresentationStrokeV1,
    PyPresentationTargetV1, PyProjectionIssueV1, PySessionDocumentObservationV1,
};
use super::render_binding;
pub(crate) use super::session_operation_result_binding::{
    PyAtomCreatedOutcomeV1, PyBondCreatedOutcomeV1, PyCreatedPresentationRootKindV1,
    PyCreatedPresentationRootOutcomeV1, PyDirectBondOperationOutcomeV1,
    PyInterchangeRecordBatchInsertedOutcomeV1, PyMoleculeHydrogensMaterializedOutcomeV1,
    PyMoleculeInsertedOutcomeV1, PyReactionCreatedOutcomeV1, PyReactionDefinitionDeletedOutcomeV1,
    PyReactionMembershipReplacedOutcomeV1, PySessionOperationOutcomeV1, PySessionOperationResultV1,
};

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("FerrumError", module.py().get_type::<FerrumError>())?;
    super::binding_feature_registration::initialize(module)?;
    module.add("DocumentError", module.py().get_type::<DocumentError>())?;
    module.add(
        "DocumentInputError",
        module.py().get_type::<DocumentInputError>(),
    )?;
    module.add(
        "DocumentLoadError",
        module.py().get_type::<DocumentLoadError>(),
    )?;
    module.add(
        "PreparedOperationForeignSessionError",
        module
            .py()
            .get_type::<PreparedOperationForeignSessionError>(),
    )?;
    module.add(
        "DocumentSerializationError",
        module.py().get_type::<DocumentSerializationError>(),
    )?;
    module.add(
        "RevisionConflictError",
        module.py().get_type::<RevisionConflictError>(),
    )?;
    module.add(
        "RevisionExhaustedError",
        module.py().get_type::<RevisionExhaustedError>(),
    )?;
    module.add(
        "HistoryUnavailableError",
        module.py().get_type::<HistoryUnavailableError>(),
    )?;
    module.add("ProjectionError", module.py().get_type::<ProjectionError>())?;
    module.add(
        "OperationValidationError",
        module.py().get_type::<OperationValidationError>(),
    )?;
    module.add(
        "InvalidAtomElementError",
        module.py().get_type::<InvalidAtomElementError>(),
    )?;
    module.add(
        "InvalidDocumentObjectIdError",
        module.py().get_type::<InvalidDocumentObjectIdError>(),
    )?;
    module.add(
        "UnknownDocumentObjectError",
        module.py().get_type::<UnknownDocumentObjectError>(),
    )?;
    module.add(
        "PreparedOperationError",
        module.py().get_type::<PreparedOperationError>(),
    )?;
    module.add(
        "PreparedOperationConsumedError",
        module.py().get_type::<PreparedOperationConsumedError>(),
    )?;
    module.add(
        "PublicationError",
        module.py().get_type::<PublicationError>(),
    )?;
    module.add(
        "InvalidDestinationError",
        module.py().get_type::<InvalidDestinationError>(),
    )?;
    module.add(
        "PublicationNotStartedError",
        module.py().get_type::<PublicationNotStartedError>(),
    )?;
    module.add(
        "PublicationPossiblyCompletedError",
        module.py().get_type::<PublicationPossiblyCompletedError>(),
    )?;
    super::attached_compact_group_binding::initialize(module)?;
    super::attached_cyclohexane_binding::initialize(module)?;
    super::free_compact_group_placement_binding::initialize(module)?;
    super::document_native_artifact_binding::register(module)?;
    module.add_class::<PyDocumentSession>()?;
    module
        .add_class::<super::live_document_operation_binding::PyLiveDocumentOperationReceiptV1>()?;
    module.add_class::<
        super::live_document_operation_binding::PyLiveCompactGroupMaterializationAvailabilityReceiptV1,
    >()?;
    module.add_class::<super::live_atom_chemistry_binding::PyLiveAtomOxidationObservationV1>()?;
    module.add_class::<super::live_document_smarts_query_v1::PyLiveDocumentSmartsReceiptV1>()?;
    module
        .add_class::<super::live_document_smarts_query_v1::PyLiveDocumentSmartsSelectedReadinessV1>(
        )?;
    module
        .add_class::<super::live_document_smarts_query_v1::PyLiveDocumentSmartsMoleculeSummaryV1>(
        )?;
    module.add_class::<super::live_document_smarts_query_v1::PyLiveDocumentSmartsRunSummaryV1>()?;
    module.add_class::<super::live_document_smarts_query_v1::PyLiveDocumentSmartsPaintV1>()?;
    module.add_class::<super::document_ingress_binding::PyXmlInputBudgetV1>()?;
    module.add_class::<super::document_ingress_binding::PyLocalInterchangeOpenDescriptorV1>()?;
    module.add_class::<super::document_ingress_binding::PyLocalInterchangeOpenRouteHandleV1>()?;
    module.add_class::<super::document_ingress_binding::PyPreparedLocalDocumentOpenV1>()?;
    module.add_class::<super::document_ingress_binding::PyLocalDocumentOriginTokenV1>()?;
    module.add_class::<
        super::document_interchange_receipt_binding::PyLocalInterchangeImportSummaryV1,
    >()?;
    module
        .add_class::<super::document_interchange_receipt_binding::PyLocalInterchangeRefusalV1>()?;
    module.add_class::<PyDocumentSnapshot>()?;
    module.add_class::<PySessionDocumentObservationV1>()?;
    module.add_class::<PyDocumentProjectionV1>()?;
    module.add_class::<PyDocumentDirectRootV1>()?;
    module.add_class::<PyPresentationStackProjectionV1>()?;
    module.add_class::<PyBracketPairProjectionV1>()?;
    module.add_class::<PyPresentationRootProjectionV1>()?;
    super::presentation_render_plan_binding::initialize(module)?;
    super::presentation_path_binding::register(module)?;
    super::presentation_text_binding::register(module)?;
    module.add_class::<PyArrowProjectionV1>()?;
    module.add_class::<PyArrowProjectionKindV1>()?;
    module.add_class::<PyArrowPathV1>()?;
    module.add_class::<PyArrowHeadShapeV1>()?;
    module.add_class::<PyPlusProjectionV1>()?;
    module.add_class::<PyPresentationFontV1>()?;
    module.add_class::<PyPolylineProjectionV1>()?;
    module.add_class::<PyBoxShapeProjectionV1>()?;
    module.add_class::<PyPolygonProjectionV1>()?;
    module.add_class::<PyPresentationTargetV1>()?;
    module.add_class::<PyPolylinePathV1>()?;
    module.add_class::<PyPolygonPathV1>()?;
    module.add_class::<PyPresentationStrokeV1>()?;
    module.add_class::<PyPresentationBoundsV1>()?;
    module.add_class::<PyPresentationFillV1>()?;
    module.add_class::<PyPresentationProjectionIssueV1>()?;
    module.add_class::<PyMoleculeProjectionV1>()?;
    module.add_class::<PyCompactGroupProjectionV1>()?;
    module.add_class::<PyAtomMarkProjectionV1>()?;
    module.add_class::<PyAtomProjectionV1>()?;
    module.add_class::<PyBondProjectionV1>()?;
    module.add_class::<PyDocumentHaworthPositionV1>()?;
    module.add_class::<PyBondEndpointV1>()?;
    module.add_class::<PyPoint3V1>()?;
    module.add_class::<PyFontFactsV1>()?;
    module.add_class::<PyProjectionIssueV1>()?;
    module.add_class::<PySessionOperationResultV1>()?;
    module.add_class::<PySessionOperationOutcomeV1>()?;
    module.add_class::<PyAtomCreatedOutcomeV1>()?;
    module.add_class::<PyBondCreatedOutcomeV1>()?;
    module.add_class::<PyCreatedPresentationRootKindV1>()?;
    module.add_class::<PyCreatedPresentationRootOutcomeV1>()?;
    module.add_class::<PyMoleculeHydrogensMaterializedOutcomeV1>()?;
    module.add_class::<PyMoleculeInsertedOutcomeV1>()?;
    module.add_class::<PyInterchangeRecordBatchInsertedOutcomeV1>()?;
    module.add_class::<PyDirectBondOperationOutcomeV1>()?;
    module.add_class::<PyReactionCreatedOutcomeV1>()?;
    module.add_class::<PyReactionMembershipReplacedOutcomeV1>()?;
    module.add_class::<PyReactionDefinitionDeletedOutcomeV1>()?;
    module.add_class::<PyDocumentOperationV1>()?;
    module.add_class::<PyAtomMarkActionV1>()?;
    module.add_class::<PyAtomMarkKindV1>()?;
    module.add_class::<PyDocumentAtomPropertyChangeV1>()?;
    module.add_class::<super::atom_rotation_binding::PyDocumentAtomRotationTargetV1>()?;
    module.add_class::<super::geometry_repair_binding::PyDocumentGeometryRepairKindV1>()?;
    module.add_class::<PyDocumentBondPropertyChangeV1>()?;
    module.add_class::<super::plus_properties_binding::PyDocumentPlusPropertyChangeV1>()?;
    module.add_class::<super::text_properties_binding::PyDocumentTextEditStyleV1>()?;
    module.add_class::<super::text_properties_binding::PyDocumentTextEditRunV1>()?;
    module.add_class::<super::text_properties_binding::PyDocumentTextPropertyChangeV1>()?;
    module.add_class::<super::presentation_deletion_binding::PyDocumentPresentationRootKindV1>()?;
    module.add_class::<super::presentation_stack_binding::PyDocumentPresentationStackOrderV1>()?;
    module
        .add_class::<super::presentation_stack_binding::PyDocumentPresentationRootSelectorV1>()?;
    module.add_class::<super::top_level_transform_binding::PyDocumentTopLevelRootKindV1>()?;
    module.add_class::<super::top_level_transform_binding::PyDocumentTopLevelRootSelectorV1>()?;
    module.add_class::<super::top_level_transform_binding::PyDocumentTopLevelAlignmentV1>()?;
    module.add_class::<super::top_level_transform_binding::PyDocumentTopLevelMirrorV1>()?;
    module.add_class::<super::arrow_properties_binding::PyDocumentArrowPropertyChangeV1>()?;
    module
        .add_class::<super::geometric_properties_binding::PyDocumentGeometricPropertyChangeV1>()?;
    module.add_class::<super::wavy_properties_binding::PyDocumentWavyPropertyChangeV1>()?;
    module.add_class::<super::bracket_binding::PyDocumentBracketPropertyChangeV1>()?;
    module.add_class::<PyPreparedWavyInsertion>()?;
    module.add_class::<PyDocumentBracketStyleV1>()?;
    module.add_class::<PyDocumentBracketBoundsV1>()?;
    module.add_class::<PyPreparedBracketInsertion>()?;
    module.add_class::<PyDocumentBondOrderV1>()?;
    module.add_class::<PyDocumentBondPresentationV1>()?;
    module.add_class::<PyDocumentBondStyleV1>()?;
    module.add_class::<super::direct_haworth_binding::PyDirectHaworthSourceV1>()?;
    module.add_class::<PyPublication>()?;
    module.add_class::<PySaveOutcome>()?;
    render_binding::initialize(module)
}
