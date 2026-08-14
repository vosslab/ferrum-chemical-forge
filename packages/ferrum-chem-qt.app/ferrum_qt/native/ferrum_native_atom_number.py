"""Focused native atom-number dialog with no legacy document authority."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


_MAX_U64 = (1 << 64) - 1


#============================================
def can_clear_selected_atom_number(tab: object | None) -> bool:
	"""Return whether one current durable atom owns a removable number pair."""
	if tab is None or tab.requires_refresh:
		return False
	try:
		return bool(tab.selected_atom_has_number())
	except (AttributeError, RuntimeError):
		return False


#============================================
class FerrumNativeAtomNumberDialog(PySide6.QtWidgets.QDialog):
	"""Collect one positive decimal number and explicit visibility value."""

	#============================================
	def __init__(self, number: int | None, show_number: bool | None,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build a small validated native-number form."""
		super().__init__(parent)
		if number is not None and (type(number) is not int or number <= 0):
			raise ValueError("native atom number must be a positive integer")
		if show_number is not None and type(show_number) is not bool:
			raise TypeError("native atom number visibility must be a bool")
		self.setWindowTitle(self.tr("Set Atom Number"))
		layout = PySide6.QtWidgets.QFormLayout(self)
		self.number_edit = PySide6.QtWidgets.QLineEdit(self)
		self.number_edit.setText(str(number if number is not None else 1))
		self.number_edit.setAccessibleName(self.tr("Positive atom number"))
		layout.addRow(self.tr("Number:"), self.number_edit)
		self.show_number = PySide6.QtWidgets.QCheckBox(self.tr("Show number"), self)
		self.show_number.setChecked(True if show_number is None else show_number)
		layout.addRow("", self.show_number)
		self.validation_message = PySide6.QtWidgets.QLabel(self)
		self.validation_message.setWordWrap(True)
		layout.addRow("", self.validation_message)
		self.buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
			parent=self,
		)
		self.buttons.accepted.connect(self.accept)
		self.buttons.rejected.connect(self.reject)
		layout.addRow(self.buttons)
		self.number_edit.textChanged.connect(self._refresh_validation)
		self._refresh_validation()

	#============================================
	def assignment(self) -> tuple[int, bool]:
		"""Return the validated exact operation values after acceptance."""
		number = _parse_positive_u64(self.number_edit.text())
		if number is None:
			raise ValueError("atom number must be a canonical positive decimal integer")
		return number, self.show_number.isChecked()

	#============================================
	@PySide6.QtCore.Slot(str)
	def _refresh_validation(self, _text: str = "") -> None:
		"""Keep acceptance and its explanation synchronized with current text."""
		valid = _parse_positive_u64(self.number_edit.text()) is not None
		button = self.buttons.button(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok,
		)
		button.setEnabled(valid)
		self.validation_message.setText(
			"" if valid else self.tr(
				"Enter a positive whole number without signs, spaces, or leading zeroes.",
			)
		)


#============================================
def _parse_positive_u64(text: str) -> int | None:
	"""Parse the exact closed Rust operation range without an arbitrary UI cap."""
	if type(text) is not str or not text or not text.isascii() or not text.isdecimal():
		return None
	value = int(text)
	if value <= 0 or value > _MAX_U64 or str(value) != text:
		return None
	return value
