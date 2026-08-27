//! Public adapter for one fenced generic compact-group attachment.

use ferrum_document::{
    AttachCompactGroupV1, AttachedCompactGroupReleaseV1, AttachedCompactGroupSessionErrorV1,
    AttachedCompactGroupTargetV1, DocumentFenceV1,
};

use super::document_request_parse_v1::{parse_document_object_id, parse_sha256_hex};
use super::execution::{ExecutionFailureV1, admit_document, hex_digest};
use super::*;

const ATTACHMENT_SCHEMA_V1: &str = "ferrum-document-compact-group-attachment-v1";

pub(super) fn execute_document_compact_group_attach(
    request: DocumentCompactGroupAttachmentRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut session = admit_document(&request.document.cdml)?;
    let source_revision = request.document.expected_revision;
    let source_digest_hex = request.document.expected_digest_hex.clone();
    let source_molecule_id = request.molecule_id.clone();
    let source_anchor_atom_id = request.anchor_atom_id.clone();
    let molecule_id = parse_document_object_id(&request.molecule_id, "molecule_id")?;
    let anchor_atom_id = parse_document_object_id(&request.anchor_atom_id, "anchor_atom_id")?;
    let expected_digest = parse_sha256_hex(&request.document.expected_digest_hex)?;
    let release = AttachedCompactGroupReleaseV1::new(request.release.x, request.release.y)
        .map_err(|_| {
            attachment_refusal(
                ProtocolCompactGroupAttachmentCategoryV1::InvalidRelease,
                ProtocolCompactGroupAttachmentRecoveryV1::ChangeRelease,
            )
        })?;
    let target = AttachedCompactGroupTargetV1::new(molecule_id, anchor_atom_id);
    let mut pending = session
        .prepare_attach_compact_group_v1(
            DocumentFenceV1::new(source_revision, expected_digest),
            target,
            AttachCompactGroupV1::new(request.catalog_key.into_document(), release),
        )
        .map_err(map_attachment_error)?;
    let committed = session
        .commit_attach_compact_group_v1(&mut pending)
        .map_err(map_attachment_error)?;
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let document = snapshot.cdml().to_owned();
    let stateless_snapshot = admit_document(&document)?
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    Ok(OperationProtocolOutcomeV1::DocumentCompactGroupAttach {
        attachment: DocumentCompactGroupAttachmentResultV1 {
            schema: ATTACHMENT_SCHEMA_V1.to_owned(),
            source_revision,
            source_digest_hex,
            molecule_id: source_molecule_id,
            anchor_atom_id: source_anchor_atom_id,
            catalog_key: request.catalog_key,
            compact_group_id: committed.compact_group_object_id().as_str().to_owned(),
            document,
            document_fence: DocumentRequestFenceV1 {
                expected_revision: stateless_snapshot.revision(),
                expected_digest_hex: hex_digest(stateless_snapshot.digest()),
            },
        },
    })
}

fn map_attachment_error(error: AttachedCompactGroupSessionErrorV1) -> ExecutionFailureV1 {
    let (category, recovery) = match error {
        AttachedCompactGroupSessionErrorV1::StaleRevision
        | AttachedCompactGroupSessionErrorV1::StaleDigest => (
            ProtocolCompactGroupAttachmentCategoryV1::StaleDocumentFence,
            ProtocolCompactGroupAttachmentRecoveryV1::RefreshAndRetry,
        ),
        AttachedCompactGroupSessionErrorV1::UnknownMolecule
        | AttachedCompactGroupSessionErrorV1::UnknownAnchor => (
            ProtocolCompactGroupAttachmentCategoryV1::UnknownTarget,
            ProtocolCompactGroupAttachmentRecoveryV1::CorrectTarget,
        ),
        AttachedCompactGroupSessionErrorV1::ForeignTarget => (
            ProtocolCompactGroupAttachmentCategoryV1::ForeignTarget,
            ProtocolCompactGroupAttachmentRecoveryV1::CorrectTarget,
        ),
        AttachedCompactGroupSessionErrorV1::InvalidPose => (
            ProtocolCompactGroupAttachmentCategoryV1::InvalidRelease,
            ProtocolCompactGroupAttachmentRecoveryV1::ChangeRelease,
        ),
        AttachedCompactGroupSessionErrorV1::CandidateAdmission => (
            ProtocolCompactGroupAttachmentCategoryV1::CandidateAdmission,
            ProtocolCompactGroupAttachmentRecoveryV1::ChooseAvailableTarget,
        ),
        AttachedCompactGroupSessionErrorV1::RendererAdmission => (
            ProtocolCompactGroupAttachmentCategoryV1::RendererAdmission,
            ProtocolCompactGroupAttachmentRecoveryV1::DocumentUnchanged,
        ),
        AttachedCompactGroupSessionErrorV1::SessionConflict
        | AttachedCompactGroupSessionErrorV1::ForeignSession
        | AttachedCompactGroupSessionErrorV1::Consumed => (
            ProtocolCompactGroupAttachmentCategoryV1::SessionConflict,
            ProtocolCompactGroupAttachmentRecoveryV1::RefreshAndRetry,
        ),
    };
    attachment_refusal(category, recovery)
}

fn attachment_refusal(
    category: ProtocolCompactGroupAttachmentCategoryV1,
    recovery: ProtocolCompactGroupAttachmentRecoveryV1,
) -> ExecutionFailureV1 {
    ExecutionFailureV1::compact_group_attachment_refusal(CompactGroupAttachmentRefusalV1 {
        category,
        recovery,
    })
}
