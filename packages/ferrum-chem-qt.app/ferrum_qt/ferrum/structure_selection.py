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
import ferrum_qt.ferrum.keyboard_canvas
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
	def _refresh_structure_selection_action(self, navigation_enabled: bool) -> None:
		"""Refresh read-only selection navigation separately from deletion permission."""
		if self._structure_tab is not None and (
			not navigation_enabled or self._active_native_tab() is not self._structure_tab
		):
			self._cancel_structure_selection()
		self._select_structure_action.setEnabled(navigation_enabled)
		self._refresh_structure_deletion_action()

	#============================================
	def _refresh_structure_deletion_action(self) -> None:
		"""Enable the registered deletion action only for this live selection."""
		action = self._delete_structure_selection_action
		selection = self._structure_selection
		action.setEnabled(
			self._structure_selection_mutation_eligible()
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
		self._template_catalog_controller.cancel_active(reopen=False)
		self._cancel_atom_insertion()
		self._cancel_line_gesture(clear_status=False)
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			self._cancel_structure_selection()
			return False
		self._replace_render_interaction_selection(None, tab)
		self._structure_tab = tab
		tab.view.show_keyboard_cursor()
		self._refresh_structure_selection_accessibility(tab)
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
		if intent.operation_id.startswith("selection.cursor.move.") and not intent.points:
			self._move_structure_selection_cursor(intent.operation_id, intent.modifiers)
			return
		if intent.operation_id == "selection.cursor.select" and not intent.points:
			self._select_structure_at_keyboard_cursor(intent.modifiers)
			return
		if intent.operation_id == "selection.delete" and not intent.points:
			self._delete_structure_selection_action.trigger()
			return
		raise RuntimeError("Ferrum structural selection received an unsupported mode intent.")

	#============================================
	def _select_structure_at(self, point: ferrum_qt.modes.base_mode.ScenePoint,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers,
			allow_empty_marquee: bool = True) -> None:
		"""Ask Rust to resolve one normalized atom, bond, or compact-group hit."""
		had_selected_targets = (
			self._structure_selection is not None
			and bool(self._structure_selection.targets)
		)
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
		is_toggle = bool(
			modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier,
		)
		if not allow_empty_marquee and not selection.targets and not (
			is_toggle and had_selected_targets
		):
			self.statusBar().showMessage(self.tr(
				"No selectable structure at document cursor.",
			), 5000)
			tab.view.viewport().setFocus()
			return
		self._structure_observation = observation
		self._replace_structure_selection(selection, tab)
		if allow_empty_marquee and not selection.targets:
			self._structure_press_scene = PySide6.QtCore.QPointF(point.x, point.y)
			self._structure_marquee = self._new_structure_marquee(tab, self._structure_press_scene)

	#============================================
	def _move_structure_selection_cursor(self, operation_id: str,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Move only the view-owned cursor while Select Structure is active."""
		direction_by_operation = {
			"selection.cursor.move.left": (-1.0, 0.0),
			"selection.cursor.move.right": (1.0, 0.0),
			"selection.cursor.move.up": (0.0, -1.0),
			"selection.cursor.move.down": (0.0, 1.0),
		}
		direction = direction_by_operation.get(operation_id)
		if direction is None:
			raise RuntimeError("Ferrum structure selection received an unknown cursor direction.")
		tab = self._active_native_tab()
		if tab is None or tab is not self._structure_tab or tab.requires_refresh:
			return
		fine = bool(modifiers & PySide6.QtCore.Qt.KeyboardModifier.ShiftModifier)
		increment = ferrum_qt.ferrum.keyboard_canvas.keyboard_cursor_increment(fine)
		point = tab.view.move_keyboard_cursor(
			float(direction[0] * increment), float(direction[1] * increment),
		)
		self._refresh_structure_selection_accessibility(tab)
		precision = "fine " if fine else ""
		self.statusBar().showMessage(self.tr(
			"{0}document cursor: {1:.1f}, {2:.1f}. Press Enter to select or Esc to cancel."
		).format(precision, point.x(), point.y()), 5000)
		tab.view.viewport().setFocus()

	#============================================
	def _select_structure_at_keyboard_cursor(self,
			modifiers: PySide6.QtCore.Qt.KeyboardModifiers) -> None:
		"""Resolve a cursor point without pointer marquee or no-hit replacement."""
		tab = self._active_native_tab()
		if tab is None or tab is not self._structure_tab or tab.requires_refresh:
			return
		point = tab.view.show_keyboard_cursor()
		self._select_structure_at(
			ferrum_qt.modes.base_mode.ScenePoint(float(point.x()), float(point.y())),
			modifiers, allow_empty_marquee=False,
		)

	#============================================
	def _structure_selection_accessibility_context(self) -> str:
		"""Describe only Rust-issued target descriptors for the active selection mode."""
		selection = self._structure_selection
		if selection is None or not selection.targets:
			summary = "No structure selected."
		else:
			kind_labels = (
				(ferrum_qt.ferrum.engine.StructureTargetKindV1.atom, "atom"),
				(ferrum_qt.ferrum.engine.StructureTargetKindV1.bond, "bond"),
				(ferrum_qt.ferrum.engine.StructureTargetKindV1.compact_group, "compact group"),
			)
			parts: list[str] = []
			for kind, label in kind_labels:
				count = sum(target.kind is kind for target in selection.targets)
				if count:
					suffix = "" if count == 1 else "s"
					parts.append(f"{count} {label}{suffix}")
			summary = "Selected: " + ", ".join(parts) + "."
		context = (
			"Select Structure mode. Enter selects at the document cursor; Shift+Enter "
			"toggles the target. {0} Escape cancels Select Structure."
		).format(summary)
		return context

	#============================================
	def _refresh_structure_selection_accessibility(self, tab: object) -> None:
		"""Give the view the active mode's Rust-derived selection summary."""
		if tab is self._structure_tab:
			tab.view.set_keyboard_cursor_accessibility_context(
				self._structure_selection_accessibility_context(),
			)

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
		if (
			not self._structure_selection_mutation_eligible()
			or self._structure_selection is None
			or not self._structure_selection.targets
		):
			self._refresh_structure_deletion_action()
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
		"""Install Rust's fenced action selection and a Qt-only bounds preview."""
		tab.replace_structure_action_selection_v1(selection)
		self._dispose_line_preview(self._structure_selection_item)
		self._structure_selection = selection
		self._structure_selection_item = None if selection is None else (
			ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
				tab, tuple(target.bounds for target in selection.targets),
			)
		)
		self._refresh_structure_deletion_action()
		self._refresh_structure_selection_accessibility(tab)
		# The tab owns structural action truth; the projection only presents bounds.
		self._refresh_actions()

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
		tab = self._structure_tab
		# Detach first: the replacement below refreshes every registered action.
		# A disabled action refresh may cancel this tool, so retaining the owner
		# until afterward would recursively re-enter cancellation.
		self._structure_tab = None
		self._dispose_structure_marquee()
		if tab is not None and not tab.is_disposed:
			self._replace_structure_selection(None, tab)
		else:
			self._dispose_line_preview(self._structure_selection_item)
			self._structure_selection_item = None
			self._structure_selection = None
		self._structure_observation = None
		if tab is not None and not tab.is_disposed:
			tab.view.set_keyboard_cursor_accessibility_context(None)
			tab.view.hide_keyboard_cursor()
			tab.view.viewport().setFocus()
