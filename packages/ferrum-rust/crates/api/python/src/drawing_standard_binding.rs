//! Frozen Python drawing-standard facts and closed mutation values.

use ferrum_document::{
    DrawingStandardPatchV1, DrawingStandardPropertyChangeV1, DrawingStandardV1, Rgb24V1,
    TransparentOrRgb24V1, VisibilityV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyString, PyTuple};

use crate::binding::operation_validation_error;

/// Authored fields from the first direct document drawing standard.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DrawingStandardV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDrawingStandardV1 {
    #[pyo3(get)]
    pub(crate) line_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) font_size: Option<f64>,
    #[pyo3(get)]
    pub(crate) font_family: Option<String>,
    #[pyo3(get)]
    pub(crate) line_color: Option<String>,
    #[pyo3(get)]
    pub(crate) area_color: Option<String>,
    #[pyo3(get)]
    pub(crate) bond_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) wedge_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) double_ratio: Option<f64>,
    #[pyo3(get)]
    pub(crate) show_hydrogens: Option<bool>,
}

impl From<&DrawingStandardV1> for PyDrawingStandardV1 {
    fn from(value: &DrawingStandardV1) -> Self {
        Self {
            line_width: value.line_width().map(|value| value.value()),
            font_size: value.font_size().map(|value| value.value()),
            font_family: value.font_family().map(str::to_owned),
            line_color: value.line_color().map(|value| value.as_str().to_owned()),
            area_color: value.area_color().and_then(|value| match value {
                TransparentOrRgb24V1::Transparent => None,
                TransparentOrRgb24V1::Rgb24(color) => Some(color.as_str().to_owned()),
            }),
            bond_width: value.bond_width().map(|value| value.value()),
            wedge_width: value.wedge_width().map(|value| value.value()),
            double_ratio: value.double_ratio().map(|value| value.value()),
            show_hydrogens: value.show_hydrogens().map(|value| match value {
                VisibilityV1::Enabled => true,
                VisibilityV1::Disabled => false,
            }),
        }
    }
}

/// One frozen exact-field drawing-standard change.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentDrawingStandardPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentDrawingStandardPropertyChangeV1 {
    pub(crate) change: DrawingStandardPropertyChangeV1,
}

#[pymethods]
impl PyDocumentDrawingStandardPropertyChangeV1 {
    /// Replace the default line width in scene points.
    #[staticmethod]
    fn line_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::number_change(py, value, DrawingStandardPropertyChangeV1::LineWidth)
    }

    /// Replace the default integer font size.
    #[staticmethod]
    fn font_size(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !value.is_exact_instance_of::<PyInt>() || value.is_instance_of::<PyBool>() {
            return Err(operation_validation_error(
                py,
                "drawing-standard font size must be an exact integer".to_owned(),
            ));
        }
        let value = value.extract::<u16>().map_err(|_| {
            operation_validation_error(py, "drawing-standard font size is outside u16".to_owned())
        })?;
        Self::validated(py, DrawingStandardPropertyChangeV1::FontSize(value))
    }

    /// Replace the default line and text colour.
    #[staticmethod]
    fn line_color(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = exact_utf8(py, value, "drawing-standard line color")?;
        let color = Rgb24V1::new(value).ok_or_else(|| {
            operation_validation_error(
                py,
                "drawing-standard line color must be #rgb or #rrggbb".to_owned(),
            )
        })?;
        Self::validated(py, DrawingStandardPropertyChangeV1::LineColor(color))
    }

    /// Replace the default label-mask colour; empty text means transparent.
    #[staticmethod]
    fn area_color(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = exact_utf8(py, value, "drawing-standard area color")?;
        let color = if value.is_empty() {
            None
        } else {
            Some(Rgb24V1::new(value).ok_or_else(|| {
                operation_validation_error(
                    py,
                    "drawing-standard area color must be empty, #rgb, or #rrggbb".to_owned(),
                )
            })?)
        };
        Self::validated(py, DrawingStandardPropertyChangeV1::AreaColor(color))
    }

    /// Replace the default spacing between parallel bond lanes.
    #[staticmethod]
    fn bond_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::number_change(py, value, DrawingStandardPropertyChangeV1::BondWidth)
    }

    /// Replace the default wedge width.
    #[staticmethod]
    fn wedge_width(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        Self::number_change(py, value, DrawingStandardPropertyChangeV1::WedgeWidth)
    }

    /// Replace the heteroatom hydrogen-display default.
    #[staticmethod]
    fn show_hydrogens(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if !value.is_exact_instance_of::<PyBool>() {
            return Err(operation_validation_error(
                py,
                "drawing-standard show hydrogens must be an exact bool".to_owned(),
            ));
        }
        Self::validated(
            py,
            DrawingStandardPropertyChangeV1::ShowHydrogens(value.extract::<bool>()?),
        )
    }
}

impl PyDocumentDrawingStandardPropertyChangeV1 {
    fn number_change(
        py: Python<'_>,
        value: &Bound<'_, PyAny>,
        factory: impl FnOnce(f64) -> DrawingStandardPropertyChangeV1,
    ) -> PyResult<Self> {
        if value.is_instance_of::<PyBool>()
            || (!value.is_exact_instance_of::<PyInt>() && !value.is_exact_instance_of::<PyFloat>())
        {
            return Err(operation_validation_error(
                py,
                "drawing-standard numeric value must be a plain number".to_owned(),
            ));
        }
        let value = value.extract::<f64>().map_err(|_| {
            operation_validation_error(
                py,
                "drawing-standard numeric value is outside f64".to_owned(),
            )
        })?;
        Self::validated(py, factory(value))
    }

    fn validated(py: Python<'_>, change: DrawingStandardPropertyChangeV1) -> PyResult<Self> {
        let patch = DrawingStandardPatchV1::new(vec![change])
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        let change =
            patch.changes().first().cloned().ok_or_else(|| {
                operation_validation_error(py, "missing standard change".to_owned())
            })?;
        Ok(Self { change })
    }
}

fn exact_utf8(py: Python<'_>, value: &Bound<'_, PyAny>, label: &str) -> PyResult<String> {
    if !value.is_exact_instance_of::<PyString>() {
        return Err(operation_validation_error(
            py,
            format!("{label} must be an exact string"),
        ));
    }
    let text = value
        .cast::<PyString>()?
        .to_str()
        .map_err(|_| operation_validation_error(py, format!("{label} must be valid UTF-8 text")))?;
    let mut copied = String::new();
    copied
        .try_reserve_exact(text.len())
        .map_err(|_| operation_validation_error(py, format!("could not allocate {label}")))?;
    copied.push_str(text);
    Ok(copied)
}

/// Register the private drawing-standard observation and mutation boundary.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyDrawingStandardV1>()?;
    module.add_class::<PyDocumentDrawingStandardPropertyChangeV1>()?;
    Ok(())
}

pub(crate) fn validate_patch(
    py: Python<'_>,
    changes: &Bound<'_, PyTuple>,
) -> PyResult<DrawingStandardPatchV1> {
    if !changes.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "drawing-standard changes must be an exact built-in tuple".to_owned(),
        ));
    }
    if changes.len() > 9 {
        return Err(operation_validation_error(
            py,
            "a drawing-standard patch accepts at most nine unique changes".to_owned(),
        ));
    }
    let changes = changes
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentDrawingStandardPropertyChangeV1>>()
                .map(|value| value.change.clone())
                .map_err(Into::into)
        })
        .collect::<PyResult<Vec<_>>>()?;
    DrawingStandardPatchV1::new(changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))
}
