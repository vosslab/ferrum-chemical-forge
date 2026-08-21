use super::{DocumentSession, DocumentSessionError, SessionOperationError, TypedDocumentError};

const CHAIN: &str = concat!(
    "<cdml><molecule id=\"m\" name=\"chain\">",
    "<atom id=\"a\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\"><point x=\"1\" y=\"0\"/></atom><atom id=\"c\"><point x=\"2\" y=\"0\"/></atom><atom id=\"d\"><point x=\"3\" y=\"0\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\"/>",
    "<bond id=\"bc\" start=\"b\" end=\"c\"/>",
    "<bond id=\"cd\" start=\"c\" end=\"d\"/>",
    "</molecule></cdml>",
);

#[test]
fn structural_deletion_splits_in_source_order_and_receipts_induced_bonds() {
    let mut session = DocumentSession::load(CHAIN).expect("fixture loads");
    let mut pending = session
        .prepare_delete_structure_v1(0, "m".to_owned(), vec!["b".to_owned()], vec![])
        .expect("plan prepares");
    let receipt = pending.receipt();
    assert_eq!(
        receipt
            .removed_atom_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["b"]
    );
    assert_eq!(
        receipt
            .removed_bond_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["ab", "bc"]
    );
    assert_eq!(receipt.components().len(), 2);
    assert_eq!(
        receipt.components()[0]
            .atom_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["a"]
    );
    assert_eq!(
        receipt.components()[1]
            .atom_ids()
            .iter()
            .map(|id| id.as_str())
            .collect::<Vec<_>>(),
        ["c", "d"]
    );
    assert_ne!(receipt.components()[1].molecule_id().as_str(), "m");
    let committed = session
        .commit_delete_structure_v1(0, &mut pending)
        .expect("commit succeeds");
    assert_eq!(committed.observation().snapshot().revision(), 1);
    let cdml = committed.observation().snapshot().cdml();
    assert!(cdml.contains("<molecule id=\"m\" name=\"chain\"><atom id=\"a\">"));
    assert!(cdml.contains("<atom id=\"c\"><point"));
    assert!(cdml.contains("<bond id=\"cd\""));
}

#[test]
fn reaction_referenced_split_is_atomic() {
    let source = format!(
        "{}<reaction id=\"r\"><reactant idref=\"m\"/></reaction>",
        CHAIN.replace("</cdml>", "")
    ) + "</cdml>";
    let mut session = DocumentSession::load(&source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot works");
    assert!(matches!(
        session.prepare_delete_structure_v1(0, "m".to_owned(), vec!["b".to_owned()], vec![]),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(
                TypedDocumentError::ReactionReferencedStructureDeletion(_)
            )
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot works"), before);
}

#[test]
fn unsupported_direct_content_is_atomic() {
    let source = "<cdml><molecule id=\"m\"><atom id=\"a\"/><note/></molecule></cdml>";
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot works");
    assert!(matches!(
        session.prepare_delete_structure_v1(0, "m".to_owned(), vec!["a".to_owned()], vec![]),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(
                TypedDocumentError::UnsupportedStructureDeletionMolecule(_)
            )
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot works"), before);
}
