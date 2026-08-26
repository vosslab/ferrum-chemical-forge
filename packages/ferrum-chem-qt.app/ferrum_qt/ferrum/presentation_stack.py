"""Ferrum ordering of durable direct-root presentation records."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
FerrumNativeDocumentTabError = native_document_tab_errors.FerrumNativeDocumentTabError


#============================================
class FerrumNativePresentationStackMixin:
	"""Selection and mutation services for Rust-owned presentation ordering."""

	#============================================
	def has_selected_presentation_stack_roots(self, minimum: int = 1) -> bool:
		"""Return whether a complete durable selection can be reordered."""
		if type(minimum) is not int or minimum < 1:
			raise ValueError("presentation stack minimum must be a positive integer")
		if self._disposed or self.requires_refresh:
			return False
		try:
			return len(self._selected_presentation_root_selectors()) >= minimum
		except FerrumNativeDocumentTabError:
			return False

	#============================================
	def reorder_selected_presentation_roots(self, order: object) -> object:
		"""Submit one exact closed ordering operation through the Rust session."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(order) is not engine.DocumentPresentationStackOrderV1:
			raise TypeError("presentation stack order must be an exact frozen Rust value")
		minimum = (
			2
			if order == engine.DocumentPresentationStackOrderV1.reverse_selected_slots
			else 1
		)
		if not self.has_selected_presentation_stack_roots(minimum):
			raise RuntimeError("select a complete durable presentation root set first")
		targets = self._selected_presentation_root_selectors()
		result = self._session.apply_live_presentation_reorder_v1(
			self.current_snapshot.revision, self.current_snapshot.digest, order, targets,
		)
		self._install_mutation_result(result)
		return result


#============================================
def install_presentation_stack_actions(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> tuple[PySide6.QtGui.QAction, ...]:
	"""Install the three closed presentation ordering actions."""
	actions = []
	for label, tooltip, mode_name in (
		(
			"Bring Presentation to Front",
			"Move selected durable presentation roots to the front",
			"bring_to_front",
		),
		(
			"Send Presentation to Back",
			"Move selected durable presentation roots to the back",
			"send_to_back",
		),
		(
			"Reverse Presentation Slots",
			"Reverse at least two selected durable presentation slots",
			"reverse_selected_slots",
		),
	):
		action = PySide6.QtGui.QAction(window.tr(label), window)
		action.setToolTip(window.tr(tooltip))
		action.triggered.connect(
			lambda _checked=False, name=mode_name: _on_reorder_presentation(window, name)
		)
		edit_menu.addAction(action)
		actions.append(action)
	return tuple(actions)


#============================================
def refresh_presentation_stack_actions(actions: tuple[PySide6.QtGui.QAction, ...],
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Reflect exact one-target and two-target ordering eligibility."""
	if type(actions) is not tuple or len(actions) != 3:
		raise TypeError("presentation stack actions must be the installed action tuple")
	available = tab is not None and active and not pending and not busy
	actions[0].setEnabled(available and tab.has_selected_presentation_stack_roots())
	actions[1].setEnabled(available and tab.has_selected_presentation_stack_roots())
	actions[2].setEnabled(available and tab.has_selected_presentation_stack_roots(2))


#============================================
def _on_reorder_presentation(window: object, mode_name: str) -> None:
	"""Map one public action to an exact frozen Rust order with visible failure."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		import ferrum_qt.ferrum.engine as engine
		order = getattr(engine.DocumentPresentationStackOrderV1, mode_name)
		tab.reorder_selected_presentation_roots(order)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(
		window.tr("Reordered Ferrum presentation."), 5000,
	)
	window._refresh_actions()
