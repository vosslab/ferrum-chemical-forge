//! Closed Python Plus-property changes and bounded operation construction.

use ferrum_document::{
    DocumentObjectIdV1, PlusPropertiesPatchV1, PlusPropertyChangeV1, PresentationFontFaceV1,
    Rgb24V1, SessionOperation, SessionOperationV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyInt, PyTuple};

use super::binding::operation_validation_error;

/// One exact direct-root Plus property change accepted by a Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentPlusPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentPlusPropertyChangeV1 {
    change: PlusPropertyChangeV1,
}

#[pymethods]
impl PyDocumentPlusPropertyChangeV1 {
    /// Select the stable backend-owned face identity.
    #[staticmethod]
    fn font_face_id(py: Python<'_>, value: String) -> PyResult<Self> {
        let face = PresentationFontFaceV1::from_id(&value)
            .ok_or_else(|| operation_validation_error(py, "unsupported_text_face".to_owned()))?;
        plus_property_change(py, PlusPropertyChangeV1::FontFace(face))
    }

    /// Replace the documented integer root font size from 4 through 144.
    #[staticmethod]
    fn font_size(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !value.is_exact_instance_of::<PyInt>() || value.is_instance_of::<PyBool>() {
            return Err(operation_validation_error(
                py,
                "Plus font size must be an exact integer from 4 through 144".to_owned(),
            ));
        }
        let value = value.extract::<u16>().map_err(|_| {
            operation_validation_error(
                py,
                "Plus font size must be an exact integer from 4 through 144".to_owned(),
            )
        })?;
        plus_property_change(py, PlusPropertyChangeV1::FontSize(value))
    }

    /// Replace the root-authoritative foreground colour.
    #[staticmethod]
    fn color(py: Python<'_>, value: String) -> PyResult<Self> {
        let value = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(py, "Plus color must be #rgb or #rrggbb".to_owned())
        })?;
        plus_property_change(py, PlusPropertyChangeV1::Color(value))
    }

    /// Replace the root background colour, or author explicit transparency.
    #[staticmethod]
    fn background_color(py: Python<'_>, value: Option<String>) -> PyResult<Self> {
        let value = match value {
            Some(value) => Rgb24V1::new(value).map(Some).ok_or_else(|| {
                operation_validation_error(
                    py,
                    "Plus background color must be #rgb, #rrggbb, or None".to_owned(),
                )
            })?,
            None => None,
        };
        plus_property_change(py, PlusPropertyChangeV1::BackgroundColor(value))
    }
}

pub(crate) fn set_plus_properties(
    py: Python<'_>,
    plus_object_id: DocumentObjectIdV1,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "Plus-properties changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 4 {
        return Err(operation_validation_error(
            py,
            "a Plus-properties patch accepts at most four unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentPlusPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let patch = PlusPropertiesPatchV1::new(plus_object_id, changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::SetPlusProperties { patch },
    ))
}

fn plus_property_change(
    py: Python<'_>,
    change: PlusPropertyChangeV1,
) -> PyResult<PyDocumentPlusPropertyChangeV1> {
    PlusPropertiesPatchV1::new(validation_object_id(), vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentPlusPropertyChangeV1 { change })
}

fn validation_object_id() -> DocumentObjectIdV1 {
    DocumentObjectIdV1::parse("ferrum-document-object-v1/00000000000000000000000000000000")
        .expect("fixed validation document-object identity")
}
