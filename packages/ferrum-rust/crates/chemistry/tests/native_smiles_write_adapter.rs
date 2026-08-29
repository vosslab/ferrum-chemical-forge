//! Direct semantic proof for an explicitly selected sealed native adapter.

use std::path::Path;

use ferrum_chemistry::{
    AtomicNumber, BondOrder, MolAtom, MolBond, MolGraph, NativeChemEngine, NativeTextOutputLimit,
};

fn atom(
    symbol: &str,
    charge: Option<i32>,
    isotope: Option<u16>,
    hydrogens: Option<u16>,
) -> MolAtom {
    MolAtom::new(
        AtomicNumber::from_symbol(symbol).expect("test element is supported"),
        charge,
        isotope,
        hydrogens,
        false,
    )
    .expect("test atom is valid")
}

#[test]
#[ignore = "requires FERRUM_CHEM_SMILES_WRITE_ADAPTER naming a sealed ABI-4 closure"]
fn sealed_adapter_writes_canonical_isotope_and_charge_smiles() {
    let adapter = std::env::var("FERRUM_CHEM_SMILES_WRITE_ADAPTER")
        .expect("set FERRUM_CHEM_SMILES_WRITE_ADAPTER to the sealed adapter path");
    let engine = NativeChemEngine::load(Path::new(&adapter)).expect("load sealed adapter");
    let ethanol = MolGraph::new(
        vec![
            atom("O", None, None, None),
            atom("C", None, None, None),
            atom("C", None, None, None),
        ],
        vec![
            MolBond::new(0, 1, BondOrder::Single, false),
            MolBond::new(1, 2, BondOrder::Single, false),
        ],
        None,
    )
    .expect("ethanol graph");
    let isotope_ammonium = MolGraph::new(
        vec![atom("N", Some(1), Some(15), Some(4))],
        Vec::new(),
        None,
    )
    .expect("isotope ammonium graph");
    let invalid_valence = MolGraph::new(vec![atom("C", None, None, Some(5))], Vec::new(), None)
        .expect("structurally valid but chemically invalid graph");

    assert_eq!(
        engine.molecule_to_smiles(&ethanol, NativeTextOutputLimit::ADAPTER_MAXIMUM),
        Ok("CCO".to_owned())
    );
    assert_eq!(
        engine.molecule_to_smiles(&isotope_ammonium, NativeTextOutputLimit::ADAPTER_MAXIMUM),
        Ok("[15NH4+]".to_owned())
    );
    assert!(
        engine
            .molecule_to_smiles(&invalid_valence, NativeTextOutputLimit::ADAPTER_MAXIMUM)
            .is_err()
    );
}
