"""Ferrum deletion of a complete durable presentation selection."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
FerrumNativeDocumentTabError = native_document_tab_errors.FerrumNativeDocumentTabError


#============================================
class FerrumNativePresentationDeletionMixin:
	"""Selection and mutation services for exact presentation-root deletion."""

	#============================================
	def has_selected_presentation_roots_for_deletion(self) -> bool:
		"""Return whether the durable selection is complete and deletable."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			return bool(self._selected_presentation_root_selectors())
		except FerrumNativeDocumentTabError:
			return False

	#============================================
	def has_one_selected_presentation_root(self) -> bool:
		"""Return whether one durable non-bracket presentation root is selected."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			return len(self._selected_presentation_root_selectors()) == 1
		except FerrumNativeDocumentTabError:
			return False

	#============================================
	def delete_selected_presentation_root(self) -> object:
		"""Delete one exact selected presentation root through the Rust session."""
		if not self.has_one_selected_presentation_root():
			raise RuntimeError("select one durable non-bracket presentation root first")
		return self.delete_selected_presentation_roots()

	#============================================
	def delete_selected_presentation_roots(self) -> object:
		"""Delete one complete durable selection through one Rust operation."""
		self._require_mutable()
		if not self.has_selected_presentation_roots_for_deletion():
			raise RuntimeError("select a complete durable presentation root set first")
		targets = self._selected_presentation_root_selectors()
		result = self._session.apply_live_presentation_deletion_v1(
			self.current_snapshot.revision, self.current_snapshot.digest, targets,
		)
		self._install_mutation_result(result)
		return result


#============================================
def install_presentation_deletion_action(window: object) -> PySide6.QtGui.QAction:
	"""Construct one generic action for renderer-owned durable presentation roots."""
	action = PySide6.QtGui.QAction(window.tr("Delete Selected Presentations"), window)
	action.setToolTip(window.tr(
		"Delete the complete selected durable presentation set through Rust",
	))
	action.triggered.connect(lambda _checked=False: _on_delete_presentation(window))
	window._register_action("edit.delete_presentations", action)
	return action


#============================================
def refresh_presentation_deletion_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Make deletion follow exact current presentation selection and authority state."""
	action.setEnabled(
		tab is not None and active and not pending and not busy
		and tab.has_selected_presentation_roots_for_deletion(),
	)


#============================================
def _on_delete_presentation(window: object) -> None:
	"""Submit one selected presentation deletion with visible failure containment."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		tab.delete_selected_presentation_roots()
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Deleted Ferrum presentation selection."), 5000)
	window._refresh_actions()
