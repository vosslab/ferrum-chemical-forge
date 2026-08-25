"""Ferrum Qt tab lifecycle and action-refresh responsibilities."""
# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.atom_element
import ferrum_qt.ferrum.drawing_standard as native_drawing_standard
import ferrum_qt.ferrum.paper_properties as native_paper_properties
import ferrum_qt.ferrum.presentation_properties
import ferrum_qt.ferrum.arrow_properties
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties
import ferrum_qt.ferrum.wavy_properties as native_wavy_properties

#============================================
class FerrumNativeMainWindowLifecycleMixin:
	"""Own tab registration, lifecycle guards, and action reachability."""

	def _register_native_tab(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			*, activate: bool,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Attach one exact Ferrum tab to this standalone public host."""
		if type(tab) is not ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
			raise TypeError("Ferrum window requires an exact FerrumNativeDocumentTab")
		if tab in self._native_tabs_by_page:
			raise ValueError("Ferrum tab is already registered")
		index = self._tab_widget.addTab(tab, tab.title)
		self._native_tabs_by_page[tab] = tab
		self._install_native_hex_grid_for_tab(tab)
		tab.selection_changed.connect(self._on_native_selection_changed)
		tab.view.display_transform_changed.connect(self._refresh_native_view_status)
		if activate:
			self._tab_widget.setCurrentIndex(index)
		# The first addTab() selects its page before this method can publish the
		# page-to-tab mapping.  Re-enter the authoritative activation lifecycle
		# only when that early Qt signal could not identify this mapped tab.
		if self._active_native_tab() is tab and self._last_native_tab is not tab:
			self._on_native_tab_changed(index)
		return tab

	#============================================
	def save_active_to_path(self, path: str) -> bool:
		"""Publish the selected Ferrum tab to a caller-supplied CDML destination."""
		tab = self._active_native_tab()
		if tab is None:
			return False
		return self._save_native_tab_to_path(tab, path)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _close_tab_at(self, index: int) -> None:
		"""Dispose one clean Ferrum tab, retaining dirty tabs for an explicit save."""
		page = self._tab_widget.widget(index)
		tab = self._native_tabs_by_page.get(page)
		if tab is None:
			return
		if self._cancel_explicit_replacement_for_target_close(tab):
			return
		if self._molecule_import_blocks_tab_close(tab):
			return
		if self._molecule_export_blocks_tab_close(tab):
			return
		if self._snapshot_export_blocks_tab_close(tab):
			return
		if self._molecule_inspection_blocks_tab_close(tab):
			return
		if self._atom_oxidation_blocks_tab_close(tab):
			return
		if self._clipboard_operation_blocks_tab_close(tab):
			return
		if self._coordinate_generation_blocks_tab_close(tab):
			return
		if self._user_template_placement_blocks_tab_close(tab):
			return
		if self._catalog_placement_blocks_tab_close(tab):
			return
		if tab.requires_refresh:
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Refresh the authoritative Rust view before closing this tab.",
			))
			return
		if tab.is_dirty:
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Save or discard the Ferrum document before closing this tab.",
			))
			return
		if self._atom_insertion_intent is not None and self._atom_insertion_intent.tab is tab:
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None and self._line_gesture_intent.tab is tab:
			if not self._cancel_line_gesture():
				self._show_edit_refusal(self._typed_refusal(
					"close_document", "busy_close",
					"Ferrum could not retire the pending cyclohexane attachment; retry cancellation before closing.",
				))
				return
		if getattr(self, "_structure_tab", None) is tab:
			self._cancel_structure_selection()
		if (
			self._direct_glycosidic_haworth_intent is not None
			and self._direct_glycosidic_haworth_intent.tab is tab
		):
			self._cancel_direct_glycosidic_haworth_intent()
		try:
			tab.dispose()
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Ferrum could not retire the live SMARTS result; refresh before closing this tab.",
			))
			return
		self._retire_molecule_report_dialog_for_tab(tab)
		self._retire_atom_oxidation_dialog_for_tab(tab)
		self._cancel_native_view_controls_for_tab(tab)
		self._retire_closed_native_tab(tab, index)
		self._refresh_actions()

	#============================================
	def _retire_closed_native_tab(self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			index: int,
			) -> None:
		"""Transfer a disposed tab from the tab host to Qt deferred deletion."""
		self._tab_widget.removeTab(index)
		self._native_tabs_by_page.pop(tab)
		tab.hide()
		tab.setParent(None)
		tab.deleteLater()

	#============================================
	def _close_current_tab(self) -> None:
		"""Close the selected page through the same clean-tab guard."""
		index = self._tab_widget.currentIndex()
		if index >= 0:
			self._close_tab_at(index)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _on_native_tab_changed(self, _index: int) -> None:
		"""Retire one outgoing SMARTS plan, then bind the incoming tab once."""
		controller = getattr(self, "_smarts_query_controller", None)
		if hasattr(self, "_native_tabs_by_page"):
			previous = self._last_native_tab
			current = self._active_native_tab()
			if previous is not None and previous is not current:
				retirement_succeeded = True
				try:
					previous._require_live_smarts_retirement_v1("tab_deactivated")
				except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
					retirement_succeeded = False
					self._show_edit_refusal(self._typed_refusal(
						"close_document", "busy_close",
						"Ferrum could not retire the live SMARTS result; refresh before editing this tab.",
					))
				if controller is not None:
					controller._deactivate_after_tab_retirement_v1(retirement_succeeded)
			self._last_native_tab = current
			if controller is not None:
				# A first registration has no outgoing tab, so it only binds its
				# already-published incoming plan and leaves the modeless dock alone.
				controller._activate_after_tab_switch_v1()
			self._on_native_view_tab_changed()
			self._refresh_actions()

	#============================================
	@PySide6.QtCore.Slot()
	def _on_native_selection_changed(self) -> None:
		"""Refresh actions after a Ferrum scene selection changes."""
		if hasattr(self, "_native_tabs_by_page"):
			self._refresh_actions()

	#============================================
	def _refresh_actions(self, *_unused: object) -> None:
		"""Make Ferrum Save and Close reachability follow the selected page."""
		self._refresh_local_document_open_action()
		tab = self._active_native_tab()
		active = tab is not None and not tab.is_disposed
		pending = active and tab.requires_refresh
		template_intent = self._user_template_placement_intent
		if (
			template_intent is not None
			and not self._user_template_placement_is_current(template_intent)
		):
			self._cancel_user_template_placement()
		catalog_intent = self._catalog_placement_intent
		if catalog_intent is not None and not self._catalog_current(catalog_intent):
			self._cancel_catalog_placement()
		busy_import = self._molecule_import_busy()
		busy_export = self._molecule_export_busy()
		busy_inspection = self._molecule_inspection_busy()
		busy_atom_oxidation = self._atom_oxidation_busy()
		busy_compact_group_materialization = self._compact_group_materialization_intent is not None
		busy_clipboard = self._clipboard_busy()
		busy_coordinates = self._coordinate_generation_intent is not None
		busy_user_template = self._user_template_placement_intent is not None
		busy_catalog_template = self._catalog_placement_intent is not None
		busy_snapshot_export = self._snapshot_export_busy()
		busy = (
			busy_import or busy_export or busy_inspection or busy_atom_oxidation or busy_compact_group_materialization or busy_clipboard or busy_coordinates
			or busy_user_template or busy_catalog_template or busy_snapshot_export
		)
		# A template placement is itself a terminal authoring intent.  Keep ordinary
		# document commands protected, but leave the exclusive authoring actions
		# reachable so selecting one can retire the template owner before it arms.
		authoring_busy = (
			busy_import or busy_export or busy_inspection or busy_atom_oxidation or busy_compact_group_materialization or busy_clipboard
			or busy_coordinates or busy_snapshot_export
		)
		if self._atom_insertion_intent is not None and (
			not active or self._atom_insertion_intent.tab is not tab or busy
		):
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None and (
			not active
			or self._line_gesture_intent.tab is not tab
			or busy
		):
			self._cancel_line_gesture()
		self._save_action.setEnabled(active and not pending and not busy)
		self._save_as_action.setEnabled(active and not pending and not busy)
		self._refresh_recovery_export_action(active, pending, busy)
		self._refresh_snapshot_export_actions(active, pending, busy)
		self._close_action.setEnabled(active and not pending and not busy)
		self._change_element_action.setEnabled(
			active
			and not busy
			and ferrum_qt.ferrum.atom_element.
			can_change_selected_atom_element(tab),
		)
		self._edit_atom_properties_action.setEnabled(
			active and not pending and not busy and tab.has_one_selected_atom(),
		)
		self._set_atom_number_action.setEnabled(
			active and not pending and not busy and tab.has_one_selected_atom(),
		)
		self._clear_atom_number_action.setEnabled(
			active and not pending and not busy and tab.selected_atom_has_number(),
		)
		can_mark = active and not pending and not busy and tab.has_one_selected_atom()
		for action in self._atom_mark_actions.values():
			action.setEnabled(can_mark)
		self._remove_atom_mark_action.setEnabled(
			can_mark and tab.selected_atom_has_marks(),
		)
		self._edit_bond_properties_action.setEnabled(
			active and not pending and not busy and tab.has_one_selected_bond(),
		)
		native_paper_properties.refresh_paper_properties_action(
			self._paper_properties_action, active, pending, busy,
		)
		native_drawing_standard.refresh_drawing_standard_action(
			self._drawing_standard_action, active, pending, busy,
		)
		ferrum_qt.ferrum.presentation_properties.refresh_plus_properties_action(
			self._edit_plus_properties_action, tab, active, pending, busy,
		)
		ferrum_qt.ferrum.arrow_properties.refresh_arrow_properties_action(
			self._edit_arrow_properties_action, tab, active, pending, busy,
		)
		native_geometric_properties.refresh_geometric_properties_action(
			self._edit_geometric_properties_action, tab, active, pending, busy,
		)
		native_wavy_properties.refresh_wavy_properties_action(
			self._edit_wavy_properties_action, tab, active, pending, busy,
		)
		self._delete_atom_action.setEnabled(active and not pending and not busy)
		self._delete_bond_action.setEnabled(active and not pending and not busy)
		self._change_bond_order_action.setEnabled(active and not pending and not busy)
		self._undo_action.setEnabled(
			active and not pending and not busy and tab.can_undo(),
		)
		self._redo_action.setEnabled(
			active and not pending and not busy and tab.can_redo(),
		)
		can_add_atom = (
			active and not pending and not authoring_busy
			and bool(tab.durable_molecule_choices())
		)
		self._add_atom_action.setEnabled(can_add_atom)
		self._add_atom_action.setToolTip(self.tr(
			"Use Next atom, then click the canvas once; Esc cancels"
			if can_add_atom else
			"Requires an active document with a durable Rust molecule",
		))
		self._add_single_bond_action.setEnabled(active and not pending and not busy)
		self._refresh_line_tool_actions(active and not pending and not authoring_busy)
		self._refresh_structure_selection_action(
			active and not pending and not authoring_busy,
		)
		self._refresh_top_level_transform_actions(tab, active, pending, authoring_busy)
		self._refresh_action.setEnabled(pending)
		self._refresh_molecule_import_actions(
			active, pending, busy_coordinates or busy_clipboard,
		)
		self._refresh_molfile_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_sdf_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_multi_sdf_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_molecule_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_molecule_inspection_actions(
			active,
			pending,
			busy_import or busy_export or busy_coordinates or busy_clipboard,
		)
		self._refresh_atom_oxidation_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_clipboard
			or busy_coordinates,
		)
		self._refresh_explicit_hydrogen_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_clipboard
			or busy_coordinates,
		)
		self._refresh_compact_group_materialization_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_clipboard
			or busy_coordinates or busy_atom_oxidation,
		)
		self._refresh_native_clipboard_actions(
			active, pending,
			busy_import or busy_export or busy_inspection or busy_coordinates,
		)
		self._refresh_molecule_name_action(active, pending, busy)
		self._refresh_linear_form_action(active, pending, busy)
		self._refresh_explicit_fragment_actions(active, pending, busy)
		self._refresh_direct_glycosidic_haworth_action(active, pending, busy)
		self._refresh_native_user_template_actions(
			active, pending,
			busy_import or busy_export or busy_inspection or busy_clipboard or busy_coordinates,
		)
		self._refresh_catalog_template_action(
			active, pending, busy_import or busy_export or busy_inspection or busy_clipboard
			or busy_coordinates or busy_user_template or busy_snapshot_export,
		)
		self._generate_coordinates_action.setEnabled(
			active and not pending and not busy and bool(tab.durable_molecule_choices()),
		)
		self._cancel_coordinates_action.setEnabled(
			busy_coordinates
			and not self._coordinate_generation_intent.worker.delivery_cancelled,
		)
		self._refresh_view_controls_actions()
		self._native_property_dock.refresh(tab)
		controller = getattr(self, "_smarts_query_controller", None)
		if controller is not None:
			controller.refresh_action(active, pending, busy)

	#============================================
	def _show_edit_refusal(self, request: object) -> None:
		"""Present a typed author-facing refusal with separate diagnostic detail."""
		import ferrum_qt.ferrum.window_refusals
		import ferrum_qt.dialogs.refusal_presenter
		if type(request) is not ferrum_qt.dialogs.refusal_presenter.RefusalRequest:
			raise TypeError("Ferrum refusal presentation requires an exact RefusalRequest")
		ferrum_qt.ferrum.window_refusals.show_refusal(self, request)


	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Dispose all clean pages and keep an unsaved Rust document live."""
		controller = getattr(self, "_smarts_query_controller", None)
		if controller is not None:
			controller.close()
		self._cancel_user_template_placement()
		self._cancel_catalog_placement()
		self._cancel_direct_glycosidic_haworth_intent()
		self._cancel_atom_insertion()
		if not self._cancel_line_gesture():
			event.ignore()
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Ferrum could not retire the pending cyclohexane attachment; retry cancellation before closing.",
			))
			return
		if self._cancel_local_document_open_for_close():
			event.ignore()
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Ferrum cancelled delivery; close again after Rust admission finishes.",
			))
			return
		if self._cancel_molecule_imports_for_close():
			event.ignore()
			return
		if self._cancel_molecule_export_for_close():
			event.ignore()
			return
		if self._cancel_snapshot_export_for_close():
			event.ignore()
			return
		if self._cancel_molecule_inspection_for_close():
			event.ignore()
			return
		if self._cancel_clipboard_operations_for_close():
			event.ignore()
			return
		if self._coordinate_generation_intent is not None:
			self._cancel_coordinate_generation()
			event.ignore()
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Ferrum cancelled delivery; close again after the current operation finishes.",
			))
			return
		if any(tab.requires_refresh for tab in self._native_tabs_by_page.values()):
			event.ignore()
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Refresh every pending authoritative Rust view before closing Ferrum.",
			))
			return
		if any(tab.is_dirty for tab in self._native_tabs_by_page.values()):
			event.ignore()
			self._show_edit_refusal(self._typed_refusal(
				"close_document", "busy_close",
				"Save or discard every Ferrum document before closing Ferrum.",
			))
			return
		self._prepare_native_view_controls_shutdown()
		for tab in tuple(self._native_tabs_by_page.values()):
			try:
				tab.dispose()
			except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabError:
				event.ignore()
				self._show_edit_refusal(self._typed_refusal(
					"close_document", "busy_close",
					"Ferrum could not retire the live SMARTS result; refresh before closing Ferrum.",
				))
				return
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				self._retire_closed_native_tab(tab, index)
		event.accept()
