"""Offscreen QGraphicsScene coverage for V2 test-only measurement capture."""

# Standard Library
import json
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
from PySide6.QtCore import Qt
from PySide6.QtGui import QBrush, QColor, QPainterPath
from PySide6.QtWidgets import QApplication, QGraphicsItemGroup, QGraphicsPathItem, QGraphicsScene

# Local modules
from measure_stack.contracts import CaptureProfile, RASTER_LAYER_MANIFEST_V2_SCHEMA, load_raster_manifest_v2
from measure_stack.qt_scene_capture import capture_scene


# ============================================
def _path_item(left: float, top: float, width: float, height: float) -> QGraphicsPathItem:
	"""Return one visible test paint item with deterministic opaque ink."""
	path = QPainterPath()
	path.addRect(left, top, width, height)
	item = QGraphicsPathItem(path)
	item.setPen(Qt.PenStyle.NoPen)
	item.setBrush(QBrush(QColor("black")))
	return item


# ============================================
def _profile() -> CaptureProfile:
	"""Return the named fixed profile used by every fixture in this focused test."""
	return CaptureProfile("test-normal-scale-v2", (-10.0, -10.0, 120.0, 80.0), 240, 160, 1.0, "presentation")


# ============================================
def _scene_with_actual_chemical_root() -> tuple[
		QGraphicsScene, QGraphicsItemGroup, QGraphicsPathItem, QGraphicsPathItem,
		QGraphicsPathItem, QGraphicsPathItem, QGraphicsPathItem,
		]:
	"""Build a root whose full label includes a decoration outside its core body."""
	scene = QGraphicsScene()
	paper = _path_item(-50.0, -50.0, 240.0, 180.0)
	paper.setBrush(QBrush(QColor("white")))
	scene.addItem(paper)
	root = QGraphicsItemGroup()
	scene.addItem(root)
	full_a = _path_item(0.0, 20.0, 8.0, 12.0)
	# The charge/decorative part is deliberately outside core ink but in the actual label item.
	full_path = full_a.path()
	full_path.addRect(10.0, 16.0, 5.0, 5.0)
	full_a.setPath(full_path)
	full_b = _path_item(92.0, 20.0, 8.0, 12.0)
	bond = _path_item(18.0, 24.0, 66.0, 4.0)
	for item in (full_a, full_b, bond):
		item.setParentItem(root)
	# The core path is a test-only isolated source map and is excluded from the composite root.
	core_a = _path_item(0.0, 20.0, 8.0, 12.0)
	core_b = _path_item(92.0, 20.0, 8.0, 12.0)
	core_a.setVisible(False)
	core_b.setVisible(False)
	scene.addItem(core_a)
	scene.addItem(core_b)
	return scene, root, full_a, full_b, bond, core_a, core_b


# ============================================
def test_capture_uses_fixed_profile_actual_full_labels_and_restores_visibility(
		tmp_path: pathlib.Path) -> None:
	"""Composite includes actual decoration while core/full/bond layers stay distinct."""
	app = QApplication.instance() or QApplication([])
	scene, root, full_a, full_b, bond, core_a, core_b = _scene_with_actual_chemical_root()
	root.setVisible(True)
	full_a.setVisible(True)
	full_b.setVisible(False)
	bond.setVisible(True)
	manifest_path = capture_scene(
		scene,
		_profile(),
		"actual_label_decoration",
		"<cdml id='actual_label_decoration'/>",
		(root,),
		{"a": ("N", full_a, core_a), "b": ("O", full_b, core_b)},
		{"ab": ("a", "b", "normal", bond)},
		[
			{"relation": "bond_endpoint", "subject_id": "ab", "object_id": "a", "expectation": "required"},
			{"relation": "bond_endpoint", "subject_id": "ab", "object_id": "b", "expectation": "required"},
			{"relation": "scene", "subject_id": "scene", "object_id": "scene", "expectation": "normal_scale"},
		],
		[],
		tmp_path / "capture",
	)
	app.processEvents()
	# Restoring visibility makes the helper safe to use around a live test projection.
	assert root.isVisible()
	assert full_a.isVisible()
	assert not full_b.isVisible()
	assert bond.isVisible()
	assert not core_a.isVisible()
	assert not core_b.isVisible()
	value = json.loads(manifest_path.read_text(encoding="utf-8"))
	assert value["schema"] == RASTER_LAYER_MANIFEST_V2_SCHEMA
	assert value["capture_profile"] == {
		"profile_id": "test-normal-scale-v2",
		"source_rect": [-10.0, -10.0, 120.0, 80.0],
		"pixel_width": 240,
		"pixel_height": 160,
		"device_pixel_ratio": 1.0,
		"scene_evaluation": "presentation",
	}
	layers = load_raster_manifest_v2(manifest_path)
	assert layers.composite.shape == (160, 240)
	assert layers.capture_profile == _profile()
	assert layers.atoms["a"].core_mask.any()
	assert layers.atoms["a"].full_label_mask is not None
	assert layers.atoms["a"].full_label_mask.sum() > layers.atoms["a"].core_mask.sum()
	assert layers.atoms["b"].full_label_mask is not None
	# Full label b was hidden before capture; its isolated actual item still paints in its layer.
	assert layers.atoms["b"].full_label_mask.any()
	assert layers.bonds[0].footprint_mask.any()
	assert layers.composite.any()


# ============================================
def test_capture_rejects_incomplete_or_forged_source_mapping(tmp_path: pathlib.Path) -> None:
	"""Every declared layer identity must map to a real item from this projection scene."""
	app = QApplication.instance() or QApplication([])
	scene, root, full_a, _full_b, bond, core_a, _core_b = _scene_with_actual_chemical_root()
	foreign_scene = QGraphicsScene()
	foreign = _path_item(0.0, 0.0, 5.0, 5.0)
	foreign_scene.addItem(foreign)
	try:
		capture_scene(
			scene,
			_profile(),
			"forged_mapping",
			"<cdml/>",
			(root,),
			{"a": ("C", full_a, core_a), "b": ("O", foreign, core_a)},
			{"ab": ("a", "b", "normal", bond)},
			[], [], tmp_path / "capture",
		)
	except ValueError as error:
		assert "projection scene" in str(error)
	else:
		raise AssertionError("foreign full-label source item was accepted")
	app.processEvents()


# ============================================
def test_capture_restores_child_visibility_when_chemical_root_starts_hidden(
		tmp_path: pathlib.Path) -> None:
	"""Isolation cannot flatten child visibility when a caller hid its molecule root."""
	app = QApplication.instance() or QApplication([])
	scene, root, full_a, full_b, bond, core_a, core_b = _scene_with_actual_chemical_root()
	root.setVisible(False)
	full_a.setVisible(True)
	full_b.setVisible(False)
	bond.setVisible(True)
	capture_scene(
		scene, _profile(), "hidden_root", "<cdml/>", (root,),
		{"a": ("C", full_a, core_a), "b": ("O", full_b, core_b)},
		{"ab": ("a", "b", "normal", bond)}, [], [], tmp_path / "capture",
	)
	assert not root.isVisible()
	# Re-enabling the root exposes exactly its pre-capture children, proving their
	# own flags were retained rather than being overwritten by effective visibility.
	root.setVisible(True)
	assert full_a.isVisible()
	assert not full_b.isVisible()
	assert bond.isVisible()
	assert not core_a.isVisible()
	assert not core_b.isVisible()
	app.processEvents()
