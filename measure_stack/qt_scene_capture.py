"""Test-only V2 Qt scene capture for final-image measurement evidence.

This module never recreates a chemical drawing. Its composite and full-label
layers are painted by the existing ``FerrumRenderProjection.scene``. A core
glyph path is a deliberately isolated test-only source map: it is derived from
the same issued Telex run as its actual full-label item, then rendered through
that same scene only while the core mask is captured.
"""

# Standard Library
import hashlib
import json
import pathlib
import re
from collections.abc import Mapping, Sequence

# Local modules
from measure_stack.contracts import CaptureProfile, RASTER_LAYER_MANIFEST_V2_SCHEMA


_IDENTITY = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$")
_MAX_ITEMS = 256


# ============================================
def _qt_types() -> tuple[object, object, object, object, object]:
	"""Import Qt lazily so pixel-only measurement users need no Qt runtime."""
	# PIP3 modules
	from PySide6.QtCore import Qt, QRectF
	from PySide6.QtGui import QColor, QImage, QPainter
	return Qt, QRectF, QColor, QImage, QPainter


# ============================================
def _identity(value: object, field: str) -> str:
	"""Require a bounded fixture identity before it reaches a filename."""
	if type(value) is not str or _IDENTITY.fullmatch(value) is None:
		raise ValueError(f"{field} must be a bounded ASCII fixture identity")
	return value


# ============================================
def _sha256(path: pathlib.Path) -> str:
	"""Hash a written layer so the strict V2 reader can detect later mutation."""
	return hashlib.sha256(path.read_bytes()).hexdigest()


# ============================================
def _write_image(image: object, path: pathlib.Path) -> dict[str, str]:
	"""Write one layer and return its closed, hash-bound manifest record."""
	if not image.save(str(path), "PNG") or not path.is_file() or path.stat().st_size == 0:
		raise OSError(f"could not write Qt capture image: {path}")
	return {"relative_path": path.name, "sha256": _sha256(path)}


# ============================================
def _ancestors(item: object) -> list[object]:
	"""Return an item and its parents, which Qt needs visible to paint it."""
	result = [item]
	parent = item.parentItem()
	while parent is not None:
		result.append(parent)
		parent = parent.parentItem()
	return result


# ============================================
def _descendants(item: object) -> list[object]:
	"""Return an item and all children that its ordinary paint subtree needs."""
	result = [item]
	for child in item.childItems():
		result.extend(_descendants(child))
	return result


# ============================================
def _explicit_visibility(item: object) -> bool:
	"""Read this item's own flag without inheriting a hidden parent flag."""
	parent = item.parentItem()
	if parent is None:
		return item.isVisible()
	return item.isVisibleTo(parent)


# ============================================
def _render(scene: object, profile: CaptureProfile) -> object:
	"""Render the unmodified existing scene at a fixed fixture-owned profile."""
	Qt, QRectF, QColor, QImage, QPainter = _qt_types()
	image = QImage(
		profile.pixel_width,
		profile.pixel_height,
		QImage.Format.Format_ARGB32_Premultiplied,
	)
	image.setDevicePixelRatio(profile.device_pixel_ratio)
	image.fill(QColor(0, 0, 0, 0))
	source = QRectF(*profile.source_rect)
	# QImage stores physical pixels but QPainter receives device-independent units
	# after a non-unit DPR is installed. Match the fixed source to that logical
	# target rectangle so a 2x profile does not silently crop the right/bottom.
	target_width = profile.pixel_width / profile.device_pixel_ratio
	target_height = profile.pixel_height / profile.device_pixel_ratio
	painter = QPainter(image)
	try:
		painter.setRenderHint(QPainter.RenderHint.Antialiasing, True)
		scene.render(
			painter,
			QRectF(0.0, 0.0, target_width, target_height),
			source,
			Qt.AspectRatioMode.IgnoreAspectRatio,
		)
	finally:
		painter.end()
	return image


# ============================================
def _render_isolated(scene: object, profile: CaptureProfile,
		visible_items: Sequence[object]) -> object:
	"""Render selected existing roots and restore every visibility flag."""
	all_items = list(scene.items())
	original = {item: _explicit_visibility(item) for item in all_items}
	try:
		for item in all_items:
			item.setVisible(False)
		for selected in visible_items:
			for ancestor in _ancestors(selected):
				ancestor.setVisible(True)
			for descendant in _descendants(selected):
				descendant.setVisible(True)
		return _render(scene, profile)
	finally:
		for item, visible in original.items():
			item.setVisible(visible)


# ============================================
def _require_scene_item(scene: object, item: object, field: str) -> None:
	"""Reject a detached or foreign item before it can forge a source layer."""
	if item is None or item.scene() is not scene:
		raise ValueError(f"{field} must be an item in the captured projection scene")


# ============================================
def _capture_atom_mappings(scene: object,
		atoms: Mapping[str, tuple[str, object, object]]) -> list[tuple[str, str, object, object]]:
	"""Validate the one-to-one source map to actual labels and core test paths."""
	if not atoms or len(atoms) > _MAX_ITEMS:
		raise ValueError("Qt capture requires a bounded nonempty atom source map")
	result = []
	for atom_id, value in atoms.items():
		atom_id = _identity(atom_id, "atom ID")
		if type(value) is not tuple or len(value) != 3:
			raise ValueError("atom source map must contain element, full-label item, and core item")
		element, full_label_item, core_item = value
		element = _identity(element, "element")
		_require_scene_item(scene, full_label_item, "full-label item")
		_require_scene_item(scene, core_item, "core-glyph item")
		if full_label_item is core_item:
			raise ValueError("full-label item and isolated core item must differ")
		result.append((atom_id, element, full_label_item, core_item))
	if len({entry[0] for entry in result}) != len(result):
		raise ValueError("atom source identities must be unique")
	if len({entry[2] for entry in result}) != len(result):
		raise ValueError("each actual full-label item needs one fixture atom identity")
	if len({entry[3] for entry in result}) != len(result):
		raise ValueError("each isolated core path needs one fixture atom identity")
	return sorted(result)


# ============================================
def _capture_bond_mappings(scene: object, bonds: Mapping[str, tuple[str, str, str, object]],
		atom_ids: set[str]) -> list[tuple[str, str, str, str, object]]:
	"""Validate final-footprint source ownership against the declared atom graph."""
	if len(bonds) > _MAX_ITEMS:
		raise ValueError("Qt capture exceeds bounded bond layer count")
	result = []
	for bond_id, value in bonds.items():
		bond_id = _identity(bond_id, "bond ID")
		if type(value) is not tuple or len(value) != 4:
			raise ValueError("bond source map must contain endpoints, style, and final item")
		start_atom, end_atom, style, item = value
		start_atom = _identity(start_atom, "bond start atom")
		end_atom = _identity(end_atom, "bond end atom")
		style = _identity(style, "bond style")
		if start_atom == end_atom or start_atom not in atom_ids or end_atom not in atom_ids:
			raise ValueError("bond endpoints must name distinct captured graph atoms")
		_require_scene_item(scene, item, "final-bond item")
		result.append((bond_id, start_atom, end_atom, style, item))
	if len({entry[0] for entry in result}) != len(result):
		raise ValueError("bond source identities must be unique")
	if len({entry[4] for entry in result}) != len(result):
		raise ValueError("each actual final-bond item needs one fixture bond identity")
	return sorted(result)


# ============================================
def capture_scene(
		scene: object,
		capture_profile: CaptureProfile,
		fixture_id: str,
		fixture_cdml: str,
		chemical_roots: Sequence[object],
		atoms: Mapping[str, tuple[str, object, object]],
		bonds: Mapping[str, tuple[str, str, str, object]],
		expected_relations: Sequence[dict[str, str]],
		negative_cases: Sequence[dict[str, str]],
		output_directory: pathlib.Path,
		) -> pathlib.Path:
	"""Capture real Qt consumer layers under one named, fixed V2 profile.

	``chemical_roots`` must be actual projection molecule roots. The composite
	therefore includes every actual label decoration and bond in the projection
	subtree, while excluding paper and UI items by construction. It is never
	rebuilt from the source maps.
	"""
	if type(capture_profile) is not CaptureProfile:
		raise ValueError("Qt capture requires an exact fixture CaptureProfile")
	fixture_id = _identity(fixture_id, "fixture ID")
	if type(fixture_cdml) is not str or not fixture_cdml:
		raise ValueError("Qt capture requires fixture CDML text for immutable provenance")
	if not chemical_roots:
		raise ValueError("Qt capture requires actual chemical projection roots")
	for root in chemical_roots:
		_require_scene_item(scene, root, "chemical projection root")
	if len(set(chemical_roots)) != len(chemical_roots):
		raise ValueError("chemical projection roots must be unique")
	if not isinstance(expected_relations, Sequence) or not isinstance(negative_cases, Sequence):
		raise ValueError("Qt capture requires fixture-owned relation sequences")
	if output_directory.exists() and any(output_directory.iterdir()):
		raise ValueError("Qt capture output directory must be empty")
	output_directory.mkdir(parents=True, exist_ok=True)
	if not output_directory.is_dir():
		raise ValueError("Qt capture output must be a directory")

	normalized_atoms = _capture_atom_mappings(scene, atoms)
	normalized_bonds = _capture_bond_mappings(
		scene, bonds, {entry[0] for entry in normalized_atoms},
	)
	composite_path = output_directory / "final_composite.png"
	composite_layer = _write_image(
		_render_isolated(scene, capture_profile, chemical_roots), composite_path,
	)
	atom_layers = []
	for atom_id, _element, full_label_item, core_item in normalized_atoms:
		core_path = output_directory / f"atom_{atom_id}_core.png"
		full_path = output_directory / f"atom_{atom_id}_full_label.png"
		atom_layers.append({
			"atom_id": atom_id,
			"core_glyph_layer": _write_image(
				_render_isolated(scene, capture_profile, [core_item]), core_path,
			),
			"full_label_layer": _write_image(
				_render_isolated(scene, capture_profile, [full_label_item]), full_path,
			),
		})
	bond_layers = []
	for bond_id, _start, _end, _style, final_item in normalized_bonds:
		path = output_directory / f"bond_{bond_id}_final.png"
		bond_layers.append({
			"bond_id": bond_id,
			"final_bond_layer": _write_image(
				_render_isolated(scene, capture_profile, [final_item]), path,
			),
		})
	manifest = {
		"schema": RASTER_LAYER_MANIFEST_V2_SCHEMA,
		"fixture_id": fixture_id,
		"fixture_cdml_sha256": hashlib.sha256(fixture_cdml.encode("utf-8")).hexdigest(),
		"capture_profile": {
			"profile_id": capture_profile.profile_id,
			"source_rect": list(capture_profile.source_rect),
			"pixel_width": capture_profile.pixel_width,
			"pixel_height": capture_profile.pixel_height,
			"device_pixel_ratio": capture_profile.device_pixel_ratio,
			"scene_evaluation": capture_profile.scene_evaluation,
		},
		"graph": {
			"atoms": [
				{"atom_id": atom_id, "element": element}
				for atom_id, element, _full, _core in normalized_atoms
			],
			"bonds": [
				{
					"bond_id": bond_id,
					"start_atom_id": start_atom,
					"end_atom_id": end_atom,
					"style": style,
				}
				for bond_id, start_atom, end_atom, style, _item in normalized_bonds
			],
		},
		"composite_layer": composite_layer,
		"atom_layers": atom_layers,
		"bond_layers": bond_layers,
		"expected_relations": list(expected_relations),
		"negative_cases": list(negative_cases),
	}
	manifest_path = output_directory / "raster_layer_manifest_v2.json"
	manifest_path.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n", encoding="utf-8")
	return manifest_path
