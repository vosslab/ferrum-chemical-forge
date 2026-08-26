"""Build the complete Ferrum menu hierarchy from the YAML declaration."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.declarative_resources


def _translate(window: PySide6.QtWidgets.QMainWindow, text: str) -> str:
	"""Return the current window translation for one declaration text key."""
	return window.tr(text)


#============================================
def _require_bound_action(registry: object, action_id: str) -> PySide6.QtGui.QAction:
	"""Return the existing QAction for one declaration or fail before assembly."""
	action = registry.get_qt_action(action_id)
	if not isinstance(action, PySide6.QtGui.QAction):
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"menus.yaml action '{action_id}' is declared but has no bound QAction.",
		)
	return action


#============================================
def _require_dynamic_menu(registry: object, menu_id: str) -> PySide6.QtWidgets.QMenu:
	"""Return the existing changing QMenu for one declaration or fail loudly."""
	menu = registry.get_dynamic_menu(menu_id)
	if not isinstance(menu, PySide6.QtWidgets.QMenu):
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"menus.yaml dynamic menu '{menu_id}' is declared but not registered.",
		)
	return menu


#============================================
def _render_items(
		menu: PySide6.QtWidgets.QMenu, items: list, window: PySide6.QtWidgets.QMainWindow,
		registry: object,
		) -> None:
	"""Render one already validated ordered declaration sequence recursively."""
	previous_was_section = False
	for item in items:
		if "action" in item:
			menu.addAction(_require_bound_action(registry, item["action"]))
			previous_was_section = False
			continue
		if "separator" in item:
			menu.addSeparator()
			previous_was_section = False
			continue
		if "dynamic_menu" in item:
			menu.addMenu(_require_dynamic_menu(registry, item["dynamic_menu"]))
			previous_was_section = False
			continue
		if "section" in item:
			section = item["section"]
			if previous_was_section:
				menu.addSeparator()
			if "label_key" in section:
				menu.addSection(_translate(window, section["label_key"]))
			_render_items(menu, section["items"], window, registry)
			previous_was_section = True
			continue
		submenu = item["submenu"]
		submenu_label = _translate(window, submenu["label_key"])
		submenu_client = PySide6.QtWidgets.QMenu(submenu_label, menu)
		submenu_client.setProperty("ferrum_menu_id", submenu["id"])
		submenu_help = _translate(window, submenu["help_key"])
		submenu_client.setToolTip(submenu_help)
		submenu_client.setStatusTip(submenu_help)
		_render_items(submenu_client, submenu["items"], window, registry)
		menu.addMenu(submenu_client)
		previous_was_section = False


#============================================
def build_declared_menus(
		window: PySide6.QtWidgets.QMainWindow, registry: object,
		) -> dict[str, PySide6.QtWidgets.QMenu]:
	"""Build the one YAML-owned menu tree from existing feature-owned clients."""
	ferrum_qt.declarative_resources.preflight_menu_declarations(registry)
	menu_bar = window.menuBar()
	if menu_bar.property("ferrum_declared_menus_assembled"):
		raise RuntimeError(
			"Ferrum menus are already assembled for this window.",
		)
	# Keep the Python wrapper alive as well as Qt ownership for offscreen windows.
	setattr(window, "_ferrum_declared_menu_bar", menu_bar)
	menus: dict[str, PySide6.QtWidgets.QMenu] = {}
	declarations = ferrum_qt.declarative_resources.load_menu_declarations()["menus"]
	for declaration in declarations:
		label = _translate(window, declaration["label_key"])
		menu = PySide6.QtWidgets.QMenu(label, window)
		menu.setProperty("ferrum_menu_id", declaration["id"])
		help_text = _translate(window, declaration["help_key"])
		menu.setToolTip(help_text)
		menu.setStatusTip(help_text)
		_render_items(menu, declaration["items"], window, registry)
		menus[declaration["id"]] = menu
	# Render the full client tree before exposing any of it in the live menu bar.
	# A late client-resolution error can then leave this window ready for a retry.
	for menu in menus.values():
		menu_bar.addMenu(menu)
	menu_bar.setProperty("ferrum_declared_menus_assembled", True)
	return menus
