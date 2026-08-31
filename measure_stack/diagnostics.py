"""Deterministic diagnostic images and JSON reports for measure_stack."""

# Standard Library
import json
import math
import pathlib
from collections.abc import Mapping

# PIP3 modules
import cv2
import numpy

# Local modules
from measure_stack.contracts import SceneLayers


# ============================================
def _canvas(mask: numpy.ndarray) -> numpy.ndarray:
    """Create a white BGR canvas from a binary ink layer."""
    result = numpy.full((*mask.shape, 3), 255, dtype=numpy.uint8)
    result[mask] = (0, 0, 0)
    return result


# ============================================
def _write_png(path: pathlib.Path, image: numpy.ndarray) -> None:
    """Publish a diagnostic atomically enough to reject OpenCV write failures."""
    if not cv2.imwrite(str(path), image) or not path.is_file() or path.stat().st_size == 0:
        raise OSError(f"could not write diagnostic PNG: {path}")


# ============================================
def _json_safe(value: object) -> object:
    """Convert diagnostic values to strict JSON without concealing failed metrics."""
    if isinstance(value, numpy.generic):
        value = value.item()
    if isinstance(value, float):
        return value if math.isfinite(value) else None
    if isinstance(value, Mapping):
        return {str(key): _json_safe(item) for key, item in value.items()}
    if isinstance(value, tuple | list):
        return [_json_safe(item) for item in value]
    return value


# ============================================
def write_diagnostics(scene: SceneLayers, report: Mapping[str, object], output: pathlib.Path) -> None:
    """Write stable overlays, a contact sheet, and canonical JSON into output."""
    output.mkdir(parents=True, exist_ok=True)
    if not output.is_dir():
        raise ValueError("diagnostic output must be a directory")
    composite = _canvas(scene.composite)
    cores = _canvas(scene.composite)
    labels = _canvas(scene.composite)
    bonds = _canvas(scene.composite)
    unexplained = _canvas(scene.composite)
    failures = _canvas(scene.composite)
    declared = numpy.zeros_like(scene.composite)
    for atom in scene.atoms.values():
        cores[atom.core_mask] = (0, 130, 0)
        full_label = getattr(atom, "full_label_mask", None)
        if full_label is None:
            full_label = atom.core_mask
        labels[full_label] = (150, 0, 150)
        declared |= full_label
    failed_ids = {item.split(":", 1)[0] for item in report["violations"] if ":" in item}
    for bond in scene.bonds:
        bonds[bond.footprint_mask] = (210, 130, 0)
        failures[bond.footprint_mask] = (0, 0, 220) if bond.bond_id in failed_ids else (0, 140, 0)
        declared |= bond.footprint_mask
    unexplained[scene.composite & ~declared] = (0, 0, 255)
    panels = (("composite", composite), ("target cores", cores), ("full labels", labels), ("bond footprints", bonds), ("unexplained foreground", unexplained), ("violations", failures))
    rendered: list[numpy.ndarray] = []
    for title, image in panels:
        panel = image.copy()
        cv2.putText(panel, title, (8, 20), cv2.FONT_HERSHEY_SIMPLEX, 0.55, (0, 0, 0), 1)
        _write_png(output / f"{title.replace(' ', '_')}.png", panel)
        rendered.append(panel)
    _write_png(output / "contact_sheet.png", numpy.concatenate(rendered, axis=1))
    metrics = output / "measurement_report.json"
    # ``violations()`` emits an explicit ``measurement is nonfinite`` category
    # before this serializer converts an unrepresentable raw numeric value to
    # null.  The report stays strict JSON while retaining the failure signal.
    metrics.write_text(json.dumps(_json_safe(report), sort_keys=True, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    if not metrics.is_file() or metrics.stat().st_size == 0:
        raise OSError("could not write measurement JSON")
