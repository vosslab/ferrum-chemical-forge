//! Fenced durable target adapters for live chemical presentation mutations.

use ferrum_document::{
    DocumentObjectIdV1, LiveChemicalPresentationTargetV1, PersistentId, TypedClass,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString, PyTuple};

use super::document_error_binding::{digest_conflict_error, document_object_id, document_result};
use super::document_session_binding::PyDocumentSession;
use super::session_operation_result_binding::PySessionOperationResultV1;

/// Resolve selected durable children of one durable molecule to typed-operation IDs.
pub(crate) fn molecule_member_source_ids(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    molecule_object_id: &DocumentObjectIdV1,
    object_ids: &Bound<'_, PyAny>,
    expected_class: TypedClass,
    label: &str,
) -> PyResult<Vec<PersistentId>> {
    let object_ids = durable_object_ids(py, object_ids, label)?;
    document_result(
        py,
        session
            .lower_live_chemical_members_v1(molecule_object_id, &object_ids, expected_class)
            .map_err(ferrum_document::DocumentSessionError::Operation),
    )
}

#[pymethods]
impl PyDocumentSession {
    /// Apply one fenced durable atom-properties patch owned by a live molecule.
    fn set_atom_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let (_, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation =
            super::document_operation_binding::atom_properties_operation(py, atom_id, changes)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Assign one fenced durable atom number owned by a live molecule.
    fn set_atom_number_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
        number: &Bound<'_, PyAny>,
        show_number: &Bound<'_, PyAny>,
    ) -> PyResult<PySessionOperationResultV1> {
        let (molecule_id, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation = super::document_operation_binding::atom_number_operation(
            py,
            molecule_id,
            atom_id,
            number,
            show_number,
        )?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Clear one fenced durable atom number owned by a live molecule.
    fn clear_atom_number_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
    ) -> PyResult<PySessionOperationResultV1> {
        let (molecule_id, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation =
            super::document_operation_binding::clear_atom_number_operation(molecule_id, atom_id);
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply one fenced durable atom mark owned by a live molecule.
    fn apply_atom_mark_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
        action: PyRef<'_, super::atom_mark_binding::PyAtomMarkActionV1>,
        kind: PyRef<'_, super::atom_mark_binding::PyAtomMarkKindV1>,
        matching_mark_index: &Bound<'_, PyAny>,
    ) -> PyResult<PySessionOperationResultV1> {
        let (molecule_id, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation = super::document_operation_binding::atom_mark_operation(
            py,
            molecule_id,
            atom_id,
            action,
            kind,
            matching_mark_index,
        )?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Replace one fenced durable atom's element through its live owner address.
    fn set_atom_element_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
        element: String,
    ) -> PyResult<PySessionOperationResultV1> {
        let (_, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation = super::document_operation_binding::atom_element_operation(atom_id, element);
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Replace one fenced durable atom's finite Cartesian point.
    fn set_atom_position_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
        x: f64,
        y: f64,
        z: f64,
    ) -> PyResult<PySessionOperationResultV1> {
        let (_, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation =
            super::document_operation_binding::atom_position_operation(py, atom_id, x, y, z)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Delete one fenced durable atom and its incident bonds through its live owner address.
    fn delete_atom_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        atom_object_id: String,
    ) -> PyResult<PySessionOperationResultV1> {
        let (_, atom_id) = fenced_live_atom_source_address(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            atom_object_id,
        )?;
        let operation = super::document_operation_binding::delete_atom_operation(atom_id);
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply a fenced durable geometric-presentation property patch.
    fn set_geometric_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        presentation_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let presentation_id = presentation_source_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            presentation_object_id,
            LiveChemicalPresentationTargetV1::Geometric,
        )?;
        let operation = super::geometric_properties_binding::set_geometric_properties(
            py,
            presentation_id,
            changes,
        )?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply a fenced durable Wavy-presentation property patch.
    fn set_wavy_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        wavy_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let wavy_id = presentation_source_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            wavy_object_id,
            LiveChemicalPresentationTargetV1::Wavy,
        )?;
        let operation = super::wavy_properties_binding::set_wavy_properties(py, wavy_id, changes)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply a fenced durable complete-bracket-pair property patch.
    fn set_bracket_pair_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        member_object_ids: &Bound<'_, PyTuple>,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let member_object_ids =
            durable_object_ids(py, member_object_ids.as_any(), "bracket members")?;
        let member_object_ids: [DocumentObjectIdV1; 2] =
            member_object_ids.try_into().map_err(|_| {
                PyValueError::new_err(
                    "bracket members must contain exactly the durable left and right members",
                )
            })?;
        let pair_id = presentation_bracket_pair_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            &member_object_ids,
        )?;
        let operation = super::bracket_binding::set_bracket_properties(py, pair_id, changes)?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }

    /// Apply one fenced durable bond-properties patch owned by a live molecule.
    fn set_bond_properties_v1(
        &mut self,
        py: Python<'_>,
        expected_revision: u64,
        expected_digest_hex: String,
        molecule_object_id: String,
        bond_object_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<PySessionOperationResultV1> {
        let bond_id = fenced_molecule_member_source_id(
            py,
            &self.session,
            expected_revision,
            &expected_digest_hex,
            molecule_object_id,
            bond_object_id,
            TypedClass::Bond,
            "bond",
        )?;
        let operation = super::document_operation_binding::bond_properties_operation(
            py,
            bond_id.as_str().to_owned(),
            changes,
        )?;
        document_result(
            py,
            self.session
                .apply_document_operation_v1(expected_revision, operation),
        )
        .map(Into::into)
    }
}

fn durable_object_ids(
    py: Python<'_>,
    values: &Bound<'_, PyAny>,
    label: &str,
) -> PyResult<Vec<DocumentObjectIdV1>> {
    if !values.is_exact_instance_of::<PyTuple>() {
        return Err(PyValueError::new_err(format!(
            "{label} must be an exact built-in tuple of durable object IDs"
        )));
    }
    let values = values.cast::<PyTuple>()?;
    let mut object_ids = Vec::new();
    object_ids
        .try_reserve_exact(values.len())
        .map_err(|_| PyValueError::new_err("live target resolution could not reserve storage"))?;
    for value in values.iter() {
        let value = value.cast::<PyString>().map_err(|_| {
            PyValueError::new_err(format!(
                "{label} must be an exact built-in tuple of durable object IDs"
            ))
        })?;
        let value = value
            .to_str()
            .map_err(|_| PyValueError::new_err(format!("{label} must be valid UTF-8 text")))?;
        let mut copied = String::new();
        copied.try_reserve_exact(value.len()).map_err(|_| {
            PyValueError::new_err("live target resolution could not reserve storage")
        })?;
        copied.push_str(value);
        object_ids.push(document_object_id(py, copied)?);
    }
    Ok(object_ids)
}

fn presentation_source_id(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    expected_revision: u64,
    expected_digest_hex: &str,
    object_id: String,
    target: LiveChemicalPresentationTargetV1,
) -> PyResult<String> {
    require_live_fence(py, session, expected_revision, expected_digest_hex)?;
    let object_id = document_object_id(py, object_id)?;
    document_result(
        py,
        session
            .lower_live_chemical_presentation_target_v1(&object_id, target)
            .map_err(ferrum_document::DocumentSessionError::Operation),
    )
}

fn presentation_bracket_pair_id(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    expected_revision: u64,
    expected_digest_hex: &str,
    member_object_ids: &[DocumentObjectIdV1; 2],
) -> PyResult<String> {
    require_live_fence(py, session, expected_revision, expected_digest_hex)?;
    document_result(
        py,
        session
            .lower_live_bracket_pair_target_v1(member_object_ids)
            .map_err(ferrum_document::DocumentSessionError::Operation),
    )
}

fn fenced_molecule_member_source_id(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    expected_revision: u64,
    expected_digest_hex: &str,
    molecule_object_id: String,
    object_id: String,
    expected_class: TypedClass,
    label: &str,
) -> PyResult<PersistentId> {
    require_live_fence(py, session, expected_revision, expected_digest_hex)?;
    let molecule_object_id = document_object_id(py, molecule_object_id)?;
    let object_id = document_object_id(py, object_id)?;
    let mut object_ids = document_result(
        py,
        session
            .lower_live_chemical_members_v1(
                &molecule_object_id,
                std::slice::from_ref(&object_id),
                expected_class,
            )
            .map_err(ferrum_document::DocumentSessionError::Operation),
    )?;
    object_ids.pop().ok_or_else(|| {
        PyValueError::new_err(format!("live {label} target lowering returned no target"))
    })
}

fn fenced_live_atom_source_address(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    expected_revision: u64,
    expected_digest_hex: &str,
    molecule_object_id: String,
    atom_object_id: String,
) -> PyResult<(String, String)> {
    require_live_fence(py, session, expected_revision, expected_digest_hex)?;
    let molecule_object_id = document_object_id(py, molecule_object_id)?;
    let atom_object_id = document_object_id(py, atom_object_id)?;
    document_result(
        py,
        session
            .lower_live_chemical_member_address_v1(
                &molecule_object_id,
                &atom_object_id,
                TypedClass::Atom,
            )
            .map_err(ferrum_document::DocumentSessionError::Operation),
    )
    .map(|(molecule_id, atom_id)| (molecule_id.as_str().to_owned(), atom_id.as_str().to_owned()))
}

fn require_live_fence(
    py: Python<'_>,
    session: &ferrum_document::DocumentSession,
    expected_revision: u64,
    expected_digest_hex: &str,
) -> PyResult<()> {
    let snapshot = document_result(py, session.snapshot())?;
    if snapshot.revision() != expected_revision {
        return document_result(
            py,
            Err(ferrum_document::DocumentSessionError::RevisionConflict {
                expected: expected_revision,
                actual: snapshot.revision(),
            }),
        );
    }
    if hex_digest(snapshot.digest()) != expected_digest_hex {
        return Err(digest_conflict_error(
            py,
            expected_revision,
            snapshot.revision(),
        )?);
    }
    Ok(())
}

fn hex_digest(value: &[u8; 32]) -> String {
    let mut result = String::with_capacity(64);
    for byte in value {
        use std::fmt::Write as _;
        let _ = write!(result, "{byte:02x}");
    }
    result
}
