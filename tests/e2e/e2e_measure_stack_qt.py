#!/usr/bin/env python3
"""Measure actual offscreen Qt projection pixels for the Rust-owned alignment corpus."""

# Standard Library
import pathlib
import sys
import argparse
import json
import hashlib
from collections import Counter


_REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
_LOCAL_QT_SOURCE_ROOT = _REPO_ROOT / "packages" / "ferrum-chem-qt.app"
_LOCAL_RUNTIME_ROOT = _REPO_ROOT / "build" / "runtime" / "python"


#============================================
def _use_local_built_runtime() -> None:
	"""Make direct E2E runs consume this checkout's approved staged runtime."""
	for required in (_LOCAL_QT_SOURCE_ROOT, _LOCAL_RUNTIME_ROOT):
		if not required.is_dir():
			raise RuntimeError(
				f"Qt measure stack requires {required}; run ./build.sh before this E2E",
			)
	managed = {str(_REPO_ROOT), str(_LOCAL_QT_SOURCE_ROOT), str(_LOCAL_RUNTIME_ROOT)}
	sys.path[:] = [
		str(_LOCAL_QT_SOURCE_ROOT), str(_LOCAL_RUNTIME_ROOT), str(_REPO_ROOT),
		*(entry for entry in sys.path if entry not in managed),
	]
	# PIP3 module
	import ferrum_chem
	actual = pathlib.Path(ferrum_chem.__file__).resolve()
	if actual.parent != _LOCAL_RUNTIME_ROOT.resolve():
		raise RuntimeError(
			f"Qt measure stack loaded ferrum_chem from {actual}, not the staged local runtime",
		)


_use_local_built_runtime()

# local E2E modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# Local modules
import e2e_atom_label_bond_alignment as alignment
import ferrum_qt.canvas.ferrum_telex
from measure_stack.contracts import CaptureProfile, load_raster_manifest_v2
from measure_stack.diagnostics import write_diagnostics
from measure_stack.measure import MeasurementPolicy, measure_scene, violations
from measure_stack.qt_scene_capture import capture_scene


_V2_FIXTURE_PATH = _REPO_ROOT / "measure_stack" / "fixtures" / "v2" / "fixtures.json"
_V2_FIXTURE_SCHEMA = "ferrum-measure-stack-fixtures-v2"
# Frozen only after a baseline capture has demonstrated that every V2 fixture
# reaches the real Qt consumer.  The explicit values make a renderer regression
# or an accidental relaxed predicate fail the baseline lane rather than bless
# its own current output.
_FROZEN_BASELINE_FAILURE_CATEGORIES = {
	"detached_endpoint": 8,
	"target_label_overlap": 7,
}
_FROZEN_EXPECTED_TYPED_REFUSALS: dict[str, str] = {}
_FROZEN_V2_CAPTURE_FIXTURE_COUNT = 12
_STYLE_BY_V2_STYLE = {
	"normal": "normal",
	"double": "double",
	"triple": "triple",
	"solid-wedge": "solid-wedge",
	"hashed-wedge": "hashed-wedge",
	"haworth-front-stroke": "haworth-front-stroke",
	"haworth-front-wedge": "haworth-front-wedge",
	"bold": "bold",
	"dashed": "dashed",
	"wavy": "wavy",
}


#============================================
def _v2_fixture_cases() -> tuple[tuple[dict[str, object], CaptureProfile], ...]:
	"""Load V2-owned Qt fixtures rather than inventing a parallel E2E mapping."""
	if not _V2_FIXTURE_PATH.is_file():
		raise ValueError("V2 fixture catalog is required for Qt measurement")
	value = json.loads(_V2_FIXTURE_PATH.read_text(encoding="utf-8"))
	if type(value) is not dict or set(value) != {"schema", "capture_profiles", "fixtures"}:
		raise ValueError("V2 fixture catalog has unknown or missing fields")
	if value["schema"] != _V2_FIXTURE_SCHEMA:
		raise ValueError("V2 fixture catalog has an unsupported schema")
	profiles = value["capture_profiles"]
	fixtures = value["fixtures"]
	if type(profiles) is not dict or type(fixtures) is not list or not fixtures:
		raise ValueError("V2 fixture catalog has invalid profiles or fixtures")
	result = []
	for fixture in fixtures:
		if type(fixture) is not dict:
			raise ValueError("V2 fixture must be an object")
		required = {
			"fixture_id", "fixture_cdml", "capture_profile_id", "graph",
			"expected_relations", "negative_cases", "synthetic_kind",
		}
		if set(fixture) != required:
			raise ValueError("V2 fixture has unknown or missing fields")
		if type(fixture["synthetic_kind"]) is not str:
			raise ValueError("V2 fixture synthetic_kind must be a string")
		# The catalog has both real-CDML projection fixtures and layer-only
		# adversarial inputs for the manifest runner.  Only the former belongs in
		# this Qt capture lane; the catalog, not a parallel ID list, decides this.
		if fixture["synthetic_kind"] != "connected":
			continue
		fixture_id = fixture["fixture_id"]
		cdml = fixture["fixture_cdml"]
		profile_id = fixture["capture_profile_id"]
		if type(fixture_id) is not str or not fixture_id or type(cdml) is not str or not cdml:
			raise ValueError("V2 fixture must name a nonempty ID and CDML payload")
		if type(profile_id) is not str or type(profiles.get(profile_id)) is not dict:
			raise ValueError("V2 fixture names an unknown capture profile")
		profile_value = profiles[profile_id]
		if set(profile_value) != {"source_rect", "pixel_width", "pixel_height", "device_pixel_ratio", "scene_evaluation"}:
			raise ValueError("V2 capture profile has unknown or missing fields")
		profile = CaptureProfile(
			profile_id, tuple(profile_value["source_rect"]), profile_value["pixel_width"],
			profile_value["pixel_height"], profile_value["device_pixel_ratio"], profile_value["scene_evaluation"],
		)
		graph = fixture["graph"]
		if type(graph) is not dict or set(graph) != {"atoms", "bonds"}:
			raise ValueError("V2 fixture graph has unknown or missing fields")
		atoms = graph["atoms"]
		bonds = graph["bonds"]
		if type(atoms) is not list or not atoms or type(bonds) is not list:
			raise ValueError("V2 fixture graph has invalid atoms or bonds")
		case_atoms = []
		for atom in atoms:
			if type(atom) is not dict or set(atom) != {"atom_id", "element"}:
				raise ValueError("V2 fixture atom has unknown or missing fields")
			case_atoms.append({"source_id": atom["atom_id"], "core_run": atom["element"]})
		case_bonds = []
		for bond in bonds:
			if type(bond) is not dict or set(bond) != {"bond_id", "start_atom_id", "end_atom_id", "style"}:
				raise ValueError("V2 fixture bond has unknown or missing fields")
			style = bond["style"]
			if style not in _STYLE_BY_V2_STYLE:
				raise ValueError("V2 fixture bond style has no measurement predicate")
			case_bonds.append({"source_id": bond["bond_id"], "style": style})
		result.append(({
			"name": fixture_id,
			"cdml": cdml,
			"atoms": case_atoms,
			"bonds": case_bonds,
			"expected_relations": fixture["expected_relations"],
			"negative_cases": fixture["negative_cases"],
		}, profile))
	return tuple(result)


#============================================
def _failure_category(message: str) -> str:
	"""Collapse fixture-specific violations into stable renderer failure categories."""
	if message.startswith("capture failure:"):
		return "capture_failure"
	if ": measurement is nonfinite (" in message:
		return "nonfinite_endpoint_measurement"
	if ": " not in message:
		return "unclassified"
	detail = message.split(": ", 1)[1]
	return {
		"bond is visibly detached from target character": "detached_endpoint",
		"bond overlaps or touches target label": "target_label_overlap",
		"bond misses target-character centerline": "centerline_miss",
		"final footprint style topology is invalid": "style_topology",
		"scene occupancy is outside fixed normal-scale policy": "scene_occupancy",
		"scene is visibly under-framed": "under_framed_scene",
		"declared bond endpoint misses its target-label neighborhood": "missing_endpoint_connection",
		"declared molecule has orphaned atom cores": "orphaned_atom_core",
	}.get(detail, detail.replace(" ", "_"))


#============================================
#============================================
def _core_item(content: object, telex: object) -> PySide6.QtWidgets.QGraphicsPathItem:
	"""Create a hidden test-only Qt core glyph path from the issued text run identity."""
	label = content.label
	core_run = label.text.runs[label.core_element_run_index]
	verified_telex = ferrum_qt.canvas.ferrum_telex.from_verified_resource(telex)
	path = alignment._label_path(content, verified_telex, (core_run,))
	item = PySide6.QtWidgets.QGraphicsPathItem(path)
	item.setPen(PySide6.QtCore.Qt.PenStyle.NoPen)
	item.setBrush(PySide6.QtGui.QBrush(PySide6.QtGui.QColor("black")))
	item.setVisible(False)
	return item


#============================================
def _fixture_layers(case: dict[str, object], observation: object,
		projection: object, telex: object) -> tuple[
		dict[str, tuple[str, object, object]], dict[str, tuple[str, str, str, object]],
		list[PySide6.QtWidgets.QGraphicsPathItem],
	]:
	"""Join fixture source identities to actual Qt item roots without geometry metadata."""
	positions = alignment._source_positions(case["cdml"])
	atom_batches = alignment._atom_batches_by_source_id(observation, positions)
	atom_rows = {row["source_id"]: row for row in case["atoms"]}
	if set(atom_batches) != set(atom_rows):
		raise ValueError("fixture atom identities differ from observed Qt targets")
	actual_items = {
		target.document_object_id: item for item, target in projection.item_targets.items()
	}
	atom_items: dict[str, tuple[str, object, object]] = {}
	temporary_cores = []
	for source_id, batch in atom_batches.items():
		expected_core = atom_rows[source_id]["core_run"]
		label = batch.content.label
		core = label.text.runs[label.core_element_run_index]
		if core.text != expected_core:
			raise ValueError("fixture core glyph identity differs from issued Qt label")
		core_item = _core_item(batch.content, telex)
		projection.scene.addItem(core_item)
		temporary_cores.append(core_item)
		try:
			full_label_item = actual_items[batch.target.document_object_id]
		except KeyError as error:
			raise ValueError("fixture atom has no actual Qt consumer label item") from error
		atom_items[source_id] = (expected_core, full_label_item, core_item)
	projection_bonds = {
		bond.source_id: bond for bond in observation.document.projection.molecules[0].bonds
	}
	bond_items = {}
	for row in case["bonds"]:
		source_id = row["source_id"]
		bond = projection_bonds.get(source_id)
		if bond is None:
			raise ValueError("fixture bond identity is absent from observation")
		try:
			item = actual_items[bond.document_object_id]
		except KeyError as error:
			raise ValueError("fixture bond has no Qt consumer item") from error
		style = _STYLE_BY_V2_STYLE.get(row["style"])
		if style is None:
			raise ValueError("fixture bond style has no measurement predicate")
		start = next(
			source_id for source_id, batch in atom_batches.items()
			if batch.target.document_object_id == bond.start.document_object_id
		)
		end = next(
			source_id for source_id, batch in atom_batches.items()
			if batch.target.document_object_id == bond.end.document_object_id
		)
		bond_items[source_id] = (start, end, style, item)
	if set(bond_items) != {row["source_id"] for row in case["bonds"]}:
		raise ValueError("fixture bonds do not exactly match Qt consumer items")
	return atom_items, bond_items, temporary_cores


#============================================
def _measure_renderable_case(case: dict[str, object], capture_profile: CaptureProfile,
		output_root: pathlib.Path) -> list[str]:
	"""Capture and independently measure one real installed Qt render projection."""
	observation, projection, telex = alignment._projection_for(case)
	temporary_cores: list[PySide6.QtWidgets.QGraphicsPathItem] = []
	try:
		atoms, bonds, temporary_cores = _fixture_layers(case, observation, projection, telex)
		manifest = capture_scene(
			projection.scene, capture_profile, case["name"], case["cdml"],
			projection.molecule_roots, atoms, bonds, case["expected_relations"],
			case["negative_cases"], output_root / case["name"],
		)
		scene_layers = load_raster_manifest_v2(manifest)
		report = measure_scene(scene_layers)
		report["violations"] = violations(report, MeasurementPolicy())
		write_diagnostics(
			scene_layers, report, output_root / case["name"] / "measurement",
		)
		return report["violations"]
	finally:
		for item in temporary_cores:
			projection.scene.removeItem(item)
		projection.dispose()


#============================================
def _write_summary(output_root: pathlib.Path, cases: tuple[tuple[dict[str, object], CaptureProfile], ...],
		failed: dict[str, list[str]], expected_refusals: dict[str, str], baseline: bool) -> dict[str, object]:
	"""Publish a classification that separates capture health from renderer evidence."""
	categories = Counter(
		_failure_category(message)
		for messages in failed.values() for message in messages
	)
	capture_failures = {
		fixture_id: messages for fixture_id, messages in failed.items()
		if any(_failure_category(message) == "capture_failure" for message in messages)
	}
	summary = {
		"schema": "ferrum-measure-stack-qt-run-summary-v2",
		"mode": "baseline" if baseline else "strict",
		"fixture_source": "v2",
		"fixture_catalog_sha256": hashlib.sha256(_V2_FIXTURE_PATH.read_bytes()).hexdigest(),
		"fixture_count": len(cases),
		"capture_profile_ids": sorted({profile.profile_id for _case, profile in cases}),
		"capture_health": {"healthy": not capture_failures, "failures": capture_failures},
		"expected_typed_refusals": expected_refusals,
		"renderer_failure_evidence": {
			"by_fixture": failed,
			"by_category": dict(sorted(categories.items())),
			"violation_count": sum(categories.values()),
		},
		"frozen_expected_failure_categories": _FROZEN_BASELINE_FAILURE_CATEGORIES,
		"frozen_expected_typed_refusals": _FROZEN_EXPECTED_TYPED_REFUSALS,
		"frozen_v2_capture_fixture_count": _FROZEN_V2_CAPTURE_FIXTURE_COUNT,
	}
	(output_root / "run_summary.json").write_text(
		json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8",
	)
	if baseline:
		(output_root / "baseline_summary.json").write_text(
			json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8",
		)
	return summary


#============================================
def main() -> int:
	"""Run all renderable atom-label fixtures through the actual Qt pixel consumer."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"--output-dir", type=pathlib.Path, required=True,
		help="empty directory to retain every V2 raster layer, report, overlay, and summary",
	)
	mode = parser.add_mutually_exclusive_group()
	mode.add_argument(
		"--baseline", action="store_true",
		help="exit zero only when Qt capture is healthy and frozen expected renderer failures match",
	)
	mode.add_argument(
		"--fail-on-violation", action="store_true",
		help="strict-red: exit nonzero for every visual-quality violation",
	)
	arguments = parser.parse_args()
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	failed = {}
	expected_refusals = {}
	output_root = arguments.output_dir.resolve()
	if output_root.exists() and any(output_root.iterdir()):
		raise ValueError("Qt measurement output directory must be empty")
	output_root.mkdir(parents=True, exist_ok=True)
	fixture_cases = _v2_fixture_cases()
	for case, profile in fixture_cases:
		try:
			case_failures = _measure_renderable_case(case, profile, output_root)
		except (AssertionError, OSError, ValueError) as error:
			case_failures = [f"capture failure: {error}"]
		if case_failures:
			failed[case["name"]] = case_failures
	summary = _write_summary(output_root, fixture_cases, failed, expected_refusals, arguments.baseline)
	app.processEvents()
	if arguments.baseline:
		actual = summary["renderer_failure_evidence"]["by_category"]
		if not summary["capture_health"]["healthy"]:
			raise AssertionError(json.dumps({"capture_health": summary["capture_health"], "output": str(output_root)}, sort_keys=True))
		if summary["fixture_source"] != "v2" or summary["fixture_count"] != _FROZEN_V2_CAPTURE_FIXTURE_COUNT:
			raise AssertionError(json.dumps({"fixture_provenance_mismatch": {"source": summary["fixture_source"], "count": summary["fixture_count"]}, "output": str(output_root)}, sort_keys=True))
		if actual != _FROZEN_BASELINE_FAILURE_CATEGORIES:
			raise AssertionError(json.dumps({"baseline_mismatch": {"expected": _FROZEN_BASELINE_FAILURE_CATEGORIES, "actual": actual}, "output": str(output_root)}, sort_keys=True))
		if expected_refusals != _FROZEN_EXPECTED_TYPED_REFUSALS:
			raise AssertionError(json.dumps({"typed_refusal_mismatch": {"expected": _FROZEN_EXPECTED_TYPED_REFUSALS, "actual": expected_refusals}, "output": str(output_root)}, sort_keys=True))
		print(json.dumps({"status": "ok", "mode": "baseline", "fixture_count": len(fixture_cases)}, sort_keys=True))
		return 0
	if failed:
		raise AssertionError(json.dumps({"failures": failed, "output": str(output_root)}, sort_keys=True))
	print(json.dumps({"status": "ok", "mode": "strict", "fixture_count": len(fixture_cases)}))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except (AssertionError, OSError, ValueError) as exc:
		print(f"e2e_measure_stack_qt: {exc}", file=sys.stderr)
		raise SystemExit(1)
