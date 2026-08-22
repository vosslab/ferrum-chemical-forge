use ferrum_domain::haworth::StandaloneDGlucoseHaworthRecipeV1;

use super::super::{DocumentHaworthPositionV1, DocumentSession, Point3V1};

fn has_haworth_facts(
    observation: &super::super::SessionDocumentObservationV1,
    molecule: &str,
) -> bool {
    observation
        .projection()
        .molecules()
        .iter()
        .find(|candidate| candidate.source_id() == Some(molecule))
        .is_some_and(|candidate| {
            candidate
                .atoms()
                .iter()
                .filter(|atom| atom.element() == Some("C"))
                .count()
                == 6
                && candidate
                    .atoms()
                    .iter()
                    .filter(|atom| atom.element() == Some("O"))
                    .count()
                    == 6
                && candidate.bonds().len() == 12
                && candidate
                    .bonds()
                    .iter()
                    .all(|bond| matches!(bond.source_type(), Some("n1") | Some("q1") | Some("w1")))
                && candidate.bonds().iter().any(|bond| {
                    bond.source_type() == Some("q1")
                        && bond.haworth_position() == Some(DocumentHaworthPositionV1::Front)
                })
                && candidate
                    .bonds()
                    .iter()
                    .filter(|bond| bond.source_type() == Some("w1"))
                    .all(|bond| bond.haworth_position() == Some(DocumentHaworthPositionV1::Front))
        })
}

#[test]
fn standalone_haworth_commit_is_atomic_reversible_and_reopens_with_its_recipe_facts() {
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty source loads");
    let mut prepared = session
        .prepare_create_standalone_haworth_v1(
            0,
            StandaloneDGlucoseHaworthRecipeV1::BetaDGlucofuranose,
            Point3V1::new(13.0, -7.0, 0.0).expect("finite anchor"),
        )
        .expect("closed recipe prepares");
    let root = prepared.molecule_identifier().as_str().to_owned();
    let accepted = session
        .commit_create_standalone_haworth_v1(0, &mut prepared)
        .expect("prepared recipe commits");
    let saved = accepted.observation().snapshot().cdml().to_owned();
    let undone = session.undo(1).expect("one accepted insertion is undoable");
    let redone = session.redo(2).expect("one accepted insertion is redoable");
    let reopened = DocumentSession::load(&saved).expect("saved CDML reopens");

    assert!(
        undone.observation().projection().molecules().is_empty()
            && has_haworth_facts(redone.observation(), &root)
            && has_haworth_facts(&reopened.observe(0).expect("reopened projection"), &root)
    );
}

#[test]
fn standalone_haworth_rejects_invalid_or_stale_or_foreign_or_consumed_receipts_without_mutation() {
    let anchor = Point3V1::new(0.0, 0.0, 0.0).expect("finite anchor");
    let mut owner =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("owner source loads");
    let mut foreign =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("foreign source loads");
    let baseline = owner.snapshot().expect("baseline snapshot");
    assert!(Point3V1::new(f64::INFINITY, 0.0, 0.0).is_err());
    assert_eq!(owner.snapshot().expect("invalid intent is inert"), baseline);

    let mut foreign_pending = owner
        .prepare_create_standalone_haworth_v1(
            0,
            StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose,
            anchor,
        )
        .expect("candidate prepares");
    let foreign_before = foreign.snapshot().expect("foreign baseline");
    assert!(
        foreign
            .commit_create_standalone_haworth_v1(0, &mut foreign_pending)
            .is_err()
    );
    assert_eq!(
        foreign.snapshot().expect("foreign stays unchanged"),
        foreign_before
    );

    let mut stale = owner
        .prepare_create_standalone_haworth_v1(
            0,
            StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucofuranose,
            anchor,
        )
        .expect("candidate prepares");
    let mut accepted = owner
        .prepare_create_standalone_haworth_v1(
            0,
            StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose,
            anchor,
        )
        .expect("current candidate prepares");
    owner
        .commit_create_standalone_haworth_v1(0, &mut accepted)
        .expect("current candidate commits");
    let after_acceptance = owner.snapshot().expect("accepted snapshot");
    assert!(
        owner
            .commit_create_standalone_haworth_v1(1, &mut stale)
            .is_err()
    );
    assert_eq!(
        owner.snapshot().expect("stale receipt is inert"),
        after_acceptance
    );
    assert!(
        owner
            .commit_create_standalone_haworth_v1(1, &mut accepted)
            .is_err()
    );
    assert_eq!(
        owner.snapshot().expect("consumed receipt is inert"),
        after_acceptance
    );
}
