//! Closed ordered SDF record facts accepted by document insertion.

use thiserror::Error;

use super::MoleculeInsertionV1;

/// Ferrum-owned opaque extension namespace for lossless imported SDF metadata.
pub const SDF_IMPORT_NAMESPACE_V1: &str = "urn:ferrum-chemical-forge:sdf-import:v1";

/// One ordered SDF property retained with its imported molecule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdfPropertyInsertionV1 {
    name: String,
    value: String,
}

impl SdfPropertyInsertionV1 {
    /// Validate the same text grammar admitted by the native SDF boundary.
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, SdfRecordInsertionV1Error> {
        let name = name.into();
        let value = value.into();
        if name.is_empty() || name.contains(['\0', '\r', '\n']) {
            return Err(SdfRecordInsertionV1Error::InvalidPropertyName);
        }
        if value.as_bytes().contains(&0) || value.contains("\n\n") || value.contains("\r\n\r\n") {
            return Err(SdfRecordInsertionV1Error::InvalidPropertyValue);
        }
        Ok(Self { name, value })
    }

    /// Return the exact imported property name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the exact imported property value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// One complete molecule plus its exact ordered SDF record metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfRecordInsertionV1 {
    molecule: MoleculeInsertionV1,
    title: String,
    properties: Vec<SdfPropertyInsertionV1>,
}

impl SdfRecordInsertionV1 {
    /// Construct one record without dropping blank titles or repeated property names.
    pub fn new(
        molecule: MoleculeInsertionV1,
        title: impl Into<String>,
        properties: Vec<SdfPropertyInsertionV1>,
    ) -> Result<Self, SdfRecordInsertionV1Error> {
        let title = title.into();
        if title.contains(['\0', '\r', '\n']) {
            return Err(SdfRecordInsertionV1Error::InvalidTitle);
        }
        Ok(Self {
            molecule,
            title,
            properties,
        })
    }

    /// Return the complete handle-free molecule insertion.
    #[must_use]
    pub fn molecule(&self) -> &MoleculeInsertionV1 {
        &self.molecule
    }

    /// Return the exact first-line SDF title, including an empty title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return properties in exact source encounter order, including duplicates.
    #[must_use]
    pub fn properties(&self) -> &[SdfPropertyInsertionV1] {
        &self.properties
    }
}

/// A nonempty ordered SDF import committed as one document history entry.
#[derive(Clone, Debug, PartialEq)]
pub struct SdfRecordBatchInsertionV1 {
    records: Vec<SdfRecordInsertionV1>,
}

impl SdfRecordBatchInsertionV1 {
    /// Retain every imported record in exact source order.
    pub fn new(records: Vec<SdfRecordInsertionV1>) -> Result<Self, SdfRecordInsertionV1Error> {
        if records.is_empty() {
            return Err(SdfRecordInsertionV1Error::EmptyBatch);
        }
        Ok(Self { records })
    }

    /// Return every source-ordered record.
    #[must_use]
    pub fn records(&self) -> &[SdfRecordInsertionV1] {
        &self.records
    }
}

/// Rejection of SDF record metadata before document identity allocation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SdfRecordInsertionV1Error {
    /// A batch must retain at least one imported record.
    #[error("SDF record insertion batch must not be empty")]
    EmptyBatch,
    /// SDF titles are single-line NUL-free UTF-8 text.
    #[error("SDF record title contains a forbidden character")]
    InvalidTitle,
    /// Property names are nonempty single-line NUL-free UTF-8 text.
    #[error("SDF property name is invalid")]
    InvalidPropertyName,
    /// Property values cannot contain NUL or an embedded blank SDF line.
    #[error("SDF property value is invalid")]
    InvalidPropertyValue,
}
