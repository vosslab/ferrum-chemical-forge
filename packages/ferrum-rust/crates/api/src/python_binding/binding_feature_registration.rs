//! Registration of feature-owned Python functions and private operation seams.

use pyo3::prelude::*;

pub(crate) fn initialize(module: &Bound<'_, PyModule>) -> PyResult<()> {
    super::chemistry_binding::initialize(module)?;
    super::curved_electron_arrow_gesture_binding::initialize(module)?;
    super::curved_normal_reaction_arrow_gesture_binding::initialize(module)?;
    super::curved_equilibrium_arrow_gesture_binding::initialize(module)?;
    super::curved_retro_arrow_gesture_binding::initialize(module)?;
    super::clipboard_fragment_binding::initialize(module)?;
    super::clipboard_cut_binding::initialize(module)?;
    super::clipboard_paste_binding::initialize(module)?;
    super::document_linear_form_binding::initialize(module)?;
    super::document_explicit_fragment_binding::initialize(module)?;
    super::document_molecule_inchi_binding::initialize(module)?;
    super::document_bond_capacity_binding::initialize(module)?;
    super::document_molecule_diagnostics_binding::initialize(module)?;
    super::document_molecule_inspection_binding::initialize(module)?;
    super::document_molecule_molblock_binding::initialize(module)?;
    super::document_molecule_sdf_binding::initialize(module)?;
    super::document_molecules_sdf_binding::initialize(module)?;
    super::document_molecule_name_binding::initialize(module)?;
    super::document_molecule_smiles_binding::initialize(module)?;
    super::document_native_artifact_binding::initialize(module)?;
    super::document_selection_svg_binding::initialize(module)?;
    super::document_user_template_binding::initialize(module)?;
    super::drawing_standard_binding::initialize(module)?;
    super::geometry_binding::initialize(module)?;
    super::molecule_coordinate_binding::initialize(module)?;
    super::paper_properties_binding::initialize(module)?;
    super::paper_size_binding::initialize(module)?;
    super::periodic_display_binding::initialize(module)?;
    super::protocol_binding::initialize(module)?;
    super::direct_haworth_binding::initialize(module)?;
    super::direct_bond_gesture_binding::initialize(module)?;
    super::text_placement_gesture_binding::initialize(module)?;
    super::presentation_creation_gesture_binding::initialize(module)?;
    super::presentation_path_gesture_binding::initialize(module)?;
    super::presentation_vector_gesture_binding::initialize(module)?;
    super::catalog_placement_binding::initialize(module)?;
    super::reaction_binding::initialize(module)?;
    super::direct_root_interaction_binding::initialize(module)?;
    super::live_document_smarts_query_v1::initialize(module)?;
    super::molecule_insertion_binding::initialize(module)?;
    super::smiles_insertion_binding::initialize(module)?;
    super::peptide_insertion_binding::initialize(module)?;
    super::inchi_insertion_binding::initialize(module)?;
    super::molblock_insertion_binding::initialize(module)?;
    super::interchange_insertion_binding::initialize(module)?;
    super::sdf_insertion_binding::initialize(module)
}
