//! Closed Haworth catalog entries lower into generic document operations.

use ferrum_catalog_placement::resolve_catalog_molecule_placement_v1;
use ferrum_document::{
    DocumentSession, PresentationGesturePoint2V1, SessionOperation, SessionOperationOutcomeV1,
    SessionOperationV1, TransitionAuthorizationV1,
};

const KEY: &str = "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose";

#[test]
fn haworth_catalog_operation_preserves_key_anchor_and_directed_stereo() {
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty CDML");
    let anchor = PresentationGesturePoint2V1::new(100.0, -25.0).expect("anchor");
    let request = resolve_catalog_molecule_placement_v1(KEY, anchor).expect("closed catalog key");
    let mut prepared = session
        .prepare_session_operation_transition_v1(
            0,
            SessionOperation::V1(SessionOperationV1::PlaceCatalogMoleculeV1(request)),
            TransitionAuthorizationV1::None,
        )
        .expect("generic transition");
    let result = session
        .commit_session_operation_transition_v1(&mut prepared)
        .expect("generic commit");
    let SessionOperationOutcomeV1::CatalogMoleculePlacementV1(outcome) = result.outcome() else {
        panic!("catalog outcome");
    };
    assert_eq!(outcome.catalog_key().as_str(), KEY);
    assert_eq!(outcome.anchor(), anchor);
    assert!(
        outcome
            .root_identifier()
            .as_str()
            .starts_with("ferrum-molecule-v1-")
    );
    assert!(
        result
            .observation()
            .snapshot()
            .cdml()
            .contains("type=\"q1\"")
    );
}
