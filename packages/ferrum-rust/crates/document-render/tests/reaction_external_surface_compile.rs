//! External-consumer proof for opaque reaction renderer capabilities.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

const OPAQUE_FLOW_SOURCE: &str =
    include_str!("fixtures/reaction_render_external_consumer/src/bin/opaque_flow.rs");
const LIFECYCLE_FLOW_SOURCE: &str = r#"
use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::{
    ReactionMembershipPatchRequestV1, RenderInteractionSessionV1,
    begin_reaction_membership_patch_v1, commit_reaction_lifecycle_v1,
    prepare_reaction_lifecycle_v1,
};
const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><molecule id=\"third\"><atom id=\"third-a\" name=\"N\"><point x=\"140\" y=\"0\"/></atom></molecule><arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
fn main() {
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let list = session.observe_reaction_list_v1(DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())).expect("list");
    let selection = session.select_reaction_v1(&list, "r").expect("selection");
    let request = ReactionMembershipPatchRequestV1::new(0, vec!["left".into()], vec!["third".into()], "a".into(), vec![], vec![]).expect("request");
    let gesture = begin_reaction_membership_patch_v1(&session, &selection, request).expect("begin");
    let mut prepared = prepare_reaction_lifecycle_v1(&mut session, &gesture).expect("prepare");
    commit_reaction_lifecycle_v1(&mut session, &mut prepared).expect("commit");
}
"#;
const FORGED_LIFECYCLE_SOURCE: &str = r#"
use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::{ReactionMembershipPatchRequestV1, RenderInteractionSessionV1, begin_reaction_membership_patch_v1};
fn main() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let fence = DocumentFenceV1::new(snapshot.revision(), *snapshot.digest());
    let request = ReactionMembershipPatchRequestV1::new(0, vec!["a".into()], vec!["b".into()], "c".into(), vec![], vec![]).expect("request");
    let _ = begin_reaction_membership_patch_v1(&session, fence, "r".into(), "digest".into(), request);
}
"#;
const CLONED_SELECTION_SOURCE: &str = r#"
use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::RenderInteractionSessionV1;
const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
fn main() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let list = session.observe_reaction_list_v1(DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())).expect("list");
    let selection = session.select_reaction_v1(&list, "r").expect("selection");
    let _copy = selection.clone();
}
"#;
const CLONED_LIFECYCLE_GESTURE_SOURCE: &str = r#"
use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::{ReactionMembershipPatchRequestV1, RenderInteractionSessionV1, begin_reaction_membership_patch_v1};
const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"left\"><atom id=\"left-a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\"><atom id=\"right-a\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"a\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
fn main() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let list = session.observe_reaction_list_v1(DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())).expect("list");
    let selection = session.select_reaction_v1(&list, "r").expect("selection");
    let request = ReactionMembershipPatchRequestV1::new(0, vec!["left".into()], vec!["right".into()], "a".into(), vec![], vec![]).expect("request");
    let gesture = begin_reaction_membership_patch_v1(&session, &selection, request).expect("begin");
    let _copy = gesture.clone();
}
"#;
const FORBIDDEN_SOURCES: &[(&str, &str)] = &[
    (
        "SMARTS prepared target construction",
        r#"use ferrum_document_render::PreparedSmartsTargetSetV1;
fn main() { let _ = PreparedSmartsTargetSetV1 {}; }"#,
    ),
    (
        "SMARTS prepared target debug extraction",
        r#"use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::RenderInteractionSessionV1;
fn main() { let session = RenderInteractionSessionV1::new(DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").unwrap()); let snapshot = session.snapshot().unwrap(); let value = session.prepare_smarts_targets_v1(DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())).unwrap(); let _ = format!("{:?}", value); }"#,
    ),
    (
        "SMARTS prepared target clone",
        r#"use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::RenderInteractionSessionV1;
fn main() { let session = RenderInteractionSessionV1::new(DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").unwrap()); let snapshot = session.snapshot().unwrap(); let value = session.prepare_smarts_targets_v1(DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())).unwrap(); let _ = value.clone(); }"#,
    ),
    (
        "SMARTS prepared target graph extraction",
        r#"use ferrum_document::{DocumentFenceV1, DocumentSession};
use ferrum_document_render::RenderInteractionSessionV1;
fn main() { let session = RenderInteractionSessionV1::new(DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").unwrap()); let snapshot = session.snapshot().unwrap(); let value = session.prepare_smarts_targets_v1(DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())).unwrap(); let _ = value.graph_v1(0); }"#,
    ),
    (
        "reaction gesture construction",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/construct_gesture.rs"),
    ),
    (
        "reaction preview import",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/import_preview.rs"),
    ),
    (
        "reaction renderer receipt import",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/import_receipt.rs"),
    ),
    (
        "reaction gesture clone",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/clone_gesture.rs"),
    ),
    (
        "prepared reaction clone",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/clone_prepared.rs"),
    ),
    (
        "reaction gesture dereference",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/deref_gesture.rs"),
    ),
    (
        "reaction gesture serialization",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/serialize_gesture.rs"),
    ),
    (
        "prepared reaction conversion",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/convert_prepared.rs"),
    ),
    (
        "prepared candidate CDML extraction",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/extract_candidate.rs"),
    ),
    (
        "prepared render plan extraction",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/extract_plan.rs"),
    ),
    (
        "prepared renderer receipt extraction",
        include_str!("fixtures/reaction_render_external_consumer/src/bin/extract_receipt.rs"),
    ),
];
static NEXT_TEMPORARY_CONSUMER: AtomicU64 = AtomicU64::new(0);

/// A standalone package makes Cargo enforce the public document-render boundary.
struct TemporaryConsumer {
    root: PathBuf,
}

impl TemporaryConsumer {
    fn new() -> Self {
        let sequence = NEXT_TEMPORARY_CONSUMER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "ferrum-reaction-render-surface-{}-{sequence}",
            std::process::id(),
        ));
        std::fs::create_dir_all(root.join("src"))
            .expect("test-owned external consumer must be creatable");
        let document_render_path = Path::new(env!("CARGO_MANIFEST_DIR"));
        let document_path = document_render_path
            .parent()
            .expect("document-render crate has a crates parent")
            .join("document");
        let manifest = format!(
            concat!(
                "[package]\n",
                "name = \"external_consumer\"\n",
                "version = \"0.0.0\"\n",
                "edition = \"2024\"\n",
                "publish = false\n\n",
                "[workspace]\n\n",
                "[dependencies]\n",
                "ferrum-document-render = {{ path = \"{}\" }}\n",
                "ferrum-document = {{ path = \"{}\" }}\n",
                "serde_json = \"*\"\n",
            ),
            document_render_path.display(),
            document_path.display(),
        );
        std::fs::write(root.join("Cargo.toml"), manifest)
            .expect("external consumer manifest must be writable");
        Self { root }
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
fn external_consumers_can_complete_the_supported_opaque_reaction_flow() {
    let consumer = TemporaryConsumer::new();
    consumer.write_main(OPAQUE_FLOW_SOURCE);
    let status = cargo_check(&consumer.manifest(), &consumer.target());

    assert!(
        status.success(),
        "an external consumer must compile begin, prepare, diagnostic, and commit reaction flow"
    );
}

#[test]
fn external_consumers_can_only_begin_lifecycle_from_an_issued_selection() {
    let consumer = TemporaryConsumer::new();
    consumer.write_main(LIFECYCLE_FLOW_SOURCE);
    assert!(cargo_check(&consumer.manifest(), &consumer.target()).success());
    consumer.write_main(FORGED_LIFECYCLE_SOURCE);
    assert!(!cargo_check(&consumer.manifest(), &consumer.target()).success());
    consumer.write_main(CLONED_SELECTION_SOURCE);
    assert!(!cargo_check(&consumer.manifest(), &consumer.target()).success());
    consumer.write_main(CLONED_LIFECYCLE_GESTURE_SOURCE);
    assert!(!cargo_check(&consumer.manifest(), &consumer.target()).success());
}

#[test]
fn external_consumers_cannot_extract_or_forge_reaction_renderer_authority() {
    let consumer = TemporaryConsumer::new();

    for (description, source) in FORBIDDEN_SOURCES {
        consumer.write_main(source);
        let status = cargo_check(&consumer.manifest(), &consumer.target());
        assert!(
            !status.success(),
            "external consumer {description} unexpectedly compiled against opaque reaction authority"
        );
    }
}
