"""Rust-owned structural selection and deletion controller."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.context_menu
import ferrum_qt.declarative_resources
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.ferrum.document_tab_errors
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.structure_selection_mode
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.modes.base_mode
import ferrum_qt.themes.document_display_palette


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
		self._structure_tab = None
		self._delete_structure_selection_action = None

	#============================================
	def _build_structure_selection_action(self) -> None:
		"""Construct the explicit direct-structure selection tool."""
		self._select_structure_action = PySide6.QtGui.QAction(self.tr("Select Structure"), self)
		self._select_structure_action.setCheckable(True)
		self._select_structure_action.setToolTip(self.tr(
			"Select atoms, normal bonds, or compact groups; Shift toggles; Delete removes supported targets through Rust.",
		))
		self._register_action("draw.selection.structure", self._select_structure_action)
		self._delete_structure_selection_action = PySide6.QtGui.QAction(
			self.tr("Delete Selection"), self,
		)
		self._delete_structure_selection_action.setToolTip(self.tr(
			"Remove supported selected targets through Rust",
		))
		self._delete_structure_selection_action.setStatusTip(self.tr(
			"Remove supported selected targets through Rust",
		))
		self._delete_structure_selection_action.triggered.connect(
			self._request_structure_deletion,
		)
		self._register_action(
			"edit.delete_selection", self._delete_structure_selection_action,
			shortcut_exemption_reason=(
				"Delete and Backspace are normalized by the active Select Structure tool "
				"and trigger this registered action."
			),
		)
		binding = ferrum_qt.ferrum.window_mode_sync.FerrumWindowToolBinding(
			self._select_structure_action, ferrum_qt.modes.base_mode.ModeId.EDIT,
			ferrum_qt.ferrum.structure_selection_mode.StructureSelectionMode(),
			self._select_structure_action.text(), False, self._mode_context,
			lambda _context: self._activate_structure_selection(),
			self._dispatch_structure_selection_intent,
			lambda _context: self._cancel_structure_selection(),
		)
		self._window_mode_sync.register_tool(binding)

	#============================================
	def _refresh_structure_selection_action(self, enabled: bool) -> None:
		"""Keep the tool available only for a live mutable tab."""
		if self._structure_tab is not None and (
			not enabled or self._active_native_tab() is not self._structure_tab
		):
			self._cancel_structure_selection()
		self._select_structure_action.setEnabled(enabled)
		self._refresh_structure_deletion_action(enabled)

	#============================================
	def _refresh_structure_deletion_action(self, selection_owner_enabled: bool) -> None:
		"""Enable the registered deletion action only for this live selection."""
		action = self._delete_structure_selection_action
		selection = self._structure_selection
		action.setEnabled(
			selection_owner_enabled
			and self._structure_tab is self._active_native_tab()
			and selection is not None
			and bool(selection.targets)
		)

	#============================================
	def _activate_structure_selection(self) -> bool:
		"""Acquire Rust-backed structural selection after shared input selects this tool."""
		cancel_capture = getattr(self, "_cancel_live_smarts_selected_root_capture_v1", None)
		if callable(cancel_capture):
			cancel_capture("Molecule choice cancelled because Select Structure was selected.")
		self._cancel_catalog_placement()
		self._cancel_atom_insertion()
		self._cancel_line_gesture(clear_status=False)
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._cancel_structure_selection()
			return False
		self._structure_tab = tab
		tab.view.viewport().setFocus()
		self.statusBar().showMessage(self.tr(
			"Select atoms, normal bonds, or compact groups; Shift toggles; Delete removes supported targets through Rust.",
		), 5000)
		return True

	#============================================
	def _dispatch_structure_selection_intent(self,
			context: ferrum_qt.modes.base_mode.ModeContext,
			intent: ferrum_qt.modes.base_mode.ModeIntent) -> None:
		"""Resolve feature-local normalized intents through one Rust selection seam."""
		dispatch_context = context.dispatch_context
		if type(dispatch_context) is not dict or dispatch_context["window"] is not self:
			raise RuntimeError("Ferrum structural selection received another window context.")
		if intent.operation_id == "selection.press" and len(intent.points) == 1:
			self._select_structure_at(intent.points[0], intent.modifiers)
			return
		if intent.operation_id == "selection.move" and len(intent.points) == 1:
			self._update_structure_marquee(intent.points[0])
			return
		if intent.operation_id == "selection.release" and len(intent.points) == 1:
			self._dispose_structure_marquee()
			return
		if intent.operation_id == "selection.marquee" and len(intent.points) == 2:
			self._finish_structure_marquee(intent.points[1], intent.modifiers)
			return
		if intent.operation_id == "selection.delete" and not intent.points:
			self._delete_structure_selection_action.trigger()
			return
		raise RuntimeError("Ferrum structural selection received an unsupported mode intent.")

	#============================================
	def _select_structure_at(self, point: ferrum_qt.modes.base_mode.ScenePoint,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Ask Rust to resolve one normalized atom, bond, or compact-group hit."""
		try:
			tab = self._active_native_tab()
			if tab is None:
				return
			observation = tab.observe_structure_interaction()
			modifier = ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle if (
				modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
			) else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			selection = tab.select_structure_interaction(
				observation, self._structure_selection,
				ferrum_qt.ferrum.engine.StructureInteractionQueryV1.point(
					point.x, point.y, modifier,
				),
			)
		except (
			ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError,
			ferrum_qt.ferrum.engine.RenderInteractionError,
			ferrum_qt.ferrum.engine.RevisionConflictError,
		) as exc:
			self._show_edit_refusal(self._structure_refusal(exc))
			return
		self._structure_observation = observation
		self._replace_structure_selection(selection, tab)
		if not selection.targets:
			self._structure_press_scene = PySide6.QtCore.QPointF(point.x, point.y)
			self._structure_marquee = self._new_structure_marquee(tab, self._structure_press_scene)

	#============================================
	def _update_structure_marquee(self, point: ferrum_qt.modes.base_mode.ScenePoint) -> None:
		"""Update the one Qt-only marquee issued by an empty Rust press selection."""
		if self._structure_marquee is None:
			return
		self._structure_marquee.setRect(self._structure_rect(point))

	#============================================
	def _finish_structure_marquee(self, point: ferrum_qt.modes.base_mode.ScenePoint,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Resolve full containment through Rust after one shared-controller drag."""
		try:
			tab = self._active_native_tab()
			if tab is None or self._structure_observation is None:
				return
			rectangle = self._structure_rect(point)
			modifier = ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle if (
				modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier
			) else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			selection = tab.select_structure_interaction(
				self._structure_observation, self._structure_selection,
				ferrum_qt.ferrum.engine.StructureInteractionQueryV1.marquee(
					float(rectangle.left()), float(rectangle.top()),
					float(rectangle.right()), float(rectangle.bottom()), modifier,
				),
			)
		except (
			ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError,
			ferrum_qt.ferrum.engine.RenderInteractionError,
			ferrum_qt.ferrum.engine.RevisionConflictError,
		) as exc:
			self._show_edit_refusal(self._structure_refusal(exc))
			return
		finally:
			self._dispose_structure_marquee()
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
		except (
			ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError,
			ferrum_qt.ferrum.engine.RenderInteractionError,
			ferrum_qt.ferrum.engine.RevisionConflictError,
		) as exc:
			if isinstance(
				exc,
				ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabMutationPresentationError,
			):
				self._replace_structure_selection(None, tab)
				self._refresh_actions()
				self._show_edit_refusal(self._unavailable_edit_refusal(
					"Selected structure was deleted, but its authoritative display still needs "
					"recovery; refresh before saving or editing.",
				))
				return
			if self._active_native_tab() is not None:
				self._replace_structure_selection(selection, self._active_native_tab())
			self._show_edit_refusal(self._structure_refusal(exc))
			return
		self._replace_structure_selection(None, tab)
		self.statusBar().showMessage(self.tr(
			"Deleted {0} atoms, {1} bonds, and {2} compact groups through Rust.".format(
				commit.removed_atom_count, commit.removed_bond_count,
				commit.removed_compact_group_count,
			),
		), 5000)
		self._refresh_actions()

	#============================================
	def _structure_refusal(self, exc: Exception) -> object:
		"""Explain backend-declared structural exclusions without scene inference."""
		category = getattr(exc, "category", None)
		if category in (
			ferrum_qt.ferrum.engine.RenderInteractionCategoryV1.display_only,
			ferrum_qt.ferrum.engine.RenderInteractionCategoryV1.unsupported_target,
		):
			return self._unavailable_edit_refusal(self.tr(
				"Selection and drawing unchanged. This target is display-only; change presentation first.",
			))
		if category == ferrum_qt.ferrum.engine.RenderInteractionCategoryV1.unrenderable_candidate:
			return self._unavailable_edit_refusal(self.tr(
				"Selection and drawing unchanged. The resulting structure cannot be rendered; change presentation first.",
			))
		if category == ferrum_qt.ferrum.engine.RenderInteractionCategoryV1.cross_molecule_selection:
			return self._unavailable_edit_refusal(self.tr(
				"Selection and drawing unchanged. Structural edits must stay within one molecule.",
			))
		if category == ferrum_qt.ferrum.engine.RenderInteractionCategoryV1.invalid_compact_group_deletion_selection:
			return self._unavailable_edit_refusal(self.tr(
				"Selection and drawing unchanged. Select exactly one compact group without atoms or bonds.",
			))
		if category == ferrum_qt.ferrum.engine.RenderInteractionCategoryV1.invalid_compact_group_deletion_topology:
			return self._unavailable_edit_refusal(self.tr(
				"Selection and drawing unchanged. This compact group needs document repair before deleting it.",
			))
		return self._render_interaction_refusal(exc)

	#============================================
	def _replace_structure_selection(self, selection: object | None, tab: object) -> None:
		"""Publish generic document-object keys before retaining Rust's opaque selection."""
		generic_targets: list[tuple[str, str]] = []
		if selection is not None:
			for target in selection.targets:
				if type(target.object_id) is not str or not target.object_id:
					raise RuntimeError("Ferrum structure selection has no durable object identity")
				generic_targets.append(("document_object", target.object_id))
		tab._require_projection().select_durable(tuple(generic_targets))
		self._dispose_line_preview(self._structure_selection_item)
		self._structure_selection = selection
		self._structure_selection_item = None if selection is None else (
			ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
				tab, tuple(target.bounds for target in selection.targets),
			)
		)
		self._refresh_structure_deletion_action(self._select_structure_action.isEnabled())

	#============================================
	def _request_structure_deletion(self) -> None:
		"""Defer the one shared selection deletion operation to the Qt event loop."""
		if self._delete_structure_selection_action.isEnabled():
			PySide6.QtCore.QTimer.singleShot(0, self._commit_structure_deletion)

	#============================================
	def _show_structure_selection_context_menu(
			self, viewport: PySide6.QtWidgets.QWidget,
			global_position: PySide6.QtCore.QPoint) -> bool:
		"""Present declared enabled actions without changing the Rust selection."""
		if viewport is not self._controller_native_viewport:
			return False
		if not self._delete_structure_selection_action.isEnabled():
			self.statusBar().showMessage(self.tr(
				"Select a structure first, then open Drawing actions.",
			), 5000)
			return True
		accessible_name, action_groups = ferrum_qt.declarative_resources.load_context_menu_placement(
			self._action_registry,
		)
		menu = ferrum_qt.actions.context_menu.build_context_menu(
			viewport, self._action_registry, action_groups, accessible_name,
		)
		if menu is None:
			self.statusBar().showMessage(self.tr(
				"Select a structure first, then open Drawing actions.",
			), 5000)
			return True
		self.statusBar().showMessage(self.tr("Selected structure actions."), 5000)
		ferrum_qt.actions.context_menu.present_context_menu(menu, viewport, global_position)
		return True

	#============================================
	def _dispose_structure_marquee(self) -> None:
		"""Discard the temporary Qt-only marquee without mutating Rust."""
		self._dispose_line_preview(self._structure_marquee)
		self._structure_marquee = None
		self._structure_press_scene = None

	#============================================
	def _new_structure_marquee(self, tab: object,
			start: PySide6.QtCore.QPointF) -> PySide6.QtWidgets.QGraphicsRectItem:
		"""Create one noninteractive selection rectangle with no selection authority."""
		scene = tab.view.scene()
		if scene is None:
			raise RuntimeError("Ferrum document has no current scene")
		item = scene.addRect(PySide6.QtCore.QRectF(start, start))
		item.setAcceptedMouseButtons(PySide6.QtCore.Qt.MouseButton.NoButton)
		item.setZValue(1_000_000.0)
		refreshable = ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRoleMaterialRefreshableV1(
			(item,),
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.SELECTION_OUTLINE,
			None, 1.5, PySide6.QtCore.Qt.PenStyle.DashLine,
		)
		refreshable.refresh_document_display_palette(tab.document_display_palette)
		ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable(
			tab, item, refreshable,
		)
		return item

	#============================================
	def _structure_rect(self, point: ferrum_qt.modes.base_mode.ScenePoint) -> PySide6.QtCore.QRectF:
		"""Return the Rust scene-coordinate marquee rectangle for one mode intent."""
		press_scene = self._structure_press_scene
		if press_scene is None:
			raise RuntimeError("Ferrum structure marquee has no press position")
		end = PySide6.QtCore.QPointF(point.x, point.y)
		rectangle = PySide6.QtCore.QRectF(press_scene, end).normalized()
		return rectangle

	#============================================
	def _cancel_structure_selection(self) -> None:
		"""Release structural transient state without mutating Rust."""
		self._structure_tab = None
		self._dispose_structure_marquee()
		self._dispose_line_preview(self._structure_selection_item)
		self._structure_selection_item = None
		self._structure_selection = None
		self._structure_observation = None
