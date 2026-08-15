"""Shared host-owned operation containment for one native document tab."""


_TAB_OPERATION_INTENTS = (
	"_smiles_import_intent", "_inchi_import_intent", "_molblock_import_intent",
	"_sdf_import_intent", "_peptide_import_intent", "_molecule_export_intent",
	"_molecule_inspection_intent", "_clipboard_copy_intent", "_clipboard_cut_intent",
	"_bond_capacity_intent",
	"_clipboard_paste_intent", "_coordinate_generation_intent",
	"_user_template_placement_intent",
	"_snapshot_export_intent",
)


#============================================
def tab_has_active_native_operation(window: object, tab: object) -> bool:
	"""Return whether existing host-owned asynchronous work retains this tab."""
	return any(
		_tab_owned_by_intent(getattr(window, name, None)) is tab
		for name in _TAB_OPERATION_INTENTS
	)


#============================================
def _tab_owned_by_intent(intent: object) -> object | None:
	"""Return the exact tab retained by one established native intent shape."""
	tab = getattr(intent, "tab", None)
	if tab is not None:
		return tab
	capture = getattr(intent, "capture", None)
	return getattr(capture, "tab", None)
