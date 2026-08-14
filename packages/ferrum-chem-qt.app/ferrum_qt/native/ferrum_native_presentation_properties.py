"""Small host integration for independent native Plus and Text editors."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_plus_properties
import ferrum_qt.native.ferrum_native_presentation_deletion
import ferrum_qt.native.ferrum_native_presentation_stack
import ferrum_qt.native.ferrum_native_text_properties


#============================================
def install_plus_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install both actions while preserving the host's established Plus handle."""
	plus_action = (
		ferrum_qt.native.ferrum_native_plus_properties.install_plus_properties_action(
			window, edit_menu,
		)
	)
	window._edit_text_properties_action = (
		ferrum_qt.native.ferrum_native_text_properties.install_text_properties_action(
			window, edit_menu,
		)
	)
	window._delete_presentation_action = (
		ferrum_qt.native.ferrum_native_presentation_deletion.
		install_presentation_deletion_action(window, edit_menu)
	)
	window._presentation_stack_actions = (
		ferrum_qt.native.ferrum_native_presentation_stack.
		install_presentation_stack_actions(window, edit_menu)
	)
	return plus_action


#============================================
def refresh_plus_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Refresh the paired actions from one authoritative selection state."""
	ferrum_qt.native.ferrum_native_plus_properties.refresh_plus_properties_action(
		action, tab, active, pending, busy,
	)
	text_action = getattr(action.parent(), "_edit_text_properties_action")
	ferrum_qt.native.ferrum_native_text_properties.refresh_text_properties_action(
		text_action, tab, active, pending, busy,
	)
	delete_action = getattr(action.parent(), "_delete_presentation_action")
	ferrum_qt.native.ferrum_native_presentation_deletion.refresh_presentation_deletion_action(
		delete_action, tab, active, pending, busy,
	)
	stack_actions = getattr(action.parent(), "_presentation_stack_actions")
	ferrum_qt.native.ferrum_native_presentation_stack.refresh_presentation_stack_actions(
		stack_actions, tab, active, pending, busy,
	)
