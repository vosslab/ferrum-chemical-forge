"""Graphics retirement entry points used by Qt undo history."""

import PySide6.QtGui
import ferrum_qt.canvas.graphics_retirement


#============================================
def dispose_undo_stack_graphics(undo_stack: PySide6.QtGui.QUndoStack,
		seen: set | None = None) -> None:
	"""Disconnect graphics retained only by commands before clearing a stack."""
	coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
	coordinator.dispose_undo_stack_graphics(undo_stack, seen)
	coordinator.raise_if_callback_failed("Undo graphics were detached after a disposal failure")


#============================================
def _dispose_command_graphics(command: PySide6.QtGui.QUndoCommand, seen: set,
		errors: list) -> None:
	"""Provide the legacy direct caller hook through the retirement coordinator."""
	coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
	coordinator._dispose_command_graphics(command, seen, False)
	errors.extend(coordinator.report.callback_errors)
