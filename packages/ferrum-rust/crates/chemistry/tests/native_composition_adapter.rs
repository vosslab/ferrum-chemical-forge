//! Direct semantic proof for an explicitly selected sealed native adapter.

use std::path::Path;

use ferrum_chemistry::{AtomicNumber, BondOrder, MolAtom, MolBond, MolGraph, NativeChemEngine};

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
#[ignore = "requires FERRUM_CHEM_COMPOSITION_ADAPTER naming a sealed ABI-4 closure"]
fn sealed_adapter_reports_implicit_isotope_and_charge_composition() {
    let adapter = std::env::var("FERRUM_CHEM_COMPOSITION_ADAPTER")
        .expect("set FERRUM_CHEM_COMPOSITION_ADAPTER to the sealed adapter path");
    let engine = NativeChemEngine::load(Path::new(&adapter)).expect("load sealed adapter");
    let methane =
        MolGraph::new(vec![atom("C", None, None, None)], vec![], None).expect("methane graph");
    let carbon_13 = MolGraph::new(vec![atom("C", None, Some(13), Some(4))], vec![], None)
        .expect("carbon-13 methane graph");
    let ammonium = MolGraph::new(vec![atom("N", Some(1), None, Some(4))], vec![], None)
        .expect("ammonium graph");
    let authored_and_physical_hydrogen = MolGraph::new(
        vec![atom("C", None, None, Some(3)), atom("H", None, None, None)],
        vec![MolBond::new(0, 1, BondOrder::Single, false)],
        None,
    )
    .expect("mixed hydrogen graph");
    let heavy_water = MolGraph::new(
        vec![
            atom("O", None, None, None),
            atom("H", None, Some(2), None),
            atom("H", None, Some(2), None),
        ],
        vec![
            MolBond::new(0, 1, BondOrder::Single, false),
            MolBond::new(0, 2, BondOrder::Single, false),
        ],
        None,
    )
    .expect("heavy-water graph");
    let isotope_order = MolGraph::new(
        vec![
            atom("N", None, Some(15), Some(2)),
            atom("O", None, Some(18), Some(1)),
        ],
        vec![MolBond::new(0, 1, BondOrder::Single, false)],
        None,
    )
    .expect("isotope-order graph");

    let methane_result = engine
        .molecule_composition(&methane)
        .expect("methane composition");
    let isotope_result = engine
        .molecule_composition(&carbon_13)
        .expect("carbon-13 composition");
    let charge_result = engine
        .molecule_composition(&ammonium)
        .expect("ammonium composition");
    let mixed_hydrogen_result = engine
        .molecule_composition(&authored_and_physical_hydrogen)
        .expect("mixed hydrogen composition");
    let heavy_water_result = engine
        .molecule_composition(&heavy_water)
        .expect("heavy-water composition");
    let isotope_order_result = engine
        .molecule_composition(&isotope_order)
        .expect("isotope-order composition");

    assert_eq!(methane_result.formula(), "CH4");
    assert_eq!(isotope_result.formula(), "[13C]H4");
    assert_eq!(charge_result.formula(), "H4N+");
    assert_eq!(mixed_hydrogen_result.formula(), "CH4");
    assert_eq!(heavy_water_result.formula(), "[2H]2O");
    assert_eq!(isotope_order_result.formula(), "H3[15N][18O]");
    assert_eq!(charge_result.net_formal_charge(), 1);
    assert!(methane_result.average_molecular_weight() > methane_result.monoisotopic_mass());
}
