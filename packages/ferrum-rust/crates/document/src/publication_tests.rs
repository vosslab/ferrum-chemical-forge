use std::fs;

use super::{
    DocumentSessionError, PublicationDurability,
    publication::publish_snapshot_with_after_parent_open,
};

#[cfg(unix)]
#[test]
fn publication_remains_in_the_opened_parent_after_path_replacement() {
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("temporary root must resolve without a symbolic link")
        .join(format!(
            "ferrum-document-descriptor-publication-{}",
            std::process::id()
        ));
    let opened_parent = root.join("opened");
    let replacement_parent = root.join("replacement");
    let displaced_parent = root.join("displaced");
    fs::create_dir_all(&opened_parent).expect("opened parent must be creatable");
    fs::create_dir(&replacement_parent).expect("replacement parent must be creatable");
    let target = opened_parent.join("saved.cdml");

    let result = publish_snapshot_with_after_parent_open(&target, "<cdml/>", || {
        fs::rename(&opened_parent, &displaced_parent)
            .expect("opened directory must move out of the visible path");
        fs::rename(&replacement_parent, &opened_parent)
            .expect("replacement directory must take the visible path");
    });

    assert!(matches!(
        result,
        Ok(PublicationDurability::Confirmed) | Ok(PublicationDurability::DirectoryEntryUnconfirmed)
    ));
    assert_eq!(
        fs::read_to_string(displaced_parent.join("saved.cdml"))
            .expect("opened directory must receive the replacement"),
        "<cdml/>"
    );
    assert!(!opened_parent.join("saved.cdml").exists());
    fs::remove_dir_all(root).expect("test directory cleanup must succeed");
}

#[test]
fn missing_parent_is_a_pre_replacement_error() {
    let path = std::env::temp_dir()
        .canonicalize()
        .expect("temporary root must resolve without a symbolic link")
        .join("ferrum-document-missing-parent")
        .join("saved.cdml");
    let error = publish_snapshot_with_after_parent_open(&path, "<cdml/>", || {})
        .expect_err("missing parent must stop publication");
    assert!(matches!(
        error,
        DocumentSessionError::PublishNotStarted { .. }
    ));
}

#[test]
fn publication_durability_is_explicit() {
    let outcome = PublicationDurability::DirectoryEntryUnconfirmed;
    assert_ne!(outcome, PublicationDurability::Confirmed);
}

#[test]
fn publication_accepts_a_concrete_nested_parent_chain() {
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("temporary root must resolve without a symbolic link")
        .join(format!(
            "ferrum-document-nested-publication-{}",
            std::process::id()
        ));
    let nested = root.join("one").join("two");
    fs::create_dir_all(&nested).expect("nested parent must create");
    let target = nested.join("saved.cdml");

    let outcome = publish_snapshot_with_after_parent_open(&target, "<cdml/>", || {})
        .expect("concrete nested parent must publish");
    assert!(matches!(
        outcome,
        PublicationDurability::Confirmed | PublicationDurability::DirectoryEntryUnconfirmed
    ));
    assert_eq!(
        fs::read_to_string(&target).expect("published nested file must read"),
        "<cdml/>"
    );
    fs::remove_dir_all(root).expect("test directory cleanup must succeed");
}

#[cfg(unix)]
#[test]
fn final_entry_swap_is_rejected_through_the_retained_directory_descriptor() {
    let root = std::env::temp_dir()
        .canonicalize()
        .expect("temporary root must resolve without a symbolic link")
        .join(format!("ferrum-document-final-swap-{}", std::process::id()));
    fs::create_dir(&root).expect("test root must create");
    let target = root.join("saved.cdml");
    let outside = root.join("outside.cdml");
    fs::write(&target, "original").expect("target fixture must write");
    fs::write(&outside, "outside").expect("outside fixture must write");

    let error = publish_snapshot_with_after_parent_open(&target, "<cdml/>", || {
        fs::remove_file(&target).expect("target fixture must remove");
        std::os::unix::fs::symlink(&outside, &target).expect("final symbolic link must create");
    })
    .expect_err("final symbolic-link swap must be rejected");
    assert!(matches!(
        error,
        DocumentSessionError::InvalidDestination { .. }
    ));
    assert_eq!(
        fs::read_to_string(&outside).expect("outside must remain readable"),
        "outside"
    );
    fs::remove_dir_all(root).expect("test directory cleanup must succeed");
}
