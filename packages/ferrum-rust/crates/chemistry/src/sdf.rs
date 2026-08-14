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

/// Compose one exact SDF record around a validated native Molfile block.
///
/// Property blocks are emitted directly by Ferrum so repeated names retain
/// their source order instead of passing through a map-valued chemistry API.
///
/// # Errors
///
/// Returns [`SdfError`] when the Molfile or property grammar is not exactly
/// representable, the output size overflows, or output storage cannot be
/// reserved.
pub fn compose_sdf_record(molblock: &str, properties: &[SdfProperty]) -> Result<String, SdfError> {
    if molblock.as_bytes().contains(&0)
        || molblock.contains('\r')
        || !molblock.ends_with("M  END\n")
    {
        return Err(SdfError::InvalidMolblockRecord);
    }
    let mut output_length = molblock
        .len()
        .checked_add("$$$$\n".len())
        .ok_or(SdfError::OutputSizeOverflow)?;
    for property in properties {
        if !valid_record_property_name(property.name()) {
            return Err(SdfError::InvalidPropertyName);
        }
        if !valid_record_property_value(property.value()) {
            return Err(SdfError::InvalidPropertyValue);
        }
        let property_length = ">  <"
            .len()
            .checked_add(property.name().len())
            .and_then(|length| length.checked_add(">\n".len()))
            .and_then(|length| length.checked_add(property.value().len()))
            .and_then(|length| length.checked_add("\n\n".len()))
            .ok_or(SdfError::OutputSizeOverflow)?;
        output_length = output_length
            .checked_add(property_length)
            .ok_or(SdfError::OutputSizeOverflow)?;
    }

    let mut output = String::new();
    output
        .try_reserve_exact(output_length)
        .map_err(|_| SdfError::ResourceAllocation)?;
    output.push_str(molblock);
    for property in properties {
        output.push_str(">  <");
        output.push_str(property.name());
        output.push_str(">\n");
        output.push_str(property.value());
        output.push_str("\n\n");
    }
    output.push_str("$$$$\n");
    Ok(output)
}

fn valid_record_property_name(name: &str) -> bool {
    !name.is_empty() && !name.contains(['\0', '\r', '\n', '<', '>'])
}

fn valid_record_property_value(value: &str) -> bool {
    !value.as_bytes().contains(&0)
        && !value.contains('\r')
        && !value.contains("\n\n")
        && !value.ends_with('\n')
        && !value.lines().any(|line| line == "$$$$")
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
    #[error("SDF property name is empty or contains NUL, a line break, or an angle bracket")]
    InvalidPropertyName,
    /// A property value cannot be represented by the closed record grammar.
    #[error("SDF property value is not representable by the exact one-record grammar")]
    InvalidPropertyValue,
    /// A native Molfile record must be NUL-free LF text ending at `M  END`.
    #[error("SDF composition requires one LF-terminated native Molfile record")]
    InvalidMolblockRecord,
    /// Exact SDF output length cannot be represented by this process.
    #[error("SDF output size overflows addressable storage")]
    OutputSizeOverflow,
    /// Exact SDF output storage could not be reserved.
    #[error("SDF output storage could not be reserved")]
    ResourceAllocation,
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

        let repeated = [
            SdfProperty::new("same", "one").expect("valid"),
            SdfProperty::new("same", "two").expect("valid"),
        ];
        assert_eq!(
            compose_sdf_record("title\nprogram\ncomment\nM  END\n", &repeated)
                .expect("record grammar is exact"),
            concat!(
                "title\nprogram\ncomment\nM  END\n",
                ">  <same>\none\n\n",
                ">  <same>\ntwo\n\n",
                "$$$$\n",
            ),
        );
        for property in [
            SdfProperty::new("bad>name", "value").expect("imported name remains retainable"),
            SdfProperty::new("field", "value\n").expect("imported trailing LF remains retainable"),
            SdfProperty::new("field", "$$$$").expect("imported terminator text remains retainable"),
        ] {
            assert!(matches!(
                compose_sdf_record("title\nprogram\ncomment\nM  END\n", &[property]),
                Err(SdfError::InvalidPropertyName | SdfError::InvalidPropertyValue),
            ));
        }
    }
}
