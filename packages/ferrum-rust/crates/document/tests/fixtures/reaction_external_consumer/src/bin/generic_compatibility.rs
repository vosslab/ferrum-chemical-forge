use ferrum_document::{DocumentFenceV1, DocumentSession};

fn main() {
    let mut session = DocumentSession::load("<cdml/>").expect("empty CDML must load");
    let snapshot = session.snapshot().expect("new session must have a snapshot");
    let fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());

    session
        .commit_complete_cdml_transaction_v1(fence, snapshot.cdml())
        .expect("generic complete-CDML transaction must remain public");
}
