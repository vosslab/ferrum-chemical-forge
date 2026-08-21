use ferrum_document_render::PreparedReactionV1;

fn main() {
    fn extract(prepared: PreparedReactionV1) {
        let _plan = prepared.plan;
    }
    let _ = extract;
}
