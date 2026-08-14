//! Ferrum-Chem's compiled Python extension entry point.

mod arrow_properties_binding;
mod atom_mark_binding;
mod atom_properties_binding;
mod atom_rotation_binding;
mod binding;
mod bond_properties_binding;
mod bracket_binding;
mod chemistry_binding;
mod document_error_binding;
mod document_ingress_binding;
mod document_molecule_inchi_binding;
mod document_operation_binding;
mod geometric_properties_binding;
mod geometry_binding;
mod geometry_repair_binding;
mod inchi_insertion_binding;
mod molblock_insertion_binding;
mod molecule_coordinate_binding;
mod paper_properties_binding;
mod paper_size_binding;
mod periodic_display_binding;
mod plus_properties_binding;
mod presentation_deletion_binding;
mod presentation_root_binding;
mod presentation_stack_binding;
mod presentation_text_binding;
mod presentation_text_render_binding;
mod projection_binding;
mod render_binding;
mod sdf_insertion_binding;
mod smiles_insertion_binding;
mod text_properties_binding;
mod top_level_transform_binding;
mod wavy_properties_binding;

use pyo3::prelude::*;

/// Initialize Ferrum-Chem's public Python extension module.
#[pymodule]
fn ferrum_chem(module: &Bound<'_, PyModule>) -> PyResult<()> {
    binding::initialize(module)
}
