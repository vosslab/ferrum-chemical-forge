"""Visible Rust-owned direct normal-bond pointer authoring."""

# Standard Library
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine
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
def test_normal_pointer_drag_uses_rust_admission_then_commits_one_bond(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""The pointer route does not mutate until Rust issues an opaque admission."""
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
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		commits = []
		commit = tab.commit_direct_bond_admission
		monkeypatch.setattr(
			tab, "commit_direct_bond_admission",
			lambda admission: commits.append(admission) or commit(admission),
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
		assert window._line_gesture_intent.direct_bond_admission is not None
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
		window.close()
		window.deleteLater()


#============================================
def test_draw_bond_freezes_next_drawing_order_across_sticky_rearms(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Only cancel/reactivation may replace the normal order given to Rust."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-order-snapshot.cdml",
	)
	prior = window._drawing_parameters.snapshot()
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		assert window._drawing_parameters.set_order_name("double")
		seen = []
		monkeypatch.setattr(
			tab, "begin_direct_bond_gesture",
			lambda atom_id, presentation, snap_enabled: seen.append(
				(atom_id, presentation, snap_enabled),
			) or object(),
		)
		window._draw_bond_action.trigger()
		assert window._line_gesture_intent.drawing.order_name == "double"
		assert window._drawing_parameters.set_order_name("triple")
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			_viewport_point(tab, "atom-c"),
		)
		qapp.processEvents()
		assert seen == [(
			"atom-c", ferrum_qt.ferrum.engine.DocumentBondPresentationV1.normal_double,
			tab.view.hex_grid_snap_enabled,
		)]
		window._reset_line_gesture_start()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			_viewport_point(tab, "atom-c"),
		)
		qapp.processEvents()
		assert [presentation for _atom_id, presentation, _snap in seen] == [
			ferrum_qt.ferrum.engine.DocumentBondPresentationV1.normal_double,
			ferrum_qt.ferrum.engine.DocumentBondPresentationV1.normal_double,
		]
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		assert window._line_gesture_intent is None
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			_viewport_point(tab, "atom-c"),
		)
		qapp.processEvents()
		assert seen[-1][1] is (
			ferrum_qt.ferrum.engine.DocumentBondPresentationV1.normal_triple
		)
	finally:
		window._drawing_parameters.set_element(prior.element)
		window._drawing_parameters.set_order_name(prior.order_name)
		window._drawing_parameters.set_presentation_name(prior.presentation_name)
		window.close()
		window.deleteLater()


#============================================
def test_escape_discards_direct_bond_admission_without_a_mutation(
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
		window.close()
		window.deleteLater()


#============================================
def test_history_retire_armed_direct_bond_before_each_transition(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""Undo and Redo clear an armed pointer action before one Rust transition."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-history-retirement.cdml",
	)
	try:
		monkeypatch.setattr(window, "_show_edit_refusal", lambda *_args: None)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
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
		assert window._line_gesture_intent is not None
		assert window._draw_bond_action.isChecked()
		committed_revision = tab.current_snapshot.revision
		history_entry_states = []
		undo = tab.undo
		redo = tab.redo
		monkeypatch.setattr(
			tab, "undo",
			lambda: history_entry_states.append((
				"undo", window._line_gesture_intent, window._draw_bond_action.isChecked(),
			)) or undo(),
		)
		monkeypatch.setattr(
			tab, "redo",
			lambda: history_entry_states.append((
				"redo", window._line_gesture_intent, window._draw_bond_action.isChecked(),
			)) or redo(),
		)

		window._undo_action.trigger()
		assert history_entry_states == [("undo", None, False)]
		assert window._line_gesture_intent is None
		assert not window._draw_bond_action.isChecked()
		assert tab.current_snapshot.revision == committed_revision + 1

		window._draw_bond_action.trigger()
		assert window._line_gesture_intent is not None
		assert window._draw_bond_action.isChecked()
		undone_revision = tab.current_snapshot.revision
		window._redo_action.trigger()
		assert history_entry_states == [("undo", None, False), ("redo", None, False)]
		assert window._line_gesture_intent is None
		assert not window._draw_bond_action.isChecked()
		assert tab.current_snapshot.revision == undone_revision + 1
	finally:
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
		window.close()
		window.deleteLater()


#============================================
def test_implicit_direct_bond_endpoint_uses_its_existing_source_identifier(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""A unique implicit carbon is an existing endpoint, never a new atom."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-implicit-existing-endpoint.cdml",
	)
	try:
		monkeypatch.setattr(window, "_show_edit_refusal", lambda *_args: None)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-o")
		implicit = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		end = implicit + PySide6.QtCore.QPoint(6, 0)
		before = tab.current_snapshot.revision
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
		assert tab.current_snapshot.revision == before + 1
		assert len(molecule.atoms) == 2
		assert len(molecule.bonds) == 1
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_ambiguous_implicit_direct_bond_endpoint_cancels_without_admission(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An equal implicit-carbon tie cannot become a new Rust endpoint."""
	cdml = """<cdml xmlns='http://www.freesoftware.fsf.org/bkchem/cdml'>
  <molecule id='mol-1'>
    <atom id='atom-o' name='O'><point x='0' y='0'/></atom>
    <atom id='atom-c-left' name='C'><point x='30' y='30'/></atom>
    <atom id='atom-c-right' name='C'><point x='42' y='30'/></atom>
  </molecule>
</cdml>"""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		cdml, "direct-bond-ambiguous-endpoint.cdml",
	)
	refusals = []
	try:
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request, _details=None: refusals.append(request),
		)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-o")
		midpoint = tab.view.mapFromScene(PySide6.QtCore.QPointF(36.0, 30.0))
		before = tab.current_snapshot.revision
		monkeypatch.setattr(
			tab, "admit_direct_bond_candidate",
			lambda *_args: pytest.fail("ambiguous endpoint reached Rust admission"),
		)
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), midpoint)
		PySide6.QtTest.QTest.mouseRelease(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, midpoint,
		)
		qapp.processEvents()
		assert refusals
		assert window._line_gesture_intent is None
		assert tab.current_snapshot.revision == before
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_invalid_implicit_direct_bond_identity_cancels_without_a_bridge(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An in-radius projection atom without a source identity is terminal."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-invalid-endpoint-identity.cdml",
	)
	refusals = []
	try:
		monkeypatch.setattr(
			window, "_show_edit_refusal", lambda request, _details=None: refusals.append(request),
		)
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-c")
		endpoint = tab.view.mapFromScene(PySide6.QtCore.QPointF(130.0, 20.0))
		invalid_atom = types.SimpleNamespace(
			position=types.SimpleNamespace(x=130.0, y=20.0), source_id=None,
		)
		invalid_observation = types.SimpleNamespace(
			projection=types.SimpleNamespace(
				molecules=(types.SimpleNamespace(atoms=(invalid_atom,)),),
			),
		)
		before = tab.current_snapshot.revision
		monkeypatch.setattr(tab, "durable_atom_at_viewport_point", lambda _point: None)
		monkeypatch.setattr(tab, "current_document_observation", lambda: invalid_observation)
		monkeypatch.setattr(
			tab, "direct_bond_existing_endpoint",
			lambda *_args: pytest.fail("invalid endpoint reached the existing-atom bridge"),
		)
		monkeypatch.setattr(
			tab, "direct_bond_new_endpoint",
			lambda *_args: pytest.fail("invalid endpoint allocated a new atom"),
		)
		monkeypatch.setattr(
			tab, "admit_direct_bond_candidate",
			lambda *_args: pytest.fail("invalid endpoint reached Rust admission"),
		)
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		intent = window._line_gesture_intent
		window._update_direct_bond_gesture(intent, endpoint)
		assert refusals
		assert window._line_gesture_intent is None
		assert tab.current_snapshot.revision == before
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_unknown_direct_bond_endpoint_outcome_fails_before_rust_admission(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An unrecognized endpoint result cannot allocate or submit a candidate."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-unknown-endpoint.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		start = _viewport_point(tab, "atom-c")
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		intent = window._line_gesture_intent
		monkeypatch.setattr(tab, "_classify_direct_bond_endpoint_at_viewport_point", lambda _point: object())
		monkeypatch.setattr(
			tab, "direct_bond_new_endpoint",
			lambda *_args: pytest.fail("unknown endpoint allocated a new atom"),
		)
		monkeypatch.setattr(
			tab, "admit_direct_bond_candidate",
			lambda *_args: pytest.fail("unknown endpoint reached Rust admission"),
		)
		with pytest.raises(RuntimeError, match="^Ferrum direct-bond endpoint classifier returned an unknown result$"):
			window._update_direct_bond_gesture(intent, start)
	finally:
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
		window.close()
		window.deleteLater()


#============================================
def test_unexpected_direct_bond_commit_error_propagates_without_refusal(
		qapp: PySide6.QtWidgets.QApplication, monkeypatch: object,
		) -> None:
	"""An unexpected receipt-commit failure remains visible to the caller."""
	window = ferrum_qt.main_window.MainWindow(object())
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_EDITABLE_CDML, "direct-bond-unexpected-commit.cdml",
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
		start = _viewport_point(tab, "atom-c")
		end = _viewport_point(tab, "atom-o")
		window._draw_bond_action.trigger()
		PySide6.QtTest.QTest.mousePress(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, start,
		)
		PySide6.QtTest.QTest.mouseMove(tab.view.viewport(), end)
		qapp.processEvents()
		monkeypatch.setattr(
			tab, "commit_direct_bond_admission",
			lambda _admission: (_ for _ in ()).throw(RuntimeError("projection failed")),
		)
		class _ReleaseEvent:
			def position(self) -> PySide6.QtCore.QPointF:
				return PySide6.QtCore.QPointF(end)
		with pytest.raises(RuntimeError, match="^projection failed$"):
			window._complete_line_gesture(_ReleaseEvent())
		assert not refusals
	finally:
		window.close()
		window.deleteLater()
