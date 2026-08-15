"""Rust-native deletion of a complete durable presentation selection."""

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
		if not selected or any(
			target.kind not in _DELETABLE_KINDS or target.identifier is None
			for target in selected
		):
			return False
		try:
			source_ids = tuple(self._presentation_source_id(target) for target in selected)
		except (RuntimeError, TypeError, ValueError):
			return False
		return self._has_complete_bracket_deletion(source_ids)

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
		import ferrum_chem
		kinds = ferrum_chem.DocumentPresentationRootKindV1
		selectors = tuple(
			ferrum_chem.DocumentPresentationRootSelectorV1.create(
				self._presentation_source_id(target), getattr(kinds, target.kind),
			)
			for target in selected
		)
		operation = ferrum_chem.DocumentOperationV1.delete_presentation_roots(
			selectors,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result)
		return result

	#============================================
	def _presentation_source_id(self, selected: object) -> str:
		"""Resolve one authenticated scene identity to its authored Rust selector."""
		if self._document_observation is None:
			raise RuntimeError("native tab has no installed document projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			target = _root_target(root)
			if target.id == selected.identifier and target.record_kind == selected.kind:
				if type(target.source_id) is not str or not target.source_id:
					raise ValueError("selected presentation has no durable source identifier")
				return target.source_id
		raise RuntimeError("selected presentation is absent from the Rust projection")

	#============================================
	def _has_complete_bracket_deletion(self, source_ids: tuple[str, ...]) -> bool:
		"""Require both authoritative bracket members when either is selected."""
		if self._document_observation is None:
			return False
		selected = frozenset(source_ids)
		pairs = self._document_observation.projection.presentation_stack.bracket_pairs
		for pair in pairs:
			selected_members = selected.intersection(pair.member_ids)
			if selected_members and selected_members != frozenset(pair.member_ids):
				return False
		return True


#============================================
def _root_target(root: object) -> object:
	"""Return the one exact target payload selected by a closed root discriminator."""
	if root.kind == "arrow":
		return root.arrow.target
	if root.kind == "plus":
		return root.plus.target
	if root.kind == "text":
		return root.text.target
	if root.kind in ("polyline", "wavy", "round_bracket"):
		return root.polyline.target
	if root.kind in ("rectangle", "square", "oval", "circle"):
		return root.shape.target
	if root.kind == "polygon":
		return root.polygon.target
	raise ValueError("Rust projection contains an unsupported presentation root kind")


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
		window._show_native_file_warning("Native Presentation Delete Error", str(exc))
		return
	window.statusBar().showMessage(window.tr("Deleted Rust-native presentation selection."), 5000)
	window._refresh_actions()
