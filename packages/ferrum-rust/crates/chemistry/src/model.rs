use std::collections::BTreeSet;

use thiserror::Error;

/// A supported chemical element, represented by its atomic number.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AtomicNumber(u8);

impl AtomicNumber {
    /// Return the element's atomic number.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for AtomicNumber {
    type Error = MolGraphError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=118).contains(&value) {
            return Err(MolGraphError::UnsupportedAtomicNumber { value });
        }
        Ok(Self(value))
    }
}

/// A Kekule bond order, independent of aromaticity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BondOrder {
    /// An authored aromatic bond whose Kekule order is not yet assigned.
    Aromatic,
    /// One shared electron pair.
    Single,
    /// Two shared electron pairs.
    Double,
    /// Three shared electron pairs.
    Triple,
    /// Four shared electron pairs.
    Quadruple,
}

impl TryFrom<u8> for BondOrder {
    type Error = MolGraphError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Aromatic),
            1 => Ok(Self::Single),
            2 => Ok(Self::Double),
            3 => Ok(Self::Triple),
            4 => Ok(Self::Quadruple),
            _ => Err(MolGraphError::UnsupportedBondOrder { value }),
        }
    }
}

/// One owned atom in [`MolGraph`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MolAtom {
    atomic_number: AtomicNumber,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    aromatic: bool,
}

impl MolAtom {
    /// Create one atom with an optional isotope mass number.
    pub fn new(
        atomic_number: AtomicNumber,
        formal_charge: Option<i32>,
        isotope: Option<u16>,
        explicit_hydrogens: Option<u16>,
        aromatic: bool,
    ) -> Result<Self, MolGraphError> {
        if isotope.is_some_and(|mass_number| mass_number == 0) {
            return Err(MolGraphError::InvalidIsotope);
        }
        Ok(Self {
            atomic_number,
            formal_charge,
            isotope,
            explicit_hydrogens,
            aromatic,
        })
    }

    /// Return the element.
    #[must_use]
    pub const fn atomic_number(&self) -> AtomicNumber {
        self.atomic_number
    }

    /// Return the formal charge.
    #[must_use]
    pub const fn formal_charge(&self) -> Option<i32> {
        self.formal_charge
    }

    /// Return the optional isotope mass number.
    #[must_use]
    pub const fn isotope(&self) -> Option<u16> {
        self.isotope
    }

    /// Return the authored explicit-hydrogen count, if one was supplied.
    #[must_use]
    pub const fn explicit_hydrogens(&self) -> Option<u16> {
        self.explicit_hydrogens
    }

    /// Report whether this atom is aromatic.
    #[must_use]
    pub const fn is_aromatic(&self) -> bool {
        self.aromatic
    }
}

/// One owned bond whose endpoints index [`MolGraph::atoms`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MolBond {
    start: usize,
    end: usize,
    order: BondOrder,
    aromatic: bool,
}

impl MolBond {
    /// Create a bond. Endpoint and aromaticity validation occurs with its graph.
    #[must_use]
    pub const fn new(start: usize, end: usize, order: BondOrder, aromatic: bool) -> Self {
        Self {
            start,
            end,
            order,
            aromatic,
        }
    }

    /// Return the first endpoint index.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Return the second endpoint index.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Return the Kekule order, even when the bond is aromatic.
    #[must_use]
    pub const fn order(&self) -> BondOrder {
        self.order
    }

    /// Report whether this bond is aromatic.
    #[must_use]
    pub const fn is_aromatic(&self) -> bool {
        self.aromatic
    }
}

/// One finite two-dimensional atom coordinate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2 {
    x: f64,
    y: f64,
}

impl Point2 {
    /// Create one finite coordinate.
    pub fn new(x: f64, y: f64) -> Result<Self, MolGraphError> {
        if !x.is_finite() || !y.is_finite() {
            return Err(MolGraphError::NonFiniteCoordinate);
        }
        Ok(Self { x, y })
    }

    /// Return the horizontal coordinate.
    #[must_use]
    pub const fn x(&self) -> f64 {
        self.x
    }

    /// Return the vertical coordinate.
    #[must_use]
    pub const fn y(&self) -> f64 {
        self.y
    }
}

/// A complete, ordered coordinate set for a graph's atoms.
#[derive(Clone, Debug, PartialEq)]
pub struct Coordinates(Vec<Point2>);

impl Coordinates {
    /// Keep coordinates owned and in the same order as the graph's atoms.
    #[must_use]
    pub fn new(points: Vec<Point2>) -> Self {
        Self(points)
    }

    /// Return coordinates in atom order.
    #[must_use]
    pub fn points(&self) -> &[Point2] {
        &self.0
    }
}

/// Canonical SMILES and a complete atom-order-aligned native 2D depiction.
///
/// The value owns only Ferrum data.  It never exposes an RDKit molecule,
/// parser, or foreign allocation to callers.
#[derive(Clone, Debug, PartialEq)]
pub struct SmilesDepiction {
    canonical_smiles: String,
    coordinates: Coordinates,
}

impl SmilesDepiction {
    /// Create a decoded native SMILES depiction.
    pub(crate) fn new(canonical_smiles: String, coordinates: Coordinates) -> Self {
        Self {
            canonical_smiles,
            coordinates,
        }
    }

    /// Return the canonical SMILES written by the selected native profile.
    #[must_use]
    pub fn canonical_smiles(&self) -> &str {
        &self.canonical_smiles
    }

    /// Return finite coordinates in the native molecule's atom order.
    #[must_use]
    pub fn coordinates(&self) -> &Coordinates {
        &self.coordinates
    }
}

/// An immutable, validated molecular graph.
#[derive(Clone, Debug, PartialEq)]
pub struct MolGraph {
    atoms: Vec<MolAtom>,
    bonds: Vec<MolBond>,
    coordinates: Option<Coordinates>,
}

impl MolGraph {
    /// Construct a graph after validating its structural invariants.
    pub fn new(
        atoms: Vec<MolAtom>,
        bonds: Vec<MolBond>,
        coordinates: Option<Coordinates>,
    ) -> Result<Self, MolGraphError> {
        if let Some(points) = &coordinates
            && points.points().len() != atoms.len()
        {
            return Err(MolGraphError::IncompleteCoordinates {
                atom_count: atoms.len(),
                coordinate_count: points.points().len(),
            });
        }

        let mut seen_edges = BTreeSet::new();
        for bond in &bonds {
            if bond.start >= atoms.len() || bond.end >= atoms.len() {
                return Err(MolGraphError::EndpointOutOfRange {
                    start: bond.start,
                    end: bond.end,
                    atom_count: atoms.len(),
                });
            }
            if bond.start == bond.end {
                return Err(MolGraphError::SelfBond { atom: bond.start });
            }
            let edge = (bond.start.min(bond.end), bond.start.max(bond.end));
            if !seen_edges.insert(edge) {
                return Err(MolGraphError::DuplicateBond {
                    start: edge.0,
                    end: edge.1,
                });
            }
            if bond.aromatic && (!atoms[bond.start].aromatic || !atoms[bond.end].aromatic) {
                return Err(MolGraphError::AromaticBondNeedsAromaticAtoms {
                    start: bond.start,
                    end: bond.end,
                });
            }
            if !bond.aromatic && bond.order == BondOrder::Aromatic {
                return Err(MolGraphError::AromaticOrderNeedsAromaticBond {
                    start: bond.start,
                    end: bond.end,
                });
            }
            if bond.aromatic && matches!(bond.order, BondOrder::Triple | BondOrder::Quadruple) {
                return Err(MolGraphError::InvalidAromaticBondOrder { order: bond.order });
            }
        }

        Ok(Self {
            atoms,
            bonds,
            coordinates,
        })
    }

    /// Return atoms in stable graph order.
    #[must_use]
    pub fn atoms(&self) -> &[MolAtom] {
        &self.atoms
    }

    /// Return bonds in stable graph order.
    #[must_use]
    pub fn bonds(&self) -> &[MolBond] {
        &self.bonds
    }

    /// Return the complete coordinate set, if the graph has one.
    #[must_use]
    pub fn coordinates(&self) -> Option<&Coordinates> {
        self.coordinates.as_ref()
    }

    /// Validate the stricter aromatic form accepted by a kekulization engine.
    ///
    /// [`Self::new`] intentionally also accepts a post-kekulized aromatic graph,
    /// whose aromatic bonds retain their flag while carrying `Single` or `Double`
    /// order. An input graph has not yet made that assignment, so each aromatic
    /// bond must instead carry [`BondOrder::Aromatic`].
    pub fn validate_kekulize_input(&self) -> Result<(), MolGraphError> {
        for bond in &self.bonds {
            if bond.aromatic {
                if bond.order != BondOrder::Aromatic {
                    return Err(MolGraphError::KekulizeInputNeedsAromaticOrder {
                        start: bond.start,
                        end: bond.end,
                        order: bond.order,
                    });
                }
                if !self.atoms[bond.start].aromatic || !self.atoms[bond.end].aromatic {
                    return Err(MolGraphError::AromaticBondNeedsAromaticAtoms {
                        start: bond.start,
                        end: bond.end,
                    });
                }
            } else if bond.order == BondOrder::Aromatic {
                return Err(MolGraphError::AromaticOrderNeedsAromaticBond {
                    start: bond.start,
                    end: bond.end,
                });
            }
        }
        Ok(())
    }
}

/// A rejected value or violated [`MolGraph`] invariant.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MolGraphError {
    /// The value does not identify a supported element.
    #[error("unsupported atomic number: {value}")]
    UnsupportedAtomicNumber { value: u8 },
    /// The value is not a supported Kekule bond order.
    #[error("unsupported bond order: {value}")]
    UnsupportedBondOrder { value: u8 },
    /// Isotope mass zero is not meaningful.
    #[error("isotope mass number must be positive")]
    InvalidIsotope,
    /// Coordinates must be finite.
    #[error("coordinate values must be finite")]
    NonFiniteCoordinate,
    /// Coordinates must describe every atom or no atoms.
    #[error("coordinate count {coordinate_count} does not match atom count {atom_count}")]
    IncompleteCoordinates {
        /// Number of graph atoms.
        atom_count: usize,
        /// Number of supplied coordinate pairs.
        coordinate_count: usize,
    },
    /// A bond endpoint does not identify an atom in this graph.
    #[error("bond endpoints {start} and {end} are outside {atom_count} atoms")]
    EndpointOutOfRange {
        /// First endpoint.
        start: usize,
        /// Second endpoint.
        end: usize,
        /// Number of graph atoms.
        atom_count: usize,
    },
    /// A bond may not connect an atom to itself.
    #[error("self bond at atom {atom}")]
    SelfBond {
        /// The atom used at both endpoints.
        atom: usize,
    },
    /// An undirected atom pair has more than one bond.
    #[error("duplicate bond between atoms {start} and {end}")]
    DuplicateBond {
        /// Lower endpoint.
        start: usize,
        /// Higher endpoint.
        end: usize,
    },
    /// Aromatic bonds require aromatic atoms at both endpoints.
    #[error("aromatic bond between atoms {start} and {end} needs aromatic endpoints")]
    AromaticBondNeedsAromaticAtoms {
        /// First endpoint.
        start: usize,
        /// Second endpoint.
        end: usize,
    },
    /// An authored aromatic order requires an aromatic bond flag.
    #[error("aromatic order between atoms {start} and {end} needs an aromatic bond flag")]
    AromaticOrderNeedsAromaticBond {
        /// First endpoint.
        start: usize,
        /// Second endpoint.
        end: usize,
    },
    /// A kekulization input must not contain an already assigned aromatic order.
    #[error("kekulization input bond between atoms {start} and {end} has {order:?} order")]
    KekulizeInputNeedsAromaticOrder {
        /// First endpoint.
        start: usize,
        /// Second endpoint.
        end: usize,
        /// Already assigned order.
        order: BondOrder,
    },
    /// Aromaticity cannot accompany triple or quadruple Kekule bonds.
    #[error("aromatic bond cannot have {order:?} Kekule order")]
    InvalidAromaticBondOrder {
        /// Rejected Kekule order.
        order: BondOrder,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carbon(aromatic: bool) -> MolAtom {
        MolAtom::new(
            AtomicNumber::try_from(6).expect("carbon is supported"),
            Some(0),
            None,
            None,
            aromatic,
        )
        .expect("valid atom")
    }

    #[test]
    fn rejects_unsupported_atom_and_bond_values() {
        assert_eq!(
            AtomicNumber::try_from(0),
            Err(MolGraphError::UnsupportedAtomicNumber { value: 0 })
        );
        assert_eq!(
            AtomicNumber::try_from(119),
            Err(MolGraphError::UnsupportedAtomicNumber { value: 119 })
        );
        assert_eq!(
            BondOrder::try_from(5),
            Err(MolGraphError::UnsupportedBondOrder { value: 5 })
        );
    }

    #[test]
    fn rejects_invalid_bond_endpoints_and_duplicate_edges() {
        let atom = carbon(false);
        assert_eq!(
            MolGraph::new(
                vec![atom.clone()],
                vec![MolBond::new(0, 1, BondOrder::Single, false)],
                None,
            ),
            Err(MolGraphError::EndpointOutOfRange {
                start: 0,
                end: 1,
                atom_count: 1,
            })
        );
        assert_eq!(
            MolGraph::new(
                vec![atom.clone(), atom.clone()],
                vec![
                    MolBond::new(0, 1, BondOrder::Single, false),
                    MolBond::new(1, 0, BondOrder::Double, false),
                ],
                None,
            ),
            Err(MolGraphError::DuplicateBond { start: 0, end: 1 })
        );
        assert_eq!(
            MolGraph::new(
                vec![atom],
                vec![MolBond::new(0, 0, BondOrder::Single, false)],
                None,
            ),
            Err(MolGraphError::SelfBond { atom: 0 })
        );
    }

    #[test]
    fn aromaticity_is_independent_but_structurally_constrained() {
        let aromatic = carbon(true);
        let plain = carbon(false);
        let pre_kekulize = MolGraph::new(
            vec![aromatic.clone(), aromatic.clone()],
            vec![MolBond::new(0, 1, BondOrder::Aromatic, true)],
            None,
        )
        .expect("pre-kekulized aromatic graph is valid");
        assert!(pre_kekulize.bonds()[0].is_aromatic());
        assert_eq!(pre_kekulize.bonds()[0].order(), BondOrder::Aromatic);
        assert_eq!(pre_kekulize.validate_kekulize_input(), Ok(()));
        let post_kekulize = MolGraph::new(
            vec![aromatic.clone(), aromatic.clone()],
            vec![MolBond::new(0, 1, BondOrder::Double, true)],
            None,
        )
        .expect("post-kekulize order retains aromaticity");
        assert_eq!(post_kekulize.bonds()[0].order(), BondOrder::Double);
        assert_eq!(
            post_kekulize.validate_kekulize_input(),
            Err(MolGraphError::KekulizeInputNeedsAromaticOrder {
                start: 0,
                end: 1,
                order: BondOrder::Double,
            })
        );
        assert_eq!(
            MolGraph::new(
                vec![aromatic.clone(), plain],
                vec![MolBond::new(0, 1, BondOrder::Single, true)],
                None,
            ),
            Err(MolGraphError::AromaticBondNeedsAromaticAtoms { start: 0, end: 1 })
        );
        assert_eq!(
            MolGraph::new(
                vec![aromatic.clone(), aromatic.clone()],
                vec![MolBond::new(0, 1, BondOrder::Triple, true)],
                None,
            ),
            Err(MolGraphError::InvalidAromaticBondOrder {
                order: BondOrder::Triple,
            })
        );
        assert_eq!(
            MolGraph::new(
                vec![aromatic.clone(), aromatic],
                vec![MolBond::new(0, 1, BondOrder::Aromatic, false)],
                None,
            ),
            Err(MolGraphError::AromaticOrderNeedsAromaticBond { start: 0, end: 1 })
        );
    }

    #[test]
    fn coordinates_are_finite_and_all_or_none() {
        assert_eq!(
            Point2::new(f64::NAN, 0.0),
            Err(MolGraphError::NonFiniteCoordinate)
        );
        let coordinate = Point2::new(1.0, -2.0).expect("finite coordinate");
        assert_eq!(
            MolGraph::new(
                vec![carbon(false), carbon(false)],
                Vec::new(),
                Some(Coordinates::new(vec![coordinate])),
            ),
            Err(MolGraphError::IncompleteCoordinates {
                atom_count: 2,
                coordinate_count: 1,
            })
        );
    }

    #[test]
    fn atom_facts_preserve_absent_and_present_values() {
        let absent = MolAtom::new(
            AtomicNumber::try_from(6).expect("carbon is supported"),
            None,
            None,
            None,
            false,
        )
        .expect("valid atom");
        let present = MolAtom::new(
            AtomicNumber::try_from(7).expect("nitrogen is supported"),
            Some(-3),
            Some(15),
            Some(2),
            true,
        )
        .expect("valid atom");
        assert_eq!(absent.formal_charge(), None);
        assert_eq!(absent.isotope(), None);
        assert_eq!(absent.explicit_hydrogens(), None);
        assert_eq!(present.formal_charge(), Some(-3));
        assert_eq!(present.isotope(), Some(15));
        assert_eq!(present.explicit_hydrogens(), Some(2));
    }
}
