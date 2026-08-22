use ferrum_document::DocumentSession;

fn main() {
    let mut session = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty CDML must load");
    let _ = session.prepare_reaction_candidate_v1(0, ());
}
