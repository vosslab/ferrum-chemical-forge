"""Closed, versioned input contracts for independent Ferrum image measurement."""

# Standard Library
import dataclasses
import hashlib
import json
import pathlib
import re
from collections.abc import Mapping

# PIP3 modules
import cv2
import numpy


RASTER_LAYER_MANIFEST_V2_SCHEMA = "ferrum-measure-stack-raster-layers-v2"
_IDENTITY = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.:-]{0,127}$")
_MAX_MANIFEST_BYTES = 1_048_576
_MAX_IMAGE_BYTES = 67_108_864
_MAX_TOTAL_BYTES = 536_870_912
_MAX_PIXELS = 33_554_432
_MAX_ATOMS = 256
_MAX_BONDS = 256


@dataclasses.dataclass(frozen=True)
class AtomLayer:
    """One target atom character's isolated core-ink mask."""

    atom_id: str
    element: str
    core_mask: numpy.ndarray
    full_label_mask: numpy.ndarray


@dataclasses.dataclass(frozen=True)
class BondLayer:
    """One final rendered bond footprint and its fixture-owned endpoints."""

    bond_id: str
    start_atom: str
    end_atom: str
    style: str
    footprint_mask: numpy.ndarray


@dataclasses.dataclass(frozen=True)
class SceneLayers:
    """Validated image layers loaded without renderer-issued geometry metadata."""

    schema: str
    composite: numpy.ndarray
    atoms: dict[str, AtomLayer]
    bonds: tuple[BondLayer, ...]
    pixel_scale: float
    viewport_width_px: int | None
    viewport_height_px: int | None
    fixture_id: str | None = None
    fixture_cdml_sha256: str | None = None
    capture_profile: "CaptureProfile | None" = None
    expected_relations: tuple[dict[str, object], ...] = ()
    negative_cases: tuple[dict[str, object], ...] = ()


@dataclasses.dataclass(frozen=True)
class CaptureProfile:
    """Fixture-owned fixed capture geometry and its evaluation semantics."""

    profile_id: str
    source_rect: tuple[float, float, float, float]
    pixel_width: int
    pixel_height: int
    device_pixel_ratio: float
    scene_evaluation: str


# ============================================
def _reject_duplicate_keys(pairs: list[tuple[object, object]]) -> dict[str, object]:
    """Reject duplicate JSON keys rather than allowing ambiguous manifests."""
    result: dict[str, object] = {}
    for key, value in pairs:
        if type(key) is not str or key in result:
            raise ValueError("manifest contains a duplicate or invalid JSON key")
        result[key] = value
    return result


# ============================================
def _reject_constant(value: str) -> None:
    """Reject nonstandard JSON NaN and Infinity tokens."""
    raise ValueError(f"manifest has unsupported JSON constant: {value}")


# ============================================
def _identity(value: object, field: str) -> str:
    """Validate a bounded fixture identity, not arbitrary display text."""
    if type(value) is not str or _IDENTITY.fullmatch(value) is None:
        raise ValueError(f"{field} must be a bounded ASCII identity")
    return value


# ============================================
def _relative_path(root: pathlib.Path, value: object) -> pathlib.Path:
    """Resolve a regular image below the manifest directory without traversal."""
    if type(value) is not str or not value:
        raise ValueError("image path must be a nonempty string")
    raw = pathlib.PurePath(value)
    if raw.is_absolute() or ".." in raw.parts:
        raise ValueError("image path must stay below the manifest directory")
    candidate = (root / raw).resolve()
    if candidate == root or root not in candidate.parents:
        raise ValueError("image path escapes the manifest directory")
    if candidate.suffix.lower() != ".png" or not candidate.is_file():
        raise ValueError("image path must be a regular PNG file")
    if candidate.stat().st_size > _MAX_IMAGE_BYTES:
        raise ValueError("image exceeds the per-file byte limit")
    return candidate


# ============================================
def _read_mask(path: pathlib.Path) -> numpy.ndarray:
    """Read a bounded PNG into a boolean ink mask using alpha when available."""
    image = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
    if image is None or image.ndim not in {2, 3}:
        raise ValueError(f"could not read image: {path}")
    if image.shape[0] * image.shape[1] > _MAX_PIXELS:
        raise ValueError("image exceeds the pixel limit")
    if image.ndim == 2:
        return image != 0
    if image.shape[2] not in {1, 2, 3, 4}:
        raise ValueError("image has unsupported channel count")
    if image.shape[2] == 4:
        return image[:, :, 3] != 0
    return numpy.any(image != 0, axis=2)


# ============================================
def _read_composite(path: pathlib.Path) -> numpy.ndarray:
    """Read composite foreground with transparent alpha as authoritative."""
    image = cv2.imread(str(path), cv2.IMREAD_UNCHANGED)
    if image is None or image.ndim not in {2, 3}:
        raise ValueError(f"could not read composite: {path}")
    if image.shape[0] * image.shape[1] > _MAX_PIXELS:
        raise ValueError("composite exceeds the pixel limit")
    if image.ndim == 2:
        return image < 250
    if image.shape[2] not in {1, 2, 3, 4}:
        raise ValueError("composite has unsupported channel count")
    if image.shape[2] == 4:
        return image[:, :, 3] != 0
    return numpy.any(image < 250, axis=2)


# ============================================
def _sha256(path: pathlib.Path) -> str:
    """Return one layer's content hash without trusting a producer declaration."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


# ============================================
def _sha256_value(value: object, field: str) -> str:
    """Require a lowercase SHA-256 digest in the durable V2 manifest."""
    if type(value) is not str or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise ValueError(f"{field} must be a lowercase SHA-256 digest")
    return value


# ============================================
def _hashed_layer(root: pathlib.Path, value: object, context: str) -> tuple[pathlib.Path, str]:
    """Resolve a layer and prove its immutable declared content digest."""
    item = _closed_mapping(value, {"relative_path", "sha256"}, context)
    path = _relative_path(root, item["relative_path"])
    digest = _sha256_value(item["sha256"], f"{context}.sha256")
    if _sha256(path) != digest:
        raise ValueError(f"{context} SHA-256 does not match its image")
    return path, digest


# ============================================
def _closed_mapping(value: object, fields: set[str], context: str) -> Mapping[str, object]:
    """Require a JSON object with exactly the documented fields."""
    if type(value) is not dict or set(value) != fields:
        raise ValueError(f"{context} has unknown or missing fields")
    return value


# ============================================
def _v2_profile(value: object) -> CaptureProfile:
    """Parse the fixed fixture capture profile without any ink-derived defaults."""
    item = _closed_mapping(
        value,
        {
            "profile_id", "source_rect", "pixel_width", "pixel_height",
            "device_pixel_ratio", "scene_evaluation",
        },
        "capture_profile",
    )
    profile_id = _identity(item["profile_id"], "capture_profile.profile_id")
    source_rect = item["source_rect"]
    if type(source_rect) is not list or len(source_rect) != 4:
        raise ValueError("capture_profile.source_rect must contain four fixed numbers")
    coordinates: list[float] = []
    for coordinate in source_rect:
        if type(coordinate) not in {int, float} or not numpy.isfinite(coordinate):
            raise ValueError("capture_profile.source_rect must be finite")
        coordinates.append(float(coordinate))
    if coordinates[2] <= 0.0 or coordinates[3] <= 0.0:
        raise ValueError("capture_profile.source_rect width and height must be positive")
    width = item["pixel_width"]
    height = item["pixel_height"]
    dpr = item["device_pixel_ratio"]
    scene_evaluation = item["scene_evaluation"]
    if type(width) is not int or type(height) is not int or width <= 0 or height <= 0:
        raise ValueError("capture_profile pixel dimensions must be positive integers")
    if width * height > _MAX_PIXELS:
        raise ValueError("capture_profile exceeds decoded-pixel limit")
    if type(dpr) not in {int, float} or not numpy.isfinite(dpr) or dpr <= 0.0:
        raise ValueError("capture_profile.device_pixel_ratio must be finite and positive")
    if scene_evaluation not in {"presentation", "raw_final_ink"}:
        raise ValueError("capture_profile.scene_evaluation must be presentation or raw_final_ink")
    return CaptureProfile(profile_id, tuple(coordinates), width, height, float(dpr), scene_evaluation)


# ============================================
def _v2_relations(value: object, atoms: set[str], bonds: set[str], field: str) -> tuple[dict[str, object], ...]:
    """Validate declarative, fixture-owned visual relations without geometry values."""
    if type(value) is not list or len(value) > _MAX_BONDS * 4:
        raise ValueError(f"{field} must be a bounded list")
    result: list[dict[str, object]] = []
    for relation in value:
        item = _closed_mapping(relation, {"relation", "subject_id", "object_id", "expectation"}, field)
        kind = _identity(item["relation"], f"{field}.relation")
        subject = _identity(item["subject_id"], f"{field}.subject_id")
        object_id = _identity(item["object_id"], f"{field}.object_id")
        expectation = _identity(item["expectation"], f"{field}.expectation")
        if kind == "bond_endpoint" and (subject not in bonds or object_id not in atoms):
            raise ValueError(f"{field} bond_endpoint identities are not in the fixture graph")
        if kind == "bond_style" and (subject not in bonds or object_id not in bonds):
            raise ValueError(f"{field} bond_style identities are not in the fixture graph")
        if kind == "nonendpoint_label" and (subject not in bonds or object_id not in atoms):
            raise ValueError(f"{field} nonendpoint_label identities are not in the fixture graph")
        if kind == "scene" and (subject != "scene" or object_id != "scene"):
            raise ValueError(f"{field} scene relation must use scene identities")
        if kind not in {"bond_endpoint", "bond_style", "scene", "nonendpoint_label"}:
            raise ValueError(f"{field} has an unsupported relation")
        result.append({"relation": kind, "subject_id": subject, "object_id": object_id, "expectation": expectation})
    return tuple(result)


# ============================================
def load_raster_manifest_v2(manifest_path: pathlib.Path) -> SceneLayers:
    """Load immutable V2 fixture/capture evidence with complete pixel-layer identity.

    V2 is intentionally noncircular: CDML identity, graph identity, fixed capture
    geometry, and expected visual relations come from the fixture, while every
    measurement input is final rendered pixels plus content hashes.
    """
    if not manifest_path.is_file() or manifest_path.stat().st_size > _MAX_MANIFEST_BYTES:
        raise ValueError("V2 manifest must be a bounded regular file")
    value = json.loads(
        manifest_path.read_text(encoding="utf-8"),
        object_pairs_hook=_reject_duplicate_keys,
        parse_constant=_reject_constant,
    )
    fields = {
        "schema", "fixture_id", "fixture_cdml_sha256", "capture_profile", "graph",
        "composite_layer", "atom_layers", "bond_layers", "expected_relations", "negative_cases",
    }
    item = _closed_mapping(value, fields, "V2 manifest")
    if item["schema"] != RASTER_LAYER_MANIFEST_V2_SCHEMA:
        raise ValueError("V2 manifest has an unsupported schema")
    fixture_id = _identity(item["fixture_id"], "fixture_id")
    cdml_hash = _sha256_value(item["fixture_cdml_sha256"], "fixture_cdml_sha256")
    profile = _v2_profile(item["capture_profile"])
    root = manifest_path.parent.resolve()
    graph = _closed_mapping(item["graph"], {"atoms", "bonds"}, "graph")
    if type(graph["atoms"]) is not list or not graph["atoms"] or len(graph["atoms"]) > _MAX_ATOMS:
        raise ValueError("graph.atoms must be a nonempty bounded list")
    if type(graph["bonds"]) is not list or len(graph["bonds"]) > _MAX_BONDS:
        raise ValueError("graph.bonds must be a bounded list")
    atom_graph: dict[str, str] = {}
    for row in graph["atoms"]:
        entry = _closed_mapping(row, {"atom_id", "element"}, "graph atom")
        atom_id = _identity(entry["atom_id"], "graph atom_id")
        if atom_id in atom_graph:
            raise ValueError("graph atom identities must be unique")
        atom_graph[atom_id] = _identity(entry["element"], "graph element")
    bond_graph: dict[str, tuple[str, str, str]] = {}
    for row in graph["bonds"]:
        entry = _closed_mapping(row, {"bond_id", "start_atom_id", "end_atom_id", "style"}, "graph bond")
        bond_id = _identity(entry["bond_id"], "graph bond_id")
        start = _identity(entry["start_atom_id"], "graph start_atom_id")
        end = _identity(entry["end_atom_id"], "graph end_atom_id")
        if bond_id in bond_graph or start == end or start not in atom_graph or end not in atom_graph:
            raise ValueError("graph bond identities or endpoints are invalid")
        bond_graph[bond_id] = (start, end, _identity(entry["style"], "graph bond style"))
    composite_path, _composite_hash = _hashed_layer(root, item["composite_layer"], "composite_layer")
    if type(item["atom_layers"]) is not list or len(item["atom_layers"]) != len(atom_graph):
        raise ValueError("atom_layers must cover every graph atom exactly once")
    atoms: dict[str, AtomLayer] = {}
    paths = [composite_path]
    for row in item["atom_layers"]:
        entry = _closed_mapping(row, {"atom_id", "core_glyph_layer", "full_label_layer"}, "atom layer")
        atom_id = _identity(entry["atom_id"], "atom layer atom_id")
        if atom_id not in atom_graph or atom_id in atoms:
            raise ValueError("atom_layers identities must exactly match graph atoms")
        core_path, _core_hash = _hashed_layer(root, entry["core_glyph_layer"], "core_glyph_layer")
        full_path, _full_hash = _hashed_layer(root, entry["full_label_layer"], "full_label_layer")
        core = _read_mask(core_path)
        full = _read_mask(full_path)
        if numpy.any(core & ~full):
            raise ValueError("core glyph ink must be contained in its full label mask")
        atoms[atom_id] = AtomLayer(atom_id, atom_graph[atom_id], core, full)
        paths.extend((core_path, full_path))
    if type(item["bond_layers"]) is not list or len(item["bond_layers"]) != len(bond_graph):
        raise ValueError("bond_layers must cover every graph bond exactly once")
    bonds: list[BondLayer] = []
    seen_bonds: set[str] = set()
    for row in item["bond_layers"]:
        entry = _closed_mapping(row, {"bond_id", "final_bond_layer"}, "bond layer")
        bond_id = _identity(entry["bond_id"], "bond layer bond_id")
        if bond_id not in bond_graph or bond_id in seen_bonds:
            raise ValueError("bond_layers identities must exactly match graph bonds")
        path, _digest = _hashed_layer(root, entry["final_bond_layer"], "final_bond_layer")
        start, end, style = bond_graph[bond_id]
        bonds.append(BondLayer(bond_id, start, end, style, _read_mask(path)))
        seen_bonds.add(bond_id)
        paths.append(path)
    if len(set(paths)) != len(paths):
        raise ValueError("each V2 layer must use a distinct image file")
    if sum(path.stat().st_size for path in paths) > _MAX_TOTAL_BYTES:
        raise ValueError("V2 image files exceed the total byte limit")
    composite = _read_composite(composite_path)
    layers = [composite, *(atom.core_mask for atom in atoms.values()), *(atom.full_label_mask for atom in atoms.values()), *(bond.footprint_mask for bond in bonds)]
    if any(layer is None or layer.shape != composite.shape for layer in layers):
        raise ValueError("all V2 image layers must have identical dimensions")
    if composite.shape != (profile.pixel_height, profile.pixel_width):
        raise ValueError("V2 composite dimensions must match the fixed capture profile")
    relations = _v2_relations(item["expected_relations"], set(atom_graph), set(bond_graph), "expected_relations")
    negative_cases = _v2_relations(item["negative_cases"], set(atom_graph), set(bond_graph), "negative_cases")
    return SceneLayers(
        RASTER_LAYER_MANIFEST_V2_SCHEMA, composite, atoms, tuple(bonds), profile.device_pixel_ratio,
        profile.pixel_width, profile.pixel_height, fixture_id, cdml_hash, profile, relations, negative_cases,
    )
