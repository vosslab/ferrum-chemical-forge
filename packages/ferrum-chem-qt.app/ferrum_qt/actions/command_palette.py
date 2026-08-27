"""Keyboard-first client for Ferrum's live action registry."""

# Standard Library
import collections.abc
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.declarative_resources
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog


#============================================
def _normalized_search_text(value: str) -> str:
	"""Return case-folded search text with identifier punctuation normalized."""
	return " ".join(re.sub(r"[^\w]+", " ", value.casefold()).split())


#============================================
def _is_subsequence(query: str, candidate: str) -> bool:
	"""Return whether every query character occurs in candidate order."""
	remaining = iter(candidate)
	return all(character in remaining for character in query)


#============================================
def _relevance_tier(
		query: str, view: ferrum_qt.actions.action_registry.LiveActionView,
		) -> int | None:
	"""Classify one matching view by user-visible query relevance."""
	normalized_query = _normalized_search_text(query)
	if not normalized_query:
		return 0
	label_and_id = tuple(_normalized_search_text(value)
		for value in (view.label, view.action_id))
	if any(normalized_query == value for value in label_and_id):
		return 0
	if any(normalized_query in value.split() for value in label_and_id):
		return 0
	if any(any(word.startswith(normalized_query) for word in value.split())
			for value in label_and_id):
		return 1
	if any(normalized_query in value for value in label_and_id):
		return 2
	if normalized_query in _normalized_search_text(view.help_text):
		return 3
	compact_query = normalized_query.replace(" ", "")
	for value in (view.label, view.help_text, view.action_id):
		normalized_value = _normalized_search_text(value)
		if _is_subsequence(compact_query, normalized_value.replace(" ", "")):
			return 4
	return None


#============================================
def ranked_matching_views(
		query: str, views: collections.abc.Iterable[
			ferrum_qt.actions.action_registry.LiveActionView,
		],
		) -> tuple[ferrum_qt.actions.action_registry.LiveActionView, ...]:
	"""Rank matching views without disturbing their registry-provided tie order."""
	ranked = tuple(
		(tier, view)
		for view in views
		if (tier := _relevance_tier(query, view)) is not None
	)
	return tuple(view for _tier, view in sorted(
		ranked, key=lambda item: (item[0], not item[1].enabled),
	))


#============================================
class _CommandPaletteSearchField(PySide6.QtWidgets.QLineEdit):
	"""Keep query editing in the field while forwarding result navigation."""

	#============================================
	def __init__(
			self, move_result_selection: collections.abc.Callable[[int], None],
			parent: PySide6.QtWidgets.QWidget,
			) -> None:
		"""Create the query field with its one keyboard-navigation boundary."""
		super().__init__(parent)
		self._move_result_selection = move_result_selection

	#============================================
	def keyPressEvent(self, event: PySide6.QtGui.QKeyEvent) -> None:
		"""Move result selection for arrows and edit query text for other keys."""
		key_steps = {
			PySide6.QtCore.Qt.Key.Key_Up: -1,
			PySide6.QtCore.Qt.Key.Key_Down: 1,
		}
		step = key_steps.get(event.key())
		if step is not None and event.modifiers() == PySide6.QtCore.Qt.KeyboardModifier.NoModifier:
			self._move_result_selection(step)
			event.accept()
			return
		super().keyPressEvent(event)


#============================================
class CommandPaletteDialog(FerrumAccessibleDialog):
	"""Present one non-modal, keyboard-first action search surface."""

	#============================================
	def __init__(self, controller: "CommandPaletteController") -> None:
		"""Build the palette presentation without duplicating command metadata."""
		super().__init__(controller.parent)
		self._controller = controller
		self.setObjectName("command-palette-dialog")
		self.setWindowTitle(self.tr("Command Palette"))
		self.setModal(False)
		self.setWindowModality(PySide6.QtCore.Qt.WindowModality.NonModal)
		self.setMinimumWidth(520)
		self.setAccessibleName(self.tr("Command Palette"))
		self.setAccessibleDescription(self.tr(
			"Search Ferrum commands. Use the arrow keys to select a command, "
			"Enter to run an available command, or Escape to close.",
		))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		query_label = PySide6.QtWidgets.QLabel(self.tr("Find a command"), self)
		query_label.setAccessibleName(self.tr("Command search label"))
		layout.addWidget(query_label)
		self.search_field = _CommandPaletteSearchField(
			controller.move_result_selection, self,
		)
		self.search_field.setObjectName("command-palette-search")
		self.search_field.setAccessibleName(self.tr("Find a command"))
		self.search_field.setPlaceholderText(self.tr("Type a command name or help topic"))
		self.search_field.setClearButtonEnabled(True)
		query_label.setBuddy(self.search_field)
		layout.addWidget(self.search_field)
		self.result_list = PySide6.QtWidgets.QListWidget(self)
		self.result_list.setObjectName("command-palette-results")
		self.result_list.setAccessibleName(self.tr("Matching commands"))
		self.result_list.setAccessibleDescription(self.tr(
			"Available commands can be run with Enter. Unavailable commands remain "
			"listed and are labelled unavailable.",
		))
		self.result_list.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.SingleSelection,
		)
		layout.addWidget(self.result_list)
		self.status_label = PySide6.QtWidgets.QLabel(self)
		self.status_label.setObjectName("command-palette-status")
		self.status_label.setAccessibleName(self.tr("Command palette status"))
		self.status_label.setWordWrap(True)
		layout.addWidget(self.status_label)
		self.setProperty("ferrum_initial_focus_widget", self.search_field)
		self.search_field.textChanged.connect(self._on_query_changed)
		self.search_field.returnPressed.connect(self._controller.activate_selected)
		self.result_list.itemActivated.connect(
			lambda _item: self._controller.activate_selected(),
		)
		self.result_list.currentRowChanged.connect(self._controller.describe_current)

	#============================================
	@PySide6.QtCore.Slot(str)
	def _on_query_changed(self, _text: str) -> None:
		"""Refresh from the field-owned text after each user query edit."""
		self._controller.refresh()

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Return focus cleanly when a user dismisses the modeless palette."""
		self._controller.restore_invoking_focus()
		super().closeEvent(event)


#============================================
class CommandPaletteController:
	"""Own transient command search, selection, invocation, and focus cleanup."""

	#============================================
	def __init__(
			self, parent: PySide6.QtWidgets.QWidget,
			registry: ferrum_qt.actions.action_registry.ActionRegistry,
			action_placements: collections.abc.Mapping[str, tuple[str, ...]] | None = None,
			) -> None:
		"""Create one reusable palette client for the supplied live registry."""
		self.parent = parent
		self._registry = registry
		self._invoking_focus: PySide6.QtWidgets.QWidget | None = None
		self._results: tuple[ferrum_qt.actions.action_registry.LiveActionView, ...] = ()
		self._action_placements = action_placements
		self.dialog = CommandPaletteDialog(self)

	#============================================
	def open(self) -> None:
		"""Show the modeless palette and direct the keyboard to its search field."""
		if self.dialog.isVisible():
			self.dialog.activateWindow()
			self.dialog.search_field.setFocus(
				PySide6.QtCore.Qt.FocusReason.ShortcutFocusReason,
			)
			return
		focus_widget = self.parent.focusWidget()
		self._invoking_focus = focus_widget if focus_widget is not None else self.parent
		self.dialog.search_field.clear()
		self.refresh()
		self.dialog.show()
		self.dialog.raise_()
		self.dialog.activateWindow()
		self.dialog.search_field.setFocus(PySide6.QtCore.Qt.FocusReason.ShortcutFocusReason)

	#============================================
	def refresh(self, _query: str | None = None) -> None:
		"""Refresh visible results from the one current registry projection."""
		del _query
		query = self.dialog.search_field.text()
		if self._action_placements is None:
			self._action_placements = (
				ferrum_qt.declarative_resources.load_action_placement_projection(
					self._registry,
				)
			)
		self._results = ranked_matching_views(query, self._registry.live_action_views())
		self.dialog.result_list.clear()
		for view in self._results:
			breadcrumb = self._action_placements.get(view.action_id, ())
			text = view.label
			if breadcrumb:
				text = self.dialog.tr("{0} - {1}").format(text, " > ".join(breadcrumb))
			if not view.enabled:
				text = self.dialog.tr("{0} - Unavailable").format(text)
			item = PySide6.QtWidgets.QListWidgetItem(text)
			item.setToolTip(view.help_text)
			item.setData(
				PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole,
				self.dialog.tr("{0}. {1}").format(text, view.help_text),
			)
			if not view.enabled:
				item.setFlags(PySide6.QtCore.Qt.ItemFlag.NoItemFlags)
				item.setData(
					PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole,
					self.dialog.tr("This command is currently unavailable."),
				)
			self.dialog.result_list.addItem(item)
		if self._results:
			self.dialog.result_list.setCurrentRow(0)
			self.describe_current(0)
		else:
			self.dialog.status_label.setText(
				self.dialog.tr("No commands match your search."),
			)

	#============================================
	def describe_current(self, row: int) -> None:
		"""Explain the selected command's current live availability."""
		if row < 0 or row >= len(self._results):
			return
		view = self._results[row]
		if view.enabled:
			self.dialog.status_label.setText(view.help_text)
		else:
			self.dialog.status_label.setText(
				self.dialog.tr("{0} is currently unavailable.").format(view.label),
			)

	#============================================
	def move_result_selection(self, step: int) -> None:
		"""Move the selected result one row while query focus remains in place."""
		count = self.dialog.result_list.count()
		if count == 0:
			return
		current_row = self.dialog.result_list.currentRow()
		if current_row < 0:
			current_row = 0
		row = max(0, min(count - 1, current_row + step))
		self.dialog.result_list.setCurrentRow(row)

	#============================================
	def activate_selected(self) -> None:
		"""Close and invoke one currently enabled registered QAction exactly once."""
		row = self.dialog.result_list.currentRow()
		if row < 0 or row >= len(self._results):
			return
		view = self._results[row]
		if not view.qt_action.isEnabled():
			self.refresh()
			self.dialog.status_label.setText(
				self.dialog.tr("{0} is currently unavailable.").format(view.label),
			)
			return
		self.dialog.close()
		view.qt_action.trigger()

	#============================================
	def restore_invoking_focus(self) -> None:
		"""Return focus to the invoking workflow before any action receives control."""
		if self._invoking_focus is not None and self._invoking_focus.isVisible():
			self._invoking_focus.setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)
		self._invoking_focus = None
