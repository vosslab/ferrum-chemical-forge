"""Behavior coverage for intentional pointer-action refusals."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.interaction_action_handoff


#============================================
class _PointerActionOwner(PySide6.QtWidgets.QWidget):
	"""Provide the public owner seam required by a real action handoff."""

	def __init__(self) -> None:
		"""Create one widget whose canvas ownership can be retired."""
		super().__init__()
		self.cancelled = False
		self.events: list[object] = []

	def cancel_active_pointer_authoring(self, *, clear_status: bool = True) -> None:
		"""Record the real handoff's required cancellation call."""
		self.cancelled = True
		self.events.append(("cancel", clear_status))


#============================================
def test_intentional_pointer_action_refusal_resets_the_checked_action(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A noncheckable refusal retires the previously checked pointer tool."""
	del qapp
	owner = _PointerActionOwner()
	reported: list[str] = []
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, reported.append,
	)
	selection_action = PySide6.QtGui.QAction("Select Structure", owner)
	selection_action.setCheckable(True)
	refusal_action = PySide6.QtGui.QAction("Attach Compact Group", owner)

	def select(_checked: bool) -> None:
		"""Claim the selection-like pointer tool for this narrow handoff contract."""

	def decline() -> None:
		"""Decline activation through the declared action-handoff contract."""
		raise ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoffRefusal(
			"A ready Ferrum drawing is required.",
		)

	handoff.connect(selection_action, select)
	handoff.connect(refusal_action, decline)
	selection_action.trigger()
	refusal_action.trigger()

	assert owner.cancelled and not selection_action.isChecked()
	assert not refusal_action.isCheckable()
	assert reported == ["A ready Ferrum drawing is required."]
	owner.close()
	owner.deleteLater()


#============================================
def test_popup_handoff_waits_for_a_real_menu_hide_before_dispatching(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A menu-triggered action runs once after its actual popup lifecycle retires."""
	owner = _PointerActionOwner()
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, lambda _detail: None,
	)
	outgoing_action = PySide6.QtGui.QAction("Select Structure", owner)
	outgoing_action.setCheckable(True)
	incoming_action = PySide6.QtGui.QAction("Direct Bond", owner)
	incoming_action.setCheckable(True)
	events: list[object] = []

	def activate(checked: bool) -> None:
		"""Record the guard state visible to the pointer-owning handler."""
		events.append(("handler", outgoing_action.isChecked(), checked))

	handoff.connect(outgoing_action, lambda _checked: None)
	handoff.connect(incoming_action, activate)
	outgoing_action.trigger()
	owner.events.clear()
	menu = PySide6.QtWidgets.QMenu(owner)
	handoff.add_registered_action_to_menu(menu, incoming_action)
	menu.popup(PySide6.QtCore.QPoint(0, 0))
	qapp.processEvents()

	assert PySide6.QtWidgets.QApplication.activePopupWidget() is menu
	incoming_action.trigger()
	assert events == []
	menu.hide()
	qapp.processEvents()

	assert owner.events + events == [
		("cancel", False),
		("handler", False, True),
	]
	owner.close()
	owner.deleteLater()
