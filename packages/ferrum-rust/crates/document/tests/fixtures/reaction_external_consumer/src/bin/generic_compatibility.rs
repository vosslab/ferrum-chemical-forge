use ferrum_document::{DocumentFenceV1, DocumentSession};

fn main() {
    let mut session = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty CDML must load");
    let snapshot = session.snapshot().expect("new session must have a snapshot");
    let fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());

    let mut pending = session
        .prepare_complete_cdml_mutation_v1(fence, snapshot.cdml())
        .expect("renderer-admitted complete-CDML transaction must prepare");
    session
        .commit_complete_cdml_mutation_v1(&mut pending)
        .expect("renderer-admitted complete-CDML transaction must commit");
}
