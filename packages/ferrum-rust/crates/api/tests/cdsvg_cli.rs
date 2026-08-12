use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";
static NEXT_TEMPORARY_PATH: AtomicU64 = AtomicU64::new(0);

#[test]
fn extracts_verified_payload_from_stdin_to_stdout() {
    let output = run_with_stdin(
        ["cdml", "extract-cdsvg", "-", "--output", "-"],
        canonical_cdsvg().as_bytes(),
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let extracted = String::from_utf8(output.stdout).expect("CDML output is UTF-8");
    ferrum_document::TypedDocument::parse(&extracted).expect("extracted output reparses");
    assert!(extracted.contains("<paper"));
}

#[test]
fn rejects_absent_multiple_and_wrong_namespace_payloads() {
    for source in [
        r#"<svg xmlns="http://www.w3.org/2000/svg"><text>no payload</text></svg>"#.to_owned(),
        format!(
            concat!(
                r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml xmlns="{CDML_NAMESPACE}"/>"#,
                r#"<cdml xmlns="{CDML_NAMESPACE}"/></svg>"#,
            ),
            CDML_NAMESPACE = CDML_NAMESPACE
        ),
        r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml><paper/></cdml></svg>"#.to_owned(),
    ] {
        let output = run_with_stdin(
            ["cdml", "extract-cdsvg", "-", "--output", "-"],
            source.as_bytes(),
        );

        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("standard input"));
    }
}

#[test]
fn invalid_cdsvg_does_not_replace_destination() {
    let output_path = temporary_path("preserved.cdml");
    fs::write(&output_path, "original contents").expect("write initial destination");

    let output = run_with_stdin_and_arguments(
        [
            "cdml",
            "extract-cdsvg",
            "-",
            "--output",
            output_path.to_str().expect("temporary path is UTF-8"),
        ],
        b"<svg xmlns=\"http://www.w3.org/2000/svg\"><text>no payload</text></svg>",
    );

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(&output_path).expect("read preserved destination"),
        "original contents"
    );
    fs::remove_file(output_path).expect("remove test destination");
}

#[test]
fn file_output_is_atomically_replaced_after_extraction() {
    let input_path = temporary_path("input.svg");
    let output_path = temporary_path("output.cdml");
    fs::write(&input_path, canonical_cdsvg()).expect("write CD-SVG input");
    fs::write(&output_path, "old destination").expect("write initial destination");

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "cdml",
            "extract-cdsvg",
            input_path.to_str().expect("temporary path is UTF-8"),
            "--output",
            output_path.to_str().expect("temporary path is UTF-8"),
        ])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    let extracted = fs::read_to_string(&output_path).expect("read published output");
    ferrum_document::TypedDocument::parse(&extracted).expect("published output reparses");
    fs::remove_file(input_path).expect("remove test input");
    fs::remove_file(output_path).expect("remove test output");
}

fn canonical_cdsvg() -> String {
    format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><g><cdml xmlns="{CDML_NAMESPACE}" "#,
            r#"version="0.16"><paper/></cdml></g></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE
    )
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
        .expect("write CD-SVG input");
    child.wait_with_output().expect("collect ferrum output")
}

fn temporary_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMPORARY_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .canonicalize()
        .expect("temporary directory must resolve without a symbolic link")
        .join(format!(
            "ferrum-api-cdsvg-cli-{}-{sequence}-{name}",
            std::process::id()
        ))
}
