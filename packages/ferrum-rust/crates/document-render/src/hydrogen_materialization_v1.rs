//! Renderer-admitted explicit hydrogen materialization.

use ferrum_document::{
    DocumentMoleculeHydrogenMaterializationRefusalV1,
    DocumentMoleculeHydrogenMaterializationRequestV1,
    DocumentMoleculeHydrogenMaterializationResultV1, DocumentSession, DocumentSessionError,
    PendingHydrogenMaterializationV1, SessionOperationResultV1,
    TypedDocumentError,
};
use thiserror::Error;

/// Closed outcome of renderer-safe explicit hydrogen materialization.
#[derive(Debug, Error)]
pub enum HydrogenMaterializationErrorV1 {
    #[error("materialization request was refused: {0}")]
    Refusal(#[from] DocumentMoleculeHydrogenMaterializationRefusalV1),
    #[error("candidate could not complete the normal document render plan")]
    RenderPreparation,
    #[error("prepared materialization receipt was already consumed")]
    Replayed,
}

/// Opaque renderer-admitted materialization transaction.
#[derive(Debug)]
pub struct PreparedHydrogenMaterializationV1 {
    pending: PendingHydrogenMaterializationV1,
}

/// Immutable renderer-safe result of one accepted materialization commit.
///
/// A changed commit carries the exact document-session result produced for the
/// installed candidate. A validated no-op has materialization facts but no
/// operation receipt because it has no new session state to install.
#[derive(Clone, Debug, PartialEq)]
pub struct CommittedHydrogenMaterializationV1 {
    materialization: DocumentMoleculeHydrogenMaterializationResultV1,
    operation_result: Option<SessionOperationResultV1>,
}

impl CommittedHydrogenMaterializationV1 {
    /// Return the public materialization outcome.
    #[must_use]
    pub fn materialization(&self) -> &DocumentMoleculeHydrogenMaterializationResultV1 {
        &self.materialization
    }

    /// Return the authoritative result for an installed live-session mutation.
    #[must_use]
    pub fn operation_result(&self) -> Option<&SessionOperationResultV1> {
        self.operation_result.as_ref()
    }
}

pub fn prepare_hydrogen_materialization_v1(
    session: &mut DocumentSession,
    request: &DocumentMoleculeHydrogenMaterializationRequestV1,
) -> Result<PreparedHydrogenMaterializationV1, HydrogenMaterializationErrorV1> {
    let pending = session
        .prepare_materialize_molecule_hydrogens_v1(request)
        .map_err(|error| match error {
            DocumentMoleculeHydrogenMaterializationRefusalV1::RendererAdmission => {
                HydrogenMaterializationErrorV1::RenderPreparation
            }
            other => HydrogenMaterializationErrorV1::Refusal(other),
        })?;
    Ok(PreparedHydrogenMaterializationV1 { pending })
}

pub fn commit_hydrogen_materialization_v1(
    session: &mut DocumentSession,
    prepared: &mut PreparedHydrogenMaterializationV1,
) -> Result<CommittedHydrogenMaterializationV1, HydrogenMaterializationErrorV1> {
    if prepared.pending.is_consumed_v1() {
        return Err(HydrogenMaterializationErrorV1::Replayed);
    }
    match session
        .commit_materialize_molecule_hydrogens_with_operation_result_v1(&mut prepared.pending)
    {
        Ok((materialization, operation_result)) => Ok(CommittedHydrogenMaterializationV1 {
            materialization,
            operation_result,
        }),
        Err(DocumentMoleculeHydrogenMaterializationRefusalV1::RendererAdmission) => {
            Err(HydrogenMaterializationErrorV1::RenderPreparation)
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrum_document::{
        DocumentAtomOxidationObservationRequestV1, DocumentAtomOxidationObservationV1,
        DocumentAtomOxidationResultV1, MoleculeInsertionAtomV1, MoleculeInsertionV1, Point3V1,
    };

    fn oxygen_session() -> DocumentSession {
        let mut session = DocumentSession::create_empty_document_v1().expect("empty document");
        let atom = MoleculeInsertionAtomV1::new(
            "O",
            Point3V1::new(0.0, 0.0, 0.0).expect("finite coordinate"),
            None,
            None,
            None,
        )
        .expect("oxygen atom");
        let insertion = MoleculeInsertionV1::new(vec![atom], Vec::new()).expect("molecule");
        let revision = session.snapshot().expect("snapshot").revision();
        let mut pending = session
            .prepare_admitted_molecule_insertion_v1(revision, &insertion)
            .expect("candidate");
        session
            .commit_admitted_molecule_insertion_v1(revision, &mut pending)
            .expect("commit");
        session
    }

    fn request(session: &DocumentSession) -> DocumentMoleculeHydrogenMaterializationRequestV1 {
        let revision = session.snapshot().expect("snapshot").revision();
        let observation = session.observe(revision).expect("observation");
        let molecule = &observation.projection().molecules()[0];
        DocumentMoleculeHydrogenMaterializationRequestV1::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            molecule.id().expect("molecule identity").clone(),
            molecule.atoms()[0].id().expect("atom identity").clone(),
        )
    }

    #[test]
    fn renderer_admitted_materialization_is_oxidation_observable_and_undoable() {
        let mut session = oxygen_session();
        let before = session.snapshot().expect("before");
        let request = request(&session);
        let mut prepared = prepare_hydrogen_materialization_v1(&mut session, &request)
            .expect("renderer-admitted candidate");
        let result =
            commit_hydrogen_materialization_v1(&mut session, &mut prepared).expect("single commit");
        assert!(result.materialization().changed());
        let after = session.snapshot().expect("after");
        let installed = result
            .operation_result()
            .expect("changed commit has an installable operation receipt");
        assert_eq!(installed.observation().snapshot(), &after);
        let observation = session.observe(after.revision()).expect("observation");
        let molecule = &observation.projection().molecules()[0];
        let oxidation = DocumentAtomOxidationObservationRequestV1::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            molecule.id().expect("molecule identity").clone(),
            molecule.atoms()[0].id().expect("atom identity").clone(),
        );
        assert_eq!(
            session.observe_atom_oxidation_v1(&oxidation),
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Accepted {
                    oxidation_number: -2
                }
            ))
        );
        let undone = session.undo(after.revision()).expect("undo");
        assert_eq!(undone.observation().snapshot().cdml(), before.cdml());
    }

    #[test]
    fn renderer_admitted_materialization_refuses_foreign_redemption_and_replay() {
        let mut session = oxygen_session();
        let request = request(&session);
        let mut prepared = prepare_hydrogen_materialization_v1(&mut session, &request)
            .expect("renderer-admitted candidate");
        let mut foreign = DocumentSession::load(session.snapshot().expect("owner snapshot").cdml())
            .expect("foreign session");
        let foreign_before = foreign.snapshot().expect("foreign before redemption");
        assert!(matches!(
            commit_hydrogen_materialization_v1(&mut foreign, &mut prepared),
            Err(HydrogenMaterializationErrorV1::Refusal(
                DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation
            ))
        ));
        assert_eq!(
            foreign.snapshot().expect("foreign after refusal"),
            foreign_before
        );

        commit_hydrogen_materialization_v1(&mut session, &mut prepared).expect("first commit");
        let after_first_commit = session.snapshot().expect("first committed snapshot");

        assert!(matches!(
            commit_hydrogen_materialization_v1(&mut session, &mut prepared),
            Err(HydrogenMaterializationErrorV1::Replayed)
        ));
        assert_eq!(
            session.snapshot().expect("after replay refusal"),
            after_first_commit
        );
    }

    #[test]
    fn renderer_bridge_issues_an_authoritative_receipt_only_for_a_changed_commit() {
        let mut session = oxygen_session();
        let materialization_request = request(&session);
        let before = session.snapshot().expect("before preparation");
        let mut prepared =
            prepare_hydrogen_materialization_v1(&mut session, &materialization_request)
                .expect("renderer-admitted candidate");

        assert_eq!(session.snapshot().expect("prepared source"), before);
        let changed = commit_hydrogen_materialization_v1(&mut session, &mut prepared)
            .expect("changed commit");
        assert!(changed.materialization().changed());
        assert_eq!(
            changed
                .operation_result()
                .expect("changed commit receipt")
                .observation()
                .snapshot(),
            &session.snapshot().expect("committed source")
        );

        let request = request(&session);
        let mut prepared = prepare_hydrogen_materialization_v1(&mut session, &request)
            .expect("validated no-op candidate");
        let no_op = commit_hydrogen_materialization_v1(&mut session, &mut prepared)
            .expect("validated no-op");
        assert!(!no_op.materialization().changed());
        assert!(no_op.operation_result().is_none());
    }

    #[test]
    fn renderer_admitted_materialization_receipt_refuses_after_session_advance() {
        let mut session = oxygen_session();
        let request = request(&session);
        let mut prepared = prepare_hydrogen_materialization_v1(&mut session, &request)
            .expect("renderer-admitted candidate");
        let current_revision = session.snapshot().expect("current snapshot").revision();
        session
            .undo(current_revision)
            .expect("supported session transition");
        let after_session_advance = session.snapshot().expect("advanced snapshot");

        assert!(matches!(
            commit_hydrogen_materialization_v1(&mut session, &mut prepared),
            Err(HydrogenMaterializationErrorV1::Refusal(
                DocumentMoleculeHydrogenMaterializationRefusalV1::StaleObservation
            ))
        ));
        assert_eq!(
            session.snapshot().expect("after stale receipt refusal"),
            after_session_advance
        );
    }

    #[test]
    fn unsupported_text_face_is_refused_before_materialization_session_exists() {
        let source = r#"<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="o" name="O"><point x="0" y="0" z="0"/></atom></molecule><plus id="bad"><point x="1" y="2"/><font family="Arial"/></plus></cdml>"#;
        assert!(matches!(
            DocumentSession::load(source),
            Err(DocumentSessionError::Load(TypedDocumentError::UnsupportedTextFace {
                root_id,
                family,
            })) if root_id == "bad" && family == "Arial"
        ));
    }
}
