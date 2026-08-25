"""Shared visible-menu interactions for native Ferrum Qt behavior tests."""

# Standard Library
import collections.abc

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets


#============================================
def click_visible_menu_action(
		window: PySide6.QtWidgets.QMainWindow, label: str,
		qapp: PySide6.QtWidgets.QApplication,
		after_menu_visible: collections.abc.Callable[[], None] | None = None,
		) -> None:
	"""Activate one labelled command through its visible top-level menu item."""
	menu_bar = window.menuBar()
	for menu_action in menu_bar.actions():
		menu = menu_action.menu()
		if menu is None:
			continue
		for candidate in menu.actions():
			if candidate.text().replace("&", "") != label:
				continue
			PySide6.QtTest.QTest.mouseClick(
				menu_bar, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu_bar.actionGeometry(menu_action).center(),
			)
			qapp.processEvents()
			if not menu.isVisible():
				raise AssertionError(f"Visible menu did not open for {label!r}")
			if after_menu_visible is not None:
				after_menu_visible()
			PySide6.QtTest.QTest.mouseClick(
				menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				menu.actionGeometry(candidate).center(),
			)
			qapp.processEvents()
			return
	raise AssertionError(f"No visible menu action is labelled {label!r}")
