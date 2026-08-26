"""Public Ferrum Qt window for the completed CDML slice."""
# Standard Library
import collections.abc
import functools
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.actions.action_registry
import ferrum_qt.ferrum.document_save
import ferrum_qt.bridge.insertion_placement
import ferrum_qt.canvas.graphics_disposal
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors
import ferrum_qt.ferrum.drawing_standard as native_drawing_standard
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.atom_properties
import ferrum_qt.ferrum.atom_mode
import ferrum_qt.ferrum.atom_element
import ferrum_qt.ferrum.atom_number
import ferrum_qt.ferrum.arrow_properties
import ferrum_qt.ferrum.bond_properties
import ferrum_qt.ferrum.clipboard
import ferrum_qt.ferrum.local_document_open
import ferrum_qt.ferrum.local_document_open_types
import ferrum_qt.ferrum.coordinate_generation
import ferrum_qt.ferrum.geometric_properties as native_geometric_properties
import ferrum_qt.ferrum.geometry_actions
import ferrum_qt.ferrum.line_tools
import ferrum_qt.ferrum.structure_selection
import ferrum_qt.ferrum.window_mode_sync
import ferrum_qt.modes.base_mode
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
import ferrum_qt.ferrum.sdf_multi_export
import ferrum_qt.ferrum.molecule_report
import ferrum_qt.ferrum.molecule_diagnostics
import ferrum_qt.ferrum.bond_capacity
import ferrum_qt.ferrum.atom_oxidation
import ferrum_qt.ferrum.explicit_hydrogen
import ferrum_qt.ferrum.compact_group_materialization
import ferrum_qt.ferrum.compact_group_authoring
import ferrum_qt.ferrum.free_compact_group_placement
import ferrum_qt.ferrum.document_installation
import ferrum_qt.ferrum.operation_presentation
import ferrum_qt.ferrum.molecule_name
import ferrum_qt.ferrum.snapshot_export
import ferrum_qt.ferrum.recovery_export
import ferrum_qt.ferrum.selection_svg
import ferrum_qt.ferrum.interaction_action_handoff
import ferrum_qt.ferrum.smarts_query_dock
import ferrum_qt.ferrum.smarts_selected_root_contract
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
#============================================
class FerrumNativeMainWindow(
		ferrum_qt.ferrum.bond_capacity.FerrumNativeBondCapacityMixin,
		ferrum_qt.ferrum.molecule_diagnostics.FerrumNativeMoleculeDiagnosticsMixin,
		ferrum_qt.ferrum.free_compact_group_placement.
		FerrumNativeFreeCompactGroupPlacementWindowMixin,
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
		ferrum_qt.ferrum.sdf_multi_export.FerrumNativeMultiSdfExportMixin,
		ferrum_qt.ferrum.sdf_export.FerrumNativeSdfExportMixin,
		ferrum_qt.ferrum.molfile_export.FerrumNativeMolfileExportMixin,
		ferrum_qt.ferrum.molecule_exports.FerrumNativeMoleculeExportsMixin,
		ferrum_qt.ferrum.molecule_report.FerrumNativeMoleculeReportMixin,
		ferrum_qt.ferrum.atom_oxidation.FerrumNativeAtomOxidationMixin,
		ferrum_qt.ferrum.explicit_hydrogen.FerrumNativeExplicitHydrogenWindowMixin,
		ferrum_qt.ferrum.compact_group_materialization.
		FerrumNativeCompactGroupMaterializationWindowMixin,
		ferrum_qt.ferrum.compact_group_authoring.
		FerrumNativeCompactGroupAuthoringWindowMixin,
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
	operation_presentation_completed = PySide6.QtCore.Signal(object)
	document_installation_completed = PySide6.QtCore.Signal(object)

	def __init__(
			self, parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: str | pathlib.Path | None = None,
			) -> None:
		"""Build the small Ferrum document host and its reachable file actions."""
		super().__init__(parent)
		self._action_registry = ferrum_qt.actions.action_registry.ActionRegistry()
		self._window_mode_sync = ferrum_qt.ferrum.window_mode_sync.FerrumWindowModeSync(
			self._action_registry,
		)
		self._controller_native_viewport = None
		self._window_mode_sync.set_native_input_host(self)
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
		self._atom_mode = ferrum_qt.ferrum.atom_mode.FerrumAtomModeFeature(self)
		self._initialize_native_user_templates(user_template_directory)
		self._initialize_catalog_placement()
		self._local_ingress_registry = (
			ferrum_qt.ferrum.local_document_open_types.
			FerrumNativeLocalIngressRegistryV1.from_rust()
		)
		self._initialize_local_document_open()
		self._initialize_view_controls()
		self._atom_insertion_intent: ferrum_qt.ferrum.atom_mode.AtomInsertionIntent | None = None
		self._initialize_line_tools()
		self._initialize_structure_selection()
		self._initialize_molecule_imports()
		self._initialize_multi_sdf_exports()
		self._initialize_sdf_exports()
		self._initialize_molfile_exports()
		self._initialize_molecule_exports()
		self._initialize_molecule_inspection()
		self._initialize_molecule_diagnostics()
		self._initialize_bond_capacity()
		self._initialize_atom_oxidation()
		self._initialize_explicit_hydrogen()
		self._initialize_compact_group_materialization()
		self._initialize_compact_group_authoring()
		self._initialize_free_compact_group_placement()
		self._initialize_native_clipboard()
		self._initialize_coordinate_generation()
		self._initialize_snapshot_exports()
		self._tab_widget = PySide6.QtWidgets.QTabWidget(self)
		self._native_selection_refresh_timer = PySide6.QtCore.QTimer(
			self._tab_widget,
		)
		self._native_selection_refresh_timer.setSingleShot(True)
		self._native_selection_refresh_timer.timeout.connect(
			self._refresh_after_native_selection,
		)
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
		self._smarts_query_action = self._smarts_query_controller.install_action()
		self._native_property_dock = (
			ferrum_qt.ferrum.property_dock.install_native_property_dock(
				self, self._edit_atom_properties_action, self._edit_bond_properties_action,
			)
		)
		self.setStatusBar(ferrum_qt.widgets.status_bar.StatusBar(self))
		self._install_native_view_status_controls()
		self._refresh_actions()

	#============================================
	def _queue_operation_presentation_v1(self, tab: object, operation_kind: str,
			terminal_kind: str, document_effect: str, source_revision: int,
			source_digest_hex: str) -> None:
		"""Publish after a modeless operation outcome receives one Qt event turn."""
		PySide6.QtCore.QTimer.singleShot(0, functools.partial(
			self._publish_operation_presentation_v1,
			tab, operation_kind, terminal_kind, document_effect,
			source_revision, source_digest_hex,
		))

	#============================================
	def _publish_operation_presentation_v1(self, tab: object, operation_kind: str,
			terminal_kind: str, document_effect: str, source_revision: int,
			source_digest_hex: str) -> bool:
		"""Emit one visible operation receipt only for its still-live source tab."""
		if PySide6.QtCore.QThread.currentThread() != self.thread():
			return False
		if (
			tab is None
			or self._native_tabs_by_page.get(tab) is not tab
			or tab.is_disposed
		):
			return False
		try:
			snapshot = tab.current_snapshot
		except native_document_tab_errors.FerrumNativeDocumentTabError:
			return False
		if document_effect == "unchanged" and (
			snapshot.revision != source_revision or snapshot.digest != source_digest_hex
		):
			return False
		if document_effect == "updated" and (
			snapshot.revision == source_revision and snapshot.digest == source_digest_hex
		):
			return False
		receipt = ferrum_qt.ferrum.operation_presentation.FerrumOperationPresentationV1(
			ferrum_qt.ferrum.operation_presentation.SCHEMA,
			operation_kind,
			terminal_kind,
			document_effect,
			source_revision,
			source_digest_hex,
			snapshot.revision,
			snapshot.digest,
		)
		self.operation_presentation_completed.emit(receipt)
		return True

	#============================================
	def _publish_document_installation_v1(self, tab: object,
			installation_kind: str, source_revision: int, source_digest_hex: str,
			expected_revision: int, expected_digest_hex: str,
			installed_record_count: int) -> bool:
		"""Publish one receipt after its exact Rust target reaches the live scene."""
		if PySide6.QtCore.QThread.currentThread() != self.thread():
			return False
		if (
			tab is None
			or self._native_tabs_by_page.get(tab) is not tab
			or tab.is_disposed
		):
			return False
		try:
			snapshot = tab.current_snapshot
		except native_document_tab_errors.FerrumNativeDocumentTabError:
			return False
		if (
			snapshot.revision != expected_revision
			or snapshot.digest != expected_digest_hex
		):
			return False
		summary = (
			ferrum_qt.ferrum.document_installation.
			accessible_summary_for_installation_kind(
				installation_kind, installed_record_count,
			)
		)
		receipt = ferrum_qt.ferrum.document_installation.FerrumDocumentInstallationV1(
			ferrum_qt.ferrum.document_installation.SCHEMA,
			installation_kind,
			source_revision,
			source_digest_hex,
			snapshot.revision,
			snapshot.digest,
			installed_record_count,
			summary,
		)
		self.document_installation_completed.emit(receipt)
		return True

	def _build_actions(self) -> None:
		"""Create Ferrum file and bounded Rust edit actions."""
		self._open_action = PySide6.QtGui.QAction(self.tr("Open"), self)
		self._open_action.triggered.connect(self._on_open)
		self._build_open_in_current_tab_action()
		self._build_local_document_open_action()
		self._register_action("file.open", self._open_action)
		getattr(self, "_install_native_recent_files_menu", lambda: None)()
		self._save_action = PySide6.QtGui.QAction(self.tr("Save"), self)
		self._save_action.triggered.connect(self._on_save)
		self._register_action("file.save", self._save_action)
		self._save_as_action = PySide6.QtGui.QAction(self.tr("Save As"), self)
		self._save_as_action.triggered.connect(self._on_save_as)
		self._register_action("file.save_as", self._save_as_action)
		self._build_recovery_export_action()
		self._build_snapshot_export_actions()
		self._build_native_user_template_file_actions()
		self._close_action = PySide6.QtGui.QAction(self.tr("Close Tab"), self)
		self._close_action.triggered.connect(self._close_current_tab)
		self._register_action("file.close", self._close_action)
		self._quit_action = PySide6.QtGui.QAction(self.tr("Quit"), self)
		self._quit_action.triggered.connect(self.close)
		self._register_action("file.quit", self._quit_action)
		self._drawing_standard_action = native_drawing_standard.install_drawing_standard_action(
			self,
		)
		self._paper_properties_action = native_paper_properties.install_paper_properties_action(
			self,
		)
		self._change_element_action = PySide6.QtGui.QAction(self.tr("Change Element"), self)
		self._change_element_action.triggered.connect(self._on_change_element)
		self._register_action("edit.atom.change_element", self._change_element_action)
		self._edit_atom_properties_action = PySide6.QtGui.QAction(
			self.tr("Edit Atom Properties"), self,
		)
		self._edit_atom_properties_action.setToolTip(
			self.tr("Edit one selected durable atom through one operation"),
		)
		self._edit_atom_properties_action.triggered.connect(self._on_edit_atom_properties)
		self._register_action("edit.atom.properties", self._edit_atom_properties_action)
		self._set_atom_number_action = PySide6.QtGui.QAction(
			self.tr("Set Atom Number"), self,
		)
		self._set_atom_number_action.triggered.connect(self._on_set_atom_number)
		self._register_action("edit.atom.set_number", self._set_atom_number_action)
		self._clear_atom_number_action = PySide6.QtGui.QAction(
			self.tr("Clear Atom Number"), self,
		)
		self._clear_atom_number_action.triggered.connect(self._on_clear_atom_number)
		self._register_action("edit.atom.clear_number", self._clear_atom_number_action)
		self._atom_mark_actions = {}
		for kind_name, label in _ATOM_MARK_ACTIONS:
			action = PySide6.QtGui.QAction(self.tr(label), self)
			action.setToolTip(self.tr(
				"Add this mark, or remove the first matching mark from the selected atom",
			))
			action.triggered.connect(functools.partial(self._on_toggle_atom_mark, kind_name))
			self._atom_mark_actions[kind_name] = action
			self._register_action(f"edit.atom_mark.{kind_name}", action)
		self._remove_atom_mark_action = PySide6.QtGui.QAction(
			self.tr("Remove Atom Mark..."), self,
		)
		self._remove_atom_mark_action.setToolTip(
			self.tr("Choose one exact mark ordinal from the selected Rust atom"),
		)
		self._remove_atom_mark_action.triggered.connect(self._on_remove_atom_mark)
		self._register_action("edit.atom_mark.remove", self._remove_atom_mark_action)
		self._edit_bond_properties_action = PySide6.QtGui.QAction(
			self.tr("Edit Bond Properties"), self,
		)
		self._edit_bond_properties_action.setToolTip(
			self.tr("Edit one selected durable bond through one operation"),
		)
		self._edit_bond_properties_action.triggered.connect(self._on_edit_bond_properties)
		self._register_action("edit.bond.properties", self._edit_bond_properties_action)
		self._reverse_selected_wedge_direction_action = PySide6.QtGui.QAction(
			self.tr("Reverse Selected Wedge Direction"), self,
		)
		self._reverse_selected_wedge_direction_action.setToolTip(self.tr(
			"Swap the tip and base of one selected solid or hashed wedge through Rust",
		))
		self._reverse_selected_wedge_direction_action.triggered.connect(
			self._on_reverse_selected_wedge_direction,
		)
		self._register_action(
			"edit.bond.reverse_wedge", self._reverse_selected_wedge_direction_action,
		)
		self._edit_plus_properties_action = (
			ferrum_qt.ferrum.presentation_properties.install_plus_properties_action(
				self,
			)
		)
		self._edit_arrow_properties_action = (
			ferrum_qt.ferrum.arrow_properties.install_arrow_properties_action(
				self,
			)
		)
		self._edit_geometric_properties_action = (
			native_geometric_properties.install_geometric_properties_action(self)
		)
		self._edit_wavy_properties_action = (
			native_wavy_properties.install_wavy_properties_action(self)
		)
		self._delete_atom_action = PySide6.QtGui.QAction(
			self.tr("Delete Selected Atom"), self,
		)
		self._delete_atom_action.setToolTip(
			self.tr("Delete one atom and every incident bond through Rust"),
		)
		self._delete_atom_action.triggered.connect(self._on_delete_selected_atom)
		self._register_action("edit.delete_atom", self._delete_atom_action)
		self._delete_bond_action = PySide6.QtGui.QAction(
			self.tr("Delete Selected Bond"), self,
		)
		self._delete_bond_action.setToolTip(
			self.tr("Delete one durable typed bond through Rust"),
		)
		self._delete_bond_action.triggered.connect(self._on_delete_selected_bond)
		self._register_action("edit.delete_bond", self._delete_bond_action)
		self._change_bond_order_action = PySide6.QtGui.QAction(
			self.tr("Change Selected Bond Order"), self,
		)
		self._change_bond_order_action.setToolTip(
			self.tr("Choose single, double, or triple for one durable Rust bond"),
		)
		self._change_bond_order_action.triggered.connect(self._on_change_bond_order)
		self._register_action("edit.bond.change_order", self._change_bond_order_action)
		self._add_atom_action = PySide6.QtGui.QAction(self.tr("Add Atom at Point"), self)
		self._add_atom_action.setCheckable(True)
		self._add_atom_action.setToolTip(
			self.tr("Use Next atom, then click the canvas once; Esc cancels"),
		)
		self._register_action("draw.atom_at_point", self._add_atom_action)
		self._window_mode_sync.register_tool(
			ferrum_qt.ferrum.window_mode_sync.FerrumWindowToolBinding(
				self._add_atom_action, ferrum_qt.modes.base_mode.ModeId.ATOM,
				self._atom_mode.controller, "Add Atom", True,
				self._mode_context, self._atom_mode.activate,
				self._atom_mode.dispatch, self._atom_mode.cancel,
			),
		)
		self._add_single_bond_action = PySide6.QtGui.QAction(
			self.tr("Add Single Bond Between Selected Atoms"), self,
		)
		self._add_single_bond_action.setToolTip(
			self.tr("Select exactly two atoms, then connect them through Rust"),
		)
		self._add_single_bond_action.triggered.connect(self._on_add_single_bond)
		self._register_action("draw.bond.connect_selected", self._add_single_bond_action)
		self._build_line_tool_actions()
		self._build_structure_selection_action()
		self._build_top_level_transform_actions()
		self._undo_action = PySide6.QtGui.QAction(self.tr("Undo"), self)
		self._undo_action.triggered.connect(self._on_undo)
		self._register_action("edit.undo", self._undo_action)
		self._redo_action = PySide6.QtGui.QAction(self.tr("Redo"), self)
		self._redo_action.triggered.connect(self._on_redo)
		self._register_action("edit.redo", self._redo_action)
		self._build_native_clipboard_actions()
		self._refresh_action = PySide6.QtGui.QAction(self.tr("Refresh Authoritative View"), self)
		self._refresh_action.triggered.connect(self._on_refresh_authoritative)
		self._register_action("view.refresh", self._refresh_action)
		self._build_view_controls_actions()
		self._build_catalog_template_action()
		self._build_native_user_template_place_action()
		self._build_molecule_import_actions()
		self._build_multi_sdf_export_actions()
		self._build_sdf_export_actions()
		self._build_molfile_export_actions()
		self._build_molecule_export_actions()
		self._build_molecule_inspection_actions()
		self._build_molecule_diagnostics_action()
		self._build_bond_capacity_actions()
		self._build_atom_oxidation_action()
		self._build_explicit_hydrogen_action()
		self._build_compact_group_materialization_action()
		self._build_compact_group_authoring_action()
		self._build_free_compact_group_placement_action()
		self._build_molecule_name_action()
		self._build_linear_form_action()
		self._build_explicit_fragment_actions()
		self._build_direct_glycosidic_haworth_action()
		self._build_coordinate_generation_actions()
		self._wire_catalog_tool_replacement()

	#============================================
	def _register_action(self, action_id: str, action: PySide6.QtGui.QAction,
			*, lifecycle: str = "static") -> None:
		"""Bind one already-wired Ferrum command to its stable menu identity."""
		if not action.toolTip() and not action.statusTip() and not action.whatsThis():
			action.setToolTip(action.text().replace("&", "").strip())
		self._action_registry.register_existing(
			action_id, action, lifecycle=lifecycle,
			shortcut_exemption_reason=(
				"This labelled desktop command has no portable default shortcut."
			),
		)

	#============================================
	def _mode_context(self) -> object:
		"""Provide the live host context without giving modes document ownership."""
		import ferrum_qt.modes.base_mode
		tab = self._active_native_tab()
		return ferrum_qt.modes.base_mode.ModeContext(
			None, {"window": self, "tab_title": None if tab is None else tab.title},
		)

	#============================================
	def _connect_interaction_action_v1(self, action: PySide6.QtGui.QAction,
			handler: object) -> None:
		"""Register one action whose handler takes canvas interaction ownership."""
		self._interaction_action_handoff.connect(action, handler)

	def _register_pointer_capture_canceller_v1(self,
			canceller: collections.abc.Callable[[bool], None]) -> None:
		"""Register the selected-root capture in the window authoring transaction."""
		self._interaction_action_handoff.register_pointer_capture_canceller(canceller)

	#============================================
	def begin_smarts_selected_root_capture(self) -> object:
		"""Cancel other pointer tools and expose one current canvas capture target."""
		self.cancel_active_pointer_authoring()
		tab = self._active_native_tab()
		contract = ferrum_qt.ferrum.smarts_selected_root_contract
		if tab is None or tab.is_disposed or tab.requires_refresh:
			return contract.FerrumSmartsSelectedRootCaptureUnavailable(
				self.tr("Open a ready Ferrum drawing, then choose one molecule on the canvas."),
			)
		return contract.FerrumSmartsSelectedRootCaptureTarget(tab, tab.view.viewport())

	#============================================
	def capture_smarts_selected_root_query(self, target: object,
			point: PySide6.QtCore.QPoint) -> object:
		"""Capture one Rust-owned molecule token through an authenticated target."""
		contract = ferrum_qt.ferrum.smarts_selected_root_contract
		tab = target.tab
		if tab is not self._active_native_tab() or tab.is_disposed or tab.requires_refresh:
			return contract.FerrumSmartsSelectedRootCaptureUnavailable(
				self.tr("Molecule choice is no longer current. Choose one molecule again."),
			)
		selection = None
		try:
			observation = tab.observe_direct_root_interaction()
			scene = tab.view.mapToScene(point)
			selection = tab.select_direct_roots(
				observation, None,
				ferrum_qt.ferrum.engine.RenderInteractionQueryV1.point(
					float(scene.x()), float(scene.y()),
					ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace,
				),
			)
			token = tab.capture_live_smarts_selected_query(selection)
		except ferrum_qt.ferrum.engine.LiveDocumentSmartsError as error:
			return contract.FerrumSmartsSelectedRootCaptureRejected(error)
		finally:
			selection = None
		return contract.FerrumSmartsSelectedRootCaptureAccepted(tab, token)

	#============================================
	def run_smarts_selected_root_query(self, tab: object, token: object,
			per_molecule_limit: int, total_limit: int) -> object:
		"""Run an opaque selected-query token only through its live tab owner."""
		if tab is not self._active_native_tab() or tab.is_disposed or tab.requires_refresh:
			raise RuntimeError("Ferrum selected molecule query is no longer current")
		return tab.run_live_smarts_selected_query_token(
			token, per_molecule_limit, total_limit,
		)

	#============================================
	def cancel_active_pointer_authoring(self, *, clear_status: bool = True) -> None:
		"""Cancel every transient canvas authoring owner in one fixed order."""
		self._interaction_action_handoff.cancel_registered_pointer_capture(
			clear_status=clear_status,
		)
		self._cancel_atom_insertion(clear_status=clear_status)
		self._cancel_line_gesture(clear_status=clear_status)
		self._cancel_structure_selection()
		self._cancel_compact_group_authoring(clear_status=clear_status)
		self._cancel_free_compact_group_placement(clear_status=clear_status)
		self._cancel_catalog_placement(clear_status=clear_status)
		self._cancel_user_template_placement(clear_status=clear_status)

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

	#============================================
	def _on_reverse_selected_wedge_direction(self) -> None:
		"""Reverse one selected solid or hashed wedge through Rust."""
		tab = self._active_native_tab()
		if tab is None:
			return
		self.cancel_active_pointer_authoring()
		try:
			tab.reverse_selected_wedge_direction()
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._refresh_actions()
			self._show_edit_refusal(self._typed_refusal(
				"edit_document", "unavailable_operation", str(exc),
			))
			return
		self.statusBar().showMessage(self.tr("Reversed selected wedge direction."), 5000)
		self._refresh_actions()

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

	def _cancel_atom_insertion(self, clear_status: bool = True) -> None:
		"""Release atom mode only through its feature-owned cleanup endpoint."""
		if self._atom_insertion_intent is not None and self._window_mode_sync.cancel():
			return
		self._atom_mode.cancel(self._mode_context(), clear_status=clear_status)

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
		"""Cancel revision-bound pointer state before one Rust history transition."""
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
