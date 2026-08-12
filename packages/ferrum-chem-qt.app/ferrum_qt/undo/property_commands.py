"""Undo commands for scalar model properties."""

# PIP3 modules
import PySide6.QtGui


#============================================
class ChangePropertyCommand(PySide6.QtGui.QUndoCommand):
	"""Store and restore one named model property."""

	#============================================
	def __init__(self, model: object, property_name: str, old_value: object,
			new_value: object, text: str = "Change Property") -> None:
		"""Capture the explicit before and after values."""
		super().__init__(text)
		self._model = model
		self._property_name = property_name
		self._old_value = old_value
		self._new_value = new_value

	#============================================
	def redo(self) -> None:
		"""Apply the new property value."""
		setattr(self._model, self._property_name, self._new_value)

	#============================================
	def undo(self) -> None:
		"""Revert to the old property value."""
		setattr(self._model, self._property_name, self._old_value)
