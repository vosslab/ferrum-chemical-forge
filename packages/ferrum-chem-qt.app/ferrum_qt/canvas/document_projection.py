"""Compatibility facade for disposable document-to-Qt projection helpers.

New code should import the focused presentation, selection, or mark module.  This
module deliberately retains the legacy public surface while coordinating only
scene-local projections; it never owns persistent document facts.
"""

# local repo modules
import ferrum_qt.canvas.mark_projection
import ferrum_qt.canvas.presentation_projection
import ferrum_qt.canvas.selection_projection

# Legacy imports remain stable while callers migrate to focused modules.
_ProjectionBinding = ferrum_qt.canvas.presentation_projection._ProjectionBinding
is_bound_presentation_projection = ferrum_qt.canvas.presentation_projection.is_bound_presentation_projection
dispose_item_callbacks = ferrum_qt.canvas.presentation_projection.dispose_item_callbacks
create_presentation_item = ferrum_qt.canvas.presentation_projection.create_presentation_item
project_presentation_objects = ferrum_qt.canvas.presentation_projection.project_presentation_objects
synchronize_document_stack_z_order = ferrum_qt.canvas.presentation_projection.synchronize_document_stack_z_order

selected_presentation_stack_root_ids = ferrum_qt.canvas.selection_projection.selected_presentation_stack_root_ids
selected_top_level_transform_keys = ferrum_qt.canvas.selection_projection.selected_top_level_transform_keys
top_level_presentation_keys_for_items = ferrum_qt.canvas.selection_projection.top_level_presentation_keys_for_items
selection_translate_targets_for_items = ferrum_qt.canvas.selection_projection.selection_translate_targets_for_items
StructuralSelectionKind = ferrum_qt.canvas.selection_projection.StructuralSelectionKind
StructuralSelectionClassification = ferrum_qt.canvas.selection_projection.StructuralSelectionClassification
classify_structural_selection = ferrum_qt.canvas.selection_projection.classify_structural_selection
structure_delete_targets_for_items = ferrum_qt.canvas.selection_projection.structure_delete_targets_for_items
persistent_selection_key = ferrum_qt.canvas.selection_projection.persistent_selection_key
atom_mark_delete_target_for_items = ferrum_qt.canvas.selection_projection.atom_mark_delete_target_for_items
select_projected_persistent_keys = ferrum_qt.canvas.selection_projection.select_projected_persistent_keys

create_mark_item = ferrum_qt.canvas.mark_projection.create_mark_item
dispose_detached_items = ferrum_qt.canvas.mark_projection.dispose_detached_items
project_marks = ferrum_qt.canvas.mark_projection.project_marks


#============================================
def project_document_presentation(document: object, scene: object) -> dict:
	"""Apply paper state and create this scene's disposable artwork projection."""
	if document.paper.attributes and hasattr(scene, "apply_paper_model"):
		scene.apply_paper_model(document.paper)
	presentation = project_presentation_objects(document, scene)
	marks = project_marks(document, scene)
	synchronize_document_stack_z_order(document, scene)
	projected = {"presentation": presentation, "marks": marks}
	return projected
