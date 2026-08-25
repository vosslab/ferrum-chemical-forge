//! Closed molecule-insertion facts accepted by the document authority.

use std::collections::BTreeSet;

use thiserror::Error;

use super::chemistry::canonicalize_stereo_reports_for_molecule;
use super::{DocumentBondPresentationV1, Point3V1};
use super::{
    DocumentStereoDepictionReportV1, DocumentStereoSemanticReportV1, DocumentStereoSemanticsErrorV1,
};

/// One atom in a complete detached molecule insertion.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeInsertionAtomV1 {
    element: String,
    position: Point3V1,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
}

impl MoleculeInsertionAtomV1 {
    /// Construct one atom from explicit CDML-representable facts.
    pub fn new(
        element: impl Into<String>,
        position: Point3V1,
        formal_charge: Option<i32>,
        isotope: Option<u16>,
        explicit_hydrogens: Option<u16>,
    ) -> Result<Self, MoleculeInsertionV1Error> {
        let element = element.into();
        if element.is_empty()
            || element
                .chars()
                .any(|character| !character.is_ascii_alphabetic())
        {
            return Err(MoleculeInsertionV1Error::InvalidElement { element });
        }
        if isotope == Some(0) {
            return Err(MoleculeInsertionV1Error::ZeroIsotope);
        }
        Ok(Self {
            element,
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
        })
    }

    /// Return the canonical element spelling.
    #[must_use]
    pub fn element(&self) -> &str {
        &self.element
    }

    /// Return the finite document position.
    #[must_use]
    pub fn position(&self) -> Point3V1 {
        self.position
    }

    /// Return the optional nonzero formal charge.
    #[must_use]
    pub fn formal_charge(&self) -> Option<i32> {
        self.formal_charge
    }

    /// Return the optional positive isotope mass number.
    #[must_use]
    pub fn isotope(&self) -> Option<u16> {
        self.isotope
    }

    /// Return the optional positive explicit-hydrogen count.
    #[must_use]
    pub fn explicit_hydrogens(&self) -> Option<u16> {
        self.explicit_hydrogens
    }
}

/// A CDML bond order that can be persisted without approximation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentBondOrderV1 {
    /// One shared electron pair.
    Single,
    /// Two shared electron pairs.
    Double,
    /// Three shared electron pairs.
    Triple,
}

impl DocumentBondOrderV1 {
    pub(crate) const fn cdml_token(self) -> &'static str {
        match self {
            Self::Single => "n1",
            Self::Double => "n2",
            Self::Triple => "n3",
        }
    }
}

/// One zero-based bond in a complete molecule insertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MoleculeInsertionBondV1 {
    start: usize,
    end: usize,
    order: DocumentBondOrderV1,
    presentation: DocumentBondPresentationV1,
}

impl MoleculeInsertionBondV1 {
    /// Construct one bond. Endpoint range and duplicate-edge checks occur on the molecule.
    #[must_use]
    pub const fn new(start: usize, end: usize, order: DocumentBondOrderV1) -> Self {
        Self {
            start,
            end,
            order,
            presentation: DocumentBondPresentationV1::Normal(order),
        }
    }

    /// Construct one bond with its exact persisted presentation.
    ///
    /// Directed wedge presentations remain single covalent bonds while retaining
    /// their authored endpoint direction through CDML serialization.
    #[must_use]
    pub const fn new_with_presentation(
        start: usize,
        end: usize,
        presentation: DocumentBondPresentationV1,
    ) -> Self {
        let order = match presentation {
            DocumentBondPresentationV1::Normal(order) => order,
            DocumentBondPresentationV1::SolidWedge | DocumentBondPresentationV1::HashedWedge => {
                DocumentBondOrderV1::Single
            }
        };
        Self {
            start,
            end,
            order,
            presentation,
        }
    }

    /// Return the first zero-based atom index.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Return the second zero-based atom index.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Return the exact persisted bond order.
    #[must_use]
    pub const fn order(&self) -> DocumentBondOrderV1 {
        self.order
    }

    /// Return the exact authored presentation used for CDML persistence.
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.presentation
    }
}

/// One complete, validated molecule ready for session-owned identity allocation.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeInsertionV1 {
    atoms: Vec<MoleculeInsertionAtomV1>,
    bonds: Vec<MoleculeInsertionBondV1>,
    name: Option<String>,
}

/// One complete molecule insertion together with optional durable semantics.
///
/// `MoleculeInsertionV1` remains restricted to topology and depiction.  This
/// request carries facts which are independently serialized with the same
/// generic document transition.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeInsertionRequestV1 {
    molecule: MoleculeInsertionV1,
    stereo_semantics: Option<DocumentStereoSemanticReportV1>,
    stereo_depictions: Option<DocumentStereoDepictionReportV1>,
}

impl MoleculeInsertionRequestV1 {
    /// Build an ordinary topology-only insertion request.
    #[must_use]
    pub const fn new(molecule: MoleculeInsertionV1) -> Self {
        Self {
            molecule,
            stereo_semantics: None,
            stereo_depictions: None,
        }
    }

    /// Build one insertion request retaining an admitted stereo semantic report.
    pub fn with_stereo_semantics(
        molecule: MoleculeInsertionV1,
        stereo_semantics: DocumentStereoSemanticReportV1,
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        Self::with_stereo_reports(molecule, Some(stereo_semantics), None)
    }

    /// Build one insertion request retaining distinct admitted stereo reports.
    pub fn with_stereo_reports(
        molecule: MoleculeInsertionV1,
        stereo_semantics: Option<DocumentStereoSemanticReportV1>,
        stereo_depictions: Option<DocumentStereoDepictionReportV1>,
    ) -> Result<Self, DocumentStereoSemanticsErrorV1> {
        let (stereo_semantics, stereo_depictions) = canonicalize_stereo_reports_for_molecule(
            &molecule,
            stereo_semantics,
            stereo_depictions,
        )?;
        Ok(Self {
            molecule,
            stereo_semantics,
            stereo_depictions,
        })
    }

    /// Return the topology and depiction payload.
    #[must_use]
    pub const fn molecule(&self) -> &MoleculeInsertionV1 {
        &self.molecule
    }

    /// Return the optional source semantic report.
    #[must_use]
    pub const fn stereo_semantics(&self) -> Option<&DocumentStereoSemanticReportV1> {
        self.stereo_semantics.as_ref()
    }

    /// Return optional source-order stereo drawing facts.
    #[must_use]
    pub const fn stereo_depictions(&self) -> Option<&DocumentStereoDepictionReportV1> {
        self.stereo_depictions.as_ref()
    }
}

impl From<MoleculeInsertionV1> for MoleculeInsertionRequestV1 {
    fn from(molecule: MoleculeInsertionV1) -> Self {
        Self::new(molecule)
    }
}

impl MoleculeInsertionV1 {
    /// Validate a nonempty graph with bounded, unique, non-self bond endpoints.
    pub fn new(
        atoms: Vec<MoleculeInsertionAtomV1>,
        bonds: Vec<MoleculeInsertionBondV1>,
    ) -> Result<Self, MoleculeInsertionV1Error> {
        if atoms.is_empty() {
            return Err(MoleculeInsertionV1Error::EmptyMolecule);
        }
        let mut edges = BTreeSet::new();
        for bond in &bonds {
            if bond.start >= atoms.len() || bond.end >= atoms.len() {
                return Err(MoleculeInsertionV1Error::BondEndpointOutOfRange {
                    start: bond.start,
                    end: bond.end,
                    atom_count: atoms.len(),
                });
            }
            if bond.start == bond.end {
                return Err(MoleculeInsertionV1Error::SelfBond { atom: bond.start });
            }
            let edge = if bond.start < bond.end {
                (bond.start, bond.end)
            } else {
                (bond.end, bond.start)
            };
            if !edges.insert(edge) {
                return Err(MoleculeInsertionV1Error::DuplicateBond {
                    start: edge.0,
                    end: edge.1,
                });
            }
        }
        Ok(Self {
            atoms,
            bonds,
            name: None,
        })
    }

    /// Return atoms in their durable source order.
    #[must_use]
    pub fn atoms(&self) -> &[MoleculeInsertionAtomV1] {
        &self.atoms
    }

    /// Return bonds in their durable source order.
    #[must_use]
    pub fn bonds(&self) -> &[MoleculeInsertionBondV1] {
        &self.bonds
    }

    /// Attach one validated human-readable molecule name for CDML persistence.
    pub fn with_name(mut self, name: impl Into<String>) -> Result<Self, MoleculeInsertionV1Error> {
        let name = name.into();
        if name.is_empty()
            || name.chars().any(|character| {
                character.is_control() || matches!(character, '<' | '>' | '&' | '\'' | '"')
            })
        {
            return Err(MoleculeInsertionV1Error::InvalidName { name });
        }
        self.name = Some(name);
        Ok(self)
    }

    /// Return the optional validated molecule name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

/// Rejection of molecule facts before document mutation or identity allocation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum MoleculeInsertionV1Error {
    /// An element spelling is blank or contains non-letter characters.
    #[error("molecule insertion element is invalid: {element}")]
    InvalidElement { element: String },
    /// A molecule name cannot be safely persisted as an XML attribute.
    #[error("molecule insertion name is invalid")]
    InvalidName { name: String },
    /// Isotope zero is the absence sentinel rather than an authored isotope.
    #[error("molecule insertion isotope must be positive when present")]
    ZeroIsotope,
    /// A complete molecule must contain at least one atom.
    #[error("molecule insertion must contain at least one atom")]
    EmptyMolecule,
    /// A bond endpoint does not name an atom in this insertion.
    #[error("molecule insertion bond {start}-{end} exceeds atom count {atom_count}")]
    BondEndpointOutOfRange {
        start: usize,
        end: usize,
        atom_count: usize,
    },
    /// A bond cannot connect an atom to itself.
    #[error("molecule insertion bond cannot connect atom {atom} to itself")]
    SelfBond { atom: usize },
    /// The same undirected edge occurs more than once.
    #[error("molecule insertion contains duplicate bond {start}-{end}")]
    DuplicateBond { start: usize, end: usize },
}
