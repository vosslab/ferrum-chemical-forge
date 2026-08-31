#!/usr/bin/env python3
"""Materialize and validate Ferrum's immutable V2 visual fixture baseline."""

# Standard Library
import argparse
import hashlib
import json
import pathlib

# PIP3 modules
import cv2
import numpy

# Local modules
from measure_stack.contracts import RASTER_LAYER_MANIFEST_V2_SCHEMA, load_raster_manifest_v2
from measure_stack.diagnostics import write_diagnostics
from measure_stack.measure import MeasurementPolicy, measure_scene, violations


FIXTURE_SCHEMA = "ferrum-measure-stack-fixtures-v2"
FIXTURE_PATH = pathlib.Path(__file__).parent / "fixtures" / "v2" / "fixtures.json"


# ============================================
def _read_fixtures(path: pathlib.Path = FIXTURE_PATH) -> dict[str, object]:
	"""Read the authored, closed fixture source used for deterministic V2 evidence."""
	value = json.loads(path.read_text(encoding="utf-8"))
	if type(value) is not dict or set(value) != {"schema", "capture_profiles", "fixtures"}:
		raise ValueError("fixture source has unknown or missing fields")
	if value["schema"] != FIXTURE_SCHEMA:
		raise ValueError("fixture source has an unsupported schema")
	if type(value["capture_profiles"]) is not dict or type(value["fixtures"]) is not list:
		raise ValueError("fixture source has invalid profiles or fixtures")
	return value


# ============================================
def _sha256_bytes(value: bytes) -> str:
	"""Return a portable content digest for immutable fixture identity."""
	return hashlib.sha256(value).hexdigest()


# ============================================
def _write_png(path: pathlib.Path, mask: numpy.ndarray) -> str:
	"""Publish one transparent binary final-ink layer and return its exact hash."""
	image = numpy.zeros((*mask.shape, 4), dtype=numpy.uint8)
	image[mask] = (0, 0, 0, 255)
	if not cv2.imwrite(str(path), image):
		raise OSError(f"could not write fixture image: {path}")
	return _sha256_bytes(path.read_bytes())


# ============================================
def _atom_center(shape: tuple[int, int], index: int, count: int) -> tuple[int, int]:
	"""Place fixture atoms on a fixed normal-scale ellipse, never fitted to ink."""
	if count == 2:
		return (
			int(shape[0] * (0.25 if index == 0 else 0.75)),
			int(shape[1] * (0.25 if index == 0 else 0.75)),
		)
	angle = -numpy.pi / 2.0 + 2.0 * numpy.pi * index / count
	return (
		int(round(shape[0] / 2.0 + shape[0] * 0.34 * numpy.sin(angle))),
		int(round(shape[1] / 2.0 + shape[1] * 0.38 * numpy.cos(angle))),
	)


# ============================================
def _atom_mask(shape: tuple[int, int], index: int, count: int) -> numpy.ndarray:
	"""Create a deterministic label block, independent of any rendered current ink."""
	mask = numpy.zeros(shape, dtype=bool)
	row, column = _atom_center(shape, index, count)
	row_radius, column_radius = (17, 13) if count == 2 else (16, 11) if count == 3 else (14, 9)
	mask[row - row_radius:row + row_radius + 1, column - column_radius:column + column_radius + 1] = True
	return mask


# ============================================
def _bond_mask(
		shape: tuple[int, int], start_core: numpy.ndarray, end_core: numpy.ndarray,
		kind: str, style: str) -> numpy.ndarray:
	"""Create final-style ink with a fixed positive endpoint clearance."""
	mask = numpy.zeros(shape, dtype=bool)
	start_points = numpy.column_stack(numpy.where(start_core)).astype(float)
	end_points = numpy.column_stack(numpy.where(end_core)).astype(float)
	start_center = start_points.mean(axis=0)
	end_center = end_points.mean(axis=0)
	unit = end_center - start_center
	unit /= numpy.linalg.norm(unit)
	start_edge = max((point - start_center) @ unit for point in start_points)
	end_edge = max((point - end_center) @ -unit for point in end_points)
	# A fixed normal-scale clearance keeps ordinary ink near the target while a
	# thick bold stroke gets enough axial room for its footprint radius.
	clearance = 6.0 if style == "bold" else 4.0 if style in {"normal", "haworth-front-stroke"} else 3.0
	start = start_center + unit * (start_edge + clearance)
	end = end_center - unit * (end_edge + clearance)
	if kind in {"detached", "orphan"}:
		end -= unit * 24.0
	normal = numpy.array((-unit[1], unit[0]))
	if kind == "target_overlap":
		start -= unit * 7.0
	if kind == "centerline_miss":
		start += normal * 6.0
		end += normal * 6.0
	def line(left: numpy.ndarray, right: numpy.ndarray, width: int) -> None:
		cv2.line(mask, tuple(numpy.rint(left[::-1]).astype(int)), tuple(numpy.rint(right[::-1]).astype(int)), True, width, cv2.LINE_8)
	if kind == "style_topology":
		line(start, end, 3)
		return mask
	if style == "double":
		line(start - normal * 4, end - normal * 4, 1)
		line(start + normal * 4, end + normal * 4, 1)
	elif style == "triple":
		for offset in (-3, 0, 3):
			line(start + normal * offset, end + normal * offset, 1)
	elif style in {"dashed", "hashed-wedge"}:
		if style == "dashed":
			for fraction in numpy.arange(0.0, 1.0, 0.16):
				line(start + (end - start) * fraction, start + (end - start) * min(fraction + 0.08, 1.0), 3)
		else:
			for fraction in numpy.linspace(0.0, 1.0, 7):
				point = start + (end - start) * fraction
				half_width = max(1, int(round(fraction * 8)))
				line(point - normal * half_width, point + normal * half_width, 1)
	elif style in {"solid-wedge", "haworth-front-wedge"}:
		polygon = numpy.rint(numpy.array((start - normal, start + normal, end + normal * 8, end - normal * 8))[:, ::-1]).astype(numpy.int32)
		cv2.fillConvexPoly(mask, polygon, True, cv2.LINE_8)
	elif style == "wavy":
		points = numpy.array([start + (end - start) * fraction + normal * 3.0 * numpy.sin(fraction * 8.0 * numpy.pi) for fraction in numpy.linspace(0.0, 1.0, 80)])
		cv2.polylines(mask, [numpy.rint(points[:, ::-1]).astype(numpy.int32)], False, True, 3, cv2.LINE_8)
	elif style == "bold":
		line(start, end, 7)
	else:
		line(start, end, 3 if style == "haworth-front-stroke" else 3)
	return mask


# ============================================
def _materialize_fixture(fixture: dict[str, object], profiles: dict[str, object], output_root: pathlib.Path) -> pathlib.Path:
	"""Build a bounded V2 manifest from declared fixture identity and final-ink layers."""
	profile_id = fixture["capture_profile_id"]
	profile = profiles[profile_id]
	if type(profile_id) is not str or type(profile) is not dict:
		raise ValueError("fixture names an unknown capture profile")
	shape = (profile["pixel_height"], profile["pixel_width"])
	fixture_directory = output_root / fixture["fixture_id"]
	fixture_directory.mkdir(parents=True, exist_ok=False)
	graph = fixture["graph"]
	atoms = graph["atoms"]
	bonds = graph["bonds"]
	atom_layers = []
	atom_masks: dict[str, numpy.ndarray] = {}
	for index, atom in enumerate(atoms):
		atom_id = atom["atom_id"]
		core = _atom_mask(shape, index, len(atoms))
		full = core.copy()
		full[:, :] |= core
		core_path = fixture_directory / f"core_{atom_id}.png"
		full_path = fixture_directory / f"label_{atom_id}.png"
		atom_layers.append({
			"atom_id": atom_id,
			"core_glyph_layer": {"relative_path": core_path.name, "sha256": _write_png(core_path, core)},
			"full_label_layer": {"relative_path": full_path.name, "sha256": _write_png(full_path, full)},
		})
		atom_masks[atom_id] = core
	bond_layers = []
	bond_masks = []
	for index, bond in enumerate(bonds):
		mask = _bond_mask(
			shape, atom_masks[bond["start_atom_id"]], atom_masks[bond["end_atom_id"]],
			fixture["synthetic_kind"], bond["style"],
		)
		if fixture["synthetic_kind"] == "third_label_collision" and index == 0:
			mask |= atom_masks["n1"]
		path = fixture_directory / f"bond_{bond['bond_id']}.png"
		bond_layers.append({"bond_id": bond["bond_id"], "final_bond_layer": {"relative_path": path.name, "sha256": _write_png(path, mask)}})
		bond_masks.append(mask)
	composite = numpy.zeros(shape, dtype=bool)
	for mask in atom_masks.values():
		composite |= mask
	for mask in bond_masks:
		composite |= mask
	if fixture["synthetic_kind"] == "cropped":
		# Deliberate composite-only edge ink proves the fixed-profile crop rule.
		composite[0:2, shape[1] // 2 - 2:shape[1] // 2 + 3] = True
	composite_path = fixture_directory / "composite.png"
	manifest = {
		"schema": RASTER_LAYER_MANIFEST_V2_SCHEMA,
		"fixture_id": fixture["fixture_id"],
		"fixture_cdml_sha256": _sha256_bytes(fixture["fixture_cdml"].encode("utf-8")),
		"capture_profile": {"profile_id": profile_id, **profile},
		"graph": graph,
		"composite_layer": {"relative_path": composite_path.name, "sha256": _write_png(composite_path, composite)},
		"atom_layers": atom_layers,
		"bond_layers": bond_layers,
		"expected_relations": fixture["expected_relations"],
		"negative_cases": fixture["negative_cases"],
	}
	path = fixture_directory / "raster_layer_manifest_v2.json"
	path.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n", encoding="utf-8")
	return path


# ============================================
def _expected_categories(fixture: dict[str, object]) -> list[str]:
	"""Map deliberate negative relations to stable measurement failure categories."""
	mapping = {
		"must_reject_detached_gap": "bond is visibly detached from target character",
		"must_reject_target_overlap": "bond collides with target full label",
		"must_reject_centerline_miss": "bond misses target-character centerline",
		"must_reject_style_topology": "final footprint style topology is invalid",
		"must_reject_cropped_scene": "composition: foreground is cropped against viewport",
		"must_reject_orphaned_atom": "composition: declared molecule has orphaned atom cores",
		"must_reject_collision": "bond collides with non-endpoint full label",
	}
	result: list[str] = []
	for relation in fixture["negative_cases"]:
		expectation = relation["expectation"]
		if expectation not in mapping:
			raise ValueError(f"fixture has unsupported negative expectation: {expectation}")
		result.append(mapping[expectation])
	return result


# ============================================
def run_fixture_baseline(output_root: pathlib.Path) -> dict[str, object]:
	"""Materialize, measure, diagnose, and enforce every fixture expectation."""
	if output_root.exists():
		if any(output_root.iterdir()):
			raise ValueError("baseline output directory must be empty")
	else:
		output_root.mkdir(parents=True)
	fixtures = _read_fixtures()
	profiles = fixtures["capture_profiles"]
	results = []
	for fixture in fixtures["fixtures"]:
		manifest_path = _materialize_fixture(fixture, profiles, output_root)
		scene = load_raster_manifest_v2(manifest_path)
		report = measure_scene(scene)
		policy_violations = violations(report, MeasurementPolicy())
		report["violations"] = policy_violations
		write_diagnostics(scene, report, manifest_path.parent / "measurement")
		expected = _expected_categories(fixture)
		missing = [category for category in expected if not any(category in violation for violation in policy_violations)]
		unexpected = policy_violations if not expected else []
		results.append({
			"fixture_id": scene.fixture_id,
			"fixture_cdml_sha256": scene.fixture_cdml_sha256,
			"capture_profile_id": scene.capture_profile.profile_id,
			"atom_count": len(scene.atoms),
			"bond_count": len(scene.bonds),
			"negative_case_count": len(scene.negative_cases),
			"manifest": str(manifest_path.relative_to(output_root)),
			"measurement": str((manifest_path.parent / "measurement" / "measurement_report.json").relative_to(output_root)),
			"policy_violations": policy_violations,
			"expected_violation_categories": expected,
			"missing_expected_violation_categories": missing,
			"unexpected_violations": unexpected,
			"accepted": not missing and not unexpected,
		})
	acceptance_failures = [
		f"{row['fixture_id']}: expected measurement categories were absent" for row in results if row["missing_expected_violation_categories"]
	] + [
		f"{row['fixture_id']}: approved fixture has policy violations" for row in results if row["unexpected_violations"]
	]
	summary = {"schema": "ferrum-measure-stack-baseline-summary-v2", "fixtures": results, "fixture_count": len(results), "acceptance_failures": acceptance_failures, "accepted": not acceptance_failures}
	(output_root / "baseline_summary.json").write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")
	return summary


# ============================================
def main() -> int:
	"""Run the explicit deterministic V2 contract baseline lane."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--output-dir", required=True, type=pathlib.Path)
	parser.add_argument("--fail-on-violation", action="store_true")
	arguments = parser.parse_args()
	try:
		summary = run_fixture_baseline(arguments.output_dir)
	except (OSError, ValueError, json.JSONDecodeError) as error:
		print(json.dumps({"status": "error", "error": str(error)}, sort_keys=True))
		return 1
	print(json.dumps({"status": "ok" if summary["accepted"] else "violation", "fixture_count": summary["fixture_count"], "violations": len(summary["acceptance_failures"])}, sort_keys=True))
	return 1 if arguments.fail_on_violation and not summary["accepted"] else 0


if __name__ == "__main__":
	raise SystemExit(main())
