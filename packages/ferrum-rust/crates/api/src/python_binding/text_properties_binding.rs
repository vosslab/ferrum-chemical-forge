//! Closed Python Text edit values and bounded operation construction.

use ferrum_document::{
    PresentationFontFaceV1, Rgb24V1, SessionOperation, SessionOperationV1, TextEditRunV1,
    TextEditStyleV1, TextPropertiesPatchV1, TextPropertyChangeV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyInt, PyTuple};

use super::binding::operation_validation_error;

/// Closed style vocabulary accepted by one Text edit run.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentTextEditStyleV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentTextEditStyleV1 {
    Bold,
    Italic,
    Subscript,
    Superscript,
}

impl From<PyDocumentTextEditStyleV1> for TextEditStyleV1 {
    fn from(value: PyDocumentTextEditStyleV1) -> Self {
        match value {
            PyDocumentTextEditStyleV1::Bold => Self::Bold,
            PyDocumentTextEditStyleV1::Italic => Self::Italic,
            PyDocumentTextEditStyleV1::Subscript => Self::Subscript,
            PyDocumentTextEditStyleV1::Superscript => Self::Superscript,
        }
    }
}

/// One exact validated character-data run accepted by a Rust Text patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentTextEditRunV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentTextEditRunV1 {
    pub(crate) run: TextEditRunV1,
}

#[pymethods]
impl PyDocumentTextEditRunV1 {
    #[getter]
    fn text(&self) -> String {
        self.run.text().to_owned()
    }

    #[getter]
    fn styles(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let values = self.run.styles().iter().map(|style| match style {
            TextEditStyleV1::Bold => PyDocumentTextEditStyleV1::Bold,
            TextEditStyleV1::Italic => PyDocumentTextEditStyleV1::Italic,
            TextEditStyleV1::Subscript => PyDocumentTextEditStyleV1::Subscript,
            TextEditStyleV1::Superscript => PyDocumentTextEditStyleV1::Superscript,
        });
        Ok(PyTuple::new(py, values)?.unbind())
    }

    /// Build one nonempty run from an exact tuple of closed styles.
    #[staticmethod]
    fn create(py: Python<'_>, text: String, styles: &Bound<'_, PyTuple>) -> PyResult<Self> {
        if !styles.is_exact_instance_of::<PyTuple>() {
            return Err(operation_validation_error(
                py,
                "Text edit run styles must be an exact built-in tuple".to_owned(),
            ));
        }
        if styles.len() > 4 {
            return Err(operation_validation_error(
                py,
                "a Text edit run accepts at most four unique closed styles".to_owned(),
            ));
        }
        let styles = styles
            .iter()
            .map(|value| {
                value
                    .extract::<PyRef<'_, PyDocumentTextEditStyleV1>>()
                    .map(|value| TextEditStyleV1::from(*value))
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;
        let run = TextEditRunV1::new(text, styles)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self { run })
    }
}

/// One exact direct-root Text property change accepted by a Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentTextPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentTextPropertyChangeV1 {
    change: TextPropertyChangeV1,
}

#[pymethods]
impl PyDocumentTextPropertyChangeV1 {
    /// Replace the complete formatted character run sequence.
    #[staticmethod]
    fn runs(py: Python<'_>, values: &Bound<'_, PyTuple>) -> PyResult<Self> {
        if !values.is_exact_instance_of::<PyTuple>() {
            return Err(operation_validation_error(
                py,
                "Text edit runs must be an exact built-in tuple".to_owned(),
            ));
        }
        let runs = values
            .iter()
            .map(|value| {
                value
                    .extract::<PyRef<'_, PyDocumentTextEditRunV1>>()
                    .map(|value| value.run.clone())
                    .map_err(Into::into)
            })
            .collect::<PyResult<Vec<_>>>()?;
        text_property_change(py, TextPropertyChangeV1::Runs(runs))
    }

    /// Select the stable backend-owned face identity.
    #[staticmethod]
    fn font_face_id(py: Python<'_>, value: String) -> PyResult<Self> {
        let face = PresentationFontFaceV1::from_id(&value)
            .ok_or_else(|| operation_validation_error(py, "unsupported_text_face".to_owned()))?;
        text_property_change(py, TextPropertyChangeV1::FontFace(face))
    }

    /// Replace the direct font's integer size from 4 through 144.
    #[staticmethod]
    fn font_size(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !value.is_exact_instance_of::<PyInt>() || value.is_instance_of::<PyBool>() {
            return Err(operation_validation_error(
                py,
                "Text font size must be an exact integer from 4 through 144".to_owned(),
            ));
        }
        let value = value.extract::<u16>().map_err(|_| {
            operation_validation_error(
                py,
                "Text font size must be an exact integer from 4 through 144".to_owned(),
            )
        })?;
        text_property_change(py, TextPropertyChangeV1::FontSize(value))
    }

    /// Replace the direct font's foreground colour.
    #[staticmethod]
    fn color(py: Python<'_>, value: String) -> PyResult<Self> {
        let value = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(py, "Text color must be #rgb or #rrggbb".to_owned())
        })?;
        text_property_change(py, TextPropertyChangeV1::Color(value))
    }

    /// Replace the root background colour, or author explicit transparency.
    #[staticmethod]
    fn background_color(py: Python<'_>, value: Option<String>) -> PyResult<Self> {
        let value = match value {
            Some(value) => Rgb24V1::new(value).map(Some).ok_or_else(|| {
                operation_validation_error(
                    py,
                    "Text background color must be #rgb, #rrggbb, or None".to_owned(),
                )
            })?,
            None => None,
        };
        text_property_change(py, TextPropertyChangeV1::BackgroundColor(value))
    }
}

pub(crate) fn set_text_properties(
    py: Python<'_>,
    text_id: String,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<SessionOperation> {
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "Text-properties changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 5 {
        return Err(operation_validation_error(
            py,
            "a Text-properties patch accepts at most five unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentTextPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    let patch = TextPropertiesPatchV1::new(text_id, changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::SetTextProperties { patch },
    ))
}

fn text_property_change(
    py: Python<'_>,
    change: TextPropertyChangeV1,
) -> PyResult<PyDocumentTextPropertyChangeV1> {
    TextPropertiesPatchV1::new("validation-text", vec![change.clone()])
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(PyDocumentTextPropertyChangeV1 { change })
}
