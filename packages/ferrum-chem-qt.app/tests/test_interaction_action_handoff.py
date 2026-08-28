"""Behavior coverage for intentional pointer-action refusals."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.interaction_action_handoff


#============================================
class _PointerActionOwner(PySide6.QtWidgets.QWidget):
	"""Provide the public owner seam required by a real action handoff."""

	def __init__(self) -> None:
		"""Create one widget whose canvas ownership can be cancelled."""
		super().__init__()
		self.cancelled = False
		self.events: list[object] = []

	def cancel_active_pointer_authoring(self, *, clear_status: bool = True) -> None:
		"""Record the real handoff's required cancellation call."""
		self.cancelled = True
		self.events.append(("cancel", clear_status))


#============================================
def test_intentional_pointer_action_refusal_preserves_the_current_pointer_tool(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A refusal during admission leaves prior ownership and checks untouched."""
	del qapp
	owner = _PointerActionOwner()
	reported: list[str] = []
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, reported.append,
	)
	selection_action = PySide6.QtGui.QAction("Select Structure", owner)
	selection_action.setCheckable(True)
	refusal_action = PySide6.QtGui.QAction("Attach Compact Group", owner)

	def select(_checked: bool) -> object:
		"""Claim the selection-like pointer tool for this narrow handoff contract."""
		return ferrum_qt.ferrum.interaction_action_handoff.FerrumAdmittedInteractionCommand(
			lambda: None,
		)

	def decline(_checked: bool) -> object:
		"""Decline activation through the declared action-handoff contract."""
		raise ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoffRefusal(
			"A ready Ferrum drawing is required.",
		)

	handoff.connect(selection_action, select)
	handoff.connect(refusal_action, decline)
	selection_action.trigger()
	refusal_action.trigger()

	assert owner.cancelled and selection_action.isChecked()
	assert not refusal_action.isCheckable()
	assert reported == ["A ready Ferrum drawing is required."]
	owner.close()
	owner.deleteLater()


#============================================
def test_admitted_command_runs_after_one_handoff_in_order(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Preparation captures facts before one cancellation and one command invoke."""
	del qapp
	owner = _PointerActionOwner()
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, lambda _detail: None,
	)
	action = PySide6.QtGui.QAction("Admitted", owner)

	def prepare(checked: bool) -> object:
		assert not checked
		owner.events.append("prepare")
		return ferrum_qt.ferrum.interaction_action_handoff.FerrumAdmittedInteractionCommand(
			lambda: owner.events.append("invoke"),
		)

	handoff.connect(action, prepare)
	action.trigger()

	assert owner.events == ["prepare", ("cancel", False), "invoke"]
	owner.close()
	owner.deleteLater()


#============================================
def test_invalid_preparation_return_reports_without_taking_ownership(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A malformed preparer result is a shared refusal before cancellation."""
	del qapp
	owner = _PointerActionOwner()
	reported: list[str] = []
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, reported.append,
	)
	action = PySide6.QtGui.QAction("Malformed", owner)
	handoff.connect(action, lambda _checked: object())
	action.trigger()

	assert owner.events == []
	assert reported == ["Ferrum interaction preparation returned an invalid command."]
	owner.close()
	owner.deleteLater()


#============================================
def test_refusal_payload_reaches_the_shared_failure_reporter(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Feature-owned presentation payloads survive generic admission refusal."""
	del qapp
	owner = _PointerActionOwner()
	reported: list[object] = []
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, reported.append,
	)
	action = PySide6.QtGui.QAction("Feature refusal", owner)
	payload = object()

	def refuse(_checked: bool) -> object:
		raise ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoffRefusal(
			"Specific recovery guidance.", payload,
		)

	handoff.connect(action, refuse)
	action.trigger()

	assert reported == [payload]
	assert owner.events == []
	owner.close()
	owner.deleteLater()


#============================================
def test_main_window_safely_wraps_invalid_handoff_refusal_payload(
		main_window: object) -> None:
	"""Only exact presentation requests cross the generic handoff boundary."""
	presented: list[object] = []
	main_window._show_edit_refusal = presented.append

	main_window._present_interaction_action_handoff_failure_v1(object())
	wrapped = presented.pop()
	assert wrapped.technical_details == (
		"Ferrum interaction action returned an invalid refusal payload."
	)

	feature_request = main_window._typed_refusal(
		"edit_document", "unavailable_operation", "Feature-owned recovery.",
	)
	main_window._present_interaction_action_handoff_failure_v1(feature_request)
	assert presented == [feature_request]


#============================================
def test_admitted_command_refuses_a_direct_second_invoke() -> None:
	"""The public command boundary cannot accidentally run an admitted action twice."""
	events: list[str] = []
	command = ferrum_qt.ferrum.interaction_action_handoff.FerrumAdmittedInteractionCommand(
		lambda: events.append("invoke"),
	)
	command.invoke()

	with pytest.raises(RuntimeError, match="already invoked"):
		command.invoke()
	assert events == ["invoke"]


#============================================
def test_popup_defers_one_preparation_and_handoff_until_hide(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A menu trigger prepares and invokes exactly once after popup teardown."""
	owner = _PointerActionOwner()
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, lambda _detail: None,
	)
	menu = PySide6.QtWidgets.QMenu(owner)
	action = PySide6.QtGui.QAction("Deferred", owner)
	menu.addAction(action)

	def prepare(_checked: bool) -> object:
		owner.events.append("prepare")
		return ferrum_qt.ferrum.interaction_action_handoff.FerrumAdmittedInteractionCommand(
			lambda: owner.events.append("invoke"),
		)

	handoff.connect(action, prepare)
	menu.popup(PySide6.QtCore.QPoint(0, 0))
	qapp.processEvents()
	action.trigger()
	assert owner.events == []
	menu.hide()
	qapp.processEvents()
	qapp.processEvents()

	assert owner.events == ["prepare", ("cancel", False), "invoke"]
	owner.close()
	owner.deleteLater()


#============================================
def test_popup_handoff_waits_for_a_real_menu_hide_before_dispatching(
		qapp: PySide6.QtWidgets.QApplication, main_window: object) -> None:
	"""A YAML-menu pointer action waits for its active popup to hide."""
	draw_menu = main_window._declared_menus["draw"]
	selection_action = main_window._action_registry.get_qt_action("draw.selection.structure")
	draw_menu.popup(PySide6.QtCore.QPoint(0, 0))
	qapp.processEvents()

	assert PySide6.QtWidgets.QApplication.activePopupWidget() is draw_menu
	selection_action.trigger()
	assert selection_action.isChecked()
	draw_menu.hide()
	qapp.processEvents()

	assert selection_action.isChecked()
	assert main_window._window_mode_sync.active_state.mode_id == "edit"
