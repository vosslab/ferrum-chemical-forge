"""Exercise CDXML import through Ferrum's public runtime-free CLI route."""

# Standard Library
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


CDXML = (
	'<?xml version="1.0" encoding="UTF-8"?>'
	'<!DOCTYPE CDXML SYSTEM '
	'"https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd">'
	'<CDXML CreationProgram="ChemDraw 23.0"><page HeightPages="1">'
	'<fragment id="source-fragment"><n id="source-atom" p="0 0" '
	'Element="8" Charge="-1" Isotope="18"/>'
	'</fragment></page></CDXML>'
)
ZERO_SCALAR_CDXML = (
	'<CDXML><page><fragment id="zero-scalar-fragment">'
	'<n id="zero-scalar-atom" p="0 0" Element="8" Charge="0" Isotope="0"/>'
	'</fragment></page></CDXML>'
)
UNSUPPORTED_CDXML = (
	'<CDXML><page><fragment id="FERRUM_CDXML_PRIVATE_SOURCE">'
	'<n id="source-atom" p="0 0" Charge="+1"/>'
	'</fragment></page></CDXML>'
)


#============================================
class CdxmlOpenE2eError(RuntimeError):
	"""Report one broken public CDXML-open contract."""


#============================================
def parse_args() -> argparse.Namespace:
	"""Read the already-built Ferrum executable path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", required=True, type=Path)
	arguments = parser.parse_args()
	return arguments


#============================================
def run_open(
		ferrum: Path, source: Path, destination: Path,
		) -> subprocess.CompletedProcess[str]:
	"""Run one explicit CDXML open through the public JSON envelope."""
	result = subprocess.run(
		[
			str(ferrum), "open", str(source), "--format", "cdxml", "--output",
			str(destination), "--json",
		],
		text=True,
		stdout=subprocess.PIPE,
		stderr=subprocess.PIPE,
		check=False,
	)
	return result


#============================================
def main() -> None:
	"""Open producer-style CDXML and prove the public success and refusal contracts."""
	arguments = parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise CdxmlOpenE2eError("--ferrum must name an existing executable")
	formats = subprocess.run(
		[str(ferrum), "formats", "--json"], text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if formats.returncode != 0 or formats.stderr:
		raise CdxmlOpenE2eError("formats did not complete cleanly")
	catalog = json.loads(formats.stdout)
	capabilities = catalog["capabilities"]
	cdxml_capability = next(
		capability for capability in capabilities
		if capability["input"]["canonical_name"] == "cdxml"
	)
	cdxml_input = cdxml_capability["input"]
	if (
		cdxml_input["canonical_name"] != "cdxml"
		or cdxml_input["aliases"] != ["cdxml"]
		or cdxml_input["suffixes"] != [".cdxml"]
		or cdxml_input["operations"] != ["document_import_new"]
		or cdxml_input["runtime_requirement"] is not None
		or cdxml_capability["output"] is not None
	):
		raise CdxmlOpenE2eError("formats did not describe the CDXML input-only capability")
	with tempfile.TemporaryDirectory(prefix="ferrum-cdxml-open-") as directory:
		temporary = Path(directory).resolve()
		source = temporary / "molecule.cdxml"
		destination = temporary / "opened.cdml"
		source.write_text(CDXML, encoding="utf-8")
		result = run_open(ferrum, source, destination)
		if result.returncode != 0 or result.stderr:
			raise CdxmlOpenE2eError(f"CDXML open failed: {result.stderr.strip()}")
		envelope = json.loads(result.stdout)
		summary = envelope["outcome"]["summary"]
		cdml = destination.read_text(encoding="utf-8")
		if (
			envelope["schema"] != "ferrum-operation-response-v1"
			or envelope["outcome"]["kind"] != "document.molecule.interchange.import.v1"
			or summary["format_id"] != "cdxml"
			or summary["loss_report"]["dropped_categories"]
			!= ["lexical_syntax", "document_view_metadata"]
			or CDXML in result.stdout
			or not cdml.startswith("<cdml")
			or 'charge="-1"' not in cdml
			or 'isotope="18"' not in cdml
		):
			raise CdxmlOpenE2eError("CDXML open did not preserve the public receipt contract")
		zero_scalar_source = temporary / "zero-scalars.cdxml"
		zero_scalar_destination = temporary / "zero-scalars.cdml"
		zero_scalar_source.write_text(ZERO_SCALAR_CDXML, encoding="utf-8")
		zero_scalar_result = run_open(
			ferrum, zero_scalar_source, zero_scalar_destination,
		)
		if zero_scalar_result.returncode != 0 or zero_scalar_result.stderr:
			raise CdxmlOpenE2eError(
				"CDXML explicit zero scalar open failed"
			)
		zero_scalar_cdml = zero_scalar_destination.read_text(encoding="utf-8")
		if (
			'charge=' in zero_scalar_cdml
			or 'isotope=' in zero_scalar_cdml
		):
			raise CdxmlOpenE2eError(
				"CDXML explicit zero scalar facts did not omit CDML attributes"
			)
		refused_source = temporary / "unsupported.cdxml"
		refused_destination = temporary / "refused.cdml"
		refused_source.write_text(UNSUPPORTED_CDXML, encoding="utf-8")
		refusal = run_open(ferrum, refused_source, refused_destination)
		refusal_envelope = json.loads(refusal.stdout)
		if (
			refusal.returncode != 1
			or refusal.stderr
			or refusal_envelope["schema"] != "ferrum-operation-error-v1"
			or refusal_envelope["error"]["operation"]
			!= "document.molecule.interchange.import.v1"
			or refusal_envelope["error"]["category"] != "conversion_failed"
			or refusal_envelope["error"]["message"]
			!= "interchange_import_refused:InvalidScalar"
			or "FERRUM_CDXML_PRIVATE_SOURCE" in refusal.stdout
			or "Charge" in refusal.stdout
			or refused_destination.exists()
		):
			raise CdxmlOpenE2eError("CDXML refusal did not remain typed, redacted, and mutation-free")


if __name__ == "__main__":
	main()
