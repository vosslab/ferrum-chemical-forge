"""Rust-authoritative renderable drawing defaults for Ferrum documents."""

# Standard Library
import dataclasses
import math
import re

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog


_COLOR = re.compile(r"#[0-9A-Fa-f]{3}(?:[0-9A-Fa-f]{3})?")


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeDrawingStandardModel:
	"""The seven document defaults honored by the current Ferrum renderer."""

	line_width: float
	font_size: int
	line_color: str
	area_color: str
	bond_width: float
	wedge_width: float
	show_hydrogens: bool


#============================================
class FerrumNativeDrawingStandardTabMixin:
	"""Document-global drawing-standard observation and mutation for one tab."""

	#============================================
	def drawing_standard_projection(self) -> object | None:
		"""Return the current frozen Rust standard after provenance checks."""
		self._require_mutable()
		if self._document_observation is None:
			raise RuntimeError("Ferrum tab has no installed document projection")
		projection = self._document_observation.projection
		if projection.revision != self.current_snapshot.revision:
			raise RuntimeError("drawing-standard revision does not match the snapshot")
		if projection.digest != self.current_snapshot.digest:
			raise RuntimeError("drawing-standard digest does not match the snapshot")
		return projection.drawing_standard

	#============================================
	def apply_drawing_standard(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed document-global Rust drawing-standard patch."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("Ferrum drawing-standard changes require an exact tuple")
		import ferrum_qt.ferrum.engine as engine
		if any(
				type(change) is not engine.DocumentDrawingStandardPropertyChangeV1
				for change in changes
				):
			raise TypeError("Ferrum drawing-standard changes require frozen Ferrum values")
		operation = engine.DocumentOperationV1.set_drawing_standard(changes)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result)
		return result


#============================================
class FerrumNativeDrawingStandardDialog(FerrumAccessibleDialog):
	"""Collect explicit changes to renderer-supported document defaults."""

	#============================================
	def __init__(self, model: FerrumNativeDrawingStandardModel,
			parent: PySide6.QtWidgets.QWidget | None = None) -> None:
		"""Build a focused form from one immutable Rust-derived model."""
		super().__init__(parent)
		self._initial = dataclasses.asdict(model)
		self.setWindowTitle("Document Drawing Defaults")
		self.setMinimumWidth(460)
		self._build_ui(model)

	#============================================
	def _build_ui(self, model: FerrumNativeDrawingStandardModel) -> None:
		"""Build controls only for defaults consumed by the current renderer."""
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		intro = PySide6.QtWidgets.QLabel(
			"Set defaults used for future and unstyled document objects. Existing "
			"explicit object styles remain unchanged.",
		)
		intro.setWordWrap(True)
		layout.addWidget(intro)
		form = PySide6.QtWidgets.QFormLayout()
		self._line_width = _width_editor("Default line width", model.line_width)
		form.addRow("Line width:", self._line_width)
		self._font_size = PySide6.QtWidgets.QSpinBox()
		self._font_size.setAccessibleName("Default atom-label font size")
		self._font_size.setRange(4, 144)
		self._font_size.setSuffix(" pt")
		self._font_size.setValue(model.font_size)
		form.addRow("Atom-label size:", self._font_size)
		self._line_color = PySide6.QtWidgets.QLineEdit(model.line_color)
		self._line_color.setAccessibleName("Default line and text color")
		self._line_color.setPlaceholderText("#000000")
		form.addRow("Line and text color:", self._line_color)
		self._area_color = PySide6.QtWidgets.QLineEdit(model.area_color)
		self._area_color.setAccessibleName("Default label background color")
		self._area_color.setPlaceholderText("Empty means transparent")
		form.addRow("Label background:", self._area_color)
		self._bond_width = _width_editor("Default multiple-bond spacing", model.bond_width)
		form.addRow("Multiple-bond spacing:", self._bond_width)
		self._wedge_width = _width_editor("Default wedge width", model.wedge_width)
		form.addRow("Wedge width:", self._wedge_width)
		self._show_hydrogens = PySide6.QtWidgets.QCheckBox(
			"Show hydrogens on heteroatoms by default",
		)
		self._show_hydrogens.setAccessibleName("Show heteroatom hydrogens by default")
		self._show_hydrogens.setChecked(model.show_hydrogens)
		form.addRow("Hydrogens:", self._show_hydrogens)
		layout.addLayout(form)
		self._error = PySide6.QtWidgets.QLabel()
		self._error.setWordWrap(True)
		self._error.setStyleSheet("color: #b00020;")
		layout.addWidget(self._error)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok
			| PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)

	#============================================
	def accept(self) -> None:
		"""Keep invalid color text visible with one actionable explanation."""
		line_color = self._line_color.text().strip()
		area_color = self._area_color.text().strip()
		if _COLOR.fullmatch(line_color) is None:
			self._error.setText("Line and text color must look like #224466.")
			self._line_color.setFocus()
			return
		if area_color and _COLOR.fullmatch(area_color) is None:
			self._error.setText(
				"Label background must be a hexadecimal color or empty for transparent.",
			)
			self._area_color.setFocus()
			return
		self._error.clear()
		super().accept()

	#============================================
	def changes(self) -> tuple[tuple[str, object], ...]:
		"""Return only fields intentionally changed by the accepted form."""
		values = (
			("line_width", self._line_width.value()),
			("font_size", self._font_size.value()),
			("line_color", self._line_color.text().strip()),
			("area_color", self._area_color.text().strip()),
			("bond_width", self._bond_width.value()),
			("wedge_width", self._wedge_width.value()),
			("show_hydrogens", self._show_hydrogens.isChecked()),
		)
		return tuple(
			(field, value) for field, value in values
			if value != self._initial[field]
		)


#============================================
def model_from_projection(standard: object | None) -> FerrumNativeDrawingStandardModel:
	"""Resolve authored values over the exact current Ferrum fallback profile."""
	import ferrum_qt.ferrum.engine as engine
	if standard is not None and type(standard) is not engine.DrawingStandardV1:
		raise TypeError("Ferrum drawing standard requires exact frozen Ferrum facts")
	values = {
		"line_width": 1.0,
		"font_size": 12.0,
		"line_color": "#000000",
		"area_color": "",
		"bond_width": 6.0,
		"wedge_width": 5.0,
		"show_hydrogens": False,
	}
	if standard is not None:
		for field in values:
			value = getattr(standard, field)
			if value is not None:
				values[field] = value
	_validate_model(values)
	return FerrumNativeDrawingStandardModel(**values)


#============================================
def _validate_model(values: dict[str, object]) -> None:
	"""Reject source facts the closed Qt controls would silently coerce."""
	for field in ("line_width", "bond_width", "wedge_width"):
		value = values[field]
		if type(value) is not float or not math.isfinite(value) or not 0.0 < value <= 1000.0:
			raise ValueError(f"document {field} is not representable by the drawing form")
	font_size = values["font_size"]
	if (
			type(font_size) is not float or not font_size.is_integer()
			or not 4 <= font_size <= 144
			):
		raise ValueError("document font size is not representable by the drawing form")
	values["font_size"] = int(font_size)
	for field in ("line_color", "area_color"):
		value = values[field]
		if type(value) is not str or (value and _COLOR.fullmatch(value) is None):
			raise ValueError(f"document {field} is not representable by the drawing form")
	if type(values["show_hydrogens"]) is not bool:
		raise ValueError("document hydrogen default is not representable by the drawing form")


#============================================
def changes_from_dialog(changes: tuple[tuple[str, object], ...]) -> tuple[object, ...]:
	"""Map exact explicit form intent to frozen Rust drawing-standard changes."""
	if type(changes) is not tuple:
		raise TypeError("Ferrum drawing-standard dialog changes must be an exact tuple")
	import ferrum_qt.ferrum.engine as engine
	change_type = engine.DocumentDrawingStandardPropertyChangeV1
	converted: list[object] = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("Ferrum drawing-standard changes must be exact field/value pairs")
		field, value = change
		factory = getattr(change_type, field, None)
		if factory is None or field not in {
				"line_width", "font_size", "line_color", "area_color",
				"bond_width", "wedge_width", "show_hydrogens",
				}:
			raise ValueError("Ferrum drawing-standard dialog supplied an unsupported field")
		converted.append(factory(value))
	return tuple(converted)


#============================================
def install_drawing_standard_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install the renderer-supported drawing-default action."""
	action = PySide6.QtGui.QAction(window.tr("Document Drawing Defaults..."), window)
	action.setToolTip(window.tr(
		"Edit document drawing defaults through one revision-bound Rust operation",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_drawing_standard(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_drawing_standard_action(action: PySide6.QtGui.QAction,
		active: bool, pending: bool, busy: bool) -> None:
	"""Disable editing while the Ferrum tab cannot accept one operation."""
	action.setEnabled(active and not pending and not busy)


#============================================
def _on_edit_drawing_standard(window: object) -> None:
	"""Collect renderable defaults and submit one closed Rust patch."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		model = model_from_projection(tab.drawing_standard_projection())
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	dialog = FerrumNativeDrawingStandardDialog(model, window)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = changes_from_dialog(dialog.changes())
		if not changes:
			window.statusBar().showMessage(window.tr("Document drawing defaults are unchanged."), 3000)
			return
		tab.apply_drawing_standard(changes)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Updated drawing defaults."), 5000)
	window._refresh_actions()


#============================================
def _width_editor(accessible_name: str, value: float) -> PySide6.QtWidgets.QDoubleSpinBox:
	"""Build one bounded scene-point width editor."""
	editor = PySide6.QtWidgets.QDoubleSpinBox()
	editor.setAccessibleName(accessible_name)
	editor.setRange(0.01, 1000.0)
	editor.setDecimals(2)
	editor.setSingleStep(0.25)
	editor.setSuffix(" pt")
	editor.setValue(value)
	return editor
