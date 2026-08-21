use ferrum_document_render::PreparedReactionV1;

fn require_clone<T: Clone>() {}

fn main() {
    require_clone::<PreparedReactionV1>();
}
