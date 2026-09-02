"""Validate and resolve Ferrum's YAML-authoritative ribbon layout."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui

# local repo modules
import ferrum_qt.declarative_resource_loader
import ferrum_qt.declarative_resources
import ferrum_qt.ribbon_contract


_RESOURCE_NAME = "ribbon_layout.yaml"
_ROLES = frozenset({"primary", "supporting"})
_PRIORITIES = frozenset({"required", "normal"})
_PRESENTATIONS = frozenset({"compact", "standard", "large"})
_ENTRY_KEYS = frozenset({"action", "role", "priority", "presentation"})
_OPTIONAL_ENTRY_KEYS = frozenset({"compact_label", "presentation_label"})
_COMPACT_LABEL_MAXIMUM = 7
_PRESENTATION_LABEL_MAXIMUM = 9
#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RibbonActionClient:
	"""One registry-owned action placed in the persistent ribbon header."""

	action_id: str
	action: PySide6.QtGui.QAction


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RibbonEntry:
	"""One live action placement within a labelled ribbon group."""

	action_id: str
	role: str
	priority: str
	presentation: str
	action: PySide6.QtGui.QAction
	compact_label: str | None = None
	presentation_label: str | None = None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RibbonGroupLayout:
	"""One user-recognizable task group and its resolved command clients."""

	id: str
	label_key: str
	overflow_label_key: str
	accent: str
	entries: tuple[RibbonEntry, ...]


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RibbonTabLayout:
	"""One task tab containing labelled command groups."""

	id: str
	label_key: str
	groups: tuple[RibbonGroupLayout, ...]


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RibbonLayout:
	"""Complete persistent header and task-tab command placement."""

	quick_access: tuple[RibbonActionClient, ...]
	global_actions: tuple[RibbonActionClient, ...]
	tabs: tuple[RibbonTabLayout, ...]

	#============================================
	def action_ids(self) -> frozenset[str]:
		"""Return every command requiring a canonical ribbon icon."""
		action_ids = {client.action_id for client in self.quick_access + self.global_actions}
		for tab in self.tabs:
			for group in tab.groups:
				action_ids.update(entry.action_id for entry in group.entries)
		return frozenset(action_ids)


#============================================
def load_ribbon_layout(registry: object) -> RibbonLayout:
	"""Load, validate, and resolve every ribbon QAction before UI construction."""
	data = ferrum_qt.declarative_resource_loader.load_packaged_yaml(_RESOURCE_NAME)
	return _resolve_layout(data, registry)


#============================================
def _resolve_layout(data: object, registry: object) -> RibbonLayout:
	"""Validate supplied YAML recursively and bind existing actions."""
	if type(data) is not dict or set(data) != {"global_actions", "quick_access", "tabs"}:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			"ribbon_layout.yaml must contain exactly global_actions, quick_access, and tabs.",
		)
	tabs = data["tabs"]
	if type(tabs) is not list or not tabs:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			"ribbon_layout.yaml.tabs must be a nonempty list.",
		)
	get_qt_action = getattr(registry, "get_qt_action", None)
	if not callable(get_qt_action):
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			"Ribbon layout needs an action registry with get_qt_action().",
		)
	seen_header_action_ids: set[str] = set()
	quick_access = _resolve_action_clients(
		data["quick_access"], "ribbon_layout.yaml.quick_access",
		seen_header_action_ids, get_qt_action,
	)
	global_actions = _resolve_action_clients(
		data["global_actions"], "ribbon_layout.yaml.global_actions",
		seen_header_action_ids, get_qt_action,
	)
	seen_tab_ids: set[str] = set()
	resolved_tabs = tuple(_resolve_tab(tab, index, seen_tab_ids, get_qt_action)
		for index, tab in enumerate(tabs))
	return RibbonLayout(quick_access, global_actions, resolved_tabs)


#============================================
def _resolve_action_clients(values: object, location: str,
		seen_action_ids: set[str], get_qt_action: object) -> tuple[RibbonActionClient, ...]:
	"""Resolve one nonempty ordered header action sequence."""
	if type(values) is not list or not values:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location} must be a nonempty list.",
		)
	clients: list[RibbonActionClient] = []
	for index, value in enumerate(values):
		action_location = f"{location}[{index}]"
		action_id = _string(value, action_location)
		if action_id in seen_action_ids:
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"Duplicate ribbon header action '{action_id}'.",
			)
		seen_action_ids.add(action_id)
		action = get_qt_action(action_id)
		if not isinstance(action, PySide6.QtGui.QAction):
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"{action_location} references unbound QAction '{action_id}'.",
			)
		clients.append(RibbonActionClient(action_id, action))
	return tuple(clients)


#============================================
def _resolve_tab(tab: object, index: int, seen_tab_ids: set[str],
		get_qt_action: object) -> RibbonTabLayout:
	"""Resolve one tab and reject malformed nested groups before widget mutation."""
	location = f"ribbon_layout.yaml.tabs[{index}]"
	data = _mapping(tab, location, {"id", "label_key", "groups"})
	tab_id = _string(data["id"], f"{location}.id")
	if tab_id in seen_tab_ids:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"Duplicate ribbon tab ID: '{tab_id}'.",
		)
	seen_tab_ids.add(tab_id)
	groups = data["groups"]
	if type(groups) is not list or not groups:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.groups must be a nonempty list.",
		)
	seen_group_ids: set[str] = set()
	seen_action_ids: set[str] = set()
	return RibbonTabLayout(tab_id, _string(data["label_key"], f"{location}.label_key"),
		tuple(_resolve_group(group, group_index, location, seen_group_ids, seen_action_ids, get_qt_action)
			for group_index, group in enumerate(groups)))


#============================================
def _resolve_group(group: object, index: int, tab_location: str,
		seen_group_ids: set[str], seen_action_ids: set[str], get_qt_action: object) -> RibbonGroupLayout:
	"""Resolve one group and preserve its placement order for keyboard traversal."""
	location = f"{tab_location}.groups[{index}]"
	data = _mapping(
		group, location, {"accent", "id", "label_key", "overflow_label_key", "entries"},
	)
	group_id = _string(data["id"], f"{location}.id")
	if group_id in seen_group_ids:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"Duplicate ribbon group ID: '{group_id}'.",
		)
	seen_group_ids.add(group_id)
	entries = data["entries"]
	if type(entries) is not list or not entries:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.entries must be a nonempty list.",
		)
	accent = _string(data["accent"], f"{location}.accent")
	if accent not in ferrum_qt.ribbon_contract.ACCENTS:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.accent must be one of: {', '.join(ferrum_qt.ribbon_contract.ACCENTS)}.",
		)
	return RibbonGroupLayout(group_id, _string(data["label_key"], f"{location}.label_key"),
		_string(data["overflow_label_key"], f"{location}.overflow_label_key"), accent,
		tuple(_resolve_entry(entry, entry_index, location, seen_action_ids, get_qt_action)
			for entry_index, entry in enumerate(entries)))


#============================================
def _resolve_entry(entry: object, index: int, group_location: str,
		seen_action_ids: set[str], get_qt_action: object) -> RibbonEntry:
	"""Resolve one declared registry action without constructing a replacement."""
	location = f"{group_location}.entries[{index}]"
	data = _entry_mapping(entry, location)
	action_id = _string(data["action"], f"{location}.action")
	if action_id in seen_action_ids:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"Duplicate ribbon action '{action_id}' in {group_location}.",
		)
	seen_action_ids.add(action_id)
	role = _string(data["role"], f"{location}.role")
	priority = _string(data["priority"], f"{location}.priority")
	if role not in _ROLES:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.role must be primary or supporting.",
		)
	if priority not in _PRIORITIES:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.priority must be required or normal.",
		)
	presentation = _string(data["presentation"], f"{location}.presentation")
	if presentation not in _PRESENTATIONS:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.presentation must be compact, standard, or large.",
		)
	compact_label = None
	if "compact_label" in data:
		compact_label = _string(data["compact_label"], f"{location}.compact_label")
		if presentation != "compact":
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"{location}.compact_label requires compact presentation.",
			)
		if len(compact_label) > _COMPACT_LABEL_MAXIMUM:
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"{location}.compact_label must contain at most {_COMPACT_LABEL_MAXIMUM} characters.",
			)
	presentation_label = None
	if "presentation_label" in data:
		presentation_label = _string(data["presentation_label"], f"{location}.presentation_label")
		if presentation == "compact":
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"{location}.presentation_label requires standard or large presentation.",
			)
		if len(presentation_label) > _PRESENTATION_LABEL_MAXIMUM:
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"{location}.presentation_label must contain at most {_PRESENTATION_LABEL_MAXIMUM} characters.",
			)
	action = get_qt_action(action_id)
	if not isinstance(action, PySide6.QtGui.QAction):
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location}.action references unbound QAction '{action_id}'.",
		)
	return RibbonEntry(
		action_id, role, priority, presentation, action, compact_label, presentation_label,
	)


#============================================
def _entry_mapping(value: object, location: str) -> dict:
	"""Require a ribbon entry with its one optional compact caption."""
	if type(value) is not dict:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location} must contain action, role, priority, and presentation.",
		)
	keys = set(value)
	if not _ENTRY_KEYS <= keys or not keys <= _ENTRY_KEYS | _OPTIONAL_ENTRY_KEYS:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location} must contain action, role, priority, and presentation, with optional labels.",
		)
	return value


#============================================
def _mapping(value: object, location: str, keys: set[str]) -> dict:
	"""Require an exact YAML mapping with a useful location in failure text."""
	if type(value) is not dict or set(value) != keys:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location} must contain exactly: {', '.join(sorted(keys))}.",
		)
	return value


#============================================
def _string(value: object, location: str) -> str:
	"""Require one nonempty declaration string."""
	if type(value) is not str or not value:
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"{location} must be a nonempty string.",
		)
	return value
