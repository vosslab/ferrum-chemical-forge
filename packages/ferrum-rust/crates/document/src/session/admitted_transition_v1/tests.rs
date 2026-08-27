//! Behavioral contracts for the document-owned admitted transition core.

use super::*;
use crate::{DrawingStandardPatchV1, DrawingStandardPropertyChangeV1, SessionOperationV1};

fn changed_operation(line_width: f64) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetDrawingStandard {
        patch: DrawingStandardPatchV1::new(vec![DrawingStandardPropertyChangeV1::LineWidth(
            line_width,
        )])
        .expect("test change is valid"),
    })
}

fn no_change_operation() -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetDrawingStandard {
        patch: DrawingStandardPatchV1::new(Vec::new()).expect("empty patch is valid"),
    })
}

fn direct_bond_operation(session: &DocumentSession) -> SessionOperation {
    let snapshot = session.snapshot().expect("snapshot");
    let fence = crate::DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());
    let point = |x, y| crate::DirectBondPoint2V1::new(x, y).expect("finite point");
    SessionOperation::V1(SessionOperationV1::CreateDirectBondV1(
        crate::CreateDirectBondV1::new(
            fence,
            crate::DirectBondEndpointIntent::NewAtomAt {
                raw_point: point(0.0, 0.0),
            },
            crate::DirectBondEndpointIntent::NewAtomAt {
                raw_point: point(40.0, 0.0),
            },
            crate::DocumentBondPresentationV1::Normal(crate::DocumentBondOrderV1::Single),
            "C".to_owned(),
            crate::DirectBondSnapPolicyV1::free(),
        )
        .expect("direct-bond request"),
    ))
}

const REACTION_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\">",
    "<molecule id=\"reactant\"><atom id=\"reactant-atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
    "<molecule id=\"product\"><atom id=\"product-atom\" name=\"O\"><point x=\"20\" y=\"0\"/></atom></molecule>",
    "<molecule id=\"replacement\"><atom id=\"replacement-atom\" name=\"N\"><point x=\"40\" y=\"0\"/></atom></molecule>",
    "<arrow id=\"reaction-arrow\"><point x=\"0\" y=\"10\"/><point x=\"40\" y=\"10\"/></arrow>",
    "</cdml>"
);

fn reaction_members(product: &str) -> Vec<(crate::DirectReactionRoleV1, String)> {
    vec![
        (crate::DirectReactionRoleV1::Reactant, "reactant".to_owned()),
        (crate::DirectReactionRoleV1::Product, product.to_owned()),
        (
            crate::DirectReactionRoleV1::Arrow,
            "reaction-arrow".to_owned(),
        ),
    ]
}

fn prepare_generic_reaction(
    session: &mut DocumentSession,
    operation: SessionOperation,
) -> PreparedSessionTransitionV1 {
    let revision = session.snapshot().expect("snapshot").revision();
    let capability = session.authoring_capability_issuer_v1().issue();
    session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            revision,
            operation,
            TransitionAuthorizationV1::authoring_capability(capability),
        ))
        .expect("generic reaction transition prepares")
}

#[test]
fn generic_reaction_outcomes_publish_only_after_successful_commit() {
    let mut session = DocumentSession::load(REACTION_SOURCE).expect("reaction fixture loads");
    let mut create = prepare_generic_reaction(
        &mut session,
        SessionOperation::V1(SessionOperationV1::CreateReactionV1(
            crate::CreateReactionV1::new(reaction_members("product")).expect("create request"),
        )),
    );
    assert!(
        !session
            .snapshot()
            .expect("precommit snapshot")
            .cdml()
            .contains("<reaction")
    );

    let created = session
        .commit_session_operation_transition_v1(&mut create)
        .expect("create commits");
    let reaction_document_object_id = match created.outcome() {
        SessionOperationOutcomeV1::ReactionCreatedV1(outcome) => {
            outcome.reaction_document_object_id().clone()
        }
        outcome => panic!("expected created reaction outcome, got {outcome:?}"),
    };

    let mut replace = prepare_generic_reaction(
        &mut session,
        SessionOperation::V1(SessionOperationV1::ReplaceReactionMembersV1(
            crate::ReplaceReactionMembersV1::new(
                "rxn-1".to_owned(),
                reaction_members("replacement"),
            )
            .expect("replacement request"),
        )),
    );
    let replaced = session
        .commit_session_operation_transition_v1(&mut replace)
        .expect("membership replacement commits");
    assert!(matches!(
        replaced.outcome(),
        SessionOperationOutcomeV1::ReactionMembershipReplacedV1(outcome)
            if outcome.reaction_document_object_id() == &reaction_document_object_id
    ));

    let mut delete = prepare_generic_reaction(
        &mut session,
        SessionOperation::V1(SessionOperationV1::DeleteReactionV1(
            crate::DeleteReactionV1::new("rxn-1".to_owned()).expect("delete request"),
        )),
    );
    let deleted = session
        .commit_session_operation_transition_v1(&mut delete)
        .expect("definition deletion commits");
    assert!(matches!(
        deleted.outcome(),
        SessionOperationOutcomeV1::ReactionDefinitionDeletedV1(outcome)
            if outcome.reaction_document_object_id() == &reaction_document_object_id
    ));
    assert!(
        !deleted
            .observation()
            .snapshot()
            .cdml()
            .contains("<reaction")
    );
}

#[test]
fn refused_generic_reactions_preserve_the_authoritative_observation() {
    let mut session = DocumentSession::load(REACTION_SOURCE).expect("reaction fixture loads");
    let before = session.snapshot().expect("snapshot");
    let missing_members = SessionOperation::V1(SessionOperationV1::CreateReactionV1(
        crate::CreateReactionV1::new(reaction_members("missing")).expect("request shape is valid"),
    ));
    let capability = session.authoring_capability_issuer_v1().issue();
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                before.revision(),
                missing_members,
                TransitionAuthorizationV1::authoring_capability(capability),
            )
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Reaction(crate::ReactionOperationRefusalV1::MissingMember)
        ))
    ));
    assert_eq!(
        session.snapshot().expect("failed preparation snapshot"),
        before
    );

    let mut stale = prepare_generic_reaction(
        &mut session,
        SessionOperation::V1(SessionOperationV1::CreateReactionV1(
            crate::CreateReactionV1::new(reaction_members("product")).expect("create request"),
        )),
    );
    session
        .apply_document_operation_v1(before.revision(), changed_operation(2.0))
        .expect("independent transition commits");
    let after_independent = session.snapshot().expect("independent snapshot");
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut stale),
        Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
    );
    assert_eq!(
        session.snapshot().expect("stale refusal snapshot"),
        after_independent
    );
}

#[test]
fn direct_bond_requires_generic_authorization_and_consumes_it_on_generic_commit() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let before = session.snapshot().expect("snapshot");
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                before.revision(),
                direct_bond_operation(&session),
                TransitionAuthorizationV1::None,
            )
        ),
        Err(DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::AuthoringCapabilityRequired
        ))
    ));
    assert_eq!(session.snapshot().expect("unchanged snapshot"), before);

    let foreign = DocumentSession::create_empty_document_v1().expect("foreign session");
    let foreign_capability = foreign.authoring_capability_issuer_v1().issue();
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                before.revision(),
                direct_bond_operation(&session),
                TransitionAuthorizationV1::authoring_capability(foreign_capability),
            )
        ),
        Err(DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::ForeignSession
        ))
    ));

    let issuer = session.authoring_capability_issuer_v1();
    let consumed_capability = issuer.issue();
    consumed_capability
        .claim_for_commit(&issuer)
        .expect("claim")
        .consume();
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                before.revision(),
                direct_bond_operation(&session),
                TransitionAuthorizationV1::authoring_capability(consumed_capability),
            )
        ),
        Err(DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::Consumed
        ))
    ));

    let capability = issuer.issue();
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            before.revision(),
            direct_bond_operation(&session),
            TransitionAuthorizationV1::authoring_capability(capability.clone()),
        ))
        .expect("generic direct-bond transition prepares");
    let presentation = prepared
        .presentation_v1()
        .expect("prepared transition exposes inert precommit paint");
    assert!(
        !presentation
            .precommit_overlay()
            .expect("direct-bond transition selects its accepted paint")
            .primitives()
            .is_empty()
    );
    let committed = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("generic direct-bond transition commits");
    let SessionOperationOutcomeV1::DirectBondV1(outcome) = committed.outcome() else {
        panic!("generic direct-bond transition returns its direct-bond receipt");
    };
    let document = session.current_document_v1();
    assert!(matches!(
        document.resolve_document_object_id(outcome.bond_document_object_id()),
        Ok(Some(_))
    ));
    assert!(matches!(
        document.resolve_document_object_id(outcome.end_atom_document_object_id()),
        Ok(Some(_))
    ));
    if let Some(atom) = outcome.second_created_atom_document_object_id() {
        assert!(matches!(
            document.resolve_document_object_id(atom),
            Ok(Some(_))
        ));
    }
    assert_eq!(
        prepared.presentation_v1(),
        Err(PreparedSessionTransitionPresentationRefusalV1::Consumed)
    );
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                session.snapshot().expect("committed snapshot").revision(),
                direct_bond_operation(&session),
                TransitionAuthorizationV1::authoring_capability(capability),
            )
        ),
        Err(DocumentSessionError::TransitionAuthorization(
            TransitionAuthorizationRefusalV1::Consumed
        ))
    ));
}

fn next_reserved_atom_identifier(session: &DocumentSession) -> String {
    let (identifier, _effects) = session
        .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
            sequences.reserve_atom(indexed)
        })
        .expect("atom identifier reserves");
    identifier.as_str().to_owned()
}

fn changed_state(session: &DocumentSession, line_width: f64) -> (u64, [u8; 32], RevisionState) {
    let current = session.admitted_history.current();
    let source_revision = current.revision();
    let source_digest = *current.digest();
    let Candidate::Changed(document) = changed_operation(line_width)
        .prepare(current.document(), source_revision, &source_digest)
        .expect("changed operation prepares")
    else {
        panic!("test operation must change the document");
    };
    let revision = current.next_revision().expect("revision advances");
    let state = RevisionState::from_document(revision, *document).expect("candidate state");
    (source_revision, source_digest, state)
}

fn prepared_generated_id_transition(
    session: &mut DocumentSession,
    line_width: f64,
) -> (String, PreparedSessionTransitionV1) {
    let (identifier, effects) = session
        .reserve_generated_ids_for_transition_v1(|sequences, indexed| {
            sequences.reserve_atom(indexed)
        })
        .expect("atom identifier reserves");
    let (source_revision, source_digest, state) = changed_state(session, line_width);
    let prepared = session
        .prepare_changed_session_transition_v1(source_revision, source_digest, state, effects)
        .expect("admitted transition prepares");
    (identifier.as_str().to_owned(), prepared)
}

#[test]
fn changed_transition_is_renderer_admitted_atomic_and_one_use() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("changed transition is admitted");
    let _result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("admitted transition commits");
    assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
    assert!(prepared.is_consumed_v1());
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::Consumed)
    );
}

#[test]
fn no_change_transition_is_history_free_and_one_use() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            no_change_operation(),
            TransitionAuthorizationV1::None,
        ))
        .expect("no-change transition prepares");
    let _result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("no-change transition completes");
    assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::Consumed)
    );
}

#[test]
fn prepared_transition_debug_reports_only_lifecycle_state() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><metadata>",
        "debug-private-candidate-content",
        "</metadata></cdml>"
    );
    let mut session = DocumentSession::load(source).expect("private source loads");
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("changed transition prepares");

    let pending_debug = format!("{prepared:?}");
    assert!(pending_debug.contains("PreparedSessionTransitionV1"));
    assert!(pending_debug.contains("lifecycle: \"pending\""));
    assert!(!pending_debug.contains("debug-private-candidate-content"));
    assert!(!pending_debug.contains("source_digest"));
    assert!(!pending_debug.contains("renderer_admission"));

    session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("transition commits");
    assert!(format!("{prepared:?}").contains("lifecycle: \"consumed\""));
}

#[test]
fn presentation_extraction_keeps_a_live_changed_transition_redeemable() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("changed transition prepares");
    let presentation = prepared
        .presentation_v1()
        .expect("live transition exposes copied presentation");

    assert!(presentation.precommit_overlay().is_none());
    session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("presentation extraction leaves transition redeemable");
    assert_eq!(
        prepared.presentation_v1(),
        Err(PreparedSessionTransitionPresentationRefusalV1::Consumed)
    );
}

#[test]
fn no_change_presentation_has_no_paint_contract_and_is_consumed_when_cancelled() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let mut prepared = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            no_change_operation(),
            TransitionAuthorizationV1::None,
        ))
        .expect("no-change transition prepares");
    let presentation = prepared
        .presentation_v1()
        .expect("live no-change transition exposes presentation");

    assert!(presentation.precommit_overlay().is_none());
    session
        .cancel_session_operation_transition_v1(&mut prepared)
        .expect("transition cancels");
    assert_eq!(
        prepared.presentation_v1(),
        Err(PreparedSessionTransitionPresentationRefusalV1::Consumed)
    );
}

#[test]
fn prepared_siblings_share_the_history_append_slot_without_mutating_history() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let mut first = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("first sibling prepares");
    let mut second = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(3.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("second sibling prepares against the same preallocated history slot");
    assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    assert!(session.admitted_history.undo_target().is_none());

    session
        .commit_session_operation_transition_v1(&mut first)
        .expect("first sibling commits");
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut second),
        Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
    );
    assert!(!second.is_consumed_v1());
}

#[test]
fn cancellation_is_semantic_and_preserves_history_resources() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let before = session.snapshot().expect("snapshot");
    let (identifier, mut prepared) = prepared_generated_id_transition(&mut session, 2.0);

    session
        .cancel_session_operation_transition_v1(&mut prepared)
        .expect("owner cancels transition");

    assert_eq!(session.snapshot().expect("snapshot"), before);
    assert!(session.admitted_history.undo_target().is_none());
    assert_eq!(next_reserved_atom_identifier(&session), identifier);
    assert!(prepared.is_consumed_v1());
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::Consumed)
    );

    let mut replacement = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("fresh transition prepares after cancellation");
    session
        .commit_session_operation_transition_v1(&mut replacement)
        .expect("replacement renderer-admitted transition commits");
    assert_eq!(session.snapshot().expect("snapshot").revision(), 1);
    assert_eq!(
        session.cancel_session_operation_transition_v1(&mut replacement),
        Err(AdmittedSessionTransitionRefusalV1::Consumed)
    );
}

#[test]
fn dropped_preparation_leaves_generated_ids_tentative_until_replacement_commits() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let (identifier, abandoned) = prepared_generated_id_transition(&mut session, 2.0);
    assert_eq!(identifier, "ferrum-atom-v1-0");
    drop(abandoned);

    assert_eq!(next_reserved_atom_identifier(&session), identifier);
    let (replacement_identifier, mut replacement) =
        prepared_generated_id_transition(&mut session, 3.0);
    assert_eq!(replacement_identifier, identifier);
    assert_eq!(next_reserved_atom_identifier(&session), identifier);

    session
        .commit_session_operation_transition_v1(&mut replacement)
        .expect("replacement commits");
    assert_eq!(next_reserved_atom_identifier(&session), "ferrum-atom-v1-1");
}

#[test]
fn foreign_cancellation_cannot_invalidate_the_owner_pending_transition() {
    let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
    let mut other = DocumentSession::create_empty_document_v1().expect("other session");
    let mut prepared = owner
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("owner transition prepares");

    assert_eq!(
        other.cancel_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
    );
    assert!(!prepared.is_consumed_v1());
    owner
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("foreign cancellation refusal leaves owner transition committable");
}

#[test]
fn consumed_transition_cannot_expose_or_bypass_renderer_admission() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let mut consumed = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("renderer-admitted transition prepares");
    assert!(
        consumed
            .presentation_v1()
            .expect("live transition presents inert display facts")
            .precommit_overlay()
            .is_none()
    );

    session
        .cancel_session_operation_transition_v1(&mut consumed)
        .expect("transition cancels");
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut consumed),
        Err(AdmittedSessionTransitionRefusalV1::Consumed)
    );

    let mut replacement = session
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("only a fresh renderer-admitted transition can replace it");
    session
        .commit_session_operation_transition_v1(&mut replacement)
        .expect("fresh renderer-admitted transition commits");
}

#[test]
fn generated_ids_install_only_after_successful_renderer_admitted_redemption() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let (identifier, mut prepared) = prepared_generated_id_transition(&mut session, 2.0);
    assert_eq!(identifier, "ferrum-atom-v1-0");
    assert_eq!(next_reserved_atom_identifier(&session), identifier);
    session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("admitted transition commits");
    assert_eq!(next_reserved_atom_identifier(&session), "ferrum-atom-v1-1");
    assert_eq!(
        session.commit_session_operation_transition_v1(&mut prepared),
        Err(AdmittedSessionTransitionRefusalV1::Consumed)
    );
    assert_eq!(next_reserved_atom_identifier(&session), "ferrum-atom-v1-1");
}

#[test]
fn generated_ids_remain_tentative_after_foreign_stale_or_complete_render_refusal() {
    let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
    let mut other = DocumentSession::create_empty_document_v1().expect("other session");
    let (identifier, mut foreign) = prepared_generated_id_transition(&mut owner, 2.0);
    assert_eq!(
        other.commit_session_operation_transition_v1(&mut foreign),
        Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
    );
    assert_eq!(next_reserved_atom_identifier(&owner), identifier);

    let (_identifier, mut stale) = prepared_generated_id_transition(&mut owner, 3.0);
    owner
        .apply_document_operation_v1(0, changed_operation(4.0))
        .expect("independent transition advances source fence");
    assert_eq!(
        owner.commit_session_operation_transition_v1(&mut stale),
        Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
    );
    assert_eq!(next_reserved_atom_identifier(&owner), identifier);

    let mut renderer_session =
        DocumentSession::create_empty_document_v1().expect("renderer session");
    let (identifier, mut renderer_refused) =
        prepared_generated_id_transition(&mut renderer_session, 2.0);
    let PreparedSessionTransitionKindV1::Changed(changed) = &mut renderer_refused.kind else {
        panic!("test transition must be changed");
    };
    changed.observation = renderer_session
        .document_observation()
        .expect("current observation");
    assert_eq!(
        renderer_session.commit_session_operation_transition_v1(&mut renderer_refused),
        Err(AdmittedSessionTransitionRefusalV1::RendererAdmission)
    );
    assert_eq!(
        renderer_session
            .snapshot()
            .expect("snapshot remains unchanged")
            .revision(),
        0
    );
    assert!(!renderer_refused.is_consumed_v1());
    assert_eq!(next_reserved_atom_identifier(&renderer_session), identifier);
}

#[test]
fn admitted_history_navigation_preserves_monotonic_revisions() {
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let changed = session
        .apply_document_operation_v1(0, changed_operation(2.0))
        .expect("changed transition commits");
    let undone = session
        .undo(changed.observation().snapshot().revision())
        .expect("renderer-admitted undo commits");
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("renderer-admitted redo commits");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(undone.observation().snapshot().revision(), 2);
    assert_eq!(redone.observation().snapshot().revision(), 3);
}

#[test]
fn foreign_or_stale_refusal_preserves_the_owner_pending_transition() {
    let mut owner = DocumentSession::create_empty_document_v1().expect("owner session");
    let mut other = DocumentSession::create_empty_document_v1().expect("other session");
    let mut foreign = owner
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            0,
            changed_operation(2.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("owner transition prepares");
    assert_eq!(
        other.commit_session_operation_transition_v1(&mut foreign),
        Err(AdmittedSessionTransitionRefusalV1::ForeignSession)
    );
    assert!(!foreign.is_consumed_v1());
    owner
        .commit_session_operation_transition_v1(&mut foreign)
        .expect("foreign refusal left owner transition redeemable");

    let mut stale = owner
        .prepare_session_operation_transition_v1(crate::SessionOperationTransitionRequestV1::new(
            1,
            changed_operation(3.0),
            TransitionAuthorizationV1::None,
        ))
        .expect("stale transition prepares");
    owner
        .apply_document_operation_v1(1, changed_operation(4.0))
        .expect("independent transition advances source fence");
    assert_eq!(
        owner.commit_session_operation_transition_v1(&mut stale),
        Err(AdmittedSessionTransitionRefusalV1::StaleSnapshot)
    );
    assert!(!stale.is_consumed_v1());
}

#[test]
fn renderer_refusal_leaves_the_source_session_unchanged() {
    let source = concat!(
        "<c:cdml xmlns:c=\"urn:ferrum:cdml\" xmlns:v=\"urn:vendor\">",
        "<c:info/><v:before/><c:metadata/>",
        "<c:standard line_width=\"1\" font_size=\"12\" font_family=\"No Such Face\" ",
        "line_color=\"#000000\" area_color=\"\" paper_type=\"Letter\" v:keep=\"yes\">",
        "<c:bond width=\"6\" wedge-width=\"5\" double-ratio=\"0.75\" ",
        "v:bond=\"keep\"><v:child/></c:bond><c:atom show_hydrogens=\"0\"/>",
        "</c:standard><c:molecule id=\"m\"><c:atom id=\"a\" name=\"C\"><c:point x=\"0\" y=\"0\"/></c:atom></c:molecule>",
        "</c:cdml>"
    );
    let mut session = DocumentSession::load(source).expect("retained source is valid");
    assert!(matches!(
        session.prepare_session_operation_transition_v1(
            crate::SessionOperationTransitionRequestV1::new(
                0,
                changed_operation(2.0),
                TransitionAuthorizationV1::None
            )
        ),
        Err(DocumentSessionError::RendererAdmission)
    ));
    assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
}
