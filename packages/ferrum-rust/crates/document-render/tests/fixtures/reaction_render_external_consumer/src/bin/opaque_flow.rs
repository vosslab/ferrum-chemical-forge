use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::{
    ReactionCreateRequestV1, begin_reaction_gesture_v1, commit_reaction_gesture_v1,
    prepare_reaction_gesture_v1,
};

const SOURCE: &str = "<cdml><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"product\"><atom id=\"product-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow></cdml>";

fn main() {
    let mut session = DocumentSession::load(SOURCE).expect("fixture CDML must load");
    let snapshot = session.snapshot().expect("fixture session has a snapshot");
    let fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());
    let request = ReactionCreateRequestV1::new(
        snapshot.revision(),
        vec!["left".into()],
        vec!["product".into()],
        "arrow".into(),
        vec![],
        vec![],
    )
    .expect("fixture request is valid");
    let gesture = begin_reaction_gesture_v1(&session, fence, request)
        .expect("opaque reaction gesture must begin");
    let mut prepared = prepare_reaction_gesture_v1(&mut session, &gesture)
        .expect("opaque reaction gesture must prepare");
    let diagnostic = format!("{prepared:?}");
    assert!(!diagnostic.is_empty());
    let committed = commit_reaction_gesture_v1(&mut session, &mut prepared)
        .expect("opaque reaction gesture must commit");
    let _reaction_id = committed.reaction_id();
}
