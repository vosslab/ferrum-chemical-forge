"""Small host integration for independent Ferrum Plus and Text editors."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.plus_properties
import ferrum_qt.ferrum.presentation_deletion
import ferrum_qt.ferrum.presentation_stack
import ferrum_qt.ferrum.text_properties


#============================================
def install_plus_properties_action(window: object) -> PySide6.QtGui.QAction:
	"""Construct paired actions while preserving the host's established Plus handle."""
	plus_action = (
		ferrum_qt.ferrum.plus_properties.install_plus_properties_action(
			window,
		)
	)
	window._edit_text_properties_action = (
		ferrum_qt.ferrum.text_properties.install_text_properties_action(
			window,
		)
	)
	window._delete_presentation_action = (
		ferrum_qt.ferrum.presentation_deletion.
		install_presentation_deletion_action(window)
	)
	window._presentation_stack_actions = (
		ferrum_qt.ferrum.presentation_stack.
		install_presentation_stack_actions(window)
	)
	return plus_action


#============================================
def refresh_plus_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Refresh the paired actions from one authoritative selection state."""
	ferrum_qt.ferrum.plus_properties.refresh_plus_properties_action(
		action, tab, active, pending, busy,
	)
	text_action = getattr(action.parent(), "_edit_text_properties_action")
	ferrum_qt.ferrum.text_properties.refresh_text_properties_action(
		text_action, tab, active, pending, busy,
	)
	delete_action = getattr(action.parent(), "_delete_presentation_action")
	ferrum_qt.ferrum.presentation_deletion.refresh_presentation_deletion_action(
		delete_action, tab, active, pending, busy,
	)
	stack_actions = getattr(action.parent(), "_presentation_stack_actions")
	ferrum_qt.ferrum.presentation_stack.refresh_presentation_stack_actions(
		stack_actions, tab, active, pending, busy,
	)
