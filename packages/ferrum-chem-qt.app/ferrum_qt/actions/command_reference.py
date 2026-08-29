"""Modeless, nonmutating command help derived from the live Qt registry."""

# Standard Library
import collections.abc
import re

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.actions.command_catalog
import ferrum_qt.declarative_resources
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog


#============================================
def _normalized_search_text(value: str) -> str:
	"""Return case-folded search text with identifier punctuation normalized."""
	return " ".join(re.sub(r"[^\w]+", " ", value.casefold()).split())


#============================================
def matching_entries(
		query: str,
		entries: collections.abc.Iterable[
			ferrum_qt.actions.command_catalog.CommandCatalogEntry,
			],
		) -> tuple[ferrum_qt.actions.command_catalog.CommandCatalogEntry, ...]:
	"""Return matching catalog entries in the catalog's stable live order."""
	normalized_query = _normalized_search_text(query)
	if not normalized_query:
		return tuple(entries)
	return tuple(entry for entry in entries if normalized_query in _searchable_text(entry))


#============================================
def _searchable_text(
		entry: ferrum_qt.actions.command_catalog.CommandCatalogEntry,
		) -> str:
	"""Join discovery facts without creating a second command metadata source."""
	return _normalized_search_text(" ".join((
		entry.label,
		entry.help_text,
		entry.action_id,
		entry.shortcut or "",
		" ".join(entry.placement),
	)))


#============================================
class CommandReferenceDialog(FerrumAccessibleDialog):
	"""Present the current command catalog without command activation controls."""

	#============================================
	def __init__(self, controller: "CommandReferenceController") -> None:
		"""Build one keyboard-first help surface owned by its controller."""
		super().__init__(controller.parent)
		self._controller = controller
		self.setObjectName("command-reference-dialog")
		self.setWindowTitle(self.tr("Command Reference"))
		self.setModal(False)
		self.setWindowModality(PySide6.QtCore.Qt.WindowModality.NonModal)
		self.setMinimumWidth(620)
		self.setAccessibleName(self.tr("Command Reference"))
		self.setAccessibleDescription(self.tr(
			"Search current Ferrum commands. This reference does not run commands.",
		))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		query_label = PySide6.QtWidgets.QLabel(self.tr("Find a command"), self)
		layout.addWidget(query_label)
		self.search_field = PySide6.QtWidgets.QLineEdit(self)
		self.search_field.setObjectName("command-reference-search")
		self.search_field.setAccessibleName(self.tr("Find a command"))
		self.search_field.setAccessibleDescription(self.tr(
			"Filter command names, help text, shortcuts, and menu locations.",
		))
		self.search_field.setPlaceholderText(self.tr("Type a command name, shortcut, or help topic"))
		self.search_field.setClearButtonEnabled(True)
		query_label.setBuddy(self.search_field)
		layout.addWidget(self.search_field)
		self.result_list = PySide6.QtWidgets.QListWidget(self)
		self.result_list.setObjectName("command-reference-results")
		self.result_list.setAccessibleName(self.tr("Matching command reference entries"))
		self.result_list.setAccessibleDescription(self.tr(
			"Read command details. Selecting an entry does not run its command.",
		))
		self.result_list.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.SingleSelection,
		)
		layout.addWidget(self.result_list)
		self.status_label = PySide6.QtWidgets.QLabel(self)
		self.status_label.setObjectName("command-reference-status")
		self.status_label.setAccessibleName(self.tr("Command reference status"))
		self.status_label.setWordWrap(True)
		layout.addWidget(self.status_label)
		self.close_button = PySide6.QtWidgets.QPushButton(self.tr("Close"), self)
		self.close_button.setObjectName("command-reference-close")
		self.close_button.setAccessibleName(self.tr("Close command reference"))
		self.close_button.setToolTip(self.tr("Close Command Reference and return to your work"))
		self.close_button.clicked.connect(self.close)
		layout.addWidget(self.close_button, 0, PySide6.QtCore.Qt.AlignmentFlag.AlignRight)
		self.setTabOrder(self.search_field, self.result_list)
		self.setTabOrder(self.result_list, self.close_button)
		self.setProperty("ferrum_initial_focus_widget", self.search_field)
		self.search_field.textChanged.connect(self._controller.refresh)

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Restore the invoking authoring focus when this modeless help closes."""
		self._controller.restore_invoking_focus()
		super().closeEvent(event)


#============================================
class CommandReferenceController:
	"""Own live catalog refresh and focus lifecycle for Command Reference."""

	#============================================
	def __init__(
			self, parent: PySide6.QtWidgets.QWidget,
			registry: ferrum_qt.actions.action_registry.ActionRegistry,
			action_placements: collections.abc.Mapping[str, tuple[str, ...]] | None = None,
			) -> None:
		"""Create one modeless reference over the supplied live action owner."""
		self.parent = parent
		self._registry = registry
		self._action_placements = action_placements
		self._invoking_focus: PySide6.QtWidgets.QWidget | None = None
		self._entries: tuple[ferrum_qt.actions.command_catalog.CommandCatalogEntry, ...] = ()
		self.dialog = CommandReferenceDialog(self)

	#============================================
	def open(self) -> None:
		"""Show a fresh catalog and direct the keyboard to its filter."""
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
	@PySide6.QtCore.Slot(str)
	def refresh(self, _query: str = "") -> None:
		"""Refresh from current live actions after a query or reopen event."""
		if self._action_placements is None:
			self._action_placements = (
				ferrum_qt.declarative_resources.load_action_placement_projection(
					self._registry,
				)
			)
		catalog = ferrum_qt.actions.command_catalog.live_command_catalog(
			self._registry, self._action_placements,
		)
		self._entries = matching_entries(self.dialog.search_field.text(), catalog)
		self.dialog.result_list.clear()
		for entry in self._entries:
			self.dialog.result_list.addItem(_catalog_item(self.dialog, entry))
		if self._entries:
			self.dialog.result_list.setCurrentRow(0)
			self.dialog.status_label.setText(self.dialog.tr("{0} command(s) found.").format(
				len(self._entries),
			))
		else:
			self.dialog.status_label.setText(self.dialog.tr(
				"No commands match your search. Try a command name, shortcut, or menu location.",
			))

	#============================================
	def restore_invoking_focus(self) -> None:
		"""Return focus to the invoking authoring control after dismissal."""
		if self._invoking_focus is not None and self._invoking_focus.isVisible():
			self._invoking_focus.setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)
		self._invoking_focus = None


#============================================
def _catalog_item(
		dialog: CommandReferenceDialog,
		entry: ferrum_qt.actions.command_catalog.CommandCatalogEntry,
		) -> PySide6.QtWidgets.QListWidgetItem:
	"""Create one read-only, accessible list item from one catalog record."""
	location = " > ".join(entry.placement) if entry.placement else dialog.tr("Unplaced")
	shortcut = entry.shortcut or dialog.tr("No shortcut")
	availability = entry.availability_description
	text = dialog.tr("{0}\n{1}\nShortcut: {2} | Location: {3}\n{4}").format(
		entry.label, entry.help_text, shortcut, location, availability,
	)
	item = PySide6.QtWidgets.QListWidgetItem(text)
	item.setToolTip(entry.help_text)
	item.setData(PySide6.QtCore.Qt.ItemDataRole.AccessibleTextRole, text)
	item.setData(
		PySide6.QtCore.Qt.ItemDataRole.AccessibleDescriptionRole,
		dialog.tr("{0} {1}").format(entry.help_text, availability),
	)
	return item
