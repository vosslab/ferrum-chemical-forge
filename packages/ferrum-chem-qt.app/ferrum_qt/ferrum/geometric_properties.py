"""Typed geometric appearance editing for the Ferrum window."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.geometric_properties_dialog


_GEOMETRIC_KINDS = frozenset((
	"rectangle", "square", "oval", "circle", "polygon", "polyline",
))
_BOX_KINDS = frozenset(("rectangle", "square", "oval", "circle"))
_TITLES = {
	"rectangle": "Rectangle",
	"square": "Square",
	"oval": "Oval",
	"circle": "Circle",
	"polygon": "Polygon",
	"polyline": "Polyline",
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeGeometricDialogModel:
	"""Appearance values copied from one exact frozen Rust projection root."""

	target_id: str
	source_id: str
	kind: str
	title: str
	line_width: float
	line_color: str
	area_color: str | None
	fillable: bool


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeBracketDialogModel:
	"""Common appearance and durable members copied from one Rust bracket pair."""

	pair_id: str
	member_target_ids: tuple[str, str]
	member_source_ids: tuple[str, str]
	title: str
	line_width: float
	line_color: str
	area_color: None
	fillable: bool


#============================================
def dialog_model_from_projection(root: object) -> FerrumNativeGeometricDialogModel:
	"""Copy only facts exactly representable by the detached geometric form."""
	import ferrum_qt.ferrum.engine as engine
	if type(root) is not engine.PresentationRootProjectionV1:
		raise TypeError("Ferrum geometric properties require an exact Ferrum root")
	if root.kind not in _GEOMETRIC_KINDS:
		raise ValueError("selected Rust presentation is not editable geometry")
	if root.kind in _BOX_KINDS:
		payload = root.shape
		expected_type = engine.BoxShapeProjectionV1
	elif root.kind == "polygon":
		payload = root.polygon
		expected_type = engine.PolygonProjectionV1
	else:
		payload = root.polyline
		expected_type = engine.PolylineProjectionV1
	if type(payload) is not expected_type:
		raise TypeError("selected Rust geometric payload does not match its root kind")
	if payload.target.record_kind != root.kind:
		raise ValueError("selected Rust geometric target kind is inconsistent")
	if type(payload.target.id) is not str or type(payload.target.source_id) is not str:
		raise ValueError("Ferrum geometric properties require a durable authored target")
	width = payload.stroke.width
	if (
		type(width) is not float
		or width < 0.1
		or width > 20.0
		or round(width, 3) != width
	):
		raise ValueError(
			"selected Rust geometric width is not representable by the current form",
		)
	color = payload.stroke.color
	if type(color) is not str:
		raise TypeError("selected Rust geometric stroke color must be a string")
	fillable = root.kind != "polyline"
	area_color = payload.fill.color if fillable else None
	if area_color is not None and type(area_color) is not str:
		raise TypeError("selected Rust geometric fill color must be a string or None")
	return FerrumNativeGeometricDialogModel(
		payload.target.id,
		payload.target.source_id,
		root.kind,
		_TITLES[root.kind],
		width,
		color,
		area_color,
		fillable,
	)


#============================================
def property_changes_from_dialog(
		root: object,
		changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map only exact detached-form intent to frozen Rust property changes."""
	model = dialog_model_from_projection(root)
	if type(changes) is not tuple:
		raise TypeError("Ferrum geometric property changes must be an exact tuple")
	import ferrum_qt.ferrum.engine as engine
	converted: list[object] = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError(
				"Ferrum geometric property changes must be exact field/value pairs",
			)
		field, value = change
		if field == "line_width" and type(value) is float:
			converted.append(
				engine.DocumentGeometricPropertyChangeV1.line_width(value),
			)
		elif field == "line_color" and type(value) is str:
			converted.append(
				engine.DocumentGeometricPropertyChangeV1.stroke_color(value),
			)
		elif field == "area_color" and model.fillable and (
				value is None or type(value) is str):
			converted.append(
				engine.DocumentGeometricPropertyChangeV1.fill_color(value),
			)
		else:
			raise ValueError("Ferrum geometric form supplied an unsupported property change")
	return tuple(converted)


#============================================
def bracket_dialog_model_from_projection(
		pair: object, roots: list[object],
		) -> FerrumNativeBracketDialogModel:
	"""Copy one exact complete pair only when the shared form preserves its facts."""
	import ferrum_qt.ferrum.engine as engine
	if type(pair) is not engine.BracketPairProjectionV1:
		raise TypeError("Ferrum bracket properties require an exact Ferrum pair")
	if (
			type(pair.pair_id) is not str
			or type(pair.member_ids) is not list
			or len(pair.member_ids) != 2
			or any(type(identifier) is not str for identifier in pair.member_ids)
		):
		raise ValueError("selected Rust bracket pair has invalid durable identity")
	width = pair.line_width
	if (
			type(width) is not float
			or width < 0.1
			or width > 20.0
			or round(width, 3) != width
		):
		raise ValueError("selected Rust bracket width is not representable by the current form")
	if type(pair.line_color) is not str:
		raise ValueError("selected Rust bracket has no common editable line color")
	members: dict[str, str] = {}
	for root in roots:
		if type(root) is not engine.PresentationRootProjectionV1:
			raise TypeError("Ferrum bracket properties require exact Ferrum roots")
		if root.kind != "polyline" or root.polyline is None:
			continue
		if root.polyline.target.source_id not in pair.member_ids:
			continue
		if type(root.polyline.target.id) is not str:
			raise ValueError("selected Rust bracket member is not durable")
		members[root.polyline.target.source_id] = root.polyline.target.id
	if members.keys() != set(pair.member_ids):
		raise ValueError("selected Rust bracket has no complete rendered member pair")
	return FerrumNativeBracketDialogModel(
		pair.pair_id,
		(members[pair.member_ids[0]], members[pair.member_ids[1]]),
		(pair.member_ids[0], pair.member_ids[1]),
		"Bracket",
		width,
		pair.line_color,
		None,
		False,
	)


#============================================
def bracket_property_changes_from_dialog(
		changes: tuple[tuple[str, object], ...],
		) -> tuple[object, ...]:
	"""Map exact common appearance intent to frozen Rust bracket changes."""
	if type(changes) is not tuple:
		raise TypeError("Ferrum bracket property changes must be an exact tuple")
	import ferrum_qt.ferrum.engine as engine
	converted: list[object] = []
	for change in changes:
		if type(change) is not tuple or len(change) != 2:
			raise TypeError("Ferrum bracket changes must be exact field/value pairs")
		field, value = change
		if field == "line_width" and type(value) is float:
			converted.append(
				engine.DocumentBracketPropertyChangeV1.line_width(value),
			)
		elif field == "line_color" and type(value) is str:
			converted.append(
				engine.DocumentBracketPropertyChangeV1.line_color(value),
			)
		else:
			raise ValueError("Ferrum bracket form supplied an unsupported property change")
	return tuple(converted)


#============================================
class FerrumNativeGeometricPropertiesMixin:
	"""Durable selection and mutation methods mixed into the Ferrum document tab."""

	#============================================
	def has_one_selected_geometric(self) -> bool:
		"""Return whether one geometric root or complete bracket pair is selected."""
		if self._disposed or self.requires_refresh:
			return False
		try:
			self.selected_geometric_dialog_state()
		except (RuntimeError, TypeError, ValueError):
			return False
		return True

	#============================================
	def selected_geometric_dialog_state(self) -> tuple[object, object]:
		"""Return one exact editable vector payload and its detached-form model."""
		try:
			pair, model = self.selected_bracket_pair_projection()
		except RuntimeError:
			root = self.selected_geometric_projection()
			return root, dialog_model_from_projection(root)
		return pair, model

	#============================================
	def selected_bracket_pair_projection(
			self,
			) -> tuple[object, FerrumNativeBracketDialogModel]:
		"""Return the exact projected pair selected through both durable members."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if len(selected) != 2 or any(target.kind != "polyline" for target in selected):
			raise RuntimeError("select both sides of exactly one bracket pair")
		if any(target.identifier is None for target in selected):
			raise RuntimeError("selected bracket has a non-durable member")
		if self._document_observation is None:
			raise RuntimeError("selected bracket has no current durable projection")
		stack = self._document_observation.projection.presentation_stack
		selected_ids = {target.identifier for target in selected}
		matches = []
		for pair in stack.bracket_pairs:
			try:
				model = bracket_dialog_model_from_projection(pair, stack.roots)
			except (TypeError, ValueError):
				continue
			if set(model.member_target_ids) == selected_ids:
				matches.append((pair, model))
		if len(matches) != 1:
			raise RuntimeError("selected polylines are not one complete Rust bracket pair")
		return matches[0]

	#============================================
	def selected_geometric_projection(self) -> object:
		"""Return the selected exact frozen Rust geometric root projection."""
		self._require_mutable()
		selected = self._require_projection().selected_durable_targets()
		if len(selected) != 1 or selected[0].kind not in _GEOMETRIC_KINDS:
			raise RuntimeError("select exactly one geometric presentation first")
		if selected[0].identifier is None or self._document_observation is None:
			raise RuntimeError("selected geometry has no current durable projection")
		for root in self._document_observation.projection.presentation_stack.roots:
			if root.kind != selected[0].kind:
				continue
			model = dialog_model_from_projection(root)
			if model.target_id == selected[0].identifier:
				return root
		raise RuntimeError("selected geometry is absent from the Rust projection")

	#============================================
	def apply_selected_geometric_properties(self, expected_target_id: str,
			expected_source_id: str, changes: tuple[object, ...]) -> object:
		"""Commit one closed geometric patch without allowing selection retargeting."""
		self._require_mutable()
		if type(expected_target_id) is not str or type(expected_source_id) is not str:
			raise TypeError("Ferrum geometric properties require durable string identifiers")
		if type(changes) is not tuple:
			raise TypeError("Ferrum geometric properties require an exact change tuple")
		root = self.selected_geometric_projection()
		model = dialog_model_from_projection(root)
		if (
			model.target_id != expected_target_id
			or model.source_id != expected_source_id
		):
			raise RuntimeError("selected geometry changed while its properties form was open")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentGeometricPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum geometric properties require exact frozen Ferrum changes")
		operation = engine.DocumentOperationV1.set_geometric_properties(
			expected_source_id, changes,
		)
		result = self._apply_current_document_operation_v1(operation)
		self._install_mutation_result(
			result, ((model.kind, model.target_id),),
		)
		return result

	#============================================
	def apply_selected_bracket_properties(self, expected_pair_id: str,
			expected_member_target_ids: tuple[str, str], changes: tuple[object, ...]) -> object:
		"""Commit one common pair patch without allowing selection retargeting."""
		self._require_mutable()
		if type(expected_pair_id) is not str or type(expected_member_target_ids) is not tuple:
			raise TypeError("Ferrum bracket properties require exact durable identifiers")
		if type(changes) is not tuple:
			raise TypeError("Ferrum bracket properties require an exact change tuple")
		_pair, model = self.selected_bracket_pair_projection()
		if (
				model.pair_id != expected_pair_id
				or model.member_target_ids != expected_member_target_ids
			):
			raise RuntimeError("selected bracket changed while its properties form was open")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentBracketPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum bracket properties require exact frozen Ferrum changes")
		operation = engine.DocumentOperationV1.set_bracket_properties(
			expected_pair_id, changes,
		)
		result = self._apply_current_document_operation_v1(operation)
		self._install_mutation_result(
			result,
			tuple(("polyline", identifier) for identifier in model.member_target_ids),
		)
		return result


#============================================
def install_geometric_properties_action(window: object,
		edit_menu: PySide6.QtWidgets.QMenu) -> PySide6.QtGui.QAction:
	"""Install one Ferrum vector-properties action on the public window."""
	action = PySide6.QtGui.QAction(window.tr("Edit Vector Properties"), window)
	action.setToolTip(window.tr(
		"Edit one selected geometric vector or complete bracket pair through Rust",
	))
	action.triggered.connect(lambda _checked=False: _on_edit_geometric_properties(window))
	edit_menu.addAction(action)
	return action


#============================================
def refresh_geometric_properties_action(action: PySide6.QtGui.QAction,
		tab: object | None, active: bool, pending: bool, busy: bool) -> None:
	"""Make the action follow exact current durable geometric selection."""
	action.setEnabled(
		active and not pending and not busy and tab.has_one_selected_geometric(),
	)


#============================================
def _on_edit_geometric_properties(window: object) -> None:
	"""Run one detached geometric form and submit only changed fields."""
	tab = window._active_native_tab()
	if tab is None:
		return
	try:
		payload, model = tab.selected_geometric_dialog_state()
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	dialog = ferrum_qt.dialogs.geometric_properties_dialog.GeometricPropertiesDialog(
		model.title,
		model.line_width,
		model.line_color,
		model.area_color,
		model.fillable,
		window,
	)
	if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
		return
	try:
		if type(model) is FerrumNativeBracketDialogModel:
			changes = bracket_property_changes_from_dialog(dialog.changes())
			tab.apply_selected_bracket_properties(
				model.pair_id, model.member_target_ids, changes,
			)
		else:
			changes = property_changes_from_dialog(payload, dialog.changes())
			tab.apply_selected_geometric_properties(
				model.target_id, model.source_id, changes,
			)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Updated Ferrum vector appearance."), 5000)
	window._refresh_actions()
