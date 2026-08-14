//! Closed Python factories for authoritative Rust document operations.

use ferrum_document::{
    AtomMarkActionV1, AtomPropertiesPatchV1, BondPropertiesPatchV1, Point3V1, SessionOperation,
    SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyInt, PyTuple};

use crate::atom_mark_binding::{PyAtomMarkActionV1, PyAtomMarkKindV1};
use crate::atom_properties_binding::PyDocumentAtomPropertyChangeV1;
use crate::binding::{PyDocumentBondOrderV1, operation_validation_error, projection_error};
use crate::bond_properties_binding::PyDocumentBondPropertyChangeV1;
use crate::paper_properties_binding::{PyDocumentPaperPropertyChangeV1, validate_patch};

/// Closed V1 operation grammar for authoritative session mutations.
///
/// This value owns a Rust enum rather than a mapping or XML fragment. It can only
/// be created by a named factory and is consumed by no Python-side parser.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentOperationV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentOperationV1 {
    pub(crate) operation: SessionOperation,
}

#[pymethods]
impl PyDocumentOperationV1 {
    /// Build the V1 operation that replaces one existing atom's element spelling.
    #[staticmethod]
    fn set_atom_element(atom_id: String, element: String) -> Self {
        Self {
            operation: SessionOperation::V1(SessionOperationV1::SetAtomElement {
                atom_id,
                element,
            }),
        }
    }

    /// Build one complete unique-field atom-properties patch.
    #[staticmethod]
    fn set_atom_properties(
        py: Python<'_>,
        atom_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        if !changes.is_exact_instance_of::<PyTuple>() {
            return Err(operation_validation_error(
                py,
                "atom-properties changes must be an exact built-in tuple".to_owned(),
            ));
        }
        if changes.len() > 9 {
            return Err(operation_validation_error(
                py,
                "an atom-properties patch accepts at most nine unique changes".to_owned(),
            ));
        }
        let changes = changes
            .iter()
            .map(|value| {
                value
                    .extract::<PyRef<'_, PyDocumentAtomPropertyChangeV1>>()
                    .map(|value| value.change.clone())
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let patch = AtomPropertiesPatchV1::new(atom_id, changes)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self {
            operation: SessionOperation::V1(SessionOperationV1::SetAtomProperties { patch }),
        })
    }

    /// Build one complete unique-field document-global paper-properties patch.
    #[staticmethod]
    fn set_paper_properties(py: Python<'_>, changes: &Bound<'_, PyTuple>) -> PyResult<Self> {
        if !changes.is_exact_instance_of::<PyTuple>() {
            return Err(operation_validation_error(
                py,
                "paper-properties changes must be an exact built-in tuple".to_owned(),
            ));
        }
        if changes.len() > 7 {
            return Err(operation_validation_error(
                py,
                "a paper-properties patch accepts at most seven unique changes".to_owned(),
            ));
        }
        let changes = changes
            .iter()
            .map(|value| {
                value
                    .extract::<PyRef<'_, PyDocumentPaperPropertyChangeV1>>()
                    .map(|value| value.change.clone())
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let patch = validate_patch(py, changes)?;
        Ok(Self {
            operation: SessionOperation::V1(SessionOperationV1::SetPaperProperties { patch }),
        })
    }

    /// Build one positive atom-number assignment with explicit visibility.
    #[staticmethod]
    fn set_atom_number(
        py: Python<'_>,
        molecule_id: String,
        atom_id: String,
        number: &Bound<'_, PyAny>,
        show_number: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        if !number.is_exact_instance_of::<PyInt>() || number.is_instance_of::<PyBool>() {
            return Err(operation_validation_error(
                py,
                "atom number must be an exact positive integer".to_owned(),
            ));
        }
        let number = number
            .extract::<u64>()
            .map_err(|_| operation_validation_error(py, "atom number is outside u64".to_owned()))?;
        if number == 0 {
            return Err(operation_validation_error(
                py,
                "atom number must be positive".to_owned(),
            ));
        }
        if !show_number.is_exact_instance_of::<PyBool>() {
            return Err(operation_validation_error(
                py,
                "atom number visibility must be an exact bool".to_owned(),
            ));
        }
        Ok(Self {
            operation: SessionOperation::V1(SessionOperationV1::SetAtomNumber {
                molecule_id,
                atom_id,
                number: Some(number),
                show_number: Some(show_number.extract::<bool>()?),
            }),
        })
    }

    /// Build one exact atom-number clear operation.
    #[staticmethod]
    fn clear_atom_number(molecule_id: String, atom_id: String) -> Self {
        Self {
            operation: SessionOperation::V1(SessionOperationV1::SetAtomNumber {
                molecule_id,
                atom_id,
                number: None,
                show_number: None,
            }),
        }
    }

    /// Build one revision-bound add or remove operation for an authored atom mark.
    #[staticmethod]
    fn apply_atom_mark(
        py: Python<'_>,
        molecule_id: String,
        atom_id: String,
        action: PyRef<'_, PyAtomMarkActionV1>,
        kind: PyRef<'_, PyAtomMarkKindV1>,
        matching_mark_index: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let matching_mark_index = if matching_mark_index.is_none() {
            None
        } else {
            if !matching_mark_index.is_exact_instance_of::<PyInt>()
                || matching_mark_index.is_instance_of::<PyBool>()
            {
                return Err(operation_validation_error(
                    py,
                    "matching mark index must be an exact nonnegative integer or None".to_owned(),
                ));
            }
            Some(matching_mark_index.extract::<u32>().map_err(|_| {
                operation_validation_error(py, "matching mark index is outside u32".to_owned())
            })?)
        };
        let action = AtomMarkActionV1::from(*action);
        if action == AtomMarkActionV1::Add && matching_mark_index.is_some() {
            return Err(operation_validation_error(
                py,
                "an add operation cannot select an existing mark".to_owned(),
            ));
        }
        Ok(Self {
            operation: SessionOperation::V1(SessionOperationV1::ApplyAtomMark {
                molecule_id,
                atom_id,
                action,
                kind: (*kind).into(),
                matching_mark_index,
            }),
        })
    }

    /// Build one complete unique-field bond-properties patch.
    #[staticmethod]
    fn set_bond_properties(
        py: Python<'_>,
        bond_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        if !changes.is_exact_instance_of::<PyTuple>() {
            return Err(operation_validation_error(
                py,
                "bond-properties changes must be an exact built-in tuple".to_owned(),
            ));
        }
        if changes.len() > 7 {
            return Err(operation_validation_error(
                py,
                "a bond-properties patch accepts at most seven unique changes".to_owned(),
            ));
        }
        let changes = changes
            .iter()
            .map(|value| {
                value
                    .extract::<PyRef<'_, PyDocumentBondPropertyChangeV1>>()
                    .map(|value| value.change.clone())
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let patch = BondPropertiesPatchV1::new(bond_id, changes)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self {
            operation: SessionOperation::V1(SessionOperationV1::SetBondProperties { patch }),
        })
    }

    /// Build one complete unique-field direct-root Plus properties patch.
    #[staticmethod]
    fn set_plus_properties(
        py: Python<'_>,
        plus_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation = crate::plus_properties_binding::set_plus_properties(py, plus_id, changes)?;
        Ok(Self { operation })
    }

    /// Build one complete unique-field direct-root Text properties patch.
    #[staticmethod]
    fn set_text_properties(
        py: Python<'_>,
        text_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation = crate::text_properties_binding::set_text_properties(py, text_id, changes)?;
        Ok(Self { operation })
    }

    /// Build one complete unique-field direct-root Arrow properties patch.
    #[staticmethod]
    fn set_arrow_properties(
        py: Python<'_>,
        arrow_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation =
            crate::arrow_properties_binding::set_arrow_properties(py, arrow_id, changes)?;
        Ok(Self { operation })
    }

    /// Build one complete unique-field geometric presentation appearance patch.
    #[staticmethod]
    fn set_geometric_properties(
        py: Python<'_>,
        presentation_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation = crate::geometric_properties_binding::set_geometric_properties(
            py,
            presentation_id,
            changes,
        )?;
        Ok(Self { operation })
    }

    /// Build one complete unique-field Wavy presentation appearance patch.
    #[staticmethod]
    fn set_wavy_properties(
        py: Python<'_>,
        wavy_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation = crate::wavy_properties_binding::set_wavy_properties(py, wavy_id, changes)?;
        Ok(Self { operation })
    }

    /// Build one complete unique-field common bracket-pair appearance patch.
    #[staticmethod]
    fn set_bracket_properties(
        py: Python<'_>,
        pair_id: String,
        changes: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation = crate::bracket_binding::set_bracket_properties(py, pair_id, changes)?;
        Ok(Self { operation })
    }

    /// Build the V1 operation that replaces one existing atom's finite point.
    #[staticmethod]
    fn set_atom_position(
        py: Python<'_>,
        atom_id: String,
        x: f64,
        y: f64,
        z: f64,
    ) -> PyResult<Self> {
        let position = match Point3V1::new(x, y, z) {
            Ok(position) => position,
            Err(error) => return Err(projection_error(py, error)?),
        };
        Ok(Self {
            operation: SessionOperation::V1(SessionOperationV1::SetAtomPosition {
                atom_id,
                position,
            }),
        })
    }

    /// Build the V1 operation that deletes one atom and every incident bond.
    #[staticmethod]
    fn delete_atom(atom_id: String) -> Self {
        Self {
            operation: SessionOperation::V1(SessionOperationV1::DeleteAtom { atom_id }),
        }
    }

    /// Build the V1 operation that deletes one durable typed bond.
    #[staticmethod]
    fn delete_bond(bond_id: String) -> Self {
        Self {
            operation: SessionOperation::V1(SessionOperationV1::DeleteBond { bond_id }),
        }
    }

    /// Build the V1 operation that deletes one exact durable presentation root.
    #[staticmethod]
    fn delete_presentation_root(
        py: Python<'_>,
        presentation_id: String,
        kind: PyRef<'_, crate::presentation_deletion_binding::PyDocumentPresentationRootKindV1>,
    ) -> PyResult<Self> {
        let operation = crate::presentation_deletion_binding::delete_presentation_root(
            py,
            presentation_id,
            kind,
        )?;
        Ok(Self { operation })
    }

    /// Build one atomic deletion of a complete durable presentation selection.
    #[staticmethod]
    fn delete_presentation_roots(py: Python<'_>, targets: &Bound<'_, PyTuple>) -> PyResult<Self> {
        let operation =
            crate::presentation_deletion_binding::delete_presentation_roots(py, targets)?;
        Ok(Self { operation })
    }

    /// Build one closed direct-root presentation stack reorder.
    #[staticmethod]
    fn reorder_presentation_roots(
        py: Python<'_>,
        order: PyRef<'_, crate::presentation_stack_binding::PyDocumentPresentationStackOrderV1>,
        targets: &Bound<'_, PyTuple>,
    ) -> PyResult<Self> {
        let operation =
            crate::presentation_stack_binding::reorder_presentation_roots(py, order, targets)?;
        Ok(Self { operation })
    }

    /// Build one translation of complete durable direct-root objects.
    #[staticmethod]
    fn translate_top_level_roots(
        py: Python<'_>,
        targets: &Bound<'_, PyTuple>,
        dx: &Bound<'_, PyAny>,
        dy: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let operation = crate::top_level_transform_binding::translate(py, targets, dx, dy)?;
        Ok(Self { operation })
    }

    /// Build one alignment of complete durable direct-root objects.
    #[staticmethod]
    fn align_top_level_roots(
        py: Python<'_>,
        targets: &Bound<'_, PyTuple>,
        alignment: PyRef<'_, crate::top_level_transform_binding::PyDocumentTopLevelAlignmentV1>,
    ) -> PyResult<Self> {
        let operation = crate::top_level_transform_binding::align(py, targets, alignment)?;
        Ok(Self { operation })
    }

    /// Build one positive finite scale around the aggregate selection center.
    #[staticmethod]
    fn scale_top_level_roots(
        py: Python<'_>,
        targets: &Bound<'_, PyTuple>,
        scale_x: &Bound<'_, PyAny>,
        scale_y: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let operation = crate::top_level_transform_binding::scale(py, targets, scale_x, scale_y)?;
        Ok(Self { operation })
    }

    /// Build one mirror around the aggregate selection center.
    #[staticmethod]
    fn mirror_top_level_roots(
        py: Python<'_>,
        targets: &Bound<'_, PyTuple>,
        orientation: PyRef<'_, crate::top_level_transform_binding::PyDocumentTopLevelMirrorV1>,
    ) -> PyResult<Self> {
        let operation = crate::top_level_transform_binding::mirror(py, targets, orientation)?;
        Ok(Self { operation })
    }

    /// Build one selected-atom rotation around a finite scene-space center.
    #[staticmethod]
    fn rotate_atoms(
        py: Python<'_>,
        targets: &Bound<'_, PyTuple>,
        center_x: &Bound<'_, PyAny>,
        center_y: &Bound<'_, PyAny>,
        angle_radians: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let operation = crate::atom_rotation_binding::rotate_atoms(
            py,
            targets,
            center_x,
            center_y,
            angle_radians,
        )?;
        Ok(Self { operation })
    }

    /// Build one supported geometry repair over complete durable molecules.
    #[staticmethod]
    fn repair_geometry(
        py: Python<'_>,
        molecule_ids: &Bound<'_, PyTuple>,
        kind: PyRef<'_, crate::geometry_repair_binding::PyDocumentGeometryRepairKindV1>,
        target_spacing_points: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let operation = crate::geometry_repair_binding::repair_geometry(
            py,
            molecule_ids,
            kind,
            target_spacing_points,
        )?;
        Ok(Self { operation })
    }

    /// Build the V1 operation that replaces one durable typed bond's order.
    #[staticmethod]
    fn set_bond_order(bond_id: String, order: PyRef<'_, PyDocumentBondOrderV1>) -> Self {
        Self {
            operation: SessionOperation::V1(SessionOperationV1::SetBondOrder {
                bond_id,
                order: (*order).into(),
            }),
        }
    }
}
