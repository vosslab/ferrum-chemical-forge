"""Test that ThemeManager toggle switches palette colors to match YAML values.

Verifies that apply_theme changes the QPalette Window color to match the
YAML gui.background value for each theme.

Usage:
	source source_me.sh && python -m pytest packages/ferrum-qt.app/tests/test_qt_theme_toggle_runtime.py -v
"""

# Standard Library
import types

# PIP3 modules
import pytest
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.display_palette_refreshable
import ferrum_qt.canvas.items.ferrum_plan_item
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.hex_grid
import ferrum_qt.main_window
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_loader
import ferrum_qt.themes.theme_manager


_NORMAL_MOLECULE_CDML = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><standard line_width="9"/><paper id="paper"/>
<molecule id="root"><atom id="oxygen" name="O"><point x="0" y="0"/></atom></molecule></cdml>
"""


class _Refreshable(ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1):
	"""Capture one tab-owned display refresh delivery."""

	def __init__(self) -> None:
		"""Start before the tab's next typed theme handoff."""
		self.palette: object | None = None

	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Retain the exact palette received through the tab boundary."""
		self.palette = palette


#============================================
def _action(
		window: PySide6.QtWidgets.QMainWindow,
		text: str,
		) -> PySide6.QtGui.QAction:
	"""Return the visible action with the requested user-facing label."""
	return next(action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == text)


#============================================
def _malformed_document_display() -> dict[str, object]:
	"""Return an inline YAML display map missing one required role."""
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	document_display = {
		role.value: palette.color(role).name()
		for role in ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1
	} | {"elements": {}}
	document_display.pop("page_fill")
	return document_display


#============================================
def _plan_material_rgba(
		item: ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem,
		) -> tuple[int, ...]:
	"""Return all cached plan materials without inspecting renderer geometry."""
	result = []
	for command in item._commands:
		if isinstance(command, ferrum_qt.canvas.items.ferrum_plan_item._Line):
			result.append(command.pen.color().rgba())
		elif isinstance(command, ferrum_qt.canvas.items.ferrum_plan_item._Fill):
			result.append(command.brush.color().rgba())
		else:
			if command.pen is not None:
				result.append(command.pen.color().rgba())
			if command.brush is not None:
				result.append(command.brush.color().rgba())
	return tuple(result)


#============================================
def test_theme_toggle_changes_palette(qapp: object, theme_manager: object) -> None:
	"""Verify apply_theme switches palette Window color to match YAML values."""
	# apply dark theme and check palette
	theme_manager.apply_theme('dark')
	dark_bg = qapp.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Window
	).name()
	assert dark_bg == '#2b2b2b', f'Expected #2b2b2b, got {dark_bg}'
	# switch to light and check
	theme_manager.apply_theme('light')
	light_bg = qapp.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Window
	).name()
	assert light_bg == '#eaeaea', f'Expected #eaeaea, got {light_bg}'


#============================================
def test_theme_toggle_roundtrip(qapp: object, theme_manager: object) -> None:
	"""Verify dark -> light -> dark roundtrip preserves palette colors."""
	theme_manager.apply_theme('dark')
	theme_manager.apply_theme('light')
	theme_manager.apply_theme('dark')
	dark_bg = qapp.palette().color(
		PySide6.QtGui.QPalette.ColorRole.Window
	).name()
	assert dark_bg == '#2b2b2b', f'Roundtrip failed: expected #2b2b2b, got {dark_bg}'


#============================================
def test_invalid_document_palette_preserves_active_application_theme(
		qapp: object,
		theme_manager: object,
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Theme validation fails before QApplication or the active theme can change."""
	theme_manager.apply_theme("light")
	before_state = (
		theme_manager.current_theme,
		qapp.palette().color(PySide6.QtGui.QPalette.ColorRole.Window).rgba(),
		qapp.styleSheet(),
	)
	malformed_document_display = _malformed_document_display()

	def load_malformed_theme(_name: str) -> dict:
		"""Supply invalid YAML to the real document-display loader."""
		return {"document_display": malformed_document_display}

	monkeypatch.setattr(
		ferrum_qt.themes.theme_loader,
		"_load_theme",
		load_malformed_theme,
	)
	with pytest.raises(
		ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError
	):
		theme_manager.apply_theme("dark")
	assert (
		theme_manager.current_theme,
		qapp.palette().color(PySide6.QtGui.QPalette.ColorRole.Window).rgba(),
		qapp.styleSheet(),
	) == before_state


#============================================
def test_theme_change_emits_name_and_display_palette(
		theme_manager: object,
		) -> None:
	"""Listeners receive one typed complete palette after an applied theme change."""
	emissions = []
	theme_manager.theme_changed.connect(emissions.append)
	theme_manager.apply_theme("dark")
	change, = emissions
	expected_palette = ferrum_qt.themes.theme_loader.get_document_display_palette(
		"dark"
	)
	assert isinstance(change, ferrum_qt.themes.theme_manager.ThemeChangeV1)
	assert change.name == "dark"
	assert change.palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PAGE_FILL
	).name() == expected_palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PAGE_FILL
	).name()


#============================================
def test_theme_change_reaches_live_tab_without_document_or_ui_mutation(
		qapp: PySide6.QtWidgets.QApplication,
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		main_window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""A live tab receives the exact palette without replacing user-owned state."""
	main_window.show()
	qapp.processEvents()
	tab = main_window.centralWidget().currentWidget()
	assert isinstance(tab, ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab)
	tab.view.viewport().setFocus()
	qapp.processEvents()
	undo_action = _action(main_window, "Undo")
	before_session = tab._session
	before_snapshot = tab.current_snapshot
	before_revision = before_snapshot.revision
	before_focus = qapp.focusWidget()
	emissions = []
	theme_manager.theme_changed.connect(emissions.append)
	target_theme = "light" if theme_manager.current_theme == "dark" else "dark"
	theme_manager.apply_theme(target_theme)
	qapp.processEvents()
	change, = emissions
	assert tab.document_display_palette is change.palette
	assert tab._session is before_session
	assert tab.current_snapshot is before_snapshot
	assert tab.current_snapshot.revision == before_revision
	assert qapp.focusWidget() is before_focus
	assert _action(main_window, "Undo") is undo_action


#============================================
def test_typed_theme_change_replaces_the_tab_palette_without_document_change(
		qapp: object,
		) -> None:
	"""One tab owns the current palette and refreshes only retained Qt material."""
	light = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	dark = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "palette.cdml", light,
	)
	refreshable = _Refreshable()
	try:
		before_snapshot = tab.current_snapshot
		tab.register_document_display_refreshable(refreshable)
		tab.apply_theme_change(
			ferrum_qt.themes.theme_manager.ThemeChangeV1("dark", dark),
		)
		assert tab.document_display_palette is dark
		assert refreshable.palette is dark
		assert tab.current_snapshot is before_snapshot
	finally:
		tab.dispose()
		qapp.processEvents()


#============================================
def test_normal_projection_palette_refresh_preserves_retained_native_roots(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A normal native projection refreshes plan, paper, and grid material in place."""
	light = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	dark = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_NORMAL_MOLECULE_CDML, "palette-root.cdml", light,
	)
	try:
		projection = tab._controller.projection
		grid = tab.view._hex_grid_item
		assert projection is not None and grid is not None
		plan, = tuple(
			item for item in projection.items
			if isinstance(item, ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem)
		)
		paper = projection.paper
		assert all(isinstance(item, ferrum_qt.canvas.display_palette_refreshable.DisplayPaletteRefreshable)
			for item in (plan, paper, grid))
		before = (
			projection.revision, projection.digest, projection.item_targets[plan],
			plan.boundingRect(), paper.rect(), grid.boundingRect(),
			_plan_material_rgba(plan),
			paper.pen().color().rgba(), paper.brush().color().rgba(),
			grid._line_pen.color().rgba(), grid._dot_brush.color().rgba(),
		)
		tab.apply_theme_change(
			ferrum_qt.themes.theme_manager.ThemeChangeV1("dark", dark),
		)
		after = (
			projection.revision, projection.digest, projection.item_targets[plan],
			plan.boundingRect(), paper.rect(), grid.boundingRect(),
			_plan_material_rgba(plan),
			paper.pen().color().rgba(), paper.brush().color().rgba(),
			grid._line_pen.color().rgba(), grid._dot_brush.color().rgba(),
		)
		assert after[:6] == before[:6]
		assert after[6:] != before[6:]
	finally:
		tab.dispose()
		qapp.processEvents()


#============================================
def test_transient_preview_refresh_preserves_document_geometry_and_focus(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A live direct-root preview changes material without changing retained state."""
	light = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	dark = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "preview.cdml", light,
	)
	root = None
	try:
		tab.show()
		qapp.processEvents()
		tab.view.viewport().setFocus()
		qapp.processEvents()
		bounds = types.SimpleNamespace(left=10.0, top=20.0, right=40.0, bottom=60.0)
		root = ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
			tab, (bounds,),
		)
		before_snapshot = tab.current_snapshot
		before_path = root.path()
		before_root_bounds = root.boundingRect()
		before_focus = tab.view.viewport().hasFocus()
		tab.apply_theme_change(
			ferrum_qt.themes.theme_manager.ThemeChangeV1("dark", dark),
		)
		assert root.pen().color().name() == dark.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_OUTLINE,
		).name()
		assert root.brush().color().name() == dark.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PREVIEW_FILL,
		).name()
		assert root.path() == before_path
		assert root.boundingRect() == before_root_bounds
		assert tab.current_snapshot is before_snapshot
		assert tab.view.viewport().hasFocus() is before_focus
	finally:
		ferrum_qt.ferrum.document_display_refresh.unregister_attached_document_display_refreshable(
			root,
		)
		tab.dispose()
		qapp.processEvents()


#============================================
def test_detached_transient_preview_receives_no_later_palette_refresh(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The centralized preview-release path ends the tab registry lifetime first."""
	light = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	dark = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "released-preview.cdml", light,
	)
	root = None
	try:
		bounds = types.SimpleNamespace(left=1.0, top=2.0, right=3.0, bottom=4.0)
		root = ferrum_qt.ferrum.direct_root_preview.create_direct_root_bounds_preview(
			tab, (bounds,),
		)
		before_color = root.pen().color().name()
		ferrum_qt.ferrum.document_display_refresh.unregister_attached_document_display_refreshable(
			root,
		)
		tab.apply_theme_change(
			ferrum_qt.themes.theme_manager.ThemeChangeV1("dark", dark),
		)
		assert root.pen().color().name() == before_color
	finally:
		ferrum_qt.ferrum.document_display_refresh.unregister_attached_document_display_refreshable(
			root,
		)
		tab.dispose()
		qapp.processEvents()
