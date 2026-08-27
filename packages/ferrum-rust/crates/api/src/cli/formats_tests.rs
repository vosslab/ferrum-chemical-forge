use clap::Parser;

use super::{Cli, run};

fn run_formats(arguments: &[&str]) -> (Vec<u8>, Vec<u8>) {
    let cli = Cli::try_parse_from(arguments).expect("formats arguments parse");
    let mut stdin = [].as_slice();
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    run(cli, &mut stdin, &mut stdout, &mut stderr).expect("formats completes");
    (stdout, stderr)
}

#[test]
fn formats_json_routes_through_production_without_runtime_or_input() {
    let (stdout, stderr) = run_formats(&["ferrum", "formats", "--json"]);
    let catalog: serde_json::Value =
        serde_json::from_slice(&stdout).expect("formats output is JSON");
    assert_eq!(catalog["schema"], "ferrum-interchange-capabilities-v1");
    for (alias, runtime_requirement) in [
        ("smiles", serde_json::json!("runtime_required")),
        ("cml", serde_json::json!("runtime_free")),
        ("sdf", serde_json::json!("runtime_required")),
        ("cdxml", serde_json::Value::Null),
    ] {
        let capability = catalog["capabilities"]
            .as_array()
            .expect("catalog capabilities")
            .iter()
            .find(|candidate| {
                candidate["input"]["aliases"]
                    .as_array()
                    .is_some_and(|aliases| aliases.iter().any(|value| value == alias))
            })
            .expect("representative capability");
        assert_eq!(
            capability["input"]["runtime_requirement"],
            runtime_requirement
        );
        if alias == "cdxml" {
            assert!(capability["output"].is_null());
        }
    }
    assert!(stderr.is_empty());
}

#[test]
fn formats_default_text_projects_the_catalog_for_humans() {
    let (json_stdout, json_stderr) = run_formats(&["ferrum", "formats", "--json"]);
    let (text_stdout, text_stderr) = run_formats(&["ferrum", "formats"]);
    let json_catalog: serde_json::Value =
        serde_json::from_slice(&json_stdout).expect("JSON catalog");
    let text = String::from_utf8(text_stdout).expect("human-readable text");
    assert!(text.lines().any(|line| {
        line.contains("input canonical=cml")
            && line.contains("output canonical=cml")
            && line.contains("runtime=runtime_free")
    }));
    assert!(text.lines().any(|line| {
        line.contains("input canonical=sdf")
            && line.contains("output canonical=sdf_v2000")
            && line.contains("runtime=runtime_required")
    }));
    assert!(text.lines().any(|line| {
        line.contains("input canonical=cdxml")
            && line.contains("runtime=not_applicable")
            && line.contains("output=none")
    }));
    assert!(
        json_catalog["capabilities"]
            .as_array()
            .is_some_and(|capabilities| {
                capabilities.iter().any(|capability| {
                    capability["input"]["canonical_name"] == "cml"
                        && capability["output"]["canonical_name"] == "cml"
                })
            })
    );
    assert!(
        json_catalog["capabilities"]
            .as_array()
            .is_some_and(|capabilities| {
                capabilities.iter().any(|capability| {
                    capability["input"]["canonical_name"] == "cdxml"
                        && capability["output"].is_null()
                })
            })
    );
    assert!(json_stderr.is_empty());
    assert!(text_stderr.is_empty());
}
