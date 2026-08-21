use ferrum_document_render::ReactionGestureV1;

fn require_clone<T: Clone>() {}

fn main() {
    require_clone::<ReactionGestureV1>();
}
