"""Public Ferrum Qt window for the completed CDML slice."""
# Standard Library
import dataclasses
import functools
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_save
import ferrum_qt.bridge.insertion_placement
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.drawing_standard as native_drawing_standard
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.atom_properties
import ferrum_qt.ferrum.atom_element
import ferrum_qt.ferrum.atom_number
import ferrum_qt.ferrum.arrow_properties
import ferrum_qt.ferrum.bond_properties
import ferrum_qt.ferrum.clipboard
import ferrum_qt.ferrum.local_document_open
import ferrum_qt.ferrum.coordinate_generation
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties
import ferrum_qt.ferrum.geometry_actions
import ferrum_qt.ferrum.line_tools
import ferrum_qt.ferrum.structure_selection
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.ferrum.haworth_tool
import ferrum_qt.ferrum.direct_glycosidic_haworth_tool
import ferrum_qt.ferrum.linear_form
import ferrum_qt.ferrum.main_window_support
import ferrum_qt.ferrum.main_window_lifecycle
import ferrum_qt.ferrum.explicit_fragments
import ferrum_qt.ferrum.molecule_imports
import ferrum_qt.ferrum.molecule_exports
import ferrum_qt.ferrum.molfile_export
import ferrum_qt.ferrum.sdf_export
import ferrum_qt.ferrum.molecule_report
import ferrum_qt.ferrum.molecule_name
import ferrum_qt.ferrum.snapshot_export
import ferrum_qt.ferrum.recovery_export
import ferrum_qt.ferrum.selection_svg
import ferrum_qt.ferrum.interaction_action_handoff
import ferrum_qt.ferrum.smarts_query_dock
import ferrum_qt.ferrum.view_controls
import ferrum_qt.ferrum.user_templates as native_user_templates
import ferrum_qt.ferrum.catalog_palette as native_catalog_palette
import ferrum_qt.ferrum.presentation_properties
import ferrum_qt.ferrum.property_dock
import ferrum_qt.ferrum.paper_properties as native_paper_properties
import ferrum_qt.ferrum.wavy_properties as native_wavy_properties
import ferrum_qt.dialogs.atom_dialog
import ferrum_qt.widgets.status_bar
_ATOM_MARK_ACTIONS = (
	("plus", "Circled Plus"),
	("minus", "Circled Minus"),
	("radical", "Radical"),
	("biradical", "Biradical"),
	("electronpair", "Electron Pair"),
	("dotted_electronpair", "Dotted Electron Pair"),
	("pz_orbital", "p Orbital"),
)
#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _AtomInsertionIntent:
	"""One revision-bound Ferrum click request awaiting an exact scene point."""

	tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	molecule_object_id: str
	element: str

#============================================
class FerrumNativeMainWindow(
		ferrum_qt.ferrum.main_window_lifecycle.
		FerrumNativeMainWindowLifecycleMixin,
		native_catalog_palette.FerrumNativeCatalogPlacementWindowMixin,
		ferrum_qt.ferrum.window_mode_sync.FerrumNativeWindowModeSyncMixin,
		ferrum_qt.ferrum.explicit_fragments.
		FerrumNativeExplicitFragmentsWindowMixin,
		ferrum_qt.ferrum.haworth_tool.FerrumNativeHaworthToolMixin,
		ferrum_qt.ferrum.direct_glycosidic_haworth_tool.
		FerrumNativeDirectGlycosidicHaworthWindowMixin,
		native_user_templates.FerrumNativeUserTemplateWindowMixin,
		ferrum_qt.ferrum.view_controls.FerrumNativeViewControlsMixin,
		ferrum_qt.ferrum.selection_svg.FerrumNativeSelectionSvgWindowMixin,
		ferrum_qt.ferrum.clipboard.FerrumNativeClipboardWindowMixin,
		ferrum_qt.ferrum.local_document_open.FerrumNativeLocalDocumentOpenMixin,
		ferrum_qt.ferrum.document_save.FerrumNativeDocumentSaveMixin,
		ferrum_qt.ferrum.recovery_export.FerrumNativeRecoveryExportWindowMixin,
		ferrum_qt.ferrum.snapshot_export.FerrumNativeSnapshotExportWindowMixin,
		ferrum_qt.ferrum.structure_selection.FerrumNativeStructureSelectionMixin,
		ferrum_qt.ferrum.line_tools.FerrumNativeLineToolsMixin,
		ferrum_qt.ferrum.molecule_imports.FerrumNativeMoleculeImportsMixin,
		ferrum_qt.ferrum.sdf_export.FerrumNativeSdfExportMixin,
		ferrum_qt.ferrum.molfile_export.FerrumNativeMolfileExportMixin,
		ferrum_qt.ferrum.molecule_exports.FerrumNativeMoleculeExportsMixin,
		ferrum_qt.ferrum.molecule_report.FerrumNativeMoleculeReportMixin,
		ferrum_qt.ferrum.molecule_name.FerrumNativeMoleculeNameWindowMixin,
		ferrum_qt.ferrum.linear_form.FerrumNativeLinearFormWindowMixin,
		ferrum_qt.ferrum.coordinate_generation.
		FerrumNativeCoordinateGenerationWindowMixin,
		ferrum_qt.ferrum.geometry_actions.FerrumNativeGeometryActionsMixin,
		ferrum_qt.ferrum.main_window_support.NativeOnlyFileFallback,
		PySide6.QtWidgets.QMainWindow,
		):
	"""A standalone public host for Rust-owned CDML tabs only.

	This window intentionally has no alternate session registry or backend
	fallback. It owns the ordinary product's vertical Rust
	open/render/save/reopen path.
	"""

	local_document_open_completed = PySide6.QtCore.Signal(str, bool)
	local_document_open_queue_drained = PySide6.QtCore.Signal(bool)

	def __init__(
			self, parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: str | pathlib.Path | None = None,
			) -> None:
		"""Build the small Ferrum document host and its reachable file actions."""
		super().__init__(parent)
		self._interaction_action_handoff = (
			ferrum_qt.ferrum.interaction_action_handoff.
			FerrumInteractionActionHandoff(
				self, self._present_interaction_action_handoff_failure_v1,
			)
		)
		getattr(self, "_initialize_native_file_menu_clients", lambda: None)()
		self._native_tabs_by_page = {}
		self._drawing_parameters = (
			ferrum_qt.ferrum.drawing_parameters.
			FerrumNativeDrawingParameters()
		)
		self._initialize_native_user_templates(user_template_directory)
		self._initialize_catalog_placement()
		self._initialize_local_document_open()
		self._initialize_view_controls()
		self._atom_insertion_intent: _AtomInsertionIntent | None = None
		self._initialize_line_tools()
		self._initialize_structure_selection()
		self._initialize_molecule_imports()
		self._initialize_sdf_exports()
		self._initialize_molfile_exports()
		self._initialize_molecule_exports()
		self._initialize_molecule_inspection()
		self._initialize_native_clipboard()
		self._initialize_coordinate_generation()
		self._initialize_snapshot_exports()
		self._tab_widget = PySide6.QtWidgets.QTabWidget(self)
		self._tab_widget.setTabsClosable(True)
		self._last_native_tab = None
		self._tab_widget.currentChanged.connect(self._on_native_tab_changed)
		self._tab_widget.tabCloseRequested.connect(self._close_tab_at)
		self.setCentralWidget(self._tab_widget)
		self.setWindowTitle(self.tr("Ferrum"))
		self.resize(1000, 700)
		self._build_actions()
		self._smarts_query_controller = (
			ferrum_qt.ferrum.smarts_query_dock.FerrumSmartsQueryController(self)
		)
		self._smarts_query_action = self._smarts_query_controller.install_action(
			self._chemistry_menu,
		)
		self._native_property_dock = (
			ferrum_qt.ferrum.property_dock.install_native_property_dock(
				self, self._edit_atom_properties_action, self._edit_bond_properties_action,
			)
		)
		self.setStatusBar(ferrum_qt.widgets.status_bar.StatusBar(self))
		self._install_native_view_status_controls()
		self._refresh_actions()

	def _build_actions(self) -> None:
		"""Create Ferrum file and bounded Rust edit actions."""
		menu = self.menuBar().addMenu(self.tr("File"))
		self._file_menu = menu
		self._open_action = PySide6.QtGui.QAction(self.tr("Open"), self)
		self._open_action.triggered.connect(self._on_open)
		menu.addAction(self._open_action)
		self._build_open_in_current_tab_action(menu)
		self._build_local_document_open_action(menu)
		getattr(self, "_install_native_recent_files_menu", lambda _menu: None)(menu)
		self._save_action = PySide6.QtGui.QAction(self.tr("Save"), self)
		self._save_action.triggered.connect(self._on_save)
		menu.addAction(self._save_action)
		self._save_as_action = PySide6.QtGui.QAction(self.tr("Save As"), self)
		self._save_as_action.triggered.connect(self._on_save_as)
		menu.addAction(self._save_as_action)
		self._build_recovery_export_action(menu)
		self._build_snapshot_export_actions(menu)
		self._build_native_user_template_file_actions(menu)
		self._close_action = PySide6.QtGui.QAction(self.tr("Close Tab"), self)
		self._close_action.triggered.connect(self._close_current_tab)
		menu.addAction(self._close_action)
		menu.addSeparator()
		self._quit_action = PySide6.QtGui.QAction(self.tr("Quit"), self)
		self._quit_action.triggered.connect(self.close)
		menu.addAction(self._quit_action)
		edit_menu = self.menuBar().addMenu(self.tr("Edit"))
		self._edit_menu = edit_menu
		self._paper_properties_action = native_paper_properties.install_paper_properties_action(
			self, edit_menu,
		)
		self._drawing_standard_action = native_drawing_standard.install_drawing_standard_action(
			self, edit_menu,
		)
		self._change_element_action = PySide6.QtGui.QAction(self.tr("Change Element"), self)
		self._change_element_action.triggered.connect(self._on_change_element)
		edit_menu.addAction(self._change_element_action)
		self._edit_atom_properties_action = PySide6.QtGui.QAction(
			self.tr("Edit Atom Properties"), self,
		)
		self._edit_atom_properties_action.setToolTip(
			self.tr("Edit one selected durable atom through one operation"),
		)
		self._edit_atom_properties_action.triggered.connect(self._on_edit_atom_properties)
		edit_menu.addAction(self._edit_atom_properties_action)
		self._set_atom_number_action = PySide6.QtGui.QAction(
			self.tr("Set Atom Number"), self,
		)
		self._set_atom_number_action.triggered.connect(self._on_set_atom_number)
		edit_menu.addAction(self._set_atom_number_action)
		self._clear_atom_number_action = PySide6.QtGui.QAction(
			self.tr("Clear Atom Number"), self,
		)
		self._clear_atom_number_action.triggered.connect(self._on_clear_atom_number)
		edit_menu.addAction(self._clear_atom_number_action)
		mark_menu = edit_menu.addMenu(self.tr("Toggle Atom Mark"))
		self._atom_mark_actions = {}
		for kind_name, label in _ATOM_MARK_ACTIONS:
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.setToolTip(self.tr(
				"Add this mark, or remove the first matching mark from the selected atom",
			))
			action.triggered.connect(functools.partial(self._on_toggle_atom_mark, kind_name))
			mark_menu.addAction(action)
			self._atom_mark_actions[kind_name] = action
		mark_menu.addSeparator()
		self._remove_atom_mark_action = PySide6.QtGui.QAction(
			self.tr("Remove Atom Mark..."), self,
		)
		self._remove_atom_mark_action.setToolTip(
			self.tr("Choose one exact mark ordinal from the selected Rust atom"),
		)
		self._remove_atom_mark_action.triggered.connect(self._on_remove_atom_mark)
		mark_menu.addAction(self._remove_atom_mark_action)
		self._edit_bond_properties_action = PySide6.QtGui.QAction(
			self.tr("Edit Bond Properties"), self,
		)
		self._edit_bond_properties_action.setToolTip(
			self.tr("Edit one selected durable bond through one operation"),
		)
		self._edit_bond_properties_action.triggered.connect(self._on_edit_bond_properties)
		edit_menu.addAction(self._edit_bond_properties_action)
		self._edit_plus_properties_action = (
			ferrum_qt.ferrum.presentation_properties.install_plus_properties_action(
				self, edit_menu,
			)
		)
		self._edit_arrow_properties_action = (
			ferrum_qt.ferrum.arrow_properties.install_arrow_properties_action(
				self, edit_menu,
			)
		)
		self._edit_geometric_properties_action = (
			native_geometric_properties.install_geometric_properties_action(self, edit_menu)
		)
		self._edit_wavy_properties_action = (
			native_wavy_properties.install_wavy_properties_action(self, edit_menu)
		)
		self._delete_atom_action = PySide6.QtGui.QAction(
			self.tr("Delete Selected Atom"), self,
		)
		self._delete_atom_action.setToolTip(
			self.tr("Delete one atom and every incident bond through Rust"),
		)
		self._delete_atom_action.triggered.connect(self._on_delete_selected_atom)
		edit_menu.addAction(self._delete_atom_action)
		self._delete_bond_action = PySide6.QtGui.QAction(
			self.tr("Delete Selected Bond"), self,
		)
		self._delete_bond_action.setToolTip(
			self.tr("Delete one durable typed bond through Rust"),
		)
		self._delete_bond_action.triggered.connect(self._on_delete_selected_bond)
		edit_menu.addAction(self._delete_bond_action)
		self._change_bond_order_action = PySide6.QtGui.QAction(
			self.tr("Change Selected Bond Order"), self,
		)
		self._change_bond_order_action.setToolTip(
			self.tr("Choose single, double, or triple for one durable Rust bond"),
		)
		self._change_bond_order_action.triggered.connect(self._on_change_bond_order)
		edit_menu.addAction(self._change_bond_order_action)
		self._add_atom_action = PySide6.QtGui.QAction(self.tr("Add Atom at Point"), self)
		self._add_atom_action.setCheckable(True)
		self._add_atom_action.setToolTip(
			self.tr("Use Next atom, then click the canvas once; Esc cancels"),
		)
		self._connect_interaction_action_v1(
			self._add_atom_action, self._on_toggle_add_atom,
		)
		self._add_interaction_action_to_menu_v1(edit_menu, self._add_atom_action)
		self._add_single_bond_action = PySide6.QtGui.QAction(
			self.tr("Add Single Bond Between Selected Atoms"), self,
		)
		self._add_single_bond_action.setToolTip(
			self.tr("Select exactly two atoms, then connect them through Rust"),
		)
		self._add_single_bond_action.triggered.connect(self._on_add_single_bond)
		edit_menu.addAction(self._add_single_bond_action)
		self._build_line_tool_actions(edit_menu)
		self._build_structure_selection_action(edit_menu)
		self._build_top_level_transform_actions(edit_menu)
		self._undo_action = PySide6.QtGui.QAction(self.tr("Undo"), self)
		self._undo_action.triggered.connect(self._on_undo)
		edit_menu.addAction(self._undo_action)
		self._redo_action = PySide6.QtGui.QAction(self.tr("Redo"), self)
		self._redo_action.triggered.connect(self._on_redo)
		edit_menu.addAction(self._redo_action)
		self._build_native_clipboard_actions(edit_menu)
		edit_menu.addSeparator()
		self._refresh_action = PySide6.QtGui.QAction(self.tr("Refresh Authoritative View"), self)
		self._refresh_action.triggered.connect(self._on_refresh_authoritative)
		edit_menu.addAction(self._refresh_action)
		chemistry_menu = self.menuBar().addMenu(self.tr("Chemistry"))
		self._chemistry_menu = chemistry_menu
		self._build_catalog_template_action(chemistry_menu)
		self._build_native_user_template_place_action(chemistry_menu)
		self._build_molecule_import_actions(chemistry_menu)
		self._build_sdf_export_actions(chemistry_menu)
		self._build_molfile_export_actions(chemistry_menu)
		self._build_molecule_export_actions(chemistry_menu)
		self._build_molecule_inspection_actions(chemistry_menu)
		self._build_molecule_name_action(chemistry_menu)
		self._build_linear_form_action(chemistry_menu)
		self._build_explicit_fragment_actions(chemistry_menu)
		self._build_direct_glycosidic_haworth_action(chemistry_menu)
		self._build_coordinate_generation_actions(chemistry_menu)
		self._build_view_controls_actions()
		self._wire_catalog_tool_replacement()

	#============================================
	def _connect_interaction_action_v1(self, action: PySide6.QtGui.QAction,
			handler: object) -> None:
		"""Register one action whose handler takes canvas interaction ownership."""
		self._interaction_action_handoff.connect(action, handler)

	def _add_interaction_action_to_menu_v1(self, menu: PySide6.QtWidgets.QMenu,
			action: PySide6.QtGui.QAction) -> None:
		"""Insert one pointer action only after its popup lifecycle is ready."""
		self._interaction_action_handoff.add_registered_action_to_menu(menu, action)

	#============================================
	def _set_interaction_capture_canceller_v1(self, canceller: object | None) -> None:
		"""Bind the current selected-root capture without exposing its state."""
		self._interaction_action_handoff.set_capture_canceller(canceller)

	def _present_interaction_action_handoff_failure_v1(self, detail: str) -> None:
		"""Present one shared handoff failure through the ordinary typed refusal route."""
		self._show_edit_refusal(self._typed_refusal(
			"edit_document", "unavailable_operation", detail,
		))
		self._refresh_actions()

	def _on_change_element(self) -> None:
		"""Collect one element spelling and submit it only to the active Ferrum tab."""
		ferrum_qt.ferrum.atom_element.run_change_selected_atom_element_dialog(self)
		self._refresh_actions()

	def _on_edit_atom_properties(self) -> None:
		"""Use the shared visual form while keeping the Rust session authoritative."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			atom = tab.selected_atom_projection()
			model = ferrum_qt.ferrum.atom_properties.dialog_model_from_projection(
				atom,
			)
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		dialog = ferrum_qt.dialogs.atom_dialog.AtomDialog(model, self)
		if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			changes = (
				ferrum_qt.ferrum.atom_properties.
				property_changes_from_dialog(dialog.changes())
			)
			tab.apply_selected_atom_properties(changes)
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Updated one Ferrum atom."), 5000)
		self._refresh_actions()

	def _on_set_atom_number(self) -> None:
		"""Collect and assign one persistent number through the active Rust tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			atom = tab.selected_atom_projection()
			dialog = (
				ferrum_qt.ferrum.atom_number.
				FerrumNativeAtomNumberDialog(atom.number, atom.show_number, self)
			)
		except Exception as exc:
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			number, show_number = dialog.assignment()
			tab.set_selected_atom_number(number, show_number)
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Updated one Ferrum atom number."), 5000)
		self._refresh_actions()

	def _on_clear_atom_number(self) -> None:
		"""Clear one selected persistent number through the active Rust tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			tab.clear_selected_atom_number()
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Cleared one Ferrum atom number."), 5000)
		self._refresh_actions()

	def _on_toggle_atom_mark(self, kind_name: str, _checked: bool = False) -> None:
		"""Toggle one closed mark kind through the active authoritative Rust tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			kind = getattr(engine.AtomMarkKindV1, kind_name)
			tab.toggle_selected_atom_mark(kind)
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Toggled one Ferrum atom mark."), 5000)
		self._refresh_actions()

	def _on_remove_atom_mark(self) -> None:
		"""Choose and remove one exact same-type ordinal from a selected Rust atom."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			marks = tab.selected_atom_marks()
			choices = tuple(
				f"{self._atom_mark_label(mark.kind)} #{mark.same_type_ordinal + 1}"
				for mark in marks
			)
			selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
				self, self.tr("Remove Atom Mark"), self.tr("Mark:"),
				choices, 0, False,
			)
			if not accepted:
				return
			mark = marks[choices.index(selected)]
			import ferrum_qt.ferrum.engine as engine
			tab.apply_selected_atom_mark(
				engine.AtomMarkActionV1.remove,
				mark.kind,
				mark.same_type_ordinal,
			)
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Removed one Ferrum atom mark."), 5000)
		self._refresh_actions()

	def _atom_mark_label(self, kind: object) -> str:
		"""Return one UI label for an exact closed extension mark value."""
		import ferrum_qt.ferrum.engine as engine
		for kind_name, label in _ATOM_MARK_ACTIONS:
			if kind == getattr(engine.AtomMarkKindV1, kind_name):
				return self.tr(label)
		raise TypeError("Ferrum atom mark projection contains an unknown kind")

	def _on_edit_bond_properties(self) -> None:
		"""Delegate the bounded visual form to its focused Ferrum adapter."""
		ferrum_qt.ferrum.bond_properties.run_bond_properties_dialog(self)

	def _on_delete_selected_atom(self) -> None:
		"""Delete one selected atom through the active Rust document session."""
		tab = self._active_native_tab()
		if tab is None:
			return
		self._cancel_atom_insertion()
		self._cancel_line_gesture()
		try:
			tab.delete_selected_atom()
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(
			self.tr("Deleted one Ferrum atom and its incident bonds."), 5000,
		)
		self._refresh_actions()

	def _on_delete_selected_bond(self) -> None:
		"""Delete one selected bond through the active Rust document session."""
		tab = self._active_native_tab()
		if tab is None:
			return
		self._cancel_atom_insertion()
		self._cancel_line_gesture()
		try:
			tab.delete_selected_bond()
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Deleted one Ferrum bond."), 5000)
		self._refresh_actions()

	def _on_change_bond_order(self) -> None:
		"""Choose and submit one closed Rust bond-order value."""
		tab = self._active_native_tab()
		if tab is None:
			return
		self._cancel_atom_insertion()
		self._cancel_line_gesture()
		labels = (self.tr("Single"), self.tr("Double"), self.tr("Triple"))
		selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
			self, self.tr("Change Bond Order"), self.tr("Bond order:"),
			labels, 0, False,
		)
		if not accepted:
			return
		import ferrum_qt.ferrum.engine as engine
		orders = (
			engine.DocumentBondOrderV1.single,
			engine.DocumentBondOrderV1.double,
			engine.DocumentBondOrderV1.triple,
		)
		try:
			tab.set_selected_bond_order(orders[labels.index(selected)])
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(
			self.tr("Changed one Ferrum bond to {0}.").format(selected.lower()), 5000,
		)
		self._refresh_actions()

	def _on_toggle_add_atom(self, checked: bool) -> None:
		"""Capture one chosen element intent, then wait for one scene click."""
		if not checked:
			self._cancel_atom_insertion()
			return
		self._cancel_line_gesture()
		tab = self._active_native_tab()
		if tab is None:
			self._cancel_atom_insertion()
			return
		choices = tab.canvas_authorable_molecule_choices()
		if not choices:
			self._cancel_atom_insertion()
			self._show_edit_refusal(self._typed_refusal(
				"edit_document", "unrenderable_molecule",
				"The installed Rust render observation has no canvas-authorable molecule plan.",
			))
			# QAction's shortcut toggles its checked state after the triggered slot
			# returns.  Settle the refused shortcut on the next Qt turn as well, so
			# the visible tool state cannot disagree with the absent insertion intent.
			PySide6.QtCore.QTimer.singleShot(0, self._cancel_atom_insertion)
			return
		drawing = self._drawing_parameters.snapshot()
		choice = choices[0]
		if len(choices) > 1:
			labels = tuple(item.label for item in choices)
			selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
				self, self.tr("Choose Molecule"), self.tr("Target molecule:"),
				labels, 0, False,
			)
			if not accepted:
				self._cancel_atom_insertion()
				return
			choice = choices[labels.index(selected)]
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		self._atom_insertion_intent = _AtomInsertionIntent(
			tab, viewport, snapshot.revision, snapshot.digest, choice.object_id,
			drawing.element,
		)
		self._synchronize_mode_state("atom")
		self._add_atom_action.setToolTip(self.tr(
			"Add {0} at the next canvas click; Escape cancels."
		).format(drawing.element))
		self._refresh_cancel_tool_action()
		viewport.installEventFilter(self)
		viewport.setFocus()
		tab.view.show_keyboard_cursor()
		self.statusBar().showMessage(
			self.tr(
				"Click once or use Arrow keys and Enter to add {0}; Shift+Arrow is fine; "
				"Esc cancels Add Atom.".format(drawing.element),
			),
		)

	def _complete_atom_insertion(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Submit one still-current captured point through the Rust transaction."""
		intent = self._atom_insertion_intent
		if intent is None:
			return
		tab = intent.tab
		snapshot = tab.current_snapshot
		if (
			self._active_native_tab() is not tab
			or tab.requires_refresh
			or snapshot.revision != intent.revision
			or snapshot.digest != intent.digest
		):
			self._cancel_atom_insertion()
			self._show_edit_refusal(self._typed_refusal(
				"use_tool", "stale_tool",
				"The document changed before the click; start Add Atom again.",
			))
			return
		try:
			point = tab.view.snap_authored_scene_point(
				tab.view.mapToScene(event.position().toPoint()),
			)
			self._cancel_atom_insertion(clear_status=False)
			tab.add_atom_at(
				intent.molecule_object_id, intent.element, float(point.x()), float(point.y()),
			)
		except Exception as exc:
			self._cancel_atom_insertion(clear_status=False)
			self._refresh_actions()
			self.statusBar().clearMessage()
			self._show_atom_insertion_refusal(exc)
			return
		self.statusBar().showMessage(self.tr("Added one free-standing Rust atom."), 5000)
		self._refresh_actions()

	#============================================
	def _show_atom_insertion_refusal(self, exc: Exception) -> None:
		"""Present the canvas-plan refusal identically for pointer and keyboard use."""
		outcome = "unrenderable_molecule" if isinstance(
			exc,
			ferrum_qt.ferrum.document_tab.
			FerrumNativeDocumentTabUnrenderableMoleculeError,
		) else "unavailable_operation"
		self._show_edit_refusal(self._typed_refusal("edit_document", outcome, str(exc)))

	#============================================
	def _cancel_atom_insertion(self, clear_status: bool = True) -> None:
		"""Release the one Ferrum viewport event boundary without changing Rust state."""
		intent = self._atom_insertion_intent
		self._atom_insertion_intent = None
		self._add_atom_action.setChecked(False)
		if intent is not None:
			intent.viewport.removeEventFilter(self)
			intent.tab.view.hide_keyboard_cursor()
		if clear_status:
			self.statusBar().clearMessage()
		self._refresh_cancel_tool_action()
		self._synchronize_mode_state()

	#============================================
	def _on_undo(self) -> None:
		"""Attempt the Rust undo operation and report typed unavailable history truth."""
		self._run_native_history_action("undo")

	#============================================
	def _on_add_single_bond(self) -> None:
		"""Connect exactly two selected atoms through the authoritative Rust session."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			tab.add_single_bond_between_selected_atoms()
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
			return
		self.statusBar().showMessage(self.tr("Added one Ferrum single bond."), 5000)
		self._refresh_actions()

	#============================================
	def _on_redo(self) -> None:
		"""Attempt the Rust redo operation and report typed unavailable history truth."""
		self._run_native_history_action("redo")

	#============================================
	def _run_native_history_action(self, name: str) -> None:
		"""Retire revision-bound pointer state before one Rust history transition."""
		tab = self._active_native_tab()
		if tab is None:
			return
		if not self._cancel_line_gesture():
			self._refresh_actions()
			return
		try:
			getattr(tab, name)()
		except Exception as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal(
				"edit_document", "no_undo", str(exc),
			))
			return
		self._refresh_actions()

	#============================================
	def _on_refresh_authoritative(self) -> None:
		"""Retry installation of one Rust-accepted observation after a display failure."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			if not tab.refresh_authoritative():
				self._show_edit_refusal(self._typed_refusal(
					"edit_document", "unavailable_operation",
					"Rust remains authoritative; the previous view is still retained.",
				))
		except Exception as exc:
			self._show_edit_refusal(self._typed_refusal("edit_document", "unavailable_operation", str(exc)))
		self._refresh_actions()

	#============================================
