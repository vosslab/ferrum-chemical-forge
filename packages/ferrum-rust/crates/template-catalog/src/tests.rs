use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use ferrum_document::DocumentSession;

use crate::{
    TEMPLATE_CATALOG_SCHEMA_V1, TemplateCatalogApplyErrorV1, TemplateCatalogLimitsV1,
    TemplateCatalogRefusalCategoryV1, TemplateCatalogSourceV1, apply_template_catalog_entry_v1,
    snapshot_template_catalog_v1,
};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);
const FIRST_TEMPLATE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\">",
    "<molecule id=\"source-molecule\" name=\"First molecule\">",
    "<atom id=\"source-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule></cdml>",
);
const SECOND_TEMPLATE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\">",
    "<molecule id=\"source-molecule\" name=\"Second molecule\">",
    "<atom id=\"source-a\" name=\"O\"><point x=\"3\" y=\"4\"/></atom>",
    "</molecule></cdml>",
);
const UNNAMED_TEMPLATE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\" version=\"26.07\">",
    "<molecule id=\"source-molecule\"><atom id=\"source-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
    "</molecule></cdml>",
);

struct TestDirectory(PathBuf);
impl TestDirectory {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .canonicalize()
            .expect("temporary root must resolve")
            .join(format!(
                "ferrum-template-catalog-{}-{}",
                std::process::id(),
                NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir(&path).expect("test directory must be creatable");
        Self(path)
    }
    fn path(&self) -> &Path {
        &self.0
    }
    fn write(&self, name: &str, content: &[u8]) {
        fs::write(self.path().join(name), content).expect("fixture must write");
    }
}
impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("test directory cleanup must succeed");
    }
}

fn limits() -> TemplateCatalogLimitsV1 {
    TemplateCatalogLimitsV1::new(8, 8 * 1024, 16 * 1024)
}
fn user_entries(
    snapshot: &crate::TemplateCatalogSnapshotV1,
) -> Vec<&crate::TemplateCatalogEntryV1> {
    snapshot
        .entries()
        .iter()
        .filter(|entry| entry.source() == TemplateCatalogSourceV1::UserDirectory)
        .collect()
}
fn has_refusal(
    snapshot: &crate::TemplateCatalogSnapshotV1,
    category: TemplateCatalogRefusalCategoryV1,
) -> bool {
    snapshot
        .refusals()
        .iter()
        .any(|refusal| refusal.category() == category)
}

#[test]
fn unnamed_user_template_uses_admitted_filename_stem_as_label() {
    let directory = TestDirectory::new();
    directory.write("named-by-file.cdml", UNNAMED_TEMPLATE.as_bytes());
    let snapshot =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("snapshot");
    let entry = user_entries(&snapshot)
        .into_iter()
        .next()
        .expect("user entry");
    assert_eq!(entry.label(), "named-by-file");
    assert!(
        entry
            .aliases()
            .iter()
            .any(|alias| alias == "named-by-file.cdml")
    );
}

#[test]
fn shipped_metadata_is_closed_and_user_entries_follow_in_key_order() {
    let directory = TestDirectory::new();
    directory.write("z.cdml", FIRST_TEMPLATE.as_bytes());
    directory.write("a.cdml", SECOND_TEMPLATE.as_bytes());
    let snapshot =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("snapshot");
    assert_eq!(snapshot.schema(), TEMPLATE_CATALOG_SCHEMA_V1);
    assert!(!snapshot.catalog_version().is_empty());
    let first_user = snapshot
        .entries()
        .iter()
        .position(|entry| entry.source() == TemplateCatalogSourceV1::UserDirectory)
        .expect("valid user templates must be listed");
    assert!(
        snapshot.entries()[..first_user]
            .iter()
            .all(|entry| entry.source() == TemplateCatalogSourceV1::Shipped)
    );
    assert!(
        snapshot.entries()[first_user..]
            .windows(2)
            .all(|pair| pair[0].key().as_str().as_bytes() < pair[1].key().as_str().as_bytes())
    );
    for (position, entry) in snapshot.entries().iter().enumerate() {
        assert_eq!(
            entry.entry_order(),
            position,
            "entry order must describe final snapshot order"
        );
        assert_eq!(entry.content_identity().algorithm(), "sha256");
        match entry.source() {
            TemplateCatalogSourceV1::Shipped => {
                assert_eq!(
                    entry.compatibility().format(),
                    crate::TemplateFormatV1::FerrumAuthoredRecipe
                );
                assert_eq!(
                    entry.compatibility().profile(),
                    "ferrum-authored-recipe-profile-v1"
                );
                assert!(entry.provenance().license_spdx().is_some());
                assert!(entry.provenance().reviewed_on().is_some());
            }
            TemplateCatalogSourceV1::UserDirectory => {
                assert_eq!(
                    entry.compatibility().format(),
                    crate::TemplateFormatV1::Cdml
                );
                assert_eq!(
                    entry.compatibility().profile(),
                    "ferrum-document-user-template-profile-v1"
                );
                assert!(entry.provenance().license_spdx().is_none());
                assert!(entry.provenance().reviewed_on().is_none());
            }
        }
    }
}

#[test]
fn content_identity_is_rename_stable_replacement_sensitive_and_deduplicated() {
    let directory = TestDirectory::new();
    directory.write("first.cdml", FIRST_TEMPLATE.as_bytes());
    let first =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("first snapshot");
    let first_entry = user_entries(&first).pop().expect("one user entry");
    let first_key = first_entry.key().as_str().to_owned();
    let first_identity = first_entry.content_identity().hex().to_owned();
    fs::rename(
        directory.path().join("first.cdml"),
        directory.path().join("renamed.cdml"),
    )
    .expect("rename fixture");
    let renamed =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("renamed snapshot");
    let renamed_entry = user_entries(&renamed).pop().expect("renamed user entry");
    assert_eq!(renamed_entry.key().as_str(), first_key);
    assert_eq!(renamed_entry.content_identity().hex(), first_identity);
    directory.write("alias.cdml", FIRST_TEMPLATE.as_bytes());
    let deduplicated = snapshot_template_catalog_v1(Some(directory.path()), limits())
        .expect("deduplicated snapshot");
    let deduplicated_entry = user_entries(&deduplicated)
        .pop()
        .expect("one deduplicated entry");
    assert_eq!(deduplicated_entry.key().as_str(), first_key);
    assert!(
        deduplicated_entry
            .aliases()
            .iter()
            .any(|alias| alias == "alias.cdml")
    );
    assert!(has_refusal(
        &deduplicated,
        TemplateCatalogRefusalCategoryV1::DuplicateContent
    ));
    directory.write("renamed.cdml", SECOND_TEMPLATE.as_bytes());
    let replaced = snapshot_template_catalog_v1(Some(directory.path()), limits())
        .expect("replacement snapshot");
    assert_ne!(replaced.identity().hex(), deduplicated.identity().hex());
    assert!(
        user_entries(&replaced)
            .iter()
            .any(|entry| entry.key().as_str() != first_key)
    );
}

#[test]
fn hostile_neighbors_and_limits_are_isolated_from_valid_entries() {
    let directory = TestDirectory::new();
    directory.write("good.cdml", FIRST_TEMPLATE.as_bytes());
    directory.write("bad.cdml", b"\xff\xfe");
    directory.write("ignored.CDML", SECOND_TEMPLATE.as_bytes());
    fs::create_dir(directory.path().join("nested.cdml")).expect("nested directory");
    fs::write(
        directory.path().join("nested.cdml").join("inside.cdml"),
        SECOND_TEMPLATE,
    )
    .expect("nested content");
    let snapshot =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("snapshot");
    assert!(!user_entries(&snapshot).is_empty());
    assert!(has_refusal(
        &snapshot,
        TemplateCatalogRefusalCategoryV1::Utf8Invalid
    ));
    assert!(has_refusal(
        &snapshot,
        TemplateCatalogRefusalCategoryV1::CandidateNotRegular
    ));
    assert!(
        user_entries(&snapshot)
            .iter()
            .all(|entry| entry.label() != "Second molecule")
    );
    let oversized = TestDirectory::new();
    oversized.write("large.cdml", FIRST_TEMPLATE.as_bytes());
    let too_large = snapshot_template_catalog_v1(
        Some(oversized.path()),
        TemplateCatalogLimitsV1::new(8, 64, 16 * 1024),
    )
    .expect("oversized snapshot");
    assert!(has_refusal(
        &too_large,
        TemplateCatalogRefusalCategoryV1::FileTooLarge
    ));
    let capped = TestDirectory::new();
    capped.write("a.cdml", FIRST_TEMPLATE.as_bytes());
    capped.write("b.cdml", SECOND_TEMPLATE.as_bytes());
    let entry_capped = snapshot_template_catalog_v1(
        Some(capped.path()),
        TemplateCatalogLimitsV1::new(1, 8 * 1024, 16 * 1024),
    )
    .expect("entry-capped snapshot");
    assert!(has_refusal(
        &entry_capped,
        TemplateCatalogRefusalCategoryV1::CatalogLimitExceeded
    ));

    let aggregate = TestDirectory::new();
    aggregate.write("a.cdml", FIRST_TEMPLATE.as_bytes());
    aggregate.write("b.cdml", SECOND_TEMPLATE.as_bytes());
    let aggregate_capped = snapshot_template_catalog_v1(
        Some(aggregate.path()),
        TemplateCatalogLimitsV1::new(8, 8 * 1024, FIRST_TEMPLATE.len() + 1),
    )
    .expect("aggregate-capped snapshot");
    assert!(has_refusal(
        &aggregate_capped,
        TemplateCatalogRefusalCategoryV1::CatalogLimitExceeded
    ));
    assert!(
        user_entries(&aggregate_capped).len() == 1,
        "aggregate cap must not masquerade as an entry or per-file cap"
    );
}

#[cfg(unix)]
#[test]
fn unsafe_posix_children_are_refused_without_following_or_blocking() {
    use std::process::Command;

    let directory = TestDirectory::new();
    directory.write("target.txt", FIRST_TEMPLATE.as_bytes());
    std::os::unix::fs::symlink("target.txt", directory.path().join("link.cdml"))
        .expect("symlink fixture");
    assert!(
        Command::new("mkfifo")
            .arg(directory.path().join("pipe.cdml"))
            .status()
            .expect("mkfifo must be installed on POSIX test hosts")
            .success(),
        "FIFO fixture must create"
    );
    let non_utf8_name = OsString::from_vec(b"\xff.cdml".to_vec());
    let raw_filename_supported =
        fs::write(directory.path().join(non_utf8_name), FIRST_TEMPLATE).is_ok();

    let snapshot =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("snapshot");
    assert!(has_refusal(
        &snapshot,
        TemplateCatalogRefusalCategoryV1::CandidateSymlink
    ));
    assert!(has_refusal(
        &snapshot,
        TemplateCatalogRefusalCategoryV1::CandidateNotRegular
    ));
    if raw_filename_supported {
        assert!(has_refusal(
            &snapshot,
            TemplateCatalogRefusalCategoryV1::FilenameNonUtf8
        ));
    }
    assert!(
        user_entries(&snapshot)
            .iter()
            .all(|entry| entry.label() != "First molecule")
    );
}

#[test]
fn missing_directory_is_an_empty_source_without_a_refusal() {
    let directory = TestDirectory::new();
    let missing = directory.path().join("does-not-exist");
    let snapshot = snapshot_template_catalog_v1(Some(&missing), limits())
        .expect("missing directory is normal");
    assert!(user_entries(&snapshot).is_empty());
    assert!(snapshot.refusals().is_empty());
}

#[test]
fn bounded_candidate_selection_is_lexical_and_enumeration_independent() {
    let directory = TestDirectory::new();
    for name in ["z.cdml", "b.cdml", "a.cdml", "y.cdml"] {
        directory.write(name, FIRST_TEMPLATE.as_bytes());
    }
    let snapshot = snapshot_template_catalog_v1(
        Some(directory.path()),
        TemplateCatalogLimitsV1::with_scan_limits(8, 2, 2, 8 * 1024, 16 * 1024),
    )
    .expect("bounded snapshot");
    let aliases: Vec<_> = user_entries(&snapshot)
        .iter()
        .flat_map(|entry| entry.aliases().iter().cloned())
        .collect();
    assert!(
        aliases
            .iter()
            .all(|name| name == "a.cdml" || name == "b.cdml")
    );
    assert!(snapshot.refusals().len() <= 2);
    let aggregate = snapshot
        .refusals()
        .iter()
        .find(|refusal| {
            refusal.category() == TemplateCatalogRefusalCategoryV1::CatalogLimitExceeded
        })
        .expect("aggregate limit fact");
    assert_eq!(aggregate.basename(), None);
    assert!(aggregate.occurrences() >= 2);
}

#[test]
fn snapshot_owned_selection_places_after_source_changes_and_rejects_stale_requests() {
    let directory = TestDirectory::new();
    directory.write("first.cdml", FIRST_TEMPLATE.as_bytes());
    let snapshot =
        snapshot_template_catalog_v1(Some(directory.path()), limits()).expect("snapshot");
    let user_key = user_entries(&snapshot)[0].key();
    directory.write("first.cdml", SECOND_TEMPLATE.as_bytes());
    fs::remove_file(directory.path().join("first.cdml")).expect("source removal");
    let mut session = DocumentSession::create_empty_document_v1().expect("empty session");
    let baseline = session.snapshot().expect("baseline");
    let placed = apply_template_catalog_entry_v1(
        &mut session,
        &snapshot,
        user_key,
        baseline.revision(),
        baseline.digest(),
        10.0,
        20.0,
    )
    .expect("old admitted snapshot remains placeable");
    assert_eq!(placed.source(), TemplateCatalogSourceV1::UserDirectory);
    let changed = session.snapshot().expect("changed");
    assert_ne!(changed.cdml(), baseline.cdml());
    let undone = session.undo(changed.revision()).expect("placement undoes");
    assert_eq!(undone.observation().snapshot().cdml(), baseline.cdml());
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("placement redoes");
    let reopened =
        DocumentSession::load(redone.observation().snapshot().cdml()).expect("placed CDML reopens");
    assert_eq!(
        reopened.snapshot().expect("reopened snapshot").cdml(),
        redone.observation().snapshot().cdml()
    );
    let before_refusal = session.snapshot().expect("before refusal");
    let stale = apply_template_catalog_entry_v1(
        &mut session,
        &snapshot,
        user_key,
        baseline.revision(),
        baseline.digest(),
        1.0,
        1.0,
    );
    assert!(matches!(
        stale,
        Err(TemplateCatalogApplyErrorV1::Session(_))
    ));
    assert_eq!(
        session
            .snapshot()
            .expect("stale request cannot mutate")
            .cdml(),
        before_refusal.cdml()
    );
    let correct_revision = before_refusal.revision();
    let mut wrong_digest = *before_refusal.digest();
    wrong_digest[0] ^= 1;
    let digest_refusal = apply_template_catalog_entry_v1(
        &mut session,
        &snapshot,
        user_key,
        correct_revision,
        &wrong_digest,
        1.0,
        1.0,
    );
    assert!(matches!(
        digest_refusal,
        Err(TemplateCatalogApplyErrorV1::Session(_))
    ));
    assert_eq!(
        session
            .snapshot()
            .expect("wrong digest cannot mutate")
            .cdml(),
        before_refusal.cdml()
    );
    assert!(
        snapshot
            .find_key_v1("user-template:v1:not-issued")
            .is_none()
    );
    let shipped_key = snapshot
        .entries()
        .iter()
        .find(|entry| entry.source() == TemplateCatalogSourceV1::Shipped)
        .expect("shipped entry")
        .key();
    let fence = session.snapshot().expect("fence");
    let shipped = apply_template_catalog_entry_v1(
        &mut session,
        &snapshot,
        shipped_key,
        fence.revision(),
        fence.digest(),
        40.0,
        50.0,
    )
    .expect("shipped entry places through same API");
    assert_eq!(shipped.source(), TemplateCatalogSourceV1::Shipped);
}
