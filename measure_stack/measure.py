"""Independent, pixel-only V2 visual-quality metrics for Ferrum bond ink.

The fixture graph supplies only identities. All geometry below is recovered from
final pixels; renderer-issued bounds, axes, and clipped endpoints are excluded.
"""

# Standard Library
import dataclasses
import math
from collections.abc import Mapping

# PIP3 modules
import cv2
import numpy

# Local modules
from measure_stack.contracts import BondLayer, SceneLayers


REPORT_SCHEMA = "ferrum-measure-stack-report-v2"
_LANE_STYLES = {"normal": 1, "bold": 1, "double": 2, "triple": 3}
_HAWORTH_FRONT_STROKE_STYLES = frozenset(("haworth-front-stroke",))
_HAWORTH_FRONT_WEDGE_STYLE = "haworth-front-wedge"


@dataclasses.dataclass(frozen=True)
class MeasurementPolicy:
    """Fixed normal-scale policy; a currently rendered image cannot tune it."""

    min_gap_strokes: float = 0.20
    min_parallel_gap_strokes: float = 0.60
    max_gap_strokes: float = 1.75
    max_gap_glyph_height: float = 0.22
    max_perpendicular_glyph_height: float = 0.12
    max_own_collision_pixels: int = 0
    max_third_collision_pixels: int = 0
    min_footprint_coverage: float = 0.995
    min_scene_occupancy: float = 0.015
    max_scene_occupancy: float = 0.33
    min_scene_margin_fraction: float = 0.025
    min_dominant_axis_occupancy: float = 0.10
    max_unexplained_foreground_fraction: float = 0.005
    max_missing_declared_fraction: float = 0.005
    max_orphaned_atom_cores: int = 0


# ============================================
def _bbox(mask: numpy.ndarray, label: str) -> tuple[int, int, int, int]:
    """Return inclusive ink bounds and reject empty target masks."""
    rows, columns = numpy.where(mask)
    if len(rows) == 0:
        raise ValueError(f"empty ink mask: {label}")
    return int(columns.min()), int(rows.min()), int(columns.max()), int(rows.max())


# ============================================
def _center(mask: numpy.ndarray, label: str) -> numpy.ndarray:
    """Use isolated target-character ink, never a complete label box."""
    left, top, right, bottom = _bbox(mask, label)
    return numpy.array(((top + bottom) / 2.0, (left + right) / 2.0), dtype=numpy.float64)


# ============================================
def _glyph_height(mask: numpy.ndarray, label: str) -> float:
    """Return visible target-character height in final-image pixels."""
    _, top, _, bottom = _bbox(mask, label)
    return float(bottom - top + 1)


# ============================================
def _full_label_mask(atom: object) -> numpy.ndarray:
    """Return mandatory complete label ink for collision and scene evidence."""
    return atom.full_label_mask


# ============================================
def _stroke_width(mask: numpy.ndarray) -> float:
    """Estimate final ink width from the interior distance field."""
    distance = cv2.distanceTransform(mask.astype(numpy.uint8), cv2.DIST_L2, 5)
    interior = distance[distance >= 1.0]
    return max(float(2.0 * numpy.percentile(interior, 75.0)), 1.0) if len(interior) else 1.0


# ============================================
def _axis(target: numpy.ndarray, other: numpy.ndarray) -> tuple[numpy.ndarray, float]:
    """Recover the directed endpoint axis solely from target-core pixels."""
    vector = other - target
    length = float(numpy.linalg.norm(vector))
    if not math.isfinite(length) or length <= 1.0:
        raise ValueError("degenerate target-core centers cannot define an endpoint direction")
    return vector / length, length


# ============================================
def _endpoint_pixels(footprint: numpy.ndarray, target: numpy.ndarray, other: numpy.ndarray) -> tuple[numpy.ndarray, numpy.ndarray, numpy.ndarray]:
    """Select final ink local to a directed endpoint and return projections."""
    unit, length = _axis(target, other)
    rows, columns = numpy.where(footprint)
    points = numpy.column_stack((rows, columns)).astype(numpy.float64)
    projections = (points - target) @ unit
    local = (projections >= -1.0) & (projections <= length * 0.35)
    if not numpy.any(local):
        local = projections <= length * 0.5
    return points[local], unit, projections[local]


# ============================================
def _endpoint_metrics(footprint: numpy.ndarray, core: numpy.ndarray, full: numpy.ndarray, target: numpy.ndarray, other: numpy.ndarray, combine_parallel_lanes: bool) -> tuple[float, float, int, int]:
    """Measure directed gap and local medial-axis deviation from final pixels."""
    points, unit, projections = _endpoint_pixels(footprint, target, other)
    core_overlap = int(numpy.count_nonzero(footprint & core))
    full_overlap = int(numpy.count_nonzero(footprint & full))
    if len(points) == 0:
        return math.inf, math.inf, core_overlap, full_overlap
    rows, columns = numpy.where(core)
    core_edge = float(numpy.max((numpy.column_stack((rows, columns)) - target) @ unit))
    # Recover the endpoint neighborhood from every visible lane. A double or
    # triple bond is structurally attached by the midpoint of its lanes, not
    # by whichever offset lane happens to clip closest to one glyph pixel.
    # This remains pixel-only: connected final-ink components supply lane
    # identity and the target core supplies the reference edge.
    _count, components = cv2.connectedComponents(footprint.astype(numpy.uint8), connectivity=8)
    # Attachment direction is a target-neighborhood fact. A wavy bond must
    # retain that centerline at its endpoint while intentionally departing
    # later along its visible span; measuring a large fraction of the bond
    # would turn that legitimate style into false optical-anchor drift.
    endpoint_band = max(1.0, _stroke_width(footprint))
    lane_ids = components[points[:, 0].astype(int), points[:, 1].astype(int)]
    if not combine_parallel_lanes:
        lane_ids = numpy.ones_like(lane_ids)
    lane_starts = []
    lane_perpendicular_centers = []
    for lane_id in numpy.unique(lane_ids):
        lane_projections = projections[lane_ids == lane_id]
        lane_start = float(numpy.min(lane_projections))
        lane_starts.append(lane_start)
        lane_points = points[(lane_ids == lane_id) & (projections <= lane_start + endpoint_band)]
        delta = lane_points - target
        signed_perpendicular = delta[:, 0] * unit[1] - delta[:, 1] * unit[0]
        # A lane is one structural vote even when antialiasing, clipping, or a
        # different terminal cap gives it more raster pixels than its sibling.
        # Weighting every pixel moved an otherwise centered double bond toward
        # the thicker lane, which is precisely the renderer fact this
        # independent metric must avoid inventing.
        lane_perpendicular_centers.append(float(numpy.median(signed_perpendicular)))
    gap = -float(full_overlap) if full_overlap else float(numpy.median(lane_starts) - core_edge)
    return gap, float(abs(numpy.median(lane_perpendicular_centers))), core_overlap, full_overlap


# ============================================
def _component_count(mask: numpy.ndarray) -> int:
    """Count eight-connected ink regions."""
    count, _ = cv2.connectedComponents(mask.astype(numpy.uint8), connectivity=8)
    return int(count - 1)


# ============================================
def _cross_section(mask: numpy.ndarray, start: numpy.ndarray, end: numpy.ndarray, fraction: float) -> list[bool]:
    """Sample a perpendicular raster section at a stable axial fraction."""
    unit, length = _axis(start, end)
    normal = numpy.array((-unit[1], unit[0]))
    point = start + unit * length * fraction
    radius = max(8, int(length * 0.30))
    result: list[bool] = []
    for offset in range(-radius, radius + 1):
        row, column = numpy.rint(point + normal * offset).astype(int)
        result.append(0 <= row < mask.shape[0] and 0 <= column < mask.shape[1] and bool(mask[row, column]))
    return result


# ============================================
def _lane_count(values: list[bool]) -> int:
    """Count contiguous ink runs in one normal cross-section."""
    return sum(value and not prior for value, prior in zip(values, [False, *values]))


# ============================================
def _footprint_terminal_widths(mask: numpy.ndarray, start: numpy.ndarray, end: numpy.ndarray) -> tuple[int, int]:
    """Measure both wedge terminals within the visible footprint, not the atom span."""
    unit, axis_length = _axis(start, end)
    rows, columns = numpy.where(mask)
    points = numpy.column_stack((rows, columns)).astype(numpy.float64)
    if len(points) == 0:
        return 0, 0
    projections = (points - start) @ unit
    first = float(projections.min())
    final = float(projections.max())
    span = final - first
    if not math.isfinite(span) or span <= 1.0:
        return 0, 0
    # The 20/80 percent terminal sections stay inside final ink even when
    # clipping shortens a wedge well before its opposite atom-center anchor.
    fractions = tuple((first + span * portion) / axis_length for portion in (0.20, 0.80))
    return tuple(sum(_cross_section(mask, start, end, fraction)) for fraction in fractions)


# ============================================
def _wavy_turns(mask: numpy.ndarray, start: numpy.ndarray, end: numpy.ndarray) -> int:
    """Count lateral-direction reversals in a connected final wavy footprint."""
    unit, length = _axis(start, end)
    normal = numpy.array((-unit[1], unit[0]))
    rows, columns = numpy.where(mask)
    points = numpy.column_stack((rows, columns)).astype(numpy.float64)
    longitudinal = (points - start) @ unit
    lateral = (points - start) @ normal
    segment_count = max(4, int(length // 3))
    samples = []
    for index in range(segment_count):
        values = lateral[(longitudinal >= length * index / segment_count) & (longitudinal < length * (index + 1) / segment_count)]
        if len(values):
            samples.append(float(numpy.median(values)))
    signs = [int(numpy.sign(value)) for value in numpy.diff(samples) if abs(value) >= 0.5]
    return sum(left != right for left, right in zip(signs, signs[1:]))


# ============================================
def _dashed_component_receipt(mask: numpy.ndarray, start: numpy.ndarray,
        end: numpy.ndarray, stroke: float) -> dict[str, object]:
    """Classify separated, collinear, dash-shaped final-ink components."""
    unit, axis_length = _axis(start, end)
    normal = numpy.array((-unit[1], unit[0]))
    count, labels = cv2.connectedComponents(mask.astype(numpy.uint8), connectivity=8)
    axial_intervals = []
    transverse_centers = []
    elongations = []
    for component in range(1, count):
        rows, columns = numpy.where(labels == component)
        points = numpy.column_stack((rows, columns)).astype(numpy.float64)
        axial = (points - start) @ unit
        transverse = (points - start) @ normal
        axial_span = float(axial.max() - axial.min())
        transverse_span = float(transverse.max() - transverse.min())
        axial_intervals.append((float(axial.min()), float(axial.max())))
        transverse_centers.append(float(abs(numpy.median(transverse))))
        elongations.append(axial_span / max(1.0, transverse_span))
    visible_fraction = (
        (max(final for _, final in axial_intervals)
        - min(first for first, _ in axial_intervals)) / axis_length
        if axial_intervals else 0.0
    )
    distributed = bool(
        axial_intervals
        and min(first for first, _ in axial_intervals) < axis_length * 0.40
        and max(final for _, final in axial_intervals) > axis_length * 0.60
    )
    passed = (
        len(axial_intervals) >= 2
        and distributed
        and visible_fraction >= 0.50
        and min(elongations, default=0.0) >= 1.25
        and max(transverse_centers, default=math.inf) <= stroke
    )
    return {
        "dashed_components_distributed": distributed,
        "dashed_min_component_elongation": min(elongations, default=0.0),
        "dashed_max_centerline_error_px": max(transverse_centers, default=math.inf),
        "dashed_visible_axis_fraction": visible_fraction,
        "dashed_component_topology_pass": passed,
    }


# ============================================
def _style_topology(bond: BondLayer, start: numpy.ndarray, end: numpy.ndarray) -> dict[str, object]:
    """Apply explicit final-ink predicates for every supported bond style."""
    mask = bond.footprint_mask
    components = _component_count(mask)
    midpoint = _cross_section(mask, start, end, 0.50)
    narrow = _cross_section(mask, start, end, 0.20)
    wide = _cross_section(mask, start, end, 0.80)
    lanes = _lane_count(midpoint)
    narrow_width = sum(narrow)
    wide_width = sum(wide)
    taper_ratio = float(wide_width / max(1, narrow_width))
    terminal_a_width, terminal_b_width = _footprint_terminal_widths(mask, start, end)
    expansion_width = max(terminal_a_width, terminal_b_width)
    tip_width = min(terminal_a_width, terminal_b_width)
    stroke = _stroke_width(mask)
    style = bond.style
    predicate = "unsupported_style"
    passed = False
    style_receipt: dict[str, object] = {}
    if style in _LANE_STYLES:
        predicate = f"{style}_lane_count"
        # Parallel double/triple strokes are deliberately separate components;
        # a single stroke remains one component.
        passed = components == _LANE_STYLES[style] and lanes == _LANE_STYLES[style]
        if style == "bold":
            predicate = "bold_single_lane_thick_stroke"
            passed = passed and stroke >= 3.0
    elif style == "dashed":
        predicate = "dashed_separated_segments"
        # Component geometry is independent of renderer-selected dash phase and
        # device-pixel sampling. Each isolated final-ink component must be an
        # elongated dash, share the atom centerline, and span both axis halves.
        style_receipt = _dashed_component_receipt(mask, start, end, stroke)
        passed = bool(style_receipt["dashed_component_topology_pass"])
    elif style == "wavy":
        predicate = "wavy_connected_lateral_turns"
        passed = components == 1 and _wavy_turns(mask, start, end) >= 2
    elif style == "solid-wedge":
        predicate = "solid_wedge_connected_expansion"
        # Source order identifies the two atom endpoints, not the stereochemical
        # wide end.  Final pixels must show one connected widening footprint in
        # either direction; endpoint attachment metrics separately judge both
        # target labels.
        passed = components == 1 and expansion_width >= max(3, 2 * tip_width)
    elif style == "hashed-wedge":
        predicate = "hashed_wedge_expanding_segments"
        _count, labels, stats, _centroids = cv2.connectedComponentsWithStats(mask.astype(numpy.uint8), connectivity=8)
        heights = [int(row[cv2.CC_STAT_HEIGHT]) for row in stats[1:]]
        passed = components >= 3 and min(heights, default=0) > 0 and max(heights, default=0) >= 2 * min(heights)
    elif style in _HAWORTH_FRONT_STROKE_STYLES:
        predicate = "haworth_front_stroke_connected_constant_width"
        passed = components == 1 and lanes == 1 and stroke >= 2.0 and taper_ratio <= 1.5
    elif style == _HAWORTH_FRONT_WEDGE_STYLE:
        predicate = "haworth_front_wedge_connected_expansion"
        passed = components == 1 and expansion_width >= max(3, 2 * tip_width)
    result = {
        "predicate": predicate,
        "components": components,
        "midpoint_lane_count": lanes,
        "stroke_width_px": stroke,
        "narrow_cross_section_px": narrow_width,
        "wide_cross_section_px": wide_width,
        "terminal_a_cross_section_px": terminal_a_width,
        "terminal_b_cross_section_px": terminal_b_width,
        "tip_cross_section_px": tip_width,
        "expansion_cross_section_px": expansion_width,
        "taper_ratio": taper_ratio,
        "style_topology_pass": passed,
    }
    result.update(style_receipt)
    return result


# ============================================
def _endpoint_report(scene: SceneLayers, bond: BondLayer, target_id: str, other_id: str) -> dict[str, object]:
    """Core pixels select attachment; full label pixels own all collision checks."""
    target = scene.atoms[target_id]
    other = scene.atoms[other_id]
    core = target.core_mask
    full = _full_label_mask(target)
    target_center = _center(core, target_id)
    other_center = _center(other.core_mask, other_id)
    gap, perpendicular, core_collision, full_collision = _endpoint_metrics(
        bond.footprint_mask, core, full, target_center, other_center,
        bond.style in {"double", "triple"},
    )
    third_collision = sum(int(numpy.count_nonzero(bond.footprint_mask & _full_label_mask(atom))) for atom_id, atom in scene.atoms.items() if atom_id not in {target_id, other_id})
    height = _glyph_height(core, target_id)
    stroke = _stroke_width(bond.footprint_mask)
    return {"target_atom": target_id, "target_element": target.element, "glyph_height_px": height, "bond_stroke_width_px": stroke, "signed_gap_px": gap, "signed_gap_strokes": gap / stroke, "signed_gap_glyph_height": gap / height, "perpendicular_error_px": perpendicular, "perpendicular_error_glyph_height": perpendicular / height, "own_core_collision_pixels": core_collision, "own_full_label_collision_pixels": full_collision, "third_full_label_collision_pixels": third_collision}


# ============================================
def _endpoint_neighborhood(footprint: numpy.ndarray, target: numpy.ndarray, target_center: numpy.ndarray, other_center: numpy.ndarray) -> dict[str, object]:
    """Diagnose declared endpoint proximity with full-label pixels."""
    points, _, _ = _endpoint_pixels(footprint, target_center, other_center)
    if len(points) == 0:
        return {"connected": False, "nearest_distance_px": None, "maximum_distance_px": 0.0}
    # This is a connection diagnostic, so its neighborhood must accommodate
    # the declared final footprint. Capping it at 16 pixels made a compliant
    # bold endpoint appear orphaned solely because its 12-pixel stroke has a
    # larger radial distance to the full target label.
    maximum = max(2.0, 2.0 * _stroke_width(footprint), 0.25 * _glyph_height(target, "target label"))
    distance = cv2.distanceTransform((~target).astype(numpy.uint8), cv2.DIST_L2, 5)
    nearest = float(min(distance[int(row), int(column)] for row, column in points))
    return {"connected": nearest <= maximum, "nearest_distance_px": nearest, "maximum_distance_px": maximum}


# ============================================
def _composition(scene: SceneLayers) -> dict[str, object] | None:
    """Measure compositing integrity and, when applicable, presentation framing."""
    foreground = scene.composite
    left, top, right, bottom = _bbox(foreground, "composite")
    height, width = foreground.shape
    margins = {"left": left / width, "top": top / height, "right": (width - right - 1) / width, "bottom": (height - bottom - 1) / height}
    expected = numpy.zeros_like(foreground)
    for atom in scene.atoms.values():
        expected |= _full_label_mask(atom)
    for bond in scene.bonds:
        expected |= bond.footprint_mask
    unexpected = foreground & ~expected
    missing = expected & ~foreground
    connections: list[dict[str, object]] = []
    connected: set[str] = set()
    degrees = {atom_id: 0 for atom_id in scene.atoms}
    for bond in scene.bonds:
        degrees[bond.start_atom] += 1
        degrees[bond.end_atom] += 1
        for target_id, other_id in ((bond.start_atom, bond.end_atom), (bond.end_atom, bond.start_atom)):
            atom = scene.atoms[target_id]
            relation = _endpoint_neighborhood(bond.footprint_mask, _full_label_mask(atom), _center(atom.core_mask, target_id), _center(scene.atoms[other_id].core_mask, other_id))
            connections.append({"bond_id": bond.bond_id, "target_atom": target_id, **relation})
            if relation["connected"]:
                connected.add(target_id)
    missing_endpoints = [f"{row['bond_id']}:{row['target_atom']}" for row in connections if not row["connected"]]
    orphaned = sorted(atom_id for atom_id, degree in degrees.items() if degree and atom_id not in connected)
    scene_evaluation = scene.capture_profile.scene_evaluation if scene.capture_profile else "presentation"
    axis_occupancy = {
        "horizontal": 1.0 - margins["left"] - margins["right"],
        "vertical": 1.0 - margins["top"] - margins["bottom"],
    }
    return {"scene_evaluation": scene_evaluation, "occupancy_fraction": float(numpy.count_nonzero(foreground) / foreground.size), "axis_occupancy_fraction": axis_occupancy, "dominant_axis_occupancy_fraction": float(max(axis_occupancy.values())), "margins_fraction": margins, "minimum_margin_fraction": float(min(margins.values())), "declared_ink_components": _component_count(expected), "composite_ink_components": _component_count(foreground), "expected_layer_pixels": int(numpy.count_nonzero(expected)), "unexplained_foreground_pixels": int(numpy.count_nonzero(unexpected)), "unexplained_foreground_fraction": float(numpy.count_nonzero(unexpected) / foreground.size), "missing_declared_pixels": int(numpy.count_nonzero(missing)), "missing_declared_fraction": float(numpy.count_nonzero(missing) / max(1, numpy.count_nonzero(expected))), "expected_endpoint_connections": connections, "missing_expected_endpoint_connections": missing_endpoints, "orphaned_atom_cores": orphaned}


# ============================================
def measure_scene(scene: SceneLayers) -> dict[str, object]:
    """Produce deterministic V2 final-ink metrics for one validated scene."""
    bonds: list[dict[str, object]] = []
    for bond in scene.bonds:
        pixels = int(numpy.count_nonzero(bond.footprint_mask))
        coverage = float(numpy.count_nonzero(bond.footprint_mask & scene.composite) / pixels) if pixels else 0.0
        start = _center(scene.atoms[bond.start_atom].core_mask, bond.start_atom)
        end = _center(scene.atoms[bond.end_atom].core_mask, bond.end_atom)
        bonds.append({"bond_id": bond.bond_id, "style": bond.style, "final_footprint_pixels": pixels, "final_footprint_coverage": coverage, "connectivity_components": _component_count(bond.footprint_mask), "style_topology": _style_topology(bond, start, end), "endpoints": [_endpoint_report(scene, bond, bond.start_atom, bond.end_atom), _endpoint_report(scene, bond, bond.end_atom, bond.start_atom)]})
    return {"schema": REPORT_SCHEMA, "input_schema": scene.schema, "pixel_scale": scene.pixel_scale, "bonds": bonds, "composition": _composition(scene)}


# ============================================
def violations(report: Mapping[str, object], policy: MeasurementPolicy) -> list[str]:
    """Return explicit V2 violations; nonfinite metrics always fail before JSON null."""
    failures: list[str] = []
    for bond in report["bonds"]:
        bond_id = bond["bond_id"]
        if bond["final_footprint_pixels"] == 0:
            failures.append(f"{bond_id}: final footprint is empty")
        if bond["final_footprint_coverage"] < policy.min_footprint_coverage:
            failures.append(f"{bond_id}: final footprint is absent from composite")
        if not bond["style_topology"]["style_topology_pass"]:
            failures.append(f"{bond_id}: final footprint style topology is invalid")
        for endpoint in bond["endpoints"]:
            label = f"{bond_id}:{endpoint['target_atom']}"
            names = [name for name, value in endpoint.items() if name.endswith(("_px", "_strokes", "_glyph_height")) and isinstance(value, (int, float))]
            nonfinite = [name for name in names if not math.isfinite(endpoint[name])]
            if nonfinite:
                failures.append(f"{label}: measurement is nonfinite ({', '.join(nonfinite)})")
                continue
            minimum_gap = (
                policy.min_parallel_gap_strokes
                if bond["style"] in {"double", "triple"}
                else policy.min_gap_strokes
            )
            if endpoint["signed_gap_strokes"] < minimum_gap:
                failures.append(f"{label}: bond overlaps or touches target label")
            if endpoint["signed_gap_strokes"] > policy.max_gap_strokes or endpoint["signed_gap_glyph_height"] > policy.max_gap_glyph_height:
                failures.append(f"{label}: bond is visibly detached from target character")
            if endpoint["perpendicular_error_glyph_height"] > policy.max_perpendicular_glyph_height:
                failures.append(f"{label}: bond misses target-character centerline")
            if endpoint["own_full_label_collision_pixels"] > policy.max_own_collision_pixels:
                failures.append(f"{label}: bond collides with target full label")
            if endpoint["third_full_label_collision_pixels"] > policy.max_third_collision_pixels:
                failures.append(f"{label}: bond collides with non-endpoint full label")
    composition = report["composition"]
    if composition is not None:
        if composition["scene_evaluation"] == "presentation":
            if not policy.min_scene_occupancy <= composition["occupancy_fraction"] <= policy.max_scene_occupancy:
                failures.append("composition: scene occupancy is outside fixed normal-scale policy")
            if composition["minimum_margin_fraction"] < policy.min_scene_margin_fraction:
                failures.append("composition: foreground is cropped against viewport")
            if composition["dominant_axis_occupancy_fraction"] < policy.min_dominant_axis_occupancy:
                failures.append("composition: scene is visibly under-framed")
        if composition["unexplained_foreground_fraction"] > policy.max_unexplained_foreground_fraction:
            failures.append("composition: composite contains unexplained foreground ink")
        if composition["missing_declared_fraction"] > policy.max_missing_declared_fraction:
            failures.append("composition: declared final ink is missing from composite")
        if composition["missing_expected_endpoint_connections"]:
            failures.append("composition: declared bond endpoint misses its target-label neighborhood")
        if len(composition["orphaned_atom_cores"]) > policy.max_orphaned_atom_cores:
            failures.append("composition: declared molecule has orphaned atom cores")
    return failures
