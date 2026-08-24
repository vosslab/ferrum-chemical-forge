//! Session-owned execution for the closed live-document operation subset.
//!
//! The public protocol remains stateless. This adapter authenticates a normal
//! V1 request against an already-owned session and returns the same protocol
//! envelope together with an installable result only when that session changed.

use ferrum_document::{
    DocumentMoleculeHydrogenMaterializationRefusalV1, DocumentSession, SessionOperationResultV1,
};

use super::document_hydrogen_materialization_v1::execute_document_molecule_hydrogen_materialize_on_session;
use super::execution::{
    ExecutionFailureV1, OperationProtocolAdmissionV1, admit_operation_request_v1,
    admit_shared_response_budget_v1, canonical_protocol_envelope_json_v1, operation_error_response,
};
use super::*;

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

/// Execute one registered mutation against an existing live session.
pub(crate) fn execute_live_document_operation_v1(
    session: &mut DocumentSession,
    request_json: &str,
) -> Result<LiveDocumentOperationReceiptV1, OperationProtocolInputErrorV1> {
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
    let envelope = admit_shared_response_budget_v1(
        envelope,
        kind,
        DOCUMENT_SMARTS_QUERY_RESPONSE_UTF8_BYTES_V1,
    );
    receipt_from_parts(
        source.revision(),
        *source.digest(),
        envelope,
        mutation_result,
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
    use ferrum_document::{MoleculeInsertionAtomV1, MoleculeInsertionV1, Point3V1};
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
            .prepare_admitted_molecule_insertion_v1(revision, &insertion)
            .expect("ordinary skeleton preparation");
        session
            .commit_admitted_molecule_insertion_v1(revision, &mut prepared)
            .expect("ordinary skeleton installation");
        session
    }

    fn live_session() -> DocumentSession {
        live_session_with_atom("O")
    }

    fn renderer_excluded_live_session() -> DocumentSession {
        DocumentSession::load(
            r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="o" name="O"><point x="0" y="0" z="0"/></atom></molecule><plus id="p"><point x="1" y="2"/><font family="Arial"/></plus></cdml>"#,
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
    fn live_and_stateless_materialization_return_the_same_canonical_protocol_response() {
        let mut session = stateless_equivalent_live_session();
        let request = request(&session);
        let live = execute_live_document_operation_v1(&mut session, &request)
            .expect("live request admission");
        let stateless = execute_operation_v1(&request).expect("stateless request admission");
        let stateless_json = String::from_utf8(
            canonical_protocol_envelope_json_v1(&stateless).expect("canonical stateless response"),
        )
        .expect("UTF-8 canonical response");

        assert_eq!(live.response_json(), stateless_json);
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
            stale["operation"]["document"][field] = stale_value;
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
