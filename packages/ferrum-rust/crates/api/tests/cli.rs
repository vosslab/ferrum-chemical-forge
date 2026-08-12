use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const SIMPLE_CDML: &str = r#"<cdml version="0.16"><molecule id="m1"><atom id="a1" name="C"><point x="1" y="2"/></atom></molecule></cdml>"#;
const OPAQUE_CDML: &str =
    r#"<cdml xmlns:q="urn:test"><q:payload id="foreign"><q:item flag="yes"/></q:payload></cdml>"#;
static NEXT_TEMPORARY_PATH: AtomicU64 = AtomicU64::new(0);

#[test]
fn inspect_from_stdin_emits_versioned_json_and_keeps_stderr_clean() {
    let output = run_with_stdin(["cdml", "inspect", "-"], SIMPLE_CDML.as_bytes());

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.last(), Some(&b'\n'));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(report["schema"], "ferrum-cdml-inspection-v1");
    assert_eq!(report["persistent_id_count"], 2);
    assert!(report["molecules"].is_array());
}

#[test]
fn validate_separates_structural_and_core_contracts() {
    let structural = run_with_stdin(["cdml", "validate", "-"], OPAQUE_CDML.as_bytes());
    let core = run_with_stdin(["cdml", "validate", "-", "--typed"], OPAQUE_CDML.as_bytes());

    assert!(structural.status.success());
    assert!(structural.stderr.is_empty());
    let report: serde_json::Value =
        serde_json::from_slice(&structural.stdout).expect("JSON report");
    assert_eq!(report["schema"], "ferrum-cdml-validation-v1");
    assert_eq!(report["valid"], true);
    assert_eq!(report["level"], "structural");
    assert!(core.status.success());
    let core_report: serde_json::Value = serde_json::from_slice(&core.stdout).expect("JSON report");
    assert_eq!(core_report["level"], "core");
}

#[test]
fn rewrite_stdout_reparses_and_preserves_opaque_payload() {
    let output = run_with_stdin(
        ["cdml", "rewrite", "-", "--output", "-"],
        OPAQUE_CDML.as_bytes(),
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let rewritten = String::from_utf8(output.stdout).expect("CDML output is UTF-8");
    ferrum_document::TypedDocument::parse(&rewritten).expect("stdout reparses as CDML");
    assert!(rewritten.contains("q:payload"));
    assert!(rewritten.contains("flag=\"yes\""));
}

#[test]
fn rewrite_check_emits_a_single_versioned_result_without_writing() {
    let output = run_with_stdin(["cdml", "rewrite", "-", "--check"], OPAQUE_CDML.as_bytes());

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    assert_eq!(report["schema"], "ferrum-cdml-rewrite-check-v1");
    assert_eq!(report["valid"], true);
    assert_eq!(report["opaque_child_count"], 1);
}

#[test]
fn malformed_input_fails_without_polluting_stdout() {
    let output = run_with_stdin(["cdml", "validate", "-"], b"<cdml>");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("standard input"));
}

#[test]
fn rewrite_does_not_replace_destination_when_input_is_invalid() {
    let output_path = temporary_path("preserved.cdml");
    fs::write(&output_path, "original contents").expect("write initial destination");
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "cdml",
            "rewrite",
            "-",
            "--output",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .stdin(Stdio::piped())
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(&output_path).expect("read preserved destination"),
        "original contents"
    );
    fs::remove_file(output_path).expect("remove test destination");
}

#[test]
fn rewrite_accepts_the_same_input_and_output_path_after_validation() {
    let document_path = temporary_path("same-path.cdml");
    fs::write(&document_path, OPAQUE_CDML).expect("write source document");
    let path = document_path.to_str().expect("temporary path is UTF-8");

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["cdml", "rewrite", path, "--output", path])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    let rewritten = fs::read_to_string(&document_path).expect("read rewritten document");
    ferrum_document::TypedDocument::parse(&rewritten).expect("same-path output reparses");
    fs::remove_file(document_path).expect("remove rewritten document");
}

#[cfg(unix)]
#[test]
fn rewrite_rejects_a_symbolic_link_destination_without_touching_its_target() {
    let target = temporary_path("link-target.cdml");
    let destination = temporary_path("link-destination.cdml");
    fs::write(&target, "original target").expect("write link target");
    std::os::unix::fs::symlink(&target, &destination).expect("create destination link");

    let output = run_with_stdin_and_arguments(
        [
            "cdml",
            "rewrite",
            "-",
            "--output",
            destination.to_str().expect("temporary path is UTF-8"),
        ],
        SIMPLE_CDML.as_bytes(),
    );

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(&target).expect("read link target"),
        "original target"
    );
    assert!(
        fs::symlink_metadata(&destination)
            .expect("read destination metadata")
            .file_type()
            .is_symlink()
    );
    fs::remove_file(destination).expect("remove destination link");
    fs::remove_file(target).expect("remove link target");
}

#[test]
fn rewrite_rejects_a_nonregular_destination_without_modifying_it() {
    let destination = temporary_path("destination-directory");
    fs::create_dir(&destination).expect("create directory destination");

    let output = run_with_stdin_and_arguments(
        [
            "cdml",
            "rewrite",
            "-",
            "--output",
            destination.to_str().expect("temporary path is UTF-8"),
        ],
        SIMPLE_CDML.as_bytes(),
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(destination.is_dir());
    fs::remove_dir(destination).expect("remove directory destination");
}

#[test]
fn usage_quick_start_document_is_a_runnable_repository_root_path() {
    let repository_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let quick_start = repository_root.join("tests/e2e/corpus/authored_document_forms.cdml");
    assert!(quick_start.is_file());

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "cdml",
            "inspect",
            quick_start.to_str().expect("repository path is UTF-8"),
        ])
        .output()
        .expect("run documented quick start");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    assert_eq!(report["schema"], "ferrum-cdml-inspection-v1");
}

#[test]
fn malformed_argument_combinations_use_argument_error_exit() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["cdml", "rewrite", "-", "--check", "--output", "-"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}

fn run_with_stdin<const N: usize>(arguments: [&str; N], source: &[u8]) -> std::process::Output {
    run_with_stdin_and_arguments(arguments, source)
}

fn run_with_stdin_and_arguments<const N: usize>(
    arguments: [&str; N],
    source: &[u8],
) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start ferrum CLI");
    child
        .stdin
        .take()
        .expect("stdin pipe exists")
        .write_all(source)
        .expect("write CDML input");
    child.wait_with_output().expect("collect ferrum output")
}

fn temporary_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMPORARY_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .canonicalize()
        .expect("temporary directory must resolve without a symbolic link")
        .join(format!(
            "ferrum-api-cli-{}-{sequence}-{name}",
            std::process::id()
        ))
}
