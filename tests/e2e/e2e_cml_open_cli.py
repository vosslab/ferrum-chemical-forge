"""Exercise the offline fixed-target CML open command against a real Ferrum CLI."""

# Standard Library
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


CML = (
	'<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray>'
	'<atom id="a1" elementType="C" x2="0" y2="0"/>'
	'</atomArray></molecule></cml>'
)


#============================================
class CmlOpenE2eError(RuntimeError):
	"""Report one public CML-open contract failure."""


#============================================
def parse_args() -> argparse.Namespace:
	"""Read the already-built Ferrum executable path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", required=True, type=Path)
	return parser.parse_args()


#============================================
def main() -> None:
	"""Open inline CML and verify separated bounded summary and CDML delivery."""
	arguments = parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise CmlOpenE2eError("--ferrum must name an existing executable")
	with tempfile.TemporaryDirectory(prefix="ferrum-cml-open-") as directory:
		temp = Path(directory).resolve()
		source = temp / "molecule.cml"
		destination = temp / "opened.cdml"
		source.write_text(CML, encoding="utf-8")
		result = subprocess.run(
			[
				str(ferrum), "open", str(source), "--format", "cml", "--output",
				str(destination), "--json",
			],
			text=True,
			stdout=subprocess.PIPE,
			stderr=subprocess.PIPE,
			check=False,
		)
		if result.returncode != 0 or result.stderr:
			raise CmlOpenE2eError(f"CML open failed: {result.stderr.strip()}")
		try:
			envelope = json.loads(result.stdout)
		except json.JSONDecodeError as error:
			raise CmlOpenE2eError("CML open did not emit one JSON response") from error
		if (
			envelope.get("schema") != "ferrum-operation-response-v1"
			or envelope.get("request_id") != "ferrum-cli"
			or envelope.get("outcome", {}).get("kind") != "document.molecule.interchange.import.v1"
			or envelope.get("outcome", {}).get("summary", {}).get("format_id")
			!= "cml_simple_molecule_import_v1"
			or "<cdml" in result.stdout
			or CML in result.stdout
			or "source_molecule_id" in result.stdout
		):
			raise CmlOpenE2eError("CML open did not keep the document out of its JSON response")
		if not destination.read_text(encoding="utf-8").startswith("<cdml"):
			raise CmlOpenE2eError("CML open did not publish the new CDML document")
		protocol_request = temp / "import-request.json"
		protocol_request.write_text(json.dumps({
			"schema": "ferrum-operation-request-v1",
			"request_id": "cml-open-e2e",
			"operation": {
				"kind": "document.molecule.interchange.import.v1",
				"format_alias": "cml", "source_utf8": CML,
			},
		}), encoding="utf-8")
		protocol = subprocess.run(
			[str(ferrum), "document", "command",
			"document.molecule.interchange.import.v1", str(protocol_request)],
			text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
		)
		if protocol.returncode != 0 or protocol.stderr:
			raise CmlOpenE2eError(f"named CML protocol failed: {protocol.stderr.strip()}")
		protocol_envelope = json.loads(protocol.stdout)
		if (
			protocol_envelope.get("schema") != "ferrum-operation-response-v1"
			or "source_molecule_id" in protocol.stdout
			or CML in protocol.stdout
		):
			raise CmlOpenE2eError("named CML protocol leaked source identity or source text")
		invalid_cml = "FERRUM_E2E_CML_REFUSAL_SOURCE"
		invalid_source = temp / "invalid.cml"
		invalid_source.write_text(invalid_cml, encoding="utf-8")
		refused_output = temp / "refused.cdml"
		refused = subprocess.run(
			[
				str(ferrum), "open", str(invalid_source), "--format", "cml", "--output",
				str(refused_output), "--json",
			],
			text=True,
			stdout=subprocess.PIPE,
			stderr=subprocess.PIPE,
			check=False,
		)
		try:
			refusal_envelope = json.loads(refused.stdout)
		except json.JSONDecodeError as error:
			raise CmlOpenE2eError("typed refusal did not emit one JSON envelope") from error
		if (
			refused.returncode != 1
			or refused.stderr
			or refusal_envelope.get("schema") != "ferrum-operation-error-v1"
			or refusal_envelope.get("request_id") != "ferrum-cli"
			or refusal_envelope.get("error", {}).get("operation")
			!= "document.molecule.interchange.import.v1"
			or refusal_envelope.get("error", {}).get("category") != "conversion_failed"
			or "<cdml" in refused.stdout
			or "<cdml" in refused.stderr
			or invalid_cml in refused.stdout
			or invalid_cml in refused.stderr
			or invalid_source.name in refused.stderr
			or refused_output.exists()
		):
			raise CmlOpenE2eError("JSON CML refusal did not preserve the canonical contract")
		protocol_request.write_text(json.dumps({
			"schema": "ferrum-operation-request-v1",
			"request_id": "cml-open-refusal-e2e",
			"operation": {
				"kind": "document.molecule.interchange.import.v1",
				"format_alias": "cml", "source_utf8": "not XML",
			},
		}), encoding="utf-8")
		protocol_refusal = subprocess.run(
			[str(ferrum), "document", "command",
			"document.molecule.interchange.import.v1", str(protocol_request)],
			text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
		)
		refusal_protocol_envelope = json.loads(protocol_refusal.stdout)
		if (
			protocol_refusal.returncode != 0
			or protocol_refusal.stderr
			or refusal_protocol_envelope.get("schema") != "ferrum-operation-error-v1"
			or "not XML" in protocol_refusal.stdout
			or "<cdml" in protocol_refusal.stdout
		):
			raise CmlOpenE2eError("named CML refusal leaked source or published a document")


if __name__ == "__main__":
	main()
