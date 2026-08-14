use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use super::artifact_publication_v1::{
    ArtifactDestinationRejectionV1, ArtifactPrepublicationPhaseV1, ArtifactPublicationDurabilityV1,
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    publish_artifact_v1, publish_artifact_with_test_seams_v1, retain_regular_source_file_v1,
};

static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);

fn test_directory(label: &str) -> PathBuf {
    let serial = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir()
        .canonicalize()
        .expect("temporary root must resolve")
        .join(format!(
            "ferrum-artifact-publication-{label}-{}-{serial}",
            std::process::id()
        ));
    fs::create_dir(&directory).expect("test directory must create");
    directory
}

#[test]
fn owned_bytes_publish_with_a_read_only_receipt() {
    let directory = test_directory("owned");
    let destination = directory.join("artifact.svg");
    let request = ArtifactPublicationRequestV1::new(destination.clone(), b"<svg/>".to_vec());
    let debug = format!("{request:?}");
    assert!(debug.contains("byte_len"));
    assert!(!debug.contains("<svg/>"));

    let outcome = publish_artifact_v1(request).expect("publication must complete");
    assert_eq!(outcome.receipt().destination(), destination);
    assert_eq!(outcome.receipt().retained_source_identity(), None);
    assert!(matches!(
        outcome,
        ArtifactPublicationOutcomeV1::ConfirmedDurable(_)
            | ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(_)
    ));
    assert_eq!(
        fs::read(&destination).expect("artifact must read"),
        b"<svg/>"
    );
    fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
}

#[cfg(unix)]
#[test]
fn observed_direct_and_hardlink_aliases_are_refused_before_temporary_creation() {
    let directory = test_directory("aliases");
    let source = directory.join("source.svg");
    fs::write(&source, "source").expect("source fixture must write");
    let direct = retain_regular_source_file_v1(File::open(&source).expect("source must open"))
        .expect("regular source must retain");
    let direct_error = publish_artifact_v1(
        ArtifactPublicationRequestV1::new(source.clone(), b"replacement".to_vec())
            .with_retained_source(direct),
    )
    .expect_err("direct observed alias must refuse publication");
    assert!(matches!(
        direct_error,
        ArtifactPublicationErrorV1::RejectedDestination {
            reason: ArtifactDestinationRejectionV1::SourceAliasesDestination,
            ..
        }
    ));

    let hard_link = directory.join("hard-link.svg");
    fs::hard_link(&source, &hard_link).expect("hard link fixture must create");
    let guard = retain_regular_source_file_v1(File::open(&source).expect("source must open"))
        .expect("regular source must retain");
    let hard_link_error = publish_artifact_v1(
        ArtifactPublicationRequestV1::new(hard_link, b"replacement".to_vec())
            .with_retained_source(guard),
    )
    .expect_err("hard-link observed alias must refuse publication");
    assert!(matches!(
        hard_link_error,
        ArtifactPublicationErrorV1::RejectedDestination {
            reason: ArtifactDestinationRejectionV1::SourceAliasesDestination,
            ..
        }
    ));

    let source_link = directory.join("source-link.svg");
    std::os::unix::fs::symlink(&source, &source_link).expect("source symlink must create");
    let symlink_guard = retain_regular_source_file_v1(
        File::open(&source_link).expect("target must open through link"),
    )
    .expect("resolved regular source must retain");
    let symlink_target_error = publish_artifact_v1(
        ArtifactPublicationRequestV1::new(source.clone(), b"replacement".to_vec())
            .with_retained_source(symlink_guard),
    )
    .expect_err("resolved source target alias must refuse publication");
    assert!(matches!(
        symlink_target_error,
        ArtifactPublicationErrorV1::RejectedDestination {
            reason: ArtifactDestinationRejectionV1::SourceAliasesDestination,
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(source).expect("source must survive"),
        "source"
    );
    fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
}

#[cfg(unix)]
#[test]
fn observed_alias_created_before_rename_is_refused_without_replacing_source() {
    let directory = test_directory("late-alias");
    let source = directory.join("source.svg");
    let destination = directory.join("artifact.svg");
    fs::write(&source, "source").expect("source fixture must write");
    let guard = retain_regular_source_file_v1(File::open(&source).expect("source must open"))
        .expect("regular source must retain");
    let hook_destination = destination.clone();
    let hook_source = source.clone();
    let error = publish_artifact_with_test_seams_v1(
        ArtifactPublicationRequestV1::new(destination.clone(), b"replacement".to_vec())
            .with_retained_source(guard),
        move |phase| {
            if phase == ArtifactPrepublicationPhaseV1::ValidateBeforeRename {
                fs::hard_link(&hook_source, &hook_destination)
                    .expect("late observed hard-link fixture must create");
            }
        },
        |_| Ok(ArtifactPublicationDurabilityV1::Confirmed),
    )
    .expect_err("late observed alias must refuse publication");
    assert!(matches!(
        error,
        ArtifactPublicationErrorV1::RejectedDestination {
            reason: ArtifactDestinationRejectionV1::SourceAliasesDestination,
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(source).expect("source must remain"),
        "source"
    );
    assert_eq!(
        fs::read_to_string(destination).expect("hard link must remain"),
        "source"
    );
    assert!(
        !fs::read_dir(&directory)
            .expect("directory must read")
            .filter_map(Result::ok)
            .any(|entry| entry.file_name().to_string_lossy().starts_with(".ferrum-")),
        "rejected pre-rename publication must leave no Ferrum temporary"
    );
    fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
}

#[cfg(unix)]
#[test]
fn cleanup_failure_preserves_typed_rejection_and_io_taxonomies() {
    use std::os::unix::fs::PermissionsExt;

    let rejected_directory = test_directory("rejection-cleanup");
    let rejected_destination = rejected_directory.join("artifact.svg");
    let rejected_hook_directory = rejected_directory.clone();
    let rejected_hook_destination = rejected_destination.clone();
    let rejection = publish_artifact_with_test_seams_v1(
        ArtifactPublicationRequestV1::new(rejected_destination, b"replacement".to_vec()),
        move |phase| {
            if phase == ArtifactPrepublicationPhaseV1::ValidateBeforeRename {
                std::os::unix::fs::symlink("outside.svg", &rejected_hook_destination)
                    .expect("late symlink must create");
                fs::set_permissions(&rejected_hook_directory, fs::Permissions::from_mode(0o500))
                    .expect("directory must become non-writable");
            }
        },
        |_| Ok(ArtifactPublicationDurabilityV1::Confirmed),
    )
    .expect_err("typed rejection plus failed cleanup must remain typed");
    fs::set_permissions(&rejected_directory, fs::Permissions::from_mode(0o700))
        .expect("directory permissions must restore");
    assert!(matches!(
        rejection,
        ArtifactPublicationErrorV1::RejectedDestinationTemporaryMayRemain {
            reason: ArtifactDestinationRejectionV1::FinalIsSymlink,
            ..
        }
    ));
    fs::remove_dir_all(rejected_directory).expect("test directory cleanup must succeed");

    let io_directory = test_directory("io-cleanup");
    let io_destination = io_directory.join("artifact.svg");
    let io_hook_directory = io_directory.clone();
    let io_error = publish_artifact_with_test_seams_v1(
        ArtifactPublicationRequestV1::new(io_destination, b"replacement".to_vec()),
        move |phase| {
            if phase == ArtifactPrepublicationPhaseV1::Rename {
                fs::set_permissions(&io_hook_directory, fs::Permissions::from_mode(0o500))
                    .expect("directory must become non-writable");
            }
        },
        |_| Ok(ArtifactPublicationDurabilityV1::Confirmed),
    )
    .expect_err("rename failure plus cleanup failure must retain I/O taxonomy");
    fs::set_permissions(&io_directory, fs::Permissions::from_mode(0o700))
        .expect("directory permissions must restore");
    assert!(matches!(
        io_error,
        ArtifactPublicationErrorV1::NotPublishedTemporaryMayRemain {
            phase: ArtifactPrepublicationPhaseV1::Rename,
            ..
        }
    ));
    fs::remove_dir_all(io_directory).expect("test directory cleanup must succeed");
}

#[cfg(unix)]
#[test]
fn final_symlink_is_refused_and_replacement_does_not_follow_it() {
    let directory = test_directory("symlink");
    let outside = directory.join("outside.svg");
    let destination = directory.join("artifact.svg");
    fs::write(&outside, "outside").expect("outside fixture must write");
    std::os::unix::fs::symlink(&outside, &destination).expect("symlink fixture must create");
    let error = publish_artifact_v1(ArtifactPublicationRequestV1::new(
        destination,
        b"replacement".to_vec(),
    ))
    .expect_err("final symlink must refuse publication");
    assert!(matches!(
        error,
        ArtifactPublicationErrorV1::RejectedDestination {
            reason: ArtifactDestinationRejectionV1::FinalIsSymlink,
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(outside).expect("outside must remain"),
        "outside"
    );
    fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
}

#[cfg(unix)]
#[test]
fn replacement_uses_private_new_file_mode_not_old_metadata() {
    use std::os::unix::fs::PermissionsExt;

    let directory = test_directory("mode");
    let destination = directory.join("artifact.svg");
    fs::write(&destination, "old").expect("old fixture must write");
    fs::set_permissions(&destination, fs::Permissions::from_mode(0o644))
        .expect("old fixture mode must set");
    publish_artifact_v1(ArtifactPublicationRequestV1::new(
        destination.clone(),
        b"new".to_vec(),
    ))
    .expect("replacement must complete");
    assert_eq!(
        fs::read(&destination).expect("replacement must read"),
        b"new"
    );
    assert_eq!(
        fs::metadata(&destination)
            .expect("replacement metadata must read")
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
}
