"""Rust-authoritative paper-properties adaptation for the native editor."""

# Standard Library
import dataclasses
import math

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.paper_catalog
import ferrum_qt.dialogs.paper_properties_dialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativePaperDialogModel:
	"""Plain form values copied from one frozen Rust paper projection."""

	attributes: dict[str, str]
	default_type: str
	default_orientation: str


#============================================
class FerrumNativePaperPropertiesMixin:
	"""Document-global paper observation and mutation for a native tab."""

	#============================================
	def paper_layout_projection(self) -> object:
		"""Return the current exact frozen Rust paper layout."""
		self._require_mutable()
		if self._document_observation is None:
			raise RuntimeError("native tab has no installed document projection")
		layout = self._document_observation.projection.paper_layout
		if layout.revision != self.current_snapshot.revision:
			raise RuntimeError("paper layout revision does not match the installed snapshot")
		if layout.digest != self.current_snapshot.digest:
			raise RuntimeError("paper layout digest does not match the installed snapshot")
		return layout

	#============================================
	def apply_paper_properties(self, changes: tuple[object, ...]) -> object:
		"""Commit one closed document-global Rust paper patch."""
		self._require_mutable()
		if type(changes) is not tuple:
			raise TypeError("native paper properties require an exact change tuple")
		import ferrum_chem
		if any(type(change) is not ferrum_chem.DocumentPaperPropertyChangeV1
				for change in changes):
			raise TypeError("native paper properties require exact frozen Ferrum changes")
		operation = ferrum_chem.DocumentOperationV1.set_paper_properties(changes)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result)
		return result


#============================================
def dialog_model_from_projection(layout: object) -> FerrumNativePaperDialogModel:
	"""Copy only named paper fields without interpreting persistent XML in Qt."""
	import ferrum_chem
	if type(layout) is not ferrum_chem.PaperLayoutProjectionV1:
		raise TypeError("native paper properties require an exact Ferrum paper layout")
	attributes = layout.paper_attributes
	if type(attributes) is not ferrum_chem.PaperAttributesV1:
		raise TypeError("native paper properties require exact Ferrum paper attributes")
	values = {
		"id": attributes.id,
		"type": attributes.type_name,
		"orientation": attributes.orientation,
		"crop_svg": attributes.crop_svg,
		"crop_margin": attributes.crop_margin,
		"use_real_minus": attributes.use_real_minus,
		"replace_minus": attributes.replace_minus,
		"size_x": attributes.size_x,
		"size_y": attributes.size_y,
	}
	if any(value is not None and type(value) is not str for value in values.values()):
		raise TypeError("native paper attributes must be strings or absent")
	_validate_representable_attributes(values)
	default_type = layout.default_type
	if type(default_type) is not str:
		raise TypeError("native paper default type must be a string")
	if layout.default_orientation == ferrum_chem.PaperOrientationV1.portrait:
		default_orientation = "portrait"
	elif layout.default_orientation == ferrum_chem.PaperOrientationV1.landscape:
		default_orientation = "landscape"
	else:
		raise ValueError("native paper default orientation is unsupported")
	return FerrumNativePaperDialogModel(
		{key: value for key, value in values.items() if value is not None},
		default_type,
		default_orientation,
	)


#============================================
def _validate_representable_attributes(values: dict[str, str | None]) -> None:
	"""Reject authored facts the existing controls would silently coerce."""
	for field in ("crop_svg", "use_real_minus", "replace_minus"):
		value = values[field]
		if value is not None and value.strip().lower() not in (
				"0", "1", "false", "true", "no", "yes", "off", "on",
				):
			raise ValueError(f"selected Rust paper {field} is not representable by the form")
	margin = values["crop_margin"]
	if margin is not None and (
			not margin.isdecimal() or int(margin) > 2147483647
			):
		raise ValueError("selected Rust paper crop margin is not representable by the form")
	if values["type"] != "custom":
		return
	for field in ("size_x", "size_y"):
		value = values[field]
		try:
			number = float(value) if value is not None else math.nan
		except ValueError as exc:
			raise ValueError(
				f"selected Rust custom paper {field} is not representable by the form",
			) from exc
		if (
			not math.isfinite(number) or number <= 0.0 or number > 1000000000000.0
			or round(number, 1) != number
		):
			raise ValueError(
				f"selected Rust custom paper {field} is not representable by the form",
			)


#============================================
def property_changes_from_dialog(
		changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map exact explicit form intent to frozen Rust paper changes."""
	import ferrum_chem
	if type(changes) is not tuple:
		raise TypeError("native paper property changes must be an exact tuple")
	converted: list[object] = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("native paper changes must be exact field/value pairs")
		field, value = change
		if field == "type" and type(value) is str:
			converted.append(ferrum_chem.DocumentPaperPropertyChangeV1.type_name(value))
		elif field == "orientation" and value == "portrait":
			converted.append(ferrum_chem.DocumentPaperPropertyChangeV1.orientation(
				ferrum_chem.PaperOrientationV1.portrait,
			))
		elif field == "orientation" and value == "landscape":
			converted.append(ferrum_chem.DocumentPaperPropertyChangeV1.orientation(
				ferrum_chem.PaperOrientationV1.landscape,
			))
		elif field in ("crop_svg", "use_real_minus", "replace_minus") \
				and type(value) is bool:
			factory = getattr(ferrum_chem.DocumentPaperPropertyChangeV1, field)
			converted.append(factory(value))
		elif field == "crop_margin" and type(value) is int and value >= 0:
			converted.append(ferrum_chem.DocumentPaperPropertyChangeV1.crop_margin(value))
		elif field == "dimensions" and type(value) is tuple and len(value) == 2:
			width, height = value
			if any(type(dimension) is not float for dimension in (width, height)):
				raise TypeError("native custom paper dimensions must be exact floats")
			converted.append(
				ferrum_chem.DocumentPaperPropertyChangeV1.dimensions(width, height),
			)
		else:
			raise ValueError("native paper dialog supplied an unsupported property change")
	return tuple(converted)


#============================================
def install_paper_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install the document-global native paper action outside the host class."""
	action = PySide6.QtGui.QAction(window.tr("Document Properties"), window)
	action.setToolTip(window.tr(
		"Edit paper properties through one revision-bound Rust operation",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_paper_properties(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_paper_properties_action(action: PySide6.QtGui.QAction,
		active: bool, pending: bool, busy: bool) -> None:
	"""Disable paper editing while the native tab cannot accept an operation."""
	action.setEnabled(active and not pending and not busy)


#============================================
def _on_edit_paper_properties(window: object) -> None:
	"""Run the existing intent-only form and submit one closed Rust patch."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		model = dialog_model_from_projection(tab.paper_layout_projection())
		catalog = ferrum_qt.bridge.paper_catalog.paper_catalog_v1()
	except Exception as exc:
		window._refresh_actions()
		window._show_native_file_warning("Native Document Properties Unavailable", str(exc))
		return
	dialog = ferrum_qt.dialogs.paper_properties_dialog.PaperPropertiesDialog(
		model.attributes, catalog, model.default_type, model.default_orientation, window,
	)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = property_changes_from_dialog(dialog.changes())
		tab.apply_paper_properties(changes)
	except Exception as exc:
		window._refresh_actions()
		window._show_native_file_warning("Native Document Properties Error", str(exc))
		return
	window.statusBar().showMessage(window.tr("Updated Rust-native paper properties."), 5000)
	window._refresh_actions()
