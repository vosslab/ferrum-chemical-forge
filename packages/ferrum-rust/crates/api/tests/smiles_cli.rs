use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMPORARY_PATH: AtomicU64 = AtomicU64::new(0);

#[test]
fn smiles_inspect_requires_an_explicit_adapter_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "inspect", "CCO"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--adapter"));
}

#[test]
fn smiles_to_smarts_requires_an_explicit_adapter_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "to-smarts", "CCO"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--adapter"));
}

#[test]
fn smiles_canonicalize_requires_an_explicit_adapter_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "canonicalize", "C(C)O"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--adapter"));
}

#[test]
fn smiles_to_molblock_requires_adapter_and_explicit_format() {
    let missing_adapter = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "to-molblock", "--format", "v2000", "CCO"])
        .output()
        .expect("run ferrum CLI");
    assert_eq!(missing_adapter.status.code(), Some(2));
    assert!(missing_adapter.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_adapter.stderr).contains("--adapter"));

    let unknown_format = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "to-molblock",
            "--adapter",
            "/not/loaded.dylib",
            "--format",
            "automatic",
            "CCO",
        ])
        .output()
        .expect("run ferrum CLI");
    assert_eq!(unknown_format.status.code(), Some(2));
    assert!(unknown_format.stdout.is_empty());
    assert!(String::from_utf8_lossy(&unknown_format.stderr).contains("invalid value"));
}

#[test]
fn smiles_to_sdf_requires_adapter_and_explicit_format() {
    let missing_adapter = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "to-sdf", "--format", "v2000", "CCO"])
        .output()
        .expect("run ferrum CLI");
    assert_eq!(missing_adapter.status.code(), Some(2));
    assert!(missing_adapter.stdout.is_empty());
    assert!(String::from_utf8_lossy(&missing_adapter.stderr).contains("--adapter"));

    let unknown_format = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "to-sdf",
            "--adapter",
            "/not/loaded.dylib",
            "--format",
            "automatic",
            "CCO",
        ])
        .output()
        .expect("run ferrum CLI");
    assert_eq!(unknown_format.status.code(), Some(2));
    assert!(unknown_format.stdout.is_empty());
    assert!(String::from_utf8_lossy(&unknown_format.stderr).contains("invalid value"));
}

#[test]
fn smiles_to_sdf_rejects_a_malformed_property_before_loading() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "to-sdf",
            "--adapter",
            "/not/loaded.dylib",
            "--format",
            "v2000",
            "--property",
            "missing-value-separator",
            "CCO",
        ])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("NAME=VALUE"));
}

#[test]
fn smiles_inspect_rejects_relative_adapter_before_loading() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "inspect", "--adapter", "adapter.dylib", "CCO"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("adapter path must be absolute"));
}

#[test]
fn smiles_to_smarts_rejects_relative_adapter_before_loading() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["smiles", "to-smarts", "--adapter", "adapter.dylib", "CCO"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("adapter path must be absolute"));
}

#[test]
fn smiles_canonicalize_rejects_relative_adapter_before_loading() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "canonicalize",
            "--adapter",
            "adapter.dylib",
            "C(C)O",
        ])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("adapter path must be absolute"));
}

#[cfg(unix)]
#[test]
fn smiles_inspect_rejects_a_symbolic_link_adapter_before_loading() {
    let target = temporary_path("adapter.dylib");
    let link = temporary_path("adapter-link.dylib");
    fs::write(&target, "not a native library").expect("write adapter target");
    std::os::unix::fs::symlink(&target, &link).expect("create adapter link");

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "inspect",
            "--adapter",
            link.to_str().expect("temporary path is UTF-8"),
            "CCO",
        ])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("regular non-symbolic-link"));
    fs::remove_file(link).expect("remove adapter link");
    fs::remove_file(target).expect("remove adapter target");
}

#[test]
#[ignore = "requires an explicit verified ABI-4 adapter closure"]
fn smiles_commands_verified_adapter_e2e_cover_inspection_smarts_molblocks_and_sdf() {
    let adapter = configured_verified_adapter();
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "inspect",
            "--adapter",
            adapter.to_str().expect("adapter path is UTF-8"),
            "CCO",
        ])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(
        output.stdout.iter().filter(|byte| **byte == b'\n').count(),
        1
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("stdout JSON");
    assert_eq!(report["schema"], "ferrum-smiles-inspection-v1");
    assert_eq!(report["adapter_abi"], 4);
    assert_eq!(report["canonical_smiles"], "CCO");
    assert_eq!(report["atoms"].as_array().map(Vec::len), Some(3));
    assert_eq!(report["bonds"].as_array().map(Vec::len), Some(2));
    let coordinates = report["coordinates"].as_array().expect("coordinates array");
    assert_eq!(coordinates.len(), 3);
    assert!(coordinates.iter().all(|coordinate| {
        coordinate["x"].as_f64().is_some_and(f64::is_finite)
            && coordinate["y"].as_f64().is_some_and(f64::is_finite)
    }));
    assert_eq!(report["atoms"][0]["chirality"], "unspecified");
    assert_eq!(report["bonds"][0]["order"], "single");
    assert_eq!(report["bonds"][0]["stereo"], "none");
    assert_eq!(report["bonds"][0]["direction"], "none");

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "canonicalize",
            "--adapter",
            adapter.to_str().expect("adapter path is UTF-8"),
            "C(C)O",
        ])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, b"CCO\n");

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "inspect",
            "--adapter",
            adapter.to_str().expect("adapter path is UTF-8"),
            "(",
        ])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).starts_with("ferrum: could not inspect SMILES:")
    );

    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "smiles",
            "to-smarts",
            "--adapter",
            adapter.to_str().expect("adapter path is UTF-8"),
            "CCO",
        ])
        .output()
        .expect("run ferrum CLI");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout, b"[#6]-[#6]-[#8]\n");

    for (format, marker) in [("v2000", "V2000"), ("v3000", "V3000")] {
        let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
            .args([
                "smiles",
                "to-molblock",
                "--adapter",
                adapter.to_str().expect("adapter path is UTF-8"),
                "--format",
                format,
                "CCO",
            ])
            .output()
            .expect("run ferrum CLI");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(String::from_utf8_lossy(&output.stdout).contains(marker));

        let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
            .args([
                "smiles",
                "to-sdf",
                "--adapter",
                adapter.to_str().expect("adapter path is UTF-8"),
                "--format",
                format,
                "--title",
                "ethanol",
                "--property",
                "source=Ferrum",
                "CCO",
            ])
            .output()
            .expect("run ferrum CLI");
        let sdf = String::from_utf8(output.stdout).expect("SDF output is UTF-8");
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert!(sdf.contains(marker));
        assert!(sdf.contains("<source>"));
        assert!(sdf.contains("\nFerrum\n"));
        assert!(sdf.ends_with("$$$$\n"));
    }
}

fn configured_verified_adapter() -> PathBuf {
    let adapter = std::env::var_os("FERRUM_SMILES_INSPECT_ADAPTER")
        .map(PathBuf::from)
        .expect("set FERRUM_SMILES_INSPECT_ADAPTER to an absolute verified ABI-4 adapter path");
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

fn temporary_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEMPORARY_PATH.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .canonicalize()
        .expect("temporary directory must resolve without a symbolic link")
        .join(format!(
            "ferrum-smiles-cli-{}-{sequence}-{name}",
            std::process::id()
        ))
}
