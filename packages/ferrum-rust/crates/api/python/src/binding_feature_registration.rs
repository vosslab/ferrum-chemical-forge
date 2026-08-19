//! Registration of feature-owned Python functions and private operation seams.

use pyo3::prelude::*;

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    crate::chemistry_binding::initialize(module)?;
    crate::clipboard_fragment_binding::initialize(module)?;
    crate::clipboard_cut_binding::initialize(module)?;
    crate::clipboard_paste_binding::initialize(module)?;
    crate::document_linear_form_binding::initialize(module)?;
    crate::document_explicit_fragment_binding::initialize(module)?;
    crate::document_molecule_inchi_binding::initialize(module)?;
    crate::document_molecule_information_binding::initialize(module)?;
    crate::document_bond_capacity_binding::initialize(module)?;
    crate::document_molecule_inspection_binding::initialize(module)?;
    crate::document_molecule_molblock_binding::initialize(module)?;
    crate::document_molecule_sdf_binding::initialize(module)?;
    crate::document_molecule_name_binding::initialize(module)?;
    crate::document_molecule_smiles_binding::initialize(module)?;
    crate::document_native_artifact_binding::initialize(module)?;
    crate::document_selection_svg_binding::initialize(module)?;
    crate::document_user_template_binding::initialize(module)?;
    crate::drawing_standard_binding::initialize(module)?;
    crate::geometry_binding::initialize(module)?;
    crate::molecule_coordinate_binding::initialize(module)?;
    crate::paper_properties_binding::initialize(module)?;
    crate::paper_size_binding::initialize(module)?;
    crate::periodic_display_binding::initialize(module)?;
    crate::protocol_binding::initialize(module)?;
    crate::direct_haworth_binding::initialize(module)?;
    crate::smiles_insertion_binding::initialize(module)?;
    crate::peptide_template_insertion_binding::initialize(module)?;
    crate::inchi_insertion_binding::initialize(module)?;
    crate::molblock_insertion_binding::initialize(module)?;
    crate::sdf_insertion_binding::initialize(module)
}
