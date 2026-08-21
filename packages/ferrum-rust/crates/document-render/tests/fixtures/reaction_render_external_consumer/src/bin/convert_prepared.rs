use ferrum_document_render::PreparedReactionV1;

fn main() {
    fn convert(prepared: PreparedReactionV1) {
        let _candidate: String = prepared.into();
    }
    let _ = convert;
}
