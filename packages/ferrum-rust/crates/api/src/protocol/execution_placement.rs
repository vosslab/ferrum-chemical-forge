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
            CatalogPlacementErrorV2::StaleSnapshot,
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    let anchor = PresentationGesturePoint2V1::new(request.anchor_x, request.anchor_y)
        .map_err(|_| ExecutionFailureV1::catalog_refusal(CatalogPlacementErrorV2::InvalidPoint))?;
    let fence = DocumentFenceV1::new(request.expected_revision, digest);
    let gesture = begin_api_catalog_placement_v2(&session, fence, &request.catalog_id)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let mut preview = preview_api_catalog_placement_v2(&mut session, &gesture, anchor)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let mut prepared = prepare_api_catalog_placement_v2(&mut session, &gesture, &mut preview)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let committed = commit_api_catalog_placement_v2(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::catalog_refusal)?;
    let snapshot = committed.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::CatalogInsert {
        document: snapshot.cdml().to_owned(),
        identifier: committed.identifier().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
    })
}

pub(super) fn execute_presentation_vector_create(
    request: PresentationVectorCreateRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    match request.appearance_policy {
        ProtocolPresentationVectorAppearancePolicyV1::EffectiveDrawingStandard => {}
    }
    if request.expected_revision != 0 {
        return Err(ExecutionFailureV1::vector_refusal(
            PresentationVectorGestureErrorV1::StaleSnapshot,
        ));
    }
    let mut session = admit_document(&request.document)?;
    let digest = parse_digest_hex(&request.expected_digest_hex)?;
    let kind = match request.vector_kind {
        ProtocolPresentationVectorKindV1::Line => PresentationVectorKindV1::Line,
        ProtocolPresentationVectorKindV1::Rectangle => PresentationVectorKindV1::Rectangle,
        ProtocolPresentationVectorKindV1::Square => PresentationVectorKindV1::Square,
        ProtocolPresentationVectorKindV1::Oval => PresentationVectorKindV1::Oval,
        ProtocolPresentationVectorKindV1::Circle => PresentationVectorKindV1::Circle,
    };
    let start =
        PresentationGesturePoint2V1::new(request.start_x, request.start_y).map_err(|_| {
            ExecutionFailureV1::document_invalid("vector start point must be finite".to_owned())
        })?;
    let end = PresentationGesturePoint2V1::new(request.end_x, request.end_y).map_err(|_| {
        ExecutionFailureV1::document_invalid("vector end point must be finite".to_owned())
    })?;
    let fence = DocumentFenceV1::new(request.expected_revision, digest);
    let gesture = begin_api_presentation_vector_gesture_v1(&session, fence, kind, start)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let preview = preview_api_presentation_vector_gesture_v1(&session, &gesture, end)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let mut prepared = prepare_api_presentation_vector_gesture_v1(&mut session, &gesture, &preview)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let committed = commit_api_presentation_vector_gesture_v1(&mut session, &mut prepared)
        .map_err(ExecutionFailureV1::vector_refusal)?;
    let renderer_observation =
        document_observation_from_accepted_operation_v1(committed.result().observation())
            .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let plan = compose_document_render_plan_v1(&renderer_observation)
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(ExecutionFailureV1::internal(
            "renderer-preflighted vector commit produced an excluded root".to_owned(),
        ));
    }
    let immutable_renderer_observation = serde_json::to_value(renderer_observation.wire())
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let snapshot = committed.result().observation().snapshot();
    Ok(OperationProtocolOutcomeV1::PresentationVectorCreate {
        document: snapshot.cdml().to_owned(),
        identifier: committed.root().presentation_id().as_str().to_owned(),
        input_revision: 0,
        committed_revision: snapshot.revision(),
        next_input_expected_revision: 0,
        digest_hex: hex_digest(snapshot.digest()),
        renderer_observation: immutable_renderer_observation,
    })
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

pub(super) fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
