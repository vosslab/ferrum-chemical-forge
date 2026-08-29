"""Immutable Qt command-reference facts derived from live Ferrum owners."""

# Standard Library
import collections.abc
import dataclasses

# PIP3 modules
import PySide6.QtGui

# local repo modules
import ferrum_qt.actions.action_registry


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class CommandCatalogEntry:
	"""One current, nonmutating presentation record for a live Ferrum command."""

	action_id: str
	label: str
	help_text: str
	shortcut: str | None
	placement: tuple[str, ...]
	enabled: bool
	qt_action: PySide6.QtGui.QAction

	@property
	def availability_description(self) -> str:
		"""Explain the action's current state without changing that state."""
		if self.enabled:
			return "This command is currently available."
		return "This command is currently unavailable."


#============================================
def live_command_catalog(
		registry: ferrum_qt.actions.action_registry.ActionRegistry,
		placements: collections.abc.Mapping[str, tuple[str, ...]],
		) -> tuple[CommandCatalogEntry, ...]:
	"""Project current registry clients and validated YAML placement exactly once."""
	return tuple(
		CommandCatalogEntry(
			action_id=view.action_id,
			label=view.label,
			help_text=view.help_text,
			shortcut=_native_shortcut_text(view.qt_action),
			placement=tuple(placements.get(view.action_id, ())),
			enabled=view.enabled,
			qt_action=view.qt_action,
		)
		for view in registry.live_action_views()
	)


#============================================
def _native_shortcut_text(action: PySide6.QtGui.QAction) -> str | None:
	"""Return one current user-facing shortcut, omitting an absent sequence."""
	shortcut = action.shortcut().toString(
		PySide6.QtGui.QKeySequence.SequenceFormat.NativeText,
	)
	return shortcut or None
