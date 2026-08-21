"""Visible Rust-owned direct normal-bond pointer authoring."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.main_window


_EDITABLE_CDML = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-c' name='C'><point x='10' y='20'/></atom>
    <atom id='atom-o' name='O'><point x='70' y='20'/></atom>
  </molecule>
</cdml>"""


#============================================
def _viewport_point(tab: object, atom_id: str) -> PySide6.QtCore.QPoint:
	"""Map one installed durable atom position through the public Qt seam."""
	return tab.view.mapFromScene(tab.durable_atom_scene_position(atom_id))


#============================================
def _select_direct_bond_profile(window: object) -> object:
	"""Select P0.1's fixed-carbon single-bond profile explicitly for its tests."""
	prior = window._drawing_parameters.snapshot()
	window._drawing_parameters.set_element("C")
	window._drawing_parameters.set_order_name("single")
	window._drawing_parameters.set_presentation_name("normal")
	return prior


#============================================
def test_normal_pointer_drag_uses_rust_preview_then_commits_one_bond(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""The pointer route does not mutate until Rust accepts its opaque preview."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond.cdml",
	)
	try:
		refusals = []
		monkeypatch.setattr(
			window, "_show_edit_refusal",
			lambda request, _details=None: refusals.append(request),
		)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		prior_choices = _select_direct_bond_profile(window)
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		commits = []
		commit = tab.commit_direct_bond_gesture
		monkeypatch.setattr(
			tab, "commit_direct_bond_gesture",
			lambda gesture, preview: commits.append((gesture, preview)) or commit(gesture, preview),
		)
		monkeypatch.setattr(
			tab, "add_bond_between_atoms",
			lambda *_args: (_ for _ in ()).throw(AssertionError("legacy bond path used")),
		)
		monkeypatch.setattr(
			tab, "add_bonded_atom_at",
			lambda *_args: (_ for _ in ()).throw(AssertionError("legacy atom path used")),
		)
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		qapp.processEvents()
		assert len(tab.current_document_observation().projection.molecules[0].bonds) == 0
		assert not refusals
		assert window._line_gesture_intent.direct_bond_preview is not None
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, end,
		)
		qapp.processEvents()
		assert len(tab.current_document_observation().projection.molecules[0].bonds) == 1
		assert len(commits) == 1
		assert window._line_gesture_intent.direct_bond_gesture is None
		assert window._draw_bond_action.isChecked()
	finally:
		window._drawing_parameters.set_element(prior_choices.element)
		window._drawing_parameters.set_order_name(prior_choices.order_name)
		window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()


#============================================
def test_escape_discards_direct_bond_preview_without_a_mutation(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""Escape disposes the checked gesture and leaves the Rust revision unchanged."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-cancel.cdml",
	)
	try:
		monkeypatch.setattr(window, "_show_edit_refusal", lambda *_args: None)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		prior_choices = _select_direct_bond_profile(window)
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		before = tab.current_snapshot.revision
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		PySide6.QtTest.QTest.keyClick(tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape)
		qapp.processEvents()
		assert tab.current_snapshot.revision == before
		assert window._line_gesture_intent is None
		assert not window._draw_bond_action.isChecked()
	finally:
		window._drawing_parameters.set_element(prior_choices.element)
		window._drawing_parameters.set_order_name(prior_choices.order_name)
		window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()


#============================================
def test_normal_pointer_drag_to_empty_space_creates_rust_carbon_endpoint(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An empty endpoint remains raw Qt input until Rust creates its carbon atom."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-new-endpoint.cdml",
	)
	try:
		monkeypatch.setattr(window, "_show_edit_refusal", lambda *_args: None)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		prior_choices = _select_direct_bond_profile(window)
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
		assert molecule.atoms[-1].element == "C"
	finally:
		window._drawing_parameters.set_element(prior_choices.element)
		window._drawing_parameters.set_order_name(prior_choices.order_name)
		window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()


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
		prior_choices = _select_direct_bond_profile(window)
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
		window._drawing_parameters.set_element(prior_choices.element)
		window._drawing_parameters.set_order_name(prior_choices.order_name)
		window._drawing_parameters.set_presentation_name(prior_choices.presentation_name)
		window.close()
		window.deleteLater()
