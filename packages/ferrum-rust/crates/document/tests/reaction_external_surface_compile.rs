//! External-consumer proof for the reaction authoring ownership boundary.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const GENERIC_COMPATIBILITY_SOURCE: &str =
    include_str!("fixtures/reaction_external_consumer/src/bin/generic_compatibility.rs");
const FORBIDDEN_REACTION_SOURCES: &[(&str, &str)] = &[
    (
        "reaction type imports",
        include_str!("fixtures/reaction_external_consumer/src/bin/forbidden_reaction_types.rs"),
    ),
    (
        "reaction candidate preparation",
        include_str!("fixtures/reaction_external_consumer/src/bin/forbidden_reaction_prepare.rs"),
    ),
    (
        "renderer-admitted reaction commit",
        include_str!("fixtures/reaction_external_consumer/src/bin/forbidden_reaction_commit.rs"),
    ),
];
static NEXT_TEMPORARY_CONSUMER: AtomicU64 = AtomicU64::new(0);

/// A standalone package makes Cargo enforce the public crate boundary.
struct TemporaryConsumer {
    root: PathBuf,
}

impl TemporaryConsumer {
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY_CONSUMER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ferrum-document-reaction-surface-{}-{sequence}",
            std::process::id(),
        ));
        std::fs::create_dir_all(path.join("src"))
            .expect("test-owned external consumer must be creatable");
        let document_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest = format!(
            concat!(
                "[package]\n",
                "name = \"external_consumer\"\n",
                "version = \"0.0.0\"\n",
                "edition = \"2024\"\n",
                "publish = false\n\n",
                "[workspace]\n\n",
                "[dependencies]\n",
                "ferrum-document = {{ path = \"{}\" }}\n",
            ),
            document_path.display(),
        );
        std::fs::write(path.join("Cargo.toml"), manifest)
            .expect("external consumer manifest must be writable");
        Self { root: path }
    }

    fn write_main(&self, source: &str) {
        std::fs::write(self.root.join("src/main.rs"), source)
            .expect("external consumer source must be writable");
    }

    fn manifest(&self) -> PathBuf {
        self.root.join("Cargo.toml")
    }

    fn target(&self) -> PathBuf {
        self.root.join("target")
    }
}

impl Drop for TemporaryConsumer {
    fn drop(&mut self) {
        let _ignored = std::fs::remove_dir_all(&self.root);
    }
}

fn cargo_check(manifest: &Path, target: &Path) -> std::process::ExitStatus {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(["check", "--quiet", "--manifest-path"])
        .arg(manifest)
        .args(["--bin", "external_consumer"])
        .env("CARGO_TARGET_DIR", target)
        .output()
        .expect("external consumer Cargo check must launch")
        .status
}

#[test]
fn external_consumers_can_use_only_the_generic_complete_cdml_session_transaction() {
    let consumer = TemporaryConsumer::new();
    consumer.write_main(GENERIC_COMPATIBILITY_SOURCE);
    let status = cargo_check(&consumer.manifest(), &consumer.target());

    assert!(
        status.success(),
        "an external consumer must compile the documented generic CDML session transaction"
    );
}

#[test]
fn external_consumers_cannot_import_or_call_document_reaction_authoring_apis() {
    let consumer = TemporaryConsumer::new();

    for (description, source) in FORBIDDEN_REACTION_SOURCES {
        consumer.write_main(source);
        let status = cargo_check(&consumer.manifest(), &consumer.target());
        assert!(
            !status.success(),
            "external consumer {description} unexpectedly compiled against removed document reaction authority"
        );
    }
}
