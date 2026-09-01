"""Tests for Ferrum's YAML-authoritative menu declaration schema."""

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.declarative_resources


#============================================
def _menu(items: list[dict]) -> dict:
	"""Return the smallest valid top-level menu around test nodes."""
	menu = {
		"id": "draw",
		"label_key": "Draw",
		"help_key": "Canvas authoring commands",
		"items": items,
	}
	return {
		"contexts": [{
			"id": "selected_structure",
			"accessible_name": "Selected structure actions",
			"groups": [{"id": "actions", "actions": ["draw.bond"]}],
		}],
		"menus": [menu],
	}


#============================================
def _ribbon(tab: dict) -> dict:
	"""Return the smallest complete persistent-header ribbon around one tab."""
	return {
		"quick_access": ["draw.bond"],
		"global_actions": ["view.command_palette"],
		"tabs": [tab],
	}


#============================================
@pytest.mark.parametrize(("item", "message"), [
	({"action": "draw.bond", "separator": True}, "node form"),
	({"section": {"id": "bonds", "items": []}}, "items must be a nonempty list"),
	(
		{"submenu": {"id": "rings", "label_key": "Rings", "items": []}},
		"help_key",
	),
])
def test_recursive_menu_schema_rejects_malformed_nodes(
		item: dict, message: str,
		) -> None:
	"""Every recursive menu node has one complete, valid declaration form."""
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match=message,
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			_menu([item]), frozenset({"draw.bond"}),
		)


#============================================
def test_recursive_menu_schema_rejects_duplicate_nested_placements() -> None:
	"""A static command cannot silently acquire two canonical menu clients."""
	data = _menu([
		{"section": {"id": "bonds", "items": [{"action": "draw.bond"}]}},
		{"submenu": {
			"id": "ring_tools",
			"label_key": "Rings",
			"help_key": "Insert ring structures",
			"items": [{"action": "draw.bond"}],
		}},
	])
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Duplicate declared menu action ID",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"draw.bond"}),
		)


#============================================
def test_recursive_menu_schema_rejects_unresolved_static_action() -> None:
	"""A missing feature binding is a construction failure, never an omission."""
	data = _menu([{"section": {
		"id": "bonds",
		"items": [{"action": "draw.unbound"}],
	}}])
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="action",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"draw.bond"}),
		)


#============================================
def test_recursive_menu_schema_rejects_unregistered_dynamic_menu() -> None:
	"""A dynamic collection has an explicit owner before YAML can place it."""
	data = _menu([{"dynamic_menu": "file.recent"}])
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="dynamic menu",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"draw.bond"}), dynamic_menu_ids=frozenset(),
		)


#============================================
@pytest.mark.parametrize("section", [
	{"id": "bonds"},
	{"items": [{"action": "draw.bond"}]},
])
def test_recursive_menu_schema_reports_missing_section_required_keys(
		section: dict,
		) -> None:
	"""Incomplete sections fail at the resource boundary with a useful error."""
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="section must contain id, items",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			_menu([{"section": section}]), frozenset({"draw.bond"}),
		)


#============================================
def test_recursive_menu_schema_rejects_duplicate_sibling_dynamic_menu_placements() -> None:
	"""Sibling entries cannot assign one changing menu two client positions."""
	data = _menu([
		{"dynamic_menu": "file.recent"},
		{"dynamic_menu": "file.recent"},
	])
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Duplicate declared dynamic menu ID",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"draw.bond"}), dynamic_menu_ids=frozenset({"file.recent"}),
		)


#============================================
def test_recursive_menu_schema_rejects_duplicate_nested_dynamic_menu_placements() -> None:
	"""Nested entries share the changing menu's one YAML insertion point."""
	data = _menu([
		{"dynamic_menu": "file.recent"},
		{"submenu": {
			"id": "history", "label_key": "History", "help_key": "Prior files",
			"items": [{"dynamic_menu": "file.recent"}],
		}},
	])
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match="Duplicate declared dynamic menu ID",
		):
		ferrum_qt.declarative_resources._validate_menu_declarations(
			data, frozenset({"draw.bond"}), dynamic_menu_ids=frozenset({"file.recent"}),
		)


#============================================
def test_action_placement_projection_prefers_declared_menu_breadcrumb() -> None:
	"""Ordinary menus take precedence over a supplemental ribbon placement."""
	menu_data = _menu([{"section": {
		"id": "reactions", "label_key": "Reactions",
		"items": [{"action": "chemistry.reaction.create"}],
	}}])
	ribbon_data = _ribbon({
		"id": "reactions", "label_key": "Reactions",
		"groups": [{
			"id": "structure", "label_key": "Reaction structure",
			"overflow_label_key": "More reaction commands", "accent": "reaction",
			"entries": [{
				"action": "chemistry.reaction.create", "role": "primary", "priority": "required",
			}],
		}],
	})
	projection = ferrum_qt.declarative_resources._build_action_placement_projection(
		menu_data, ribbon_data,
		frozenset({"chemistry.reaction.create", "draw.bond", "view.command_palette"}),
	)
	assert projection["chemistry.reaction.create"] == ("Draw", "Reactions")


#============================================
def test_action_placement_projection_uses_ribbon_fallback_and_omits_unplaced_actions() -> None:
	"""Ribbon-only actions retain their declared path while absent actions stay absent."""
	ribbon_data = _ribbon({
		"id": "reactions", "label_key": "Reactions",
		"groups": [{
			"id": "structure", "label_key": "Reaction structure",
			"overflow_label_key": "More reaction commands", "accent": "reaction",
			"entries": [{
				"action": "chemistry.reaction.create", "role": "primary", "priority": "required",
			}],
		}],
	})
	projection = ferrum_qt.declarative_resources._build_action_placement_projection(
		_menu([{"action": "draw.bond"}]), ribbon_data,
		frozenset({
			"chemistry.reaction.create", "draw.bond", "draw.unplaced", "view.command_palette",
		}),
	)
	assert projection["chemistry.reaction.create"] == ("Reactions", "Reaction structure")
	assert projection.get("draw.unplaced", ()) == ()


#============================================
def test_action_placement_projection_refuses_unresolved_ribbon_action() -> None:
	"""A ribbon reference without an action declaration cannot become a breadcrumb."""
	ribbon_data = _ribbon({
		"id": "draw", "label_key": "Draw",
		"groups": [{
			"id": "tools", "label_key": "Tools", "overflow_label_key": "More tools",
			"accent": "drawing",
			"entries": [{
				"action": "draw.unbound", "role": "primary", "priority": "required",
			}],
		}],
	})
	with pytest.raises(
			ferrum_qt.declarative_resources.DeclarativeResourceError,
			match=r"ribbon_layout\.yaml\.tabs\[0\]\.groups\[0\]\.entries\[0\]\.action references unresolved action 'draw\.unbound'\.",
		):
		ferrum_qt.declarative_resources._build_action_placement_projection(
			_menu([{"action": "draw.bond"}]), ribbon_data,
			frozenset({"draw.bond", "view.command_palette"}),
		)
