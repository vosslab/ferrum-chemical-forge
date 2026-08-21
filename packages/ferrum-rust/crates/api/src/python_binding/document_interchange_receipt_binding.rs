//! Redacted Python facts for descriptor-issued interchange preparation.

use pyo3::prelude::*;
use serde::Serialize;

use super::document_error_binding::DocumentInputError;
use crate::interchange_import_v1::InterchangeImportRefusalV1;
use crate::protocol::DocumentInterchangeImportSummaryV1;

/// Safe, immutable outcome facts for one prepared local interchange document.
///
/// The source path, source bytes, source identifiers, and generated CDML stay
/// inside Rust.  This is deliberately data only: it has no commit capability.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "LocalInterchangeImportSummaryV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyLocalInterchangeImportSummaryV1 {
    #[pyo3(get)]
    format_id: String,
    #[pyo3(get)]
    profile_id: String,
    #[pyo3(get)]
    imported_record_count: u32,
    #[pyo3(get)]
    atom_count: u32,
    #[pyo3(get)]
    bond_count: u32,
    #[pyo3(get)]
    document_revision: u64,
    #[pyo3(get)]
    document_digest_hex: String,
    #[pyo3(get)]
    source_kind: String,
    #[pyo3(get)]
    source_identifiers_reallocated: bool,
    #[pyo3(get)]
    dropped_categories: Vec<String>,
}

impl PyLocalInterchangeImportSummaryV1 {
    pub(crate) fn from_summary(summary: &DocumentInterchangeImportSummaryV1) -> Self {
        Self {
            format_id: summary.format_id.clone(),
            profile_id: summary.profile_id.clone(),
            imported_record_count: summary.imported_record_count,
            atom_count: summary.atom_count,
            bond_count: summary.bond_count,
            document_revision: summary.document_revision,
            document_digest_hex: summary.document_digest_hex.clone(),
            source_kind: closed_name(summary.provenance.source_kind),
            source_identifiers_reallocated: summary.loss_report.source_identifiers_reallocated,
            dropped_categories: summary
                .loss_report
                .dropped_categories
                .iter()
                .copied()
                .map(closed_name)
                .collect(),
        }
    }
}

/// Closed, redacted interchange refusal facts for UI recovery decisions.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "LocalInterchangeRefusalV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyLocalInterchangeRefusalV1 {
    #[pyo3(get)]
    category: String,
    #[pyo3(get)]
    reason: String,
    #[pyo3(get)]
    recovery: String,
}

impl PyLocalInterchangeRefusalV1 {
    fn from_refusal(refusal: InterchangeImportRefusalV1) -> Self {
        Self {
            category: closed_name(refusal.category()),
            reason: closed_name(refusal.reason()),
            recovery: closed_name(refusal.recovery()),
        }
    }
}

pub(crate) fn local_interchange_refusal(
    py: Python<'_>,
    refusal: InterchangeImportRefusalV1,
) -> PyResult<PyErr> {
    let refusal = Py::new(py, PyLocalInterchangeRefusalV1::from_refusal(refusal))?;
    let error = DocumentInputError::new_err("document input rejected at interchange");
    let value = error.value(py);
    value.setattr("origin", "file")?;
    value.setattr("stage", "interchange")?;
    value.setattr("limit", py.None())?;
    value.setattr("actual", py.None())?;
    value.setattr("observed_at_least", py.None())?;
    value.setattr("refusal", refusal.clone_ref(py))?;
    value.setattr("category", refusal.bind(py).getattr("category")?)?;
    value.setattr("reason", refusal.bind(py).getattr("reason")?)?;
    value.setattr("recovery", refusal.bind(py).getattr("recovery")?)?;
    Ok(error)
}

fn closed_name<T: Serialize>(value: T) -> String {
    serde_json::to_value(value)
        .expect("closed interchange facts serialize")
        .as_str()
        .expect("closed interchange fact is a string")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use crate::interchange_import_v1::{
        CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1, InterchangeImportRefusalReasonV1,
        SDF_IMPORT_FORMAT_V1,
    };
    use crate::protocol::{
        DocumentInterchangeImportLossReportV1, DocumentInterchangeProvenanceV1,
        DocumentInterchangeSourceKindV1,
    };

    use super::*;

    #[test]
    fn generic_summary_and_refusal_facts_are_redacted_and_closed() {
        for (format_id, profile_id) in [
            (
                CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
                "cml_simple_molecule_v1",
            ),
            (SDF_IMPORT_FORMAT_V1, "sdf_v1"),
        ] {
            let summary = DocumentInterchangeImportSummaryV1 {
                format_id: format_id.to_owned(),
                profile_id: profile_id.to_owned(),
                imported_record_count: 1,
                atom_count: 2,
                bond_count: 1,
                document_revision: 1,
                document_digest_hex: "a".repeat(64),
                provenance: DocumentInterchangeProvenanceV1 {
                    format_id: format_id.to_owned(),
                    profile_id: profile_id.to_owned(),
                    source_kind: DocumentInterchangeSourceKindV1::RegularFile,
                },
                loss_report: DocumentInterchangeImportLossReportV1 {
                    source_identifiers_reallocated: true,
                    dropped_categories: Vec::new(),
                },
            };
            let facts = PyLocalInterchangeImportSummaryV1::from_summary(&summary);
            assert_eq!(facts.format_id, format_id);
            assert_eq!(facts.profile_id, profile_id);
            assert_eq!(facts.source_kind, "regular_file");
            assert!(facts.source_identifiers_reallocated);
        }

        let refusal =
            PyLocalInterchangeRefusalV1::from_refusal(InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::ChemistryRuntimeUnavailable,
            ));
        assert_eq!(refusal.category, "chemistry_unavailable");
        assert_eq!(refusal.reason, "chemistry_runtime_unavailable");
        assert_eq!(refusal.recovery, "install_chemistry_runtime");
    }
}
