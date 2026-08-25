use super::{DocumentSession, DocumentSessionError, SessionOperationError, TypedDocumentError};

const CHAIN: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\" name=\"chain\">",
    "<atom id=\"a\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\"><point x=\"1\" y=\"0\"/></atom><atom id=\"c\"><point x=\"2\" y=\"0\"/></atom><atom id=\"d\"><point x=\"3\" y=\"0\"/></atom>",
    "<bond id=\"ab\" start=\"a\" end=\"b\"/>",
    "<bond id=\"bc\" start=\"b\" end=\"c\"/>",
    "<bond id=\"cd\" start=\"c\" end=\"d\"/>",
    "</molecule></cdml>",
);

#[test]
fn structural_deletion_splits_in_source_order_and_receipts_induced_bonds() {
    let mut session = DocumentSession::load(CHAIN).expect("fixture loads");
    let observation = session.observe(0).expect("fixture projects");
    let initial_molecule = &observation.projection().molecules()[0];
    let molecule = initial_molecule
        .id()
        .expect("fixture molecule is durable")
        .clone();
    let atom = initial_molecule.atoms()[1]
        .id()
        .expect("fixture atom is durable")
        .clone();
    let surviving_child_ids = vec![
        initial_molecule.atoms()[0]
            .id()
            .expect("surviving atom is durable")
            .clone(),
        initial_molecule.atoms()[2]
            .id()
            .expect("surviving atom is durable")
            .clone(),
        initial_molecule.atoms()[3]
            .id()
            .expect("surviving atom is durable")
            .clone(),
        initial_molecule.bonds()[2]
            .id()
            .expect("surviving bond is durable")
            .clone(),
    ];
    let mut pending = session
        .prepare_delete_structure_v1(0, &molecule, &[atom], &[])
        .expect("plan prepares");
    let receipt = pending.receipt();
    assert_eq!(receipt.removed_atom_ids().len(), 1);
    assert_eq!(receipt.removed_bond_ids().len(), 2);
    assert_eq!(receipt.components().len(), 2);
    let committed = session
        .commit_delete_structure_v1(0, &mut pending)
        .expect("commit succeeds");
    assert_eq!(committed.observation().snapshot().revision(), 1);
    let molecules = committed.observation().projection().molecules();
    assert_eq!(molecules.len(), 2);
    assert_eq!(molecules[0].atoms().len(), 1);
    assert_eq!(molecules[1].atoms().len(), 2);
    assert_eq!(molecules[1].bonds().len(), 1);
    let split_root_ids = molecules
        .iter()
        .map(|split| split.id().expect("split root is durable").clone())
        .collect::<Vec<_>>();
    assert_eq!(split_root_ids[0], molecule);
    assert_ne!(split_root_ids[0], split_root_ids[1]);
    let committed_child_ids = molecules
        .iter()
        .flat_map(|split| {
            split
                .atoms()
                .iter()
                .filter_map(|atom| atom.id().cloned())
                .chain(split.bonds().iter().filter_map(|bond| bond.id().cloned()))
        })
        .collect::<Vec<_>>();
    assert_eq!(committed_child_ids, surviving_child_ids);
    for object_id in split_root_ids.iter().chain(&surviving_child_ids) {
        assert!(
            session
                .current_document_v1()
                .resolve_document_object_id(object_id)
                .is_some()
        );
    }
    let undone = session.undo(1).expect("split deletion undoes");
    for object_id in &surviving_child_ids {
        assert!(
            session
                .current_document_v1()
                .resolve_document_object_id(object_id)
                .is_some()
        );
    }
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("split deletion redoes");
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("serialized split document reopens");
    for object_id in split_root_ids.iter().chain(&surviving_child_ids) {
        assert!(
            reopened
                .current_document_v1()
                .resolve_document_object_id(object_id)
                .is_some()
        );
    }
}

#[test]
fn reaction_referenced_split_is_atomic() {
    let source = format!(
        "{}<reaction id=\"r\"><reactant idref=\"m\"/></reaction>",
        CHAIN.replace("</cdml>", "")
    ) + "</cdml>";
    let mut session = DocumentSession::load(&source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot works");
    let observation = session.observe(0).expect("fixture projects");
    let molecule = observation.projection().molecules()[0]
        .id()
        .expect("fixture molecule is durable")
        .clone();
    let atom = observation.projection().molecules()[0].atoms()[1]
        .id()
        .expect("fixture atom is durable")
        .clone();
    assert!(matches!(
        session.prepare_delete_structure_v1(0, &molecule, &[atom], &[]),
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
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\"><point x=\"0\" y=\"0\"/></atom><note/></molecule></cdml>";
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot works");
    let observation = session.observe(0).expect("fixture projects");
    let molecule = observation.projection().molecules()[0]
        .id()
        .expect("fixture molecule is durable")
        .clone();
    let atom = observation.projection().molecules()[0].atoms()[0]
        .id()
        .expect("fixture atom is durable")
        .clone();
    assert!(matches!(
        session.prepare_delete_structure_v1(0, &molecule, &[atom], &[]),
        Err(DocumentSessionError::Operation(
            SessionOperationError::Candidate(
                TypedDocumentError::UnsupportedStructureDeletionMolecule(_)
            )
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot works"), before);
}
