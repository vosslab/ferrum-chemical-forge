"""Rust-owned P0.3 structural selection and deletion controller."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.direct_root_preview


#============================================
class FerrumNativeStructureSelectionMixin:
	"""Own transient direct-structure selection; Rust owns targets and deletion."""

	#============================================
	def _initialize_structure_selection(self) -> None:
		"""Initialize the one optional structural canvas controller."""
		self._structure_selection = None
		self._structure_observation = None
		self._structure_selection_item = None
		self._structure_marquee = None
		self._structure_press_scene = None
		self._structure_viewport = None
		self._structure_tab = None

	#============================================
	def _build_structure_selection_action(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the explicit direct atom/bond selection tool."""
		self._select_structure_action = PySide6.QtGui.QAction(self.tr("Select Structure"), self)
		self._select_structure_action.setCheckable(True)
		self._select_structure_action.setToolTip(self.tr(
			"Select atoms or normal bonds; Shift toggles; Delete removes through Rust.",
		))
		self._connect_interaction_action_v1(
			self._select_structure_action, self._toggle_structure_selection,
		)
		menu.addAction(self._select_structure_action)

	#============================================
	def _refresh_structure_selection_action(self, enabled: bool) -> None:
		"""Keep the tool available only for a live mutable tab."""
		if self._structure_viewport is not None and (
			not enabled or self._active_native_tab() is not self._structure_tab
		):
			self._cancel_structure_selection()
		self._select_structure_action.setEnabled(enabled)

	#============================================
	def _toggle_structure_selection(self, checked: bool) -> None:
		"""Install or retire the structural interaction event boundary."""
		cancel_capture = getattr(self, "_cancel_live_smarts_selected_root_capture_v1", None)
		if callable(cancel_capture):
			cancel_capture("Molecule choice cancelled because Select Structure was selected.")
		if not checked:
			self._cancel_structure_selection()
			return
		self._cancel_catalog_placement()
		self._cancel_atom_insertion()
		self._cancel_line_gesture(clear_status=False)
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._cancel_structure_selection()
			return
		self._structure_viewport = tab.view.viewport()
		self._structure_tab = tab
		self._structure_viewport.installEventFilter(self)
		self._structure_viewport.setFocus()
		self.statusBar().showMessage(self.tr(
			"Select atoms or normal bonds; Shift toggles; Delete removes selected structure.",
		), 5000)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Handle the structural mode before unrelated pointer tools."""
		if watched is not self._structure_viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.FocusOut:
			self._cancel_structure_selection()
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.ShortcutOverride:
			if event.key() in (
				PySide6.QtCore.Qt.Key.Key_Delete,
				PySide6.QtCore.Qt.Key.Key_Backspace,
			):
				event.accept()
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			key = event.key()
			if key == PySide6.QtCore.Qt.Key.Key_Escape:
				self._cancel_structure_selection()
				return True
			if key in (PySide6.QtCore.Qt.Key.Key_Delete, PySide6.QtCore.Qt.Key.Key_Backspace):
				PySide6.QtCore.QTimer.singleShot(0, self._commit_structure_deletion)
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress:
			if event.button() == PySide6.QtCore.Qt.MouseButton.LeftButton:
				self._select_structure_at(event.position().toPoint(), event.modifiers())
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseMove:
			if self._structure_marquee is not None:
				self._structure_marquee.setRect(self._structure_rect(event.position().toPoint()))
				return True
			return False
		if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonRelease:
			if event.button() == PySide6.QtCore.Qt.MouseButton.LeftButton and self._structure_marquee is not None:
				self._finish_structure_marquee(event.position().toPoint(), event.modifiers())
				return True
		return False

	#============================================
	def _select_structure_at(self, point: PySide6.QtCore.QPoint,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Ask Rust to resolve one atom/bond hit or start a visual marquee."""
		try:
			import ferrum_qt.ferrum.engine as engine
			tab = self._active_native_tab()
			if tab is None:
				return
			observation = tab.observe_structure_interaction()
			modifier = engine.RenderInteractionModifierV1.toggle if (
				modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
			) else engine.RenderInteractionModifierV1.replace
			scene = tab.view.mapToScene(point)
			selection = tab.select_structure_interaction(
				observation, self._structure_selection,
				engine.StructureInteractionQueryV1.point(float(scene.x()), float(scene.y()), modifier),
			)
		except Exception as exc:
			self._show_edit_refusal(self._structure_refusal(exc))
			return
		self._structure_observation = observation
		self._replace_structure_selection(selection, tab)
		if not selection.targets:
			self._structure_press_scene = scene
			self._structure_marquee = self._new_structure_marquee(tab, scene)

	#============================================
	def _finish_structure_marquee(self, point: PySide6.QtCore.QPoint,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Resolve full containment through Rust after one visual-only rectangle."""
		try:
			import ferrum_qt.ferrum.engine as engine
			tab = self._active_native_tab()
			if tab is None or self._structure_observation is None:
				return
			rectangle = self._structure_rect(point)
			modifier = engine.RenderInteractionModifierV1.toggle if (
				modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
			) else engine.RenderInteractionModifierV1.replace
			selection = tab.select_structure_interaction(
				self._structure_observation, self._structure_selection,
				engine.StructureInteractionQueryV1.marquee(
					float(rectangle.left()), float(rectangle.top()),
					float(rectangle.right()), float(rectangle.bottom()), modifier,
				),
			)
		except Exception as exc:
			self._show_edit_refusal(self._structure_refusal(exc))
			return
		finally:
			self._retire_structure_marquee()
		self._replace_structure_selection(selection, tab)

	#============================================
	def _commit_structure_deletion(self) -> None:
		"""Delete the exact opaque Rust selection as one history operation."""
		if self._structure_selection is None or not self._structure_selection.targets:
			return
		selection = self._structure_selection
		try:
			tab = self._structure_tab
			if tab is None:
				return
			commit = tab.commit_structure_deletion(selection)
		except Exception as exc:
			if self._active_native_tab() is not None:
				self._replace_structure_selection(selection, self._active_native_tab())
			self._show_edit_refusal(self._structure_refusal(exc))
			return
		self._replace_structure_selection(None, tab)
		self.statusBar().showMessage(self.tr(
			"Deleted {0} atoms and {1} bonds through Rust.".format(
				len(commit.removed_atoms), len(commit.removed_bonds),
			),
		), 5000)
		self._refresh_actions()

	#============================================
	def _structure_refusal(self, exc: Exception) -> str:
		"""Explain backend-declared structural exclusions without scene inference."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(exc, "category", None)
		if category in (
			engine.RenderInteractionCategoryV1.display_only,
			engine.RenderInteractionCategoryV1.unsupported_target,
		):
			return self.tr(
				"Selection and drawing unchanged. This target is display-only; change presentation first.",
			)
		if category == engine.RenderInteractionCategoryV1.cross_molecule_selection:
			return self.tr(
				"Selection and drawing unchanged. Structural edits must stay within one molecule.",
			)
		return self._render_interaction_refusal(exc)

	#============================================
	def _replace_structure_selection(self, selection: object | None, tab: object) -> None:
		"""Project only backend-issued structure bounds as a disposable overlay."""
		self._retire_line_preview(self._structure_selection_item)
		self._structure_selection = selection
		self._structure_selection_item = None if selection is None else (
			ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
				tab, tuple(target.bounds for target in selection.targets),
			)
		)

	#============================================
	def _new_structure_marquee(self, tab: object,
			start: PySide6.QtCore.QPointF) -> PySide6.QtWidgets.QGraphicsRectItem:
		"""Create a noninteractive rectangle with no selection authority."""
		scene = tab.view.scene()
		if scene is None:
			raise RuntimeError("Ferrum document has no current scene")
		pen = PySide6.QtGui.QPen(PySide6.QtWidgets.QApplication.palette().highlight().color())
		pen.setStyle(PySide6.QtCore.Qt.PenStyle.DashLine)
		item = scene.addRect(PySide6.QtCore.QRectF(start, start), pen)
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		item.setZValue(1_000_000.0)
		return item

	#============================================
	def _structure_rect(self, point: PySide6.QtCore.QPoint) -> PySide6.QtCore.QRectF:
		"""Return a visual viewport-converted rectangle for Rust containment."""
		assert self._structure_press_scene is not None
		end = self._active_native_tab().view.mapToScene(point)
		return PySide6.QtCore.QRectF(self._structure_press_scene, end).normalized()

	#============================================
	def _retire_structure_marquee(self) -> None:
		"""Retire the temporary Qt-only marquee."""
		self._retire_line_preview(self._structure_marquee)
		self._structure_marquee = None
		self._structure_press_scene = None

	#============================================
	def _cancel_structure_selection(self) -> None:
		"""Release structural event capture and overlays without mutating Rust."""
		if hasattr(self, "_select_structure_action"):
			self._select_structure_action.setChecked(False)
		if self._structure_viewport is not None:
			self._structure_viewport.removeEventFilter(self)
		self._structure_viewport = None
		self._structure_tab = None
		self._retire_structure_marquee()
		self._retire_line_preview(self._structure_selection_item)
		self._structure_selection_item = None
		self._structure_selection = None
		self._structure_observation = None
