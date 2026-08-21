//! Closed ordered interchange record facts accepted by document insertion.

use thiserror::Error;

use super::MoleculeInsertionV1;

/// Ferrum-owned extension namespace for lossless imported interchange metadata.
pub const INTERCHANGE_IMPORT_NAMESPACE_V1: &str = "urn:ferrum-chemical-forge:interchange-import:v1";

/// One ordered interchange property retained with its imported molecule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterchangePropertyInsertionV1 {
    name: String,
    value: String,
}

impl InterchangePropertyInsertionV1 {
    /// Validate the same text grammar admitted by the native SDF boundary.
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, InterchangeRecordInsertionV1Error> {
        let name = name.into();
        let value = value.into();
        if name.is_empty() || name.contains(['\0', '\r', '\n']) {
            return Err(InterchangeRecordInsertionV1Error::InvalidPropertyName);
        }
        if value.as_bytes().contains(&0) || value.contains("\n\n") || value.contains("\r\n\r\n") {
            return Err(InterchangeRecordInsertionV1Error::InvalidPropertyValue);
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

/// One complete molecule plus its exact ordered interchange record metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct InterchangeRecordInsertionV1 {
    molecule: MoleculeInsertionV1,
    title: String,
    properties: Vec<InterchangePropertyInsertionV1>,
}

impl InterchangeRecordInsertionV1 {
    /// Construct one record without dropping blank titles or repeated property names.
    pub fn new(
        molecule: MoleculeInsertionV1,
        title: impl Into<String>,
        properties: Vec<InterchangePropertyInsertionV1>,
    ) -> Result<Self, InterchangeRecordInsertionV1Error> {
        let title = title.into();
        if title.contains(['\0', '\r', '\n']) {
            return Err(InterchangeRecordInsertionV1Error::InvalidTitle);
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

    /// Return the exact imported title, including an empty title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return properties in exact source encounter order, including duplicates.
    #[must_use]
    pub fn properties(&self) -> &[InterchangePropertyInsertionV1] {
        &self.properties
    }
}

/// A nonempty ordered interchange import committed as one document history entry.
#[derive(Clone, Debug, PartialEq)]
pub struct InterchangeRecordBatchInsertionV1 {
    records: Vec<InterchangeRecordInsertionV1>,
}

impl InterchangeRecordBatchInsertionV1 {
    /// Retain every imported record in exact source order.
    pub fn new(
        records: Vec<InterchangeRecordInsertionV1>,
    ) -> Result<Self, InterchangeRecordInsertionV1Error> {
        if records.is_empty() {
            return Err(InterchangeRecordInsertionV1Error::EmptyBatch);
        }
        Ok(Self { records })
    }

    /// Return every source-ordered record.
    #[must_use]
    pub fn records(&self) -> &[InterchangeRecordInsertionV1] {
        &self.records
    }
}

/// Rejection of interchange record metadata before document identity allocation.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum InterchangeRecordInsertionV1Error {
    /// A batch must retain at least one imported record.
    #[error("interchange record insertion batch must not be empty")]
    EmptyBatch,
    /// Interchange titles are single-line NUL-free UTF-8 text.
    #[error("interchange record title contains a forbidden character")]
    InvalidTitle,
    /// Property names are nonempty single-line NUL-free UTF-8 text.
    #[error("interchange property name is invalid")]
    InvalidPropertyName,
    /// Property values cannot contain NUL or an embedded blank record line.
    #[error("interchange property value is invalid")]
    InvalidPropertyValue,
}
