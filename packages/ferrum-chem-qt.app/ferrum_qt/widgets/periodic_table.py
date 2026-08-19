"""UI-only periodic-table chooser with no chemistry model dependency."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets


ELEMENTS = (
	("H", "Hydrogen", 0, 0), ("He", "Helium", 0, 17),
	("Li", "Lithium", 1, 0), ("Be", "Beryllium", 1, 1),
	("B", "Boron", 1, 12), ("C", "Carbon", 1, 13),
	("N", "Nitrogen", 1, 14), ("O", "Oxygen", 1, 15),
	("F", "Fluorine", 1, 16), ("Ne", "Neon", 1, 17),
	("Na", "Sodium", 2, 0), ("Mg", "Magnesium", 2, 1),
	("Al", "Aluminium", 2, 12), ("Si", "Silicon", 2, 13),
	("P", "Phosphorus", 2, 14), ("S", "Sulfur", 2, 15),
	("Cl", "Chlorine", 2, 16), ("Ar", "Argon", 2, 17),
	("K", "Potassium", 3, 0), ("Ca", "Calcium", 3, 1),
	("Fe", "Iron", 3, 7), ("Cu", "Copper", 3, 10),
	("Zn", "Zinc", 3, 11), ("Br", "Bromine", 3, 16), ("I", "Iodine", 4, 16),
)


#============================================
class PeriodicTablePopup(PySide6.QtWidgets.QDialog):
	"""Choose an element symbol; a controller decides whether to mutate Rust."""

	element_selected = PySide6.QtCore.Signal(str)

	#============================================
	def __init__(self, parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Create a keyboard-reachable table of common chemistry elements."""
		super().__init__(parent)
		self.setWindowTitle(self.tr("Select Element"))
		self.setAccessibleName(self.tr("Periodic table"))
		self._selected_symbol = ""
		layout = PySide6.QtWidgets.QGridLayout(self)
		layout.setSpacing(2)
		for symbol, name, row, column in ELEMENTS:
			button = PySide6.QtWidgets.QPushButton(symbol, self)
			button.setObjectName(f"element-{symbol}")
			button.setMinimumSize(36, 36)
			button.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
			button.setAccessibleName(self.tr(f"{symbol}, {name}"))
			button.setToolTip(self.tr(name))
			button.clicked.connect(lambda _checked=False, value=symbol: self._select(value))
			layout.addWidget(button, row, column)

	#============================================
	def _select(self, symbol: str) -> None:
		"""Return the chosen UI value without applying any document mutation."""
		self._selected_symbol = symbol
		self.element_selected.emit(symbol)
		self.accept()

	#============================================
	@staticmethod
	def pick_element(parent: PySide6.QtWidgets.QWidget | None = None) -> str:
		"""Run the chooser and return its selected symbol or an empty result."""
		dialog = PeriodicTablePopup(parent)
		result = dialog.exec()
		if result == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return dialog._selected_symbol
		return ""
