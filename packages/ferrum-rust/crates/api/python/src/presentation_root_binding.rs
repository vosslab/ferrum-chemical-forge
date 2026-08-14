//! Frozen Python discriminated union for direct-root presentation projections.

use ferrum_document::PresentationRootProjectionV1;
use pyo3::prelude::*;

use crate::presentation_text_binding::PyTextProjectionV1;
use crate::projection_binding::{
    PyArrowProjectionV1, PyBoxShapeProjectionV1, PyPlusProjectionV1, PyPolygonProjectionV1,
    PyPolylineProjectionV1,
};

/// One closed presentation root kind and its exact payload.
#[pyclass(frozen, name = "PresentationRootProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationRootProjectionV1 {
    #[pyo3(get)]
    pub(crate) kind: String,
    #[pyo3(get)]
    pub(crate) arrow: Option<PyArrowProjectionV1>,
    #[pyo3(get)]
    pub(crate) plus: Option<PyPlusProjectionV1>,
    #[pyo3(get)]
    pub(crate) text: Option<PyTextProjectionV1>,
    #[pyo3(get)]
    pub(crate) polyline: Option<PyPolylineProjectionV1>,
    #[pyo3(get)]
    pub(crate) shape: Option<PyBoxShapeProjectionV1>,
    #[pyo3(get)]
    pub(crate) polygon: Option<PyPolygonProjectionV1>,
}

impl From<&PresentationRootProjectionV1> for PyPresentationRootProjectionV1 {
    fn from(value: &PresentationRootProjectionV1) -> Self {
        let mut result = Self {
            kind: String::new(),
            arrow: None,
            plus: None,
            text: None,
            polyline: None,
            shape: None,
            polygon: None,
        };
        match value {
            PresentationRootProjectionV1::Arrow { arrow } => {
                result.kind = "arrow".to_owned();
                result.arrow = Some(arrow.into());
            }
            PresentationRootProjectionV1::Plus { plus } => {
                result.kind = "plus".to_owned();
                result.plus = Some(plus.into());
            }
            PresentationRootProjectionV1::Text { text } => {
                result.kind = "text".to_owned();
                result.text = Some(text.into());
            }
            PresentationRootProjectionV1::Polyline { polyline } => {
                result.kind = "polyline".to_owned();
                result.polyline = Some(polyline.into());
            }
            PresentationRootProjectionV1::Wavy { polyline } => {
                result.kind = "wavy".to_owned();
                result.polyline = Some(polyline.into());
            }
            PresentationRootProjectionV1::RoundBracket { polyline } => {
                result.kind = "round_bracket".to_owned();
                result.polyline = Some(polyline.into());
            }
            PresentationRootProjectionV1::Rectangle { shape } => {
                result.kind = "rectangle".to_owned();
                result.shape = Some(shape.into());
            }
            PresentationRootProjectionV1::Square { shape } => {
                result.kind = "square".to_owned();
                result.shape = Some(shape.into());
            }
            PresentationRootProjectionV1::Oval { shape } => {
                result.kind = "oval".to_owned();
                result.shape = Some(shape.into());
            }
            PresentationRootProjectionV1::Circle { shape } => {
                result.kind = "circle".to_owned();
                result.shape = Some(shape.into());
            }
            PresentationRootProjectionV1::Polygon { polygon } => {
                result.kind = "polygon".to_owned();
                result.polygon = Some(polygon.into());
            }
        }
        result
    }
}
