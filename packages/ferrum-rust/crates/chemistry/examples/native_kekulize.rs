//! Exercise the safe native engine with a deterministic aromatic graph.
//!
//! This example is intentionally dependency-free so the native-wheel E2E can
//! use it as a stable Rust-side oracle.  It emits exactly one JSON object on
//! stdout after the engine has returned an owned [`MolGraph`].

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use ferrum_chemistry::{
    ADAPTER_ABI_VERSION, AtomicNumber, BondOrder, ChemEngine, KekulizeOptions, MolAtom, MolBond,
    MolGraph, NativeChemEngine,
};

//============================================
// Command line

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("native_kekulize: {error}");
            ExitCode::FAILURE
        }
    }
}

//============================================
// Proof workflow

fn run() -> Result<(), String> {
    let adapter = adapter_path(env::args().skip(1))?;
    let input = aromatic_benzene()?;
    let engine = NativeChemEngine::load(&adapter).map_err(|error| error.to_string())?;
    let output = engine
        .kekulize(&input, KekulizeOptions::default())
        .map_err(|error| error.to_string())?;
    println!(
        "{{\"abi_version\":{},\"input\":{},\"output\":{}}}",
        ADAPTER_ABI_VERSION,
        graph_json(&input),
        graph_json(&output),
    );
    Ok(())
}

fn adapter_path(arguments: impl Iterator<Item = String>) -> Result<PathBuf, String> {
    let values: Vec<_> = arguments.collect();
    match values.as_slice() {
        [flag, value] if flag == "--adapter" && !value.is_empty() => Ok(PathBuf::from(value)),
        _ => Err("usage: native_kekulize --adapter /absolute/path/libferrum_chem.dylib".to_owned()),
    }
}

fn aromatic_benzene() -> Result<MolGraph, String> {
    let carbon = AtomicNumber::try_from(6).map_err(|error| error.to_string())?;
    let atoms = vec![
        MolAtom::new(carbon, Some(0), None, None, true),
        MolAtom::new(carbon, None, Some(13), None, true),
        MolAtom::new(carbon, None, None, Some(1), true),
        MolAtom::new(carbon, None, None, None, true),
        MolAtom::new(carbon, None, None, None, true),
        MolAtom::new(carbon, None, None, None, true),
    ]
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| error.to_string())?;
    let bonds = (0..6)
        .map(|start| MolBond::new(start, (start + 1) % 6, BondOrder::Aromatic, true))
        .collect();
    MolGraph::new(atoms, bonds, None).map_err(|error| error.to_string())
}

//============================================
// Stable JSON protocol

fn graph_json(graph: &MolGraph) -> String {
    let atoms = graph
        .atoms()
        .iter()
        .map(|atom| {
            format!(
                concat!(
                    "{{\"atomic_number\":{},\"aromatic\":{},",
                    "\"formal_charge\":{},\"isotope\":{},",
                    "\"explicit_hydrogens\":{}}}"
                ),
                atom.atomic_number().get(),
                atom.is_aromatic(),
                optional_i32_json(atom.formal_charge()),
                optional_u16_json(atom.isotope()),
                optional_u16_json(atom.explicit_hydrogens()),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let bonds = graph
        .bonds()
        .iter()
        .map(|bond| {
            format!(
                "{{\"start\":{},\"end\":{},\"order\":\"{}\",\"aromatic\":{}}}",
                bond.start(),
                bond.end(),
                bond_order_name(bond.order()),
                bond.is_aromatic(),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("{{\"atoms\":[{atoms}],\"bonds\":[{bonds}]}}")
}

const fn bond_order_name(order: BondOrder) -> &'static str {
    match order {
        BondOrder::Aromatic => "aromatic",
        BondOrder::Single => "single",
        BondOrder::Double => "double",
        BondOrder::Triple => "triple",
        BondOrder::Quadruple => "quadruple",
    }
}

fn optional_i32_json(value: Option<i32>) -> String {
    value.map_or_else(|| "null".to_owned(), |number| number.to_string())
}

fn optional_u16_json(value: Option<u16>) -> String {
    value.map_or_else(|| "null".to_owned(), |number| number.to_string())
}

//============================================
// Pure command protocol tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_json_retains_optional_fact_presence() {
        let graph = aromatic_benzene().expect("benzene graph is valid");

        let json = graph_json(&graph);

        assert!(json.contains("\"formal_charge\":0"));
        assert!(json.contains("\"isotope\":13"));
        assert!(json.contains("\"explicit_hydrogens\":1"));
        assert!(json.contains("\"formal_charge\":null"));
        assert!(json.contains("\"order\":\"aromatic\""));
    }

    #[test]
    fn adapter_argument_is_strict_and_requires_a_value() {
        assert_eq!(
            adapter_path(
                [
                    "--adapter".to_owned(),
                    "/tmp/libferrum_chem.dylib".to_owned()
                ]
                .into_iter()
            ),
            Ok(PathBuf::from("/tmp/libferrum_chem.dylib")),
        );
        assert!(adapter_path(["--wrong".to_owned(), "path".to_owned()].into_iter()).is_err());
        assert!(adapter_path(["--adapter".to_owned()].into_iter()).is_err());
    }
}
