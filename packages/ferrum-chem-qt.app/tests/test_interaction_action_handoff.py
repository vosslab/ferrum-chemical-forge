"""Behavior coverage for intentional pointer-action refusals."""

# Standard Library
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
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

	def cancel_active_pointer_authoring(self, *, clear_status: bool = True) -> None:
		"""Record the real handoff's required cancellation call."""
		del clear_status
		self.cancelled = True


#============================================
def test_intentional_pointer_action_refusal_resets_the_checked_action(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""A declared refusal reaches the window route without retaining tool ownership."""
	del qapp
	owner = _PointerActionOwner()
	reported: list[str] = []
	handoff = ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoff(
		owner, reported.append,
	)
	action = PySide6.QtGui.QAction("Draw", owner)
	action.setCheckable(True)

	def decline(_checked: bool) -> None:
		"""Decline activation through the declared action-handoff contract."""
		raise ferrum_qt.ferrum.interaction_action_handoff.FerrumInteractionActionHandoffRefusal(
			"A ready Ferrum drawing is required.",
		)

	handoff.connect(action, decline)
	action.trigger()

	assert owner.cancelled and not action.isChecked()
	assert reported == ["A ready Ferrum drawing is required."]
	owner.close()
	owner.deleteLater()
