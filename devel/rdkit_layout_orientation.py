#!/usr/bin/env python3
"""Record the isolated RDKit reference layout orientation decision."""

# Standard Library
import argparse
import json
import math
import pathlib
import subprocess


CAPABILITY = "rdkit-layout-orientation-v1"
REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
ORACLE_PYTHON = REPO_ROOT / "tests" / "e2e" / "oracle" / ".venv" / "bin" / "python"
ORACLE_CHILD = REPO_ROOT / "devel" / "rdkit_layout_orientation_child.py"
ORACLE_REQUIREMENTS = pathlib.Path("tests/e2e/oracle/pip_requirements.txt")
DEFAULT_REPORT = (
	REPO_ROOT / "docs" / "active_plans" / "reports" / "rdkit_layout_orientation.json"
)


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse the isolated interpreter and report destination controls."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"-p",
		"--oracle-python",
		dest="oracle_python",
		type=pathlib.Path,
		default=ORACLE_PYTHON,
		help="Python executable inside the isolated RDKit oracle environment",
	)
	parser.add_argument(
		"-r",
		"--report",
		dest="report_path",
		type=pathlib.Path,
		default=DEFAULT_REPORT,
		help="JSON report path",
	)
	args = parser.parse_args()
	return args


#============================================
def child_result(oracle_python: pathlib.Path) -> dict:
	"""Run one oracle process and validate its exactly-one-object protocol."""
	command = [str(oracle_python), str(ORACLE_CHILD)]
	result = subprocess.run(
		command,
		cwd=REPO_ROOT,
		text=True,
		capture_output=True,
		check=False,
	)
	if result.returncode != 0:
		raise RuntimeError(
			"oracle child exited " + str(result.returncode) + ": " + result.stderr.strip()
		)
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise RuntimeError("oracle child stdout must contain exactly one JSON object")
	output = json.loads(lines[0])
	if not isinstance(output, dict):
		raise RuntimeError("oracle child stdout JSON must be an object")
	return output


#============================================
def validate_coordinates(coordinates: object, atom_count: int, label: str) -> list[dict]:
	"""Validate finite, complete, input-index-aligned coordinate records."""
	if not isinstance(coordinates, list) or len(coordinates) != atom_count:
		raise RuntimeError(label + " does not contain one coordinate for every atom")
	validated = []
	for index, coordinate in enumerate(coordinates):
		if not isinstance(coordinate, dict) or coordinate["index"] != index:
			raise RuntimeError(label + " coordinate indices are not input-aligned")
		values = {"index": index}
		for field in ("x", "y", "z"):
			value = coordinate[field]
			if isinstance(value, bool) or not isinstance(value, (int, float)):
				raise RuntimeError(label + " contains a non-numeric " + field + " coordinate")
			if not math.isfinite(value):
				raise RuntimeError(label + " contains a non-finite " + field + " coordinate")
			values[field] = value
		validated.append(values)
	return validated


#============================================
def validate_measurement(output: dict) -> dict:
	"""Validate both modes and return the normalized, comparison-ready facts."""
	if output["capability"] != CAPABILITY:
		raise RuntimeError("oracle child reported an unexpected capability")
	facts = output["facts"]
	if not isinstance(facts, dict):
		raise RuntimeError("oracle child facts must be an object")
	atom_count = facts["atom_count"]
	if isinstance(atom_count, bool) or not isinstance(atom_count, int) or atom_count < 1:
		raise RuntimeError("oracle child atom count must be a positive integer")
	measurements = output["measurements"]
	if not isinstance(measurements, list) or len(measurements) != 2:
		raise RuntimeError("oracle child must measure exactly two explicit orientations")
	by_orientation = {}
	for measurement in measurements:
		if not isinstance(measurement, dict):
			raise RuntimeError("oracle child measurement must be an object")
		orientation = measurement["canon_orient"]
		if not isinstance(orientation, bool) or orientation in by_orientation:
			raise RuntimeError("oracle child orientations must be unique booleans")
		label = "canonOrient=" + str(orientation)
		by_orientation[orientation] = validate_coordinates(
			measurement["coordinates"], atom_count, label,
		)
	if set(by_orientation) != {False, True}:
		raise RuntimeError("oracle child must explicitly measure canonOrient false and true")
	if by_orientation[False] == by_orientation[True]:
		raise RuntimeError("the asymmetric oracle molecule did not distinguish orientations")
	normalized = {
		"facts": facts,
		"coordinates": {
			"canon_orient_false": by_orientation[False],
			"canon_orient_true": by_orientation[True],
		},
	}
	return normalized


#============================================
def write_report(path: pathlib.Path, report: dict) -> None:
	"""Write one deterministic ASCII JSON report."""
	path.parent.mkdir(parents=True, exist_ok=True)
	report_text = json.dumps(report, indent=2, sort_keys=True) + "\n"
	path.write_text(report_text, encoding="ascii")


#============================================
def main() -> None:
	"""Measure the two oracle layouts once and persist the verified decision input."""
	args = parse_args()
	report = {
		"capability": CAPABILITY,
		"decision": (
			"Future Ferrum layout selects the reference-wrapper behavior by passing "
			"canonOrient=True explicitly."
		),
		"generator_command": "source source_me.sh && python3 devel/rdkit_layout_orientation.py",
		"generator_script": "devel/rdkit_layout_orientation.py",
		"oracle_command": [str(args.oracle_python), str(ORACLE_CHILD)],
		"oracle_requirements": str(ORACLE_REQUIREMENTS),
		"scope": (
			"Isolated RDKit oracle measurement only; it does not implement Ferrum "
			"layout or establish a coordinate tolerance."
		),
	}
	if not args.oracle_python.is_file():
		report["status"] = "harness-error"
		report["error"] = "isolated RDKit Python was not found"
		write_report(args.report_path, report)
		print(json.dumps(report, sort_keys=True))
		raise SystemExit(2)
	try:
		output = child_result(args.oracle_python)
		measurement = validate_measurement(output)
	except (json.JSONDecodeError, OSError, RuntimeError) as error:
		report["status"] = "harness-error"
		report["error"] = str(error)
		write_report(args.report_path, report)
		print(json.dumps(report, sort_keys=True))
		raise SystemExit(2) from error
	report["facts"] = measurement["facts"]
	report["coordinates"] = measurement["coordinates"]
	report["same_oracle_process"] = True
	report["orientations_diverge"] = True
	report["status"] = "measured"
	write_report(args.report_path, report)
	print(json.dumps(report, sort_keys=True))


if __name__ == "__main__":
	main()
