//! Frozen Python DTOs for Rust-issued presentation-path replay commands.

use ferrum_document::Point3V1;
use ferrum_render::{PathCommandV1, PathKindV1, lower_presentation_points_path_v1};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

use crate::binding::FerrumError;
use crate::presentation_root_binding::PyPresentationRootProjectionV1;
use crate::projection_binding::{PyPoint3V1, PyPolylineProjectionV1};

create_exception!(ferrum_chem, PresentationPathError, FerrumError);

/// One source-issued command that Qt may replay without choosing geometry.
#[pyclass(frozen, name = "PresentationPathCommandV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationPathCommandV1 {
    #[pyo3(get)]
    kind: String,
    #[pyo3(get)]
    point: Option<PyPoint3V1>,
    #[pyo3(get)]
    control_1: Option<PyPoint3V1>,
    #[pyo3(get)]
    control_2: Option<PyPoint3V1>,
}

/// One immutable Rust-lowered presentation path.
#[pyclass(frozen, name = "PresentationPathV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationPathV1 {
    #[pyo3(get)]
    kind: String,
    commands: Vec<PyPresentationPathCommandV1>,
}

#[pymethods]
impl PyPresentationPathV1 {
    #[getter]
    fn commands(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        frozen_tuple(py, &self.commands)
    }
}

/// Lower one accepted round-bracket root into Rust-owned replay commands.
///
/// This intentionally refuses ordinary polylines. Document projection remains
/// the authority that rejects normal CDML spline roots until that feature has a
/// separate document contract.
#[pyfunction]
pub(crate) fn lower_round_bracket_presentation_path_v1(
    root: PyRef<'_, PyPresentationRootProjectionV1>,
) -> PyResult<PyPresentationPathV1> {
    let polyline = validated_round_bracket(&root)?;
    let points = polyline
        .path
        .points
        .iter()
        .map(point)
        .collect::<PyResult<Vec<_>>>()?;
    let path = lower_presentation_points_path_v1(&points, PathKindV1::AuthoredSpline)
        .map_err(|error| PresentationPathError::new_err(error.to_string()))?;
    Ok(path.into())
}

impl From<ferrum_render::PresentationPathV1> for PyPresentationPathV1 {
    fn from(value: ferrum_render::PresentationPathV1) -> Self {
        Self {
            kind: match value.kind() {
                PathKindV1::Polyline => "polyline".to_owned(),
                PathKindV1::AuthoredSpline => "authored_spline".to_owned(),
            },
            commands: value.commands().iter().copied().map(Into::into).collect(),
        }
    }
}

impl From<PathCommandV1> for PyPresentationPathCommandV1 {
    fn from(value: PathCommandV1) -> Self {
        match value {
            PathCommandV1::MoveTo(point) => Self {
                kind: "move_to".to_owned(),
                point: Some(point_from_render(point)),
                control_1: None,
                control_2: None,
            },
            PathCommandV1::LineTo(point) => Self {
                kind: "line_to".to_owned(),
                point: Some(point_from_render(point)),
                control_1: None,
                control_2: None,
            },
            PathCommandV1::CubicTo {
                control_1,
                control_2,
                end,
            } => Self {
                kind: "cubic_to".to_owned(),
                point: Some(point_from_render(end)),
                control_1: Some(point_from_render(control_1)),
                control_2: Some(point_from_render(control_2)),
            },
            PathCommandV1::Close => Self {
                kind: "close".to_owned(),
                point: None,
                control_1: None,
                control_2: None,
            },
        }
    }
}

fn validated_round_bracket(
    root: &PyPresentationRootProjectionV1,
) -> PyResult<&PyPolylineProjectionV1> {
    if root.kind != "round_bracket"
        || root.arrow.is_some()
        || root.plus.is_some()
        || root.text.is_some()
        || root.shape.is_some()
        || root.polygon.is_some()
    {
        return Err(PresentationPathError::new_err(
            "presentation path lowering only accepts a round bracket root",
        ));
    }
    root.polyline
        .as_ref()
        .ok_or_else(|| PresentationPathError::new_err("round bracket root has no polyline payload"))
}

fn point(value: &PyPoint3V1) -> PyResult<Point3V1> {
    Point3V1::new(value.x, value.y, value.z)
        .map_err(|error| PresentationPathError::new_err(error.to_string()))
}

fn point_from_render(value: ferrum_render::RenderPoint) -> PyPoint3V1 {
    PyPoint3V1 {
        x: value.x(),
        y: value.y(),
        z: 0.0,
    }
}

fn frozen_tuple<T>(py: Python<'_>, values: &[T]) -> PyResult<Py<PyTuple>>
where
    T: Clone + for<'a> IntoPyObject<'a>,
{
    Ok(PyTuple::new(py, values.iter().cloned())?.unbind())
}

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "PresentationPathError",
        module.py().get_type::<PresentationPathError>(),
    )?;
    module.add_function(wrap_pyfunction!(
        lower_round_bracket_presentation_path_v1,
        module
    )?)?;
    module.add_class::<PyPresentationPathV1>()?;
    module.add_class::<PyPresentationPathCommandV1>()
}
