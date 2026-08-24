//! Literal Haworth catalog compilation keeps the receipt's depiction facts.

use ferrum_catalog_placement::{
    begin_catalog_placement_v1, catalog_molecule_placement_gesture_v1,
    preview_catalog_placement_v1, resolve_catalog_placement_v1,
};
use ferrum_document::{
    DocumentFenceV1, DocumentSession, PendingCatalogMoleculePlacementV1,
    PresentationGesturePoint2V1,
};

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

fn pending(
    session: &mut DocumentSession,
    gesture: &ferrum_catalog_placement::CatalogPlacementGestureV1,
    preview: &ferrum_catalog_placement::CatalogPlacementPreviewV1,
) -> PendingCatalogMoleculePlacementV1 {
    let request = resolve_catalog_placement_v1(gesture, preview).expect("catalog request");
    session
        .prepare_catalog_molecule_placement_v1(
            catalog_molecule_placement_gesture_v1(gesture),
            request,
        )
        .expect("document pending")
}

#[test]
fn closed_haworth_entries_commit_and_retain_directed_stereo_tokens() {
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
        let mut prepared = pending(&mut session, &gesture, &preview);
        let identifier = prepared.identifier().to_owned();
        let accepted = session
            .commit_catalog_molecule_placement_v1(&mut prepared)
            .expect("commit");
        let source = accepted.observation().snapshot().cdml();
        assert!(source.contains("type=\"q1\""));
        assert!(source.contains("haworth_position=\"front\""));
        assert!(identifier.starts_with("ferrum-molecule-v1-"));
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
    let mut prepared = pending(&mut session, &gesture, &preview);
    let identifier = prepared.identifier().to_owned();
    session
        .commit_catalog_molecule_placement_v1(&mut prepared)
        .expect("commit");
    assert!(identifier.starts_with("ferrum-molecule-v1-"));
    assert_ne!(identifier, "ferrum-molecule-v1-0");
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
    let discarded = pending(&mut session, &first, &first_preview);
    let identifier = discarded.identifier().to_owned();
    drop(discarded);

    let second = begin_catalog_placement_v1(&session, fence(&session), KEYS[0]).expect("gesture");
    let second_preview = preview_catalog_placement_v1(&session, &second, anchor).expect("preview");
    let mut accepted = pending(&mut session, &second, &second_preview);
    assert_eq!(accepted.identifier(), identifier);
    session
        .commit_catalog_molecule_placement_v1(&mut accepted)
        .expect("commit");
}
