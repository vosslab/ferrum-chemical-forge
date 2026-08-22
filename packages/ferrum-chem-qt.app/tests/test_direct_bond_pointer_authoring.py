"""Visible Rust-owned direct normal-bond pointer authoring."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_EDITABLE_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
    <atom id='atom-o' name='O'><point x='70' y='20'/></atom>
  </molecule>
</cdml>"""
_EMPTY_CDML = "<cdml xmlns='urn:ferrum:cdml'/>"
_ORDER_CDML = """<cdml xmlns='urn:ferrum:cdml'>
  <molecule id='mol-1'>
    <atom id='first-start' name='C'><point x='10' y='20'/></atom>
    <atom id='first-end' name='C'><point x='70' y='20'/></atom>
    <atom id='second-start' name='C'><point x='10' y='80'/></atom>
    <atom id='second-end' name='C'><point x='70' y='80'/></atom>
  </molecule>
</cdml>"""


#============================================
def _viewport_point(tab: object, atom_id: str) -> PySide6.QtCore.QPoint:
	"""Map one installed durable atom position through the public Qt seam."""
	return tab.view.mapFromScene(tab.durable_atom_scene_position(atom_id))


#============================================
def _close_window(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Deliver ordinary deferred Qt deletion after a window interaction."""
	window.close()
	window.deleteLater()
	PySide6.QtCore.QCoreApplication.sendPostedEvents(
		None, PySide6.QtCore.QEvent.Type.DeferredDelete,
	)
	qapp.processEvents()


#============================================
def test_normal_pointer_drag_uses_rust_admission_then_commits_one_bond(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The pointer route does not mutate until Rust issues an opaque admission."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond.cdml",
	)
	prior = window._drawing_parameters.snapshot()
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		assert window._drawing_parameters.set_order_name("single")
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		qapp.processEvents()
		assert len(tab.current_document_observation().projection.molecules[0].bonds) == 0
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		qapp.processEvents()
		assert len(tab.current_document_observation().projection.molecules[0].bonds) == 1
	finally:
		window._drawing_parameters.set_element(prior.element)
		window._drawing_parameters.set_order_name(prior.order_name)
		_close_window(qapp, window)


#============================================
def test_blank_canvas_direct_bond_redeems_one_new_new_admission(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A blank NewNew preview becomes one Rust mutation when released."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EMPTY_CDML, "blank-direct-bond.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = tab.view.mapFromScene(PySide6.QtCore.QPointF(100.0, 100.0))
		end = tab.view.mapFromScene(PySide6.QtCore.QPointF(180.0, 100.0))
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		qapp.processEvents()
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		qapp.processEvents()
		molecule = tab.current_document_observation().projection.molecules[0]
		assert (len(molecule.atoms), len(molecule.bonds)) == (2, 1)
	finally:
		_close_window(qapp, window)


#============================================
def test_pointer_direct_bond_freezes_order_after_valid_start_then_refreshes(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The next valid pointer gesture adopts a later Next Drawing order."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_ORDER_CDML, "direct-bond-order-snapshot.cdml",
	)
	prior = window._drawing_parameters.snapshot()
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		assert window._drawing_parameters.set_order_name("double")
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			_viewport_point(tab, "first-start"),
		)
		assert window._drawing_parameters.set_order_name("triple")
		first_end = _viewport_point(tab, "first-end")
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), first_end)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, first_end,
		)
		qapp.processEvents()
		assert tab.current_document_observation().projection.molecules[0].bonds[0].source_type == "n2"

		assert window._drawing_parameters.set_order_name("single")
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			_viewport_point(tab, "second-start"),
		)
		second_end = _viewport_point(tab, "second-end")
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), second_end)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, second_end,
		)
		qapp.processEvents()
		assert [bond.source_type for bond in tab.current_document_observation().projection.molecules[0].bonds] == [
			"n2", "n1",
		]
	finally:
		window._drawing_parameters.set_element(prior.element)
		window._drawing_parameters.set_order_name(prior.order_name)
		_close_window(qapp, window)


#============================================
def test_escape_discards_direct_bond_admission_without_a_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape disposes the checked gesture and leaves the Rust revision unchanged."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-cancel.cdml",
	)
	prior = window._drawing_parameters.snapshot()
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		assert window._drawing_parameters.set_order_name("single")
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert not window._draw_bond_action.isChecked()
	finally:
		window._drawing_parameters.set_element(prior.element)
		window._drawing_parameters.set_order_name(prior.order_name)
		_close_window(qapp, window)


#============================================
def test_normal_pointer_drag_to_empty_space_creates_rust_carbon_endpoint(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""An empty endpoint remains raw Qt input until Rust creates its carbon atom."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-new-endpoint.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-c")
		empty = tab.view.mapFromScene(PySide6.QtCore.QPointF(130.0, 20.0))
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), empty)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, empty,
		)
		qapp.processEvents()
		molecule = tab.current_document_observation().projection.molecules[0]
		assert len(molecule.atoms) == 3
		assert len(molecule.bonds) == 1
	finally:
		_close_window(qapp, window)


#============================================
def test_implicit_direct_bond_endpoint_uses_its_existing_source_identifier(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A unique implicit carbon is an existing endpoint, never a new atom."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-implicit-existing-endpoint.cdml",
	)
	prior = window._drawing_parameters.snapshot()
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		assert window._drawing_parameters.set_order_name("single")
		start = _viewport_point(tab, "atom-o")
		implicit = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		end = implicit + PySide6.QtCore.QPoint(6, 0)
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		qapp.processEvents()
		molecule = tab.current_document_observation().projection.molecules[0]
		assert len(molecule.atoms) == 2
		assert len(molecule.bonds) == 1
	finally:
		window._drawing_parameters.set_element(prior.element)
		window._drawing_parameters.set_order_name(prior.order_name)
		_close_window(qapp, window)


#============================================
def test_typed_direct_bond_refusal_is_visible_and_terminal(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A Rust self-loop refusal cancels the gesture before release can mutate."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-refusal.cdml",
	)
	try:
		refusals = []
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request, _details=None: refusals.append(request),
		)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-c")
		before = tab.current_snapshot.revision
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), start)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		qapp.processEvents()
		assert refusals
		assert window._line_gesture_intent is None
		assert tab.current_snapshot.revision == before
	finally:
		_close_window(qapp, window)
