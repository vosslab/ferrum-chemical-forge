"""Chemistry menu facade.

The public handler names remain here for menu registrars and compatibility callers.
Implementation families own query, asynchronous text import, and fragment behavior.
"""

import PySide6.QtCore
import PySide6.QtWidgets

from ferrum_qt.actions.action_registry import MenuAction
import ferrum_qt.actions.chemistry_fragment_actions
import ferrum_qt.actions.chemistry_info_actions
import ferrum_qt.actions.chemistry_text_import_actions


_PYSIDE6 = PySide6
_fragment = ferrum_qt.actions.chemistry_fragment_actions
_info = ferrum_qt.actions.chemistry_info_actions
_text_import = ferrum_qt.actions.chemistry_text_import_actions

# Compatibility exports for existing menu clients and focused behavior tests.
_capture_fragment_create_submit = _fragment._capture_fragment_create_submit
_capture_linear_form_submit = _fragment._capture_linear_form_submit
_convert_to_linear = _fragment._convert_to_linear
_create_fragment = _fragment._create_fragment
_external_component = _fragment._external_component
_fragment_choices = _fragment._fragment_choices
_fragment_ids_for_models = _fragment._fragment_ids_for_models
_linear_coordinate_changes = _fragment._linear_coordinate_changes
_linear_id_normalization_is_safe = _fragment._linear_id_normalization_is_safe
_linear_label_bounds = _fragment._linear_label_bounds
_linear_label_safe_spacing = _fragment._linear_label_safe_spacing
_linear_selection = _fragment._linear_selection
_linear_warning = _fragment._linear_warning
_ordered_fragment_selection = _fragment._ordered_fragment_selection
_ordered_linear_path = _fragment._ordered_linear_path
_ordered_path_bonds = _fragment._ordered_path_bonds
_other_bond_atom = _fragment._other_bond_atom
_view_fragments = _fragment._view_fragments
_active_smiles_document_session = _info._active_smiles_document_session
_atom_chemistry_display_label = _info._atom_chemistry_display_label
_chemistry_check = _info._chemistry_check
_chemistry_info = _info._chemistry_info
_compute_formula = _info._compute_formula
_compute_molecular_weight = _info._compute_molecular_weight
_expand_groups = _info._expand_groups
_gen_smiles = _info._gen_smiles
_get_mols_for_info = _info._get_mols_for_info
_int_to_roman_oxidation = _info._int_to_roman_oxidation
_oxidation_number = _info._oxidation_number
_selected_chemistry_check = _info._selected_chemistry_check
_selected_oxidation_number = _info._selected_oxidation_number
_set_name = _info._set_name
_MoleculeInsertionResultRelay = _text_import._MoleculeInsertionResultRelay
MoleculeInsertionDelivery = _text_import.MoleculeInsertionDelivery
_read_inchi = _text_import._read_inchi
_read_peptide = _text_import._read_peptide
_read_smiles = _text_import._read_smiles
_show_inchi_import_error = _text_import._show_inchi_import_error
_show_peptide_import_error = _text_import._show_peptide_import_error
_show_smiles_import_error = _text_import._show_smiles_import_error
_start_text_import = _text_import._start_text_import
_text_import_error_stage = _text_import._text_import_error_stage


def register_chemistry_actions(registry: object, app: object) -> None:
	"""Register Chemistry-menu actions without owning their behavior."""
	def has_selection() -> bool:
		return app.document is not None and app.document.has_selection

	def one_synchronized_direct_root_molecule_selected() -> bool:
		session = _active_smiles_document_session(app)
		return bool(
			session is not None and session.can_write_authoritative_snapshot
			and len(session.document.selected_direct_root_molecule_ids) == 1
		)

	def groups_selected() -> bool:
		session = _active_smiles_document_session(app)
		if session is None or not session.can_write_authoritative_snapshot:
			return False
		groups = tuple(app.document.selected_groups) if app.document is not None else ()
		if len(groups) != 1:
			return False
		item = groups[0]
		model = getattr(item, "group_model", None)
		molecule = model.parent() if model is not None else None
		return bool(
			app.document.is_current_projection_item(item)
			and model is not None and model.implicit_expandable
			and type(model.group_id) is str and model.group_id
			and molecule in app.document.molecules
			and type(getattr(molecule, "mol_id", None)) is str and molecule.mol_id
		)

	entries = (
		("chemistry.info", "Info", "Display summary formula and other info on all selected molecules", _chemistry_info, None),
		("chemistry.check", "Check chemistry", "Check if the selected objects have chemical meaning", _chemistry_check, has_selection),
		("chemistry.expand_groups", "Expand groups", "Expand one selected implicit group through OASA", _expand_groups, groups_selected),
		("chemistry.oxidation_number", "Compute oxidation number", "Compute and display the oxidation number of selected atoms", _oxidation_number, has_selection),
		("chemistry.read_smiles", "Import SMILES", "Import a SMILES string and convert it to structure", _read_smiles, None),
		("chemistry.read_inchi", "Import InChI", "Import an InChI string and convert it to structure", _read_inchi, None),
		("chemistry.read_peptide", "Import Peptide Sequence", "Import a peptide amino acid sequence and convert it to structure", _read_peptide, None),
		("chemistry.gen_smiles", "Export SMILES", "Export SMILES for the selected structure", _gen_smiles, one_synchronized_direct_root_molecule_selected),
		("chemistry.set_name", "Set molecule name", "Set the name of the selected molecule", _set_name, one_synchronized_direct_root_molecule_selected),
		("chemistry.create_fragment", "Create fragment", "Create a fragment from the selected part of the molecule", _create_fragment, has_selection),
		("chemistry.view_fragments", "View fragments", "Show already defined fragments", _view_fragments, None),
		("chemistry.convert_to_linear", "Convert selection to linear form", "Convert selected part of chain to linear fragment", _convert_to_linear, has_selection),
	)
	for action_id, label_key, help_key, handler, enabled_when in entries:
		registry.register(MenuAction(
			id=action_id, label_key=label_key, help_key=help_key, accelerator=None,
			handler=lambda handler=handler: handler(app), enabled_when=enabled_when,
		))
