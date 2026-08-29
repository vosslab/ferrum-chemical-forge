"""Exercise all singular selected-root export presentations through Ferrum's CLI."""

from __future__ import annotations

# Standard Library
import argparse
import json
from pathlib import Path
import shutil
import stat
import subprocess
import sys

from e2e_workspace import E2EWorkspaceLease


MOLECULE_ID = "ferrum-document-object-v1/00000000000000000000000000000031"
ATOM_ID = "ferrum-document-object-v1/00000000000000000000000000000032"
FORMATS = (
	"molfile_v2000",
	"molfile_v3000",
	"sdf_v2000",
	"sdf_v3000",
	"canonical_smiles",
	"inchi_standard",
	"inchi_fixed_hydrogen",
)
CDML = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:object="urn:ferrum:document-object:v1">'
	f'<molecule id="methane" object:id="{MOLECULE_ID}">'
	f'<atom id="carbon" object:id="{ATOM_ID}" name="C"><point x="0" y="0"/></atom>'
	"</molecule></cdml>"
)
UNEXPANDED_GROUP_MOLECULE_ID = "ferrum-document-object-v1/00000000000000000000000000000041"
UNEXPANDED_GROUP_CDML = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:object="urn:ferrum:document-object:v1">'
	f'<molecule id="unexpanded" object:id="{UNEXPANDED_GROUP_MOLECULE_ID}">'
	'<compact-group id="methyl" '
	'object:id="ferrum-document-object-v1/00000000000000000000000000000042" '
	'version="1" catalog-key="methyl" attachment-index="0" orientation-degrees="0">'
	'<point x="20" y="0"/></compact-group></molecule></cdml>'
)
POINTLESS_MOLECULE_ID = "ferrum-document-object-v1/00000000000000000000000000000051"
POINTLESS_CDML = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:object="urn:ferrum:document-object:v1">'
	f'<molecule id="pointless" object:id="{POINTLESS_MOLECULE_ID}">'
	'<atom id="carbon" object:id="ferrum-document-object-v1/00000000000000000000000000000052" '
	'name="C"/></molecule></cdml>'
)

# Keep this fixture tied to ferrum_chem::molblock_text_upper_bound for V2000:
# 4096 + title bytes + atoms * 256 + bonds * 128 + 4096.  The fixture has no
# title or bonds, and its native admission upper bound must exceed the public
# selected-export text ceiling before a native writer is called.
MOLFILE_V2000_UPPER_BOUND_FIXED_BYTES = 2 * 4096
MOLFILE_V2000_UPPER_BOUND_ATOM_BYTES = 256
SELECTED_EXPORT_TEXT_LIMIT_BYTES = 128 * 1024
OVER_LIMIT_ATOM_COUNT = (
	(SELECTED_EXPORT_TEXT_LIMIT_BYTES - MOLFILE_V2000_UPPER_BOUND_FIXED_BYTES)
	// MOLFILE_V2000_UPPER_BOUND_ATOM_BYTES + 1
)


#============================================
class DocumentMoleculeExportCliE2eError(RuntimeError):
	"""Report one broken public selected-root export contract."""


#============================================
def parse_arguments() -> argparse.Namespace:
	"""Read the already-built Ferrum command path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", required=True, type=Path)
	return parser.parse_args()


#============================================
def run(
	ferrum: Path, *arguments: str, input_text: str = "",
) -> subprocess.CompletedProcess[str]:
	"""Run one public CLI request with isolated text streams."""
	return subprocess.run(
		[str(ferrum), *arguments],
		input=input_text,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)


#============================================
def require_exit(
	result: subprocess.CompletedProcess[str], expected: int, label: str,
) -> None:
	"""Require one exact process exit status."""
	if result.returncode != expected:
		raise DocumentMoleculeExportCliE2eError(
			f"{label} exited {result.returncode}, expected {expected}: {result.stderr.strip()}"
		)


#============================================
def one_envelope(result: subprocess.CompletedProcess[str], label: str) -> dict[str, object]:
	"""Decode exactly one canonical protocol response without accepting diagnostics."""
	if result.stderr:
		raise DocumentMoleculeExportCliE2eError(f"{label} wrote stderr: {result.stderr.strip()}")
	lines = result.stdout.splitlines()
	if len(lines) != 1:
		raise DocumentMoleculeExportCliE2eError(f"{label} did not emit one JSON envelope")
	try:
		value = json.loads(lines[0])
	except json.JSONDecodeError as error:
		raise DocumentMoleculeExportCliE2eError(
			f"{label} emitted invalid JSON: {error.msg}"
		) from error
	if not isinstance(value, dict):
		raise DocumentMoleculeExportCliE2eError(f"{label} did not emit a JSON object")
	return value


#============================================
def request(request_id: str, operation: dict[str, object]) -> str:
	"""Serialize one complete frozen operation request."""
	return json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": request_id,
		"operation": operation,
	}, separators=(",", ":"), sort_keys=True)


#============================================
def inspect_fence(ferrum: Path) -> dict[str, object]:
	"""Obtain the only accepted fence for the exact source CDML."""
	result = run(ferrum, "protocol", "run", "-", input_text=request("export-inspect", {
		"kind": "document.inspect", "document": CDML,
	}))
	require_exit(result, 0, "export inspection")
	envelope = one_envelope(result, "export inspection")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.inspect":
		raise DocumentMoleculeExportCliE2eError("inspection omitted a document fence")
	fence = outcome.get("document_fence")
	if (
		not isinstance(fence, dict)
		or not isinstance(fence.get("expected_revision"), int)
		or not isinstance(fence.get("expected_digest_hex"), str)
		or not fence["expected_digest_hex"]
	):
		raise DocumentMoleculeExportCliE2eError("inspection emitted no usable document fence")
	return fence


#============================================
def export_operation(
	fence: dict[str, object], molecule_id: str, format_name: str, document: str = CDML,
) -> dict[str, object]:
	"""Build the sole named selected-root export operation."""
	return {
		"kind": "document.molecule.export.v1",
		"document": {
			"cdml": document,
			"expected_revision": fence["expected_revision"],
			"expected_digest_hex": fence["expected_digest_hex"],
		},
		"molecule_id": molecule_id,
		"format": format_name,
	}


#============================================
def named_export(
	ferrum: Path, request_text: str, output: Path | None = None,
) -> subprocess.CompletedProcess[str]:
	"""Run the explicit named presentation of the frozen export request."""
	arguments = ["document", "command", "document.molecule.export.v1", "-"]
	if output is not None:
		arguments.extend(["--output", str(output)])
	return run(ferrum, *arguments, input_text=request_text)


#============================================
def named_export_file(
	ferrum: Path, request_path: Path, output: Path | None = None,
) -> subprocess.CompletedProcess[str]:
	"""Run one named request from its retained source file."""
	arguments = ["document", "command", "document.molecule.export.v1", str(request_path)]
	if output is not None:
		arguments.extend(["--output", str(output)])
	return run(ferrum, *arguments)


#============================================
def successful_export_text(
	envelope: dict[str, object], request_id: str, fence: dict[str, object], format_name: str,
) -> str:
	"""Require one complete named result and return its exact text payload."""
	if (
		envelope.get("schema") != "ferrum-operation-response-v1"
		or envelope.get("request_id") != request_id
	):
		raise DocumentMoleculeExportCliE2eError("successful named export lost envelope identity")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or outcome.get("kind") != "document.molecule.export.v1":
		raise DocumentMoleculeExportCliE2eError("successful named export omitted its typed outcome")
	export = outcome.get("export")
	if (
		not isinstance(export, dict)
		or export.get("source_revision") != fence["expected_revision"]
		or export.get("source_digest_hex") != fence["expected_digest_hex"]
		or export.get("molecule_id") != MOLECULE_ID
		or export.get("format") != format_name
		or not isinstance(export.get("text"), str)
	):
		raise DocumentMoleculeExportCliE2eError("successful named export omitted frozen result facts")
	return export["text"]


#============================================
def typed_refusal(
	envelope: dict[str, object], request_id: str, category: str,
) -> None:
	"""Require the one closed export-refusal fact and no duplicate human diagnostic."""
	error = envelope.get("error")
	if (
		envelope.get("schema") != "ferrum-operation-error-v1"
		or envelope.get("request_id") != request_id
		or not isinstance(error, dict)
		or error.get("operation") != "document.molecule.export.v1"
	):
		raise DocumentMoleculeExportCliE2eError("named refusal omitted its export error envelope")
	refusal = error.get("document_molecule_export_refusal")
	if not isinstance(refusal, dict) or refusal.get("category") != category:
		raise DocumentMoleculeExportCliE2eError(
			f"named refusal omitted category {category!r}: {refusal!r}"
		)


#============================================
def check_direct_and_named_successes(ferrum: Path, workspace: Path, fence: dict[str, object]) -> None:
	"""Prove every closed format has matching raw and canonical JSON presentations."""
	source = workspace / "drawing.cdml"
	source.write_text(CDML, encoding="utf-8")
	for format_name in FORMATS:
		request_id = f"selected-export-{format_name}"
		request_text = request(request_id, export_operation(fence, MOLECULE_ID, format_name))
		named = named_export(ferrum, request_text)
		require_exit(named, 0, f"named {format_name} export")
		expected_text = successful_export_text(
			one_envelope(named, f"named {format_name} export"), request_id, fence, format_name,
		)
		direct = run(
			ferrum, "document", "export", "--input", str(source), "--molecule-id", MOLECULE_ID,
			"--format", format_name,
		)
		require_exit(direct, 0, f"direct {format_name} export")
		if direct.stderr or direct.stdout != expected_text:
			raise DocumentMoleculeExportCliE2eError(
				f"direct {format_name} stdout did not exactly match the named result text"
			)
		destination = workspace / f"{format_name}.out"
		published = run(
			ferrum, "document", "export", "--input", str(source), "--molecule-id", MOLECULE_ID,
			"--format", format_name, "--output", str(destination),
		)
		require_exit(published, 0, f"published {format_name} export")
		if published.stdout or published.stderr or destination.read_text(encoding="utf-8") != expected_text:
			raise DocumentMoleculeExportCliE2eError(
				f"published {format_name} content did not exactly match the named result text"
			)


#============================================
def check_safe_publication(ferrum: Path, workspace: Path, fence: dict[str, object]) -> None:
	"""Prove normal named publication and source/existing/symlink refusal behavior."""
	source = workspace / "drawing.cdml"
	request_text = request("named-publication", export_operation(fence, MOLECULE_ID, "canonical_smiles"))
	named_destination = workspace / "named-response.json"
	named = named_export(ferrum, request_text, named_destination)
	require_exit(named, 0, "named response publication")
	if named.stdout or named.stderr:
		raise DocumentMoleculeExportCliE2eError("named output publication used standard streams")
	published = named_destination.read_text(encoding="utf-8")
	if json.loads(published) != one_envelope(named_export(ferrum, request_text), "named stdout"):
		raise DocumentMoleculeExportCliE2eError("named output was not the canonical response envelope")
	request_source = workspace / "named-request.json"
	request_source.write_text(request_text, encoding="utf-8")
	request_baseline = request_source.read_bytes()
	result = named_export_file(ferrum, request_source, request_source)
	if result.returncode == 0 or result.stdout or not result.stderr:
		raise DocumentMoleculeExportCliE2eError("named request/output identity was not safely refused")
	if request_source.read_bytes() != request_baseline:
		raise DocumentMoleculeExportCliE2eError("named request/output identity changed the request source")
	baseline = source.read_bytes()
	source_alias = source.with_name("source-alias.cdml")
	source_alias.hardlink_to(source)
	for destination, label in ((source, "source identity"), (source_alias, "source alias")):
		result = run(
			ferrum, "document", "export", "--input", str(source), "--molecule-id", MOLECULE_ID,
			"--format", "canonical_smiles", "--output", str(destination),
		)
		if result.returncode == 0 or result.stdout or not result.stderr or source.read_bytes() != baseline:
			raise DocumentMoleculeExportCliE2eError(f"direct {label} output was not safely refused")
	existing = workspace / "existing.out"
	existing.write_text("preserve existing output\n", encoding="utf-8")
	result = run(
		ferrum, "document", "export", "--input", str(source), "--molecule-id", MOLECULE_ID,
		"--format", "canonical_smiles", "--output", str(existing),
	)
	if result.returncode == 0 or existing.read_text(encoding="utf-8") != "preserve existing output\n":
		raise DocumentMoleculeExportCliE2eError("existing direct destination was changed on refusal")
	symlink_target = workspace / "symlink-target.out"
	symlink_target.write_text("preserve symlink target\n", encoding="utf-8")
	symlink = workspace / "symlink.out"
	symlink.symlink_to(symlink_target.name)
	result = run(
		ferrum, "document", "export", "--input", str(source), "--molecule-id", MOLECULE_ID,
		"--format", "canonical_smiles", "--output", str(symlink),
	)
	if result.returncode == 0 or symlink_target.read_text(encoding="utf-8") != "preserve symlink target\n":
		raise DocumentMoleculeExportCliE2eError("symlink direct destination was changed on refusal")


#============================================
def check_named_refusals(ferrum: Path, workspace: Path, fence: dict[str, object]) -> None:
	"""Exercise all five typed refusal categories and named route/publication refusal."""
	stale_fence = dict(fence)
	stale_fence["expected_digest_hex"] = changed_digest(str(fence["expected_digest_hex"]))
	unexpanded_group_fence = inspect_document(ferrum, UNEXPANDED_GROUP_CDML)
	pointless_fence = {
		"expected_revision": 0,
		"expected_digest_hex": "0" * 64,
	}
	refusals = (
		("snapshot-not-admitted", export_operation(stale_fence, MOLECULE_ID, "canonical_smiles"), "snapshot_not_admitted"),
		("pointless-document", export_operation(pointless_fence, POINTLESS_MOLECULE_ID, "canonical_smiles", POINTLESS_CDML), "snapshot_not_admitted"),
		("unknown-root", export_operation(fence, "ferrum-document-object-v1/00000000000000000000000000000099", "canonical_smiles"), "unknown_or_non_direct_root"),
		("non-direct-root", export_operation(fence, ATOM_ID, "canonical_smiles"), "unknown_or_non_direct_root"),
		("unexpanded-group-molfile", export_operation(unexpanded_group_fence, UNEXPANDED_GROUP_MOLECULE_ID, "molfile_v2000", UNEXPANDED_GROUP_CDML), "representation_unsupported"),
		("unexpanded-group-sdf", export_operation(unexpanded_group_fence, UNEXPANDED_GROUP_MOLECULE_ID, "sdf_v2000", UNEXPANDED_GROUP_CDML), "representation_unsupported"),
	)
	for request_id, operation, category in refusals:
		result = named_export(ferrum, request(request_id, operation))
		require_exit(result, 1, f"named {request_id} refusal")
		typed_refusal(one_envelope(result, f"named {request_id} refusal"), request_id, category)
	limit_source = workspace / "over-limit.cdml"
	limit_source.write_text(over_limit_cdml(), encoding="utf-8")
	limit_fence = inspect_document(ferrum, limit_source.read_text(encoding="utf-8"))
	limit_request_id = "output-limit-exceeded"
	limit_request = request(
		limit_request_id,
		export_operation(limit_fence, over_limit_molecule_id(), "molfile_v2000", limit_source.read_text(encoding="utf-8")),
	)
	limit_destination = workspace / "over-limit.out"
	limit_direct = run(
		ferrum, "document", "export", "--input", str(limit_source), "--molecule-id", over_limit_molecule_id(),
		"--format", "molfile_v2000", "--output", str(limit_destination),
	)
	if limit_direct.returncode == 0 or limit_destination.exists():
		raise DocumentMoleculeExportCliE2eError("over-limit direct export published an artifact")
	limit_named = named_export(ferrum, limit_request)
	require_exit(limit_named, 1, "named output-limit refusal")
	typed_refusal(one_envelope(limit_named, "named output-limit refusal"), limit_request_id, "output_limit_exceeded")
	mismatch = request("named-route-mismatch", {"kind": "document.inspect", "document": CDML})
	result = named_export(ferrum, mismatch)
	require_exit(result, 1, "named route mismatch")
	envelope = one_envelope(result, "named route mismatch")
	if envelope.get("error", {}).get("category") != "invalid_request":
		raise DocumentMoleculeExportCliE2eError("named route mismatch did not retain invalid_request")
	response_destination = workspace / "typed-refusal.json"
	result = named_export(ferrum, request("named-output-refusal", refusals[0][1]), response_destination)
	require_exit(result, 1, "named typed-refusal publication")
	if result.stdout or result.stderr:
		raise DocumentMoleculeExportCliE2eError("named typed-refusal publication used standard streams")
	typed_refusal(
		json.loads(response_destination.read_text(encoding="utf-8")), "named-output-refusal", "snapshot_not_admitted",
	)


#============================================
def inspect_document(ferrum: Path, document: str) -> dict[str, object]:
	"""Return the authoritative fence for a supplied valid CDML document."""
	result = run(ferrum, "protocol", "run", "-", input_text=request("over-limit-inspect", {
		"kind": "document.inspect", "document": document,
	}))
	require_exit(result, 0, "over-limit inspection")
	envelope = one_envelope(result, "over-limit inspection")
	outcome = envelope.get("outcome")
	if not isinstance(outcome, dict) or not isinstance(outcome.get("document_fence"), dict):
		raise DocumentMoleculeExportCliE2eError("over-limit inspection omitted a fence")
	return outcome["document_fence"]


#============================================
def changed_digest(digest_hex: str) -> str:
	"""Return one other valid digest spelling without predicting a document digest."""
	if not digest_hex:
		raise DocumentMoleculeExportCliE2eError("document inspection emitted an empty digest")
	return digest_hex[:-1] + ("0" if digest_hex[-1] != "0" else "1")


#============================================
def over_limit_molecule_id() -> str:
	"""Return the durable root ID used only to exercise the fixed export limit."""
	return "ferrum-document-object-v1/00000000000000000000000000001000"


#============================================
def over_limit_cdml() -> str:
	"""Build a coordinate-bearing root that exceeds V2000's native output bound."""
	atoms = []
	for index in range(OVER_LIMIT_ATOM_COUNT):
		object_id = f"ferrum-document-object-v1/{index + 2000:032d}"
		atoms.append(
			f'<atom id="a{index}" object:id="{object_id}" name="C">'
			f'<point x="{index}" y="0"/></atom>'
		)
	return (
		'<cdml xmlns="urn:ferrum:cdml" xmlns:object="urn:ferrum:document-object:v1">'
		f'<molecule id="over-limit" object:id="{over_limit_molecule_id()}">'
		f'{"".join(atoms)}</molecule></cdml>'
	)


#============================================
def check_unavailable_runtime(ferrum: Path, workspace: Path, fence: dict[str, object]) -> None:
	"""Require direct and named chemistry-unavailable outcomes without a bundle alias."""
	isolation = workspace / "without-runtime"
	(isolation / "bin").mkdir(parents=True)
	program = ferrum.with_name("ferrum.program")
	if not program.is_file():
		raise DocumentMoleculeExportCliE2eError("staged Ferrum launcher lacks ferrum.program")
	unavailable = isolation / "bin" / "ferrum"
	shutil.copy2(program, unavailable)
	unavailable.chmod(unavailable.stat().st_mode | stat.S_IXUSR)
	request_id = "chemistry-unavailable"
	request_text = request(request_id, export_operation(fence, MOLECULE_ID, "canonical_smiles"))
	named = named_export(unavailable, request_text)
	require_exit(named, 1, "named unavailable-runtime export")
	typed_refusal(one_envelope(named, "named unavailable-runtime export"), request_id, "chemistry_unavailable")
	source = workspace / "drawing.cdml"
	direct = run(
		unavailable, "document", "export", "--input", str(source), "--molecule-id", MOLECULE_ID,
		"--format", "canonical_smiles",
	)
	if direct.returncode == 0 or direct.stdout or not direct.stderr.startswith("ferrum: "):
		raise DocumentMoleculeExportCliE2eError("direct unavailable-runtime export missed its human refusal")


#============================================
def main() -> int:
	"""Run the singular export direct/named public CLI matrix."""
	arguments = parse_arguments()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise DocumentMoleculeExportCliE2eError("--ferrum must name an existing executable")
	with E2EWorkspaceLease() as directory:
		workspace = Path(directory)
		fence = inspect_fence(ferrum)
		check_direct_and_named_successes(ferrum, workspace, fence)
		check_named_refusals(ferrum, workspace, fence)
		check_safe_publication(ferrum, workspace, fence)
		check_unavailable_runtime(ferrum, workspace, fence)
	print(json.dumps({
		"schema": "ferrum-document-molecule-export-cli-e2e-v1", "status": "ok",
	}, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except DocumentMoleculeExportCliE2eError as error:
		print(f"document molecule export CLI E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
