"""Rust-catalog projection for the UI-only periodic-table chooser."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

#============================================
class PeriodicTablePopup(PySide6.QtWidgets.QDialog):
	"""Choose an element symbol; a controller decides whether to mutate Rust."""

	element_selected = PySide6.QtCore.Signal(str)

	#============================================
	def __init__(self, entries: tuple[object, ...],
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Project the Rust-issued picker entries as a keyboard-reachable table."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Select Element"))
		self.setAccessibleName(self.tr("Periodic table"))
		self._selected_symbol = ""
		layout = PySide6.QtWidgets.QGridLayout(self)
		layout.setSpacing(2)
		for entry in entries:
			button = PySide6.QtWidgets.QPushButton(entry.symbol, self)
			button.setObjectName(f"element-{entry.symbol}")
			button.setMinimumSize(36, 36)
			button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
			button.setAccessibleName(self.tr(f"{entry.symbol}, {entry.display_name}"))
			button.setAccessibleDescription(self.tr(
				f"{entry.symbol}, {entry.display_name}. Category: {entry.category}.",
			))
			button.setToolTip(self.tr(entry.display_name))
			button.setStyleSheet(f"background-color: {entry.color};")
			button.clicked.connect(
				lambda _checked=False, symbol=entry.symbol: self._select(symbol),
			)
			layout.addWidget(button, entry.grid_row, entry.grid_column)

	#============================================
	def _select(self, symbol: str) -> None:
		"""Return the chosen UI value without applying any document mutation."""
		self._selected_symbol = symbol
		self.element_selected.emit(symbol)
		self.accept()

	#============================================
	@staticmethod
	def pick_element(entries: tuple[object, ...],
			parent: PySide6.QtWidgets.QWidget | None = None) -> str:
		"""Run the supplied Rust-catalog projection and return UI intent or none."""
		dialog = PeriodicTablePopup(entries, parent)
		result = dialog.exec()
		if result == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return dialog._selected_symbol
		return ""
