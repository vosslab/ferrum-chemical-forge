use ferrum_document_render::ReactionGestureV1;

fn main() {
    fn serialize(gesture: &ReactionGestureV1) {
        let _json = serde_json::to_string(gesture);
    }
    let _ = serialize;
}
