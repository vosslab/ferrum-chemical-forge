"""Exercise real multi-root SDF document export through the local Ferrum CLI."""

# Standard Library
import argparse
import json
from pathlib import Path
import subprocess
import tempfile


CDML = """<cdml xmlns="urn:ferrum:cdml" version="26.08">
<molecule id="left" name="Left"><atom id="left-c" name="C"><point x="0" y="0"/></atom></molecule>
<molecule id="right" name="Right"><atom id="right-o" name="O"><point x="80" y="0"/></atom></molecule>
</cdml>"""


#============================================
class DocumentExportSdfE2eError(RuntimeError):
	"""Report one public document SDF CLI contract failure."""


#============================================
def parse_args() -> argparse.Namespace:
	"""Read the already-built local Ferrum executable path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--ferrum", required=True, type=Path)
	return parser.parse_args()


#============================================
def invoke(
	ferrum: Path, source: Path, destination: Path, version: str, *molecule_ids: str,
) -> subprocess.CompletedProcess[str]:
	"""Run one real selected-root export with explicit source IDs and SDF syntax."""
	command = [str(ferrum), "document", "export-sdf", "--input", str(source)]
	for molecule_id in molecule_ids:
		command.extend(["--molecule-id", molecule_id])
	command.extend(["--version", version, "--output", str(destination)])
	return subprocess.run(
		command, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
	)


#============================================
def record_titles(sdf: str) -> list[str]:
	"""Read the public title line of every complete SDF record."""
	records = [record for record in sdf.split("$$$$\n") if record]
	return [record.split("\n", 1)[0] for record in records]


#============================================
def reimported_record_smiles(ferrum: Path, source: Path) -> list[str]:
	"""Re-open each published SDF record through the normal CLI interchange parser."""
	records = [record for record in source.read_text(encoding="utf-8").split("$$$$\n") if record]
	smiles: list[str] = []
	for index, record in enumerate(records):
		record_source = source.with_name(f"{source.stem}-record-{index}.sdf")
		record_source.write_text(f"{record}$$$$\n", encoding="utf-8")
		converted = subprocess.run(
			[str(ferrum), "convert", str(record_source), "--to", "smiles", "--json"],
			text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
		)
		if converted.returncode != 0:
			raise DocumentExportSdfE2eError(
				f"SDF record re-import did not convert to SMILES: {converted.stderr.strip()}"
			)
		try:
			outcome = json.loads(converted.stdout)["outcome"]
			text = outcome["text"]
		except (json.JSONDecodeError, KeyError, TypeError) as error:
			raise DocumentExportSdfE2eError("SDF record re-import returned no conversion text") from error
		if not isinstance(text, str):
			raise DocumentExportSdfE2eError("SDF record re-import returned non-text conversion output")
		smiles.append(text.strip())
	return smiles


#============================================
def check_successful_export(
	ferrum: Path, source: Path, destination: Path, version: str, syntax: str,
) -> None:
	"""Prove one requested syntax produces canonically ordered complete records."""
	success = invoke(ferrum, source, destination, version, "right", "left")
	if success.returncode != 0:
		raise DocumentExportSdfE2eError(f"{version} selected SDF export failed: {success.stderr.strip()}")
	sdf = destination.read_text(encoding="utf-8")
	if record_titles(sdf) != ["Left", "Right"]:
		raise DocumentExportSdfE2eError(f"{version} records did not use Rust canonical document order")
	if syntax not in sdf:
		raise DocumentExportSdfE2eError(f"{version} export did not contain its requested SDF syntax")
	if version == "v2000" and reimported_record_smiles(ferrum, destination) != ["C", "O"]:
		raise DocumentExportSdfE2eError(
			"SDF record re-import did not retain canonical C then O molecule identity"
		)


#============================================
def main() -> None:
	"""Prove both SDF dialects plus atomic refusal using the local CLI."""
	arguments = parse_args()
	ferrum = arguments.ferrum.resolve()
	if not ferrum.is_file():
		raise DocumentExportSdfE2eError("--ferrum must name an existing executable")
	with tempfile.TemporaryDirectory(prefix="ferrum-document-export-sdf-") as directory:
		temp = Path(directory).resolve()
		source = temp / "document.cdml"
		source.write_text(CDML, encoding="utf-8")
		check_successful_export(ferrum, source, temp / "selected-v2000.sdf", "v2000", "V2000")
		check_successful_export(ferrum, source, temp / "selected-v3000.sdf", "v3000", "V3000")
		refused_destination = temp / "refused.sdf"
		seeded_destination = b"existing destination must survive refusal\n"
		refused_destination.write_bytes(seeded_destination)
		refused = invoke(ferrum, source, refused_destination, "v3000", "left", "missing")
		if refused.returncode == 0 or not refused.stderr:
			raise DocumentExportSdfE2eError("typed SDF refusal did not report an export error")
		if refused_destination.read_bytes() != seeded_destination:
			raise DocumentExportSdfE2eError("typed SDF refusal changed the existing destination")


if __name__ == "__main__":
	main()
