//! Python constructors for interaction queries and translation snapping.

use super::{
    super::document_error_binding::document_object_id as parse_document_object_id, types::*,
};
use crate::{
    RenderInteractionModifierV1, RenderInteractionQueryV1, RenderInteractionSnapV1,
    StructureInteractionQueryV1,
};
use pyo3::prelude::*;

#[pyclass(frozen, module = "ferrum_chem", name = "RenderInteractionQueryV1")]
pub(crate) struct PyQuery {
    pub(super) query: RenderInteractionQueryV1,
}
#[pymethods]
impl PyQuery {
    #[staticmethod]
    fn point(x: f64, y: f64, modifier: PyRef<'_, PyModifier>) -> Self {
        Self {
            query: RenderInteractionQueryV1::Point {
                x,
                y,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    fn marquee(
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        modifier: PyRef<'_, PyModifier>,
    ) -> Self {
        Self {
            query: RenderInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    #[pyo3(signature = (document_object_id, modifier = None))]
    fn root(
        py: Python<'_>,
        document_object_id: String,
        modifier: Option<PyRef<'_, PyModifier>>,
    ) -> PyResult<Self> {
        let document_object_id = parse_document_object_id(py, document_object_id)?;
        Ok(Self {
            query: RenderInteractionQueryV1::Root {
                document_object_id,
                modifier: modifier.map_or(RenderInteractionModifierV1::Replace, |value| {
                    (*value).into()
                }),
            },
        })
    }
    #[staticmethod]
    fn clear() -> Self {
        Self {
            query: RenderInteractionQueryV1::Clear,
        }
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "StructureInteractionQueryV1")]
pub(crate) struct PyStructureQuery {
    pub(super) query: StructureInteractionQueryV1,
}
#[pymethods]
impl PyStructureQuery {
    #[staticmethod]
    fn point(x: f64, y: f64, modifier: PyRef<'_, PyModifier>) -> Self {
        Self {
            query: StructureInteractionQueryV1::Point {
                x,
                y,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    fn marquee(
        left: f64,
        top: f64,
        right: f64,
        bottom: f64,
        modifier: PyRef<'_, PyModifier>,
    ) -> Self {
        Self {
            query: StructureInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                modifier: (*modifier).into(),
            },
        }
    }
    #[staticmethod]
    fn clear() -> Self {
        Self {
            query: StructureInteractionQueryV1::Clear,
        }
    }
}
#[pyclass(frozen, module = "ferrum_chem", name = "RenderInteractionSnapV1")]
pub(crate) struct PySnap {
    pub(super) snap: RenderInteractionSnapV1,
}
#[pymethods]
impl PySnap {
    #[new]
    fn new(axis: PyRef<'_, PyAxis>) -> Self {
        Self {
            snap: RenderInteractionSnapV1::new((*axis).into()),
        }
    }
    #[staticmethod]
    fn free() -> Self {
        Self {
            snap: RenderInteractionSnapV1::free(),
        }
    }
    #[staticmethod]
    fn with_grid_policy(axis: PyRef<'_, PyAxis>, policy: PyRef<'_, PyGridSnapPolicy>) -> Self {
        Self {
            snap: RenderInteractionSnapV1::with_grid_policy((*axis).into(), (*policy).into()),
        }
    }
}
