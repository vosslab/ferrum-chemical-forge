use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use ferrum_document::DocumentSession;

const COORDINATE_CDML: &str = concat!(
    "<cdml version=\"1.0\"><molecule id=\"m1\">",
    "<atom id=\"a1\" name=\"C\"><point x=\"10\" y=\"20\" z=\"3\"/></atom>",
    "<atom id=\"a2\" name=\"C\"><point x=\"30\" y=\"20\"/></atom>",
    "<atom id=\"a3\" name=\"O\"><point x=\"50\" y=\"20\" z=\"-1\"/></atom>",
    "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"n1\"/>",
    "<bond id=\"b2\" start=\"a2\" end=\"a3\" type=\"n1\"/>",
    "</molecule></cdml>"
);

#[test]
fn generate_coordinates_requires_explicit_adapter_target_and_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["cdml", "generate-coordinates", "-"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("--adapter"));
    assert!(diagnostic.contains("--molecule-id"));
    assert!(diagnostic.contains("--output"));
}

#[test]
fn generate_coordinates_resolves_the_document_target_before_loading_native_code() {
    let output = run_with_stdin(
        [
            "cdml",
            "generate-coordinates",
            "--adapter",
            "/adapter-does-not-exist.dylib",
            "--molecule-id",
            "not-m1",
            "-",
            "--output",
            "-",
        ],
        COORDINATE_CDML,
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    assert!(diagnostic.contains("no durable molecule"));
    assert!(diagnostic.contains("not-m1"));
    assert!(!diagnostic.contains("adapter-does-not-exist"));
}

#[test]
fn generate_coordinates_rejects_a_relative_adapter_before_dynamic_loading() {
    let output = run_with_stdin(
        [
            "cdml",
            "generate-coordinates",
            "--adapter",
            "adapter.dylib",
            "--molecule-id",
            "m1",
            "-",
            "--output",
            "-",
        ],
        COORDINATE_CDML,
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("adapter path must be absolute"));
}

#[test]
#[ignore = "requires an explicit verified ABI-4 adapter closure"]
fn generate_coordinates_with_verified_adapter_preserves_document_placement() {
    let adapter = configured_verified_adapter();
    let output = run_with_stdin(
        [
            "cdml",
            "generate-coordinates",
            "--adapter",
            adapter.to_str().expect("adapter path is UTF-8"),
            "--molecule-id",
            "m1",
            "-",
            "--output",
            "-",
        ],
        COORDINATE_CDML,
    );

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let generated = String::from_utf8(output.stdout).expect("generated CDML is UTF-8");
    let session = DocumentSession::load(&generated).expect("generated CDML loads");
    let observation = session.observe(0).expect("generated CDML projects");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m1"))
        .expect("selected molecule remains present");
    let points = molecule
        .atoms()
        .iter()
        .map(|atom| atom.position())
        .collect::<Vec<_>>();
    assert_eq!(points.len(), 3);
    assert!(
        points.iter().all(|point| {
            point.x().is_finite() && point.y().is_finite() && point.z().is_finite()
        })
    );
    assert_eq!(
        points.iter().map(|point| point.z()).collect::<Vec<_>>(),
        [3.0, 0.0, -1.0]
    );

    let centroid_x = points.iter().map(|point| point.x()).sum::<f64>() / points.len() as f64;
    let centroid_y = points.iter().map(|point| point.y()).sum::<f64>() / points.len() as f64;
    assert_near(centroid_x, 30.0);
    assert_near(centroid_y, 20.0);
    let mean_bond_length =
        (distance(&points[0], &points[1]) + distance(&points[1], &points[2])) / 2.0;
    assert_near(mean_bond_length, 20.0);

    let signed_area = (points[1].x() - points[0].x()) * (points[2].y() - points[0].y())
        - (points[1].y() - points[0].y()) * (points[2].x() - points[0].x());
    assert_ne!(
        signed_area, 0.0,
        "native depiction replaces the collinear input"
    );
}

fn configured_verified_adapter() -> PathBuf {
    let adapter = std::env::var_os("FERRUM_CDML_COORDINATE_ADAPTER")
        .map(PathBuf::from)
        .expect("set FERRUM_CDML_COORDINATE_ADAPTER to an absolute verified ABI-4 adapter path");
    assert!(
        adapter.is_absolute(),
        "configured adapter path must be absolute"
    );
    assert!(
        adapter.is_file(),
        "configured adapter path must be a regular file"
    );
    adapter
}

fn run_with_stdin<const N: usize>(arguments: [&str; N], source: &str) -> std::process::Output {
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
        .write_all(source.as_bytes())
        .expect("write CDML input");
    child.wait_with_output().expect("collect ferrum output")
}

fn distance(first: &ferrum_document::Point3V1, second: &ferrum_document::Point3V1) -> f64 {
    (first.x() - second.x()).hypot(first.y() - second.y())
}

fn assert_near(actual: f64, expected: f64) {
    let roundoff = f64::EPSILON * 32.0 * expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= roundoff,
        "{actual} differs from {expected} by more than placement roundoff {roundoff}"
    );
}
