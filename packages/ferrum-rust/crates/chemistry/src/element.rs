//! Periodic-table symbol conversion for Ferrum-owned atomic numbers.

use crate::MolGraphError;

const ELEMENT_SYMBOLS: [&str; 118] = [
    "H", "He", "Li", "Be", "B", "C", "N", "O", "F", "Ne", "Na", "Mg", "Al", "Si", "P", "S", "Cl",
    "Ar", "K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As",
    "Se", "Br", "Kr", "Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In",
    "Sn", "Sb", "Te", "I", "Xe", "Cs", "Ba", "La", "Ce", "Pr", "Nd", "Pm", "Sm", "Eu", "Gd", "Tb",
    "Dy", "Ho", "Er", "Tm", "Yb", "Lu", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl",
    "Pb", "Bi", "Po", "At", "Rn", "Fr", "Ra", "Ac", "Th", "Pa", "U", "Np", "Pu", "Am", "Cm", "Bk",
    "Cf", "Es", "Fm", "Md", "No", "Lr", "Rf", "Db", "Sg", "Bh", "Hs", "Mt", "Ds", "Rg", "Cn", "Nh",
    "Fl", "Mc", "Lv", "Ts", "Og",
];

pub(crate) fn symbol(atomic_number: u8) -> &'static str {
    ELEMENT_SYMBOLS[usize::from(atomic_number - 1)]
}

pub(crate) fn atomic_number(symbol: &str) -> Result<u8, MolGraphError> {
    ELEMENT_SYMBOLS
        .iter()
        .position(|candidate| *candidate == symbol)
        .map(|index| u8::try_from(index + 1).expect("the element table has 118 entries"))
        .ok_or_else(|| MolGraphError::UnsupportedElementSymbol {
            value: symbol.to_owned(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_element_symbol_round_trips_to_its_one_based_atomic_number() {
        for (index, symbol) in ELEMENT_SYMBOLS.iter().enumerate() {
            let number = atomic_number(symbol).expect("table symbol is supported");
            assert_eq!(usize::from(number), index + 1);
            assert_eq!(super::symbol(number), *symbol);
        }
    }
}
