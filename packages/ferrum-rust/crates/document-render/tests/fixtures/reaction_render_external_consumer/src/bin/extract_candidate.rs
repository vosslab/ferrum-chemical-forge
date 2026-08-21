use ferrum_document_render::PreparedReactionV1;

fn main() {
    fn extract(prepared: PreparedReactionV1) {
        let _candidate = prepared.candidate_cdml;
    }
    let _ = extract;
}
