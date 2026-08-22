//! Durable bond deletion and normal-order mutation behavior.

use ferrum_core::BondOrder;

use super::{
    DELETE_SOURCE, DocumentBondOrderV1, DocumentSession, DocumentSessionError, SessionOperation,
    SessionOperationError, SessionOperationV1,
};

fn delete_bond(bond_id: &str) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::DeleteBond {
        bond_id: bond_id.to_owned(),
    })
}

fn set_bond_order(bond_id: &str, order: DocumentBondOrderV1) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetBondOrder {
        bond_id: bond_id.to_owned(),
        order,
    })
}

#[test]
fn bond_deletion_removes_only_the_selected_bond_and_is_one_history_entry() {
    let mut session = DocumentSession::load(DELETE_SOURCE).expect("source must load");
    let deleted = session
        .submit(0, delete_bond("ab"))
        .expect("durable bond must delete");
    let molecule = &deleted.observation().projection().molecules()[0];
    assert_eq!(deleted.observation().snapshot().revision(), 1);
    assert_eq!(molecule.atoms().len(), 3);
    assert_eq!(
        molecule
            .bonds()
            .iter()
            .map(|bond| bond.source_id().expect("fixture bond is durable"))
            .collect::<Vec<_>>(),
        ["bc", "ac"]
    );
    assert!(
        deleted
            .observation()
            .snapshot()
            .cdml()
            .contains("retained-bond=\"ab\"")
    );

    let undone = session.undo(1).expect("bond deletion must undo once");
    assert_eq!(
        undone.observation().projection().molecules()[0]
            .bonds()
            .len(),
        3
    );
    let redone = session.redo(2).expect("bond deletion must redo once");
    assert_eq!(
        redone.observation().projection().molecules()[0]
            .bonds()
            .len(),
        2
    );
}

#[test]
fn bond_deletion_rejects_unknown_identity_without_state_change() {
    let mut session = DocumentSession::load(DELETE_SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.submit(0, delete_bond("missing")),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownBond(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

#[test]
fn bond_order_change_is_typed_noop_aware_and_one_history_entry() {
    let mut session = DocumentSession::load(DELETE_SOURCE).expect("source must load");
    let no_change = session
        .submit(0, set_bond_order("ab", DocumentBondOrderV1::Single))
        .expect("current order must be a no-op");
    assert_eq!(no_change.observation().snapshot().revision(), 0);

    let changed = session
        .submit(0, set_bond_order("ab", DocumentBondOrderV1::Double))
        .expect("durable bond order must change");
    let bond = changed.observation().projection().molecules()[0]
        .bonds()
        .iter()
        .find(|bond| bond.source_id() == Some("ab"))
        .expect("changed bond remains projected");
    assert_eq!(changed.observation().snapshot().revision(), 1);
    assert_eq!(bond.order(), Some(BondOrder::Double));
    assert_eq!(bond.source_type(), Some("n2"));
    assert!(
        changed
            .observation()
            .snapshot()
            .cdml()
            .contains("retained-bond=\"ab\"")
    );

    let repeated = session
        .submit(1, set_bond_order("ab", DocumentBondOrderV1::Double))
        .expect("repeated order must remain a no-op");
    assert_eq!(repeated.observation().snapshot().revision(), 1);
    let undone = session.undo(1).expect("one change must undo once");
    let undone_bond = undone.observation().projection().molecules()[0]
        .bonds()
        .iter()
        .find(|bond| bond.source_id() == Some("ab"))
        .expect("undone bond remains projected");
    assert_eq!(undone_bond.order(), Some(BondOrder::Single));
    let redone = session.redo(2).expect("one change must redo once");
    let redone_bond = redone.observation().projection().molecules()[0]
        .bonds()
        .iter()
        .find(|bond| bond.source_id() == Some("ab"))
        .expect("redone bond remains projected");
    assert_eq!(redone_bond.order(), Some(BondOrder::Double));
}

#[test]
fn bond_order_change_rejects_unknown_identity_without_state_change() {
    let mut session = DocumentSession::load(DELETE_SOURCE).expect("source must load");
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.submit(0, set_bond_order("missing", DocumentBondOrderV1::Triple),),
        Err(DocumentSessionError::Operation(
            SessionOperationError::UnknownBond(_)
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}
