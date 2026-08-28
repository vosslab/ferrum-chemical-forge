"""Behavior coverage for application-owned Ferrum drawing choices."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.drawing_parameters
import ferrum_qt.ferrum.drawing_parameters_client
import ferrum_qt.ferrum.document_tab
import ferrum_qt.themes.theme_loader
import ferrum_qt.widgets.periodic_table


#============================================
class _ValueStore:
	"""Minimal value-store seam representing the product preference boundary."""

	#============================================
	def __init__(self) -> None:
		"""Start with no persisted application choices."""
		self.values = {}

	#============================================
	def value(self, key: str, default: object) -> object:
		"""Return one stored value or the caller's ordinary product default."""
		return self.values.get(key, default)

	#============================================
	def set_value(self, key: str, value: object) -> None:
		"""Persist one accepted preference value through the store seam."""
		self.values[key] = value


#============================================
def test_valid_next_drawing_choices_round_trip_through_application_store() -> None:
	"""A completed choice returns with conventional element spelling after recreation."""
	store = _ValueStore()
	parameters = ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParameters(
		store,
	)
	parameters.set_element("cL")
	parameters.set_order_name("triple")
	recreated = ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParameters(
		store,
	)
	assert recreated.snapshot() == (
		ferrum_qt.ferrum.drawing_parameters.
		FerrumNativeDrawingParametersSnapshot("Cl", "triple")
	)


#============================================
def test_invalid_next_drawing_choices_keep_last_effective_choice() -> None:
	"""An unfinished invalid edit leaves the next authoring operation unchanged."""
	parameters = ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParameters(
		_ValueStore(),
	)
	parameters.set_element("N")
	parameters.set_order_name("double")
	parameters.set_element("N2")
	parameters.set_order_name("aromatic")
	assert parameters.snapshot() == (
		ferrum_qt.ferrum.drawing_parameters.
		FerrumNativeDrawingParametersSnapshot("N", "double")
	)


#============================================
def test_periodic_picker_updates_the_shared_next_drawing_model(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: object,
		) -> None:
	"""Picker acceptance refreshes peer clients through the shared preference model."""
	parameters = ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParameters(
		_ValueStore(),
	)
	first_client = (
		ferrum_qt.ferrum.drawing_parameters_client.
		FerrumNativeDrawingParametersClient(parameters)
	)
	peer_client = (
		ferrum_qt.ferrum.drawing_parameters_client.
		FerrumNativeDrawingParametersClient(parameters)
	)
	monkeypatch.setattr(
		ferrum_qt.widgets.periodic_table.PeriodicTablePopup, "pick_element",
		lambda _entries, _parent: "O",
	)
	first_client.periodic_table_button.click()
	qapp.processEvents()
	assert parameters.snapshot().element == "O"
	assert peer_client.element_combo.currentText() == "O"


#============================================
def test_periodic_picker_cancel_retains_the_effective_preference(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: object,
		) -> None:
	"""Cancellation is a no-op on the document-free next-drawing preference."""
	parameters = ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParameters(
		_ValueStore(),
	)
	client = (
		ferrum_qt.ferrum.drawing_parameters_client.
		FerrumNativeDrawingParametersClient(parameters)
	)
	calls: list[object] = []
	original_set_element = parameters.set_element
	monkeypatch.setattr(
		parameters, "set_element",
		lambda value: calls.append(value) or original_set_element(value),
	)
	monkeypatch.setattr(
		ferrum_qt.widgets.periodic_table.PeriodicTablePopup, "pick_element",
		lambda _entries, _parent: "",
	)
	client.periodic_table_button.click()
	qapp.processEvents()
	assert calls == []
	assert parameters.snapshot().element == "C"


#============================================
def test_next_drawing_return_commits_text_without_opening_periodic_picker(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: object,
		) -> None:
	"""Return from the atom editor cannot activate the nonterminal chooser."""
	parameters = ferrum_qt.ferrum.drawing_parameters.FerrumNativeDrawingParameters(
		_ValueStore(),
	)
	cancel_action = PySide6.QtGui.QAction()
	parent = PySide6.QtWidgets.QWidget()
	dialog = ferrum_qt.ferrum.drawing_parameters_client.FerrumNativeDrawingParametersDialog(
		parameters, cancel_action, parent,
	)
	picker_calls: list[bool] = []
	rejected: list[bool] = []
	dialog.rejected.connect(lambda: rejected.append(True))
	monkeypatch.setattr(
		ferrum_qt.widgets.periodic_table.PeriodicTablePopup, "pick_element",
		lambda _entries, _parent: picker_calls.append(True) or "O",
	)
	dialog.show()
	qapp.processEvents()
	editor = dialog.client.element_combo.lineEdit()
	assert editor is not None
	editor.setFocus()
	editor.selectAll()
	PySide6.QtTest.QTest.keyClicks(editor, "N")
	PySide6.QtTest.QTest.keyClick(editor, PySide6.QtCore.Qt.Key.Key_Return)
	qapp.processEvents()
	assert parameters.snapshot().element == "N"
	assert picker_calls == []
	assert dialog.isVisible()
	assert rejected == []
	dialog.client.periodic_table_button.click()
	qapp.processEvents()
	assert picker_calls == [True]
	assert parameters.snapshot().element == "O"
	dialog.close()


#============================================
def test_periodic_picker_choice_preserves_live_document_history_and_selection(
		qapp: PySide6.QtWidgets.QApplication, main_window: object,
		monkeypatch: object,
		) -> None:
	"""Choosing a Rust-issued element changes only the shared next-drawing preference."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='mol-1'><atom id='a1' "
		"name='C'><point x='10' y='10'/></atom></molecule></cdml>",
		"periodic-preference-invariance.cdml",
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	main_window._register_native_tab(tab, activate=True)
	main_window.show()
	qapp.processEvents()
	atom_action = main_window._action_registry.get_qt_action("draw.atom_at_point")
	assert main_window._window_mode_sync.select_action(atom_action)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tab.view.mapFromScene(PySide6.QtCore.QPointF(30.0, 40.0)),
	)
	qapp.processEvents()
	assert main_window._window_mode_sync.select_action(main_window._select_structure_action)
	PySide6.QtTest.QTest.mouseClick(
		tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 10.0)),
	)
	qapp.processEvents()
	selection = main_window._structure_selection
	assert selection is not None
	before_document = (
		tab.current_snapshot.revision, tab.current_snapshot.digest,
		tab.can_undo(), tab.can_redo(),
		tuple(target.object_id for target in selection.targets),
	)
	monkeypatch.setattr(
		ferrum_qt.widgets.periodic_table.PeriodicTablePopup, "pick_element",
		lambda entries, _parent: next((
			entry.symbol for entry in entries if entry.symbol == "O"
		)),
	)
	client = main_window._authoring_ribbon._drawing_parameters_client
	client.periodic_table_button.click()
	qapp.processEvents()
	after_selection = main_window._structure_selection
	assert after_selection is not None
	after_document = (
		tab.current_snapshot.revision, tab.current_snapshot.digest,
		tab.can_undo(), tab.can_redo(),
		tuple(target.object_id for target in after_selection.targets),
	)
	assert after_document == before_document
	assert main_window._drawing_parameters.snapshot().element == "O"
