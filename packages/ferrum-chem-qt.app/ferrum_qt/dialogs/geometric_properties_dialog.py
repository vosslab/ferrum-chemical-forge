"""Detached editor for geometric presentation stroke and fill appearance."""

# Standard Library
import collections.abc
import math

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
def _validated_color(color: str, field_name: str) -> str:
	"""Return one canonical valid color without changing persistent state."""
	if type(color) is not str or not color:
		raise ValueError(f"{field_name} must be a nonempty color string")
	parsed = PySide6.QtGui.QColor(color)
	if not parsed.isValid():
		raise ValueError(f"{field_name} must be a valid color")
	return parsed.name()


#============================================
def _button_foreground(color: str) -> str:
	"""Return readable text for one valid color-button background."""
	return "#ffffff" if PySide6.QtGui.QColor(color).lightness() < 128 else "#000000"


#============================================
class GeometricPropertiesDialog(PySide6.QtWidgets.QDialog):
	"""Edit width, stroke color, and optional fill as detached plain intent.

	The dialog owns only provisional scalar values.  Its caller receives the
	immutable ``changes()`` result and submits that intent through the owning
	document session; this widget never mutates a projection or persistent CDML.
	"""

	#============================================
	def __init__(
			self, title: str, line_width: float, line_color: str,
			area_color: str | None, fillable: bool,
			parent: PySide6.QtWidgets.QWidget | None = None,
			) -> None:
		"""Initialize one focused geometric appearance form."""
		if type(title) is not str or not title.strip():
			raise ValueError("Title must be a nonempty string")
		if type(line_width) not in (int, float) or not math.isfinite(line_width):
			raise ValueError("Stroke width must be finite")
		if not 0.1 <= line_width <= 20.0:
			raise ValueError("Stroke width must be between 0.1 and 20.0")
		if type(fillable) is not bool:
			raise TypeError("Fillable must be a boolean")
		if area_color is not None and type(area_color) is not str:
			raise TypeError("Fill color must be a color string or None")
		super().__init__(parent)
		self._line_color = _validated_color(line_color, "Stroke color")
		self._area_color = _validated_color(area_color or "#ffffff", "Fill color")
		self._fillable = fillable
		self.setWindowTitle(f"{title} Properties")
		self.setMinimumWidth(300)
		layout = PySide6.QtWidgets.QFormLayout(self)
		self._width_spin = PySide6.QtWidgets.QDoubleSpinBox()
		self._width_spin.setRange(0.1, 20.0)
		self._width_spin.setDecimals(3)
		self._width_spin.setSingleStep(0.1)
		self._width_spin.setValue(line_width)
		self._width_spin.setAccessibleName("Stroke width")
		layout.addRow("Stroke width:", self._width_spin)
		self._line_color_button = self._color_button(
			"Stroke color", "Choose the shape or line stroke color", self._pick_line_color,
		)
		layout.addRow("Stroke color:", self._line_color_button)
		self._fill_check: PySide6.QtWidgets.QCheckBox | None = None
		self._area_color_button: PySide6.QtWidgets.QPushButton | None = None
		if fillable:
			self._fill_check = PySide6.QtWidgets.QCheckBox("Fill shape")
			self._fill_check.setChecked(area_color is not None)
			self._fill_check.setAccessibleName("Fill shape")
			self._fill_check.toggled.connect(self._set_fill_enabled)
			layout.addRow("Fill:", self._fill_check)
			self._area_color_button = self._color_button(
				"Fill color", "Choose the shape fill color", self._pick_area_color,
			)
			layout.addRow("Fill color:", self._area_color_button)
			self._set_fill_enabled(self._fill_check.isChecked())
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		buttons.button(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok).setText("Apply")
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addRow(buttons)
		self._refresh_color_buttons()
		self._initial_values = self.get_values()

	#============================================
	def _color_button(
			self, name: str, tooltip: str, callback: collections.abc.Callable[[], None],
			) -> PySide6.QtWidgets.QPushButton:
		"""Build one labeled, keyboard-focusable color picker."""
		button = PySide6.QtWidgets.QPushButton()
		button.setMinimumHeight(28)
		button.setAccessibleName(name)
		button.setToolTip(tooltip)
		button.clicked.connect(callback)
		return button

	#============================================
	def _pick_line_color(self) -> None:
		"""Choose a stroke color without changing persistent state."""
		color = PySide6.QtWidgets.QColorDialog.getColor(
			PySide6.QtGui.QColor(self._line_color), self, "Stroke Color",
		)
		if color.isValid():
			self._line_color = color.name()
			self._refresh_color_buttons()

	#============================================
	def _pick_area_color(self) -> None:
		"""Choose a fill color without changing persistent state."""
		color = PySide6.QtWidgets.QColorDialog.getColor(
			PySide6.QtGui.QColor(self._area_color), self, "Fill Color",
		)
		if color.isValid():
			self._area_color = color.name()
			self._refresh_color_buttons()

	#============================================
	def _set_fill_enabled(self, enabled: bool) -> None:
		"""Expose fill color only while Fill shape is selected."""
		if self._area_color_button is not None:
			self._area_color_button.setEnabled(enabled)

	#============================================
	def _refresh_color_buttons(self) -> None:
		"""Show color values as readable text as well as background swatches."""
		self._style_color_button(self._line_color_button, self._line_color)
		if self._area_color_button is not None:
			self._style_color_button(self._area_color_button, self._area_color)

	#============================================
	def _style_color_button(
			self, button: PySide6.QtWidgets.QPushButton, color: str,
			) -> None:
		"""Apply one accessible text-and-swatch representation to a button."""
		button.setText(color)
		foreground = _button_foreground(color)
		button.setStyleSheet(
			f"background-color: {color}; color: {foreground}; "
			"border: 1px solid #888;",
		)

	#============================================
	def get_values(self) -> dict[str, float | str | None]:
		"""Return every plain editable appearance value."""
		values: dict[str, float | str | None] = {
			"line_width": self._width_spin.value(),
			"line_color": self._line_color,
		}
		if self._fillable:
			if self._fill_check is None:
				raise RuntimeError("Fill control is unavailable")
			values["area_color"] = (
				self._area_color if self._fill_check.isChecked() else None
			)
		return values

	#============================================
	def changes(self) -> tuple[tuple[str, float | str | None], ...]:
		"""Return only explicit validated values changed after initialization."""
		values = self.get_values()
		changes = tuple(
			(name, value) for name, value in values.items()
			if value != self._initial_values[name]
		)
		return changes
