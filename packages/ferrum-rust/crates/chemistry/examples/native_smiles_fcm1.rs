//! Exercise the ABI-4 FCM1 SMILES molecule operation in one fresh process.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use ferrum_chemistry::{ADAPTER_ABI_VERSION, NativeChemEngine};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("native_smiles_fcm1: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let adapter = adapter_path(env::args().skip(1))?;
    let molecule = NativeChemEngine::load(&adapter)
        .map_err(|error| error.to_string())?
        .smiles_to_molecule("OCC")
        .map_err(|error| error.to_string())?;
    let graph = molecule.molecule();
    let coordinates = graph.coordinates().ok_or("FCM1 omitted coordinates")?;
    if !coordinates
        .points()
        .iter()
        .all(|point| point.x().is_finite() && point.y().is_finite())
    {
        return Err("FCM1 returned a non-finite coordinate".to_owned());
    }
    println!(
        "{{\"abi_version\":{},\"canonical_smiles\":\"{}\",\"atom_count\":{},\"bond_count\":{},\"coordinate_count\":{}}}",
        ADAPTER_ABI_VERSION,
        molecule.canonical_smiles(),
        graph.atoms().len(),
        graph.bonds().len(),
        coordinates.points().len(),
    );
    Ok(())
}

fn adapter_path(arguments: impl Iterator<Item = String>) -> Result<PathBuf, String> {
    let values: Vec<_> = arguments.collect();
    match values.as_slice() {
        [flag, value] if flag == "--adapter" && !value.is_empty() => Ok(PathBuf::from(value)),
        _ => Err(
            "usage: native_smiles_fcm1 --adapter /absolute/path/libferrum_chem.dylib".to_owned(),
        ),
    }
}
