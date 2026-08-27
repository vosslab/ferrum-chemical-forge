"""Load and validate Ferrum-owned menu declarations."""

# Standard Library
import collections.abc

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.declarative_resource_loader


_MENU_RESOURCE = "menus.yaml"
_CONTEXT_MENU_ID = "selected_structure"


#============================================
DeclarativeResourceError = ferrum_qt.declarative_resource_loader.DeclarativeResourceError


#============================================
def load_menu_declarations() -> dict:
	"""Load the packaged static menu declaration mapping."""
	data = ferrum_qt.declarative_resource_loader.load_packaged_yaml(_MENU_RESOURCE)
	if type(data) is not dict:
		raise DeclarativeResourceError("Ferrum resource 'menus.yaml' must contain a YAML mapping.")
	return data


#============================================
def _require_mapping(value: object, location: str) -> dict:
	"""Return a mapping or raise a resource-specific validation error."""
	if type(value) is not dict:
		raise DeclarativeResourceError(f"{location} must be a mapping.")
	return value


#============================================
def _require_string(value: object, location: str) -> str:
	"""Return a nonempty string or raise a resource-specific validation error."""
	if type(value) is not str or not value:
		raise DeclarativeResourceError(f"{location} must be a nonempty string.")
	return value


#============================================
def _require_keys(value: dict, required: set[str], location: str) -> None:
	"""Require one declaration mapping to use exactly its schema keys."""
	if set(value) != required:
		raise DeclarativeResourceError(
			f"{location} must contain exactly: {', '.join(sorted(required))}.",
		)


#============================================
def _validate_menu_items(
		items: object, location: str, action_ids: frozenset[str],
		dynamic_menu_ids: frozenset[str], node_ids: set[str],
		declared_action_ids: set[str], declared_dynamic_menu_ids: set[str],
		) -> None:
	"""Validate one ordered menu-item sequence recursively."""
	if type(items) is not list or not items:
		raise DeclarativeResourceError(f"{location} must be a nonempty list.")
	for index, item in enumerate(items):
		item_location = f"{location}[{index}]"
		item = _require_mapping(item, item_location)
		if set(item) == {"action"}:
			action_id = _require_string(item["action"], f"{item_location}.action")
			if action_id in declared_action_ids:
				raise DeclarativeResourceError(
					f"Duplicate declared menu action ID: '{action_id}'.",
				)
			if action_id not in action_ids:
				raise DeclarativeResourceError(
					f"{item_location} references unresolved action '{action_id}'.",
				)
			declared_action_ids.add(action_id)
			continue
		if set(item) == {"separator"}:
			if item["separator"] is not True:
				raise DeclarativeResourceError(f"{item_location}.separator must be true.")
			continue
		if set(item) == {"dynamic_menu"}:
			menu_id = _require_string(item["dynamic_menu"], f"{item_location}.dynamic_menu")
			if menu_id in declared_dynamic_menu_ids:
				raise DeclarativeResourceError(
					f"Duplicate declared dynamic menu ID: '{menu_id}'.",
				)
			if menu_id not in dynamic_menu_ids:
				raise DeclarativeResourceError(
					f"{item_location} references unresolved dynamic menu '{menu_id}'.",
				)
			declared_dynamic_menu_ids.add(menu_id)
			continue
		if set(item) not in ({"section"}, {"submenu"}):
			raise DeclarativeResourceError(
				f"{item_location} must declare exactly one node form.",
			)
		node_kind = "section" if "section" in item else "submenu"
		node = _require_mapping(item[node_kind], f"{item_location}.{node_kind}")
		if node_kind == "submenu":
			_require_keys(
				node, {"id", "label_key", "help_key", "items"},
				f"{item_location}.submenu",
			)
		else:
			if set(node) not in ({"id", "items"}, {"id", "label_key", "items"}):
				raise DeclarativeResourceError(
					f"{item_location}.section must contain id, items, and optional label_key.",
				)
		node_id = _require_string(node["id"], f"{item_location}.{node_kind}.id")
		if node_id in node_ids:
			raise DeclarativeResourceError(f"Duplicate menu node ID: '{node_id}'.")
		node_ids.add(node_id)
		if node_kind == "section" and "label_key" in node:
			_require_string(node["label_key"], f"{item_location}.section.label_key")
		if node_kind == "submenu":
			_require_string(node["label_key"], f"{item_location}.submenu.label_key")
			_require_string(node["help_key"], f"{item_location}.submenu.help_key")
		_validate_menu_items(
			node["items"], f"{item_location}.{node_kind}.items", action_ids,
			dynamic_menu_ids, node_ids, declared_action_ids,
			declared_dynamic_menu_ids,
		)


#============================================
def _validate_menu_declarations(
		data: dict, action_ids: frozenset[str],
		dynamic_menu_ids: frozenset[str] = frozenset(),
		) -> None:
	"""Require one complete, resolvable, recursively declared menu tree."""
	_require_keys(data, {"menus", "contexts"}, "menus.yaml")
	menus = data["menus"]
	if type(menus) is not list or not menus:
		raise DeclarativeResourceError("menus.yaml 'menus' must be a nonempty list.")
	node_ids: set[str] = set()
	declared_action_ids: set[str] = set()
	declared_dynamic_menu_ids: set[str] = set()
	for index, menu in enumerate(menus):
		location = f"menus[{index}]"
		menu = _require_mapping(menu, location)
		_require_keys(menu, {"id", "label_key", "help_key", "items"}, location)
		menu_id = _require_string(menu["id"], f"{location}.id")
		if menu_id in node_ids:
			raise DeclarativeResourceError(f"Duplicate menu node ID: '{menu_id}'.")
		node_ids.add(menu_id)
		_require_string(menu["label_key"], f"{location}.label_key")
		_require_string(menu["help_key"], f"{location}.help_key")
		_validate_menu_items(
			menu["items"], f"{location}.items", action_ids, dynamic_menu_ids,
			node_ids, declared_action_ids, declared_dynamic_menu_ids,
		)
	_validate_context_declarations(data, action_ids)


#============================================
def _validate_context_declarations(data: dict, action_ids: frozenset[str]) -> None:
	"""Require each context placement to use registered, unique action IDs."""
	contexts = data["contexts"]
	if type(contexts) is not list or not contexts:
		raise DeclarativeResourceError("menus.yaml 'contexts' must be a nonempty list.")
	context_ids: set[str] = set()
	for index, context in enumerate(contexts):
		location = f"contexts[{index}]"
		context = _require_mapping(context, location)
		_require_keys(context, {"id", "accessible_name", "groups"}, location)
		context_id = _require_string(context["id"], f"{location}.id")
		if context_id in context_ids:
			raise DeclarativeResourceError(f"Duplicate context menu ID: '{context_id}'.")
		context_ids.add(context_id)
		_require_string(context["accessible_name"], f"{location}.accessible_name")
		groups = context["groups"]
		if type(groups) is not list or not groups:
			raise DeclarativeResourceError(f"{location}.groups must be a nonempty list.")
		group_ids: set[str] = set()
		declared_action_ids: set[str] = set()
		for group_index, group in enumerate(groups):
			group_location = f"{location}.groups[{group_index}]"
			group = _require_mapping(group, group_location)
			_require_keys(group, {"id", "actions"}, group_location)
			group_id = _require_string(group["id"], f"{group_location}.id")
			if group_id in group_ids:
				raise DeclarativeResourceError(f"Duplicate context group ID: '{group_id}'.")
			group_ids.add(group_id)
			actions = group["actions"]
			if type(actions) is not list or not actions:
				raise DeclarativeResourceError(f"{group_location}.actions must be a nonempty list.")
			for action_index, action_id in enumerate(actions):
				action_location = f"{group_location}.actions[{action_index}]"
				action_id = _require_string(action_id, action_location)
				if action_id in declared_action_ids:
					raise DeclarativeResourceError(
						f"Duplicate declared context action ID: '{action_id}'.",
					)
				if action_id not in action_ids:
					raise DeclarativeResourceError(
						f"{action_location} references unresolved action '{action_id}'.",
					)
				declared_action_ids.add(action_id)


#============================================
def _registry_action_ids(registry: object) -> frozenset[str]:
	"""Return declared action IDs from a registry or plain collection."""
	all_actions = getattr(registry, "all_actions", None)
	if callable(all_actions):
		return frozenset(all_actions())
	if isinstance(registry, collections.abc.Collection):
		if not all(type(action_id) is str for action_id in registry):
			raise DeclarativeResourceError("Action ID collections must contain strings.")
		return frozenset(registry)
	raise DeclarativeResourceError(
		"Resource preflight needs an action registry or a collection of action IDs.",
	)


#============================================
def _registry_dynamic_menu_ids(registry: object) -> frozenset[str]:
	"""Return state-derived menu IDs from the active action registry."""
	dynamic_menu_ids = getattr(registry, "dynamic_menu_ids", None)
	if callable(dynamic_menu_ids):
		return frozenset(dynamic_menu_ids())
	return frozenset()


#============================================
def _require_registry_method(registry: object, name: str) -> collections.abc.Callable:
	"""Return one live-client lookup method required for menu assembly."""
	method = getattr(registry, name, None)
	if not callable(method):
		raise DeclarativeResourceError(
			"Menu assembly needs an action registry with live Qt client bindings.",
		)
	return method


#============================================
def _validate_live_menu_clients(data: dict, registry: object) -> None:
	"""Require every validated declaration reference to have its Qt client now."""
	get_qt_action = _require_registry_method(registry, "get_qt_action")
	get_dynamic_menu = _require_registry_method(registry, "get_dynamic_menu")

	def validate_items(items: list, location: str) -> None:
		"""Validate all Qt clients in one already schema-validated item sequence."""
		for index, item in enumerate(items):
			item_location = f"{location}[{index}]"
			if "action" in item:
				action_id = item["action"]
				if not isinstance(get_qt_action(action_id), PySide6.QtGui.QAction):
					raise DeclarativeResourceError(
						f"{item_location} action '{action_id}' has no bound QAction.",
					)
			elif "dynamic_menu" in item:
				menu_id = item["dynamic_menu"]
				if not isinstance(get_dynamic_menu(menu_id), PySide6.QtWidgets.QMenu):
					raise DeclarativeResourceError(
						f"{item_location} dynamic menu '{menu_id}' has no bound QMenu.",
					)
			elif "section" in item:
				validate_items(item["section"]["items"], f"{item_location}.section.items")
			elif "submenu" in item:
				validate_items(item["submenu"]["items"], f"{item_location}.submenu.items")

	for index, menu in enumerate(data["menus"]):
		validate_items(menu["items"], f"menus[{index}].items")


#============================================
def load_context_menu_placement(registry: object,
		context_id: str = _CONTEXT_MENU_ID) -> tuple[str, tuple[tuple[str, ...], ...]]:
	"""Return one validated context-menu name and ordered action groups."""
	action_ids = _registry_action_ids(registry)
	data = load_menu_declarations()
	_validate_context_declarations(data, action_ids)
	get_qt_action = _require_registry_method(registry, "get_qt_action")
	for context in data["contexts"]:
		if context["id"] != context_id:
			continue
		groups = tuple(tuple(group["actions"]) for group in context["groups"])
		for action_ids in groups:
			for action_id in action_ids:
				if not isinstance(get_qt_action(action_id), PySide6.QtGui.QAction):
					raise DeclarativeResourceError(
						f"Context action '{action_id}' has no bound QAction.",
					)
		return context["accessible_name"], groups
	raise DeclarativeResourceError(f"Unknown Ferrum context menu ID: '{context_id}'.")


#============================================
def preflight_menu_declarations(registry: object) -> None:
	"""Validate only the YAML-owned menu tree against its live registry clients."""
	action_ids = _registry_action_ids(registry)
	dynamic_menu_ids = _registry_dynamic_menu_ids(registry)
	data = load_menu_declarations()
	_validate_menu_declarations(data, action_ids, dynamic_menu_ids)
	_validate_live_menu_clients(data, registry)
