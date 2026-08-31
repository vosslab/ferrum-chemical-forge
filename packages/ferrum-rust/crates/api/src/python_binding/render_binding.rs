//! API-owned render observation entry points and Python module registration.

use ferrum_document::DocumentRenderObservationErrorV1;
use ferrum_render::verified_molecule_label_font;
use pyo3::create_exception;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

use super::binding::{FerrumError, map_document_error};

pub(crate) use super::render_plan_binding::{
    PyDocumentPlusRenderV1, PyPresentationTextBoundsV1, PyRenderObservationV2, observation,
    plus_from,
};
pub(crate) use super::render_primitive_binding::{
    PyGlyphPlacementV1, PyRenderOperationV3, PyRenderPaintV3, PyRenderPointV1, PyRenderTargetV1,
    frozen_tuple, operation_from, paint_from,
};

create_exception!(ferrum_chem, RenderObservationError, FerrumError);
create_exception!(ferrum_chem, RenderDepictionError, RenderObservationError);
create_exception!(ferrum_chem, RenderProvenanceError, RenderObservationError);

#[pyclass(frozen, name = "VerifiedMoleculeLabelFont", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyVerifiedMoleculeLabelFont {
    #[pyo3(get)]
    resource_id: String,
    data: Vec<u8>,
    #[pyo3(get)]
    byte_length: u64,
    #[pyo3(get)]
    sha256: String,
    #[pyo3(get)]
    family: String,
    #[pyo3(get)]
    postscript_name: String,
}

#[pymethods]
impl PyVerifiedMoleculeLabelFont {
    #[getter]
    fn data(&self, py: Python<'_>) -> Py<PyBytes> {
        PyBytes::new(py, &self.data).unbind()
    }
}

#[pyfunction]
pub(crate) fn molecule_label_font() -> PyResult<PyVerifiedMoleculeLabelFont> {
    let resource = verified_molecule_label_font()
        .map_err(|error| RenderDepictionError::new_err(error.to_string()))?;
    Ok(PyVerifiedMoleculeLabelFont {
        resource_id: resource.resource_id().to_owned(),
        data: resource.bytes().to_vec(),
        byte_length: resource.byte_length(),
        sha256: resource.sha256().to_owned(),
        family: resource.family().to_owned(),
        postscript_name: resource.postscript_name().to_owned(),
    })
}

pub(crate) fn error_result(
    py: Python<'_>,
    error: DocumentRenderObservationErrorV1,
) -> PyResult<PyErr> {
    match error {
        DocumentRenderObservationErrorV1::Document(error) => map_document_error(py, error),
        DocumentRenderObservationErrorV1::Render(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::StereoDepiction(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::StereoProjection(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::Projection(error) => {
            Ok(RenderDepictionError::new_err(error.to_string()))
        }
        DocumentRenderObservationErrorV1::ProjectionMismatch => Ok(RenderProvenanceError::new_err(
            "render observation projection identity did not match its authoritative document",
        )),
        DocumentRenderObservationErrorV1::ProvenanceMismatch => Ok(RenderProvenanceError::new_err(
            "render observation provenance did not match its authoritative document",
        )),
    }
}

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add(
        "RenderObservationError",
        module.py().get_type::<RenderObservationError>(),
    )?;
    module.add(
        "RenderDepictionError",
        module.py().get_type::<RenderDepictionError>(),
    )?;
    module.add(
        "RenderProvenanceError",
        module.py().get_type::<RenderProvenanceError>(),
    )?;
    module.add_function(wrap_pyfunction!(molecule_label_font, module)?)?;
    super::render_primitive_binding::register(module)?;
    super::render_plan_binding::register(module)?;
    super::presentation_text_render_binding::register(module)?;
    module.add_class::<PyVerifiedMoleculeLabelFont>()
}
