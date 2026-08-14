//! Ordered SDF record facts accepted by the native RDKit writer.

use std::collections::BTreeSet;

use thiserror::Error;

use crate::{MolGraph, SmilesMolecule};

/// One ordered, text-valued SDF property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdfProperty {
    name: String,
    value: String,
}

impl SdfProperty {
    /// Construct a property representable by RDKit's SD writer.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Result<Self, SdfError> {
        let name = name.into();
        let value = value.into();
        if name.is_empty() || name.contains(['\0', '\r', '\n']) {
            return Err(SdfError::InvalidPropertyName);
        }
        if value.as_bytes().contains(&0) || value.contains("\n\n") || value.contains("\r\n\r\n") {
            return Err(SdfError::InvalidPropertyValue);
        }
        Ok(Self { name, value })
    }

    /// Return the property name exactly as supplied.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the property value exactly as supplied.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// One molecule, title, and ordered property sequence for SDF export.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfRecord {
    molecule: MolGraph,
    title: String,
    properties: Vec<SdfProperty>,
}

/// One molecule, title, and ordered property sequence parsed from SDF input.
///
/// Unlike [`SdfRecord`], imported records may contain repeated property names.
/// They are retained as separate ordered entries because the source file did.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportedSdfRecord {
    molecule: SmilesMolecule,
    title: String,
    properties: Vec<SdfProperty>,
}

impl ImportedSdfRecord {
    pub(crate) fn from_native(
        molecule: SmilesMolecule,
        title: String,
        properties: Vec<SdfProperty>,
    ) -> Self {
        Self {
            molecule,
            title,
            properties,
        }
    }

    /// Return the complete owned molecule and canonical SMILES.
    #[must_use]
    pub fn molecule(&self) -> &SmilesMolecule {
        &self.molecule
    }

    /// Return the first-line record title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return properties in exact source encounter order, including duplicates.
    #[must_use]
    pub fn properties(&self) -> &[SdfProperty] {
        &self.properties
    }
}

impl SdfRecord {
    /// Construct one record without allowing ambiguous duplicate property names.
    pub fn new(
        molecule: MolGraph,
        title: impl Into<String>,
        properties: Vec<SdfProperty>,
    ) -> Result<Self, SdfError> {
        let title = title.into();
        if title.contains(['\0', '\r', '\n']) {
            return Err(SdfError::InvalidTitle);
        }
        if molecule.coordinates().is_none() {
            return Err(SdfError::CoordinatesRequired);
        }
        let mut names = BTreeSet::new();
        for property in &properties {
            if !names.insert(property.name()) {
                return Err(SdfError::DuplicatePropertyName {
                    name: property.name().to_owned(),
                });
            }
        }
        Ok(Self {
            molecule,
            title,
            properties,
        })
    }

    /// Return the complete immutable molecule.
    #[must_use]
    pub fn molecule(&self) -> &MolGraph {
        &self.molecule
    }

    /// Return the first-line record title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return properties in authored output order.
    #[must_use]
    pub fn properties(&self) -> &[SdfProperty] {
        &self.properties
    }
}

/// A title or property cannot be represented without silent RDKit omission.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SdfError {
    /// A record title would break the first line.
    #[error("SDF title contains NUL or a line break")]
    InvalidTitle,
    /// A property name is empty or would break its header line.
    #[error("SDF property name is empty or contains NUL or a line break")]
    InvalidPropertyName,
    /// A property value contains NUL or an unrepresentable blank line.
    #[error("SDF property value contains NUL or an unrepresentable blank line")]
    InvalidPropertyValue,
    /// A property map cannot retain duplicate ordered names.
    #[error("SDF property name is duplicated: {name}")]
    DuplicatePropertyName {
        /// The repeated property name.
        name: String,
    },
    /// SDF export needs the atom-aligned coordinates written in each record.
    #[error("SDF export requires one coordinate for every atom")]
    CoordinatesRequired,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtomicNumber, Coordinates, MolAtom, Point2};

    fn molecule() -> MolGraph {
        MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::try_from(6).expect("carbon is supported"),
                    Some(0),
                    None,
                    None,
                    false,
                )
                .expect("carbon is valid"),
            ],
            Vec::new(),
            Some(Coordinates::new(vec![
                Point2::new(0.0, 0.0).expect("origin is finite"),
            ])),
        )
        .expect("single carbon graph is valid")
    }

    #[test]
    fn records_preserve_property_order_and_reject_silent_writer_loss() {
        let second = SdfProperty::new("second", "line one\nline two").expect("valid property");
        let first = SdfProperty::new("first", "").expect("empty value is representable");
        let record = SdfRecord::new(
            molecule(),
            "record title",
            vec![second.clone(), first.clone()],
        )
        .expect("ordered record is valid");
        assert_eq!(record.properties(), &[second, first]);

        assert_eq!(
            SdfProperty::new("bad", "line one\n\nline three"),
            Err(SdfError::InvalidPropertyValue),
        );
        assert_eq!(
            SdfRecord::new(
                molecule(),
                "title",
                vec![
                    SdfProperty::new("same", "one").expect("valid"),
                    SdfProperty::new("same", "two").expect("valid"),
                ],
            ),
            Err(SdfError::DuplicatePropertyName {
                name: "same".to_owned(),
            }),
        );
    }
}
