//! Exact Python factories for durable-root transforms.

use ferrum_document::{
    SessionOperation, SessionOperationV1, TopLevelRootKindV1, TopLevelRootLayoutTransformModeV1,
    TopLevelRootLayoutTransformV1, TopLevelRootSelectorV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyTuple};

use super::binding::operation_validation_error;

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentTopLevelRootKindV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentTopLevelRootKindV1 {
    Molecule,
    Arrow,
    Plus,
    Text,
    Rectangle,
    Square,
    Oval,
    Circle,
    Polygon,
    Polyline,
}

impl From<PyDocumentTopLevelRootKindV1> for TopLevelRootKindV1 {
    fn from(value: PyDocumentTopLevelRootKindV1) -> Self {
        match value {
            PyDocumentTopLevelRootKindV1::Molecule => Self::Molecule,
            PyDocumentTopLevelRootKindV1::Arrow => Self::Arrow,
            PyDocumentTopLevelRootKindV1::Plus => Self::Plus,
            PyDocumentTopLevelRootKindV1::Text => Self::Text,
            PyDocumentTopLevelRootKindV1::Rectangle => Self::Rectangle,
            PyDocumentTopLevelRootKindV1::Square => Self::Square,
            PyDocumentTopLevelRootKindV1::Oval => Self::Oval,
            PyDocumentTopLevelRootKindV1::Circle => Self::Circle,
            PyDocumentTopLevelRootKindV1::Polygon => Self::Polygon,
            PyDocumentTopLevelRootKindV1::Polyline => Self::Polyline,
        }
    }
}

#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentTopLevelRootSelectorV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentTopLevelRootSelectorV1 {
    selector: TopLevelRootSelectorV1,
    #[pyo3(get)]
    root_id: String,
    #[pyo3(get)]
    kind: PyDocumentTopLevelRootKindV1,
}

#[pymethods]
impl PyDocumentTopLevelRootSelectorV1 {
    #[staticmethod]
    fn create(
        py: Python<'_>,
        root_id: String,
        kind: PyRef<'_, PyDocumentTopLevelRootKindV1>,
    ) -> PyResult<Self> {
        let selector = TopLevelRootSelectorV1::new(root_id.clone(), (*kind).into())
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self {
            selector,
            root_id,
            kind: *kind,
        })
    }
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentTopLevelAlignmentV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentTopLevelAlignmentV1 {
    Top,
    Bottom,
    Left,
    Right,
    CenterX,
    CenterY,
}

#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentTopLevelMirrorV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentTopLevelMirrorV1 {
    Vertical,
    Horizontal,
}

pub(crate) fn align(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    alignment: PyRef<'_, PyDocumentTopLevelAlignmentV1>,
) -> PyResult<SessionOperation> {
    let transform = match *alignment {
        PyDocumentTopLevelAlignmentV1::Top => TopLevelRootLayoutTransformModeV1::AlignTop,
        PyDocumentTopLevelAlignmentV1::Bottom => TopLevelRootLayoutTransformModeV1::AlignBottom,
        PyDocumentTopLevelAlignmentV1::Left => TopLevelRootLayoutTransformModeV1::AlignLeft,
        PyDocumentTopLevelAlignmentV1::Right => TopLevelRootLayoutTransformModeV1::AlignRight,
        PyDocumentTopLevelAlignmentV1::CenterX => TopLevelRootLayoutTransformModeV1::AlignCenterX,
        PyDocumentTopLevelAlignmentV1::CenterY => TopLevelRootLayoutTransformModeV1::AlignCenterY,
    };
    operation(py, targets, transform)
}

pub(crate) fn scale(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    scale_x: &Bound<'_, PyAny>,
    scale_y: &Bound<'_, PyAny>,
) -> PyResult<SessionOperation> {
    operation(
        py,
        targets,
        TopLevelRootLayoutTransformModeV1::Scale {
            scale_x: exact_scale(py, scale_x, "x")?,
            scale_y: exact_scale(py, scale_y, "y")?,
        },
    )
}

pub(crate) fn mirror(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    orientation: PyRef<'_, PyDocumentTopLevelMirrorV1>,
) -> PyResult<SessionOperation> {
    let transform = match *orientation {
        PyDocumentTopLevelMirrorV1::Vertical => TopLevelRootLayoutTransformModeV1::MirrorVertical,
        PyDocumentTopLevelMirrorV1::Horizontal => {
            TopLevelRootLayoutTransformModeV1::MirrorHorizontal
        }
    };
    operation(py, targets, transform)
}

pub(crate) fn selectors(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
) -> PyResult<Vec<TopLevelRootSelectorV1>> {
    if !targets.is_exact_instance_of::<PyTuple>() {
        return Err(operation_validation_error(
            py,
            "top-level translation anchor targets must be an exact built-in tuple".to_owned(),
        ));
    }
    targets
        .iter()
        .map(|value| {
            value
                .extract::<PyRef<'_, PyDocumentTopLevelRootSelectorV1>>()
                .map(|target| target.selector.clone())
                .map_err(Into::into)
        })
        .collect()
}

fn operation(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    transform: TopLevelRootLayoutTransformModeV1,
) -> PyResult<SessionOperation> {
    let targets = selectors(py, targets)?;
    let transform = TopLevelRootLayoutTransformV1::new(targets, transform)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::ApplyTopLevelRootLayoutTransformV1(transform),
    ))
}

fn exact_finite(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    operation: &str,
    axis: &str,
) -> PyResult<f64> {
    if !(value.is_exact_instance_of::<PyFloat>() || value.is_exact_instance_of::<PyInt>())
        || value.is_instance_of::<PyBool>()
    {
        return Err(operation_validation_error(
            py,
            format!("top-level {operation} {axis} must be an exact int or float"),
        ));
    }
    let value = value.extract::<f64>().map_err(|_| {
        operation_validation_error(
            py,
            format!("top-level {operation} {axis} is outside finite f64"),
        )
    })?;
    value.is_finite().then_some(value).ok_or_else(|| {
        operation_validation_error(py, format!("top-level {operation} {axis} must be finite"))
    })
}

fn exact_scale(py: Python<'_>, value: &Bound<'_, PyAny>, axis: &str) -> PyResult<f64> {
    let value = exact_finite(py, value, "scale", axis)?;
    (value > 0.0).then_some(value).ok_or_else(|| {
        operation_validation_error(
            py,
            format!("top-level scale {axis} must be greater than zero"),
        )
    })
}
