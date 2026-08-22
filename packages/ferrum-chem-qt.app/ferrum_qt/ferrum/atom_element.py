"""Shared bounded element-change dialog for Ferrum document tabs."""

import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine as engine


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
	except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
		# Rust accepted the edit. Recover its pending authoritative projection once;
		# this is display recovery, never a second document submission.
		recovered = tab.refresh_authoritative()
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(
			"The atom element was changed. Ferrum refreshed the authoritative Rust display."
			if recovered else
			"The atom element was changed, but its authoritative display still needs "
			"recovery; refresh before saving or editing.",
		))
		return True
	except engine.OperationValidationError as exc:
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return False
	return True
