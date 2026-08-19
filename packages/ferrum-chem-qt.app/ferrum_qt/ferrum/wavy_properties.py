"""Typed Wavy appearance editing for the Ferrum window."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.wavy_dialog


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeWavyDialogModel:
	"""Appearance facts copied from one exact frozen Rust Wavy root."""

	target_id: str
	source_id: str
	line_width: float
	line_color: str


#============================================
def dialog_model_from_projection(root: object) -> FerrumNativeWavyDialogModel:
	"""Copy only Wavy facts exactly representable by the detached form."""
	import ferrum_qt.ferrum.engine as engine
	if type(root) is not engine.PresentationRootProjectionV1:
		raise TypeError("Ferrum Wavy properties require an exact Ferrum root")
	if root.kind != "wavy" or type(root.polyline) is not engine.PolylineProjectionV1:
		raise ValueError("selected Rust presentation is not an editable Wavy line")
	if root.polyline.target.record_kind != "polyline":
		raise ValueError("selected Rust Wavy target kind is inconsistent")
	if (
		type(root.polyline.target.id) is not str
		or type(root.polyline.target.source_id) is not str
	):
		raise ValueError("Ferrum Wavy properties require a durable authored target")
	width = root.polyline.stroke.width
	if (
		type(width) is not float
		or width < 0.1
		or width > 20.0
		or round(width, 3) != width
	):
		raise ValueError("selected Rust Wavy width is not representable by the current form")
	color = root.polyline.stroke.color
	if type(color) is not str:
		raise TypeError("selected Rust Wavy line color must be a string")
	return FerrumNativeWavyDialogModel(
		root.polyline.target.id,
		root.polyline.target.source_id,
		width,
		color,
	)


#============================================
def property_changes_from_dialog(
		changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map only exact detached-form intent to frozen Rust Wavy changes."""
	if type(changes) is not tuple:
		raise TypeError("Ferrum Wavy property changes must be an exact tuple")
	import ferrum_qt.ferrum.engine as engine
	converted: list[object] = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("Ferrum Wavy property changes must be exact field/value pairs")
		field, value = change
		if field == "width" and type(value) is float:
			converted.append(
				engine.DocumentWavyPropertyChangeV1.line_width(value),
			)
		elif field == "line_color" and type(value) is str:
			converted.append(
				engine.DocumentWavyPropertyChangeV1.line_color(value),
			)
		else:
			raise ValueError("Ferrum Wavy form supplied an unsupported property change")
	return tuple(converted)


#============================================
class FerrumNativeWavyPropertiesMixin:
	"""Durable selection and mutation methods mixed into the Ferrum document tab."""

	#============================================
	def has_one_selected_wavy(self) -> bool:
		"""Return whether one rendered durable Wavy root is selected."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			self.selected_wavy_projection()
		except (RuntimeError, TypeError, ValueError):
			return False
		return True

	#============================================
	def selected_wavy_projection(self) -> object:
		"""Return the selected exact frozen Rust Wavy root projection."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if len(selected) != 1 or selected[0].kind != "polyline":
			raise RuntimeError("select exactly one Wavy presentation first")
		if selected[0].identifier is None or self._document_observation is None:
			raise RuntimeError("selected Wavy line has no current durable projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind != "wavy":
				continue
			model = dialog_model_from_projection(root)
			if model.target_id == selected[0].identifier:
				return root
		raise RuntimeError("selected Wavy line is absent from the Rust projection")

	#============================================
	def apply_selected_wavy_properties(self, expected_target_id: str,
			expected_source_id: str, changes: tuple[object, ...]) -> object:
		"""Commit one closed Wavy patch without allowing selection retargeting."""
		self._require_mutable()
		if type(expected_target_id) is not str or type(expected_source_id) is not str:
			raise TypeError("Ferrum Wavy properties require durable string identifiers")
		if type(changes) is not tuple:
			raise TypeError("Ferrum Wavy properties require an exact change tuple")
		root = self.selected_wavy_projection()
		model = dialog_model_from_projection(root)
		if model.target_id != expected_target_id or model.source_id != expected_source_id:
			raise RuntimeError("selected Wavy line changed while its properties form was open")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentWavyPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum Wavy properties require exact frozen Ferrum changes")
		operation = engine.DocumentOperationV1.set_wavy_properties(
			expected_source_id, changes,
		)
		result = self._session.submit(self.current_snapshot.revision, operation)
		self._install_mutation_result(result, (("polyline", model.target_id),))
		return result


#============================================
def install_wavy_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one Ferrum Wavy-properties action on the public window."""
	action = PySide6.QtGui.QAction(window.tr("Edit Wavy Properties"), window)
	action.setToolTip(window.tr(
		"Edit one selected Wavy line through one operation",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_wavy_properties(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_wavy_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Make the action follow exact current durable Wavy selection."""
	action.setEnabled(
		active and not pending and not busy and tab.has_one_selected_wavy(),
	)


#============================================
def _on_edit_wavy_properties(window: object) -> None:
	"""Run one detached Wavy form and submit only changed fields."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		root = tab.selected_wavy_projection()
		model = dialog_model_from_projection(root)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	dialog = ferrum_qt.dialogs.wavy_dialog.WavyDialog(
		model.line_width, model.line_color, window,
	)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		changes = property_changes_from_dialog(dialog.changes())
		tab.apply_selected_wavy_properties(
			model.target_id, model.source_id, changes,
		)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Updated one Ferrum Wavy line."), 5000)
	window._refresh_actions()
