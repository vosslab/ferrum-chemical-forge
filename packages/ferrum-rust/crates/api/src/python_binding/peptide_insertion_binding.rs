//! Qt-owned native peptide preparation bridge.

use ferrum_chemistry::{ChemistryError as RustChemistryError, NativeChemEngine};
use ferrum_document::{
    PeptideStructurePlanDocumentPreparationErrorV1, prepare_peptide_structure_plan_for_document_v1,
};
use ferrum_domain::{
    FerrumPeptideProfileV1, PeptideStructurePlanErrorV1, PeptideSyntaxError,
    build_peptide_structure_plan_v1, parse_one_letter_sequence,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyString;

use super::binding::FerrumError;
use super::geometry_binding::PyInsertionPlacementV1;
use super::molecule_insertion_binding::{PyMoleculeInsertionV1, structured_insertion_error};

create_exception!(ferrum_chem, FerrumPeptideInsertionError, FerrumError);
create_exception!(
    ferrum_chem,
    FerrumPeptideSyntaxError,
    FerrumPeptideInsertionError
);
create_exception!(
    ferrum_chem,
    UnsupportedFerrumPeptideProfileError,
    FerrumPeptideInsertionError
);

const OPERATION: &str = "prepare_ferrum_peptide_insertion_v1";

enum NativePreparationFailure {
    Load(RustChemistryError),
    Prepare(PeptideStructurePlanDocumentPreparationErrorV1),
}

/// Prepare a closed native peptide plan for ordinary Ferrum insertion.
///
/// This Qt-owned bridge accepts exact, nonempty uppercase one-letter syntax.
/// It builds the closed native-17 zwitterionic plan directly; no source grammar
/// is constructed or parsed.
#[pyfunction]
fn prepare_ferrum_peptide_insertion_v1(
    py: Python<'_>,
    sequence: &Bound<'_, PyString>,
    placement: PyRef<'_, PyInsertionPlacementV1>,
) -> PyResult<PyMoleculeInsertionV1> {
    let sequence = match parse_one_letter_sequence(sequence.to_str()?) {
        Ok(sequence) => sequence,
        Err(error) => return Err(syntax_error(py, error)?),
    };
    let plan = match build_peptide_structure_plan_v1(
        &sequence,
        FerrumPeptideProfileV1::Native17ZwitterionicTermini,
    ) {
        Ok(plan) => plan,
        Err(error) => return Err(map_plan_error(py, error)?),
    };
    let placement = placement.placement();
    let library_path = super::chemistry_binding::packaged_library_path(py, OPERATION)?;
    let worker_path = library_path.clone();
    let result = py.detach(move || {
        let engine =
            NativeChemEngine::load(&worker_path).map_err(NativePreparationFailure::Load)?;
        prepare_peptide_structure_plan_for_document_v1(&engine, &plan, placement)
            .map_err(NativePreparationFailure::Prepare)
    });
    match result {
        Ok(prepared) => PyMoleculeInsertionV1::from_prepared(prepared)
            .map_err(|error| FerrumPeptideInsertionError::new_err(error.to_string())),
        Err(NativePreparationFailure::Load(error)) => Err(
            super::chemistry_binding::map_load_error(py, OPERATION, &library_path, error)?,
        ),
        Err(NativePreparationFailure::Prepare(
            PeptideStructurePlanDocumentPreparationErrorV1::Chemistry(error),
        )) => Err(super::chemistry_binding::map_packaged_operation_error(
            py,
            OPERATION,
            &library_path,
            error,
        )?),
        Err(NativePreparationFailure::Prepare(error)) => {
            let py_error =
                structured_insertion_error(py, FerrumPeptideInsertionError::new_err, error)?;
            Err(py_error)
        }
    }
}

fn map_plan_error(py: Python<'_>, error: PeptideStructurePlanErrorV1) -> PyResult<PyErr> {
    match error {
        PeptideStructurePlanErrorV1::UnsupportedResidue {
            position,
            residue,
            profile,
        } => profile_error(py, position, residue.to_string(), profile),
        PeptideStructurePlanErrorV1::AllocationFailed => structured_insertion_error(
            py,
            FerrumPeptideInsertionError::new_err,
            PeptideStructurePlanErrorV1::AllocationFailed,
        ),
    }
}

fn syntax_error(py: Python<'_>, error: PeptideSyntaxError) -> PyResult<PyErr> {
    let reason = error.to_string();
    let py_error = FerrumPeptideSyntaxError::new_err(reason.clone());
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
) -> PyResult<PyErr> {
    let reason = format!(
        "residue {residue} at position {position} is unsupported by native peptide profile {profile}"
    );
    let py_error = UnsupportedFerrumPeptideProfileError::new_err(reason.clone());
    let value = py_error.value(py);
    value.setattr("reason", reason)?;
    value.setattr("position", position)?;
    value.setattr("residue", residue)?;
    value.setattr("profile", profile)?;
    Ok(py_error)
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "FerrumPeptideInsertionError",
        module.py().get_type::<FerrumPeptideInsertionError>(),
    )?;
    module.add(
        "FerrumPeptideSyntaxError",
        module.py().get_type::<FerrumPeptideSyntaxError>(),
    )?;
    module.add(
        "UnsupportedFerrumPeptideProfileError",
        module
            .py()
            .get_type::<UnsupportedFerrumPeptideProfileError>(),
    )?;
    module.add_function(wrap_pyfunction!(
        prepare_ferrum_peptide_insertion_v1,
        module
    )?)?;
    Ok(())
}
