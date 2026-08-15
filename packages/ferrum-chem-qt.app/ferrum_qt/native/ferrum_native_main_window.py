"""Public Rust-native Ferrum Qt window for the completed CDML slice."""
# Standard Library
import dataclasses
import functools
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.window_native_files
import ferrum_qt.bridge.insertion_placement
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_drawing_standard as native_drawing_standard
import ferrum_qt.native.ferrum_native_drawing_parameters
import ferrum_qt.native.ferrum_native_atom_properties
import ferrum_qt.native.ferrum_native_atom_element
import ferrum_qt.native.ferrum_native_atom_number
import ferrum_qt.native.ferrum_native_arrow_properties
import ferrum_qt.native.ferrum_native_bond_properties
import ferrum_qt.native.ferrum_native_clipboard
import ferrum_qt.native.ferrum_native_cdml_open
import ferrum_qt.native.ferrum_native_coordinate_generation
import ferrum_qt.native.ferrum_native_geometric_properties as native_geometric_properties
import ferrum_qt.native.ferrum_native_geometry_actions
import ferrum_qt.native.ferrum_native_line_tools
import ferrum_qt.native.ferrum_native_haworth_tool
import ferrum_qt.native.ferrum_native_direct_glycosidic_haworth_tool
import ferrum_qt.native.ferrum_native_linear_form
import ferrum_qt.native.ferrum_native_main_window_support
import ferrum_qt.native.ferrum_native_explicit_fragments
import ferrum_qt.native.ferrum_native_molecule_imports
import ferrum_qt.native.ferrum_native_molecule_exports
import ferrum_qt.native.ferrum_native_molfile_export
import ferrum_qt.native.ferrum_native_sdf_export
import ferrum_qt.native.ferrum_native_molecule_inspection
import ferrum_qt.native.ferrum_native_molecule_name
import ferrum_qt.native.ferrum_native_snapshot_export
import ferrum_qt.native.ferrum_native_recovery_export
import ferrum_qt.native.ferrum_native_selection_svg
import ferrum_qt.native.ferrum_native_view_controls
import ferrum_qt.native.ferrum_native_user_templates as native_user_templates
import ferrum_qt.native.ferrum_native_presentation_properties
import ferrum_qt.native.ferrum_native_property_dock
import ferrum_qt.native.ferrum_native_paper_properties as native_paper_properties
import ferrum_qt.native.ferrum_native_wavy_properties as native_wavy_properties
import ferrum_qt.dialogs.atom_dialog
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
	"""One revision-bound native click request awaiting an exact scene point."""

	tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	molecule_object_id: str
	element: str

#============================================
class FerrumNativeMainWindow(
		ferrum_qt.native.ferrum_native_explicit_fragments.
		FerrumNativeExplicitFragmentsWindowMixin,
		ferrum_qt.native.ferrum_native_haworth_tool.FerrumNativeHaworthToolMixin,
		ferrum_qt.native.ferrum_native_direct_glycosidic_haworth_tool.
		FerrumNativeDirectGlycosidicHaworthWindowMixin,
		native_user_templates.FerrumNativeUserTemplateWindowMixin,
		ferrum_qt.native.ferrum_native_view_controls.FerrumNativeViewControlsMixin,
		ferrum_qt.native.ferrum_native_selection_svg.FerrumNativeSelectionSvgWindowMixin,
		ferrum_qt.native.ferrum_native_clipboard.FerrumNativeClipboardWindowMixin,
		ferrum_qt.native.ferrum_native_cdml_open.FerrumNativeCdmlOpenMixin,
		ferrum_qt.window_native_files.WindowNativeFileMixin,
		ferrum_qt.native.ferrum_native_recovery_export.FerrumNativeRecoveryExportWindowMixin,
		ferrum_qt.native.ferrum_native_snapshot_export.FerrumNativeSnapshotExportWindowMixin,
		ferrum_qt.native.ferrum_native_line_tools.FerrumNativeLineToolsMixin,
		ferrum_qt.native.ferrum_native_molecule_imports.FerrumNativeMoleculeImportsMixin,
		ferrum_qt.native.ferrum_native_sdf_export.FerrumNativeSdfExportMixin,
		ferrum_qt.native.ferrum_native_molfile_export.FerrumNativeMolfileExportMixin,
		ferrum_qt.native.ferrum_native_molecule_exports.FerrumNativeMoleculeExportsMixin,
		ferrum_qt.native.ferrum_native_molecule_inspection.FerrumNativeMoleculeInspectionMixin,
		ferrum_qt.native.ferrum_native_molecule_name.FerrumNativeMoleculeNameWindowMixin,
		ferrum_qt.native.ferrum_native_linear_form.FerrumNativeLinearFormWindowMixin,
		ferrum_qt.native.ferrum_native_coordinate_generation.
		FerrumNativeCoordinateGenerationWindowMixin,
		ferrum_qt.native.ferrum_native_geometry_actions.FerrumNativeGeometryActionsMixin,
		ferrum_qt.native.ferrum_native_main_window_support.NativeOnlyFileFallback,
		PySide6.QtWidgets.QMainWindow,
		):
	"""A standalone public host for Rust-owned CDML tabs only.

	This window intentionally has no alternate session registry or backend
	fallback. It owns the ordinary product's vertical Rust
	open/render/save/reopen path.
	"""

	local_cdml_open_completed = PySide6.QtCore.Signal(str, bool)
	local_cdml_open_queue_drained = PySide6.QtCore.Signal(bool)

	#============================================
	def __init__(
			self, parent: PySide6.QtWidgets.QWidget | None = None, *,
			user_template_directory: str | pathlib.Path | None = None,
			) -> None:
		"""Build the small native document host and its reachable file actions."""
		super().__init__(parent)
		getattr(self, "_initialize_native_file_menu_clients", lambda: None)()
		self._native_tabs_by_page = {}
		self._drawing_parameters = (
			ferrum_qt.native.ferrum_native_drawing_parameters.
			FerrumNativeDrawingParameters()
		)
		self._initialize_native_user_templates(user_template_directory)
		self._initialize_local_cdml_open()
		self._initialize_view_controls()
		self._atom_insertion_intent: _AtomInsertionIntent | None = None
		self._initialize_line_tools()
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
		self._tab_widget.currentChanged.connect(self._on_native_tab_changed)
		self._tab_widget.tabCloseRequested.connect(self._close_tab_at)
		self.setCentralWidget(self._tab_widget)
		self.setWindowTitle(self.tr("Ferrum-Qt Native"))
		self.resize(1000, 700)
		self._build_actions()
		self._native_property_dock = (
			ferrum_qt.native.ferrum_native_property_dock.install_native_property_dock(
				self, self._edit_atom_properties_action, self._edit_bond_properties_action,
			)
		)
		self.statusBar()
		self._install_native_view_status_controls()
		self._refresh_actions()

	#============================================
	def _build_actions(self) -> None:
		"""Create native file and bounded Rust edit actions."""
		menu = self.menuBar().addMenu(self.tr("File"))
		self._file_menu = menu
		self._open_action = PySide6.QtGui.QAction(self.tr("Open"), self)
		self._open_action.triggered.connect(self._on_open)
		menu.addAction(self._open_action)
		self._build_open_in_current_tab_action(menu)
		self._build_local_cdml_open_action(menu)
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
		quit_action = PySide6.QtGui.QAction(self.tr("Quit"), self)
		quit_action.triggered.connect(self.close)
		menu.addAction(quit_action)
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
			self.tr("Edit one selected durable atom through one Rust-native operation"),
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
			self.tr("Edit one selected durable bond through one Rust-native operation"),
		)
		self._edit_bond_properties_action.triggered.connect(self._on_edit_bond_properties)
		edit_menu.addAction(self._edit_bond_properties_action)
		self._edit_plus_properties_action = (
			ferrum_qt.native.ferrum_native_presentation_properties.install_plus_properties_action(
				self, edit_menu,
			)
		)
		self._edit_arrow_properties_action = (
			ferrum_qt.native.ferrum_native_arrow_properties.install_arrow_properties_action(
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
		self._add_atom_action.triggered.connect(self._on_toggle_add_atom)
		edit_menu.addAction(self._add_atom_action)
		self._add_single_bond_action = PySide6.QtGui.QAction(
			self.tr("Add Single Bond Between Selected Atoms"), self,
		)
		self._add_single_bond_action.setToolTip(
			self.tr("Select exactly two atoms, then connect them through Rust"),
		)
		self._add_single_bond_action.triggered.connect(self._on_add_single_bond)
		edit_menu.addAction(self._add_single_bond_action)
		self._build_line_tool_actions(edit_menu)
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

	#============================================
	def _on_change_element(self) -> None:
		"""Collect one element spelling and submit it only to the active native tab."""
		ferrum_qt.native.ferrum_native_atom_element.run_change_selected_atom_element_dialog(self)
		self._refresh_actions()

	#============================================
	def _on_edit_atom_properties(self) -> None:
		"""Use the shared visual form while keeping the Rust session authoritative."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			atom = tab.selected_atom_projection()
			model = ferrum_qt.native.ferrum_native_atom_properties.dialog_model_from_projection(
				atom,
			)
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Atom Properties Unavailable", str(exc))
			return
		dialog = ferrum_qt.dialogs.atom_dialog.AtomDialog(model, self)
		if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			changes = (
				ferrum_qt.native.ferrum_native_atom_properties.
				property_changes_from_dialog(dialog.changes())
			)
			tab.apply_selected_atom_properties(changes)
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Atom Properties Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Updated one Rust-native atom."), 5000)
		self._refresh_actions()

	#============================================
	def _on_set_atom_number(self) -> None:
		"""Collect and assign one persistent number through the active Rust tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			atom = tab.selected_atom_projection()
			dialog = (
				ferrum_qt.native.ferrum_native_atom_number.
				FerrumNativeAtomNumberDialog(atom.number, atom.show_number, self)
			)
		except Exception as exc:
			self._show_native_file_warning("Native Atom Number Unavailable", str(exc))
			return
		if dialog.exec() != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		try:
			number, show_number = dialog.assignment()
			tab.set_selected_atom_number(number, show_number)
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Atom Number Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Updated one Rust-native atom number."), 5000)
		self._refresh_actions()

	#============================================
	def _on_clear_atom_number(self) -> None:
		"""Clear one selected persistent number through the active Rust tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			tab.clear_selected_atom_number()
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Atom Number Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Cleared one Rust-native atom number."), 5000)
		self._refresh_actions()

	#============================================
	def _on_toggle_atom_mark(self, kind_name: str, _checked: bool = False) -> None:
		"""Toggle one closed mark kind through the active authoritative Rust tab."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			import ferrum_chem
			kind = getattr(ferrum_chem.AtomMarkKindV1, kind_name)
			tab.toggle_selected_atom_mark(kind)
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Atom Mark Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Toggled one Rust-native atom mark."), 5000)
		self._refresh_actions()

	#============================================
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
			import ferrum_chem
			tab.apply_selected_atom_mark(
				ferrum_chem.AtomMarkActionV1.remove,
				mark.kind,
				mark.same_type_ordinal,
			)
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Atom Mark Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Removed one Rust-native atom mark."), 5000)
		self._refresh_actions()

	#============================================
	def _atom_mark_label(self, kind: object) -> str:
		"""Return one UI label for an exact closed extension mark value."""
		import ferrum_chem
		for kind_name, label in _ATOM_MARK_ACTIONS:
			if kind == getattr(ferrum_chem.AtomMarkKindV1, kind_name):
				return self.tr(label)
		raise TypeError("native atom mark projection contains an unknown kind")

	#============================================
	def _on_edit_bond_properties(self) -> None:
		"""Delegate the bounded visual form to its focused native adapter."""
		ferrum_qt.native.ferrum_native_bond_properties.run_bond_properties_dialog(self)

	#============================================
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
			self._show_native_file_warning("Native Delete Atom Error", str(exc))
			return
		self.statusBar().showMessage(
			self.tr("Deleted one Rust-native atom and its incident bonds."), 5000,
		)
		self._refresh_actions()

	#============================================
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
			self._show_native_file_warning("Native Delete Bond Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Deleted one Rust-native bond."), 5000)
		self._refresh_actions()

	#============================================
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
		import ferrum_chem
		orders = (
			ferrum_chem.DocumentBondOrderV1.single,
			ferrum_chem.DocumentBondOrderV1.double,
			ferrum_chem.DocumentBondOrderV1.triple,
		)
		try:
			tab.set_selected_bond_order(orders[labels.index(selected)])
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native Bond Order Error", str(exc))
			return
		self.statusBar().showMessage(
			self.tr("Changed one Rust-native bond to {0}.").format(selected.lower()), 5000,
		)
		self._refresh_actions()

	#============================================
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
		choices = tab.durable_molecule_choices()
		if not choices:
			self._cancel_atom_insertion()
			self._show_native_file_warning(
				"Native Add Atom Unavailable",
				"This document has no durable molecule that Rust can edit.",
			)
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
		self._add_atom_action.setToolTip(self.tr(
			"Add {0} at the next canvas click; Escape cancels."
		).format(drawing.element))
		self._refresh_cancel_tool_action()
		viewport.installEventFilter(self)
		viewport.setFocus()
		self.statusBar().showMessage(
			self.tr(
				"Click once to add {0}; Esc cancels Add Atom.".format(drawing.element),
			),
		)

	#============================================
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
			self._show_native_file_warning(
				"Native Add Atom Stale",
				"The document changed before the click; start Add Atom again.",
			)
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
			self._show_native_file_warning("Native Add Atom Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Added one free-standing Rust atom."), 5000)
		self._refresh_actions()

	#============================================
	def _cancel_atom_insertion(self, clear_status: bool = True) -> None:
		"""Release the one native viewport event boundary without changing Rust state."""
		intent = self._atom_insertion_intent
		self._atom_insertion_intent = None
		self._add_atom_action.setChecked(False)
		if intent is not None:
			intent.viewport.removeEventFilter(self)
		if clear_status:
			self.statusBar().clearMessage()
		self._refresh_cancel_tool_action()

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
			self._show_native_file_warning("Native Add Bond Error", str(exc))
			return
		self.statusBar().showMessage(self.tr("Added one Rust-native single bond."), 5000)
		self._refresh_actions()

	#============================================
	def _on_redo(self) -> None:
		"""Attempt the Rust redo operation and report typed unavailable history truth."""
		self._run_native_history_action("redo")

	#============================================
	def _run_native_history_action(self, name: str) -> None:
		"""Run one bounded history command without deriving availability in Python."""
		tab = self._active_native_tab()
		if tab is None:
			return
		try:
			getattr(tab, name)()
		except Exception as exc:
			self._refresh_actions()
			self._show_native_file_warning("Native History Unavailable", str(exc))
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
				self._show_native_file_warning(
					"Native View Refresh Failed",
					"Rust remains authoritative; the previous view is still retained.",
				)
		except Exception as exc:
			self._show_native_file_warning("Native View Refresh Error", str(exc))
		self._refresh_actions()

	#============================================
	def _register_native_tab(
			self,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			*, activate: bool,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Attach one exact Rust-native tab to this standalone public host."""
		if type(tab) is not ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
			raise TypeError("native window requires an exact FerrumNativeDocumentTab")
		if tab in self._native_tabs_by_page:
			raise ValueError("native tab is already registered")
		index = self._tab_widget.addTab(tab, tab.title)
		self._native_tabs_by_page[tab] = tab
		self._install_native_hex_grid_for_tab(tab)
		tab.selection_changed.connect(self._on_native_selection_changed)
		tab.view.display_transform_changed.connect(self._refresh_native_view_status)
		if activate:
			self._tab_widget.setCurrentIndex(index)
		if self._active_native_tab() is tab:
			self._on_native_view_tab_changed()
		self._refresh_actions()
		return tab

	#============================================
	def save_active_to_path(self, path: str) -> bool:
		"""Publish the selected native tab to a caller-supplied CDML destination."""
		tab = self._active_native_tab()
		if tab is None:
			return False
		return self._save_native_tab_to_path(tab, path)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _close_tab_at(self, index: int) -> None:
		"""Dispose one clean native tab, retaining dirty tabs for an explicit save."""
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
		if self._clipboard_operation_blocks_tab_close(tab):
			return
		if self._coordinate_generation_blocks_tab_close(tab):
			return
		if self._user_template_placement_blocks_tab_close(tab):
			return
		if tab.requires_refresh:
			self._show_native_file_warning(
				"Authoritative Refresh Required",
				"Refresh the authoritative Rust view before closing this tab.",
			)
			return
		if tab.is_dirty:
			self._show_native_file_warning(
				"Unsaved Rust Changes",
				"Save or discard the native document before closing this tab.",
			)
			return
		if self._atom_insertion_intent is not None and self._atom_insertion_intent.tab is tab:
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None and self._line_gesture_intent.tab is tab:
			self._cancel_line_gesture()
		if (
			self._direct_glycosidic_haworth_intent is not None
			and self._direct_glycosidic_haworth_intent.tab is tab
		):
			self._cancel_direct_glycosidic_haworth_intent()
		self._cancel_native_view_controls_for_tab(tab)
		self._tab_widget.removeTab(index)
		self._native_tabs_by_page.pop(tab)
		tab.hide()
		tab.setParent(None)
		tab.dispose()
		self._refresh_actions()

	#============================================
	def _close_current_tab(self) -> None:
		"""Close the selected page through the same clean-tab guard."""
		index = self._tab_widget.currentIndex()
		if index >= 0:
			self._close_tab_at(index)

	#============================================
	@PySide6.QtCore.Slot(int)
	def _on_native_tab_changed(self, _index: int) -> None:
		"""Refresh actions after Qt reports a selected native page index."""
		if hasattr(self, "_native_tabs_by_page"):
			self._on_native_view_tab_changed()
			self._refresh_actions()

	#============================================
	@PySide6.QtCore.Slot()
	def _on_native_selection_changed(self) -> None:
		"""Refresh actions after a native scene selection changes."""
		if hasattr(self, "_native_tabs_by_page"):
			self._refresh_actions()

	#============================================
	def _refresh_actions(self, *_unused: object) -> None:
		"""Make native Save and Close reachability follow the selected page."""
		self._refresh_local_cdml_open_action()
		tab = self._active_native_tab()
		active = tab is not None and not tab._disposed
		pending = active and tab.requires_refresh
		template_intent = self._user_template_placement_intent
		if (
			template_intent is not None
			and not self._user_template_placement_is_current(template_intent)
		):
			self._cancel_user_template_placement()
		busy_import = self._molecule_import_busy()
		busy_export = self._molecule_export_busy()
		busy_inspection = self._molecule_inspection_busy()
		busy_clipboard = self._clipboard_busy()
		busy_coordinates = self._coordinate_generation_intent is not None
		busy_user_template = self._user_template_placement_intent is not None
		busy_snapshot_export = self._snapshot_export_busy()
		busy = (
			busy_import or busy_export or busy_inspection or busy_clipboard or busy_coordinates
			or busy_user_template or busy_snapshot_export
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
			and ferrum_qt.native.ferrum_native_atom_element.
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
		ferrum_qt.native.ferrum_native_presentation_properties.refresh_plus_properties_action(
			self._edit_plus_properties_action, tab, active, pending, busy,
		)
		ferrum_qt.native.ferrum_native_arrow_properties.refresh_arrow_properties_action(
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
		can_add_atom = active and not pending and not busy and bool(tab.durable_molecule_choices())
		self._add_atom_action.setEnabled(can_add_atom)
		self._add_atom_action.setToolTip(self.tr(
			"Use Next atom, then click the canvas once; Esc cancels"
			if can_add_atom else
			"Requires an active document with a durable Rust molecule",
		))
		self._add_single_bond_action.setEnabled(active and not pending and not busy)
		self._refresh_line_tool_actions(active and not pending and not busy)
		self._refresh_top_level_transform_actions(tab, active, pending, busy)
		self._undo_action.setEnabled(active and not pending and not busy)
		self._redo_action.setEnabled(active and not pending and not busy)
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
		self._refresh_molecule_export_actions(
			active, pending, busy_import or busy_coordinates or busy_clipboard,
		)
		self._refresh_molecule_inspection_actions(
			active,
			pending,
			busy_import or busy_export or busy_coordinates or busy_clipboard,
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
		self._generate_coordinates_action.setEnabled(
			active and not pending and not busy and bool(tab.durable_molecule_choices()),
		)
		self._cancel_coordinates_action.setEnabled(
			busy_coordinates
			and not self._coordinate_generation_intent.worker.delivery_cancelled,
		)
		self._refresh_view_controls_actions()
		self._native_property_dock.refresh(tab)

	#============================================
	def _show_native_file_warning(self, title: str, message: str) -> None:
		"""Present one actionable native-file failure."""
		PySide6.QtWidgets.QMessageBox.warning(self, self.tr(title), self.tr(message))

	#============================================
	def closeEvent(self, event: PySide6.QtGui.QCloseEvent) -> None:
		"""Dispose all clean pages and keep an unsaved Rust document live."""
		self._cancel_user_template_placement()
		self._cancel_direct_glycosidic_haworth_intent()
		if self._cancel_local_cdml_open_for_close():
			event.ignore()
			self._show_native_file_warning(
				"Native CDML Open Still Running",
				"Ferrum cancelled delivery; close again after Rust admission finishes.",
			)
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
			self._show_native_file_warning(
				"Native Coordinates Still Running",
				"Ferrum cancelled delivery; close again after native work finishes.",
			)
			return
		if any(tab.requires_refresh for tab in self._native_tabs_by_page.values()):
			event.ignore()
			self._show_native_file_warning(
				"Authoritative Refresh Required",
				"Refresh every pending authoritative Rust view before closing Ferrum-Qt.",
			)
			return
		if any(tab.is_dirty for tab in self._native_tabs_by_page.values()):
			event.ignore()
			self._show_native_file_warning(
				"Unsaved Rust Changes",
				"Save or discard every native document before closing Ferrum-Qt.",
			)
			return
		self._cancel_atom_insertion()
		self._cancel_line_gesture()
		self._prepare_native_view_controls_shutdown()
		for tab in tuple(self._native_tabs_by_page.values()):
			index = self._tab_widget.indexOf(tab)
			if index >= 0:
				self._tab_widget.removeTab(index)
			tab.hide()
			tab.setParent(None)
			tab.dispose()
		self._native_tabs_by_page.clear()
		event.accept()
