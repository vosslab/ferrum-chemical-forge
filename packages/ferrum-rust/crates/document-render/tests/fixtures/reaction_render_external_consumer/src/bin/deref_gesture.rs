use ferrum_document_render::ReactionGestureV1;

fn main() {
    fn dereference(gesture: ReactionGestureV1) {
        let _value = *gesture;
    }
    let _ = dereference;
}
