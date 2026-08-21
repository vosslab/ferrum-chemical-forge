//! Frozen Python boundary for authoritative paper observation and mutation.

use ferrum_document::{
    paper_size_v1, PaperAttributesV1, PaperDimensionsMmV1, PaperLayoutProjectionV1,
    PaperOrientationV1, PaperPageIssueV1, PaperPageV1, PaperPropertiesPatchV1,
    PaperPropertyChangeV1, ViewportAttributesV1,
};
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyBool, PyFloat, PyInt};

use super::binding::operation_validation_error;

/// Closed paper orientation accepted by the V1 document editor.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PaperOrientationV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyPaperOrientationV1 {
    Portrait,
    Landscape,
}

impl From<PyPaperOrientationV1> for PaperOrientationV1 {
    fn from(value: PyPaperOrientationV1) -> Self {
        match value {
            PyPaperOrientationV1::Portrait => Self::Portrait,
            PyPaperOrientationV1::Landscape => Self::Landscape,
        }
    }
}

impl From<PaperOrientationV1> for PyPaperOrientationV1 {
    fn from(value: PaperOrientationV1) -> Self {
        match value {
            PaperOrientationV1::Portrait => Self::Portrait,
            PaperOrientationV1::Landscape => Self::Landscape,
        }
    }
}

/// Closed compatibility issue attached to resolved paper geometry.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "PaperPageIssueV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyPaperPageIssueV1 {
    UnsupportedType,
    UnsupportedOrientation,
    InvalidCustomDimensions,
}

impl From<PaperPageIssueV1> for PyPaperPageIssueV1 {
    fn from(value: PaperPageIssueV1) -> Self {
        match value {
            PaperPageIssueV1::UnsupportedType => Self::UnsupportedType,
            PaperPageIssueV1::UnsupportedOrientation => Self::UnsupportedOrientation,
            PaperPageIssueV1::InvalidCustomDimensions => Self::InvalidCustomDimensions,
        }
    }
}

/// Backend-issued physical page and exact scene rectangle.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PaperPageV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyPaperPageV1 {
    #[pyo3(get)]
    pub(crate) width_mm: f64,
    #[pyo3(get)]
    pub(crate) height_mm: f64,
    #[pyo3(get)]
    pub(crate) scene_left: f64,
    #[pyo3(get)]
    pub(crate) scene_top: f64,
    #[pyo3(get)]
    pub(crate) scene_right: f64,
    #[pyo3(get)]
    pub(crate) scene_bottom: f64,
    #[pyo3(get)]
    pub(crate) issue: Option<PyPaperPageIssueV1>,
}

impl From<PaperPageV1> for PyPaperPageV1 {
    fn from(value: PaperPageV1) -> Self {
        Self {
            width_mm: value.width_mm(),
            height_mm: value.height_mm(),
            scene_left: value.scene_left(),
            scene_top: value.scene_top(),
            scene_right: value.scene_right(),
            scene_bottom: value.scene_bottom(),
            issue: value.issue().map(Into::into),
        }
    }
}

/// Exact recognized fields from one direct core paper record.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PaperAttributesV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyPaperAttributesV1 {
    #[pyo3(get)]
    pub(crate) id: Option<String>,
    #[pyo3(get)]
    pub(crate) type_name: Option<String>,
    #[pyo3(get)]
    pub(crate) orientation: Option<String>,
    #[pyo3(get)]
    pub(crate) crop_svg: Option<String>,
    #[pyo3(get)]
    pub(crate) crop_margin: Option<String>,
    #[pyo3(get)]
    pub(crate) use_real_minus: Option<String>,
    #[pyo3(get)]
    pub(crate) replace_minus: Option<String>,
    #[pyo3(get)]
    pub(crate) size_x: Option<String>,
    #[pyo3(get)]
    pub(crate) size_y: Option<String>,
}

impl From<&PaperAttributesV1> for PyPaperAttributesV1 {
    fn from(value: &PaperAttributesV1) -> Self {
        Self {
            id: value.id().map(str::to_owned),
            type_name: value.type_name().map(str::to_owned),
            orientation: value.orientation().map(str::to_owned),
            crop_svg: value.crop_svg().map(str::to_owned),
            crop_margin: value.crop_margin().map(str::to_owned),
            use_real_minus: value.use_real_minus().map(str::to_owned),
            replace_minus: value.replace_minus().map(str::to_owned),
            size_x: value.size_x().map(str::to_owned),
            size_y: value.size_y().map(str::to_owned),
        }
    }
}

/// Exact recognized fields from one direct core viewport record.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "ViewportAttributesV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyViewportAttributesV1 {
    #[pyo3(get)]
    pub(crate) id: Option<String>,
    #[pyo3(get)]
    pub(crate) viewport: Option<String>,
}

impl From<&ViewportAttributesV1> for PyViewportAttributesV1 {
    fn from(value: &ViewportAttributesV1) -> Self {
        Self {
            id: value.id().map(str::to_owned),
            viewport: value.viewport().map(str::to_owned),
        }
    }
}

/// Frozen paper-layout facts copied from one document observation.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "PaperLayoutProjectionV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyPaperLayoutProjectionV1 {
    #[pyo3(get)]
    pub(crate) schema: String,
    #[pyo3(get)]
    pub(crate) revision: u64,
    #[pyo3(get)]
    pub(crate) digest: String,
    #[pyo3(get)]
    pub(crate) paper_present: bool,
    #[pyo3(get)]
    pub(crate) paper_attributes: PyPaperAttributesV1,
    #[pyo3(get)]
    pub(crate) effective_paper_attributes: PyPaperAttributesV1,
    #[pyo3(get)]
    pub(crate) viewport_attributes: PyViewportAttributesV1,
    #[pyo3(get)]
    pub(crate) default_type: String,
    #[pyo3(get)]
    pub(crate) default_orientation: PyPaperOrientationV1,
    #[pyo3(get)]
    pub(crate) page: PyPaperPageV1,
}

impl From<&PaperLayoutProjectionV1> for PyPaperLayoutProjectionV1 {
    fn from(value: &PaperLayoutProjectionV1) -> Self {
        Self {
            schema: value.schema().to_owned(),
            revision: value.revision(),
            digest: value
                .digest()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect(),
            paper_present: value.paper_present(),
            paper_attributes: value.paper_attributes().into(),
            effective_paper_attributes: value.effective_paper_attributes().into(),
            viewport_attributes: value.viewport_attributes().into(),
            default_type: value.default_type().to_owned(),
            default_orientation: value.default_orientation().into(),
            page: value.page().into(),
        }
    }
}

/// One exact paper-property change accepted by a complete Rust patch.
#[pyclass(
    frozen,
    module = "ferrum_chem",
    name = "DocumentPaperPropertyChangeV1",
    skip_from_py_object
)]
#[derive(Clone)]
pub(crate) struct PyDocumentPaperPropertyChangeV1 {
    pub(crate) change: PaperPropertyChangeV1,
}

#[pymethods]
impl PyDocumentPaperPropertyChangeV1 {
    /// Replace the exact recognized paper type.
    #[staticmethod]
    fn type_name(py: Python<'_>, value: String) -> PyResult<Self> {
        if paper_size_v1(&value).is_none() {
            return Err(operation_validation_error(
                py,
                "paper type is unsupported".to_owned(),
            ));
        }
        Ok(Self {
            change: PaperPropertyChangeV1::Type(value),
        })
    }

    /// Replace paper orientation.
    #[staticmethod]
    fn orientation(value: PyRef<'_, PyPaperOrientationV1>) -> Self {
        Self {
            change: PaperPropertyChangeV1::Orientation((*value).into()),
        }
    }

    /// Replace SVG crop intent.
    #[staticmethod]
    fn crop_svg(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            change: PaperPropertyChangeV1::CropSvg(exact_bool(py, value, "paper crop flag")?),
        })
    }

    /// Replace the nonnegative SVG crop margin.
    #[staticmethod]
    fn crop_margin(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        let value = exact_nonnegative_integer(py, value, "paper crop margin")?;
        Ok(Self {
            change: PaperPropertyChangeV1::CropMargin(value),
        })
    }

    /// Replace real-minus intent.
    #[staticmethod]
    fn use_real_minus(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            change: PaperPropertyChangeV1::UseRealMinus(exact_bool(
                py,
                value,
                "paper real-minus flag",
            )?),
        })
    }

    /// Replace SVG hyphen-replacement intent.
    #[staticmethod]
    fn replace_minus(py: Python<'_>, value: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self {
            change: PaperPropertyChangeV1::ReplaceMinus(exact_bool(
                py,
                value,
                "paper replace-minus flag",
            )?),
        })
    }

    /// Replace positive finite custom dimensions in millimetres.
    #[staticmethod]
    fn dimensions(
        py: Python<'_>,
        width: &Bound<'_, PyAny>,
        height: &Bound<'_, PyAny>,
    ) -> PyResult<Self> {
        let width = exact_number(py, width, "paper width")?;
        let height = exact_number(py, height, "paper height")?;
        let dimensions = PaperDimensionsMmV1::try_new(width, height)
            .map_err(|error| operation_validation_error(py, error.to_string()))?;
        Ok(Self {
            change: PaperPropertyChangeV1::Dimensions(dimensions),
        })
    }
}

fn exact_nonnegative_integer(
    py: Python<'_>,
    value: &Bound<'_, PyAny>,
    label: &str,
) -> PyResult<u64> {
    if !value.is_exact_instance_of::<PyInt>() || value.is_instance_of::<PyBool>() {
        return Err(operation_validation_error(
            py,
            format!("{label} must be an exact integer"),
        ));
    }
    value.extract::<u64>().map_err(|_| {
        operation_validation_error(py, format!("{label} is outside the supported range"))
    })
}

fn exact_bool(py: Python<'_>, value: &Bound<'_, PyAny>, label: &str) -> PyResult<bool> {
    if !value.is_exact_instance_of::<PyBool>() {
        return Err(operation_validation_error(
            py,
            format!("{label} must be an exact bool"),
        ));
    }
    value.extract::<bool>()
}

fn exact_number(py: Python<'_>, value: &Bound<'_, PyAny>, label: &str) -> PyResult<f64> {
    if value.is_instance_of::<PyBool>()
        || (!value.is_exact_instance_of::<PyInt>() && !value.is_exact_instance_of::<PyFloat>())
    {
        return Err(operation_validation_error(
            py,
            format!("{label} must be a plain number"),
        ));
    }
    value
        .extract::<f64>()
        .map_err(|_| operation_validation_error(py, format!("{label} is outside f64")))
}

/// Register the closed paper observation and mutation boundary.
pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyPaperOrientationV1>()?;
    module.add_class::<PyPaperPageIssueV1>()?;
    module.add_class::<PyPaperPageV1>()?;
    module.add_class::<PyPaperAttributesV1>()?;
    module.add_class::<PyViewportAttributesV1>()?;
    module.add_class::<PyPaperLayoutProjectionV1>()?;
    module.add_class::<PyDocumentPaperPropertyChangeV1>()?;
    Ok(())
}

pub(crate) fn validate_patch(
    py: Python<'_>,
    changes: Vec<PaperPropertyChangeV1>,
) -> PyResult<PaperPropertiesPatchV1> {
    PaperPropertiesPatchV1::new(changes)
        .map_err(|error| operation_validation_error(py, error.to_string()))
}
