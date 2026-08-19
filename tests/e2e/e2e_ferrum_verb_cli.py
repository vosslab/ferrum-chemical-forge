#!/usr/bin/env python3
"""Exercise Ferrum's human CLI verbs against the frozen protocol executor."""

# Standard Library
import argparse
import base64
import json
import os
import pathlib
import subprocess
import sys
import tempfile

# PIP3 modules
import defusedxml.ElementTree


CDML = (
	'<cdml><molecule id="m"><atom id="a" name="C">'
	'<point x="10" y="20"/></atom></molecule></cdml>'
)


#============================================
class VerbE2eError(RuntimeError):
	"""Report a broken command, stream, exit-status, or semantic-equivalence contract."""


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse the path to one already-built Ferrum executable."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", type=pathlib.Path, required=True)
	parser.add_argument(
		"--engine-bundle", type=pathlib.Path,
		help="optional validated native engine bundle for successful convert/coords checks",
	)
	args = parser.parse_args()
	return args


#============================================
def run(
		ferrum: pathlib.Path, *arguments: str, input_text: str = "",
		home: pathlib.Path | None = None,
		) -> subprocess.CompletedProcess[str]:
	"""Run one public command with isolated captured standard streams."""
	environment = os.environ.copy()
	if home is not None:
		environment["HOME"] = str(home)
	result = subprocess.run(
		[str(ferrum), *arguments],
		input=input_text,
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		env=environment,
		check=False,
	)
	return result


#============================================
def require_exit(
		result: subprocess.CompletedProcess[str], expected: int, label: str,
		) -> None:
	"""Require one exact public exit status with an actionable failure."""
	if result.returncode != expected:
		raise VerbE2eError(
			f"{label} exited {result.returncode}, expected {expected}: "
			f"{result.stderr.strip()}"
		)


#============================================
def json_object(text: str, label: str) -> dict[str, object]:
	"""Decode one object without comparing its serialized bytes."""
	value = json.loads(text)
	if not isinstance(value, dict):
		raise VerbE2eError(f"{label} did not emit a JSON object")
	return value


#============================================
def protocol_envelope(
		ferrum: pathlib.Path, operation: dict[str, object], home: pathlib.Path,
		) -> dict[str, object]:
	"""Execute one equivalent frozen request for semantic verb comparison."""
	request = {
		"schema": "ferrum-operation-request-v1",
		"request_id": "ferrum-cli",
		"operation": operation,
	}
	request_text = json.dumps(request, separators=(",", ":"), sort_keys=True)
	result = run(ferrum, "protocol", "run", "-", input_text=request_text, home=home)
	require_exit(result, 0, "protocol run")
	if result.stderr:
		raise VerbE2eError("protocol run wrote a diagnostic for a successful request")
	envelope = json_object(result.stdout, "protocol run")
	return envelope


#============================================
def semantic_xml(text: str, label: str) -> tuple:
	"""Return a whitespace-neutral XML tree for semantic artifact comparison."""
	root = defusedxml.ElementTree.fromstring(text)

	def project(element: object) -> tuple:
		attributes = tuple(sorted(element.attrib.items()))
		content = (element.text or "").strip()
		children = tuple(project(child) for child in element)
		projection = (element.tag, attributes, content, children)
		return projection

	projection = project(root)
	return projection


#============================================
def outcome(envelope: dict[str, object], kind: str) -> dict[str, object]:
	"""Return one required successful protocol outcome of the selected kind."""
	value = envelope["outcome"]
	if not isinstance(value, dict) or value["kind"] != kind:
		raise VerbE2eError(f"protocol response did not contain {kind}")
	return value


#============================================
def check_inspect(
		ferrum: pathlib.Path, source: pathlib.Path, home: pathlib.Path,
		) -> None:
	"""Compare inspect's human and envelope surfaces to protocol execution."""
	operation = {"kind": "document.inspect", "document": CDML}
	expected = protocol_envelope(ferrum, operation, home)
	json_result = run(ferrum, "inspect", str(source), "--json", home=home)
	require_exit(json_result, 0, "inspect --json")
	if json_object(json_result.stdout, "inspect --json") != expected:
		raise VerbE2eError("inspect --json differs semantically from protocol run")
	human_result = run(ferrum, "inspect", "-", input_text=CDML, home=home)
	require_exit(human_result, 0, "inspect stdin")
	if json_object(human_result.stdout, "inspect report") != outcome(
			expected, "document.inspect",
		)["report"]:
		raise VerbE2eError("inspect report differs from its protocol outcome")


#============================================
def check_validate(
		ferrum: pathlib.Path, source: pathlib.Path, home: pathlib.Path,
		) -> None:
	"""Compare typed validation and exercise completed-refusal behavior."""
	operation = {
		"kind": "document.validate", "document": CDML, "level": "typed",
	}
	expected = protocol_envelope(ferrum, operation, home)
	json_result = run(
		ferrum, "validate", str(source), "--level", "typed", "--json", home=home,
	)
	require_exit(json_result, 0, "validate --json")
	if json_object(json_result.stdout, "validate --json") != expected:
		raise VerbE2eError("validate --json differs semantically from protocol run")
	human_result = run(
		ferrum, "validate", "-", "--level", "typed", input_text=CDML, home=home,
	)
	require_exit(human_result, 0, "validate stdin")
	if json_object(human_result.stdout, "validate report") != outcome(
			expected, "document.validate",
		)["report"]:
		raise VerbE2eError("validate report differs from its protocol outcome")
	refusal = run(ferrum, "validate", "-", input_text="not CDML", home=home)
	require_exit(refusal, 1, "invalid CDML admission")
	if refusal.stdout or not refusal.stderr.startswith("ferrum: "):
		raise VerbE2eError("input refusal did not use the human diagnostic channel")


#============================================
def check_rewrite(
		ferrum: pathlib.Path, source: pathlib.Path, destination: pathlib.Path,
		home: pathlib.Path,
		) -> str:
	"""Compare rewrite semantics and return its stream output for composition."""
	operation = {"kind": "document.rewrite", "document": CDML}
	expected = protocol_envelope(ferrum, operation, home)
	json_result = run(ferrum, "rewrite", str(source), "--json", home=home)
	require_exit(json_result, 0, "rewrite --json")
	if json_object(json_result.stdout, "rewrite --json") != expected:
		raise VerbE2eError("rewrite --json differs semantically from protocol run")
	expected_document = outcome(expected, "document.rewrite")["document"]
	if not isinstance(expected_document, str):
		raise VerbE2eError("protocol rewrite outcome lacks its CDML document")
	stream_result = run(ferrum, "rewrite", "-", input_text=CDML, home=home)
	require_exit(stream_result, 0, "rewrite stdin/stdout")
	if semantic_xml(stream_result.stdout, "rewrite stdout") != semantic_xml(
			expected_document, "protocol rewrite",
		):
		raise VerbE2eError("rewrite output differs semantically from protocol run")
	file_result = run(
		ferrum, "rewrite", str(source), "--output", str(destination), home=home,
	)
	require_exit(file_result, 0, "rewrite file publication")
	if file_result.stdout or semantic_xml(
			destination.read_text(encoding="utf-8"), "published rewrite",
		) != semantic_xml(expected_document, "protocol rewrite"):
		raise VerbE2eError("published rewrite differs semantically from protocol run")
	return stream_result.stdout


#============================================
def check_render(
		ferrum: pathlib.Path, source: pathlib.Path, destination: pathlib.Path,
		home: pathlib.Path,
		) -> None:
	"""Compare raw and safely published SVG to the protocol artifact."""
	operation = {"kind": "document.render_artifact", "document": CDML, "format": "svg"}
	expected = protocol_envelope(ferrum, operation, home)
	json_result = run(
		ferrum, "render", str(source), "--to", "svg", "--json", home=home,
	)
	require_exit(json_result, 0, "render --json")
	if json_object(json_result.stdout, "render --json") != expected:
		raise VerbE2eError("render --json differs semantically from protocol run")
	encoded = outcome(expected, "document.render_artifact")["artifact_base64"]
	if not isinstance(encoded, str):
		raise VerbE2eError("protocol render outcome lacks its artifact")
	expected_svg = base64.b64decode(encoded, validate=True).decode("utf-8")
	stream_result = run(
		ferrum, "render", "-", "--to", "svg", input_text=CDML, home=home,
	)
	require_exit(stream_result, 0, "render stdin/stdout")
	if semantic_xml(stream_result.stdout, "render stdout") != semantic_xml(
			expected_svg, "protocol render",
		):
		raise VerbE2eError("render output differs semantically from protocol run")
	file_result = run(
		ferrum, "render", str(source), "--output", str(destination), home=home,
	)
	require_exit(file_result, 0, "render file publication")
	if file_result.stdout or semantic_xml(
			destination.read_text(encoding="utf-8"), "published render",
		) != semantic_xml(expected_svg, "protocol render"):
		raise VerbE2eError("published render differs semantically from protocol run")


#============================================
def check_engine_verbs(
		ferrum: pathlib.Path, temp: pathlib.Path, home: pathlib.Path,
		bundle: pathlib.Path | None,
		) -> None:
	"""Compare convert and coords to their frozen protocol operations."""
	convert_source = temp / "methane.smi"
	convert_source.write_text("C\n", encoding="utf-8")
	convert_operation = {
		"kind": "chemistry.convert",
		"input": {"format": "smiles", "text": "C\n"},
		"output_format": "smiles",
	}
	convert_expected = protocol_envelope(ferrum, convert_operation, home)
	convert_json = run(
		ferrum, "convert", str(convert_source), "--to", "smiles", "--json", home=home,
	)
	require_exit(convert_json, 0, "convert --json")
	if json_object(convert_json.stdout, "convert --json") != convert_expected:
		raise VerbE2eError("convert --json differs semantically from protocol run")

	coords_operation = {"kind": "document.generate_coordinates", "document": CDML}
	coords_expected = protocol_envelope(ferrum, coords_operation, home)
	coords_json = run(ferrum, "coords", "-", "--json", input_text=CDML, home=home)
	require_exit(coords_json, 0, "coords --json")
	if json_object(coords_json.stdout, "coords --json") != coords_expected:
		raise VerbE2eError("coords --json differs semantically from protocol run")

	if bundle is None:
		for label, expected in [("convert", convert_expected), ("coords", coords_expected)]:
			if expected.get("error", {}).get("category") != "chemistry_unavailable":
				raise VerbE2eError(f"{label} did not make missing chemistry a typed refusal")
		return
	if "outcome" not in convert_expected or "outcome" not in coords_expected:
		raise VerbE2eError(
			"the supplied engine bundle did not complete both engine operations: "
			f"convert={convert_expected.get('error')!r}; "
			f"coords={coords_expected.get('error')!r}"
		)

	convert_outcome = outcome(convert_expected, "chemistry.convert")
	converted = convert_outcome["text"]
	if not isinstance(converted, str):
		raise VerbE2eError("convert protocol outcome lacks text")
	convert_stream = run(
		ferrum, "convert", "-", "--from", "smiles", "--to", "smiles",
		input_text="C\n", home=home,
	)
	require_exit(convert_stream, 0, "convert stdin/stdout")
	if convert_stream.stdout != converted:
		raise VerbE2eError("convert stream output differs from protocol outcome")
	convert_destination = temp / "methane.out.smi"
	convert_file = run(
		ferrum, "convert", str(convert_source), "--to", "smiles", "-o",
		str(convert_destination), home=home,
	)
	require_exit(convert_file, 0, "convert file publication")
	if convert_file.stdout or convert_destination.read_text(encoding="utf-8") != converted:
		raise VerbE2eError("convert publication differs from protocol outcome")

	coords_outcome = outcome(coords_expected, "document.generate_coordinates")
	coordinated = coords_outcome["document"]
	if not isinstance(coordinated, str):
		raise VerbE2eError("coords protocol outcome lacks CDML")
	coords_stream = run(ferrum, "coords", "-", input_text=CDML, home=home)
	require_exit(coords_stream, 0, "coords stdin/stdout")
	if semantic_xml(coords_stream.stdout, "coords stdout") != semantic_xml(
			coordinated, "coords protocol",
		):
		raise VerbE2eError("coords stream output differs from protocol outcome")
	coords_destination = temp / "coordinated.cdml"
	coords_file = run(
		ferrum, "coords", str(temp / "drawing.cdml"), "-o", str(coords_destination),
		home=home,
	)
	require_exit(coords_file, 0, "coords file publication")
	if coords_file.stdout or semantic_xml(
			coords_destination.read_text(encoding="utf-8"), "published coords",
		) != semantic_xml(coordinated, "coords protocol"):
		raise VerbE2eError("coords publication differs from protocol outcome")


#============================================
def check_exit_channels(
		ferrum: pathlib.Path, rewritten: str, home: pathlib.Path,
		) -> None:
	"""Check stream composition plus processing and usage exit channels."""
	composed = run(
		ferrum, "validate", "-", "--level", "structural", input_text=rewritten,
		home=home,
	)
	require_exit(composed, 0, "rewrite-to-validate composition")
	missing_format = run(ferrum, "render", "-", input_text=CDML, home=home)
	require_exit(missing_format, 1, "render missing format")
	usage = run(ferrum, "validate", home=home)
	require_exit(usage, 2, "validate usage")
	if not missing_format.stderr.startswith("ferrum: ") or not usage.stderr:
		raise VerbE2eError("failed command diagnostics did not use standard error")


#============================================
def main() -> int:
	"""Run every existing human verb through file, stream, and protocol paths."""
	args = parse_args()
	ferrum = args.ferrum.resolve()
	if not ferrum.is_file():
		raise VerbE2eError("--ferrum must name an existing executable")
	with tempfile.TemporaryDirectory(prefix="ferrum-verb-e2e-") as temp_text:
		# Resolve a platform temporary-directory alias before the no-follow publisher.
		temp = pathlib.Path(temp_text).resolve()
		home = temp / "home"
		home.mkdir()
		if args.engine_bundle is not None:
			bundle = args.engine_bundle.resolve()
			if not bundle.is_dir():
				raise VerbE2eError("--engine-bundle must name an existing bundle directory")
			installed = run(ferrum, "engine", "install", str(bundle), home=home)
			require_exit(installed, 0, "engine bundle installation")
		else:
			bundle = None
		source = temp / "drawing.cdml"
		source.write_text(CDML, encoding="utf-8")
		check_inspect(ferrum, source, home)
		check_validate(ferrum, source, home)
		rewritten = check_rewrite(ferrum, source, temp / "rewritten.cdml", home)
		check_render(ferrum, source, temp / "drawing.svg", home)
		check_engine_verbs(ferrum, temp, home, bundle)
		check_exit_channels(ferrum, rewritten, home)
	result = {"schema": "ferrum-verb-cli-e2e-v1", "status": "ok"}
	print(json.dumps(result, sort_keys=True))
	return 0


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except (VerbE2eError, json.JSONDecodeError, OSError, ValueError) as error:
		print(f"verb CLI E2E error: {error}", file=sys.stderr)
		raise SystemExit(1)
