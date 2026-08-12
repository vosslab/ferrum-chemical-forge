#!/usr/bin/env python3
"""Record the macOS Cairo/FreeType Telex pre-tolerance measurement receipt."""

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path


EXPECTED_SHA256 = "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871"
EXPECTED_BYTES = 38940
CORPUS = ("C", "Cl", "Br", "H2", "NH3+", "I")


def main() -> int:
	root = Path(__file__).resolve().parent.parent
	asset = root / "crates/render/assets/fonts/Telex-Regular.ttf"
	contents = asset.read_bytes()
	if len(contents) != EXPECTED_BYTES or hashlib.sha256(contents).hexdigest() != EXPECTED_SHA256:
		raise RuntimeError("the bundled Telex asset does not match the approved source digest")
	command = [
		"cargo",
		"test",
		"-p",
		"ferrum-render",
		"font_environment",
		"--",
		"--nocapture",
	]
	completed = subprocess.run(command, cwd=root, check=False, text=True, capture_output=True)
	receipt = {
		"schema": "ferrum-m12-cairo-pre-tolerance-receipt-v1",
		"font_id": "ferrum-telex-regular-v1",
		"sha256": EXPECTED_SHA256,
		"bytes": EXPECTED_BYTES,
		"hint_style": "none",
		"hint_metrics": "off",
		"platform": platform.platform(),
		"architecture": platform.machine(),
		"corpus": CORPUS,
		"cairo_freetype_metric_test_exit": completed.returncode,
		"status": "pre_tolerance_only",
	}
	print(json.dumps(receipt, sort_keys=True))
	if completed.returncode:
		sys.stderr.write(completed.stderr)
	return completed.returncode


if __name__ == "__main__":
	raise SystemExit(main())
