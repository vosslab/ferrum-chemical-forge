//! Closed, validated atom-properties intent for one atomic session operation.

use std::collections::HashSet;

use thiserror::Error;

use super::{PersistentId, PositiveFiniteV1, Rgb24V1};

/// One supported durable atom-property change.
#[derive(Clone, Debug, PartialEq)]
pub enum AtomPropertyChangeV1 {
    /// Replace the authored element spelling.
    Element(String),
    /// Replace formal charge; zero removes the optional CDML attribute.
    FormalCharge(i32),
    /// Replace or clear authored valence.
    Valence(Option<u16>),
    /// Replace or clear authored isotope mass number.
    Isotope(Option<u16>),
    /// Replace multiplicity; one removes the optional CDML attribute.
    Multiplicity(u16),
    /// Persist explicit atom visibility.
    Show(bool),
    /// Persist explicit hydrogen-label visibility.
    ShowHydrogens(bool),
    /// Replace the direct label-font size.
    FontSize(PositiveFiniteV1),
    /// Replace the direct label-font colour.
    LabelColor(Rgb24V1),
}

impl AtomPropertyChangeV1 {
    fn kind(&self) -> AtomPropertyKindV1 {
        match self {
            Self::Element(_) => AtomPropertyKindV1::Element,
            Self::FormalCharge(_) => AtomPropertyKindV1::FormalCharge,
            Self::Valence(_) => AtomPropertyKindV1::Valence,
            Self::Isotope(_) => AtomPropertyKindV1::Isotope,
            Self::Multiplicity(_) => AtomPropertyKindV1::Multiplicity,
            Self::Show(_) => AtomPropertyKindV1::Show,
            Self::ShowHydrogens(_) => AtomPropertyKindV1::ShowHydrogens,
            Self::FontSize(_) => AtomPropertyKindV1::FontSize,
            Self::LabelColor(_) => AtomPropertyKindV1::LabelColor,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum AtomPropertyKindV1 {
    Element,
    FormalCharge,
    Valence,
    Isotope,
    Multiplicity,
    Show,
    ShowHydrogens,
    FontSize,
    LabelColor,
}

impl AtomPropertyKindV1 {
    fn name(self) -> &'static str {
        match self {
            Self::Element => "element",
            Self::FormalCharge => "formal charge",
            Self::Valence => "valence",
            Self::Isotope => "isotope",
            Self::Multiplicity => "multiplicity",
            Self::Show => "visibility",
            Self::ShowHydrogens => "hydrogen visibility",
            Self::FontSize => "font size",
            Self::LabelColor => "label color",
        }
    }
}

/// One validated, source-ID-targeted atom-properties patch.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomPropertiesPatchV1 {
    atom_id: PersistentId,
    changes: Vec<AtomPropertyChangeV1>,
}

impl AtomPropertiesPatchV1 {
    /// Validate one complete dialog intent without reading or changing a document.
    pub fn new(
        atom_id: impl Into<String>,
        changes: Vec<AtomPropertyChangeV1>,
    ) -> Result<Self, AtomPropertiesPatchV1Error> {
        let atom_id = PersistentId::new(atom_id.into())
            .map_err(|_| AtomPropertiesPatchV1Error::InvalidAtomId)?;
        let mut kinds = HashSet::with_capacity(changes.len());
        for change in &changes {
            let kind = change.kind();
            if !kinds.insert(kind) {
                return Err(AtomPropertiesPatchV1Error::DuplicateChange {
                    property: kind.name(),
                });
            }
            match change {
                AtomPropertyChangeV1::Element(value) if !valid_atom_element(value) => {
                    return Err(AtomPropertiesPatchV1Error::InvalidElement);
                }
                AtomPropertyChangeV1::Isotope(Some(0)) => {
                    return Err(AtomPropertiesPatchV1Error::ZeroIsotope);
                }
                AtomPropertyChangeV1::Multiplicity(0) => {
                    return Err(AtomPropertiesPatchV1Error::ZeroMultiplicity);
                }
                _ => {}
            }
        }
        Ok(Self { atom_id, changes })
    }

    /// Return the durable authored atom identifier.
    #[must_use]
    pub fn atom_id(&self) -> &PersistentId {
        &self.atom_id
    }

    /// Return unique property changes in caller order.
    #[must_use]
    pub fn changes(&self) -> &[AtomPropertyChangeV1] {
        &self.changes
    }
}

pub(crate) fn valid_atom_element(value: &str) -> bool {
    !value.trim().is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphabetic())
}

/// Invalid atom-properties intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum AtomPropertiesPatchV1Error {
    /// The durable atom identifier is empty or otherwise invalid.
    #[error("atom properties require a valid persistent atom ID")]
    InvalidAtomId,
    /// The element spelling is not a nonblank plain ASCII name.
    #[error("atom element must be a nonblank plain element spelling")]
    InvalidElement,
    /// Isotope zero must be represented by clearing the authored fact.
    #[error("atom isotope must be absent or a positive mass number")]
    ZeroIsotope,
    /// Multiplicity must be positive.
    #[error("atom multiplicity must be positive")]
    ZeroMultiplicity,
    /// One closed property appeared more than once in a single patch.
    #[error("atom property change is duplicated: {property}")]
    DuplicateChange { property: &'static str },
}
