//! Literal Haworth catalog compilation keeps the receipt's depiction facts.

use ferrum_catalog_placement::{
    begin_catalog_placement_v1, commit_catalog_placement_v1, prepare_catalog_placement_v1,
    preview_catalog_placement_v1,
};
use ferrum_document::{DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1};

const KEYS: &[&str] = &[
    "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose",
    "biomolecules/carbohydrates/d-glucose/beta-d-glucopyranose",
    "biomolecules/carbohydrates/d-glucose/alpha-d-glucofuranose",
    "biomolecules/carbohydrates/d-glucose/beta-d-glucofuranose",
];

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("snapshot");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

#[test]
fn closed_haworth_entries_compile_literal_coordinates_and_directed_stereo_tokens() {
    for key in KEYS {
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty CDML");
        let gesture = begin_catalog_placement_v1(&session, fence(&session), key).expect("key");
        let preview = preview_catalog_placement_v1(
            &session,
            &gesture,
            PresentationGesturePoint2V1::new(100.0, -25.0).expect("anchor"),
        )
        .expect("literal preview");
        assert_eq!(preview.overlay().atom_points.len(), 12);
        assert_eq!(preview.overlay().bond_segments.len(), 12);
        let mut prepared = prepare_catalog_placement_v1(&mut session, &gesture, &preview)
            .expect("renderer-preflighted receipt");
        let accepted = commit_catalog_placement_v1(&mut session, &mut prepared).expect("commit");
        let source = accepted.result().observation().snapshot().cdml();
        assert_eq!(source.matches("<atom ").count(), 12);
        assert_eq!(source.matches("<bond ").count(), 12);
        assert_eq!(source.matches("type=\"q1\"").count(), 1);
        assert_eq!(source.matches("type=\"w1\"").count(), 2);
        assert_eq!(source.matches("haworth_position=\"front\"").count(), 3);
        assert!(accepted.identifier().starts_with("ferrum-molecule-v1-"));
        let molecule = accepted
            .result()
            .observation()
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.source_id() == Some(accepted.identifier()))
            .expect("inserted molecule");
        let q1 = molecule
            .bonds()
            .iter()
            .find(|bond| bond.source_type() == Some("q1"))
            .expect("front stroke bond");
        assert!(
            q1.start()
                .source_id()
                .is_some_and(|identifier| identifier.starts_with("ferrum-atom-v1-"))
        );
        assert!(
            q1.end()
                .source_id()
                .is_some_and(|identifier| identifier.starts_with("ferrum-atom-v1-"))
        );
        session.undo(1).expect("undo");
        let redone = session.redo(2).expect("redo");
        assert!(
            redone
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"q1\"")
        );
    }
}

#[test]
fn haworth_catalog_uses_document_ids_which_respect_opaque_declarations() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"host\"><atom id=\"host-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><opaque id=\"ferrum-molecule-v1-0\"><retained/></opaque><opaque id=\"ferrum-atom-v1-0\"><retained/></opaque></cdml>";
    let mut session = DocumentSession::load(source).expect("opaque source");
    let gesture = begin_catalog_placement_v1(&session, fence(&session), KEYS[0]).expect("key");
    let preview = preview_catalog_placement_v1(
        &session,
        &gesture,
        PresentationGesturePoint2V1::new(0.0, 0.0).expect("anchor"),
    )
    .expect("preview");
    let mut prepared =
        prepare_catalog_placement_v1(&mut session, &gesture, &preview).expect("prepare");
    let committed = commit_catalog_placement_v1(&mut session, &mut prepared).expect("commit");
    assert!(committed.identifier().starts_with("ferrum-molecule-v1-"));
    assert_ne!(committed.identifier(), "ferrum-molecule-v1-0");
    assert!(
        session
            .snapshot()
            .expect("snapshot")
            .cdml()
            .contains("<opaque id=\"ferrum-molecule-v1-0\"")
    );
}

#[test]
fn discarded_haworth_catalog_candidate_leaves_document_allocation_tentative() {
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty CDML");
    let anchor = PresentationGesturePoint2V1::new(0.0, 0.0).expect("anchor");
    let first = begin_catalog_placement_v1(&session, fence(&session), KEYS[0]).expect("gesture");
    let first_preview = preview_catalog_placement_v1(&session, &first, anchor).expect("preview");
    let discarded =
        prepare_catalog_placement_v1(&mut session, &first, &first_preview).expect("candidate");
    let identifier = discarded.identifier().to_owned();
    drop(discarded);

    let second = begin_catalog_placement_v1(&session, fence(&session), KEYS[0]).expect("gesture");
    let second_preview = preview_catalog_placement_v1(&session, &second, anchor).expect("preview");
    let mut accepted = prepare_catalog_placement_v1(&mut session, &second, &second_preview)
        .expect("replacement candidate");
    assert_eq!(accepted.identifier(), identifier);
    commit_catalog_placement_v1(&mut session, &mut accepted).expect("commit");
}
