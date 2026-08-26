//! Catalog and presentation placement execution.

use super::*;

pub(super) fn execute_catalog_list(
    request: CatalogListRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let manifest = catalog_manifest_v1();
    let family = request.family.map(|value| match value {
        ProtocolCatalogFamilyV1::System => CatalogFamilyV1::System,
        ProtocolCatalogFamilyV1::Biomolecule => CatalogFamilyV1::Biomolecule,
    });
    let entries = search_catalog_v1(
        family,
        request.category.as_deref(),
        request.query.as_deref(),
    )
    .into_iter()
    .map(|entry| CatalogEntrySummaryV1 {
        id: entry.key().as_str().to_owned(),
        family: match entry.family() {
            CatalogFamilyV1::System => ProtocolCatalogFamilyV1::System,
            CatalogFamilyV1::Biomolecule => ProtocolCatalogFamilyV1::Biomolecule,
        },
        category: CatalogCategorySummaryV1 {
            id: entry.category().key().to_owned(),
            name: entry.category().label().to_owned(),
            order: entry.category().order(),
        },
        name: entry.label().to_owned(),
        provenance: CatalogProvenanceSummaryV1 {
            source_kind: entry.provenance().source_kind().to_owned(),
            source_id: entry.provenance().source_id().to_owned(),
            license_spdx: entry.provenance().license_spdx().to_owned(),
        },
    })
    .collect();
    Ok(OperationProtocolOutcomeV1::CatalogList {
        catalog_schema: manifest.schema().to_owned(),
        catalog_version: manifest.catalog_version().to_owned(),
        entries,
    })
}

pub(super) fn execute_catalog_insert(
    request: CatalogInsertRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.expected_revision != 0 {
        return Err(ExecutionFailureV1::catalog_refusal(
            CatalogPlacementErrorV1::StaleSnapshot,
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    if session
        .snapshot()
        .map_err(|_| ExecutionFailureV1::catalog_refusal(CatalogPlacementErrorV1::SessionConflict))?
        .digest()
        != &digest
    {
        return Err(ExecutionFailureV1::catalog_refusal(
            CatalogPlacementErrorV1::StaleSnapshot,
        ));
    }
    let anchor = PresentationGesturePoint2V1::new(request.anchor_x, request.anchor_y)
        .map_err(|_| ExecutionFailureV1::catalog_refusal(CatalogPlacementErrorV1::InvalidPoint))?;
    let operation = SessionOperation::V1(SessionOperationV1::PlaceCatalogMoleculeV1(
        resolve_catalog_molecule_placement_v1(&request.catalog_id, anchor)
            .map_err(ExecutionFailureV1::catalog_refusal)?,
    ));
    let mut prepared = session
        .prepare_session_operation_transition_v1(
            ferrum_document::SessionOperationTransitionRequestV1::new(
                request.expected_revision,
                operation,
                TransitionAuthorizationV1::None,
            ),
        )
        .map_err(|error| ExecutionFailureV1::catalog_refusal(catalog_prepare_error(error)))?;
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .map_err(|error| ExecutionFailureV1::catalog_refusal(catalog_commit_error(error)))?;
    let SessionOperationOutcomeV1::CatalogMoleculePlacementV1(outcome) = result.outcome() else {
        return Err(ExecutionFailureV1::catalog_refusal(
            CatalogPlacementErrorV1::SessionConflict,
        ));
    };
    let snapshot = result.observation().snapshot();
    Ok(OperationProtocolOutcomeV1::CatalogInsert {
        document: snapshot.cdml().to_owned(),
        identifier: outcome.root_identifier().as_str().to_owned(),
        committed_revision: snapshot.revision(),
        document_fence: DocumentRequestFenceV1 {
            expected_revision: 0,
            expected_digest_hex: hex_digest(snapshot.digest()),
        },
    })
}

fn catalog_prepare_error(error: DocumentSessionError) -> CatalogPlacementErrorV1 {
    match error {
        DocumentSessionError::RendererAdmission => CatalogPlacementErrorV1::RenderPreparation,
        DocumentSessionError::RevisionConflict { .. } => CatalogPlacementErrorV1::StaleSnapshot,
        _ => CatalogPlacementErrorV1::SessionConflict,
    }
}

fn catalog_commit_error(error: AdmittedSessionTransitionRefusalV1) -> CatalogPlacementErrorV1 {
    match error {
        AdmittedSessionTransitionRefusalV1::StaleSnapshot => CatalogPlacementErrorV1::StaleSnapshot,
        AdmittedSessionTransitionRefusalV1::ForeignSession => {
            CatalogPlacementErrorV1::ForeignSession
        }
        AdmittedSessionTransitionRefusalV1::Consumed => CatalogPlacementErrorV1::Consumed,
        AdmittedSessionTransitionRefusalV1::RendererAdmission => {
            CatalogPlacementErrorV1::RenderPreparation
        }
        AdmittedSessionTransitionRefusalV1::ProvisionalCapability => {
            CatalogPlacementErrorV1::SessionConflict
        }
    }
}

pub(super) fn parse_digest_hex(value: &str) -> Result<[u8; 32], ExecutionFailureV1> {
    if value.len() != 64 {
        return Err(ExecutionFailureV1::document_invalid(
            "expected_digest_hex must contain 64 lowercase or uppercase hexadecimal characters"
                .to_owned(),
        ));
    }
    let mut digest = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let text = std::str::from_utf8(pair).expect("hex input is ASCII-sized");
        digest[index] = u8::from_str_radix(text, 16).map_err(|_| {
            ExecutionFailureV1::document_invalid(
                "expected_digest_hex must contain hexadecimal characters".to_owned(),
            )
        })?;
    }
    Ok(digest)
}

pub(crate) fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
