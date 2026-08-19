"""Build declared Ferrum menu clients from existing QActions."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.declarative_resources


#============================================
def _existing_menu(
		menu_bar: PySide6.QtWidgets.QMenuBar, label: str,
		) -> PySide6.QtWidgets.QMenu | None:
	"""Return a menu already owned by the window for one translated label."""
	for action in menu_bar.actions():
		menu = action.menu()
		if menu is not None and menu.title().replace("&", "") == label:
			return menu
	return None


#============================================
def build_declared_menus(
		window: PySide6.QtWidgets.QMainWindow, registry: object,
		) -> dict[str, PySide6.QtWidgets.QMenu]:
	"""Ensure every declared static action has one existing menu client.

	Existing menus and actions are deliberately reused.  Feature-owned menus
	remain intact while the compact Ferrum declaration verifies the common
	product commands have a single discoverable placement.
	"""
	ferrum_qt.declarative_resources.preflight_declarative_resources(registry)
	# Keep the Python wrapper alive as well as Qt ownership.  This matters for
	# a freshly constructed offscreen QMainWindow, whose implicit menu bar can
	# otherwise be collected before its QMenu children are queried.
	menu_bar = window.menuBar()
	setattr(window, "_ferrum_declared_menu_bar", menu_bar)
	menus: dict[str, PySide6.QtWidgets.QMenu] = {}
	for declaration in ferrum_qt.declarative_resources.load_menu_declarations()["menus"]:
		label = window.tr(declaration["label_key"])
		menu = _existing_menu(menu_bar, label)
		if menu is None:
			# Give the menu the durable window owner before adding its menu-bar
			# client.  It avoids a transient wrapper lifetime on offscreen Qt.
			menu = PySide6.QtWidgets.QMenu(label, window)
			menu_bar.addMenu(menu)
		menu.setToolTip(window.tr(declaration["help_key"]))
		menu.setStatusTip(window.tr(declaration["help_key"]))
		menus[declaration["name"]] = menu
		present = set(menu.actions())
		for item in declaration["items"]:
			if item.get("separator") is True:
				continue
			action = registry.get_qt_action(item["action"])
			if action is not None and action not in present:
				menu.addAction(action)
				present.add(action)
	return menus
