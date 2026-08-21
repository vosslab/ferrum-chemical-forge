"""Shared keyboard and assistive-technology policy for Ferrum dialogs."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class DialogAccessibilityMetadata:
	"""Recorded accessibility contract for one constructed dialog class.

	The registry is deliberately populated from real widgets, after each dialog
	has finished building.  Tests can therefore check the actual traversal
	contract rather than a second, stale list of control names.
	"""

	dialog_class: str
	initial_focus: str
	tab_order: tuple[str, ...]
	accept_button: str | None
	reject_button: str | None


DIALOG_ACCESSIBILITY_METADATA: dict[str, DialogAccessibilityMetadata] = {}


#============================================
def _semantic_name(widget: PySide6.QtWidgets.QWidget) -> str:
	"""Return a stable spoken name, adding a useful fallback where needed."""
	name = widget.accessibleName().strip()
	if name:
		return name
	if isinstance(widget, PySide6.QtWidgets.QAbstractButton):
		name = widget.text().replace("&", "").strip()
	if not name and isinstance(widget, PySide6.QtWidgets.QLineEdit):
		name = widget.placeholderText().strip()
	if not name:
		name = widget.objectName().strip().replace("_", " ")
	if not name:
		name = widget.metaObject().className()
	widget.setAccessibleName(name)
	return name


#============================================
def _task_controls(dialog: PySide6.QtWidgets.QDialog) -> list[PySide6.QtWidgets.QWidget]:
	"""Return the live keyboard controls in layout (task) order.

	Qt preserves child construction order, which matches each Ferrum dialog's
	form/read order.  Buttons are moved to the end so the conventional final
	choice follows the fields rather than interrupting them.
	"""
	controls = [
		widget for widget in dialog.findChildren(PySide6.QtWidgets.QWidget)
		if widget.focusPolicy() != PySide6.QtCore.Qt.FocusPolicy.NoFocus
		and widget.isEnabled()
		and not widget.isHidden()
		and widget.window() is dialog
		and not isinstance(widget.parentWidget(), (
			PySide6.QtWidgets.QAbstractSpinBox,
			PySide6.QtWidgets.QComboBox,
		))
		and not isinstance(widget, PySide6.QtWidgets.QDialogButtonBox)
	]
	ordinary = [
		widget for widget in controls
		if not isinstance(widget, PySide6.QtWidgets.QAbstractButton)
	]
	buttons = [
		widget for widget in controls
		if isinstance(widget, PySide6.QtWidgets.QAbstractButton)
	]
	return ordinary + buttons


#============================================
def finalize_dialog_accessibility(dialog: PySide6.QtWidgets.QDialog) -> None:
	"""Apply one explicit focus cycle, semantic names, and standard keys."""
	dialog.setAccessibleName(dialog.windowTitle() or type(dialog).__qualname__)
	if not dialog.accessibleDescription().strip():
		dialog.setAccessibleDescription(
			"Ferrum dialog. Use Tab to move through controls, Enter to confirm, "
			"or Escape to cancel.",
		)
	controls = _task_controls(dialog)
	for widget in dialog.findChildren(PySide6.QtWidgets.QWidget):
		if (
			widget.focusPolicy() != PySide6.QtCore.Qt.FocusPolicy.NoFocus
			and widget.window() is dialog
		):
			_semantic_name(widget)
	preferred = dialog.property("ferrum_initial_focus_widget")
	if isinstance(preferred, PySide6.QtWidgets.QWidget) and preferred in controls:
		controls.remove(preferred)
		controls.insert(0, preferred)
	accept_button: PySide6.QtWidgets.QAbstractButton | None = None
	reject_button: PySide6.QtWidgets.QAbstractButton | None = None
	for widget in controls:
		_semantic_name(widget)
		if isinstance(widget, PySide6.QtWidgets.QPushButton):
			if widget.text().replace("&", "").strip().lower() in (
				"ok", "apply", "save", "create fragment", "start placement",
			):
				accept_button = widget
			if widget.text().replace("&", "").strip().lower() in (
				"cancel", "close",
			):
				reject_button = widget
	if accept_button is not None:
		accept_button.setDefault(True)
		accept_button.setAutoDefault(True)
	if reject_button is not None:
		reject_button.setAutoDefault(False)
	if len(controls) > 1:
		for before, after in zip(controls, controls[1:]):
			dialog.setTabOrder(before, after)
	if controls:
		controls[0].setFocus(PySide6.QtCore.Qt.FocusReason.OtherFocusReason)
	metadata = DialogAccessibilityMetadata(
		dialog_class=type(dialog).__qualname__,
		initial_focus=_semantic_name(controls[0]) if controls else "",
		tab_order=tuple(_semantic_name(widget) for widget in controls),
		accept_button=_semantic_name(accept_button) if accept_button is not None else None,
		reject_button=_semantic_name(reject_button) if reject_button is not None else None,
	)
	DIALOG_ACCESSIBILITY_METADATA[metadata.dialog_class] = metadata


#============================================
class FerrumAccessibleDialog(PySide6.QtWidgets.QDialog):
	"""QDialog base that applies Ferrum's live dialog contract on first show."""

	#============================================
	def showEvent(self, event: PySide6.QtGui.QShowEvent) -> None:
		"""Finalize controls only after Qt has made every child visible."""
		finalize_dialog_accessibility(self)
		super().showEvent(event)
