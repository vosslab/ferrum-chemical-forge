//! Frozen Python boundary for finite Ferrum canvas geometry.
//!
//! The Qt frontend receives owned scalar tuples or a frozen placement DTO.  This
//! module never accepts a Qt value, Python sequence, callback, or mutable mapping.

use ferrum_geometry::{
    CDML_POINTS_PER_CENTIMETRE_V1, CdmlLength, GeometryError as RustGeometryError, HexGrid,
    MoleculePlacementV1, Point2, ScenePoints,
};
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt, PyTuple};

use crate::binding::FerrumError;

create_exception!(ferrum_chem, GeometryError, FerrumError);

/// One immutable validated molecule-insertion placement request.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "InsertionPlacementV1",
    skip_from_py_object
)]
pub(crate) struct PyInsertionPlacementV1 {
    placement: MoleculePlacementV1,
}

impl PyInsertionPlacementV1 {
    pub(crate) const fn placement(&self) -> MoleculePlacementV1 {
        self.placement
    }
}

#[pymethods]
impl PyInsertionPlacementV1 {
    /// Return the requested destination mean bond length.
    #[getter]
    fn bond_length_pt(&self) -> f64 {
        self.placement.bond_length()
    }

    /// Return the requested destination centroid x coordinate.
    #[getter]
    fn anchor_x(&self) -> f64 {
        self.placement.anchor().x()
    }

    /// Return the requested destination centroid y coordinate.
    #[getter]
    fn anchor_y(&self) -> f64 {
        self.placement.anchor().y()
    }
}

/// Return the exact CDML points-per-centimetre scale used by this V1 boundary.
#[pyfunction]
fn cdml_points_per_cm_v1() -> f64 {
    CDML_POINTS_PER_CENTIMETRE_V1
}

/// Convert one finite CDML centimetre coordinate into finite scene points.
#[pyfunction]
fn cm_to_points_v1(py: Python<'_>, centimetres: &Bound<'_, PyAny>) -> PyResult<f64> {
    let value = finite_number(py, centimetres, "centimetres")?;
    let centimetres = geometry_result(py, CdmlLength::try_from_centimetres(value))?;
    geometry_result(py, centimetres.as_scene_points()).map(ScenePoints::as_scene_points)
}

/// Convert one finite scene-point coordinate into finite CDML centimetres.
#[pyfunction]
fn points_to_cm_v1(py: Python<'_>, points: &Bound<'_, PyAny>) -> PyResult<f64> {
    let value = finite_number(py, points, "points")?;
    let points = geometry_result(py, ScenePoints::try_from_scene_points(value))?;
    geometry_result(py, points.as_centimetres()).map(CdmlLength::as_centimetres)
}

/// Return bounded immutable pointy-top hex-grid vertices inside one rectangle.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn hex_grid_points_v1(
    py: Python<'_>,
    x_min: &Bound<'_, PyAny>,
    y_min: &Bound<'_, PyAny>,
    x_max: &Bound<'_, PyAny>,
    y_max: &Bound<'_, PyAny>,
    spacing: &Bound<'_, PyAny>,
) -> PyResult<Py<PyTuple>> {
    let (minimum, maximum, grid) = grid_request(py, x_min, y_min, x_max, y_max, spacing)?;
    let points = geometry_result(py, grid.points_in_rect(minimum, maximum))?;
    point_tuple(py, points.unwrap_or_default())
}

/// Return bounded immutable pointy-top honeycomb edges inside one rectangle.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
fn hex_grid_edges_v1(
    py: Python<'_>,
    x_min: &Bound<'_, PyAny>,
    y_min: &Bound<'_, PyAny>,
    x_max: &Bound<'_, PyAny>,
    y_max: &Bound<'_, PyAny>,
    spacing: &Bound<'_, PyAny>,
) -> PyResult<Py<PyTuple>> {
    let (minimum, maximum, grid) = grid_request(py, x_min, y_min, x_max, y_max, spacing)?;
    let edges = geometry_result(py, grid.honeycomb_edges_in_rect(minimum, maximum))?;
    let values = edges.unwrap_or_default();
    let python_edges = values.into_iter().map(|edge| {
        let start = (edge.start.x(), edge.start.y());
        let end = (edge.end.x(), edge.end.y());
        (start, end)
    });
    Ok(PyTuple::new(py, python_edges)?.unbind())
}

/// Snap one finite scene point to its deterministic nearest hex-grid vertex.
#[pyfunction]
fn snap_to_hex_grid_v1(
    py: Python<'_>,
    x: &Bound<'_, PyAny>,
    y: &Bound<'_, PyAny>,
    spacing: &Bound<'_, PyAny>,
) -> PyResult<(f64, f64)> {
    let point = point_from(py, x, y, "point")?;
    let grid = hex_grid(py, spacing)?;
    let snapped = geometry_result(py, grid.snap(point))?;
    Ok((snapped.x(), snapped.y()))
}

/// Validate and return one finite positive display-grid spacing.
#[pyfunction]
fn normalize_hex_grid_spacing_v1(py: Python<'_>, spacing: &Bound<'_, PyAny>) -> PyResult<f64> {
    positive_spacing(py, spacing)
}

/// Validate molecule insertion scale and anchor without accepting frontend state.
#[pyfunction]
fn validate_insertion_placement_v1(
    py: Python<'_>,
    bond_length_pt: &Bound<'_, PyAny>,
    anchor_x: &Bound<'_, PyAny>,
    anchor_y: &Bound<'_, PyAny>,
) -> PyResult<PyInsertionPlacementV1> {
    let bond_length_pt = positive_number(py, bond_length_pt, "insertion bond length")?;
    let anchor = point_from(py, anchor_x, anchor_y, "insertion anchor")?;
    let placement = geometry_result(py, MoleculePlacementV1::new(bond_length_pt, anchor))?;
    Ok(PyInsertionPlacementV1 { placement })
}

fn grid_request(
    py: Python<'_>,
    x_min: &Bound<'_, PyAny>,
    y_min: &Bound<'_, PyAny>,
    x_max: &Bound<'_, PyAny>,
    y_max: &Bound<'_, PyAny>,
    spacing: &Bound<'_, PyAny>,
) -> PyResult<(Point2, Point2, HexGrid)> {
    let minimum = point_from(py, x_min, y_min, "rectangle minimum")?;
    let maximum = point_from(py, x_max, y_max, "rectangle maximum")?;
    let grid = hex_grid(py, spacing)?;
    Ok((minimum, maximum, grid))
}

fn hex_grid(py: Python<'_>, spacing: &Bound<'_, PyAny>) -> PyResult<HexGrid> {
    let spacing = positive_spacing(py, spacing)?;
    let origin = geometry_result(py, Point2::new(0.0, 0.0))?;
    geometry_result(py, HexGrid::new(spacing, origin))
}

fn point_from(
    py: Python<'_>,
    x: &Bound<'_, PyAny>,
    y: &Bound<'_, PyAny>,
    label: &str,
) -> PyResult<Point2> {
    let x = finite_number(py, x, &format!("{label} x"))?;
    let y = finite_number(py, y, &format!("{label} y"))?;
    geometry_result(py, Point2::new(x, y))
}

fn positive_spacing(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<f64> {
    positive_number(py, value, "hex-grid spacing")
}

fn positive_number(py: Python<'_>, value: &Bound<'_, PyAny>, label: &str) -> PyResult<f64> {
    let number = finite_number(py, value, label)?;
    if number <= 0.0 {
        return Err(geometry_error(
            py,
            format!("{label} must be greater than zero"),
        ));
    }
    Ok(number)
}

fn finite_number(py: Python<'_>, value: &Bound<'_, PyAny>, label: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>()
        || (!value.is_instance_of::<PyFloat>() && !value.is_instance_of::<PyInt>())
    {
        return Err(geometry_error(
            py,
            format!("{label} must be a finite built-in number"),
        ));
    }
    let number = value
        .extract::<f64>()
        .map_err(|_| geometry_error(py, format!("{label} must be a finite built-in number")))?;
    if !number.is_finite() {
        return Err(geometry_error(py, format!("{label} must be finite")));
    }
    Ok(number)
}

fn point_tuple(py: Python<'_>, points: Vec<Point2>) -> PyResult<Py<PyTuple>> {
    let values = points.into_iter().map(|point| (point.x(), point.y()));
    Ok(PyTuple::new(py, values)?.unbind())
}

fn geometry_result<T>(py: Python<'_>, result: Result<T, RustGeometryError>) -> PyResult<T> {
    result.map_err(|error| geometry_error(py, error.to_string()))
}

pub(crate) fn geometry_error(_py: Python<'_>, message: String) -> PyErr {
    GeometryError::new_err(message)
}

/// Register the public finite-geometry V1 boundary.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("GeometryError", module.py().get_type::<GeometryError>())?;
    module.add_class::<PyInsertionPlacementV1>()?;
    module.add_function(wrap_pyfunction!(cdml_points_per_cm_v1, module)?)?;
    module.add_function(wrap_pyfunction!(cm_to_points_v1, module)?)?;
    module.add_function(wrap_pyfunction!(points_to_cm_v1, module)?)?;
    module.add_function(wrap_pyfunction!(hex_grid_points_v1, module)?)?;
    module.add_function(wrap_pyfunction!(hex_grid_edges_v1, module)?)?;
    module.add_function(wrap_pyfunction!(snap_to_hex_grid_v1, module)?)?;
    module.add_function(wrap_pyfunction!(normalize_hex_grid_spacing_v1, module)?)?;
    module.add_function(wrap_pyfunction!(validate_insertion_placement_v1, module)?)?;
    Ok(())
}
