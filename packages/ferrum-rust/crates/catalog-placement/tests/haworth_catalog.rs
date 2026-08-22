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
        assert_eq!(source.matches("position=\"front\"").count(), 3);
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
        let expected_start = format!("{}-a3", accepted.identifier());
        let expected_end = format!("{}-a4", accepted.identifier());
        assert_eq!(q1.start().source_id(), Some(expected_start.as_str()));
        assert_eq!(q1.end().source_id(), Some(expected_end.as_str()));
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
fn literal_haworth_namespace_reserves_opaque_declarations() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"host\"><atom id=\"host-a\" name=\"C\"><point x=\"0\" y=\"0\"/><opaque id=\"ferrum-catalog-d-glucose-haworth-1-a1\"><retained/></opaque></atom></molecule></cdml>";
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
    assert_ne!(committed.identifier(), "ferrum-catalog-d-glucose-haworth-1");
    assert!(
        session
            .snapshot()
            .expect("snapshot")
            .cdml()
            .contains("<opaque id=\"ferrum-catalog-d-glucose-haworth-1-a1\"")
    );
}
