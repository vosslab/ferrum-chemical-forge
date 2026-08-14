#!/usr/bin/env python3
"""Measure the closed M12 Telex corpus with Ferrum and Qt QRawFont."""

import hashlib
import json
import os
import platform
import subprocess  # nosec B404
import sys
from pathlib import Path

import PySide6
import PySide6.QtGui
import PySide6.QtWidgets


EXPECTED_SHA256 = "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871"
EXPECTED_BYTES = 38940
CORPUS = ("C", "Cl", "Br", "H2", "NH3+", "I")


#============================================
def digest_file(path: Path) -> str:
	"""Return the SHA-256 digest of one regular local file."""
	contents = path.read_bytes()
	digest = hashlib.sha256(contents).hexdigest()
	return digest


#============================================
def command_output(command: list[str], root: Path) -> str:
	"""Run one local inspection command and return its standard output."""
	# Callers below supply only fixed, read-only Git commands from this module.
	completed = subprocess.run(  # nosec B603
		command, cwd=root, check=True, text=True, capture_output=True
	)
	output = completed.stdout
	return output


#============================================
def ferrum_rows(root: Path) -> list[dict]:
	"""Collect Ferrum's current closed-corpus design-metric rows."""
	command = [
		"cargo",
		"test",
		"-p",
		"ferrum-render",
		"metric_receipt_rows_cover_the_complete_pre_tolerance_corpus",
		"--",
		"--nocapture",
	]
	# This exact Cargo test is fixed local maintainer tooling, not external input.
	completed = subprocess.run(  # nosec B603
		command, cwd=root, check=False, text=True, capture_output=True
	)
	if completed.returncode:
		sys.stderr.write(completed.stderr)
		raise RuntimeError("Ferrum did not emit M12 design-metric rows")
	rows = []
	for line in completed.stdout.splitlines():
		if line.startswith("M12_METRIC_JSONL:"):
			rows.append(json.loads(line.removeprefix("M12_METRIC_JSONL:")))
	labels = tuple(row["label"] for row in rows)
	if labels != CORPUS:
		raise RuntimeError("Ferrum did not emit the complete ordered M12 corpus")
	return rows


#============================================
def rect_metrics(font: PySide6.QtGui.QRawFont, glyph_ids: list[int]) -> dict:
	"""Measure a glyph sequence in Qt's QRawFont coordinate convention."""
	advances = font.advancesForGlyphIndexes(glyph_ids)
	pen_x = 0.0
	min_x = None
	min_y = None
	max_x = None
	max_y = None
	for glyph_id, advance in zip(glyph_ids, advances, strict=True):
		rectangle = font.boundingRect(glyph_id)
		glyph_min_x = pen_x + rectangle.x()
		glyph_max_x = glyph_min_x + rectangle.width()
		glyph_min_y = rectangle.y()
		glyph_max_y = glyph_min_y + rectangle.height()
		if min_x is None or glyph_min_x < min_x:
			min_x = glyph_min_x
		if min_y is None or glyph_min_y < min_y:
			min_y = glyph_min_y
		if max_x is None or glyph_max_x > max_x:
			max_x = glyph_max_x
		if max_y is None or glyph_max_y > max_y:
			max_y = glyph_max_y
		pen_x += advance.x()
	if min_x is None or min_y is None or max_x is None or max_y is None:
		raise RuntimeError("the M12 corpus unexpectedly produced an empty glyph sequence")
	metrics = {
		"bearing": {"x": min_x, "y": min_y},
		"width": max_x - min_x,
		"height": max_y - min_y,
		"advance": {"x": pen_x, "y": 0.0},
	}
	return metrics


#============================================
def normalized_qt_metrics(
	font: PySide6.QtGui.QRawFont,
	text: str,
	size: float,
	scale: float,
) -> dict:
	"""Return Qt raw design metrics and the same metrics in Ferrum scene scale."""
	glyph_ids = font.glyphIndexesForString(text)
	if len(glyph_ids) != len(text):
		raise RuntimeError(f"Qt did not map every M12 scalar in {text!r}")
	raw_metrics = rect_metrics(font, glyph_ids)
	factor = size * scale / font.unitsPerEm()
	normalized = {
		"bearing": {
			"x": raw_metrics["bearing"]["x"] * factor,
			"y": raw_metrics["bearing"]["y"] * factor,
		},
		"width": raw_metrics["width"] * factor,
		"height": raw_metrics["height"] * factor,
		"advance": {
			"x": raw_metrics["advance"]["x"] * factor,
			"y": 0.0,
		},
	}
	result = {"glyph_ids": glyph_ids, "raw_design_units": raw_metrics, "normalized_scene": normalized}
	return result


#============================================
def normalized_qt_baseline(font: PySide6.QtGui.QRawFont, size: float) -> dict:
	"""Return Qt baseline metrics in raw design units and Ferrum scene scale."""
	raw_metrics = {
		"ascent": font.ascent(),
		"descent": font.descent(),
		"height": font.ascent() + font.descent() + font.leading(),
	}
	factor = size / font.unitsPerEm()
	normalized = {name: value * factor for name, value in raw_metrics.items()}
	result = {"raw_design_units": raw_metrics, "normalized_scene": normalized}
	return result


#============================================
def metric_deltas(ferrum: dict, qt: dict) -> dict:
	"""Return Qt-minus-Ferrum deltas for comparable run metrics."""
	metrics = {}
	for name in ("width", "height"):
		metrics[name] = qt["normalized_scene"][name] - ferrum[name]
	for name in ("bearing", "advance"):
		metrics[name] = {
			"x": qt["normalized_scene"][name]["x"] - ferrum[name]["x"],
			"y": qt["normalized_scene"][name]["y"] - ferrum[name]["y"],
		}
	return metrics


#============================================
def baseline_deltas(ferrum: dict, qt: dict) -> dict:
	"""Return Qt-minus-Ferrum deltas for one comparable baseline measurement."""
	deltas = {
		name: qt["normalized_scene"][name] - ferrum[name]
		for name in ("ascent", "descent", "height")
	}
	return deltas


#============================================
def maximum_delta(comparisons: list[dict]) -> dict:
	"""Find the greatest absolute observed normalized metric delta and its source."""
	observations = []
	for comparison in comparisons:
		baseline = comparison["baseline_comparison"]["delta_qt_minus_ferrum"]
		for metric, value in baseline.items():
			observations.append(
				{"label": comparison["label"], "run": "baseline", "metric": metric, "delta": value}
			)
		for run in comparison["runs"]:
			deltas = run["delta_qt_minus_ferrum"]
			for metric in ("width", "height"):
				observations.append(
					{
						"label": comparison["label"],
						"run": run["text"],
						"metric": metric,
						"delta": deltas[metric],
					}
				)
			for metric in ("bearing", "advance"):
				for axis, value in deltas[metric].items():
					observations.append(
						{
							"label": comparison["label"],
							"run": run["text"],
							"metric": metric + "." + axis,
							"delta": value,
						}
					)
	maximum = max(observations, key=lambda observation: abs(observation["delta"]))
	maximum["absolute_delta"] = abs(maximum["delta"])
	return maximum


#============================================
def main() -> int:
	"""Emit one reproducible, one-time M12 comparison receipt as JSON."""
	if not sys.dont_write_bytecode:
		raise RuntimeError(
			"bytecode writing is enabled; run `source source_me.sh && python3 -B "
			"devel/measure_m12_font_metrics.py`"
		)
	root = Path(__file__).resolve().parent.parent
	asset = root / "crates/render/assets/fonts/Telex-Regular.ttf"
	contents = asset.read_bytes()
	if len(contents) != EXPECTED_BYTES or hashlib.sha256(contents).hexdigest() != EXPECTED_SHA256:
		raise RuntimeError("the bundled Telex asset does not match the approved source digest")
	# The receipt measures font data only, so an offscreen Qt platform avoids a GUI dependency.
	if "QT_QPA_PLATFORM" not in os.environ:
		os.environ["QT_QPA_PLATFORM"] = "offscreen"
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	font = PySide6.QtGui.QRawFont(
		contents,
		1000.0,
		PySide6.QtGui.QFont.HintingPreference.PreferNoHinting,
	)
	if not font.isValid() or font.unitsPerEm() != 1000.0:
		raise RuntimeError("Qt QRawFont did not open the pinned Telex design face")
	rows = ferrum_rows(root)
	comparisons = []
	for row in rows:
		qt_baseline = normalized_qt_baseline(font, 12.0)
		baseline_comparison = {
			"ferrum": row["baseline"],
			"qt_qrawfont": qt_baseline,
			"delta_qt_minus_ferrum": baseline_deltas(row["baseline"], qt_baseline),
		}
		comparison_runs = []
		for ferrum_run in row["runs"]:
			qt_run = normalized_qt_metrics(
				font,
				ferrum_run["text"],
				ferrum_run["size"],
				ferrum_run["scale"],
			)
			ferrum_glyph_ids = [glyph["id"] for glyph in ferrum_run["glyphs"]]
			comparison_runs.append(
				{
					"text": ferrum_run["text"],
					"ferrum": ferrum_run,
					"qt_qrawfont": qt_run,
					"glyph_ids_equal": qt_run["glyph_ids"] == ferrum_glyph_ids,
					"delta_qt_minus_ferrum": metric_deltas(ferrum_run, qt_run),
				}
			)
		comparisons.append(
			{
				"label": row["label"],
				"ferrum_row": row,
				"baseline_comparison": baseline_comparison,
				"runs": comparison_runs,
			}
		)
	qt_gui = Path(PySide6.QtGui.__file__).resolve()
	qt_framework = qt_gui.parent / "Qt/lib/QtGui.framework/Versions/A/QtGui"
	receipt = {
		"schema": "ferrum-m12-qrawfont-design-metric-receipt-v1",
		"scope": "one-time macOS receipt for the closed Telex V1 corpus; not a CI gate",
		"comparison_status": "observations_only_no_tolerance_or_pass_claim",
		"platform": {"system": platform.platform(), "architecture": platform.machine()},
		"python": {"implementation": platform.python_implementation(), "version": sys.version},
		"asset": {
			"path": str(asset),
			"bytes": len(contents),
			"sha256": hashlib.sha256(contents).hexdigest(),
			"license_path": str(root / "crates/render/assets/licenses/Telex-OFL-1.1.txt"),
		},
		"ferrum": {
			"git_revision": command_output(["git", "rev-parse", "HEAD"], root).strip(),
			"git_status_porcelain_sha256": hashlib.sha256(
				command_output(["git", "status", "--porcelain=v1"], root).encode()
			).hexdigest(),
			"git_head_diff_sha256": hashlib.sha256(
				command_output(["git", "diff", "HEAD", "--binary"], root).encode()
			).hexdigest(),
			"cargo_lock_sha256": digest_file(root / "Cargo.lock"),
			"metric_backend": "ttf-parser-design-v1",
		},
		"independent_reference": {
			"name": "Qt QRawFont",
			"not_freetype": True,
			"pyside6_version": PySide6.__version__,
			"qt_version": PySide6.QtCore.qVersion(),
			"python_module": str(qt_gui),
			"python_module_sha256": digest_file(qt_gui),
			"qt_gui_binary": str(qt_framework),
			"qt_gui_binary_sha256": digest_file(qt_framework),
			"load_flags": {
				"pixel_size": 1000.0,
				"hinting_preference": "PreferNoHinting",
				"font_index": 0,
				"application_platform": os.environ["QT_QPA_PLATFORM"],
				"source": "pinned Telex bytes passed directly to QRawFont",
			},
			"units_per_em": font.unitsPerEm(),
		},
		"corpus": list(CORPUS),
		"corpus_sha256": hashlib.sha256("\\n".join(CORPUS).encode()).hexdigest(),
		"comparisons": comparisons,
		"observed_maximum_absolute_normalized_metric_delta": maximum_delta(comparisons),
	}
	print(json.dumps(receipt, sort_keys=True, separators=(",", ":")))
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
