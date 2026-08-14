"""Shared bounded element-change dialog for Rust-native document tabs."""

# PIP3 modules
import PySide6.QtWidgets


#============================================
def can_change_selected_atom_element(tab: object | None) -> bool:
	"""Return whether one current durable atom can receive an element change."""
	if tab is None or tab.requires_refresh:
		return False
	try:
		return bool(tab.has_one_selected_atom())
	except (AttributeError, RuntimeError):
		return False


#============================================
def run_change_selected_atom_element_dialog(window: object) -> bool:
	"""Collect untrusted dialog text and submit it to the active Rust tab only."""
	tab = window._active_native_tab()
	if not can_change_selected_atom_element(tab):
		return False
	element, accepted = PySide6.QtWidgets.QInputDialog.getText(
		window, window.tr("Change Atom Element"), window.tr("Element symbol:"),
	)
	if not accepted:
		return False
	try:
		tab.change_selected_atom_element(element)
	except Exception as exc:
		window._show_native_file_warning("Native Edit Error", str(exc))
		return False
	return True
