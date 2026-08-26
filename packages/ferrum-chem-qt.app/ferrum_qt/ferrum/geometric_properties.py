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

	member_target_ids: tuple[str, str]
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
	if type(payload.target.document_object_id) is not str:
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
		payload.target.document_object_id,
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
	members = pair.members
	if (
			type(members) not in (list, tuple)
			or len(members) != 2
			or any(type(identifier) is not str for identifier in members)
			or members[0] == members[1]
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
	member_target_ids: dict[str, str] = {}
	for root in roots:
		if type(root) is not engine.PresentationRootProjectionV1:
			raise TypeError("Ferrum bracket properties require exact Ferrum roots")
		if root.kind != "polyline" or root.polyline is None:
			continue
		document_object_id = root.polyline.target.document_object_id
		if document_object_id not in members:
			continue
		if type(document_object_id) is not str:
			raise ValueError("selected Rust bracket member is not durable")
		if document_object_id in member_target_ids:
			raise ValueError("selected Rust bracket has duplicate rendered members")
		member_target_ids[document_object_id] = document_object_id
	if set(member_target_ids) != set(members):
		raise ValueError("selected Rust bracket has no complete rendered member pair")
	return FerrumNativeBracketDialogModel(
		(member_target_ids[members[0]], member_target_ids[members[1]]),
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
	def _current_presentation_entries(self) -> tuple[object, ...]:
		"""Return the exact current Rust presentation payload sequence."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		observation = self._document_observation
		if type(observation) is not engine.SessionDocumentObservationV1:
			raise RuntimeError("Ferrum tab has no current document observation")
		entries = observation.projection.presentation_stack.entries
		if type(entries) is not tuple:
			raise RuntimeError("Rust presentation entries are not an exact DTO tuple")
		if any(type(entry) is not engine.PresentationRootProjectionV1 for entry in entries):
			raise RuntimeError("Rust presentation entries contain an invalid DTO")
		return entries

	#============================================
	def _selected_geometric_root(
			self, document_object_id: str, selector_kind: object,
			) -> object:
		"""Join one typed direct-root selector to one exact geometric payload."""
		import ferrum_qt.ferrum.engine as engine
		if type(document_object_id) is not str or not document_object_id:
			raise RuntimeError("selected geometry lacks a durable document-object identity")
		presentation_kinds = engine.DocumentPresentationRootKindV1
		matches: list[object] = []
		for root in self._current_presentation_entries():
			if root.kind not in _GEOMETRIC_KINDS:
				continue
			model = dialog_model_from_projection(root)
			if model.target_id != document_object_id:
				continue
			expected_kind = getattr(presentation_kinds, model.kind, None)
			if type(expected_kind) is not presentation_kinds:
				raise RuntimeError("Rust geometric presentation selector kind is unavailable")
			if selector_kind != expected_kind:
				raise RuntimeError("selected geometry has an inconsistent Rust root kind")
			matches.append(root)
		if len(matches) != 1:
			raise RuntimeError("selected geometry is absent or duplicated in the Rust projection")
		return matches[0]

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
		"""Return the one current complete pair selected through durable render targets."""
		self._require_mutable()
		selected = self._selected_presentation_root_selectors()
		import ferrum_qt.ferrum.engine as engine
		polyline_kind = engine.DocumentPresentationRootKindV1.polyline
		if (
				len(selected) != 2
				or any(type(identifier) is not str or not identifier or kind != polyline_kind
						for identifier, kind in selected)
			):
			raise RuntimeError("select both members of one bracket pair first")
		selected_ids = frozenset(identifier for identifier, _kind in selected)
		if len(selected_ids) != 2:
			raise RuntimeError("select two distinct members of one bracket pair first")
		roots = self._current_presentation_entries()
		bracket_pairs = self._document_observation.projection.presentation_stack.bracket_pairs
		if type(bracket_pairs) is not tuple:
			raise RuntimeError("Rust bracket pairs are not an exact DTO tuple")
		for pair in bracket_pairs:
			model = bracket_dialog_model_from_projection(pair, roots)
			if frozenset(model.member_target_ids) == selected_ids:
				return pair, model
		raise RuntimeError("selected polylines are not one complete editable bracket pair")

	#============================================
	def selected_geometric_projection(self) -> object:
		"""Return the selected exact frozen Rust geometric root projection."""
		self._require_mutable()
		selected = self._selected_presentation_root_selectors()
		if len(selected) != 1:
			raise RuntimeError("select exactly one geometric presentation first")
		return self._selected_geometric_root(*selected[0])

	#============================================
	def apply_selected_geometric_properties(self, expected_target_id: str,
			changes: tuple[object, ...]) -> object:
		"""Commit one closed geometric patch without allowing selection retargeting."""
		self._require_mutable()
		if type(expected_target_id) is not str:
			raise TypeError("Ferrum geometric properties require durable string identifiers")
		if type(changes) is not tuple:
			raise TypeError("Ferrum geometric properties require an exact change tuple")
		root = self.selected_geometric_projection()
		model = dialog_model_from_projection(root)
		if model.target_id != expected_target_id:
			raise RuntimeError("selected geometry changed while its properties form was open")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentGeometricPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum geometric properties require exact frozen Ferrum changes")
		snapshot = self.current_snapshot
		result = self._session.set_geometric_properties_v1(
			snapshot.revision, snapshot.digest, expected_target_id, changes,
		)
		self._install_mutation_result(
			result, (model.target_id,),
		)
		return result

	#============================================
	def apply_selected_bracket_properties(self, expected_member_target_ids: tuple[str, str],
			changes: tuple[object, ...]) -> object:
		"""Commit one common pair patch without allowing selection retargeting."""
		self._require_mutable()
		if type(expected_member_target_ids) is not tuple:
			raise TypeError("Ferrum bracket properties require exact durable identifiers")
		if type(changes) is not tuple:
			raise TypeError("Ferrum bracket properties require an exact change tuple")
		_pair, model = self.selected_bracket_pair_projection()
		if model.member_target_ids != expected_member_target_ids:
			raise RuntimeError("selected bracket changed while its properties form was open")
		import ferrum_qt.ferrum.engine as engine
		if any(type(change) is not engine.DocumentBracketPropertyChangeV1
				for change in changes):
			raise TypeError("Ferrum bracket properties require exact frozen Ferrum changes")
		snapshot = self.current_snapshot
		result = self._session.set_bracket_pair_properties_v1(
			snapshot.revision, snapshot.digest, expected_member_target_ids, changes,
		)
		self._install_mutation_result(
			result,
			model.member_target_ids,
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
			tab.apply_selected_bracket_properties(model.member_target_ids, changes)
		else:
			changes = property_changes_from_dialog(payload, dialog.changes())
			tab.apply_selected_geometric_properties(model.target_id, changes)
	except Exception as exc:
		window._refresh_actions()
		window._show_edit_refusal(window._unavailable_edit_refusal(str(exc)))
		return
	window.statusBar().showMessage(window.tr("Updated Ferrum vector appearance."), 5000)
	window._refresh_actions()
