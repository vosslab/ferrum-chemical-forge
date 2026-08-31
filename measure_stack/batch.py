"""Measure a directory of V2 raster manifests and publish one run summary.

This is an explicit developer/E2E entry point.  It keeps the independent
measurement library as the sole evaluator while providing the native Rust and
Qt capture lanes a common, machine-readable aggregate receipt.
"""

# Standard Library
import argparse
import json
import pathlib

# Local modules
from measure_stack.contracts import load_raster_manifest_v2
from measure_stack.diagnostics import write_diagnostics
from measure_stack.measure import MeasurementPolicy, measure_scene, violations


SUMMARY_SCHEMA = "ferrum-measure-stack-run-summary-v2"
MANIFEST_NAME = "raster_layer_manifest_v2.json"


# ============================================
def _category(violation: str) -> str:
    """Classify a policy violation without changing its measured detail."""
    if "detached" in violation:
        return "detached_endpoint"
    if "overlaps or touches target label" in violation or "collides with target full label" in violation:
        return "target_label_overlap"
    if "centerline" in violation:
        return "centerline_error"
    if "style topology" in violation:
        return "style_topology"
    if "non-endpoint full label" in violation:
        return "third_label_collision"
    if "misses its target-label neighborhood" in violation:
        return "missing_endpoint_connection"
    if "orphaned atom cores" in violation:
        return "orphaned_atom_core"
    if "cropped" in violation:
        return "cropped_scene"
    return "other"


# ============================================
def measure_manifest_tree(manifest_root: pathlib.Path) -> dict[str, object]:
    """Publish per-case diagnostics and aggregate fixed-policy evidence."""
    manifests = sorted(manifest_root.rglob(MANIFEST_NAME))
    if not manifests:
        raise ValueError(f"no {MANIFEST_NAME} files below {manifest_root}")
    rows: list[dict[str, object]] = []
    categories: dict[str, int] = {}
    for manifest_path in manifests:
        scene = load_raster_manifest_v2(manifest_path)
        report = measure_scene(scene)
        case_violations = violations(report, MeasurementPolicy())
        report["violations"] = case_violations
        measurement_directory = manifest_path.parent / "measurement"
        write_diagnostics(scene, report, measurement_directory)
        for violation in case_violations:
            category = _category(violation)
            categories[category] = categories.get(category, 0) + 1
        rows.append({
            "fixture_id": scene.fixture_id,
            "manifest": str(manifest_path.relative_to(manifest_root)),
            "measurement": str((measurement_directory / "measurement_report.json").relative_to(manifest_root)),
            "violations": case_violations,
        })
    summary = {
        "schema": SUMMARY_SCHEMA,
        "manifest_count": len(manifests),
        "fixture_count": len(rows),
        "violation_count": sum(len(row["violations"]) for row in rows),
        "failure_categories": dict(sorted(categories.items())),
        "fixtures": rows,
    }
    (manifest_root / "run_summary.json").write_text(
        json.dumps(summary, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )
    return summary


# ============================================
def main() -> int:
    """Run the aggregate V2 measurement lane over one manifest root."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest-root", required=True, type=pathlib.Path)
    parser.add_argument("--fail-on-violation", action="store_true")
    arguments = parser.parse_args()
    try:
        summary = measure_manifest_tree(arguments.manifest_root)
    except (OSError, ValueError, json.JSONDecodeError) as error:
        print(json.dumps({"status": "error", "error": str(error)}, sort_keys=True))
        return 1
    print(json.dumps({"status": "ok", "fixtures": summary["fixture_count"], "violations": summary["violation_count"]}, sort_keys=True))
    return 1 if arguments.fail_on_violation and summary["violation_count"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
