"""Focused acceptance and hostile-input tests for RasterLayerManifestV2."""

# Standard Library
import json
import pathlib

# PIP3 modules
import pytest

# Local modules
from measure_stack.contracts import RASTER_LAYER_MANIFEST_V2_SCHEMA, load_raster_manifest_v2
from measure_stack.runner import run_fixture_baseline


# ============================================
def _manifest_path(tmp_path: pathlib.Path) -> pathlib.Path:
	"""Materialize the immutable normal fixture for one isolated contract test."""
	run_fixture_baseline(tmp_path)
	return tmp_path / "chlorine_normal_horizontal_mask" / "raster_layer_manifest_v2.json"


# ============================================
def _rewrite(path: pathlib.Path, change: callable) -> None:
	"""Apply one intentional hostile-input mutation to a generated V2 manifest."""
	value = json.loads(path.read_text(encoding="utf-8"))
	change(value)
	path.write_text(json.dumps(value, sort_keys=True), encoding="utf-8")


# ============================================
def test_v2_accepts_complete_non_circular_capture_contract(tmp_path: pathlib.Path) -> None:
	"""The approved fixture supplies fixed profile, graph identity, and all mask roles."""
	scene = load_raster_manifest_v2(_manifest_path(tmp_path))
	assert scene.schema == RASTER_LAYER_MANIFEST_V2_SCHEMA
	assert scene.capture_profile is not None
	assert scene.capture_profile.scene_evaluation == "presentation"
	assert all(atom.full_label_mask is not None for atom in scene.atoms.values())
	assert scene.fixture_cdml_sha256 is not None


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
	"""V2 binds output size to an authored profile instead of current ink extents."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["capture_profile"].pop("pixel_width"))
	with pytest.raises(ValueError, match="unknown or missing"):
		load_raster_manifest_v2(manifest)
	manifest = _manifest_path(tmp_path / "dimensions")
	_rewrite(manifest, lambda value: value["capture_profile"].update({"pixel_width": 479}))
	with pytest.raises(ValueError, match="dimensions must match"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_requires_explicit_capture_evaluation_semantics(tmp_path: pathlib.Path) -> None:
	"""A raster diagnostic cannot silently inherit presentation viewport policy."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["capture_profile"].pop("scene_evaluation"))
	with pytest.raises(ValueError, match="unknown or missing"):
		load_raster_manifest_v2(manifest)
	manifest = _manifest_path(tmp_path / "invalid-semantics")
	_rewrite(manifest, lambda value: value["capture_profile"].update({"scene_evaluation": "auto_fit"}))
	with pytest.raises(ValueError, match="scene_evaluation"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_rejects_missing_full_label_layer(tmp_path: pathlib.Path) -> None:
	"""V2 requires full-label collision evidence in addition to target core glyph ink."""
	manifest = _manifest_path(tmp_path)
	_rewrite(manifest, lambda value: value["atom_layers"][0].pop("full_label_layer"))
	with pytest.raises(ValueError, match="unknown or missing"):
		load_raster_manifest_v2(manifest)


# ============================================
def test_v2_baseline_runner_writes_deterministic_summary(tmp_path: pathlib.Path) -> None:
	"""The explicit aggregate lane materializes every authored visual relation fixture."""
	summary = run_fixture_baseline(tmp_path)
	saved = json.loads((tmp_path / "baseline_summary.json").read_text(encoding="utf-8"))
	assert summary == saved
	assert saved["schema"] == "ferrum-measure-stack-baseline-summary-v2"
	assert {row["fixture_id"] for row in saved["fixtures"]} >= {
		"chlorine_normal_horizontal_mask", "haworth_front_stroke_and_wedge",
		"negative_detached_endpoint_v1", "negative_orphan_atom_v1",
	}
