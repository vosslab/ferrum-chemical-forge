//! Closed chemistry vocabulary conversion for Python-owned molecule values.

use ferrum_chemistry::{AtomChirality, BondDirection, BondOrder, BondStereo};

use super::{
    PySmilesAtomChiralityV1, PySmilesBondDirectionV1, PySmilesBondOrderV1, PySmilesBondStereoV1,
};

pub(super) fn atom_chirality(value: AtomChirality) -> PySmilesAtomChiralityV1 {
    match value {
        AtomChirality::Unspecified => PySmilesAtomChiralityV1::Unspecified,
        AtomChirality::TetrahedralCw => PySmilesAtomChiralityV1::TetrahedralCw,
        AtomChirality::TetrahedralCcw => PySmilesAtomChiralityV1::TetrahedralCcw,
        AtomChirality::Other => PySmilesAtomChiralityV1::Other,
    }
}

pub(super) fn bond_order(value: BondOrder) -> PySmilesBondOrderV1 {
    match value {
        BondOrder::Aromatic => PySmilesBondOrderV1::Aromatic,
        BondOrder::Single => PySmilesBondOrderV1::Single,
        BondOrder::Double => PySmilesBondOrderV1::Double,
        BondOrder::Triple => PySmilesBondOrderV1::Triple,
        BondOrder::Quadruple => PySmilesBondOrderV1::Quadruple,
    }
}

pub(super) fn bond_stereo(value: BondStereo) -> PySmilesBondStereoV1 {
    match value {
        BondStereo::None => PySmilesBondStereoV1::None,
        BondStereo::Any => PySmilesBondStereoV1::Any,
        BondStereo::Z => PySmilesBondStereoV1::Z,
        BondStereo::E => PySmilesBondStereoV1::E,
        BondStereo::Cis => PySmilesBondStereoV1::Cis,
        BondStereo::Trans => PySmilesBondStereoV1::Trans,
        BondStereo::Other => PySmilesBondStereoV1::Other,
    }
}

pub(super) fn bond_direction(value: BondDirection) -> PySmilesBondDirectionV1 {
    match value {
        BondDirection::None => PySmilesBondDirectionV1::None,
        BondDirection::BeginWedge => PySmilesBondDirectionV1::BeginWedge,
        BondDirection::BeginDash => PySmilesBondDirectionV1::BeginDash,
        BondDirection::EndUpRight => PySmilesBondDirectionV1::EndUpRight,
        BondDirection::EndDownRight => PySmilesBondDirectionV1::EndDownRight,
        BondDirection::Other => PySmilesBondDirectionV1::Other,
    }
}
