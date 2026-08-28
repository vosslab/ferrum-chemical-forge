//! Capability-free, complete molecule facts for pure chemistry lowering.

use ferrum_core::{BondOrder, BondStyle};
use serde::Serialize;
use thiserror::Error;

use crate::Point3V1;

/// Closed inventory of non-atom vertices that makes absence a chemistry fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NonAtomVertexKindV1 {
    CompactGroup,
    MoleculeText,
    Query,
}

/// A retained non-atom's topology category only; it intentionally carries no identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonAtomVertexFact {
    kind: NonAtomVertexKindV1,
    source_order: u32,
}
impl NonAtomVertexFact {
    #[must_use]
    pub const fn new(kind: NonAtomVertexKindV1, source_order: u32) -> Self {
        Self { kind, source_order }
    }
    #[must_use]
    pub const fn kind(self) -> NonAtomVertexKindV1 {
        self.kind
    }
    #[must_use]
    pub const fn source_order(self) -> u32 {
        self.source_order
    }
}

/// One atom input in the exact graph-position order.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectMoleculeGraphAtomFact {
    element: Option<String>,
    position: Point3V1,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    free_sites: Option<u16>,
}

/// Complete authored atom facts in the lowerer's graph-position order.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectMoleculeGraphAtomInput {
    pub element: Option<String>,
    pub position: Point3V1,
    pub formal_charge: Option<i32>,
    pub isotope: Option<u16>,
    pub explicit_hydrogens: Option<u16>,
    pub valence: Option<u16>,
    pub multiplicity: Option<u16>,
    pub free_sites: Option<u16>,
}
impl DirectMoleculeGraphAtomFact {
    #[must_use]
    pub fn new(input: DirectMoleculeGraphAtomInput) -> Self {
        Self {
            element: input.element,
            position: input.position,
            formal_charge: input.formal_charge,
            isotope: input.isotope,
            explicit_hydrogens: input.explicit_hydrogens,
            valence: input.valence,
            multiplicity: input.multiplicity,
            free_sites: input.free_sites,
        }
    }
    #[must_use]
    pub fn element(&self) -> Option<&str> {
        self.element.as_deref()
    }
    #[must_use]
    pub const fn position(&self) -> Point3V1 {
        self.position
    }
    #[must_use]
    pub const fn formal_charge(&self) -> Option<i32> {
        self.formal_charge
    }
    #[must_use]
    pub const fn isotope(&self) -> Option<u16> {
        self.isotope
    }
    #[must_use]
    pub const fn explicit_hydrogens(&self) -> Option<u16> {
        self.explicit_hydrogens
    }
    #[must_use]
    pub const fn valence(&self) -> Option<u16> {
        self.valence
    }
    #[must_use]
    pub const fn multiplicity(&self) -> Option<u16> {
        self.multiplicity
    }
    #[must_use]
    pub const fn free_sites(&self) -> Option<u16> {
        self.free_sites
    }
}

/// Closed endpoint category without a durable identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectMoleculeGraphEndpoint {
    Atom(usize),
    NonAtom,
    Unknown,
    Missing,
}

/// One chemistry bond input in source order.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectMoleculeGraphBondFact {
    start: DirectMoleculeGraphEndpoint,
    end: DirectMoleculeGraphEndpoint,
    order: Option<BondOrder>,
    style: Option<BondStyle>,
}
impl DirectMoleculeGraphBondFact {
    #[must_use]
    pub const fn new(
        start: DirectMoleculeGraphEndpoint,
        end: DirectMoleculeGraphEndpoint,
        order: Option<BondOrder>,
        style: Option<BondStyle>,
    ) -> Self {
        Self {
            start,
            end,
            order,
            style,
        }
    }
    #[must_use]
    pub const fn start(&self) -> DirectMoleculeGraphEndpoint {
        self.start
    }
    #[must_use]
    pub const fn end(&self) -> DirectMoleculeGraphEndpoint {
        self.end
    }
    #[must_use]
    pub const fn order(&self) -> Option<BondOrder> {
        self.order
    }
    #[must_use]
    pub fn style(&self) -> Option<&BondStyle> {
        self.style.as_ref()
    }
}

/// Complete, capability-free input to the shared native graph lowerer.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectMoleculeGraphFacts {
    atoms: Vec<DirectMoleculeGraphAtomFact>,
    bonds: Vec<DirectMoleculeGraphBondFact>,
    non_atoms: Vec<NonAtomVertexFact>,
    include_coordinates: bool,
}
impl DirectMoleculeGraphFacts {
    #[must_use]
    pub const fn new(
        atoms: Vec<DirectMoleculeGraphAtomFact>,
        bonds: Vec<DirectMoleculeGraphBondFact>,
        non_atoms: Vec<NonAtomVertexFact>,
        include_coordinates: bool,
    ) -> Self {
        Self {
            atoms,
            bonds,
            non_atoms,
            include_coordinates,
        }
    }
    #[must_use]
    pub fn atoms(&self) -> &[DirectMoleculeGraphAtomFact] {
        &self.atoms
    }
    #[must_use]
    pub fn bonds(&self) -> &[DirectMoleculeGraphBondFact] {
        &self.bonds
    }
    #[must_use]
    pub fn non_atoms(&self) -> &[NonAtomVertexFact] {
        &self.non_atoms
    }
    #[must_use]
    pub const fn include_coordinates(&self) -> bool {
        self.include_coordinates
    }
}

/// Closed conversion refusal for projection-to-facts adaptation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DirectMoleculeGraphFactsError {
    #[error("bond endpoint cannot be resolved against molecule atoms")]
    UnresolvedEndpoint,
}

#[cfg(test)]
mod tests {
    use super::NonAtomVertexKindV1;

    #[test]
    fn non_atom_vertex_kinds_use_stable_snake_case_wire_spelling() {
        for (kind, expected) in [
            (NonAtomVertexKindV1::CompactGroup, "\"compact_group\""),
            (NonAtomVertexKindV1::MoleculeText, "\"molecule_text\""),
            (NonAtomVertexKindV1::Query, "\"query\""),
        ] {
            assert_eq!(
                serde_json::to_string(&kind).expect("kind serializes"),
                expected
            );
        }
    }
}
