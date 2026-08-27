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

    /// Return the canonical case-sensitive element symbol.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        crate::element::symbol(self.0)
    }

    /// Resolve one canonical case-sensitive element symbol.
    pub fn from_symbol(symbol: &str) -> Result<Self, MolGraphError> {
        crate::element::atomic_number(symbol).map(Self)
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

/// Stable atom chirality fact decoded from the ABI-4 molecule envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtomChirality {
    /// No tetrahedral chirality is specified.
    Unspecified,
    /// RDKit tetrahedral clockwise chirality.
    TetrahedralCw,
    /// RDKit tetrahedral counter-clockwise chirality.
    TetrahedralCcw,
    /// A recognized but not otherwise modeled chirality class.
    Other,
}

/// Stable bond stereo fact decoded from the ABI-4 molecule envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BondStereo {
    None,
    Any,
    Z,
    E,
    Cis,
    Trans,
    Other,
}

/// Stable bond drawing direction decoded from the ABI-4 molecule envelope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BondDirection {
    None,
    BeginWedge,
    BeginDash,
    EndUpRight,
    EndDownRight,
    Other,
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
    chirality: AtomChirality,
    radical_electrons: u8,
    no_implicit: bool,
    atom_map_number: Option<u32>,
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
            chirality: AtomChirality::Unspecified,
            radical_electrons: 0,
            no_implicit: false,
            atom_map_number: None,
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

    /// Return the decoded chirality fact.
    #[must_use]
    pub const fn chirality(&self) -> AtomChirality {
        self.chirality
    }

    /// Return the exact radical-electron count.
    #[must_use]
    pub const fn radical_electrons(&self) -> u8 {
        self.radical_electrons
    }

    /// Report whether RDKit disabled implicit hydrogens for this atom.
    #[must_use]
    pub const fn no_implicit(&self) -> bool {
        self.no_implicit
    }

    /// Return the optional source atom-map number.
    #[must_use]
    pub const fn atom_map_number(&self) -> Option<u32> {
        self.atom_map_number
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_native(
        atomic_number: AtomicNumber,
        formal_charge: i32,
        isotope: u16,
        explicit_hydrogens: u16,
        aromatic: bool,
        chirality: AtomChirality,
        radical_electrons: u8,
        no_implicit: bool,
        atom_map_number: u32,
    ) -> Result<Self, MolGraphError> {
        Ok(Self {
            atomic_number,
            formal_charge: Some(formal_charge),
            isotope: (isotope != 0).then_some(isotope),
            explicit_hydrogens: Some(explicit_hydrogens),
            aromatic,
            chirality,
            radical_electrons,
            no_implicit,
            atom_map_number: (atom_map_number != 0).then_some(atom_map_number),
        })
    }
}

/// One owned bond whose endpoints index [`MolGraph::atoms`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MolBond {
    start: usize,
    end: usize,
    order: BondOrder,
    aromatic: bool,
    stereo: BondStereo,
    direction: BondDirection,
    stereo_atoms: Option<(usize, usize)>,
}

/// A directional bond request that cannot be represented by the public model.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum MolBondDirectionError {
    /// A directional constructor needs a direction other than [`BondDirection::None`].
    #[error("a directed bond requires a direction")]
    DirectionRequired,
    /// The public model does not assign semantics to this direction.
    #[error("bond direction {direction:?} is unsupported")]
    UnsupportedDirection {
        /// Rejected direction.
        direction: BondDirection,
    },
    /// A modeled direction can only annotate a non-aromatic single bond.
    #[error(
        "bond direction {direction:?} requires a non-aromatic single bond, received {order:?} aromatic={aromatic}"
    )]
    DirectionRequiresNonAromaticSingleBond {
        /// Requested direction.
        direction: BondDirection,
        /// Rejected bond order.
        order: BondOrder,
        /// Rejected aromaticity flag.
        aromatic: bool,
    },
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
            stereo: BondStereo::None,
            direction: BondDirection::None,
            stereo_atoms: None,
        }
    }

    /// Create a public directional non-aromatic single bond.
    pub fn directed(
        start: usize,
        end: usize,
        order: BondOrder,
        aromatic: bool,
        direction: BondDirection,
    ) -> Result<Self, MolBondDirectionError> {
        match direction {
            BondDirection::None => return Err(MolBondDirectionError::DirectionRequired),
            BondDirection::Other => {
                return Err(MolBondDirectionError::UnsupportedDirection { direction });
            }
            BondDirection::BeginWedge
            | BondDirection::BeginDash
            | BondDirection::EndUpRight
            | BondDirection::EndDownRight => {}
        }
        if order != BondOrder::Single || aromatic {
            return Err(
                MolBondDirectionError::DirectionRequiresNonAromaticSingleBond {
                    direction,
                    order,
                    aromatic,
                },
            );
        }
        Ok(Self {
            start,
            end,
            order,
            aromatic,
            stereo: BondStereo::None,
            direction,
            stereo_atoms: None,
        })
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

    /// Return the exact bond stereo fact.
    #[must_use]
    pub const fn stereo(&self) -> BondStereo {
        self.stereo
    }

    /// Return the exact bond direction fact.
    #[must_use]
    pub const fn direction(&self) -> BondDirection {
        self.direction
    }

    /// Return the optional source stereo reference pair.
    #[must_use]
    pub const fn stereo_atoms(&self) -> Option<(usize, usize)> {
        self.stereo_atoms
    }

    pub(crate) const fn from_native(
        start: usize,
        end: usize,
        order: BondOrder,
        aromatic: bool,
        stereo: BondStereo,
        direction: BondDirection,
        stereo_atoms: Option<(usize, usize)>,
    ) -> Self {
        Self {
            start,
            end,
            order,
            aromatic,
            stereo,
            direction,
            stereo_atoms,
        }
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

/// Complete immutable SMILES molecule returned by the ABI-4 native adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct SmilesMolecule {
    canonical_smiles: String,
    molecule: MolGraph,
}

impl SmilesMolecule {
    /// Construct a complete engine-independent SMILES result.
    pub fn new(
        canonical_smiles: impl Into<String>,
        molecule: MolGraph,
    ) -> Result<Self, MolGraphError> {
        let canonical_smiles = canonical_smiles.into();
        if canonical_smiles.is_empty() || canonical_smiles.as_bytes().contains(&0) {
            return Err(MolGraphError::InvalidCanonicalSmiles);
        }
        Ok(Self {
            canonical_smiles,
            molecule,
        })
    }

    /// Return the canonical label produced by the selected native profile.
    #[must_use]
    pub fn canonical_smiles(&self) -> &str {
        &self.canonical_smiles
    }

    /// Return all graph facts and atom-order-aligned coordinates.
    #[must_use]
    pub fn molecule(&self) -> &MolGraph {
        &self.molecule
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
        for (bond_index, bond) in bonds.iter().enumerate() {
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
            if matches!(
                bond.direction,
                BondDirection::BeginWedge
                    | BondDirection::BeginDash
                    | BondDirection::EndUpRight
                    | BondDirection::EndDownRight
            ) && (bond.order != BondOrder::Single || bond.aromatic)
            {
                return Err(MolGraphError::DirectedBondRequiresNonAromaticSingleBond {
                    bond_index,
                    direction: bond.direction,
                    order: bond.order,
                    aromatic: bond.aromatic,
                });
            }
            if let Some((first, second)) = bond.stereo_atoms {
                if first >= atoms.len()
                    || second >= atoms.len()
                    || first == second
                    || first == bond.start
                    || first == bond.end
                    || second == bond.start
                    || second == bond.end
                {
                    return Err(MolGraphError::InvalidStereoReferences);
                }
            } else if bond.stereo != BondStereo::None {
                return Err(MolGraphError::MissingStereoReferences);
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
    /// A canonical SMILES result must be nonempty and contain no NUL byte.
    #[error("canonical SMILES must be nonempty and contain no NUL byte")]
    InvalidCanonicalSmiles,
    /// The value does not identify a supported element.
    #[error("unsupported atomic number: {value}")]
    UnsupportedAtomicNumber { value: u8 },
    /// The value is not one of the 118 canonical case-sensitive element symbols.
    #[error("unsupported element symbol: {value}")]
    UnsupportedElementSymbol { value: String },
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
    /// A modeled direction can only annotate a non-aromatic single bond.
    #[error(
        "bond {bond_index} direction {direction:?} requires a non-aromatic single bond, received {order:?} aromatic={aromatic}"
    )]
    DirectedBondRequiresNonAromaticSingleBond {
        /// Source-order bond position.
        bond_index: usize,
        /// Rejected native direction.
        direction: BondDirection,
        /// Rejected bond order.
        order: BondOrder,
        /// Rejected aromaticity flag.
        aromatic: bool,
    },
    /// A stereo-coded bond must name two distinct non-endpoint reference atoms.
    #[error("bond stereo references are invalid")]
    InvalidStereoReferences,
    /// Stereo values other than `None` need a pair of reference atoms.
    #[error("bond stereo requires source atom references")]
    MissingStereoReferences,
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
    fn public_directed_bonds_preserve_each_supported_direction() {
        for direction in [
            BondDirection::BeginWedge,
            BondDirection::BeginDash,
            BondDirection::EndUpRight,
            BondDirection::EndDownRight,
        ] {
            let bond = MolBond::directed(2, 0, BondOrder::Single, false, direction)
                .expect("supported directional single bond");
            assert_eq!((bond.start(), bond.end()), (2, 0));
            assert_eq!(bond.order(), BondOrder::Single);
            assert!(!bond.is_aromatic());
            assert_eq!(bond.direction(), direction);
        }
    }

    #[test]
    fn public_directed_bonds_refuse_unmodeled_or_invalid_requests() {
        assert_eq!(
            MolBond::directed(0, 1, BondOrder::Single, false, BondDirection::None),
            Err(MolBondDirectionError::DirectionRequired)
        );
        assert_eq!(
            MolBond::directed(0, 1, BondOrder::Single, false, BondDirection::Other),
            Err(MolBondDirectionError::UnsupportedDirection {
                direction: BondDirection::Other,
            })
        );
        assert_eq!(
            MolBond::directed(0, 1, BondOrder::Double, false, BondDirection::BeginWedge),
            Err(
                MolBondDirectionError::DirectionRequiresNonAromaticSingleBond {
                    direction: BondDirection::BeginWedge,
                    order: BondOrder::Double,
                    aromatic: false,
                }
            )
        );
        assert_eq!(
            MolBond::directed(0, 1, BondOrder::Single, true, BondDirection::BeginDash),
            Err(
                MolBondDirectionError::DirectionRequiresNonAromaticSingleBond {
                    direction: BondDirection::BeginDash,
                    order: BondOrder::Single,
                    aromatic: true,
                }
            )
        );
    }

    #[test]
    fn graph_rejects_invalid_native_modeled_directions_but_accepts_other() {
        let plain = carbon(false);
        let aromatic = carbon(true);
        for (bond, expected) in [
            (
                MolBond::from_native(
                    0,
                    1,
                    BondOrder::Double,
                    false,
                    BondStereo::None,
                    BondDirection::BeginWedge,
                    None,
                ),
                MolGraphError::DirectedBondRequiresNonAromaticSingleBond {
                    bond_index: 0,
                    direction: BondDirection::BeginWedge,
                    order: BondOrder::Double,
                    aromatic: false,
                },
            ),
            (
                MolBond::from_native(
                    0,
                    1,
                    BondOrder::Single,
                    true,
                    BondStereo::None,
                    BondDirection::EndDownRight,
                    None,
                ),
                MolGraphError::DirectedBondRequiresNonAromaticSingleBond {
                    bond_index: 0,
                    direction: BondDirection::EndDownRight,
                    order: BondOrder::Single,
                    aromatic: true,
                },
            ),
        ] {
            let atoms = if bond.is_aromatic() {
                vec![aromatic.clone(), aromatic.clone()]
            } else {
                vec![plain.clone(), plain.clone()]
            };
            assert_eq!(MolGraph::new(atoms, vec![bond], None), Err(expected));
        }

        let other = MolBond::from_native(
            0,
            1,
            BondOrder::Double,
            false,
            BondStereo::None,
            BondDirection::Other,
            None,
        );
        assert!(MolGraph::new(vec![plain.clone(), plain], vec![other], None).is_ok());
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
