"""Focused Qt behavior for the Rust-authoritative reaction role composer."""

# Standard Library
import pathlib
import types

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.reaction_composer
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.engine


_REACTION_CDML = """<cdml version='26.08'>
<molecule id='left'><atom id='left-a' name='C'><point x='0' y='0'/></atom></molecule>
<molecule id='right'><atom id='right-a' name='O'><point x='160' y='0'/></atom></molecule>
<arrow id='arrow'><point x='40' y='0'/><point x='120' y='0'/></arrow>
</cdml>"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one offscreen application for modeless composer widgets."""
	app = PySide6.QtWidgets.QApplication.instance()
	return app if app is not None else PySide6.QtWidgets.QApplication([])


#============================================
def _choice(identifier: str, kind: str, order: int) -> object:
	"""Make one frozen-shape stand-in without a document or XML route."""
	return types.SimpleNamespace(
		identifier=identifier, kind=kind, source_order=order,
		availability="eligible", label=f"{kind} {identifier}",
	)


#============================================
def _exclusion(identifier: str, reason: str, recovery: str) -> object:
	"""Make one backend-shaped unavailable root diagnostic for Qt-only rendering."""
	return types.SimpleNamespace(
		diagnostic_key=identifier, reason=reason, recovery=recovery,
		label=f"Vector {identifier}",
	)


#============================================
def _check(panel: object, role: str, identifier: str) -> None:
	"""Choose a role row through its visible check state."""
	list_widget = panel._lists[role]
	for index in range(list_widget.count()):
		item = list_widget.item(index)
		if item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == identifier:
			item.setCheckState(PySide6.QtCore.Qt.CheckState.Checked)
			return
	raise AssertionError(f"missing reaction composer row {identifier!r}")


#============================================
def test_role_lists_preserve_source_order_and_exclusive_membership(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A molecule cannot be both reactant and product and arrow is singular."""
	parent = PySide6.QtWidgets.QWidget()
	panel = ferrum_qt.ferrum.reaction_composer._ReactionComposerPanel(
		(
			_choice("product", "molecule", 4), _choice("arrow", "arrow", 2),
			_choice("reactant", "molecule", 1),
		), (), parent,
	)
	try:
		_check(panel, "reactants", "reactant")
		_check(panel, "products", "product")
		_check(panel, "arrow", "arrow")
		_check(panel, "products", "reactant")
		reactants, products, arrow, _conditions, _pluses = panel.request()
		assert reactants == []
		assert (products, arrow) == (["reactant", "product"], "arrow")
	finally:
		parent.deleteLater()


#============================================
def test_escape_cancels_without_emitting_a_submit(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Escape is a panel-only cancellation even after a valid role assignment."""
	parent = PySide6.QtWidgets.QWidget()
	panel = ferrum_qt.ferrum.reaction_composer._ReactionComposerPanel(
		(
			_choice("left", "molecule", 1), _choice("right", "molecule", 2),
			_choice("arrow", "arrow", 3),
		), (), parent,
	)
	cancelled = []
	submitted = []
	panel.cancelled.connect(lambda: cancelled.append(True))
	panel.submitted.connect(lambda: submitted.append(True))
	try:
		_check(panel, "reactants", "left")
		_check(panel, "products", "right")
		_check(panel, "arrow", "arrow")
		event = PySide6.QtGui.QKeyEvent(
			PySide6.QtCore.QEvent.Type.KeyPress, PySide6.QtCore.Qt.Key.Key_Escape,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
		)
		panel.keyPressEvent(event)
		assert cancelled == [True]
		assert submitted == []
	finally:
		parent.deleteLater()


#============================================
def test_vector_exclusions_are_unavailable_diagnostics_with_recovery(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A vector exclusion never enters role or selected-usable-root rows."""
	parent = PySide6.QtWidgets.QWidget()
	panel = ferrum_qt.ferrum.reaction_composer._ReactionComposerPanel(
		(_choice("molecule", "molecule", 1),),
		(_exclusion("vector", "display_only", "choose_supported_member"),), parent,
	)
	try:
		usable = panel.findChild(
			PySide6.QtWidgets.QListWidget, "reaction-composer-selected-usable-roots",
		)
		unavailable = panel.findChild(
			PySide6.QtWidgets.QListWidget, "reaction-composer-unavailable-roots",
		)
		assert usable is not None and unavailable is not None
		assert "vector" not in [usable.item(index).text() for index in range(usable.count())]
		assert unavailable.count() == 1
		assert "Reason: display_only" in unavailable.item(0).text()
		assert "Recovery: choose_supported_member" in unavailable.item(0).text()
		assert "Choose a supported molecule" in unavailable.item(0).text()
		assert all("vector" not in panel._lists[role].item(index).text()
			for role in panel._lists for index in range(panel._lists[role].count()))
	finally:
		parent.deleteLater()


#============================================
def test_live_window_commits_roles_only_through_the_rust_reaction_bridge(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		tmp_path: pathlib.Path,
		) -> None:
	"""The ordinary ribbon route reprojects an accepted native reaction commit."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_REACTION_CDML, "reaction-input.cdml",
	)
	main_window._register_native_tab(tab, activate=True)
	observation = tab.observe_direct_root_interaction()
	selection = None
	for identifier in ("left", "right", "arrow"):
		modifier = (
			ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
		)
		selection = tab.select_direct_roots(
			observation, selection,
			ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(identifier, modifier),
		)
	main_window._replace_render_interaction_selection(selection, tab)
	main_window._create_reaction_action.trigger()
	qapp.processEvents()
	panel = main_window._reaction_composer._panel
	assert panel is not None, main_window.statusBar().currentMessage()
	try:
		_check(panel, "reactants", "left")
		_check(panel, "products", "right")
		_check(panel, "arrow", "arrow")
		panel.submitted.emit()
		qapp.processEvents()
		assert "<reaction id=\"rxn-1\"" in tab.current_snapshot.cdml
		assert {root.identifier for root in main_window._render_interaction_selection.roots} == {
			"left", "right", "arrow",
		}
	finally:
		# Persist the accepted transaction before teardown so the live dirty-tab
		# policy does not open its interactive close confirmation during pytest.
		tab.save_atomic(tmp_path / "reaction-input.cdml")
		main_window._reaction_composer.close()
		tab.dispose()


#============================================
def test_window_focus_loss_terminally_retires_composer_without_document_mutation(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Window deactivation clears only disposable reaction form and root-selection state."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_REACTION_CDML, "reaction-focus-loss.cdml",
	)
	main_window._register_native_tab(tab, activate=True)
	observation = tab.observe_direct_root_interaction()
	selection = None
	for identifier in ("left", "right", "arrow"):
		modifier = (
			ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace
			if selection is None else ferrum_qt.ferrum.engine.RenderInteractionModifierV1.toggle
		)
		selection = tab.select_direct_roots(
			observation, selection,
			ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(identifier, modifier),
		)
	main_window._replace_render_interaction_selection(selection, tab)
	main_window._create_reaction_action.trigger()
	qapp.processEvents()
	composer = main_window._reaction_composer
	panel = composer._panel
	assert panel is not None
	_check(panel, "reactants", "left")
	before = tab.current_snapshot.cdml
	qapp.sendEvent(main_window, PySide6.QtCore.QEvent(
		PySide6.QtCore.QEvent.Type.WindowDeactivate,
	))
	qapp.processEvents()
	try:
		assert tab.current_snapshot.cdml == before
		assert composer._dock is None and composer._panel is None
		assert composer._choices is None and composer._revision is None and composer._digest is None
		assert main_window._render_interaction_selection is None
		assert main_window._create_reaction_action.isEnabled()
		assert "Select the reaction members again" in main_window.statusBar().currentMessage()
	finally:
		tab.dispose()


#============================================
def test_canvas_focus_transition_keeps_the_live_reaction_composer(
		main_window: object, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A normal selection click moving focus into this window's canvas is not focus loss."""
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_REACTION_CDML, "reaction-canvas-focus.cdml",
	)
	main_window._register_native_tab(tab, activate=True)
	observation = tab.observe_direct_root_interaction()
	selection = tab.select_direct_roots(
		observation, None,
		ferrum_qt.ferrum.engine.RenderInteractionQueryV1.root(
			"left", ferrum_qt.ferrum.engine.RenderInteractionModifierV1.replace,
		),
	)
	main_window._replace_render_interaction_selection(selection, tab)
	main_window._create_reaction_action.trigger()
	qapp.processEvents()
	composer = main_window._reaction_composer
	panel = composer._panel
	assert panel is not None
	try:
		composer._on_application_focus_changed(panel, tab.view.viewport())
		assert composer._panel is panel
		assert main_window._render_interaction_selection is selection
	finally:
		composer.close()
		tab.dispose()
