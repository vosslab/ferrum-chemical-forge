//! Experimental Ferrum adapter for strict supported peptide templates.

use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::{
    PeptideTemplateMoleculeBuildErrorV1, build_supported_peptide_template_molecule_insertion_v1,
};
use ferrum_domain::PeptideSyntaxError;
use ferrum_domain::{
    PeptideTemplateInsertionErrorV1, compile_supported_peptide_template_request_v1,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::binding::FerrumError;
use super::geometry_binding::PyInsertionPlacementV1;
use super::smiles_insertion_binding::{PyMoleculeInsertionV1, map_build_error};

create_exception!(ferrum_chem, PeptideTemplateInsertionError, FerrumError);
create_exception!(
    ferrum_chem,
    PeptideTemplateSyntaxError,
    PeptideTemplateInsertionError
);
create_exception!(
    ferrum_chem,
    UnsupportedPeptideTemplateProfileError,
    PeptideTemplateInsertionError
);
create_exception!(
    ferrum_chem,
    PeptideTemplateResourceError,
    PeptideTemplateInsertionError
);

const OPERATION: &str = "prepare_supported_peptide_template_molecule_v1";

enum NativePreparationFailure {
    Load(RustChemistryError),
    Build(PeptideTemplateMoleculeBuildErrorV1),
}

/// Prepare a strict supported peptide template for ordinary Ferrum insertion.
///
/// Experimental internal-to-Ferrum API. Input is exact, nonempty uppercase
/// one-letter syntax; only `ACDEFGIKLMNQRSTVY` has a native V1 template. H, P,
/// and W are rejected before native loading.
#[pyfunction]
fn prepare_supported_peptide_template_molecule_v1(
    py: Python<'_>,
    sequence: &Bound<'_, PyString>,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyMoleculeInsertionV1> {
    let sequence_text = sequence.to_str()?;
    let request = match compile_supported_peptide_template_request_v1(sequence_text) {
        Ok(request) => request,
        Err(error) => return Err(map_preflight_error(py, error)?),
    };
    let placement = placement.placement();
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        build_supported_peptide_template_molecule_insertion_v1(&engine, &request, placement)
            .map_err(NativePreparationFailure::Build)
    });
    match result {
        Ok(insertion) => Ok(PyMoleculeInsertionV1::new(insertion)),
        Err(NativePreparationFailure::Load(error)) => Err(
            super::chemistry_binding::map_load_error(py, OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Build(PeptideTemplateMoleculeBuildErrorV1::Build(error))) => {
            Err(map_build_error(py, &library_path, error)?)
        }
    }
}

fn map_preflight_error(py: Python<'_>, error: PeptideTemplateInsertionErrorV1) -> PyResult<PyErr> {
    match error {
        PeptideTemplateInsertionErrorV1::Syntax(error) => syntax_error(py, error),
        PeptideTemplateInsertionErrorV1::NativeProfile {
            position,
            residue,
            profile,
            supported_alphabet,
        } => profile_error(
            py,
            position,
            residue.to_string(),
            profile,
            supported_alphabet,
        ),
        PeptideTemplateInsertionErrorV1::UnsupportedProfile(error) => {
            unexpected_profile_error(py, error.to_string())
        }
        PeptideTemplateInsertionErrorV1::ResourceAdmission {
            submitted_bytes,
            max_submitted_bytes,
        } => resource_error(
            py,
            error_message(submitted_bytes, max_submitted_bytes),
            submitted_bytes,
            max_submitted_bytes,
        ),
    }
}

fn unexpected_profile_error(py: Python<'_>, reason: String) -> PyResult<PyErr> {
    let py_error = PeptideTemplateInsertionError::new_err(reason.clone());
    py_error.value(py).setattr("reason", reason)?;
    Ok(py_error)
}

fn syntax_error(py: Python<'_>, error: PeptideSyntaxError) -> PyResult<PyErr> {
    let reason = error.to_string();
    let py_error = PeptideTemplateSyntaxError::new_err(reason.clone());
    let value = py_error.value(py);
    let (position, found, alphabet) = match error {
        PeptideSyntaxError::EmptySequence | PeptideSyntaxError::AllocationFailed => {
            (None, None, None)
        }
        PeptideSyntaxError::UnsupportedResidue {
            position,
            found,
            supported_alphabet,
        } => (
            Some(position),
            Some(found.to_string()),
            Some(supported_alphabet),
        ),
    };
    value.setattr("reason", reason)?;
    value.setattr("position", position)?;
    value.setattr("found", found)?;
    value.setattr("alphabet", alphabet)?;
    Ok(py_error)
}

fn profile_error(
    py: Python<'_>,
    position: usize,
    residue: String,
    profile: &str,
    supported_alphabet: &str,
) -> PyResult<PyErr> {
    let reason = format!(
        concat!(
            "residue {} at position {} is unsupported by native peptide-template ",
            "profile {}; supported alphabet is {}"
        ),
        residue, position, profile, supported_alphabet,
    );
    let py_error = UnsupportedPeptideTemplateProfileError::new_err(reason.clone());
    let value = py_error.value(py);
    value.setattr("reason", reason)?;
    value.setattr("position", position)?;
    value.setattr("residue", residue)?;
    value.setattr("profile", profile)?;
    value.setattr("supported_alphabet", supported_alphabet)?;
    Ok(py_error)
}

fn resource_error(
    py: Python<'_>,
    reason: String,
    submitted_bytes: usize,
    max_submitted_bytes: usize,
) -> PyResult<PyErr> {
    let py_error = PeptideTemplateResourceError::new_err(reason.clone());
    let value = py_error.value(py);
    value.setattr("reason", reason)?;
    value.setattr("submitted_bytes", submitted_bytes)?;
    value.setattr("max_submitted_bytes", max_submitted_bytes)?;
    Ok(py_error)
}

fn error_message(submitted_bytes: usize, max_submitted_bytes: usize) -> String {
    format!(
        concat!(
            "supported peptide-template input has {} bytes, above the ",
            "{}-byte V1 admission budget"
        ),
        submitted_bytes, max_submitted_bytes,
    )
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "PeptideTemplateInsertionError",
        module.py().get_type::<PeptideTemplateInsertionError>(),
    )?;
    module.add(
        "PeptideTemplateSyntaxError",
        module.py().get_type::<PeptideTemplateSyntaxError>(),
    )?;
    module.add(
        "UnsupportedPeptideTemplateProfileError",
        module
            .py()
            .get_type::<UnsupportedPeptideTemplateProfileError>(),
    )?;
    module.add(
        "PeptideTemplateResourceError",
        module.py().get_type::<PeptideTemplateResourceError>(),
    )?;
    module.add_function(wrap_pyfunction!(
        prepare_supported_peptide_template_molecule_v1,
        module
    )?)?;
    Ok(())
}
