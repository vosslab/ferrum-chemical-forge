use std::process::{Command, Stdio};

#[test]
fn molblock_inspect_requires_an_explicit_adapter_option() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["molblock", "inspect", "-"])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--adapter"));
}

#[test]
fn molblock_inspect_rejects_empty_input_before_loading_the_adapter() {
    let output = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args([
            "molblock",
            "inspect",
            "--adapter",
            "/not/a/loaded/adapter.dylib",
            "-",
        ])
        .output()
        .expect("run ferrum CLI");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("molblock input is invalid"));
}

#[test]
fn molblock_inspect_rejects_a_relative_adapter_before_native_loading() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
        .args(["molblock", "inspect", "--adapter", "adapter.dylib", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("run ferrum CLI");
    std::io::Write::write_all(
        child.stdin.as_mut().expect("piped standard input"),
        b"name\nprogram\ncomment\n  0  0  0  0  0  0  0  0  0  0999 V2000\nM  END\n",
    )
    .expect("write standard input");
    let output = child.wait_with_output().expect("collect ferrum CLI output");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("adapter path must be absolute"));
}

#[test]
#[ignore = "requires an explicit verified ABI-4 adapter closure"]
fn molblock_inspect_verified_adapter_reports_complete_molecule() {
    let adapter = std::env::var_os("FERRUM_SMILES_INSPECT_ADAPTER")
        .expect("set FERRUM_SMILES_INSPECT_ADAPTER to a verified ABI-4 adapter");
    let adapter = adapter
        .to_str()
        .expect("configured adapter path is valid UTF-8");
    for format in ["v2000", "v3000"] {
        let writer = Command::new(env!("CARGO_BIN_EXE_ferrum"))
            .args([
                "smiles",
                "to-molblock",
                "--adapter",
                adapter,
                "--format",
                format,
                "F/C=C/F",
            ])
            .output()
            .expect("run Ferrum molblock writer");
        assert!(writer.status.success());

        let mut child = Command::new(env!("CARGO_BIN_EXE_ferrum"))
            .args(["molblock", "inspect", "--adapter", adapter, "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run Ferrum molblock inspector");
        std::io::Write::write_all(
            child.stdin.as_mut().expect("piped standard input"),
            &writer.stdout,
        )
        .expect("write generated molblock");
        let output = child.wait_with_output().expect("collect Ferrum CLI output");

        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        assert_eq!(
            output.stdout.iter().filter(|byte| **byte == b'\n').count(),
            1
        );
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("stdout JSON");
        assert_eq!(report["schema"], "ferrum-molblock-inspection-v1");
        assert_eq!(report["adapter_abi"], 4);
        assert_eq!(report["molecule"]["canonical_smiles"], "F/C=C/F");
        assert_eq!(
            report["molecule"]["atoms"].as_array().map(Vec::len),
            Some(4)
        );
        assert_eq!(
            report["molecule"]["coordinates"].as_array().map(Vec::len),
            Some(4)
        );
    }
}
