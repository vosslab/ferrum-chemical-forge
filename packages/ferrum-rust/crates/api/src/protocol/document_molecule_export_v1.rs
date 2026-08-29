//! Protocol adapter for one selected direct-root text export.

use ferrum_chemistry::ChemistryError;
use ferrum_document::{
    DocumentMoleculeExportError, DocumentMoleculeExportFormat, DocumentMoleculeExportRequest,
    DocumentObjectIdV1, export_prepared_document_molecule, prepare_document_molecule_export,
};

use super::super::dto::{
    DocumentMoleculeExportFormatV1, DocumentMoleculeExportRefusalV1,
    DocumentMoleculeExportRequestV1, DocumentMoleculeExportResultV1, OperationProtocolOutcomeV1,
    ProtocolDocumentMoleculeExportCategoryV1, ProtocolDocumentMoleculeExportRecoveryV1,
};
use super::super::frozen_document_snapshot_v1::{
    FrozenDocumentSnapshotAdmissionErrorV1, FrozenDocumentSnapshotV1,
};
use super::super::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1};
use super::{ExecutionFailureV1, hex_digest};

pub(super) fn execute_document_molecule_export<R: ChemistryRuntimeV1>(
    request: DocumentMoleculeExportRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let snapshot = FrozenDocumentSnapshotV1::admit(
        &request.document.cdml,
        request.document.expected_revision,
        &request.document.expected_digest_hex,
    )
    .map_err(map_snapshot_error)?;
    let molecule_id = DocumentObjectIdV1::parse(request.molecule_id.clone()).map_err(|_| {
        export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::UnknownOrNonDirectRoot,
            ProtocolDocumentMoleculeExportRecoveryV1::SelectDirectMoleculeRoot,
        )
    })?;
    let prepared = prepare_document_molecule_export(
        snapshot.observation(),
        &DocumentMoleculeExportRequest::new(
            snapshot.observation().snapshot().revision(),
            *snapshot.observation().snapshot().digest(),
            molecule_id,
            map_format(request.format),
        ),
    )
    .map_err(map_export_error)?;
    let export = runtime
        .with_engine(|engine| {
            Ok(export_prepared_document_molecule(engine, prepared).map_err(map_export_error))
        })
        .map_err(map_runtime_error)??;
    Ok(OperationProtocolOutcomeV1::DocumentMoleculeExport {
        export: DocumentMoleculeExportResultV1 {
            source_revision: snapshot.source_revision(),
            source_digest_hex: hex_digest(snapshot.source_digest()),
            molecule_id: request.molecule_id,
            format: request.format,
            text: export.text().to_owned(),
        },
    })
}

const fn map_format(format: DocumentMoleculeExportFormatV1) -> DocumentMoleculeExportFormat {
    match format {
        DocumentMoleculeExportFormatV1::MolfileV2000 => DocumentMoleculeExportFormat::MolfileV2000,
        DocumentMoleculeExportFormatV1::MolfileV3000 => DocumentMoleculeExportFormat::MolfileV3000,
        DocumentMoleculeExportFormatV1::SdfV2000 => DocumentMoleculeExportFormat::SdfV2000,
        DocumentMoleculeExportFormatV1::SdfV3000 => DocumentMoleculeExportFormat::SdfV3000,
        DocumentMoleculeExportFormatV1::CanonicalSmiles => {
            DocumentMoleculeExportFormat::CanonicalSmiles
        }
        DocumentMoleculeExportFormatV1::InchiStandard => {
            DocumentMoleculeExportFormat::InchiStandard
        }
        DocumentMoleculeExportFormatV1::InchiFixedHydrogen => {
            DocumentMoleculeExportFormat::InchiFixedHydrogen
        }
    }
}

fn map_snapshot_error(error: FrozenDocumentSnapshotAdmissionErrorV1) -> ExecutionFailureV1 {
    match error {
        FrozenDocumentSnapshotAdmissionErrorV1::MalformedDigest(message) => {
            ExecutionFailureV1::invalid_request(message)
        }
        FrozenDocumentSnapshotAdmissionErrorV1::DigestMismatch => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::SnapshotNotAdmitted,
            ProtocolDocumentMoleculeExportRecoveryV1::RefreshAuthenticatedSnapshot,
        ),
        FrozenDocumentSnapshotAdmissionErrorV1::DocumentAdmission(_) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::SnapshotNotAdmitted,
            ProtocolDocumentMoleculeExportRecoveryV1::RefreshAuthenticatedSnapshot,
        ),
        FrozenDocumentSnapshotAdmissionErrorV1::DocumentInvalid(_) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::SnapshotNotAdmitted,
            ProtocolDocumentMoleculeExportRecoveryV1::RefreshAuthenticatedSnapshot,
        ),
        FrozenDocumentSnapshotAdmissionErrorV1::Internal(message) => {
            ExecutionFailureV1::internal(message)
        }
    }
}

fn map_runtime_error(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    match error {
        ChemistryRuntimeErrorV1::Unavailable | ChemistryRuntimeErrorV1::Chemistry(_) => {
            export_refusal(
                ProtocolDocumentMoleculeExportCategoryV1::ChemistryUnavailable,
                ProtocolDocumentMoleculeExportRecoveryV1::RestoreChemistryRuntime,
            )
        }
    }
}

fn map_export_error(error: DocumentMoleculeExportError) -> ExecutionFailureV1 {
    match error {
        DocumentMoleculeExportError::Chemistry(ChemistryError::TextOutputLimitExceeded {
            ..
        }) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::OutputLimitExceeded,
            ProtocolDocumentMoleculeExportRecoveryV1::SelectSmallerRoot,
        ),
        DocumentMoleculeExportError::Chemistry(_) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::ChemistryUnavailable,
            ProtocolDocumentMoleculeExportRecoveryV1::RestoreChemistryRuntime,
        ),
        DocumentMoleculeExportError::Observation(
            ferrum_document::DocumentMoleculeInspectionErrorV1::UnknownDirectMolecule { .. },
        )
        | DocumentMoleculeExportError::Metadata(
            ferrum_document::InterchangeRecordMetadataErrorV1::UnknownDirectMolecule,
        ) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::UnknownOrNonDirectRoot,
            ProtocolDocumentMoleculeExportRecoveryV1::SelectDirectMoleculeRoot,
        ),
        DocumentMoleculeExportError::Observation(_) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::SnapshotNotAdmitted,
            ProtocolDocumentMoleculeExportRecoveryV1::RefreshAuthenticatedSnapshot,
        ),
        DocumentMoleculeExportError::Metadata(_)
        | DocumentMoleculeExportError::UnsupportedMolecule(_)
        | DocumentMoleculeExportError::Sdf(_) => export_refusal(
            ProtocolDocumentMoleculeExportCategoryV1::RepresentationUnsupported,
            ProtocolDocumentMoleculeExportRecoveryV1::ChooseSupportedRepresentation,
        ),
    }
}

fn export_refusal(
    category: ProtocolDocumentMoleculeExportCategoryV1,
    recovery: ProtocolDocumentMoleculeExportRecoveryV1,
) -> ExecutionFailureV1 {
    ExecutionFailureV1::document_molecule_export_refusal(DocumentMoleculeExportRefusalV1 {
        category,
        recovery,
    })
}

#[cfg(test)]
#[test]
fn native_text_output_limit_has_the_closed_protocol_refusal() {
    let failure = map_export_error(DocumentMoleculeExportError::Chemistry(
        ChemistryError::TextOutputLimitExceeded {
            codec: "SMILES",
            maximum: Some(128 * 1024),
        },
    ));

    assert_eq!(
        failure.document_molecule_export_refusal,
        Some(DocumentMoleculeExportRefusalV1 {
            category: ProtocolDocumentMoleculeExportCategoryV1::OutputLimitExceeded,
            recovery: ProtocolDocumentMoleculeExportRecoveryV1::SelectSmallerRoot,
        })
    );
}
