"""Load and validate Ferrum-owned menu and drawing-tool declarations."""

# Standard Library
import collections.abc

# PIP3 modules
import yaml

# local repo modules
import ferrum_qt.resource_paths


_MENU_RESOURCE = "menus.yaml"
_MODE_RESOURCE = "modes.yaml"
_SEPARATOR = "---"

# These are the tool commands that the current Ferrum window exposes.  Future
# mode-controller work must extend this vocabulary before declaring a new mode.
SUPPORTED_TOOL_ACTION_IDS = frozenset({
	"mode.atom", "mode.draw", "tool.cancel",
})


#============================================
class DeclarativeResourceError(ValueError):
	"""Report invalid Ferrum declarative UI resource data."""


#============================================
def _load_yaml_resource(filename: str) -> dict:
	"""Return one packaged YAML mapping, rejecting non-mapping documents."""
	path = ferrum_qt.resource_paths.get_resource_path(filename)
	with open(path, "r", encoding="utf-8") as fh:
		data = yaml.safe_load(fh)
	if type(data) is not dict:
		raise DeclarativeResourceError(
			f"Ferrum resource '{path.name}' must contain a YAML mapping.",
		)
	return data


#============================================
def load_menu_declarations() -> dict:
	"""Load the packaged static menu declaration mapping."""
	return _load_yaml_resource(_MENU_RESOURCE)


#============================================
def load_mode_declarations() -> dict:
	"""Load the packaged drawing-tool declaration mapping."""
	return _load_yaml_resource(_MODE_RESOURCE)


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
def _is_optional(item: dict) -> bool:
	"""Return whether an action declaration may be absent from this release."""
	optional = item.get("optional", False)
	if type(optional) is not bool:
		raise DeclarativeResourceError("Declaration 'optional' must be boolean.")
	return optional


#============================================
def _validate_menu_declarations(data: dict, action_ids: frozenset[str]) -> None:
	"""Require unique menu names and resolvable nonoptional action references."""
	menus = data.get("menus")
	if type(menus) is not list or not menus:
		raise DeclarativeResourceError("menus.yaml 'menus' must be a nonempty list.")
	menu_names: set[str] = set()
	declared_action_ids: set[str] = set()
	for index, menu in enumerate(menus):
		location = f"menus[{index}]"
		menu = _require_mapping(menu, location)
		name = _require_string(menu.get("name"), f"{location}.name")
		if name in menu_names:
			raise DeclarativeResourceError(f"Duplicate menu ID: '{name}'.")
		menu_names.add(name)
		_require_string(menu.get("label_key"), f"{location}.label_key")
		_require_string(menu.get("help_key"), f"{location}.help_key")
		if menu.get("side") not in {"left", "right"}:
			raise DeclarativeResourceError(f"{location}.side must be 'left' or 'right'.")
		items = menu.get("items")
		if type(items) is not list or not items:
			raise DeclarativeResourceError(f"{location}.items must be a nonempty list.")
		for item_index, item in enumerate(items):
			item_location = f"{location}.items[{item_index}]"
			item = _require_mapping(item, item_location)
			if item.get("separator") is True:
				if set(item) != {"separator"}:
					raise DeclarativeResourceError(
					f"{item_location} separator declarations cannot have other keys.",
				)
				continue
			action_id = _require_string(item.get("action"), f"{item_location}.action")
			if action_id in declared_action_ids:
				raise DeclarativeResourceError(
					f"Duplicate declared menu action ID: '{action_id}'.",
				)
			declared_action_ids.add(action_id)
			if action_id not in action_ids and not _is_optional(item):
				raise DeclarativeResourceError(
					f"{item_location} references unregistered action '{action_id}'.",
				)


#============================================
def _validate_mode_declarations(data: dict, action_ids: frozenset[str]) -> None:
	"""Require unique toolbar mode IDs and supported tool-action references."""
	modes = _require_mapping(data.get("modes"), "modes.yaml 'modes'")
	toolbar_order = data.get("toolbar_order")
	if type(toolbar_order) is not list or not toolbar_order:
		raise DeclarativeResourceError(
			"modes.yaml 'toolbar_order' must be a nonempty list.",
		)
	seen_mode_ids: set[str] = set()
	for index, mode_id in enumerate(toolbar_order):
		if mode_id == _SEPARATOR:
			continue
		mode_id = _require_string(mode_id, f"toolbar_order[{index}]")
		if mode_id in seen_mode_ids:
			raise DeclarativeResourceError(f"Duplicate toolbar mode ID: '{mode_id}'.")
		if mode_id not in modes:
			raise DeclarativeResourceError(
				f"toolbar_order references unknown mode '{mode_id}'.",
			)
		seen_mode_ids.add(mode_id)
	if set(modes) != seen_mode_ids:
		unplaced = sorted(set(modes) - seen_mode_ids)
		raise DeclarativeResourceError(
			f"Mode declarations missing from toolbar_order: {', '.join(unplaced)}.",
		)
	for mode_id, declaration in modes.items():
		location = f"modes.{mode_id}"
		declaration = _require_mapping(declaration, location)
		_require_string(declaration.get("label_key"), f"{location}.label_key")
		_require_string(declaration.get("help_key"), f"{location}.help_key")
		action_id = _require_string(declaration.get("action"), f"{location}.action")
		optional = _is_optional(declaration)
		if action_id not in SUPPORTED_TOOL_ACTION_IDS and not optional:
			raise DeclarativeResourceError(
				f"{location} references unsupported tool action '{action_id}'.",
			)
		if action_id not in action_ids and not optional:
			raise DeclarativeResourceError(
				f"{location} references unregistered action '{action_id}'.",
			)


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
def preflight_declarative_resources(registry: object) -> None:
	"""Validate package declarations against the active Ferrum action registry."""
	action_ids = _registry_action_ids(registry)
	_validate_menu_declarations(load_menu_declarations(), action_ids)
	_validate_mode_declarations(load_mode_declarations(), action_ids)
