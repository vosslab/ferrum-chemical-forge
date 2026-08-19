"""Fast behavioral checks for the disposable Ferrum grid projection."""

# Standard Library
import collections.abc
import math

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.canvas.scene


#============================================
@pytest.fixture
def scene(
		qapp: object,
		) -> collections.abc.Iterator[ferrum_qt.canvas.scene.ChemScene]:
	"""Create one isolated disposable grid scene."""
	del qapp
	canvas_scene = ferrum_qt.canvas.scene.ChemScene(theme_name="light")
	yield canvas_scene
	canvas_scene.deleteLater()


#============================================
def test_grid_visibility_tracks_the_live_overlay(scene: object) -> None:
	"""Hiding and restoring the grid updates the one visible projection."""
	overlay = scene._grid_overlay
	scene.set_grid_visible(False)
	hidden = not scene.grid_visible and not overlay.isVisible()
	scene.set_grid_visible(True)
	assert hidden and scene.grid_visible and overlay.isVisible()


#============================================
def test_grid_uses_one_disposable_scene_item(scene: object) -> None:
	"""The grid contributes one scene-owned decoration, not a child forest."""
	overlay = scene._grid_overlay
	assert overlay in scene.items() and overlay.childItems() == []


#============================================
def test_grid_theme_changes_keep_one_live_projection(scene: object) -> None:
	"""A theme update preserves the visible grid projection identity."""
	overlay = scene._grid_overlay
	scene.apply_theme("light")
	assert scene._grid_overlay is overlay and overlay.isVisible()


#============================================
def test_grid_spacing_preserves_hex_snap_behavior(scene: object) -> None:
	"""A geometry refresh leaves snapping on the requested hex lattice."""
	scene.set_grid_spacing_pt(48.0)
	snapped = scene.snap_to_grid(53.0, 53.0)
	n_index = snapped[0] / (math.sqrt(3.0) * 48.0 / 2.0)
	m_index = snapped[1] / 48.0 - round(n_index) / 2.0
	assert (n_index, m_index) == pytest.approx(
		(round(n_index), round(m_index)), abs=1e-6,
	)
