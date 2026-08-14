use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

const CDML: &str = concat!(
    "<cdml><molecule id=\"m\">",
    "<atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom>",
    "</molecule></cdml>",
);
static NEXT_TEMPORARY_PATH: AtomicU64 = AtomicU64::new(0);

#[test]
fn render_svg_from_bounded_stdin_emits_one_structurally_valid_artifact() {
    let output = run_with_stdin(
        ["cdml", "render", "svg", "-", "--output", "-"],
        CDML.as_bytes(),
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let svg = String::from_utf8(output.stdout).expect("SVG is UTF-8");
    let mut tree = xot::Xot::new();
    tree.parse(&svg).expect("stdout is structurally valid XML");
    assert!(svg.starts_with("<svg "));
    assert!(svg.contains("data-ferrum-source-order"));
}

#[test]
fn render_pdf_from_bounded_stdin_emits_native_vector_pdf_bytes() {
    let output = run_with_stdin(
        ["cdml", "render", "pdf", "-", "--output", "-"],
        CDML.as_bytes(),
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(output.stdout.starts_with(b"%PDF-"));
    assert!(output.stdout.trim_ascii_end().ends_with(b"%%EOF"));
    assert!(!output.stdout.windows(4).any(|window| window == b"<svg"));
}

#[test]
fn render_png_from_bounded_stdin_emits_the_requested_native_raster_size() {
    let output = run_with_stdin(
        [
            "cdml", "render", "png", "-", "--output", "-", "--width", "17", "--height", "13",
        ],
        CDML.as_bytes(),
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(output.stdout.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(&output.stdout[12..16], b"IHDR");
    assert_eq!(
        u32::from_be_bytes(output.stdout[16..20].try_into().expect("PNG width bytes")),
        17
    );
    assert_eq!(
        u32::from_be_bytes(output.stdout[20..24].try_into().expect("PNG height bytes")),
        13
    );
}

#[test]
fn render_svg_file_publication_preserves_the_source_document() {
    let source = temporary_path("source.cdml");
    let destination = temporary_path("artifact.svg");
    fs::write(&source, CDML).expect("write source");

    let output = run_with_paths(&source, &destination, &[]);

    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    assert_eq!(fs::read_to_string(&source).expect("read source"), CDML);
    let svg = fs::read_to_string(&destination).expect("read SVG artifact");
    xot::Xot::new().parse(&svg).expect("artifact is valid XML");
    fs::remove_file(source).expect("remove source");
    fs::remove_file(destination).expect("remove artifact");
}

#[test]
fn render_svg_refuses_the_source_as_destination_without_modifying_it() {
    let source = temporary_path("same-path.cdml");
    fs::write(&source, CDML).expect("write source");

    let output = run_with_paths(&source, &source, &[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("SourceAliasesDestination"));
    assert_eq!(fs::read_to_string(&source).expect("read source"), CDML);
    fs::remove_file(source).expect("remove source");
}

#[test]
fn render_svg_output_limit_preserves_an_existing_destination() {
    let source = temporary_path("limited-source.cdml");
    let destination = temporary_path("preserved.svg");
    fs::write(&source, CDML).expect("write source");
    fs::write(&destination, "preserved").expect("write destination");

    let output = run_with_paths(&source, &destination, &["--max-output-bytes", "1"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("completed-artifact cap"));
    assert_eq!(
        fs::read_to_string(&destination).expect("read destination"),
        "preserved"
    );
    fs::remove_file(source).expect("remove source");
    fs::remove_file(destination).expect("remove destination");
}

#[test]
fn render_pdf_complexity_limit_preserves_an_existing_destination() {
    let source = temporary_path("limited-pdf-source.cdml");
    let destination = temporary_path("preserved.pdf");
    fs::write(&source, CDML).expect("write source");
    fs::write(&destination, "preserved").expect("write destination");

    let output = run_render_with_paths("pdf", &source, &destination, &["--max-path-commands", "1"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("DrawPathCommands complexity"));
    assert_eq!(
        fs::read_to_string(&destination).expect("read destination"),
        "preserved"
    );
    fs::remove_file(source).expect("remove source");
    fs::remove_file(destination).expect("remove destination");
}

#[test]
fn render_png_raw_limit_preserves_an_existing_destination() {
    let source = temporary_path("limited-png-source.cdml");
    let destination = temporary_path("preserved.png");
    fs::write(&source, CDML).expect("write source");
    fs::write(&destination, "preserved").expect("write destination");

    let output = run_render_with_paths(
        "png",
        &source,
        &destination,
        &["--width", "10", "--height", "10", "--max-raw-bytes", "399"],
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires 400 bytes"));
    assert_eq!(
        fs::read_to_string(&destination).expect("read destination"),
        "preserved"
    );
    fs::remove_file(source).expect("remove source");
    fs::remove_file(destination).expect("remove destination");
}

#[cfg(unix)]
#[test]
fn render_svg_rejects_a_symlink_input_before_output_publication() {
    let source = temporary_path("symlink-target.cdml");
    let link = temporary_path("symlink-source.cdml");
    let destination = temporary_path("symlink-artifact.svg");
    fs::write(&source, CDML).expect("write source");
    std::os::unix::fs::symlink(&source, &link).expect("create input symlink");

    let output = run_with_paths(&link, &destination, &[]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("symlink"));
    assert!(!destination.exists());
    fs::remove_file(link).expect("remove symlink");
    fs::remove_file(source).expect("remove source");
}

fn run_with_paths(source: &Path, destination: &Path, trailing: &[&str]) -> std::process::Output {
    run_render_with_paths("svg", source, destination, trailing)
}

fn run_render_with_paths(
    format: &str,
    source: &Path,
    destination: &Path,
    trailing: &[&str],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ferrum"));
    command.args([
        "cdml",
        "render",
        format,
        source.to_str().expect("source path is UTF-8"),
        "--output",
        destination.to_str().expect("destination path is UTF-8"),
    ]);
    command.args(trailing);
    command.output().expect("run ferrum CLI")
}

fn run_with_stdin<const N: usize>(arguments: [&str; N], source: &[u8]) -> std::process::Output {
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
        .expect("temporary directory resolves")
        .join(format!(
            "ferrum-render-cli-{}-{sequence}-{name}",
            std::process::id()
        ))
}
