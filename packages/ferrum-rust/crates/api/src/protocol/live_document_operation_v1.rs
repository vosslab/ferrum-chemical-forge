//! Session-owned execution for the closed live-document operation subset.
//!
//! The public protocol remains stateless. This adapter authenticates a normal
//! V1 request against an already-owned session and returns the same protocol
//! envelope together with an installable result only when that session changed.

use ferrum_document::{
    DocumentCompactGroupMaterializationRefusalV1, DocumentMoleculeHydrogenMaterializationRefusalV1,
    DocumentObjectIdV1, DocumentSession, DocumentSessionError, SessionOperation,
    SessionOperationError, SessionOperationOutcomeV1, SessionOperationResultV1,
    SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
};
use serde::{Deserialize, Serialize};

use super::document_compact_group_materialization_v1::{
    compact_group_materialization_outcome, compact_refusal,
    execute_document_compact_group_materialize_transition_on_session, parse_digest,
};
use super::document_hydrogen_materialization_v1::execute_document_molecule_hydrogen_materialize_on_session;
use super::execution::{
    ExecutionFailureV1, OperationProtocolAdmissionV1, admit_operation_request_v1,
    admit_shared_response_budget_v1, canonical_protocol_envelope_json_v1, hex_digest,
    operation_error_response,
};
use super::*;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveDocumentCompactGroupOperationEnvelopeV1 {
    #[serde(rename = "schema")]
    _schema: ProtocolRequestSchemaV1,
    request_id: String,
    operation: LiveDocumentCompactGroupMaterializationRequestV1,
}

#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum LiveDocumentCompactGroupMaterializationRequestV1 {
    #[serde(rename = "document.compact-group.materialize.v1")]
    Materialize {
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        compact_group_object_id: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LiveDocumentCompactGroupMaterializationAvailabilityRequestV1 {
    expected_revision: u64,
    expected_digest_hex: String,
    molecule_object_id: String,
    compact_group_object_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CompactGroupMaterializationAvailabilityV1 {
    Eligible,
    StaleDocumentFence,
    UnknownOrForeignTarget,
    IneligibleTarget,
    RendererPreparationRefused,
    InvalidDocumentState,
}

#[derive(Serialize)]
struct LiveDocumentCompactGroupMaterializationAvailabilityResponseV1 {
    schema: &'static str,
    document_fence: LiveDocumentCompactGroupMaterializationAvailabilityFenceV1,
    molecule_object_id: String,
    compact_group_object_id: String,
    availability: CompactGroupMaterializationAvailabilityV1,
}

#[derive(Serialize)]
struct LiveDocumentCompactGroupMaterializationAvailabilityFenceV1 {
    expected_revision: u64,
    expected_digest_hex: String,
}

/// Owned result of one authenticated live-document operation.
pub(crate) struct LiveDocumentOperationReceiptV1 {
    response_json: String,
    mutation_result: Option<SessionOperationResultV1>,
    source_revision: u64,
    source_digest: [u8; 32],
}

impl LiveDocumentOperationReceiptV1 {
    #[must_use]
    pub(crate) fn response_json(&self) -> &str {
        &self.response_json
    }

    #[must_use]
    pub(crate) fn mutation_result(&self) -> Option<&SessionOperationResultV1> {
        self.mutation_result.as_ref()
    }

    #[must_use]
    pub(crate) const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    #[must_use]
    pub(crate) const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }
}

/// Frozen result of one read-only compact-group availability query.
pub(crate) struct LiveDocumentCompactGroupMaterializationAvailabilityReceiptV1 {
    response_json: String,
}

impl LiveDocumentCompactGroupMaterializationAvailabilityReceiptV1 {
    #[must_use]
    pub(crate) fn response_json(&self) -> &str {
        &self.response_json
    }
}

/// Query whether one exact durable compact-group address is currently eligible.
///
/// This performs the same preparation path as mutation but never commits the
/// prepared transition. The returned fact is advisory and remains fenced to
/// the exact current live session document.
pub(crate) fn query_live_compact_group_materialization_availability_v1(
    session: &mut DocumentSession,
    request_json: &str,
) -> Result<
    LiveDocumentCompactGroupMaterializationAvailabilityReceiptV1,
    OperationProtocolInputErrorV1,
> {
    let request: LiveDocumentCompactGroupMaterializationAvailabilityRequestV1 =
        serde_json::from_str(request_json).map_err(OperationProtocolInputErrorV1::InvalidJson)?;
    let availability = compact_group_materialization_availability(session, &request);
    let response = LiveDocumentCompactGroupMaterializationAvailabilityResponseV1 {
        schema: "ferrum-live-document-compact-group-materialization-availability-v1",
        document_fence: LiveDocumentCompactGroupMaterializationAvailabilityFenceV1 {
            expected_revision: request.expected_revision,
            expected_digest_hex: request.expected_digest_hex,
        },
        molecule_object_id: request.molecule_object_id,
        compact_group_object_id: request.compact_group_object_id,
        availability,
    };
    let response_json = serde_json::to_string(&response).map_err(internal_input_error)?;
    Ok(LiveDocumentCompactGroupMaterializationAvailabilityReceiptV1 { response_json })
}

fn compact_group_materialization_availability(
    session: &mut DocumentSession,
    request: &LiveDocumentCompactGroupMaterializationAvailabilityRequestV1,
) -> CompactGroupMaterializationAvailabilityV1 {
    let snapshot = match session.snapshot() {
        Ok(snapshot) => snapshot,
        Err(_) => return CompactGroupMaterializationAvailabilityV1::InvalidDocumentState,
    };
    if request.expected_revision != snapshot.revision()
        || request.expected_digest_hex != hex_digest(snapshot.digest())
    {
        return CompactGroupMaterializationAvailabilityV1::StaleDocumentFence;
    }
    let molecule_object_id = match DocumentObjectIdV1::parse(&request.molecule_object_id) {
        Ok(value) => value,
        Err(_) => return CompactGroupMaterializationAvailabilityV1::UnknownOrForeignTarget,
    };
    let compact_group_object_id = match DocumentObjectIdV1::parse(&request.compact_group_object_id)
    {
        Ok(value) => value,
        Err(_) => return CompactGroupMaterializationAvailabilityV1::UnknownOrForeignTarget,
    };
    let observation = match session.observe(snapshot.revision()) {
        Ok(observation) => observation,
        Err(_) => return CompactGroupMaterializationAvailabilityV1::InvalidDocumentState,
    };
    let target_is_owned_by_molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.id() == Some(&molecule_object_id))
        .is_some_and(|molecule| {
            molecule
                .compact_groups()
                .iter()
                .any(|compact_group| compact_group.id() == &compact_group_object_id)
        });
    if !target_is_owned_by_molecule {
        return CompactGroupMaterializationAvailabilityV1::UnknownOrForeignTarget;
    }
    let transition = SessionOperationTransitionRequestV1::new(
        request.expected_revision,
        SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(
            ferrum_document::DocumentCompactGroupMaterializationRequestV1::new(
                request.expected_revision,
                *snapshot.digest(),
                molecule_object_id,
                compact_group_object_id,
            ),
        )),
        TransitionAuthorizationV1::none(),
    );
    match session.prepare_session_operation_transition_v1(transition) {
        Ok(_) => CompactGroupMaterializationAvailabilityV1::Eligible,
        Err(error) => availability_from_prepare_error(error),
    }
}

fn availability_from_prepare_error(
    error: DocumentSessionError,
) -> CompactGroupMaterializationAvailabilityV1 {
    match error {
        DocumentSessionError::RevisionConflict { .. }
        | DocumentSessionError::Operation(SessionOperationError::CompactGroupMaterialization(
            DocumentCompactGroupMaterializationRefusalV1::StaleObservation
            | DocumentCompactGroupMaterializationRefusalV1::DigestMismatch,
        )) => CompactGroupMaterializationAvailabilityV1::StaleDocumentFence,
        DocumentSessionError::Operation(SessionOperationError::CompactGroupMaterialization(
            DocumentCompactGroupMaterializationRefusalV1::RendererAdmission,
        ))
        | DocumentSessionError::RendererAdmission => {
            CompactGroupMaterializationAvailabilityV1::RendererPreparationRefused
        }
        DocumentSessionError::Operation(SessionOperationError::CompactGroupMaterialization(
            DocumentCompactGroupMaterializationRefusalV1::InvalidTarget
            | DocumentCompactGroupMaterializationRefusalV1::UnsupportedRecipe
            | DocumentCompactGroupMaterializationRefusalV1::InvalidTopology
            | DocumentCompactGroupMaterializationRefusalV1::InvalidCandidate,
        )) => CompactGroupMaterializationAvailabilityV1::IneligibleTarget,
        _ => CompactGroupMaterializationAvailabilityV1::InvalidDocumentState,
    }
}

/// Execute one registered mutation against an existing live session.
pub(crate) fn execute_live_document_operation_v1(
    session: &mut DocumentSession,
    request_json: &str,
) -> Result<LiveDocumentOperationReceiptV1, OperationProtocolInputErrorV1> {
    if live_compact_group_operation_requested(request_json)? {
        return execute_live_compact_group_materialization_v1(session, request_json);
    }
    let request = match admit_operation_request_v1(request_json)? {
        OperationProtocolAdmissionV1::Request(request) => request,
        OperationProtocolAdmissionV1::Response(response) => {
            return receipt_from_envelope(session, response);
        }
    };
    let source = session.snapshot().map_err(internal_input_error)?;
    let kind = request.operation.kind();
    let request_id = request.request_id.clone();
    let (envelope, mutation_result) = match request.operation {
        OperationProtocolOperationV1::DocumentMoleculeHydrogenMaterialize(request) => {
            if request.document.cdml != source.cdml() {
                (
                    operation_error_response(
                        Some(request_id),
                        Some(kind),
                        ExecutionFailureV1::hydrogen_materialization_refusal(
                            DocumentMoleculeHydrogenMaterializationRefusalV1::DigestMismatch,
                        ),
                    ),
                    None,
                )
            } else {
                match execute_document_molecule_hydrogen_materialize_on_session(session, request) {
                    Ok((outcome, mutation_result)) => (
                        OperationProtocolEnvelopeV1::Success(OperationProtocolResponseV1 {
                            schema: ProtocolResponseSchemaV1::V1,
                            request_id,
                            outcome,
                        }),
                        mutation_result,
                    ),
                    Err(error) => (
                        operation_error_response(Some(request_id), Some(kind), error),
                        None,
                    ),
                }
            }
        }
        _ => (
            operation_error_response(
                Some(request_id),
                Some(kind),
                ExecutionFailureV1::invalid_request(
                    "operation is not registered for a live document session".to_owned(),
                ),
            ),
            None,
        ),
    };
    let envelope =
        admit_shared_response_budget_v1(envelope, kind, OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1);
    receipt_from_parts(
        source.revision(),
        *source.digest(),
        envelope,
        mutation_result,
    )
}

fn live_compact_group_operation_requested(
    request_json: &str,
) -> Result<bool, OperationProtocolInputErrorV1> {
    let value: serde_json::Value =
        serde_json::from_str(request_json).map_err(OperationProtocolInputErrorV1::InvalidJson)?;
    Ok(value
        .get("operation")
        .and_then(|operation| operation.get("kind"))
        .and_then(serde_json::Value::as_str)
        == Some("document.compact-group.materialize.v1"))
}

fn execute_live_compact_group_materialization_v1(
    session: &mut DocumentSession,
    request_json: &str,
) -> Result<LiveDocumentOperationReceiptV1, OperationProtocolInputErrorV1> {
    let request: LiveDocumentCompactGroupOperationEnvelopeV1 =
        serde_json::from_str(request_json).map_err(OperationProtocolInputErrorV1::InvalidJson)?;
    let LiveDocumentCompactGroupMaterializationRequestV1::Materialize {
        expected_revision,
        expected_digest_hex,
        molecule_object_id,
        compact_group_object_id,
    } = request.operation;
    let source = session.snapshot().map_err(internal_input_error)?;
    let kind = ProtocolOperationKindV1::DocumentCompactGroupMaterialize;
    let request_id = request.request_id;
    let (envelope, mutation_result) = match parse_live_compact_group_targets(
        expected_revision,
        &expected_digest_hex,
        &molecule_object_id,
        &compact_group_object_id,
    ) {
        Ok(document_request) => {
            match execute_document_compact_group_materialize_transition_on_session(
                session,
                document_request,
            ) {
                Ok(result) => {
                    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(materialization) =
                        result.outcome()
                    else {
                        return Err(internal_input_error(
                            "generic compact-group transition returned an unexpected outcome",
                        ));
                    };
                    let snapshot = session.snapshot().map_err(internal_input_error)?;
                    let outcome = compact_group_materialization_outcome(
                        expected_revision,
                        expected_digest_hex,
                        molecule_object_id,
                        compact_group_object_id,
                        materialization.focus_atom_id().as_str().to_owned(),
                        snapshot.cdml().to_owned(),
                        snapshot.revision(),
                        hex_digest(snapshot.digest()),
                    );
                    (
                        OperationProtocolEnvelopeV1::Success(OperationProtocolResponseV1 {
                            schema: ProtocolResponseSchemaV1::V1,
                            request_id,
                            outcome,
                        }),
                        Some(result),
                    )
                }
                Err(error) => (
                    operation_error_response(Some(request_id), Some(kind), error),
                    None,
                ),
            }
        }
        Err(error) => (
            operation_error_response(Some(request_id), Some(kind), error),
            None,
        ),
    };
    let envelope =
        admit_shared_response_budget_v1(envelope, kind, OPERATION_PROTOCOL_RESPONSE_UTF8_BYTES_V1);
    receipt_from_parts(
        source.revision(),
        *source.digest(),
        envelope,
        mutation_result,
    )
}

fn parse_live_compact_group_targets(
    expected_revision: u64,
    expected_digest_hex: &str,
    molecule_object_id: &str,
    compact_group_object_id: &str,
) -> Result<ferrum_document::DocumentCompactGroupMaterializationRequestV1, ExecutionFailureV1> {
    let molecule_object_id = DocumentObjectIdV1::parse(molecule_object_id).map_err(|_| {
        compact_refusal(
            ProtocolCompactGroupMaterializationCategoryV1::UnknownOrForeignTarget,
            ProtocolCompactGroupMaterializationRecoveryV1::CorrectTarget,
        )
    })?;
    let compact_group_object_id =
        DocumentObjectIdV1::parse(compact_group_object_id).map_err(|_| {
            compact_refusal(
                ProtocolCompactGroupMaterializationCategoryV1::UnknownOrForeignTarget,
                ProtocolCompactGroupMaterializationRecoveryV1::CorrectTarget,
            )
        })?;
    let expected_digest = parse_digest(expected_digest_hex)?;
    Ok(
        ferrum_document::DocumentCompactGroupMaterializationRequestV1::new(
            expected_revision,
            expected_digest,
            molecule_object_id,
            compact_group_object_id,
        ),
    )
}

fn receipt_from_envelope(
    session: &DocumentSession,
    envelope: OperationProtocolEnvelopeV1,
) -> Result<LiveDocumentOperationReceiptV1, OperationProtocolInputErrorV1> {
    let source = session.snapshot().map_err(internal_input_error)?;
    receipt_from_parts(source.revision(), *source.digest(), envelope, None)
}

fn receipt_from_parts(
    source_revision: u64,
    source_digest: [u8; 32],
    envelope: OperationProtocolEnvelopeV1,
    mutation_result: Option<SessionOperationResultV1>,
) -> Result<LiveDocumentOperationReceiptV1, OperationProtocolInputErrorV1> {
    let response_json = String::from_utf8(
        canonical_protocol_envelope_json_v1(&envelope).map_err(internal_input_error)?,
    )
    .map_err(internal_input_error)?;
    Ok(LiveDocumentOperationReceiptV1 {
        response_json,
        mutation_result,
        source_revision,
        source_digest,
    })
}

fn internal_input_error(error: impl std::fmt::Display) -> OperationProtocolInputErrorV1 {
    OperationProtocolInputErrorV1::InvalidJson(serde_json::Error::io(std::io::Error::other(
        error.to_string(),
    )))
}

#[cfg(test)]
mod tests {
    use ferrum_document::{
        MoleculeInsertionAtomV1, MoleculeInsertionV1, Point3V1, SessionOperation,
        SessionOperationTransitionRequestV1, SessionOperationV1, TransitionAuthorizationV1,
    };
    use serde_json::json;

    use super::*;

    fn live_session_with_atom(element: &str) -> DocumentSession {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let atom = MoleculeInsertionAtomV1::new(
            element,
            Point3V1::new(0.0, 0.0, 0.0).expect("finite position"),
            None,
            None,
            None,
        )
        .expect("oxygen atom");
        let insertion = MoleculeInsertionV1::new(vec![atom], Vec::new()).expect("molecule");
        let revision = session.snapshot().expect("session snapshot").revision();
        let mut prepared = session
            .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
                revision,
                SessionOperation::V1(SessionOperationV1::InsertMoleculeV1(insertion.into())),
                TransitionAuthorizationV1::none(),
            ))
            .expect("ordinary skeleton preparation");
        session
            .commit_session_operation_transition_v1(&mut prepared)
            .expect("ordinary skeleton installation");
        session
    }

    fn live_session() -> DocumentSession {
        live_session_with_atom("O")
    }

    fn renderer_excluded_live_session() -> DocumentSession {
        DocumentSession::load(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="o" name="O"><point x="0" y="0" z="0"/></atom></molecule><text id="t"><point x="1" y="2"/><font family="Telex"/><ftext><b>x</b></ftext></text></cdml>"#,
        )
        .expect("ordinary source document")
    }

    fn stateless_equivalent_live_session() -> DocumentSession {
        let authored = live_session();
        let source = authored.snapshot().expect("authored source snapshot");
        DocumentSession::load(source.cdml()).expect("reload request-owned source")
    }

    fn request(session: &DocumentSession) -> String {
        let snapshot = session.snapshot().expect("session snapshot");
        let observation = session
            .observe(snapshot.revision())
            .expect("session observation");
        let molecule = &observation.projection().molecules()[0];
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "live-materialization-test",
            "operation": {
                "kind": "document.molecule.hydrogen.materialize.v1",
                "document": {
                    "cdml": snapshot.cdml(),
                    "expected_revision": snapshot.revision(),
                    "expected_digest_hex": digest,
                },
                "molecule_id": molecule.id().expect("durable molecule").as_str(),
                "anchor_atom_id": molecule.atoms()[0].id().expect("durable atom").as_str(),
            },
        })
        .to_string()
    }

    fn compact_group_live_session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>")
            .expect("typed compact-group session")
    }

    fn two_compact_group_live_session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"first-anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"first-group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"first-outside\" start=\"first-anchor\" end=\"first-group\" type=\"n1\"/></molecule><molecule id=\"second\"><atom id=\"second-anchor\" name=\"C\"><point x=\"40\" y=\"0\"/></atom><compact-group id=\"second-group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"60\" y=\"0\"/></compact-group><bond id=\"second-outside\" start=\"second-anchor\" end=\"second-group\" type=\"n1\"/></molecule></cdml>")
            .expect("two typed compact-group session")
    }

    fn unsupported_compact_group_live_session() -> DocumentSession {
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"ethyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>")
            .expect("unsupported compact-group session")
    }

    fn compact_group_request(session: &DocumentSession) -> String {
        let snapshot = session.snapshot().expect("session snapshot");
        let observation = session
            .observe(snapshot.revision())
            .expect("session observation");
        let molecule = &observation.projection().molecules()[0];
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "live-compact-group-materialization-test",
            "operation": {
                "kind": "document.compact-group.materialize.v1",
                "expected_revision": snapshot.revision(),
                "expected_digest_hex": digest,
                "molecule_object_id": molecule.id().expect("durable molecule").as_str(),
                "compact_group_object_id": molecule.compact_groups()[0].id().as_str(),
            },
        })
        .to_string()
    }

    fn compact_group_availability_request(session: &DocumentSession) -> String {
        let snapshot = session.snapshot().expect("session snapshot");
        let observation = session
            .observe(snapshot.revision())
            .expect("session observation");
        let molecule = &observation.projection().molecules()[0];
        json!({
            "expected_revision": snapshot.revision(),
            "expected_digest_hex": hex_digest(snapshot.digest()),
            "molecule_object_id": molecule.id().expect("durable molecule").as_str(),
            "compact_group_object_id": molecule.compact_groups()[0].id().as_str(),
        })
        .to_string()
    }

    fn stateless_compact_group_request(session: &DocumentSession) -> String {
        let snapshot = session.snapshot().expect("session snapshot");
        let observation = session
            .observe(snapshot.revision())
            .expect("session observation");
        let molecule = &observation.projection().molecules()[0];
        let digest = snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "stateless-compact-group-materialization-test",
            "operation": {
                "kind": "document.compact-group.materialize.v1",
                "document": {
                    "cdml": snapshot.cdml(),
                    "expected_revision": snapshot.revision(),
                    "expected_digest_hex": digest,
                },
                "molecule_id": molecule.id().expect("durable molecule").as_str(),
                "compact_group_id": molecule.compact_groups()[0].id().as_str(),
            },
        })
        .to_string()
    }

    fn response_json(receipt: &LiveDocumentOperationReceiptV1) -> serde_json::Value {
        serde_json::from_str(receipt.response_json()).expect("public response JSON")
    }

    fn assert_durable_id(value: &serde_json::Value) {
        assert!(
            value
                .as_str()
                .is_some_and(|id| id.starts_with("ferrum-document-object-v1/")),
            "expected durable document object identifier, got {value}"
        );
    }

    fn assert_document_fence(value: &serde_json::Value, expected_revision: u64) {
        assert_eq!(value["expected_revision"], expected_revision);
        assert!(
            value["expected_digest_hex"].as_str().is_some_and(|digest| {
                digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            }),
            "expected SHA-256 document digest, got {}",
            value["expected_digest_hex"]
        );
    }

    #[test]
    fn live_materialization_returns_only_the_committed_session_result() {
        let mut session = stateless_equivalent_live_session();
        let before = session.snapshot().expect("before snapshot");
        let request = request(&session);
        let receipt = execute_live_document_operation_v1(&mut session, &request)
            .expect("live request admission");

        assert!(receipt.mutation_result().is_some());
        assert_eq!(receipt.source_revision(), before.revision());
        assert!(receipt.response_json().contains("\"status\":\"applied\""));
        assert_ne!(session.snapshot().expect("after snapshot"), before);
    }

    #[test]
    fn live_and_stateless_materialization_have_equivalent_public_outcomes() {
        let mut session = stateless_equivalent_live_session();
        let request = request(&session);
        let live = execute_live_document_operation_v1(&mut session, &request)
            .expect("live request admission");
        let stateless = execute_operation_v1(&request).expect("stateless request admission");
        let stateless_json = String::from_utf8(
            canonical_protocol_envelope_json_v1(&stateless).expect("canonical stateless response"),
        )
        .expect("UTF-8 canonical response");
        let live = response_json(&live);
        let stateless: serde_json::Value =
            serde_json::from_str(&stateless_json).expect("stateless public response JSON");

        for response in [&live, &stateless] {
            assert_eq!(response["schema"], "ferrum-operation-response-v1");
            assert_eq!(
                response["outcome"]["kind"],
                "document.molecule.hydrogen.materialize.v1"
            );
            assert_eq!(
                response["outcome"]["materialization"]["schema"],
                "ferrum-document-molecule-hydrogen-materialization-v1"
            );
            assert_eq!(response["outcome"]["materialization"]["status"], "applied");
            assert_eq!(
                response["outcome"]["materialization"]["added_hydrogen_count"],
                2
            );
            assert_durable_id(&response["outcome"]["materialization"]["molecule_id"]);
            assert_durable_id(&response["outcome"]["materialization"]["anchor_atom_id"]);
            assert_document_fence(&response["outcome"]["materialization"]["document_fence"], 0);
        }
        assert_eq!(
            live["outcome"]["materialization"]["source_revision"],
            stateless["outcome"]["materialization"]["source_revision"]
        );
        assert_eq!(
            live["outcome"]["materialization"]["source_digest_hex"],
            stateless["outcome"]["materialization"]["source_digest_hex"]
        );
    }

    #[test]
    fn live_and_stateless_compact_group_materialization_have_equivalent_public_outcomes() {
        let mut session = compact_group_live_session();
        let live_request = compact_group_request(&session);
        let live = execute_live_document_operation_v1(&mut session, &live_request)
            .expect("live compact-group request admission");
        let stateless_session = compact_group_live_session();
        let stateless = execute_operation_v1(&stateless_compact_group_request(&stateless_session))
            .expect("stateless compact-group request admission");
        let stateless_json = canonical_protocol_envelope_json_v1(&stateless)
            .expect("canonical stateless compact-group response");
        let live = response_json(&live);
        let stateless: serde_json::Value =
            serde_json::from_slice(&stateless_json).expect("stateless public response JSON");

        for response in [&live, &stateless] {
            assert_eq!(response["schema"], "ferrum-operation-response-v1");
            assert_eq!(
                response["outcome"]["kind"],
                "document.compact-group.materialize.v1"
            );
            let materialization = &response["outcome"]["materialization"];
            assert_eq!(
                materialization["schema"],
                "ferrum-document-compact-group-materialization-v1"
            );
            assert_durable_id(&materialization["molecule_id"]);
            assert_durable_id(&materialization["compact_group_id"]);
            assert_durable_id(&materialization["replacement_focus_atom_id"]);
            assert!(
                materialization["document"]
                    .as_str()
                    .is_some_and(|document| {
                        document.contains("<cdml") && !document.contains("<compact-group")
                    })
            );
        }
        assert_document_fence(&live["outcome"]["materialization"]["document_fence"], 1);
        assert_document_fence(
            &stateless["outcome"]["materialization"]["document_fence"],
            0,
        );
    }

    #[test]
    fn live_compact_group_materialization_refusal_preserves_the_live_session() {
        let mut session = compact_group_live_session();
        let before = session.snapshot().expect("source snapshot");
        let mut request: serde_json::Value =
            serde_json::from_str(&compact_group_request(&session)).expect("compact request");
        request["operation"]["compact_group_object_id"] = json!(
            "ferrum-document-object-v1/63646d6c2f636f6d706163742d67726f7570/source/6d697373696e672d67726f7570"
        );

        let receipt = execute_live_document_operation_v1(&mut session, &request.to_string())
            .expect("refused live compact-group request admission");
        let response: serde_json::Value =
            serde_json::from_str(receipt.response_json()).expect("public response JSON");

        assert!(receipt.mutation_result().is_none());
        assert_eq!(response["schema"], "ferrum-operation-error-v1");
        assert_eq!(session.snapshot().expect("after refusal"), before);
    }

    #[test]
    fn live_compact_group_availability_accepts_only_the_current_durable_address() {
        let mut session = compact_group_live_session();
        let before = session.snapshot().expect("source snapshot");
        let request = compact_group_availability_request(&session);
        let receipt =
            query_live_compact_group_materialization_availability_v1(&mut session, &request)
                .expect("availability request admission");

        assert!(
            receipt
                .response_json()
                .contains("\"availability\":\"eligible\"")
        );
        assert_eq!(session.snapshot().expect("unchanged snapshot"), before);
    }

    #[test]
    fn live_compact_group_availability_refuses_stale_or_source_style_addresses() {
        let mut session = compact_group_live_session();
        let mut stale: serde_json::Value =
            serde_json::from_str(&compact_group_availability_request(&session))
                .expect("availability request");
        stale["expected_revision"] = json!(stale["expected_revision"].as_u64().unwrap() + 1);
        let stale = query_live_compact_group_materialization_availability_v1(
            &mut session,
            &stale.to_string(),
        )
        .expect("stale availability receipt");
        let mut source_style: serde_json::Value =
            serde_json::from_str(&compact_group_availability_request(&session))
                .expect("availability request");
        source_style["compact_group_object_id"] = json!("group");
        let source_style = query_live_compact_group_materialization_availability_v1(
            &mut session,
            &source_style.to_string(),
        )
        .expect("source-style availability receipt");

        assert!(stale.response_json().contains("stale_document_fence"));
        assert!(
            source_style
                .response_json()
                .contains("unknown_or_foreign_target")
        );
    }

    #[test]
    fn live_compact_group_availability_refuses_a_group_owned_by_another_molecule() {
        let mut session = two_compact_group_live_session();
        let before = session.snapshot().expect("source snapshot");
        let snapshot = session.snapshot().expect("session snapshot");
        let observation = session
            .observe(snapshot.revision())
            .expect("session observation");
        let first = &observation.projection().molecules()[0];
        let second = &observation.projection().molecules()[1];
        let request = json!({
            "expected_revision": snapshot.revision(),
            "expected_digest_hex": hex_digest(snapshot.digest()),
            "molecule_object_id": first.id().expect("first durable molecule").as_str(),
            "compact_group_object_id": second.compact_groups()[0].id().as_str(),
        })
        .to_string();
        let receipt =
            query_live_compact_group_materialization_availability_v1(&mut session, &request)
                .expect("wrong-parent availability receipt");
        let response: serde_json::Value =
            serde_json::from_str(receipt.response_json()).expect("public response JSON");

        assert_eq!(
            response["schema"],
            "ferrum-live-document-compact-group-materialization-availability-v1"
        );
        assert_eq!(response["availability"], "unknown_or_foreign_target");
        assert_eq!(session.snapshot().expect("unchanged snapshot"), before);
    }

    #[test]
    fn live_compact_group_availability_refuses_an_unsupported_recipe() {
        let mut session = unsupported_compact_group_live_session();
        let request = compact_group_availability_request(&session);
        let receipt =
            query_live_compact_group_materialization_availability_v1(&mut session, &request)
                .expect("unsupported-recipe availability receipt");

        assert!(receipt.response_json().contains("ineligible_target"));
    }

    #[test]
    fn live_materialization_is_one_normal_history_transition_with_undo_and_redo() {
        let mut session = stateless_equivalent_live_session();
        let before = session.snapshot().expect("source snapshot");
        let operation_request = request(&session);
        let receipt = execute_live_document_operation_v1(&mut session, &operation_request)
            .expect("accepted live request");
        assert!(receipt.mutation_result().is_some());
        let applied = session.snapshot().expect("applied snapshot");
        assert!(session.can_undo());

        session
            .undo(applied.revision())
            .expect("undo materialization");
        let undone = session.snapshot().expect("undone snapshot");
        assert_eq!(undone.cdml(), before.cdml());
        assert!(session.can_redo());

        session
            .redo(undone.revision())
            .expect("redo materialization");
        assert_eq!(
            session.snapshot().expect("redone snapshot").cdml(),
            applied.cdml()
        );
    }

    #[test]
    fn renderer_preflight_unavailable_materialization_preserves_the_live_session() {
        let mut session = renderer_excluded_live_session();
        let before = session.snapshot().expect("source snapshot");
        let operation_request = request(&session);
        let receipt = execute_live_document_operation_v1(&mut session, &operation_request)
            .expect("renderer-preflight request admission");
        let response: serde_json::Value =
            serde_json::from_str(receipt.response_json()).expect("public response JSON");

        assert!(receipt.mutation_result().is_none());
        assert_eq!(
            response["outcome"]["materialization"]["status"],
            "unavailable"
        );
        assert_eq!(
            response["outcome"]["materialization"]["unavailable_reason"],
            "render_preparation"
        );
        assert_eq!(
            session
                .snapshot()
                .expect("after renderer-preflight refusal"),
            before
        );
    }

    #[test]
    fn live_materialization_stale_witness_and_no_op_keep_the_session_stable() {
        let mut session = stateless_equivalent_live_session();
        let request = request(&session);
        let request_value: serde_json::Value = serde_json::from_str(&request).expect("request");
        let before_stale = session.snapshot().expect("before stale request");
        for (field, stale_value) in [
            ("expected_revision", json!(before_stale.revision() + 1)),
            ("expected_digest_hex", json!("0".repeat(64))),
        ] {
            let mut stale = request_value.clone();
            stale["operation"][field] = stale_value;
            let stale_receipt =
                execute_live_document_operation_v1(&mut session, &stale.to_string())
                    .expect("stale request admission");
            assert!(stale_receipt.mutation_result().is_none());
            assert_eq!(
                session.snapshot().expect("after stale request"),
                before_stale
            );
        }

        execute_live_document_operation_v1(&mut session, &request).expect("first materialization");
        let before_replay = session.snapshot().expect("before replay request");
        let replay_receipt = execute_live_document_operation_v1(&mut session, &request)
            .expect("replayed request admission");
        assert!(replay_receipt.mutation_result().is_none());
        assert_eq!(
            session.snapshot().expect("after replay request"),
            before_replay
        );
    }
}
