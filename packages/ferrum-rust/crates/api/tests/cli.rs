use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../tests/e2e/corpus")
        .join(name)
}

#[test]
fn inspect_emits_json_and_keeps_stderr_clean() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "cdml",
            "inspect",
            fixture_path("authored_document_forms.cdml")
                .to_str()
                .expect("fixture path is UTF-8"),
        ])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout is JSON");
    assert_eq!(report["schema"], "ferrum-cdml-inspection-v1");
    assert!(report["molecules"].is_array());
}

#[test]
fn rewrite_stdout_reparses_as_cdml() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "cdml",
            "rewrite",
            fixture_path("opaque_namespace_preservation.cdml")
                .to_str()
                .expect("fixture path is UTF-8"),
            "--output",
            "-",
        ])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let rewritten = String::from_utf8(output.stdout).expect("CDML output is UTF-8");
    ferrum_document::TypedDocument::parse(&rewritten).expect("stdout reparses as CDML");
}

#[test]
fn malformed_stdin_fails_without_polluting_stdout() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["cdml", "inspect", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start ferrum CLI");
    child
        .stdin
        .take()
        .expect("stdin pipe exists")
        .write_all(b"<cdml>")
        .expect("write malformed input");
    let output = child.wait_with_output().expect("collect ferrum output");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("standard input"));
}

#[test]
fn missing_subcommand_uses_argument_error_exit() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Usage:"));
}
