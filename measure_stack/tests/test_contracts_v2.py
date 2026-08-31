"""Focused acceptance and hostile-input tests for RasterLayerManifestV2."""

# Standard Library
import hashlib
import json
import pathlib
from collections.abc import Callable

# PIP3 modules
import cv2
import numpy
import pytest

# Local modules
from measure_stack.contracts import (
	CaptureProfile,
	RASTER_LAYER_MANIFEST_V2_SCHEMA,
	load_raster_manifest_v2,
)


# ============================================
def _manifest_path(tmp_path: pathlib.Path) -> pathlib.Path:
	"""Write one minimal inline V2 manifest and its isolated PNG layers."""
	tmp_path.mkdir(parents=True, exist_ok=True)
	composite = numpy.full((2, 2), 255, dtype=numpy.uint8)
	composite[0, 0] = 0
	mask = numpy.zeros((2, 2), dtype=numpy.uint8)
	mask[0, 0] = 255
	layers = {
		"composite.png": composite,
		"core.png": mask,
		"full.png": mask,
	}
	for name, pixels in layers.items():
		if not cv2.imwrite(str(tmp_path / name), pixels):
			raise RuntimeError(f"could not write inline test layer {name}")

	def layer(name: str) -> dict[str, str]:
		path = tmp_path / name
		return {
			"relative_path": name,
			"sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
		}

	manifest = {
		"schema": RASTER_LAYER_MANIFEST_V2_SCHEMA,
		"fixture_id": "inline_fixture",
		"fixture_cdml_sha256": "0" * 64,
		"capture_profile": {
			"profile_id": "inline_profile",
			"source_rect": [0.0, 0.0, 2.0, 2.0],
			"pixel_width": 2,
			"pixel_height": 2,
			"device_pixel_ratio": 1.0,
			"scene_evaluation": "presentation",
		},
		"graph": {"atoms": [{"atom_id": "a", "element": "C"}], "bonds": []},
		"composite_layer": layer("composite.png"),
		"atom_layers": [{
			"atom_id": "a",
			"core_glyph_layer": layer("core.png"),
			"full_label_layer": layer("full.png"),
		}],
		"bond_layers": [],
		"expected_relations": [],
		"negative_cases": [],
	}
	path = tmp_path / "raster_layer_manifest_v2.json"
	path.write_text(json.dumps(manifest), encoding="utf-8")
	return path


# ============================================
def _rewrite(path: pathlib.Path, change: Callable[[dict[str, object]], None]) -> None:
	"""Apply one intentional hostile-input mutation to a generated V2 manifest."""
	value = json.loads(path.read_text(encoding="utf-8"))
	change(value)
	path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


# ============================================
def test_v2_accepts_complete_non_circular_capture_contract(tmp_path: pathlib.Path) -> None:
	"""A complete inline manifest retains its fixed profile and full-label mask."""
	scene = load_raster_manifest_v2(_manifest_path(tmp_path))
	assert scene.capture_profile == CaptureProfile(
		"inline_profile", (0.0, 0.0, 2.0, 2.0), 2, 2, 1.0, "presentation",
	)
	assert scene.atoms["a"].full_label_mask.any()


# ============================================
def test_v2_rejects_hash_mismatch_and_path_escape(tmp_path: pathlib.Path) -> None:
	"""A producer cannot substitute a layer or escape fixture ownership."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["composite_layer"].update({"sha256": "0" * 64}))
	with pytest.raises(ValueError, match="SHA-256"):
		load_raster_manifest_v2(manifest)
	manifest = _manifest_path(tmp_path / "paths")
	_rewrite(manifest, lambda value: value["composite_layer"].update({"relative_path": "../composite.png"}))
	with pytest.raises(ValueError, match="stay below"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_rejects_every_predecessor_schema(tmp_path: pathlib.Path) -> None:
	"""No reader fallback can turn incomplete V1 evidence into V2 acceptance."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value.update({"schema": "ferrum-glyph-bond-raster-layers-v1"}))
	with pytest.raises(ValueError, match="unsupported schema"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_rejects_auto_fit_or_missing_fixed_profile_dimensions(tmp_path: pathlib.Path) -> None:
	"""V2 requires an authored output size rather than current ink extents."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["capture_profile"].pop("pixel_width"))
	with pytest.raises(ValueError, match="unknown or missing"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_requires_explicit_capture_evaluation_semantics(tmp_path: pathlib.Path) -> None:
	"""A raster diagnostic cannot silently inherit presentation viewport policy."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["capture_profile"].pop("scene_evaluation"))
	with pytest.raises(ValueError, match="unknown or missing"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_rejects_missing_full_label_layer(tmp_path: pathlib.Path) -> None:
	"""V2 requires full-label collision evidence in addition to target core glyph ink."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["atom_layers"][0].pop("full_label_layer"))
	with pytest.raises(ValueError, match="unknown or missing"):
		load_raster_manifest_v2(manifest)
