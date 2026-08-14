"""Behavior and import-boundary coverage for scalar display geometry."""

# Standard Library
import ast
import math
import pathlib

# PIP3 modules
import PySide6.QtWidgets
import pytest
import ferrum_chem

# local repo modules
import ferrum_qt.bridge.display_geometry
import ferrum_qt.config.geometry_units
import ferrum_qt.widgets.periodic_table


_CONSUMER_MODULES = (
	"ferrum_qt/widgets/periodic_table.py",
	"ferrum_qt/canvas/scene.py",
	"ferrum_qt/config/geometry_units.py",
	"ferrum_qt/canvas/items/render_ops_painter.py",
	"ferrum_qt/canvas/items/atom_item.py",
	"ferrum_qt/canvas/items/bond_item.py",
)
_QT_PACKAGE_ROOT = pathlib.Path(__file__).parents[1] / "ferrum_qt"


#============================================
def test_scalar_geometry_bridge_returns_immutable_finite_lattice_values() -> None:
	"""A bounded grid request returns safe scalar facts instead of OASA containers."""
	points = ferrum_qt.bridge.display_geometry.hex_grid_points(
		0.0, 0.0, 120.0, 90.0, 30.0,
	)
	edges = ferrum_qt.bridge.display_geometry.hex_grid_edges(
		0.0, 0.0, 120.0, 90.0, 30.0,
	)

	assert isinstance(points, tuple) and all(math.isfinite(value) for point in points for value in point)
	assert isinstance(edges, tuple) and all(
		math.isfinite(value) for edge in edges for point in edge for value in point
	)


#============================================
def test_scalar_geometry_bridge_rejects_nonfinite_snap_input() -> None:
	"""Invalid interaction coordinates receive the stable typed boundary error."""
	with pytest.raises(ferrum_chem.GeometryError, match="must be finite"):
		ferrum_qt.bridge.display_geometry.snap_to_hex_grid(math.inf, 0.0, 40.0)


#============================================
@pytest.mark.parametrize("invalid_spacing", (math.nan, math.inf, True, 0.0, -1.0))
def test_scene_keeps_its_grid_when_spacing_is_invalid(
		main_window: object, invalid_spacing: object,
		) -> None:
	"""A rejected spacing update preserves the current visible interaction lattice."""
	scene = main_window.scene
	before = scene.grid_spacing_pt
	scene.set_grid_spacing_pt(invalid_spacing)

	assert scene.grid_spacing_pt == before


#============================================
def test_scene_spacing_commit_keeps_current_grid_when_bridge_build_fails(
		main_window: object, monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""A failed new projection leaves the old visible lattice authoritative."""
	scene = main_window.scene
	overlay = scene._grid_overlay
	before = scene.grid_spacing_pt

	def fail_grid_build(*args: object) -> tuple:
		del args
		raise ValueError("simulated display-geometry failure")

	monkeypatch.setattr(
		ferrum_qt.bridge.display_geometry, "hex_grid_edges", fail_grid_build,
	)
	with pytest.raises(ValueError, match="simulated display-geometry failure"):
		scene.set_grid_spacing_pt(52.0)

	assert scene.grid_spacing_pt == before and scene._grid_overlay is overlay


#============================================
@pytest.mark.parametrize("invalid_value", (math.nan, math.inf, True))
def test_scalar_geometry_bridge_rejects_non_numeric_coordinate_forms(
		invalid_value: object,
		) -> None:
	"""The boundary rejects values that cannot describe a CDML coordinate."""
	with pytest.raises(ferrum_chem.GeometryError):
		ferrum_qt.bridge.display_geometry.cm_to_points(invalid_value)


#============================================
def test_geometry_units_round_trip_through_plain_bridge_scale() -> None:
	"""Qt configuration preserves the backend CDML physical coordinate scale."""
	centimetres = 2.54
	converted = ferrum_qt.config.geometry_units.pt_to_cm(
		ferrum_qt.config.geometry_units.cm_to_pt(centimetres)
	)

	assert converted == pytest.approx(centimetres)


#============================================
def test_periodic_table_widget_projects_bridge_color(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A visible element button receives its normalized scalar bridge color."""
	del qapp
	dialog = ferrum_qt.widgets.periodic_table.PeriodicTablePopup()
	carbon_button = next(
		button for button in dialog.findChildren(PySide6.QtWidgets.QPushButton)
		if button.text() == "C"
	)
	color = ferrum_qt.bridge.display_geometry.element_category_color("C")
	style_sheet = carbon_button.styleSheet()
	dialog.close()
	dialog.deleteLater()

	assert color in style_sheet


#============================================
def test_periodic_table_symbols_are_exactly_the_ferrum_picker_catalog() -> None:
	"""The Qt popup exposes every and only backend-supported picker symbol."""
	widget_symbols = tuple(entry[0] for entry in ferrum_qt.widgets.periodic_table.ELEMENTS)
	backend_entries = ferrum_chem.periodic_display_entries_v1()

	assert widget_symbols == tuple(entry.symbol for entry in backend_entries)
	assert all(
		entry.color == ferrum_qt.bridge.display_geometry.element_category_color(entry.symbol)
		for entry in backend_entries
	)


#============================================
def test_display_geometry_consumers_have_no_direct_oasa_imports() -> None:
	"""Qt consumers receive OASA-derived display facts only through their bridge."""
	violations = []
	for relative_path in _CONSUMER_MODULES:
		path = _QT_PACKAGE_ROOT / relative_path.removeprefix("ferrum_qt/")
		module = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
		for node in ast.walk(module):
			if isinstance(node, ast.Import):
				for alias in node.names:
					if alias.name == "oasa" or alias.name.startswith("oasa."):
						violations.append(f"{relative_path}:{node.lineno}:{alias.name}")
			if isinstance(node, ast.ImportFrom) and node.module is not None:
				if node.module == "oasa" or node.module.startswith("oasa."):
					violations.append(f"{relative_path}:{node.lineno}:{node.module}")

	assert not violations, "direct OASA imports in Qt display consumers: " + ", ".join(violations)


#============================================
def test_display_geometry_has_no_oasa_color_catalog_residual() -> None:
	"""The color bridge is now a direct typed Ferrum boundary."""
	path = _QT_PACKAGE_ROOT / "bridge" / "display_geometry.py"
	module = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
	imports = [
		alias.name
		for node in ast.walk(module)
		if isinstance(node, ast.Import)
		for alias in node.names
		if alias.name == "oasa" or alias.name.startswith("oasa.")
	]

	assert imports == []
