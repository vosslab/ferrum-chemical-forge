"""Synthetic contract and image-metric tests for Ferrum's measurement stack."""

# Standard Library
import dataclasses
import json
import math
import pathlib

# PIP3 modules
import cv2
import numpy

# Local modules
from measure_stack.contracts import (
    AtomLayer,
    BondLayer,
    RASTER_LAYER_MANIFEST_V2_SCHEMA,
    SceneLayers,
)
from measure_stack.diagnostics import write_diagnostics
from measure_stack.measure import MeasurementPolicy, measure_scene, violations


# ============================================
def _style_mask(style: str, good: bool) -> numpy.ndarray:
    """Create one controlled final footprint for each supported V2 style predicate."""
    mask = numpy.zeros((100, 160), dtype=bool)
    if not good:
        if style == "bold":
            mask[49:51, 26:136] = True
        elif style == "normal":
            mask[42:45, 26:136] = True
            mask[54:57, 26:136] = True
        elif style == "double":
            mask[48:52, 26:136] = True
        elif style == "triple":
            mask[42:45, 26:136] = True
            mask[54:57, 26:136] = True
        elif style == "dashed":
            mask[48:52, 26:136] = True
        elif style in {
            "wavy",
            "solid-wedge",
            "hashed-wedge",
            "haworth-front-stroke",
            "haworth-front-wedge",
        }:
            mask[48:52, 26:136] = True
            if style == "haworth-front-stroke":
                mask[:, :] = False
                mask[48:52, 26:70] = True
                mask[48:52, 76:136] = True
        return mask
    if style == "normal":
        mask[48:52, 26:136] = True
    elif style == "bold":
        mask[45:55, 26:136] = True
    elif style == "double":
        mask[42:45, 26:136] = True
        mask[54:57, 26:136] = True
    elif style == "triple":
        mask[39:42, 26:136] = True
        mask[48:51, 26:136] = True
        mask[57:60, 26:136] = True
    elif style == "dashed":
        for left, right in ((26, 50), (61, 88), (99, 136)):
            mask[48:52, left:right] = True
    elif style == "wavy":
        points = numpy.array([
            (26, 50), (40, 43), (54, 57), (68, 43), (82, 57),
            (96, 43), (110, 57), (124, 43), (136, 50),
        ], dtype=numpy.int32)
        cv2.polylines(mask.view(numpy.uint8), [points], False, 1, thickness=3)
    elif style == "solid-wedge":
        cv2.fillConvexPoly(mask.view(numpy.uint8), numpy.array([(26, 49), (136, 37), (136, 63)], dtype=numpy.int32), 1)
    elif style == "hashed-wedge":
        for index, left in enumerate(range(28, 128, 18)):
            half_height = index + 1
            mask[50 - half_height:51 + half_height, left:left + 11] = True
    elif style == "haworth-front-stroke":
        mask[46:54, 26:136] = True
    elif style == "haworth-front-wedge":
        cv2.fillConvexPoly(mask.view(numpy.uint8), numpy.array([(26, 49), (136, 37), (136, 63)], dtype=numpy.int32), 1)
    else:
        raise AssertionError(f"test does not cover style {style}")
    return mask


# ============================================
def _style_scene(style: str, good: bool) -> SceneLayers:
    """Build a direct V2 scene with complete masks, independent of old manifests."""
    shape = (100, 160)
    core_a = numpy.zeros(shape, dtype=bool)
    core_b = numpy.zeros(shape, dtype=bool)
    core_a[40:60, 12:22] = True
    core_b[40:60, 138:148] = True
    atoms = {
        "a": AtomLayer("a", "C", core_a, core_a.copy()),
        "b": AtomLayer("b", "O", core_b, core_b.copy()),
    }
    footprint = _style_mask(style, good)
    return SceneLayers(
        RASTER_LAYER_MANIFEST_V2_SCHEMA,
        core_a | core_b | footprint,
        atoms,
        (BondLayer("bond", "a", "b", style, footprint),),
        1.0,
        shape[1],
        shape[0],
    )


# ============================================
def _reverse_footprint_direction(scene: SceneLayers) -> SceneLayers:
    """Mirror a wedge footprint while retaining its declared source endpoints."""
    bond = scene.bonds[0]
    footprint = numpy.fliplr(bond.footprint_mask)
    composite = footprint.copy()
    for atom in scene.atoms.values():
        composite |= atom.full_label_mask
    return dataclasses.replace(
        scene,
        composite=composite,
        bonds=(BondLayer(bond.bond_id, bond.start_atom, bond.end_atom, bond.style, footprint),),
    )


# ============================================
def test_wedge_topology_is_independent_of_declared_endpoint_order() -> None:
    """A wedge may widen toward either source endpoint without becoming invalid."""
    for style in ("solid-wedge", "haworth-front-wedge"):
        reverse = measure_scene(_reverse_footprint_direction(_style_scene(style, good=True)))
        assert reverse["bonds"][0]["style_topology"]["style_topology_pass"], style


# ============================================
def test_parallel_endpoint_axis_is_not_weighted_by_lane_raster_area() -> None:
    """A centered double bond remains centered when its lanes rasterize unequally."""
    scene = _style_scene("double", good=True)
    footprint = scene.bonds[0].footprint_mask.copy()
    # Deliberately thicken just the upper lane while preserving both lane
    # centerlines. Pixel-count aggregation would pull the measured attachment
    # off the axis; one median per connected lane must not.
    footprint[40:42, 26:136] = True
    footprint[57:61, 26:136] = True
    altered = dataclasses.replace(
        scene,
        composite=scene.composite | footprint,
        bonds=(dataclasses.replace(scene.bonds[0], footprint_mask=footprint),),
    )
    endpoints = measure_scene(altered)["bonds"][0]["endpoints"]
    assert max(endpoint["perpendicular_error_glyph_height"] for endpoint in endpoints) < 0.03


# ============================================
def test_v2_style_predicates_accept_good_and_reject_deliberately_bad_final_ink() -> None:
    """Every supported style has a positive and adversarial raster topology oracle."""
    for style in (
        "normal",
        "double",
        "triple",
        "dashed",
        "bold",
        "wavy",
        "solid-wedge",
        "hashed-wedge",
        "haworth-front-stroke",
        "haworth-front-wedge",
    ):
        good = measure_scene(_style_scene(style, good=True))["bonds"][0]["style_topology"]
        bad = measure_scene(_style_scene(style, good=False))["bonds"][0]["style_topology"]
        assert good["style_topology_pass"], f"synthetic good {style}: {good}"
        assert not bad["style_topology_pass"], f"synthetic bad {style}: {bad}"


# ============================================
def test_haworth_front_forms_reject_each_others_final_ink_topology() -> None:
    """q1 foreground strokes and w1 foreground wedges are independently classified."""
    stroke_scene = _style_scene("haworth-front-stroke", good=True)
    wedge_scene = _style_scene("haworth-front-wedge", good=True)
    stroke_as_wedge = SceneLayers(
        stroke_scene.schema,
        stroke_scene.composite,
        stroke_scene.atoms,
        (BondLayer("bond", "a", "b", "haworth-front-wedge", stroke_scene.bonds[0].footprint_mask),),
        stroke_scene.pixel_scale,
        stroke_scene.viewport_width_px,
        stroke_scene.viewport_height_px,
    )
    wedge_as_stroke = SceneLayers(
        wedge_scene.schema,
        wedge_scene.composite,
        wedge_scene.atoms,
        (BondLayer("bond", "a", "b", "haworth-front-stroke", wedge_scene.bonds[0].footprint_mask),),
        wedge_scene.pixel_scale,
        wedge_scene.viewport_width_px,
        wedge_scene.viewport_height_px,
    )
    assert not measure_scene(stroke_as_wedge)["bonds"][0]["style_topology"]["style_topology_pass"]
    assert not measure_scene(wedge_as_stroke)["bonds"][0]["style_topology"]["style_topology_pass"]


# ============================================
def test_v2_uses_full_label_masks_for_collision_not_only_core_attachment() -> None:
    """A bond through a charge/decorative label pixel fails even when the core is clear."""
    scene = _style_scene("normal", good=True)
    shape = scene.composite.shape
    decorated_core = numpy.zeros(shape, dtype=bool)
    decorated_core[40:60, 76:84] = True
    full_label = decorated_core.copy()
    full_label[47:53, 100:108] = True
    atoms = {**scene.atoms, "third": AtomLayer("third", "N", decorated_core, full_label)}
    footprint = scene.bonds[0].footprint_mask.copy()
    footprint[48:52, 100:108] = True
    composite = scene.composite | decorated_core | full_label | footprint
    collision_scene = SceneLayers(
        RASTER_LAYER_MANIFEST_V2_SCHEMA,
        composite,
        atoms,
        (BondLayer("bond", "a", "b", "normal", footprint),),
        1.0,
        shape[1],
        shape[0],
    )
    report = measure_scene(collision_scene)
    endpoint = report["bonds"][0]["endpoints"][0]
    assert endpoint["third_full_label_collision_pixels"] > 0
    assert any("non-endpoint full label" in item for item in violations(report, MeasurementPolicy()))


# ============================================
def test_v2_composition_rejects_unexplained_foreground_and_underframing() -> None:
    """Fixed-profile composition rejects stray composite ink and excessive empty canvas."""
    scene = _style_scene("normal", good=True)
    composite = scene.composite.copy()
    composite[2:12, 2:12] = True
    altered = SceneLayers(
        RASTER_LAYER_MANIFEST_V2_SCHEMA,
        composite,
        scene.atoms,
        scene.bonds,
        scene.pixel_scale,
        scene.viewport_width_px,
        scene.viewport_height_px,
    )
    report = measure_scene(altered)
    assert report["composition"] is not None
    assert report["composition"]["unexplained_foreground_pixels"] == 100
    assert any("unexplained foreground" in item for item in violations(report, MeasurementPolicy()))


# ============================================
def test_raw_final_ink_profile_omits_viewport_framing_but_keeps_integrity_checks() -> None:
    """Native diagnostic rasters are geometry evidence, not user-facing framing."""
    scene = _style_scene("normal", good=True)
    profile = scene.capture_profile
    assert profile is None
    from measure_stack.contracts import CaptureProfile
    raw_scene = SceneLayers(
        scene.schema, scene.composite, scene.atoms, scene.bonds, scene.pixel_scale,
        scene.viewport_width_px, scene.viewport_height_px, capture_profile=CaptureProfile(
            "raw-test-v2", (-200.0, -200.0, 400.0, 400.0), 160, 100, 1.0, "raw_final_ink",
        ),
    )
    report = measure_scene(raw_scene)
    assert report["composition"]["scene_evaluation"] == "raw_final_ink"
    assert not any("scene occupancy" in item or "under-framed" in item for item in violations(report, MeasurementPolicy()))
    broken_composite = raw_scene.composite.copy()
    broken_composite[2:12, 2:12] = True
    broken = dataclasses.replace(raw_scene, composite=broken_composite)
    assert any("unexplained foreground" in item for item in violations(measure_scene(broken), MeasurementPolicy()))


# ============================================
def test_v2_composition_and_diagnostics_are_reported(tmp_path: pathlib.Path) -> None:
    """V2 scenes receive fixed-profile composition metrics and diagnostics."""
    scene = _style_scene("normal", good=True)
    report = measure_scene(scene)
    report["violations"] = violations(report, MeasurementPolicy())
    output = tmp_path / "output"
    write_diagnostics(scene, report, output)
    assert report["composition"] is not None
    assert (output / "measurement_report.json").is_file()
    assert (output / "contact_sheet.png").is_file()


def test_nonfinite_measurement_is_a_violation_and_writes_json_null(tmp_path: pathlib.Path) -> None:
    """Diagnostic JSON preserves a nonfinite failure category without NaN tokens."""
    scene = _style_scene("normal", good=True)
    report = measure_scene(scene)
    endpoint = report["bonds"][0]["endpoints"][0]
    endpoint["perpendicular_error_px"] = math.inf
    endpoint["perpendicular_error_glyph_height"] = math.inf
    report["violations"] = violations(report, MeasurementPolicy())
    assert any("measurement is nonfinite" in item for item in report["violations"])
    output = tmp_path / "nonfinite-output"
    write_diagnostics(scene, report, output)
    saved = json.loads((output / "measurement_report.json").read_text(encoding="utf-8"))
    saved_endpoint = saved["bonds"][0]["endpoints"][0]
    assert saved_endpoint["perpendicular_error_px"] is None
    assert saved_endpoint["perpendicular_error_glyph_height"] is None
