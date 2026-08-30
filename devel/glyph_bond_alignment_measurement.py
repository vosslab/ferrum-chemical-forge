#!/usr/bin/env python3
"""Independent, pixel-only glyph/bond alignment measurements.

This developer library intentionally accepts raster layers plus fixture graph
identity.  It must not be given render-plan bounds, attachment axes, clearances,
or clipped endpoints: doing so would turn an independent measurement into a
second assertion of renderer-issued values.
"""

# Standard Library
import argparse
import dataclasses
import json
import math
import pathlib
import re
import tempfile
from collections.abc import Mapping, Sequence

# PIP3 modules
import cv2
import numpy


SCHEMA = "ferrum-glyph-bond-alignment-measurement-v1"
_MASK_SUFFIXES = frozenset((".png", ".bmp", ".tif", ".tiff"))
_RASTER_MANIFEST_SCHEMA = "ferrum-glyph-bond-raster-layers-v1"
_IDENTITY_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$")
_MAX_MANIFEST_BYTES = 1_048_576
_MAX_RASTER_FILE_BYTES = 67_108_864
_MAX_TOTAL_RASTER_FILE_BYTES = 536_870_912
_MAX_RASTER_PIXELS = 33_554_432
_MAX_LAYER_COUNT = 256
_MAX_BOND_COUNT = 128


@dataclasses.dataclass(frozen=True)
class BondIdentity:
	"""Fixture-owned graph identity for one independently measured bond."""

	bond_id: str
	start_atom: str
	end_atom: str
	style: str


@dataclasses.dataclass(frozen=True)
class MeasurementThresholds:
	"""Developer gate thresholds expressed in raster pixels at the selected scale."""

	max_centerline_error: float = 24.0
	min_signed_intended_label_gap: float = 1.0
	max_non_endpoint_collision_pixels: int = 0
	min_final_footprint_coverage: float = 0.995


def _read_image(path: pathlib.Path) -> numpy.ndarray:
	"""Read one bounded raster layer without allowing color semantics to leak in."""
	if path.suffix.lower() not in _MASK_SUFFIXES:
		raise ValueError(f"unsupported raster suffix: {path}")
	if not path.is_file():
		raise ValueError(f"raster layer is not a regular file: {path}")
	if path.stat().st_size > _MAX_RASTER_FILE_BYTES:
		raise ValueError(f"raster layer exceeds {_MAX_RASTER_FILE_BYTES} byte limit: {path}")
	image = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
	if image is None:
		raise ValueError(f"could not read raster layer: {path}")
	if image.ndim not in {2, 3} or image.shape[0] * image.shape[1] > _MAX_RASTER_PIXELS:
		raise ValueError(f"raster layer has unsupported dimensions: {path}")
	if image.ndim == 3 and image.shape[2] not in {1, 2, 3, 4}:
		raise ValueError(f"raster layer has unsupported channel count: {path}")
	return image


def _read_mask(path: pathlib.Path) -> numpy.ndarray:
	"""Read one image as a boolean ink mask without color semantics."""
	image = _read_image(path)
	if image.ndim == 2:
		return image != 0
	if image.shape[2] == 4:
		return image[:, :, 3] != 0
	return numpy.any(image != 0, axis=2)


def _foreground_mask(path: pathlib.Path) -> numpy.ndarray:
	"""Read composite foreground; alpha is authoritative when the image has it."""
	image = _read_image(path)
	if image.ndim == 2:
		return image != 255
	if image.shape[2] == 4:
		return image[:, :, 3] != 0
	return numpy.any(image != 255, axis=2)


def _validate_same_shape(layers: Sequence[numpy.ndarray]) -> tuple[int, int]:
	"""Reject mismatched raster layers before comparing their pixels."""
	if not layers or len(layers) > _MAX_LAYER_COUNT:
		raise ValueError("measurement requires raster layers")
	shape = layers[0].shape
	if len(shape) != 2 or not all(layer.shape == shape for layer in layers):
		raise ValueError("all raster layers must have one identical two-dimensional shape")
	if shape[0] * shape[1] > _MAX_RASTER_PIXELS:
		raise ValueError("raster layer exceeds pixel limit")
	return shape


def _core_center(mask: numpy.ndarray, identity: str) -> numpy.ndarray:
	"""Return the actual glyph-ink box center, refusing absent target ink."""
	points = numpy.argwhere(mask)
	if len(points) == 0:
		raise ValueError(f"target core glyph mask is empty: {identity}")
	minimum = points.min(axis=0)
	maximum = points.max(axis=0)
	return (minimum.astype(numpy.float64) + maximum.astype(numpy.float64)) / 2.0


def _signed_gap(mask: numpy.ndarray, target: numpy.ndarray) -> float | None:
	"""Measure nearest target-to-footprint gap, negative when their ink overlaps."""
	if not numpy.any(mask):
		return None
	overlap = int(numpy.count_nonzero(mask & target))
	if overlap:
		return -float(overlap)
	distance = cv2.distanceTransform((~target).astype(numpy.uint8), cv2.DIST_L2, 5)
	return float(distance[mask].min())


def _perpendicular_error(
	mask: numpy.ndarray, target_center: numpy.ndarray, other_center: numpy.ndarray,
) -> float | None:
	"""Measure the footprint's local attachment medial line against the glyph-center axis."""
	direction = other_center - target_center
	length = float(numpy.linalg.norm(direction))
	if length == 0.0:
		return None
	unit = direction / length
	points = numpy.argwhere(mask).astype(numpy.float64)
	if len(points) == 0:
		return None
	# The footprint's full medial line is robust to parallel lanes whose clipped
	# endpoints intentionally exit a glyph at different longitudinal distances.
	# Its perpendicular median remains pixel-derived and is valid for ordinary,
	# multiple, wedge, dashed, wavy, and Haworth final footprints.
	delta = points - target_center
	perpendicular_offsets = delta[:, 0] * unit[1] - delta[:, 1] * unit[0]
	return float(abs(numpy.median(perpendicular_offsets)))


def measure_layers(
	composite: numpy.ndarray,
	core_glyph_masks: Mapping[str, numpy.ndarray],
	bond_footprint_masks: Mapping[str, numpy.ndarray],
	bonds: Sequence[BondIdentity],
) -> dict[str, object]:
	"""Measure graph-labeled raster layers without inspecting render-plan geometry."""
	if not core_glyph_masks:
		raise ValueError("measurement requires at least one target core glyph mask")
	if len(bonds) > _MAX_BOND_COUNT:
		raise ValueError("measurement exceeds bond identity limit")
	_validate_same_shape((composite, *core_glyph_masks.values(), *bond_footprint_masks.values()))
	bond_ids = set()
	for bond in bonds:
		if bond.bond_id in bond_ids or bond.start_atom == bond.end_atom:
			raise ValueError("bond identities must be unique and connect two distinct atoms")
		bond_ids.add(bond.bond_id)
	if bond_ids != set(bond_footprint_masks):
		raise ValueError("bond identities and final footprint masks must have one-to-one coverage")
	centers = {atom_id: _core_center(mask, atom_id) for atom_id, mask in core_glyph_masks.items()}
	measurements = []
	for bond in bonds:
		if bond.start_atom not in centers or bond.end_atom not in centers:
			raise ValueError(f"{bond.bond_id} references a target core glyph that is absent")
		mask = bond_footprint_masks.get(bond.bond_id)
		if mask is None:
			raise ValueError(f"bond footprint is absent: {bond.bond_id}")
		ink_pixels = int(numpy.count_nonzero(mask))
		coverage = float(numpy.count_nonzero(mask & composite) / ink_pixels) if ink_pixels else 0.0
		endpoints = []
		for target, other in ((bond.start_atom, bond.end_atom), (bond.end_atom, bond.start_atom)):
			collision = sum(
				int(numpy.count_nonzero(mask & other_mask))
				for atom_id, other_mask in core_glyph_masks.items()
				if atom_id not in {target, other}
			)
			endpoints.append({
				"target_atom": target,
				"centerline_perpendicular_error_px": _perpendicular_error(
					mask, centers[target], centers[other],
				),
				"signed_intended_label_gap_px": _signed_gap(mask, core_glyph_masks[target]),
				"non_endpoint_label_collision_pixels": collision,
			})
		measurements.append({
			"bond_id": bond.bond_id,
			"style": bond.style,
			"final_footprint_pixels": ink_pixels,
			"final_footprint_coverage": coverage,
			"endpoints": endpoints,
		})
	return {"schema": SCHEMA, "bonds": measurements}


def violations(report: Mapping[str, object], thresholds: MeasurementThresholds) -> list[str]:
	"""Return developer-gate violations without silently changing thresholds."""
	result = []
	for bond in report["bonds"]:
		bond_id = bond["bond_id"]
		if bond["final_footprint_pixels"] == 0:
			result.append(f"{bond_id}: final footprint is empty")
		if bond["final_footprint_coverage"] < thresholds.min_final_footprint_coverage:
			result.append(f"{bond_id}: final footprint coverage is below threshold")
		for endpoint in bond["endpoints"]:
			error = endpoint["centerline_perpendicular_error_px"]
			gap = endpoint["signed_intended_label_gap_px"]
			if type(error) is not float or not math.isfinite(error) or error > thresholds.max_centerline_error:
				result.append(f"{bond_id}:{endpoint['target_atom']}: centerline error exceeds threshold")
			if (
				type(gap) is not float
				or not math.isfinite(gap)
				or gap < thresholds.min_signed_intended_label_gap
			):
				result.append(f"{bond_id}:{endpoint['target_atom']}: intended label gap is below threshold")
			if (
					endpoint["non_endpoint_label_collision_pixels"]
					> thresholds.max_non_endpoint_collision_pixels
			):
				result.append(f"{bond_id}:{endpoint['target_atom']}: non-endpoint label collision")
	return result


def _mask_canvas(mask: numpy.ndarray) -> numpy.ndarray:
	"""Create a readable white-background BGR canvas from one binary raster layer."""
	image = numpy.where(mask[:, :, None], 0, 255).astype(numpy.uint8)
	return cv2.cvtColor(image, cv2.COLOR_GRAY2BGR)


def _write_image(image: numpy.ndarray, output: pathlib.Path) -> None:
	"""Write one diagnostic image and reject silent OpenCV publication failures."""
	if not cv2.imwrite(str(output), image):
		raise OSError(f"could not write alignment diagnostic: {output}")
	if not output.is_file() or output.stat().st_size == 0:
		raise OSError(f"alignment diagnostic was not published: {output}")


def _draw_mask_set(
	canvas: numpy.ndarray,
	masks: Mapping[str, numpy.ndarray],
	color: tuple[int, int, int],
) -> numpy.ndarray:
	"""Overlay independently issued binary masks without inferring vector geometry."""
	for mask in masks.values():
		canvas[mask] = color
	return canvas


def _draw_heading(image: numpy.ndarray, heading: str) -> numpy.ndarray:
	"""Annotate a diagnostic panel with its pixel-layer role."""
	cv2.putText(image, heading, (8, 18), cv2.FONT_HERSHEY_SIMPLEX, 0.45, (0, 0, 0), 1)
	return image


def _write_diagnostics(
	composite: numpy.ndarray,
	core_glyph_masks: Mapping[str, numpy.ndarray],
	bond_footprint_masks: Mapping[str, numpy.ndarray],
	report: Mapping[str, object],
	output_directory: pathlib.Path,
) -> None:
	"""Publish annotated overlays and a four-panel contact sheet for one fixture case."""
	composite_overlay = _draw_heading(_mask_canvas(composite), "normal composite")
	core_overlay = _draw_heading(
		_draw_mask_set(_mask_canvas(composite), core_glyph_masks, (0, 150, 0)),
		"target core glyph masks",
	)
	bond_overlay = _draw_heading(
		_draw_mask_set(_mask_canvas(composite), bond_footprint_masks, (220, 140, 0)),
		"final bond footprint masks",
	)
	failing_bond_ids = {
		bond["bond_id"]
		for bond in report["bonds"]
		if any(
			item.startswith(f"{bond['bond_id']}:")
			or item == f"{bond['bond_id']}: final footprint is empty"
			for item in report["violations"]
		)
	}
	failure_overlay = _mask_canvas(composite)
	for bond_id, mask in bond_footprint_masks.items():
		failure_overlay[mask] = (0, 0, 220) if bond_id in failing_bond_ids else (0, 150, 0)
	failure_overlay = _draw_heading(failure_overlay, "failed bond footprints red")
	for filename, image in (
		("normal_composite_overlay.png", composite_overlay),
		("target_core_glyph_masks_overlay.png", core_overlay),
		("final_bond_footprints_overlay.png", bond_overlay),
		("alignment_failures_overlay.png", failure_overlay),
	):
		_write_image(image, output_directory / filename)
	contact_sheet = numpy.concatenate(
		(composite_overlay, core_overlay, bond_overlay, failure_overlay), axis=1,
	)
	_write_image(contact_sheet, output_directory / "alignment_contact_sheet.png")


def _reject_duplicate_json_keys(pairs: list[tuple[object, object]]) -> dict[str, object]:
	"""Parse JSON objects as closed maps instead of silently accepting duplicate keys."""
	result = {}
	for key, value in pairs:
		if type(key) is not str or key in result:
			raise ValueError("measurement manifest has a duplicate or invalid JSON key")
		result[key] = value
	return result


def _reject_json_constant(value: str) -> None:
	"""Reject nonstandard JSON constants such as NaN and Infinity."""
	raise ValueError(f"measurement manifest has unsupported JSON constant: {value}")


def _validate_identity(value: object, field: str) -> str:
	"""Accept bounded ASCII fixture identities only; they never carry render geometry."""
	if type(value) is not str or _IDENTITY_PATTERN.fullmatch(value) is None:
		raise ValueError(f"{field} must be a bounded ASCII fixture identity")
	return value


def _resolve_raster_path(root: pathlib.Path, relative: object) -> pathlib.Path:
	"""Resolve one relative layer under the manifest directory without escape paths."""
	if type(relative) is not str or not relative:
		raise ValueError("raster path must be a nonempty string")
	relative_path = pathlib.PurePath(relative)
	if relative_path.is_absolute() or ".." in relative_path.parts:
		raise ValueError("raster path must stay below its manifest directory")
	candidate = (root / relative_path).resolve()
	if candidate == root or root not in candidate.parents:
		raise ValueError("raster path escapes its manifest directory")
	return candidate


def _validate_manifest_bonds(value: object, core_ids: set[str]) -> list[BondIdentity]:
	"""Validate every graph identity and its cross-field relationships before reading rasters."""
	if type(value) is not list or not value or len(value) > _MAX_BOND_COUNT:
		raise ValueError("bonds must be a nonempty bounded list")
	identities = []
	bond_ids = set()
	for entry in value:
		if type(entry) is not dict or set(entry) != {"bond_id", "start_atom", "end_atom", "style"}:
			raise ValueError("each bond identity must have exactly bond_id, start_atom, end_atom, style")
		bond_id = _validate_identity(entry["bond_id"], "bond_id")
		start_atom = _validate_identity(entry["start_atom"], "start_atom")
		end_atom = _validate_identity(entry["end_atom"], "end_atom")
		style = _validate_identity(entry["style"], "style")
		if bond_id in bond_ids or start_atom == end_atom:
			raise ValueError("bond identities must be unique and connect two distinct atoms")
		if start_atom not in core_ids or end_atom not in core_ids:
			raise ValueError("bond endpoint identity is absent from target core glyph masks")
		bond_ids.add(bond_id)
		identities.append(BondIdentity(bond_id, start_atom, end_atom, style))
	return identities


def _load_manifest(
	path: pathlib.Path,
) -> tuple[numpy.ndarray, dict[str, numpy.ndarray], dict[str, numpy.ndarray], list[BondIdentity]]:
	"""Load the closed developer handoff manifest and its adjacent raster layers."""
	# ASVS V1.5.2 and V2.2.1: accept only bounded JSON data of this closed schema.
	if not path.is_file() or path.stat().st_size > _MAX_MANIFEST_BYTES:
		raise ValueError("measurement manifest must be a bounded regular file")
	manifest_bytes = path.read_bytes()
	value = json.loads(
		manifest_bytes.decode("utf-8"),
		object_pairs_hook=_reject_duplicate_json_keys,
		parse_constant=_reject_json_constant,
	)
	required_fields = {
		"schema", "normal_composite", "target_core_glyph_masks",
		"final_bond_footprints", "bonds",
	}
	if type(value) is not dict or set(value) != required_fields:
		raise ValueError("measurement manifest has unknown or missing fields")
	if value["schema"] != _RASTER_MANIFEST_SCHEMA:
		raise ValueError("unsupported measurement raster manifest schema")
	root = path.parent.resolve()
	if type(value["target_core_glyph_masks"]) is not dict or not value["target_core_glyph_masks"]:
		raise ValueError("target_core_glyph_masks must be a nonempty object")
	if type(value["final_bond_footprints"]) is not dict:
		raise ValueError("final_bond_footprints must be an object")
	if len(value["target_core_glyph_masks"]) + len(value["final_bond_footprints"]) + 1 > _MAX_LAYER_COUNT:
		raise ValueError("measurement manifest exceeds layer limit")
	core_ids = {_validate_identity(atom_id, "target core glyph identity") for atom_id in value["target_core_glyph_masks"]}
	identities = _validate_manifest_bonds(value["bonds"], core_ids)
	# ASVS V2.1.1, V2.1.2, V2.1.3, and V2.2.3: cross-check related identities.
	for bond_id in value["final_bond_footprints"]:
		_validate_identity(bond_id, "final bond footprint identity")
	if set(value["final_bond_footprints"]) != {identity.bond_id for identity in identities}:
		raise ValueError("final_bond_footprints must name every bond identity exactly once")
	paths = [
		_resolve_raster_path(root, value["normal_composite"]),
		*[_resolve_raster_path(root, relative) for relative in value["target_core_glyph_masks"].values()],
		*[_resolve_raster_path(root, relative) for relative in value["final_bond_footprints"].values()],
	]
	if len(set(paths)) != len(paths):
		raise ValueError("each raster layer must use a distinct file")
	if sum(item.stat().st_size for item in paths) > _MAX_TOTAL_RASTER_FILE_BYTES:
		raise ValueError("measurement raster layers exceed total byte limit")
	core = {
		atom_id: _read_mask(_resolve_raster_path(root, relative))
		for atom_id, relative in value["target_core_glyph_masks"].items()
	}
	bonds = {
		bond_id: _read_mask(_resolve_raster_path(root, relative))
		for bond_id, relative in value["final_bond_footprints"].items()
	}
	return _foreground_mask(paths[0]), core, bonds, identities


def _self_test() -> None:
	"""Run a deterministic pixel oracle for metrics, collision detection, and diagnostics."""
	composite = numpy.zeros((24, 40), dtype=bool)
	left = numpy.zeros_like(composite)
	right = numpy.zeros_like(composite)
	third = numpy.zeros_like(composite)
	left[8:16, 3:7] = True
	right[8:16, 33:37] = True
	third[2:6, 20:24] = True
	bond_masks = {}
	for style, rows in {
		"normal": ((11, 13),),
		"double": ((9, 10), (13, 14)),
		"triple": ((8, 9), (11, 13), (15, 16)),
		"dashed": ((11, 13),),
		"bold": ((9, 15),),
		"wavy": ((10, 12),),
		"wedge": ((10, 14),),
		"hashed": ((11, 13),),
		"haworth-front": ((11, 13),),
	}.items():
		mask = numpy.zeros_like(composite)
		for row_start, row_end in rows:
			mask[row_start:row_end, 7:33] = True
		if style == "dashed":
			mask[:, 13:17] = False
			mask[:, 23:27] = False
		if style == "wavy":
			mask[10:12, 18:24] = False
			mask[12:14, 18:24] = True
		if style == "wedge":
			for column in range(7, 33):
				half_width = max(1, (column - 6) // 6)
				mask[12 - half_width:12 + half_width, column] = True
		if style == "hashed":
			mask[:, 12:16] = False
			mask[:, 20:24] = False
		bond_masks[style] = mask
		composite |= mask
	composite |= left | right
	identities = [
		BondIdentity(f"ab-{style}", "a", "b", style)
		for style in bond_masks
	]
	footprints = {f"ab-{style}": mask for style, mask in bond_masks.items()}
	report = measure_layers(
		composite,
		{"a": left, "b": right},
		footprints,
		identities,
	)
	for entry in report["bonds"]:
		if entry["final_footprint_pixels"] == 0 or entry["final_footprint_coverage"] != 1.0:
			raise RuntimeError("independent raster metric oracle failed footprint coverage")
		for endpoint in entry["endpoints"]:
			if endpoint["signed_intended_label_gap_px"] != 1.0:
				raise RuntimeError("independent raster metric oracle failed label gap")
			if endpoint["centerline_perpendicular_error_px"] is None:
				raise RuntimeError("independent raster metric oracle failed attachment locus")
	if violations(report, MeasurementThresholds()):
		raise RuntimeError("independent raster metric oracle unexpectedly violated thresholds")
	normal_mask = bond_masks["normal"]
	crossing_report = measure_layers(
		composite,
		{"a": left, "b": right, "c": third},
		{"ab": normal_mask | third},
		[BondIdentity("ab", "a", "b", "normal")],
	)
	if not any("non-endpoint label collision" in item for item in violations(crossing_report, MeasurementThresholds())):
		raise RuntimeError("independent raster metric oracle failed collision detection")
	overlap_report = measure_layers(
		composite,
		{"a": left, "b": right},
		{"ab": normal_mask | left},
		[BondIdentity("ab", "a", "b", "normal")],
	)
	if not any("intended label gap is below threshold" in item for item in violations(overlap_report, MeasurementThresholds())):
		raise RuntimeError("independent raster metric oracle failed negative gap rejection")
	with tempfile.TemporaryDirectory() as temporary_directory:
		output = pathlib.Path(temporary_directory)
		report["violations"] = violations(report, MeasurementThresholds())
		_write_diagnostics(composite, {"a": left, "b": right}, footprints, report, output)
		for filename in (
			"normal_composite_overlay.png", "target_core_glyph_masks_overlay.png",
			"final_bond_footprints_overlay.png", "alignment_failures_overlay.png",
			"alignment_contact_sheet.png",
		):
			if cv2.imread(str(output / filename), cv2.IMREAD_UNCHANGED) is None:
				raise RuntimeError("independent raster diagnostic oracle did not write a readable image")


def main() -> int:
	"""Run the local pixel-only measurement lane."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--manifest", type=pathlib.Path)
	parser.add_argument(
		"--output-dir", type=pathlib.Path,
		default=pathlib.Path("output_glyph_alignment"),
	)
	parser.add_argument("--fail-on-violation", action="store_true")
	parser.add_argument("--self-test", action="store_true")
	arguments = parser.parse_args()
	if arguments.self_test:
		_self_test()
		print(json.dumps({"status": "ok", "self_test": True}))
		return 0
	if arguments.manifest is None:
		parser.error("--manifest is required unless --self-test is selected")
	composite, cores, footprints, bonds = _load_manifest(arguments.manifest)
	report = measure_layers(composite, cores, footprints, bonds)
	report["violations"] = violations(report, MeasurementThresholds())
	arguments.output_dir.mkdir(parents=True, exist_ok=True)
	if not arguments.output_dir.is_dir():
		raise ValueError(f"output path is not a directory: {arguments.output_dir}")
	metrics_path = arguments.output_dir / "alignment_metrics.json"
	metrics_path.write_text(json.dumps(report, indent=2, allow_nan=False) + "\n", encoding="utf-8")
	if not metrics_path.is_file() or metrics_path.stat().st_size == 0:
		raise OSError(f"could not write alignment metrics: {metrics_path}")
	_write_diagnostics(composite, cores, footprints, report, arguments.output_dir)
	print(json.dumps({"status": "ok", "violations": len(report["violations"])}))
	return 1 if arguments.fail_on_violation and report["violations"] else 0


if __name__ == "__main__":
	raise SystemExit(main())
