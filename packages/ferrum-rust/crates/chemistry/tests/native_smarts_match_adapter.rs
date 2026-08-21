//! Direct semantic proof for an explicitly selected sealed native adapter.

use std::path::Path;

use ferrum_chemistry::{
    AtomicNumber, BondOrder, ChemistryError, MolAtom, MolBond, MolGraph, NativeChemEngine,
    SmartsMatchOptions, SmartsMatchUnavailableReason,
};

fn carbon() -> MolAtom {
    MolAtom::new(
        AtomicNumber::from_symbol("C").expect("carbon is supported"),
        None,
        None,
        None,
        false,
    )
    .expect("carbon atom is valid")
}

fn ethane() -> MolGraph {
    MolGraph::new(
        vec![carbon(), carbon()],
        vec![MolBond::new(0, 1, BondOrder::Single, false)],
        None,
    )
    .expect("ethane graph is valid")
}

#[test]
#[ignore = "requires FERRUM_CHEM_SMARTS_MATCH_ADAPTER naming a sealed ABI-5 closure"]
fn sealed_adapter_matches_bounds_and_rejects_invalid_smarts() {
    let adapter = std::env::var("FERRUM_CHEM_SMARTS_MATCH_ADAPTER")
        .expect("set FERRUM_CHEM_SMARTS_MATCH_ADAPTER to the sealed adapter path");
    let engine = NativeChemEngine::load(Path::new(&adapter)).expect("load sealed adapter");
    let target = ethane();

    let matched = engine
        .smarts_match("C", &target, SmartsMatchOptions::new(2).expect("valid cap"))
        .expect("carbon matches ethane");
    assert_eq!(matched.rows(), &[vec![0], vec![1]]);
    assert!(!matched.truncated());

    let no_match = engine
        .smarts_match("N", &target, SmartsMatchOptions::new(2).expect("valid cap"))
        .expect("nitrogen does not match ethane");
    assert!(no_match.rows().is_empty());
    assert!(!no_match.truncated());

    let capped = engine
        .smarts_match("C", &target, SmartsMatchOptions::new(1).expect("valid cap"))
        .expect("capped carbon match");
    assert_eq!(capped.rows(), &[vec![0]]);
    assert!(capped.truncated());

    assert_eq!(
        engine.smarts_match("[", &target, SmartsMatchOptions::new(1).expect("valid cap")),
        Err(ChemistryError::SmartsMatchUnavailable {
            reason: SmartsMatchUnavailableReason::NativeRejected,
        })
    );
}
