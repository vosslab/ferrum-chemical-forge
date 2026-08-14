"""Run Ferrum's structural CDML preservation gate over the committed corpus."""

# Standard Library
import argparse
import json
import pathlib
import subprocess


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
CORPUS_ROOT = REPO_ROOT / "tests" / "e2e" / "corpus"


#============================================
class CdmlPreservationE2eError(RuntimeError):
	"""Raised when a corpus document does not survive Ferrum's rewrite contract."""


#============================================
def _corpus_paths() -> tuple[pathlib.Path, ...]:
	"""Return every committed CDML corpus input in stable path order."""
	paths = tuple(sorted(CORPUS_ROOT.glob("*.cdml")))
	if not paths:
		raise CdmlPreservationE2eError("the committed CDML corpus is empty")
	return paths


#============================================
def _check_document(ferrum: pathlib.Path, source: pathlib.Path) -> dict[str, object]:
	"""Run the public structural rewrite check for one corpus document."""
	result = subprocess.run(
		[str(ferrum), "cdml", "rewrite", str(source), "--check"],
		cwd=REPO_ROOT, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise CdmlPreservationE2eError(
			"rewrite check failed for %s: %s" % (
				source.relative_to(REPO_ROOT), result.stderr.strip(),
			),
		)
	if result.stderr:
		raise CdmlPreservationE2eError(
			f"rewrite check wrote unexpected diagnostics for {source.name}",
		)
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise CdmlPreservationE2eError(
			f"rewrite check did not emit one JSON object for {source.name}",
		)
	try:
		report = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise CdmlPreservationE2eError(
			f"rewrite check emitted invalid JSON for {source.name}",
		) from error
	if report.get("schema") != "ferrum-cdml-rewrite-check-v1" or report.get("valid") is not True:
		raise CdmlPreservationE2eError(
			f"rewrite check emitted an invalid contract for {source.name}",
		)
	return {
		"path": source.relative_to(REPO_ROOT).as_posix(),
		"opaque_child_count": report["opaque_child_count"],
		"persistent_id_count": report["persistent_id_count"],
		"top_level_record_count": report["top_level_record_count"],
		"typed_record_counts": report["typed_record_counts"],
	}


#============================================
def main() -> int:
	"""Check every corpus input and print one machine-readable receipt."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"--ferrum", type=pathlib.Path, required=True,
		help="path to the already-built Ferrum CLI executable",
	)
	arguments = parser.parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise CdmlPreservationE2eError("--ferrum must name an existing executable")
	results = tuple(_check_document(ferrum, source) for source in _corpus_paths())
	print(json.dumps({
		"schema": "ferrum-cdml-preservation-corpus-v1",
		"document_count": len(results),
		"results": results,
		"status": "preserved",
	}, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
