"""Rust-owned catalog palette and opaque shipped-template placement."""

import dataclasses
import math

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

@dataclasses.dataclass(frozen=True, slots=True)
class _Intent:
	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	key: str
	mouse_tracking: bool


class FerrumCatalogPalette(PySide6.QtWidgets.QDialog):
	"""A compact projection of immutable Rust catalog summaries."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		super().__init__(parent)
		self.setWindowTitle(self.tr("Insert Template"))
		self.setAccessibleName(self.tr("Ferrum template palette"))
		self.setAccessibleDescription(self.tr("Search Rust-owned templates, then place one on the canvas."))
		self.resize(620, 420)
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		row = PySide6.QtWidgets.QHBoxLayout()
		row.addWidget(PySide6.QtWidgets.QLabel(self.tr("Family:"), self))
		self.family = PySide6.QtWidgets.QComboBox(self)
		self.family.addItem(self.tr("System"), "system")
		self.family.addItem(self.tr("Biomolecule"), "biomolecule")
		self.family.setAccessibleName(self.tr("Template family"))
		row.addWidget(self.family)
		row.addWidget(PySide6.QtWidgets.QLabel(self.tr("Category:"), self))
		self.category = PySide6.QtWidgets.QComboBox(self)
		self.category.setAccessibleName(self.tr("Template category"))
		row.addWidget(self.category, 1)
		layout.addLayout(row)
		self.search = PySide6.QtWidgets.QLineEdit(self)
		self.search.setPlaceholderText(self.tr("Search templates"))
		self.search.setAccessibleName(self.tr("Search templates"))
		layout.addWidget(self.search)
		self.results = PySide6.QtWidgets.QListWidget(self)
		self.results.setAccessibleName(self.tr("Ferrum template results"))
		layout.addWidget(self.results, 1)
		self.details = PySide6.QtWidgets.QLabel(self)
		self.details.setWordWrap(True)
		layout.addWidget(self.details)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self.place_button = buttons.addButton(self.tr("Place on Canvas"), PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole)
		buttons.addButton(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel)
		layout.addWidget(buttons)
		buttons.accepted.connect(self.accept)
		buttons.rejected.connect(self.reject)
		self.family.currentIndexChanged.connect(self._refresh)
		self.category.currentIndexChanged.connect(self._refresh)
		self.search.textChanged.connect(self._refresh)
		self.results.currentItemChanged.connect(self._update_details)
		self.results.itemDoubleClicked.connect(lambda _item: self.accept())
		self._summaries: tuple[object, ...] = ()
		self._refresh()
		self.search.setFocus()

	def selected_key(self) -> str | None:
		item = self.results.currentItem()
		return None if item is None else item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)

	def _refresh(self) -> None:
		import ferrum_qt.ferrum.engine as engine
		try:
			summaries = tuple(engine.list_catalog_v1(self.family.currentData(), None, self.search.text().strip() or None))
		except Exception as error:
			self._summaries = ()
			self.results.clear()
			self.details.setText(self.tr("Catalog unavailable: {0}").format(str(error)))
			self.place_button.setEnabled(False)
			return
		selected_category = self.category.currentData()
		categories = tuple(sorted({summary.category for summary in summaries}))
		self.category.blockSignals(True)
		self.category.clear()
		self.category.addItem(self.tr("All categories"), None)
		for category in categories:
			self.category.addItem(category.replace("_", " ").title(), category)
		self.category.setCurrentIndex(max(self.category.findData(selected_category), 0))
		self.category.blockSignals(False)
		category = self.category.currentData()
		self._summaries = tuple(value for value in summaries if category is None or value.category == category)
		self.results.clear()
		for summary in self._summaries:
			item = PySide6.QtWidgets.QListWidgetItem("{0}  [{1}]".format(summary.label, summary.category.replace("_", " ")))
			item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, summary.key)
			item.setToolTip(self.tr("{0}; provenance: {1}").format(summary.key, summary.provenance_source))
			self.results.addItem(item)
		if self.results.count():
			self.results.setCurrentRow(0)
		else:
			self._update_details()

	def _update_details(self, *_unused: object) -> None:
		summary = next((value for value in self._summaries if value.key == self.selected_key()), None)
		self.place_button.setEnabled(summary is not None)
		self.details.setText(self.tr("No matching Ferrum templates.") if summary is None else self.tr("{0} | {1} | {2}").format(summary.key, summary.category.replace("_", " "), summary.provenance_source))


class FerrumNativeCatalogPlacementTabMixin:
	"""Commit one closed catalog request through generic document authority."""

	def place_catalog_molecule(
			self, revision: int, digest: str, key: str, x: float, y: float,
			) -> object:
		self._require_mutable()
		commit = self._session.place_catalog_molecule_v1(revision, digest, key, x, y)
		try:
			self._install_mutation_result(commit.result, (commit.root_identifier,))
		except Exception as error:
			from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
			if isinstance(error, FerrumNativeDocumentTabMutationPresentationError):
				error.accepted_receipt = commit
			raise
		return commit


class FerrumNativeCatalogPlacementWindowMixin:
	"""Own the palette lifecycle and semantic catalog placement capture."""

	def _initialize_catalog_placement(self) -> None:
		self._catalog_placement_intent: _Intent | None = None

	def _build_catalog_template_action(self) -> None:
		self._insert_catalog_template_action = PySide6.QtGui.QAction(self.tr("Insert Template..."), self)
		self._insert_catalog_template_action.setToolTip(self.tr("Browse Rust-owned templates, then place one on the canvas"))
		self._connect_interaction_action_v1(
			self._insert_catalog_template_action, self._on_insert_catalog_template,
		)
		self._action_registry.register_existing(
			"chemistry.template.insert", self._insert_catalog_template_action,
			shortcut_exemption_reason="Available by its labelled Chemistry menu client.",
		)

	def _wire_catalog_tool_replacement(self) -> None:
		"""Cancel catalog capture before a replacement tool installs its filter.

		A checkable QAction emits ``toggled(True)`` before its ``triggered(True)``
		handlers run.  The catalog owns the same window event-filter object as the
		line and selection tools, so cancelling from ``triggered`` could remove a
		filter the incoming tool had just installed.  Cancel the outgoing catalog
		owner during the earlier state transition instead.
		"""
		actions = (
			self._add_atom_action, self._draw_bond_action, self._draw_arrow_action,
			self._draw_plus_action, self._insert_text_action,
			self._insert_cyclohexane_ring_action, self._draw_wavy_action,
			self._attach_cyclohexane_ring_action,
			self._draw_bracket_action, self._draw_round_bracket_action,
			self._select_structure_action,
			self._move_atom_action, self._rotate_atoms_action,
			self._translate_roots_action, self._place_user_template_action,
			*self._draw_vector_actions.values(),
		)
		for action in actions:
			action.toggled.connect(
				lambda checked, changed_action=action:
				self._cancel_catalog_capture_before_authoring_activation(
					changed_action, checked,
				),
			)

	def _cancel_catalog_capture_before_authoring_activation(
			self, action: PySide6.QtGui.QAction, checked: bool,
			) -> None:
		"""Release catalog capture at the checked transition of one shared tool."""
		if checked:
			self._cancel_catalog_placement()

	def _on_insert_catalog_template(self) -> None:
		if self._catalog_placement_intent is not None:
			self._cancel_catalog_placement()
			return
		palette = FerrumCatalogPalette(self)
		if palette.exec() == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			key = palette.selected_key()
			if key is None or not self.start_catalog_placement(key):
				self._show_edit_refusal(self._unavailable_edit_refusal("The selected Ferrum template is unavailable. Choose it again."))

	def start_catalog_placement(self, key: str) -> bool:
		if self._catalog_placement_intent is not None or self._catalog_busy():
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		self._replace_authoring_owner_with_catalog()
		snapshot, viewport = tab.current_snapshot, tab.view.viewport()
		self._catalog_placement_intent = _Intent(
			tab, viewport, snapshot.revision, snapshot.digest, key, viewport.hasMouseTracking(),
		)
		viewport.setMouseTracking(True)
		viewport.installEventFilter(self)
		viewport.setCursor(PySide6.QtCore.Qt.CursorShape.CrossCursor)
		viewport.setFocus()
		self.statusBar().showMessage(self.tr("Click the canvas to place the template. Escape cancels."))
		self._refresh_actions()
		return True

	def _replace_authoring_owner_with_catalog(self) -> None:
		"""Cancel every competing authoring owner before catalog pointer capture."""
		self._cancel_atom_insertion()
		self._cancel_structure_selection()
		self._cancel_line_gesture()
		self._cancel_user_template_placement()

	def _catalog_busy(self) -> bool:
		return self._molecule_import_busy() or self._molecule_export_busy() or self._molecule_inspection_busy() or self._clipboard_busy() or self._coordinate_generation_intent is not None or self._atom_insertion_intent is not None or self._line_gesture_intent is not None or self._user_template_placement_intent is not None

	def eventFilter(self, watched: PySide6.QtCore.QObject, event: PySide6.QtCore.QEvent) -> bool:
		intent = self._catalog_placement_intent
		if intent is None or watched is not intent.viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress and event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
			self._cancel_catalog_placement()
			return True
		if event.type() == PySide6.QtCore.QEvent.Type.FocusOut:
			self._cancel_catalog_placement()
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseMove:
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress:
			if event.button() == PySide6.QtCore.Qt.MouseButton.RightButton:
				self._cancel_catalog_placement()
			elif event.button() == PySide6.QtCore.Qt.MouseButton.LeftButton:
				self._commit_catalog(event)
			else:
				return False
			return True
		return False

	def _commit_catalog(self, event: PySide6.QtGui.QMouseEvent) -> None:
		intent = self._catalog_placement_intent
		if intent is None:
			return
		try:
			if not self._catalog_current(intent):
				raise RuntimeError("The document changed; choose the template again.")
			point = intent.tab.view.snap_authored_scene_point(
				intent.tab.view.mapToScene(event.position().toPoint()),
			)
			if not math.isfinite(point.x()) or not math.isfinite(point.y()):
				raise ValueError("the canvas point is not finite")
			self._cancel_catalog_placement(False)
			commit = intent.tab.place_catalog_molecule(
				intent.revision, intent.digest, intent.key, float(point.x()), float(point.y()),
			)
		except Exception as error:
			self._cancel_catalog_placement(False)
			if hasattr(error, "accepted_receipt") and intent is not None:
				message = "Template was placed, but the display still needs recovery."
				commit = error.accepted_receipt
				try:
					if intent.tab.refresh_authoritative():
						message = "Template was placed; Ferrum refreshed the authoritative Rust display."
				except Exception:
					pass
				if message.startswith("Template was placed; "):
					self.statusBar().showMessage(self.tr(message), 5000)
					self._refresh_actions()
					target = commit.result.observation.snapshot
					self._publish_document_installation_v1(
						intent.tab, "catalog_template", intent.revision, intent.digest,
						target.revision, target.digest, 1,
					)
					return
				self._show_edit_refusal(self._unavailable_edit_refusal(message))
			else:
				self._show_edit_refusal(self._unavailable_edit_refusal(str(error)))
			self._refresh_actions()
			return
		self.statusBar().showMessage(self.tr("Placed one Ferrum template."), 5000)
		self._refresh_actions()
		target = commit.result.observation.snapshot
		self._publish_document_installation_v1(
			intent.tab, "catalog_template", intent.revision, intent.digest,
			target.revision, target.digest, 1,
		)

	def _catalog_current(self, intent: _Intent) -> bool:
		if self._active_native_tab() is not intent.tab or self._native_tabs_by_page.get(intent.tab) is not intent.tab:
			return False
		try:
			snapshot = intent.tab.current_snapshot
		except Exception:
			return False
		return snapshot.revision == intent.revision and snapshot.digest == intent.digest

	def _cancel_catalog_placement(self, clear_status: bool = True) -> None:
		intent = self._catalog_placement_intent
		self._catalog_placement_intent = None
		if intent is not None:
			intent.viewport.removeEventFilter(self)
			intent.viewport.setMouseTracking(intent.mouse_tracking)
			intent.viewport.unsetCursor()
		if clear_status:
			self.statusBar().clearMessage()

	def _catalog_placement_blocks_tab_close(self, tab: object) -> bool:
		if self._catalog_placement_intent is None or self._catalog_placement_intent.tab is not tab:
			return False
		self._cancel_catalog_placement()
		return True

	def _refresh_catalog_template_action(self, active: bool, pending: bool, other_busy: bool) -> None:
		intent = self._catalog_placement_intent
		if intent is not None and not self._catalog_current(intent):
			self._cancel_catalog_placement()
		self._insert_catalog_template_action.setEnabled(active and not pending and not other_busy)
