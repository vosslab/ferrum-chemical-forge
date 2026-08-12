#!/usr/bin/env python3
"""Compare the narrow ordered molecule-core carrier in isolated child processes."""

# Standard Library
import argparse
import hashlib
import json
import pathlib
import subprocess


TAG = "26.02a1"
COMMIT = "c6876626386e3e739c8e8a9f45c4348b2cdb926a"
CAPABILITY = "ordered-molecule-core-carrier-v1"
REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
ORACLE_PYTHON = REPO_ROOT / "tests" / "e2e" / "oracle" / ".venv" / "bin" / "python"
OASA_CHILD = REPO_ROOT / "tests" / "e2e" / "oracle" / "e2e_oasa_molecule_core_child.py"
DEFAULT_REPORT = REPO_ROOT / "docs" / "active_plans" / "reports" / "oracle_molecule_core.json"


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse the small set of operational harness controls."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"-p",
		"--oracle-python",
		dest="oracle_python",
		type=pathlib.Path,
		default=ORACLE_PYTHON,
		help="Python executable inside the isolated OASA environment",
	)
	parser.add_argument(
		"-r",
		"--report",
		dest="report_path",
		type=pathlib.Path,
		default=DEFAULT_REPORT,
		help="JSON report path",
	)
	parser.add_argument(
		"-m",
		"--mutate-rust",
		dest="mutate_rust",
		action="store_true",
		help="perform the intentional divergence diagnostic",
	)
	args = parser.parse_args()
	return args


#============================================
def make_request() -> dict:
	"""Return a compact input exercising ordered endpoints and shared fields."""
	request = {
		"capability": CAPABILITY,
		"atoms": [
			{"element": "C", "formal_charge": 0, "explicit_hydrogens": 2},
			{"element": "O", "formal_charge": 0, "explicit_hydrogens": 0},
			{"element": "N", "formal_charge": 1, "explicit_hydrogens": 1},
		],
		"bonds": [
			{"start": 0, "end": 1, "order": 2, "type": "n"},
			{"start": 2, "end": 0, "order": 1, "type": "w"},
		],
	}
	return request


#============================================
def child_result(command: list[str], request_text: str) -> dict:
	"""Run one worker and validate its one-object stdout protocol."""
	result = subprocess.run(
		command,
		cwd=REPO_ROOT,
		input=request_text,
		text=True,
		capture_output=True,
		check=False,
	)
	if result.returncode != 0:
		raise RuntimeError(
			"child exited " + str(result.returncode) + ": " + result.stderr.strip()
		)
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise RuntimeError("child stdout must contain exactly one JSON object")
	output = json.loads(lines[0])
	if not isinstance(output, dict):
		raise RuntimeError("child stdout JSON must be an object")
	return output


#============================================
def compare_values(path: str, left: object, right: object) -> list[dict]:
	"""Return exact, path-addressed differences without a tolerance."""
	differences = []
	if isinstance(left, dict) and isinstance(right, dict):
		keys = sorted(set(left) | set(right))
		for key in keys:
			child_path = path + "." + key
			if key not in left or key not in right:
				differences.append(
					{"path": child_path, "oasa": left.get(key), "ferrum": right.get(key)}
				)
			else:
				differences.extend(compare_values(child_path, left[key], right[key]))
	elif isinstance(left, list) and isinstance(right, list):
		limit = max(len(left), len(right))
		for index in range(limit):
			child_path = path + "[" + str(index) + "]"
			if index >= len(left) or index >= len(right):
				differences.append(
					{
						"path": child_path,
						"oasa": left[index] if index < len(left) else None,
						"ferrum": right[index] if index < len(right) else None,
					}
				)
			else:
				differences.extend(compare_values(child_path, left[index], right[index]))
	elif left != right:
		differences.append({"path": path, "oasa": left, "ferrum": right})
	return differences


#============================================
def make_report_base(request: dict, request_text: str, args: argparse.Namespace) -> dict:
	"""Return facts and scope shared by successful and failed runs."""
	report = {
		"capability": CAPABILITY,
		"oracle": {"tag": TAG, "commit": COMMIT},
		"request": request,
		"request_sha256": hashlib.sha256(request_text.encode("ascii")).hexdigest(),
		"commands": {
			"oasa": [str(args.oracle_python), str(OASA_CHILD)],
			"ferrum": [
				"cargo",
				"run",
				"--quiet",
				"--manifest-path",
				"packages/ferrum-rust/Cargo.toml",
				"--package",
				"ferrum-core",
				"--example",
				"oracle_molecule_core",
			],
		},
		"source_hierarchy": {
			"classification": "required-compatibility",
			"basis": "OASA observed construction behavior for this narrow carrier",
		},
		"scope_exclusions": [
			"CDML parsing, serialization, and opaque extension preservation",
			"coordinates, identifiers, isotope, valence, multiplicity, and free sites",
			"non-atom vertices, perceived/default chemistry, aromatic flags, and depiction",
			"OASA graph-set iteration order; comparison preserves request sequence only",
		],
		"comparison_rule": "Exact equality: all compared fields are discrete.",
		"intentional_mutation": args.mutate_rust,
	}
	return report


#============================================
def write_report(path: pathlib.Path, report: dict) -> None:
	"""Persist the complete machine-readable result."""
	path.parent.mkdir(parents=True, exist_ok=True)
	report_text = json.dumps(report, indent=2, sort_keys=True) + "\n"
	path.write_text(report_text, encoding="ascii")


#============================================
def main() -> None:
	"""Run OASA and Rust workers, write a report, and return the classified exit."""
	args = parse_args()
	request = make_request()
	request_text = json.dumps(request, separators=(",", ":"), sort_keys=True)
	report = make_report_base(request, request_text, args)
	if not args.oracle_python.is_file():
		report["status"] = "harness-error"
		report["error"] = "isolated OASA Python was not found"
		write_report(args.report_path, report)
		print(json.dumps(report, sort_keys=True))
		raise SystemExit(2)
	try:
		oasa_output = child_result(report["commands"]["oasa"], request_text)
		rust_command = report["commands"]["ferrum"].copy()
		if args.mutate_rust:
			rust_command.append("--")
			rust_command.append("--mutate-projection")
			report["commands"]["ferrum"] = rust_command
		ferrum_output = child_result(rust_command, request_text)
	except (json.JSONDecodeError, OSError, RuntimeError) as error:
		report["status"] = "harness-error"
		report["error"] = str(error)
		write_report(args.report_path, report)
		print(json.dumps(report, sort_keys=True))
		raise SystemExit(2) from error
	oasa_projection = oasa_output["projection"]
	ferrum_projection = ferrum_output["projection"]
	differences = compare_values("projection", oasa_projection, ferrum_projection)
	report["facts"] = {"oasa": oasa_output["facts"], "ferrum": ferrum_output["facts"]}
	report["normalized_outputs"] = {"oasa": oasa_projection, "ferrum": ferrum_projection}
	report["field_comparison"] = differences
	report["status"] = "match" if not differences else "divergence"
	write_report(args.report_path, report)
	print(json.dumps(report, sort_keys=True))
	if differences:
		raise SystemExit(1)


if __name__ == "__main__":
	main()
