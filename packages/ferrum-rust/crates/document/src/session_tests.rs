use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::{DocumentSession, DocumentSessionError, TypedDocument, element_name};

const CDML_NAMESPACE: &str = "urn:ferrum:cdml";

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

const SOURCE: &str = r#"<cdml xmlns="urn:ferrum:cdml" xmlns:vendor="urn:vendor"><molecule id="m"><atom id="a" name="C"><point x="1" y="2"/></atom></molecule><vendor:extension untouched="yes"><child/></vendor:extension></cdml>"#;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .expect("temporary root must resolve without a symbolic link")
            .join(format!(
                "ferrum-document-session-{}-{}",
                std::process::id(),
                NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&path).expect("test directory must be creatable");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("test directory cleanup must succeed");
    }
}

#[test]
fn load_snapshots_one_retained_tree() {
    let session = DocumentSession::load(SOURCE).expect("valid CDML must load");
    let snapshot = session.snapshot().expect("loaded tree must serialize");

    assert!(snapshot.cdml().contains("untouched=\"yes\""));
    let reparsed = TypedDocument::parse(snapshot.cdml()).expect("snapshot must reparse");
    assert!(
        reparsed
            .to_xml()
            .expect("reparsed tree must serialize")
            .contains("vendor:extension")
    );
}

#[test]
fn empty_document_constructor_creates_a_clean_observable_canonical_baseline() {
    let session = DocumentSession::create_empty_document_v1().expect("empty document must load");
    let observation = session.observe(0).expect("empty document must project");
    let retained = TypedDocument::parse(observation.snapshot().cdml())
        .expect("empty snapshot must remain CDML");
    let xml = retained.indexed().xml();
    let root = xml
        .tree
        .document_element(xml.document)
        .expect("retained CDML must have a root");

    assert_eq!(
        element_name(&xml.tree, root),
        Some(("cdml".to_owned(), CDML_NAMESPACE.to_owned()))
    );
    assert_eq!(retained.root().attribute("version"), Some("26.07"));
    assert!(observation.projection().molecules().is_empty());
    assert!(
        observation
            .projection()
            .presentation_stack()
            .roots()
            .is_empty()
    );
    assert!(!observation.snapshot().is_dirty());
}

#[test]
fn empty_document_constructor_reopens_as_a_clean_revision_zero_baseline() {
    let mut session =
        DocumentSession::create_empty_document_v1().expect("empty document must load");
    let saved = session.snapshot().expect("empty document must serialize");
    let reopened = DocumentSession::load(saved.cdml()).expect("saved empty document must reopen");

    assert_eq!(
        reopened
            .snapshot()
            .expect("reopened snapshot must serialize")
            .revision(),
        0
    );
    assert!(
        !reopened
            .observe(0)
            .expect("reopened empty document must project")
            .snapshot()
            .is_dirty()
    );
    assert!(matches!(
        session.undo(0),
        Err(DocumentSessionError::HistoryUnavailable)
    ));
}

#[test]
fn invalid_cdml_is_reported_as_a_load_failure() {
    assert!(matches!(
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule></cdml>"),
        Err(DocumentSessionError::Load(_))
    ));
    assert!(matches!(
        DocumentSession::load("<not-cdml/>"),
        Err(DocumentSessionError::Load(_))
    ));
    assert!(matches!(
        DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"same\"/><text id=\"same\"/></cdml>"
        ),
        Err(DocumentSessionError::Load(_))
    ));
}

#[test]
fn atomic_save_publishes_an_exact_snapshot() {
    let directory = TestDirectory::new();
    let target = directory.path().join("saved.cdml");
    let mut session = DocumentSession::load(SOURCE).expect("valid CDML must load");

    let published = session.save_atomic(&target, 0).expect("save must publish");
    assert_eq!(
        fs::read_to_string(&target).expect("published file must read"),
        published.published_snapshot().cdml()
    );
    assert!(
        fs::read_dir(directory.path())
            .expect("directory must read")
            .all(|entry| !entry
                .expect("entry must read")
                .file_name()
                .to_string_lossy()
                .contains(".tmp"))
    );

    let reopened = DocumentSession::load(published.published_snapshot().cdml())
        .expect("published CDML must reopen");
    assert_eq!(
        reopened.snapshot().expect("reopened snapshot must work"),
        *published.published_snapshot()
    );
    assert!(matches!(
        published.outcome(),
        super::SaveOutcome::Confirmed | super::SaveOutcome::DirectoryEntryUnconfirmed
    ));
}

#[test]
fn symbolic_link_destination_is_rejected_without_following_it() {
    let directory = TestDirectory::new();
    let target = directory.path().join("outside.cdml");
    fs::write(&target, "original").expect("outside target must write");
    let destination = directory.path().join("linked.cdml");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&target, &destination).expect("symlink must create");
    let mut session = DocumentSession::load(SOURCE).expect("valid CDML must load");

    assert!(matches!(
        session.save_atomic(&destination, 0),
        Err(DocumentSessionError::InvalidDestination { .. })
    ));
    assert_eq!(
        fs::read_to_string(target).expect("link target must remain readable"),
        "original"
    );
}

#[cfg(unix)]
#[test]
fn symbolic_link_parent_is_rejected_without_publishing_through_it() {
    let directory = TestDirectory::new();
    let trusted_parent = directory.path().join("trusted");
    let linked_parent = directory.path().join("linked");
    fs::create_dir(&trusted_parent).expect("trusted parent must create");
    std::os::unix::fs::symlink(&trusted_parent, &linked_parent)
        .expect("parent symbolic link must create");
    let destination = linked_parent.join("saved.cdml");
    let mut session = DocumentSession::load(SOURCE).expect("valid CDML must load");

    assert!(matches!(
        session.save_atomic(&destination, 0),
        Err(DocumentSessionError::InvalidDestination { .. })
    ));
    assert!(!trusted_parent.join("saved.cdml").exists());
}

#[test]
fn nonregular_destination_is_rejected() {
    let directory = TestDirectory::new();
    let mut session = DocumentSession::load(SOURCE).expect("valid CDML must load");

    assert!(matches!(
        session.save_atomic(directory.path(), 0),
        Err(DocumentSessionError::InvalidDestination { .. })
    ));
}

#[test]
fn missing_parent_stops_before_publication() {
    let directory = TestDirectory::new();
    let target = directory.path().join("missing-parent").join("saved.cdml");
    let mut session = DocumentSession::load(SOURCE).expect("valid CDML must load");

    assert!(matches!(
        session.save_atomic(&target, 0),
        Err(DocumentSessionError::PublishNotStarted { .. })
    ));
}

#[cfg(unix)]
#[test]
fn existing_read_only_file_can_be_replaced_without_reusing_its_mode() {
    use std::os::unix::fs::PermissionsExt;

    let directory = TestDirectory::new();
    let target = directory.path().join("read-only.cdml");
    fs::write(&target, "previous").expect("target fixture must write");
    fs::set_permissions(&target, fs::Permissions::from_mode(0o400))
        .expect("target mode must be set");
    let mut session = DocumentSession::load(SOURCE).expect("valid CDML must load");

    session
        .save_atomic(&target, 0)
        .expect("directory-authorized replacement must succeed");
    assert_ne!(
        fs::metadata(&target)
            .expect("replacement metadata must read")
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
}
