"""Bond properties dialog."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bond_presentation


# -- bond order labels indexed by value --
_ORDER_LABELS = {
	1: "Single",
	2: "Double",
	3: "Triple",
}
_ORDER_VALUES = {v: k for k, v in _ORDER_LABELS.items()}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class BondDialogCapabilities:
	"""Optional visual limits for a route with a smaller rendering vocabulary."""

	normal_style_only: bool = False
	wedge_width_available: bool = True
	dynamic_native_widths: bool = False
	allowed_style_chars: tuple[str, ...] = ()


#============================================
NATIVE_RENDER_CAPABILITIES = BondDialogCapabilities(
	allowed_style_chars=("n", "w", "h"),
	wedge_width_available=True,
	dynamic_native_widths=True,
)


#============================================
class BondDialog(PySide6.QtWidgets.QDialog):
	"""Dialog for editing bond properties.

	Presents a form with fields for bond order, type, centering,
	line width, bond width, wedge width, and color.

	Args:
		bond_model: The BondModel whose properties to edit.
		parent: Optional parent widget.
	"""

	#============================================
	def __init__(
			self, bond_model: object, parent: object | None = None,
			capabilities: BondDialogCapabilities | None = None,
			) -> None:
		"""Initialize a detached bond-properties value editor.

		Args:
			bond_model: The BondModel whose properties to edit.
			parent: Optional parent widget.
			capabilities: Optional route-specific visual limits.  Omitted preserves
				the complete legacy form.
		"""
		super().__init__(parent)
		self._capabilities = capabilities or BondDialogCapabilities()
		# Copy every display scalar before constructing Qt controls.  The accepted
		# backend patch can replace the projected BondModel while this dialog still
		# exists, so it must never retain or later inspect that transient wrapper.
		self._initial_values = {
			"order": bond_model.order,
			"type": bond_model.type,
			"center": bool(bond_model.center),
			"line_width": bond_model.line_width,
			"bond_width": bond_model.bond_width,
			"wedge_width": bond_model.wedge_width,
			"color": bond_model.line_color,
		}
		self._color = self._initial_values["color"]
		self.setWindowTitle("Bond Properties")
		self.setMinimumWidth(300)
		self._build_ui()
		self._populate_from_model()

	#============================================
	def _build_ui(self) -> None:
		"""Build the form layout with all property fields."""
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		form = PySide6.QtWidgets.QFormLayout()

		# order
		self._order_combo = PySide6.QtWidgets.QComboBox()
		for label in _ORDER_LABELS.values():
			self._order_combo.addItem(label)
		form.addRow("Order:", self._order_combo)

		# type
		self._type_combo = PySide6.QtWidgets.QComboBox()
		for type_char, label in self._type_choices():
			self._type_combo.addItem(label, type_char)
		form.addRow("Type:", self._type_combo)

		# center double bond
		self._center_check = PySide6.QtWidgets.QCheckBox()
		form.addRow("Center double bond:", self._center_check)

		# line width
		self._line_width_spin = PySide6.QtWidgets.QDoubleSpinBox()
		self._line_width_spin.setRange(0.1, 20.0)
		self._line_width_spin.setSingleStep(0.5)
		self._line_width_spin.setDecimals(1)
		form.addRow("Line width:", self._line_width_spin)

		# bond width
		self._bond_width_spin = PySide6.QtWidgets.QDoubleSpinBox()
		self._bond_width_spin.setRange(0.1, 40.0)
		self._bond_width_spin.setSingleStep(0.5)
		self._bond_width_spin.setDecimals(1)
		form.addRow("Bond width:", self._bond_width_spin)

		# wedge width
		self._wedge_width_spin = PySide6.QtWidgets.QDoubleSpinBox()
		self._wedge_width_spin.setRange(0.1, 40.0)
		self._wedge_width_spin.setSingleStep(0.5)
		self._wedge_width_spin.setDecimals(1)
		form.addRow("Wedge width:", self._wedge_width_spin)

		# color button
		self._color_button = PySide6.QtWidgets.QPushButton()
		self._color_button.setFixedHeight(24)
		self._color_button.clicked.connect(self._pick_color)
		form.addRow("Color:", self._color_button)

		layout.addLayout(form)

		# ok / cancel buttons
		button_box = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel
		)
		button_box.accepted.connect(self.accept)
		button_box.rejected.connect(self.reject)
		layout.addWidget(button_box)
		self._order_combo.currentIndexChanged.connect(self._refresh_capability_controls)
		self._configure_static_capability_controls()

	#============================================
	def _type_choices(self) -> tuple[tuple[str, str], ...]:
		"""Return only the bond-style choices available on this visual route."""
		choices = ferrum_qt.bond_presentation.choices_for_display(
			self._initial_values["type"],
		)
		if self._capabilities.normal_style_only:
			return tuple(choice for choice in choices if choice[0] == "n")
		if self._capabilities.allowed_style_chars:
			return tuple(
				choice for choice in choices
				if choice[0] in self._capabilities.allowed_style_chars
			)
		return choices

	#============================================
	def _configure_static_capability_controls(self) -> None:
		"""Make unsupported route features unavailable before the user submits."""
		if self._capabilities.normal_style_only:
			self._type_combo.setEnabled(False)
			self._type_combo.setToolTip(
				"Native rendering currently supports Normal bond style only.",
			)
		if not self._capabilities.wedge_width_available:
			self._wedge_width_spin.setEnabled(False)
			self._wedge_width_spin.setToolTip(
				"Native wedge rendering is not available, so wedge width cannot be edited.",
			)
		self._refresh_capability_controls()

	#============================================
	def _refresh_capability_controls(self) -> None:
		"""Keep order-dependent native controls honest as the user changes order."""
		if not self._capabilities.dynamic_native_widths:
			return
		order = _ORDER_VALUES.get(self._order_combo.currentText(), 1)
		center_available = order == 2
		bond_width_available = order in (2, 3)
		self._center_check.setEnabled(center_available)
		self._bond_width_spin.setEnabled(bond_width_available)
		if not center_available:
			self._center_check.setToolTip(
				"Native rendering supports centering only for a double bond.",
			)
		else:
			self._center_check.setToolTip("")
		if not bond_width_available:
			self._bond_width_spin.setToolTip(
				"Native rendering uses bond width only for double and triple bonds.",
			)
		else:
			self._bond_width_spin.setToolTip("")

	#============================================
	def _populate_from_model(self) -> None:
		"""Fill dialog fields from the current bond model values."""
		# order
		order_label = _ORDER_LABELS.get(self._initial_values["order"], "Single")
		idx = self._order_combo.findText(order_label)
		if idx >= 0:
			self._order_combo.setCurrentIndex(idx)
		# type
		idx = self._type_combo.findData(self._initial_values["type"])
		if idx >= 0:
			self._type_combo.setCurrentIndex(idx)
		# center
		center_val = self._initial_values["center"]
		self._center_check.setChecked(bool(center_val))
		# widths
		self._line_width_spin.setValue(self._initial_values["line_width"])
		self._bond_width_spin.setValue(self._initial_values["bond_width"])
		self._wedge_width_spin.setValue(self._initial_values["wedge_width"])
		# color
		self._color = self._initial_values["color"]
		self._update_color_button()

	#============================================
	def _pick_color(self) -> None:
		"""Open a color picker dialog and update the color button."""
		color = PySide6.QtWidgets.QColorDialog.getColor(
			PySide6.QtGui.QColor(self._color), self, "Bond Color"
		)
		if color.isValid():
			self._color = color.name()
			self._update_color_button()

	#============================================
	def _update_color_button(self) -> None:
		"""Set the color button background to the currently selected color."""
		self._color_button.setStyleSheet(
			f"background-color: {self._color}; border: 1px solid #888;"
		)

	#============================================
	def get_values(self) -> dict:
		"""Return dict of edited values.

		Returns:
			Dictionary mapping property names to their new values.
		"""
		order_label = self._order_combo.currentText()
		order_val = _ORDER_VALUES.get(order_label, 1)
		type_val = self._type_combo.currentData()
		values = {
			"order": order_val,
			"type": type_val,
			"center": self._center_check.isChecked(),
			"line_width": self._line_width_spin.value(),
			"bond_width": self._bond_width_spin.value(),
			"wedge_width": self._wedge_width_spin.value(),
			"line_color": self._color,
		}
		return values

	#============================================
	def changes(self) -> tuple[tuple[str, object], ...]:
		"""Return only deliberate value changes using backend CDML field names."""
		values = self.get_values()
		current = {
			"order": values["order"], "type": values["type"],
			"center": values["center"], "line_width": values["line_width"],
			"bond_width": values["bond_width"], "wedge_width": values["wedge_width"],
			"color": values["line_color"],
		}
		return tuple(
			(name, value) for name, value in current.items()
			if value != self._initial_values[name]
		)

	#============================================
	@staticmethod
	def edit_bond(bond_model: object, parent: object | None = None) -> bool:
		"""Convenience: show dialog, apply changes if accepted.

		Args:
			bond_model: The BondModel to edit.
			parent: Optional parent widget.

		Returns:
			True if changes were accepted and applied, False otherwise.
		"""
		dialog = BondDialog(bond_model, parent)
		result = dialog.exec()
		if result != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return False
		values = dialog.get_values()
		for key, value in values.items():
			setattr(bond_model, key, value)
		return bool(dialog.changes())
