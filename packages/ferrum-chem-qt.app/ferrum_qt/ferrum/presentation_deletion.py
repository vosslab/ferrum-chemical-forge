"""Ferrum deletion of a complete durable presentation selection."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets


_DELETABLE_KINDS = frozenset({
	"arrow", "plus", "text", "polyline", "rectangle", "square", "oval", "circle",
	"polygon",
})


#============================================
class FerrumNativePresentationDeletionMixin:
	"""Selection and mutation services for exact presentation-root deletion."""

	#============================================
	def has_selected_presentation_roots_for_deletion(self) -> bool:
		"""Return whether the durable selection is complete and deletable."""
		if self._disposed or self.requires_refresh:
			return False
		projection = self._controller.projection
		if projection is None:
			return False
		selected = projection.selected_durable_targets()
		return bool(selected) and all(
			target.kind in _DELETABLE_KINDS and target.durable_object_id is not None
			for target in selected
		)

	#============================================
	def has_one_selected_presentation_root(self) -> bool:
		"""Return whether one durable non-bracket presentation root is selected."""
		projection = self._controller.projection
		return (
			projection is not None
			and len(projection.selected_durable_targets()) == 1
			and self.has_selected_presentation_roots_for_deletion()
		)

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
		selected = self._controller.projection.selected_durable_targets()
		import ferrum_qt.ferrum.engine as engine
		kinds = engine.DocumentPresentationRootKindV1
		targets = tuple(
			(target.durable_object_id, getattr(kinds, target.kind))
			for target in selected
		)
		result = self._session.apply_live_presentation_deletion_v1(
			self.current_snapshot.revision, self.current_snapshot.digest, targets,
		)
		self._install_mutation_result(result)
		return result


#============================================
def install_presentation_deletion_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one generic action for renderer-owned durable presentation roots."""
	action = PySide6.QtGui.QAction(window.tr("Delete Selected Presentations"), window)
	action.setToolTip(window.tr(
		"Delete the complete selected durable presentation set through Rust",
	))
	action.triggered.connect(lambda _checked=False: _on_delete_presentation(window))
	edit_menu.addAction(action)
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
