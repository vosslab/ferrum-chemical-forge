//! Exact Python factories for durable-root transforms.

use ferrum_document::{
    SessionOperation, SessionOperationV1, TopLevelRootKindV1, TopLevelRootSelectorV1,
    TopLevelTransformModeV1, TopLevelTransformV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyTuple};

use crate::binding::operation_validation_error;
use crate::binding::{PyDocumentSession, document_result};

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

/// Immutable private-adapter receipt for one authored complete-root move.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "TopLevelTranslationAnchorV1",
    skip_from_py_object
)]
pub(crate) struct PyTopLevelTranslationAnchorV1 {
    selectors: Vec<PyDocumentTopLevelRootSelectorV1>,
    #[pyo3(get)]
    source_revision: u64,
    #[pyo3(get)]
    source_digest: String,
    #[pyo3(get)]
    anchor_x: f64,
    #[pyo3(get)]
    anchor_y: f64,
}

#[pymethods]
impl PyTopLevelTranslationAnchorV1 {
    #[getter]
    fn selectors(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        let values = self
            .selectors
            .iter()
            .cloned()
            .map(|selector| Py::new(py, selector))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyTuple::new(py, values)?.unbind())
    }
}

#[pymethods]
impl PyDocumentSession {
    /// Observe a private authored-coordinate anchor for one complete-root move.
    fn observe_top_level_translation_anchor_v1(
        &self,
        py: Python<'_>,
        expected_revision: u64,
        targets: &Bound<'_, PyTuple>,
    ) -> PyResult<PyTopLevelTranslationAnchorV1> {
        let targets = selectors(py, targets)?;
        document_result(
            py,
            ferrum_api::observe_top_level_translation_anchor_v1(
                &self.session,
                expected_revision,
                targets,
            ),
        )
        .map(PyTopLevelTranslationAnchorV1::from_anchor)
    }
}

impl PyTopLevelTranslationAnchorV1 {
    pub(crate) fn from_anchor(anchor: ferrum_document::TopLevelTranslationAnchorV1) -> Self {
        let (anchor_x, anchor_y) = anchor.anchor();
        let selectors = anchor
            .selectors()
            .iter()
            .cloned()
            .map(|selector| {
                let kind = match selector.kind() {
                    TopLevelRootKindV1::Molecule => PyDocumentTopLevelRootKindV1::Molecule,
                    TopLevelRootKindV1::Arrow => PyDocumentTopLevelRootKindV1::Arrow,
                    TopLevelRootKindV1::Plus => PyDocumentTopLevelRootKindV1::Plus,
                    TopLevelRootKindV1::Text => PyDocumentTopLevelRootKindV1::Text,
                    TopLevelRootKindV1::Rectangle => PyDocumentTopLevelRootKindV1::Rectangle,
                    TopLevelRootKindV1::Square => PyDocumentTopLevelRootKindV1::Square,
                    TopLevelRootKindV1::Oval => PyDocumentTopLevelRootKindV1::Oval,
                    TopLevelRootKindV1::Circle => PyDocumentTopLevelRootKindV1::Circle,
                    TopLevelRootKindV1::Polygon => PyDocumentTopLevelRootKindV1::Polygon,
                    TopLevelRootKindV1::Polyline => PyDocumentTopLevelRootKindV1::Polyline,
                };
                PyDocumentTopLevelRootSelectorV1 {
                    root_id: selector.root_id().as_str().to_owned(),
                    kind,
                    selector,
                }
            })
            .collect();
        Self {
            selectors,
            source_revision: anchor.source_revision(),
            source_digest: crate::binding::hex_digest(anchor.source_digest()),
            anchor_x,
            anchor_y,
        }
    }
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

pub(crate) fn translate(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    dx: &Bound<'_, PyAny>,
    dy: &Bound<'_, PyAny>,
) -> PyResult<SessionOperation> {
    operation(
        py,
        targets,
        TopLevelTransformModeV1::Translate {
            dx: exact_finite(py, dx, "translation", "x")?,
            dy: exact_finite(py, dy, "translation", "y")?,
        },
    )
}

pub(crate) fn align(
    py: Python<'_>,
    targets: &Bound<'_, PyTuple>,
    alignment: PyRef<'_, PyDocumentTopLevelAlignmentV1>,
) -> PyResult<SessionOperation> {
    let transform = match *alignment {
        PyDocumentTopLevelAlignmentV1::Top => TopLevelTransformModeV1::AlignTop,
        PyDocumentTopLevelAlignmentV1::Bottom => TopLevelTransformModeV1::AlignBottom,
        PyDocumentTopLevelAlignmentV1::Left => TopLevelTransformModeV1::AlignLeft,
        PyDocumentTopLevelAlignmentV1::Right => TopLevelTransformModeV1::AlignRight,
        PyDocumentTopLevelAlignmentV1::CenterX => TopLevelTransformModeV1::AlignCenterX,
        PyDocumentTopLevelAlignmentV1::CenterY => TopLevelTransformModeV1::AlignCenterY,
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
        TopLevelTransformModeV1::Scale {
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
        PyDocumentTopLevelMirrorV1::Vertical => TopLevelTransformModeV1::MirrorVertical,
        PyDocumentTopLevelMirrorV1::Horizontal => TopLevelTransformModeV1::MirrorHorizontal,
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
    transform: TopLevelTransformModeV1,
) -> PyResult<SessionOperation> {
    let targets = selectors(py, targets)?;
    let transform = TopLevelTransformV1::new(targets, transform)
        .map_err(|error| operation_validation_error(py, error.to_string()))?;
    Ok(SessionOperation::V1(
        SessionOperationV1::TransformTopLevelRoots { transform },
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
