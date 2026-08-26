"""Feature-owned lifecycle and native input adapter for Add Atom."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.modes.base_mode
import ferrum_qt.modes.controllers


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class AtomInsertionIntent:
	"""One revision-bound atom placement awaiting a normalized input intent."""

	tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	molecule_object_id: str
	element: str


#============================================
class FerrumAtomModeFeature:
	"""Own Add Atom lifecycle after the shared native input adapter normalizes input."""

	def __init__(self, window: object) -> None:
		"""Bind the feature to one native Ferrum window without owning its document."""
		self._window = window
		self._intent: AtomInsertionIntent | None = None
		self.controller = ferrum_qt.modes.controllers.AtomMode()

	def activate(self, context: ferrum_qt.modes.base_mode.ModeContext) -> bool:
		"""Freeze one authorable molecule and install the feature input boundary."""
		del context
		window = self._window
		window._cancel_line_gesture()
		tab = window._active_native_tab()
		if tab is None:
			window._show_edit_refusal(window._typed_refusal(
				"edit_document", "unavailable_operation",
				"Open a Ferrum drawing before placing an atom.",
			))
			return False
		choices = tab.canvas_authorable_molecule_choices()
		if not choices:
			window._show_edit_refusal(window._typed_refusal(
				"edit_document", "unrenderable_molecule",
				"The installed Rust render observation has no canvas-authorable molecule plan.",
			))
			return False
		drawing = window._drawing_parameters.snapshot()
		choice = choices[0]
		if len(choices) > 1:
			labels = tuple(item.label for item in choices)
			selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
				window, window.tr("Choose Molecule"), window.tr("Target molecule:"),
				labels, 0, False,
			)
			if not accepted:
				return False
			choice = choices[labels.index(selected)]
		snapshot = tab.current_snapshot
		intent = AtomInsertionIntent(
			tab, tab.view.viewport(), snapshot.revision, snapshot.digest,
			choice.object_id, drawing.element,
		)
		self._intent = intent
		window._atom_insertion_intent = intent
		window._add_atom_action.setToolTip(window.tr(
			"Add {0} at the next canvas click; Escape cancels.",
		).format(drawing.element))
		window._refresh_cancel_tool_action()
		intent.viewport.setFocus()
		tab.view.show_keyboard_cursor()
		window.statusBar().showMessage(window.tr(
			"Click once or use Arrow keys and Enter to add {0}; Shift+Arrow is fine; "
			"Esc cancels Add Atom.",
		).format(drawing.element), 5000)
		return True

	def dispatch(self, context: ferrum_qt.modes.base_mode.ModeContext,
			intent: ferrum_qt.modes.base_mode.ModeIntent) -> None:
		"""Submit only the AtomMode semantic placement operation."""
		del context
		if intent.operation_id != "atom.place":
			raise RuntimeError("Add Atom received an unrelated mode intent.")
		placement = self._intent
		if placement is None:
			return
		if len(intent.points) == 0:
			point = placement.tab.view.show_keyboard_cursor()
			x, y = float(point.x()), float(point.y())
		elif len(intent.points) == 1:
			point = intent.points[0]
			x, y = point.x, point.y
		else:
			raise RuntimeError("Add Atom accepts exactly one scene point.")
		self._place(placement, x, y)

	def cancel(self, context: ferrum_qt.modes.base_mode.ModeContext,
			*, clear_status: bool = True) -> None:
		"""Release atom input ownership without changing the Rust document."""
		del context
		intent = self._intent
		self._intent = None
		window = self._window
		window._atom_insertion_intent = None
		if intent is not None:
			intent.tab.view.hide_keyboard_cursor()
		if clear_status:
			window.statusBar().clearMessage()
		window._refresh_cancel_tool_action()

	def _place(self, intent: AtomInsertionIntent, x: float, y: float) -> None:
		"""Apply one current, normalized atom point through the authoritative tab."""
		window = self._window
		tab = intent.tab
		snapshot = tab.current_snapshot
		if (
			window._active_native_tab() is not tab
			or tab.requires_refresh
			or snapshot.revision != intent.revision
			or snapshot.digest != intent.digest
		):
			window._window_mode_sync.cancel()
			window._show_edit_refusal(window._typed_refusal(
				"use_tool", "stale_tool",
				"The document changed before the click; start Add Atom again.",
			))
			return
		try:
			tab.add_atom_at(intent.molecule_object_id, intent.element, x, y)
		except Exception as exc:
			window._window_mode_sync.cancel()
			window._refresh_actions()
			window.statusBar().clearMessage()
			self._show_refusal(exc)
			return
		window._window_mode_sync.cancel()
		window.statusBar().showMessage(window.tr("Added one free-standing Rust atom."), 5000)
		window._refresh_actions()

	def _show_refusal(self, exc: Exception) -> None:
		"""Present canvas-plan failures identically for pointer and keyboard input."""
		outcome = "unrenderable_molecule" if isinstance(
			exc,
			ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabUnrenderableMoleculeError,
		) else "unavailable_operation"
		self._window._show_edit_refusal(self._window._typed_refusal(
			"edit_document", outcome, str(exc),
		))
