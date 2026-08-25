//! Frozen Python DTOs copied from one Rust document observation.

use ferrum_core::{BondOrder, BondStyle};
use ferrum_document::{
    AtomMarkKindV1, AtomMarkProjectionV1, AtomProjectionV1, BondEndpointV1, BondProjectionV1,
    BoxShapeProjectionV1, CompactGroupProjectionV1, DocumentHaworthPositionV1,
    DocumentProjectionV1, FontFactsV1,
    MoleculeProjectionV1, PlusProjectionV1, PolygonPathV1, PolygonProjectionV1, PolylinePathV1,
    PolylineProjectionV1, PresentationBoundsV1, PresentationFactProvenanceV1, PresentationFillV1,
    PresentationFontV1, PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1,
    PresentationRecordKindV1, PresentationStackProjectionV1, PresentationStrokeV1,
    PresentationTargetV1, ProjectionIssueCodeV1, ProjectionIssueV1, SessionDocumentObservationV1,
    VisibilityV1,
};
use pyo3::prelude::*;

pub(crate) use super::arrow_projection_binding::{
    PyArrowHeadShapeV1, PyArrowPathV1, PyArrowProjectionKindV1, PyArrowProjectionV1,
};
use super::atom_mark_binding::PyAtomMarkKindV1;
use super::binding::PyDocumentBondOrderV1;
use super::binding::PyDocumentSnapshot;
use super::bond_properties_binding::PyDocumentBondStyleV1;
use super::bracket_binding::PyBracketPairProjectionV1;
use super::drawing_standard_binding::PyDrawingStandardV1;
use super::paper_properties_binding::PyPaperLayoutProjectionV1;
use super::presentation_root_binding::PyPresentationRootProjectionV1;

#[pyclass(frozen, name = "Point3V1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPoint3V1 {
    #[pyo3(get)]
    pub(crate) x: f64,
    #[pyo3(get)]
    pub(crate) y: f64,
    #[pyo3(get)]
    pub(crate) z: f64,
}

#[pyclass(frozen, name = "FontFactsV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyFontFactsV1 {
    #[pyo3(get)]
    pub(crate) family: Option<String>,
    #[pyo3(get)]
    pub(crate) size: Option<f64>,
    #[pyo3(get)]
    pub(crate) color: Option<String>,
}

impl From<&FontFactsV1> for PyFontFactsV1 {
    fn from(value: &FontFactsV1) -> Self {
        Self {
            family: value.family().map(str::to_owned),
            size: value.size().map(|v| v.value()),
            color: value.color().map(|v| v.as_str().to_owned()),
        }
    }
}

#[pyclass(frozen, name = "BondEndpointV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyBondEndpointV1 {
    #[pyo3(get)]
    pub(crate) source_id: Option<String>,
    #[pyo3(get)]
    pub(crate) object_id: Option<String>,
    #[pyo3(get)]
    pub(crate) kind: String,
}

impl From<&BondEndpointV1> for PyBondEndpointV1 {
    fn from(value: &BondEndpointV1) -> Self {
        Self {
            source_id: value.source_id().map(str::to_owned),
            object_id: value.object_id().map(|v| v.as_str().to_owned()),
            kind: format!("{:?}", value.kind()).to_ascii_lowercase(),
        }
    }
}

#[pyclass(frozen, name = "AtomMarkProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyAtomMarkProjectionV1 {
    #[pyo3(get)]
    pub(crate) kind: PyAtomMarkKindV1,
    #[pyo3(get)]
    pub(crate) source_order: u32,
    #[pyo3(get)]
    pub(crate) same_type_ordinal: u32,
    #[pyo3(get)]
    pub(crate) angle_degrees: f64,
    #[pyo3(get)]
    pub(crate) radial_offset: f64,
    #[pyo3(get)]
    pub(crate) size: f64,
    #[pyo3(get)]
    pub(crate) draw_circle: bool,
    #[pyo3(get)]
    pub(crate) line_width: f64,
}

impl From<&AtomMarkProjectionV1> for PyAtomMarkProjectionV1 {
    fn from(value: &AtomMarkProjectionV1) -> Self {
        Self {
            kind: py_atom_mark_kind(value.kind()),
            source_order: value.source_order(),
            same_type_ordinal: value.same_type_ordinal(),
            angle_degrees: value.angle_degrees(),
            radial_offset: value.radial_offset(),
            size: value.size().value(),
            draw_circle: value.draw_circle(),
            line_width: value.line_width().value(),
        }
    }
}

fn py_atom_mark_kind(value: AtomMarkKindV1) -> PyAtomMarkKindV1 {
    match value {
        AtomMarkKindV1::Plus => PyAtomMarkKindV1::Plus,
        AtomMarkKindV1::Minus => PyAtomMarkKindV1::Minus,
        AtomMarkKindV1::Radical => PyAtomMarkKindV1::Radical,
        AtomMarkKindV1::Biradical => PyAtomMarkKindV1::Biradical,
        AtomMarkKindV1::Electronpair => PyAtomMarkKindV1::Electronpair,
        AtomMarkKindV1::DottedElectronpair => PyAtomMarkKindV1::DottedElectronpair,
        AtomMarkKindV1::PzOrbital => PyAtomMarkKindV1::PzOrbital,
    }
}

#[pyclass(frozen, name = "AtomProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyAtomProjectionV1 {
    #[pyo3(get)]
    pub(crate) id: Option<String>,
    #[pyo3(get)]
    pub(crate) projection_key: String,
    #[pyo3(get)]
    pub(crate) source_id: Option<String>,
    #[pyo3(get)]
    pub(crate) source_order: u32,
    #[pyo3(get)]
    pub(crate) element: Option<String>,
    #[pyo3(get)]
    pub(crate) position: PyPoint3V1,
    #[pyo3(get)]
    pub(crate) formal_charge: Option<i32>,
    #[pyo3(get)]
    pub(crate) isotope: Option<u16>,
    #[pyo3(get)]
    pub(crate) explicit_hydrogens: Option<u16>,
    #[pyo3(get)]
    pub(crate) valence: Option<u16>,
    #[pyo3(get)]
    pub(crate) multiplicity: Option<u16>,
    #[pyo3(get)]
    pub(crate) free_sites: Option<u16>,
    #[pyo3(get)]
    pub(crate) number: Option<u64>,
    #[pyo3(get)]
    pub(crate) show_number: Option<bool>,
    #[pyo3(get)]
    pub(crate) label_font: Option<PyFontFactsV1>,
    #[pyo3(get)]
    pub(crate) label_text: Option<String>,
    #[pyo3(get)]
    pub(crate) show: Option<bool>,
    #[pyo3(get)]
    pub(crate) show_hydrogens: Option<bool>,
    #[pyo3(get)]
    pub(crate) marks: Vec<PyAtomMarkProjectionV1>,
}

impl From<&AtomProjectionV1> for PyAtomProjectionV1 {
    fn from(value: &AtomProjectionV1) -> Self {
        let point = value.position();
        Self {
            id: value.id().map(|v| v.as_str().to_owned()),
            projection_key: value.projection_key().as_str().to_owned(),
            source_id: value.source_id().map(str::to_owned),
            source_order: value.source_order(),
            element: value.element().map(str::to_owned),
            position: PyPoint3V1 {
                x: point.x(),
                y: point.y(),
                z: point.z(),
            },
            formal_charge: value.formal_charge(),
            isotope: value.isotope(),
            explicit_hydrogens: value.explicit_hydrogens(),
            valence: value.valence(),
            multiplicity: value.multiplicity(),
            free_sites: value.free_sites(),
            number: value.number(),
            show_number: value.show_number().map(visibility),
            label_font: value.label_font().map(Into::into),
            label_text: value.label_text().map(|v| v.as_str().to_owned()),
            show: value.show().map(visibility),
            show_hydrogens: value.hydrogens().map(visibility),
            marks: value.marks().iter().map(Into::into).collect(),
        }
    }
}

fn visibility(value: VisibilityV1) -> bool {
    match value {
        VisibilityV1::Enabled => true,
        VisibilityV1::Disabled => false,
    }
}

/// Closed authored Haworth depth carried only by a projected bond fact.
#[pyclass(
    frozen,
    eq,
    hash,
    module = "ferrum_chem",
    name = "DocumentHaworthPositionV1",
    rename_all = "snake_case",
    skip_from_py_object
)]
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) enum PyDocumentHaworthPositionV1 {
    Front,
    Back,
}

#[pyclass(frozen, name = "BondProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyBondProjectionV1 {
    #[pyo3(get)]
    pub(crate) id: Option<String>,
    #[pyo3(get)]
    pub(crate) projection_key: String,
    #[pyo3(get)]
    pub(crate) source_id: Option<String>,
    #[pyo3(get)]
    pub(crate) source_order: u32,
    #[pyo3(get)]
    pub(crate) start: PyBondEndpointV1,
    #[pyo3(get)]
    pub(crate) end: PyBondEndpointV1,
    #[pyo3(get)]
    pub(crate) source_type: Option<String>,
    #[pyo3(get)]
    pub(crate) order: Option<PyDocumentBondOrderV1>,
    #[pyo3(get)]
    pub(crate) style: Option<PyDocumentBondStyleV1>,
    #[pyo3(get)]
    pub(crate) haworth_position: Option<PyDocumentHaworthPositionV1>,
    #[pyo3(get)]
    pub(crate) line_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) bond_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) wedge_width: Option<f64>,
    #[pyo3(get)]
    pub(crate) center: Option<bool>,
    #[pyo3(get)]
    pub(crate) color: Option<String>,
}

impl From<&BondProjectionV1> for PyBondProjectionV1 {
    fn from(value: &BondProjectionV1) -> Self {
        Self {
            id: value.id().map(|v| v.as_str().to_owned()),
            projection_key: value.projection_key().as_str().to_owned(),
            source_id: value.source_id().map(str::to_owned),
            source_order: value.source_order(),
            start: value.start().into(),
            end: value.end().into(),
            source_type: value.source_type().map(str::to_owned),
            order: value.order().and_then(py_bond_order),
            style: value.style().and_then(py_bond_style),
            haworth_position: value.haworth_position().map(py_haworth_position),
            line_width: value.line_width().map(|width| width.value()),
            bond_width: value.bond_width().map(|width| width.value()),
            wedge_width: value.wedge_width().map(|width| width.value()),
            center: value.center(),
            color: value.color().map(|color| color.as_str().to_owned()),
        }
    }
}

fn py_haworth_position(value: DocumentHaworthPositionV1) -> PyDocumentHaworthPositionV1 {
    match value {
        DocumentHaworthPositionV1::Front => PyDocumentHaworthPositionV1::Front,
        DocumentHaworthPositionV1::Back => PyDocumentHaworthPositionV1::Back,
    }
}

fn py_bond_order(value: BondOrder) -> Option<PyDocumentBondOrderV1> {
    match value {
        BondOrder::Single => Some(PyDocumentBondOrderV1::Single),
        BondOrder::Double => Some(PyDocumentBondOrderV1::Double),
        BondOrder::Triple => Some(PyDocumentBondOrderV1::Triple),
        BondOrder::Aromatic | BondOrder::Other(_) => None,
    }
}

fn py_bond_style(value: &BondStyle) -> Option<PyDocumentBondStyleV1> {
    match value {
        BondStyle::Normal => Some(PyDocumentBondStyleV1::Normal),
        BondStyle::Wedge => Some(PyDocumentBondStyleV1::Wedge),
        BondStyle::Hashed => Some(PyDocumentBondStyleV1::HashedWedge),
        BondStyle::Adder => Some(PyDocumentBondStyleV1::Adder),
        BondStyle::Bold => Some(PyDocumentBondStyleV1::Bold),
        BondStyle::Dashed => Some(PyDocumentBondStyleV1::Dashed),
        BondStyle::Dotted => Some(PyDocumentBondStyleV1::Dotted),
        BondStyle::Wavy => Some(PyDocumentBondStyleV1::Wavy),
        BondStyle::HaworthFront => Some(PyDocumentBondStyleV1::HaworthFront),
        BondStyle::Other(_) => None,
    }
}

/// Frozen typed compact-group facts copied from one molecule projection.
#[pyclass(frozen, name = "CompactGroupProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyCompactGroupProjectionV1 {
    #[pyo3(get)]
    pub(crate) id: String,
    #[pyo3(get)]
    pub(crate) catalog_key: String,
    #[pyo3(get)]
    pub(crate) label: String,
    #[pyo3(get)]
    pub(crate) anchor: PyPoint3V1,
    #[pyo3(get)]
    pub(crate) attachment_index: u8,
    #[pyo3(get)]
    pub(crate) orientation_degrees: f64,
    #[pyo3(get)]
    pub(crate) source_order: u32,
}

impl From<&CompactGroupProjectionV1> for PyCompactGroupProjectionV1 {
    fn from(value: &CompactGroupProjectionV1) -> Self {
        let anchor = value.anchor();
        Self {
            id: value.id().as_str().to_owned(),
            catalog_key: value.catalog_key().as_str().to_owned(),
            label: value.label().to_owned(),
            anchor: PyPoint3V1 {
                x: anchor.x(),
                y: anchor.y(),
                z: anchor.z(),
            },
            attachment_index: value.attachment_index(),
            orientation_degrees: value.orientation_degrees(),
            source_order: value.source_order(),
        }
    }
}

#[pyclass(frozen, name = "MoleculeProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyMoleculeProjectionV1 {
    #[pyo3(get)]
    pub(crate) id: Option<String>,
    #[pyo3(get)]
    pub(crate) projection_key: String,
    #[pyo3(get)]
    pub(crate) source_id: Option<String>,
    #[pyo3(get)]
    pub(crate) source_order: u32,
    #[pyo3(get)]
    pub(crate) name: Option<String>,
    #[pyo3(get)]
    pub(crate) atoms: Vec<PyAtomProjectionV1>,
    #[pyo3(get)]
    pub(crate) compact_groups: Vec<PyCompactGroupProjectionV1>,
    #[pyo3(get)]
    pub(crate) bonds: Vec<PyBondProjectionV1>,
}

impl From<&MoleculeProjectionV1> for PyMoleculeProjectionV1 {
    fn from(value: &MoleculeProjectionV1) -> Self {
        Self {
            id: value.id().map(|v| v.as_str().to_owned()),
            projection_key: value.projection_key().as_str().to_owned(),
            source_id: value.source_id().map(str::to_owned),
            source_order: value.source_order(),
            name: value.name().map(str::to_owned),
            atoms: value.atoms().iter().map(Into::into).collect(),
            compact_groups: value.compact_groups().iter().map(Into::into).collect(),
            bonds: value.bonds().iter().map(Into::into).collect(),
        }
    }
}

#[pyclass(frozen, name = "ProjectionIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyProjectionIssueV1 {
    #[pyo3(get)]
    pub(crate) code: String,
    #[pyo3(get)]
    pub(crate) path: String,
    #[pyo3(get)]
    pub(crate) detail: String,
}
impl From<&ProjectionIssueV1> for PyProjectionIssueV1 {
    fn from(value: &ProjectionIssueV1) -> Self {
        Self {
            code: py_projection_issue_code(value.code()).to_owned(),
            path: value.path().to_owned(),
            detail: value.detail().to_owned(),
        }
    }
}

fn py_projection_issue_code(value: ProjectionIssueCodeV1) -> &'static str {
    match value {
        ProjectionIssueCodeV1::MissingBondEndpoint => "missing_bond_endpoint",
        ProjectionIssueCodeV1::UnsupportedBondEndpoint => "unsupported_bond_endpoint",
        ProjectionIssueCodeV1::UnknownBondEndpoint => "unknown_bond_endpoint",
        ProjectionIssueCodeV1::UnsupportedBondType => "unsupported_bond_type",
        ProjectionIssueCodeV1::InvalidPresentationFact => "invalid_presentation_fact",
    }
}

#[pyclass(frozen, name = "DocumentProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyDocumentProjectionV1 {
    #[pyo3(get)]
    pub(crate) schema: String,
    #[pyo3(get)]
    pub(crate) revision: u64,
    #[pyo3(get)]
    pub(crate) digest: String,
    #[pyo3(get)]
    pub(crate) is_dirty: bool,
    #[pyo3(get)]
    pub(crate) paper_layout: PyPaperLayoutProjectionV1,
    #[pyo3(get)]
    pub(crate) drawing_standard: Option<PyDrawingStandardV1>,
    #[pyo3(get)]
    pub(crate) molecules: Vec<PyMoleculeProjectionV1>,
    #[pyo3(get)]
    pub(crate) presentation_stack: PyPresentationStackProjectionV1,
    #[pyo3(get)]
    pub(crate) issues: Vec<PyProjectionIssueV1>,
}

impl From<&DocumentProjectionV1> for PyDocumentProjectionV1 {
    fn from(value: &DocumentProjectionV1) -> Self {
        Self {
            schema: value.schema().to_owned(),
            revision: value.revision(),
            digest: value.digest().iter().map(|b| format!("{b:02x}")).collect(),
            is_dirty: value.is_dirty(),
            paper_layout: value.paper_layout().into(),
            drawing_standard: value.drawing_standard().map(Into::into),
            molecules: value.molecules().iter().map(Into::into).collect(),
            presentation_stack: value.presentation_stack().into(),
            issues: value.issues().iter().map(Into::into).collect(),
        }
    }
}

/// Frozen direct-root presentation facts copied from one document observation.
#[pyclass(frozen, name = "PresentationStackProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationStackProjectionV1 {
    #[pyo3(get)]
    pub(crate) schema: String,
    #[pyo3(get)]
    pub(crate) revision: u64,
    #[pyo3(get)]
    pub(crate) digest: String,
    #[pyo3(get)]
    pub(crate) roots: Vec<PyPresentationRootProjectionV1>,
    #[pyo3(get)]
    pub(crate) bracket_pairs: Vec<PyBracketPairProjectionV1>,
    #[pyo3(get)]
    pub(crate) issues: Vec<PyPresentationProjectionIssueV1>,
}

impl From<&PresentationStackProjectionV1> for PyPresentationStackProjectionV1 {
    fn from(value: &PresentationStackProjectionV1) -> Self {
        Self {
            schema: value.schema().to_owned(),
            revision: value.revision(),
            digest: hex_digest(value.digest()),
            roots: value.roots().iter().map(Into::into).collect(),
            bracket_pairs: value.bracket_pairs().iter().map(Into::into).collect(),
            issues: value.issues().iter().map(Into::into).collect(),
        }
    }
}

/// Resolved font facts for one fixed-content presentation root.
#[pyclass(frozen, name = "PresentationFontV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationFontV1 {
    #[pyo3(get)]
    pub(crate) font_face_id: String,
    #[pyo3(get)]
    pub(crate) font_face_provenance: String,
    #[pyo3(get)]
    pub(crate) size: f64,
    #[pyo3(get)]
    pub(crate) size_provenance: String,
    #[pyo3(get)]
    pub(crate) color: String,
    #[pyo3(get)]
    pub(crate) color_provenance: String,
}

impl From<&PresentationFontV1> for PyPresentationFontV1 {
    fn from(value: &PresentationFontV1) -> Self {
        Self {
            font_face_id: value.font_face().id().to_owned(),
            font_face_provenance: presentation_fact_provenance(value.font_face_provenance())
                .to_owned(),
            size: value.size().value(),
            size_provenance: presentation_fact_provenance(value.size_provenance()).to_owned(),
            color: value.color().as_str().to_owned(),
            color_provenance: presentation_fact_provenance(value.color_provenance()).to_owned(),
        }
    }
}

/// One fixed-content plus before verified glyph layout.
#[pyclass(frozen, name = "PlusProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPlusProjectionV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) anchor: PyPoint3V1,
    #[pyo3(get)]
    pub(crate) font: PyPresentationFontV1,
    #[pyo3(get)]
    pub(crate) background: PyPresentationFillV1,
}

impl From<&PlusProjectionV1> for PyPlusProjectionV1 {
    fn from(value: &PlusProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            anchor: PyPoint3V1 {
                x: value.anchor().x(),
                y: value.anchor().y(),
                z: value.anchor().z(),
            },
            font: value.font().into(),
            background: value.background().into(),
        }
    }
}

/// One direct-root non-spline polyline projection.
#[pyclass(frozen, name = "PolylineProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPolylineProjectionV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) path: PyPolylinePathV1,
    #[pyo3(get)]
    pub(crate) stroke: PyPresentationStrokeV1,
}

impl From<&PolylineProjectionV1> for PyPolylineProjectionV1 {
    fn from(value: &PolylineProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            path: value.path().into(),
            stroke: value.stroke().into(),
        }
    }
}

/// Durable-or-local target provenance for one presentation root.
#[pyclass(frozen, name = "PresentationTargetV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationTargetV1 {
    #[pyo3(get)]
    pub(crate) id: Option<String>,
    #[pyo3(get)]
    pub(crate) projection_key: String,
    #[pyo3(get)]
    pub(crate) source_id: Option<String>,
    #[pyo3(get)]
    pub(crate) source_order: u32,
    #[pyo3(get)]
    pub(crate) record_kind: String,
}

impl From<&PresentationTargetV1> for PyPresentationTargetV1 {
    fn from(value: &PresentationTargetV1) -> Self {
        Self {
            id: value.id().map(|id| id.as_str().to_owned()),
            projection_key: value.projection_key().as_str().to_owned(),
            source_id: value.source_id().map(str::to_owned),
            source_order: value.source_order(),
            record_kind: presentation_record_kind(value.record_kind()).to_owned(),
        }
    }
}

/// Every ordered point of a supported direct-root polyline.
#[pyclass(frozen, name = "PolylinePathV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPolylinePathV1 {
    #[pyo3(get)]
    pub(crate) points: Vec<PyPoint3V1>,
}

impl From<&PolylinePathV1> for PyPolylinePathV1 {
    fn from(value: &PolylinePathV1) -> Self {
        Self {
            points: value
                .points()
                .iter()
                .map(|point| PyPoint3V1 {
                    x: point.x(),
                    y: point.y(),
                    z: point.z(),
                })
                .collect(),
        }
    }
}

/// Fully resolved display stroke values and their explicit selection sources.
#[pyclass(frozen, name = "PresentationStrokeV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationStrokeV1 {
    #[pyo3(get)]
    pub(crate) color: String,
    #[pyo3(get)]
    pub(crate) color_provenance: String,
    #[pyo3(get)]
    pub(crate) width: f64,
    #[pyo3(get)]
    pub(crate) width_provenance: String,
}

impl From<&PresentationStrokeV1> for PyPresentationStrokeV1 {
    fn from(value: &PresentationStrokeV1) -> Self {
        Self {
            color: value.color().as_str().to_owned(),
            color_provenance: presentation_fact_provenance(value.color_provenance()).to_owned(),
            width: value.width().value(),
            width_provenance: presentation_fact_provenance(value.width_provenance()).to_owned(),
        }
    }
}

/// Normalized finite scene bounds copied from a box-shape projection.
#[pyclass(frozen, name = "PresentationBoundsV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationBoundsV1 {
    #[pyo3(get)]
    pub(crate) left: f64,
    #[pyo3(get)]
    pub(crate) top: f64,
    #[pyo3(get)]
    pub(crate) right: f64,
    #[pyo3(get)]
    pub(crate) bottom: f64,
}

impl From<PresentationBoundsV1> for PyPresentationBoundsV1 {
    fn from(value: PresentationBoundsV1) -> Self {
        Self {
            left: value.left(),
            top: value.top(),
            right: value.right(),
            bottom: value.bottom(),
        }
    }
}

/// Resolved optional shape fill and the source that supplied it.
#[pyclass(frozen, name = "PresentationFillV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationFillV1 {
    #[pyo3(get)]
    pub(crate) color: Option<String>,
    #[pyo3(get)]
    pub(crate) color_provenance: String,
}

impl From<&PresentationFillV1> for PyPresentationFillV1 {
    fn from(value: &PresentationFillV1) -> Self {
        Self {
            color: value.color().map(|color| color.as_str().to_owned()),
            color_provenance: presentation_fact_provenance(value.color_provenance()).to_owned(),
        }
    }
}

/// One rectangle, square, oval, or circle payload.
#[pyclass(frozen, name = "BoxShapeProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyBoxShapeProjectionV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) bounds: PyPresentationBoundsV1,
    #[pyo3(get)]
    pub(crate) stroke: PyPresentationStrokeV1,
    #[pyo3(get)]
    pub(crate) fill: PyPresentationFillV1,
}

impl From<&BoxShapeProjectionV1> for PyBoxShapeProjectionV1 {
    fn from(value: &BoxShapeProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            bounds: value.bounds().into(),
            stroke: value.stroke().into(),
            fill: value.fill().into(),
        }
    }
}

/// Every ordered point of one closed direct-root polygon.
#[pyclass(frozen, name = "PolygonPathV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPolygonPathV1 {
    #[pyo3(get)]
    pub(crate) points: Vec<PyPoint3V1>,
}

impl From<&PolygonPathV1> for PyPolygonPathV1 {
    fn from(value: &PolygonPathV1) -> Self {
        Self {
            points: value
                .points()
                .iter()
                .map(|point| PyPoint3V1 {
                    x: point.x(),
                    y: point.y(),
                    z: point.z(),
                })
                .collect(),
        }
    }
}

/// One direct-root polygon payload.
#[pyclass(frozen, name = "PolygonProjectionV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPolygonProjectionV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) path: PyPolygonPathV1,
    #[pyo3(get)]
    pub(crate) stroke: PyPresentationStrokeV1,
    #[pyo3(get)]
    pub(crate) fill: PyPresentationFillV1,
}

impl From<&PolygonProjectionV1> for PyPolygonProjectionV1 {
    fn from(value: &PolygonProjectionV1) -> Self {
        Self {
            target: value.target().into(),
            path: value.path().into(),
            stroke: value.stroke().into(),
            fill: value.fill().into(),
        }
    }
}

/// A deterministic direct-root projection issue with its display-only target.
#[pyclass(frozen, name = "PresentationProjectionIssueV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PyPresentationProjectionIssueV1 {
    #[pyo3(get)]
    pub(crate) target: PyPresentationTargetV1,
    #[pyo3(get)]
    pub(crate) code: String,
    #[pyo3(get)]
    pub(crate) detail: String,
}

impl From<&PresentationProjectionIssueV1> for PyPresentationProjectionIssueV1 {
    fn from(value: &PresentationProjectionIssueV1) -> Self {
        Self {
            target: value.target().into(),
            code: presentation_issue_code(value.code()).to_owned(),
            detail: value.detail().to_owned(),
        }
    }
}

fn presentation_fact_provenance(value: PresentationFactProvenanceV1) -> &'static str {
    match value {
        PresentationFactProvenanceV1::Root => "root",
        PresentationFactProvenanceV1::Standard => "standard",
        PresentationFactProvenanceV1::Builtin => "builtin",
    }
}

fn presentation_record_kind(value: PresentationRecordKindV1) -> &'static str {
    match value {
        PresentationRecordKindV1::Arrow => "arrow",
        PresentationRecordKindV1::Plus => "plus",
        PresentationRecordKindV1::Text => "text",
        PresentationRecordKindV1::Polyline => "polyline",
        PresentationRecordKindV1::Rectangle => "rectangle",
        PresentationRecordKindV1::Square => "square",
        PresentationRecordKindV1::Oval => "oval",
        PresentationRecordKindV1::Circle => "circle",
        PresentationRecordKindV1::Polygon => "polygon",
    }
}

fn presentation_issue_code(value: PresentationProjectionIssueCodeV1) -> &'static str {
    match value {
        PresentationProjectionIssueCodeV1::InvalidArrowGeometry => "invalid_arrow_geometry",
        PresentationProjectionIssueCodeV1::InvalidArrowFact => "invalid_arrow_fact",
        PresentationProjectionIssueCodeV1::UnsupportedArrowType => "unsupported_arrow_type",
        PresentationProjectionIssueCodeV1::UnsupportedArrowSpline => "unsupported_arrow_spline",
        PresentationProjectionIssueCodeV1::InvalidPlusGeometry => "invalid_plus_geometry",
        PresentationProjectionIssueCodeV1::InvalidTextGeometry => "invalid_text_geometry",
        PresentationProjectionIssueCodeV1::InvalidTextContent => "invalid_text_content",
        PresentationProjectionIssueCodeV1::InvalidFontFact => "invalid_font_fact",
        PresentationProjectionIssueCodeV1::UnsupportedTextFace => "unsupported_text_face",
        PresentationProjectionIssueCodeV1::InvalidPolylineGeometry => "invalid_polyline_geometry",
        PresentationProjectionIssueCodeV1::InvalidShapeGeometry => "invalid_shape_geometry",
        PresentationProjectionIssueCodeV1::InvalidPolygonGeometry => "invalid_polygon_geometry",
        PresentationProjectionIssueCodeV1::UnsupportedSpline => "unsupported_spline",
        PresentationProjectionIssueCodeV1::UnsupportedPolylineStyle => "unsupported_polyline_style",
        PresentationProjectionIssueCodeV1::InvalidStrokeFact => "invalid_stroke_fact",
        PresentationProjectionIssueCodeV1::InvalidFillFact => "invalid_fill_fact",
    }
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[pyclass(frozen, name = "SessionDocumentObservationV1", skip_from_py_object)]
#[derive(Clone)]
pub(crate) struct PySessionDocumentObservationV1 {
    observation: SessionDocumentObservationV1,
    #[pyo3(get)]
    pub(crate) snapshot: PyDocumentSnapshot,
    #[pyo3(get)]
    pub(crate) projection: PyDocumentProjectionV1,
}
impl From<SessionDocumentObservationV1> for PySessionDocumentObservationV1 {
    fn from(value: SessionDocumentObservationV1) -> Self {
        Self {
            observation: value.clone(),
            snapshot: value.snapshot().clone().into(),
            projection: value.projection().into(),
        }
    }
}

impl PySessionDocumentObservationV1 {
    pub(crate) fn observation(&self) -> &SessionDocumentObservationV1 {
        &self.observation
    }
}
