#!/usr/bin/env python3
"""Compare Ferrum's corpus projection with the isolated historical oracle."""

# Standard Library
import argparse
import json
import pathlib
import subprocess

# PIP3 modules
import defusedxml.ElementTree


CAPABILITY = "corpus-molecule-core"
REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
CORPUS_DIR = REPO_ROOT / "tests" / "e2e" / "corpus"
ORACLE_PYTHON = REPO_ROOT / "tests" / "e2e" / "oracle" / ".venv" / "bin" / "python"
ORACLE_CHILD = REPO_ROOT / "tests" / "e2e" / "oracle" / "e2e_oasa_corpus_molecule_child.py"
DEFAULT_REPORT = (
	REPO_ROOT / "docs" / "active_plans" / "reports" / "corpus_molecule_parity.json"
)
ORACLE_REQUIREMENTS = pathlib.Path("tests/e2e/oracle/pip_requirements.txt")
ATOM_EXACT_FIELDS = ("index", "id", "symbol", "element", "x", "y", "z")
ATOM_OPTIONAL_FIELDS = (
	"formal_charge",
	"explicit_hydrogens",
	"isotope",
	"valence",
	"multiplicity",
	"free_sites",
)
BOND_EXACT_FIELDS = ("id", "start", "end", "type")
SOURCE_BOND_FIELDS = ("id", "start", "end", "start_kind", "end_kind", "type")
SOURCE_VERTEX_CLASSES = (
	("atom", "atoms"),
	("group", "groups"),
	("text", "texts"),
	("query", "queries"),
)


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse oracle environment, report path, and diagnostic mutation controls."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"-r",
		"--report",
		dest="report_path",
		type=pathlib.Path,
		default=DEFAULT_REPORT,
		help="JSON report path",
	)
	mutation_group = parser.add_mutually_exclusive_group()
	mutation_group.add_argument(
		"--mutate-ferrum",
		action="store_true",
		help="change one Ferrum atom symbol to prove the comparison catches drift",
	)
	mutation_group.add_argument(
		"--mutate-ferrum-non-atom",
		action="store_true",
		help="remove one Ferrum non-atom vertex to prove direct-source checking catches drift",
	)
	args = parser.parse_args()
	return args


#============================================
def corpus_paths() -> tuple[pathlib.Path, ...]:
	"""Return the complete committed corpus in stable filename order."""
	paths = tuple(sorted(CORPUS_DIR.glob("*.cdml")))
	if not paths:
		raise RuntimeError("the committed CDML corpus is empty")
	return paths


#============================================
def child_result(command: list[str], request_text: str | None = None) -> dict:
	"""Run one worker and validate its exactly-one-object stdout protocol."""
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
def rust_command(corpus_path: pathlib.Path) -> list[str]:
	"""Return the self-contained Rust projection command for one corpus file."""
	relative_path = corpus_path.relative_to(REPO_ROOT).as_posix()
	command = [
		"cargo",
		"run",
		"--quiet",
		"--manifest-path",
		"packages/ferrum-rust/Cargo.toml",
		"--package",
		"ferrum-document",
		"--example",
		"corpus_projection",
		"--",
		relative_path,
	]
	return command


#============================================
def oracle_request(corpus_path: pathlib.Path) -> str:
	"""Return the stable one-object request for the historical worker."""
	request = {
		"capability": CAPABILITY,
		"corpus_path": corpus_path.relative_to(REPO_ROOT).as_posix(),
	}
	request_text = json.dumps(request, separators=(",", ":"), sort_keys=True)
	return request_text


#============================================
def xml_local_name(tag: str) -> str:
	"""Return an ElementTree tag's namespace-independent local name."""
	if tag.startswith("{"):
		return tag.split("}", maxsplit=1)[1]
	return tag


#============================================
def source_non_atom_facts(corpus_path: pathlib.Path) -> list[dict]:
	"""Read direct CDML facts the chemistry-only historical oracle cannot expose."""
	root = defusedxml.ElementTree.parse(corpus_path).getroot()
	molecule_facts = []
	for molecule in root:
		if xml_local_name(molecule.tag) != "molecule":
			continue
		vertices = {field: [] for _, field in SOURCE_VERTEX_CLASSES}
		endpoint_positions = {}
		for record in molecule:
			local_name = xml_local_name(record.tag)
			for vertex_class, field in SOURCE_VERTEX_CLASSES:
				if local_name != vertex_class:
					continue
				index = len(vertices[field])
				identifier = record.attrib["id"]
				vertices[field].append({"id": identifier, "index": index})
				endpoint_positions[identifier] = (vertex_class, index)
		non_atom_bonds = []
		for record in molecule:
			if xml_local_name(record.tag) != "bond":
				continue
			start_kind, start_index = endpoint_positions[record.attrib["start"]]
			end_kind, end_index = endpoint_positions[record.attrib["end"]]
			if start_kind == "atom" and end_kind == "atom":
				continue
			non_atom_bonds.append(
				{
					"id": record.attrib.get("id"),
					"start": start_index,
					"end": end_index,
					"start_kind": start_kind,
					"end_kind": end_kind,
					"type": record.attrib.get("type"),
				}
			)
		molecule_facts.append(
			{
				"groups": vertices["groups"],
				"texts": vertices["texts"],
				"queries": vertices["queries"],
				"typed_non_atom_bonds": non_atom_bonds,
			}
		)
	return molecule_facts


#============================================
def compare_exact(
		unexpected: list[dict], agreements: list[str], path: str,
		oracle_value: object, ferrum_value: object,
		) -> None:
	"""Record one exact agreement or an unclassified difference."""
	if oracle_value == ferrum_value:
		agreements.append(path)
		return
	unexpected.append({"path": path, "oracle": oracle_value, "ferrum": ferrum_value})


#============================================
def classify(
		classified: list[dict], path: str, oracle_value: object,
		ferrum_value: object, classification: str, basis: str,
		) -> None:
	"""Record one expected or unverifiable difference with its authority."""
	classified.append(
		{
			"path": path,
			"oracle": oracle_value,
			"ferrum": ferrum_value,
			"classification": classification,
			"basis": basis,
		}
	)


#============================================
def compare_atom(
		oracle_atom: dict, ferrum_atom: dict, path: str,
		agreements: list[str], classified: list[dict], unexpected: list[dict],
		) -> None:
	"""Compare exact source facts and classify oracle-supplied defaults."""
	for field in ATOM_EXACT_FIELDS:
		compare_exact(
			unexpected, agreements, path + "." + field,
			oracle_atom.get(field), ferrum_atom.get(field),
		)
	for field in ATOM_OPTIONAL_FIELDS:
		field_path = path + "." + field
		ferrum_value = ferrum_atom.get(field)
		oracle_value = oracle_atom.get(field)
		if ferrum_value is None and oracle_value is not None:
			classify(
				classified,
				field_path,
				oracle_value,
				ferrum_value,
				"intended-source-presence",
				"CDML omitted the field; Ferrum preserves absence while the oracle computes a default",
			)
		else:
			compare_exact(unexpected, agreements, field_path, oracle_value, ferrum_value)


#============================================
def atom_only_bonds(ferrum_molecule: dict) -> tuple[list[dict], list[dict]]:
	"""Split bonds the historical reader can represent from typed extra bonds."""
	comparable = []
	extra = []
	for bond in ferrum_molecule["bonds"]:
		if bond["start_kind"] == "atom" and bond["end_kind"] == "atom":
			comparable.append(bond)
		else:
			extra.append(bond)
	return comparable, extra


#============================================
def compare_bond(
		oracle_bond: dict, ferrum_bond: dict, document_version: str | None,
		path: str, agreements: list[str], classified: list[dict],
		unexpected: list[dict],
		) -> None:
	"""Compare one atom-only bond under the versioned CDML token contract."""
	for field in BOND_EXACT_FIELDS:
		compare_exact(
			unexpected, agreements, path + "." + field,
			oracle_bond.get(field), ferrum_bond.get(field),
		)
	order_path = path + ".order"
	if (
			document_version == "0.8"
			and ferrum_bond.get("type") == "d"
			and oracle_bond.get("order") == 1
			and ferrum_bond.get("order") == 2
			):
		classify(
			classified,
			order_path,
			1,
			2,
			"intended-format-correction",
			"CDML 0.8 uses single-letter d as normal double, above current reader behavior",
		)
	else:
		compare_exact(
			unexpected, agreements, order_path,
			oracle_bond.get("order"), ferrum_bond.get("order"),
		)
	classify(
		classified,
		path + ".style",
		None,
		ferrum_bond.get("style"),
		"unverifiable-oracle-field",
		"the historical projection exposes no independent Ferrum bond-style enum",
	)
	classify(
		classified,
		path + ".aromatic",
		None,
		ferrum_bond.get("aromatic"),
		"unverifiable-oracle-field",
		"the historical CDML reader carries no aromatic source-presence flag",
	)


#============================================
def compare_molecule(
		oracle_molecule: dict, ferrum_molecule: dict, document_version: str | None,
		source_molecule: dict, path: str, agreements: list[str], classified: list[dict],
		unexpected: list[dict],
		) -> None:
	"""Compare one molecule against the oracle and direct source facts."""
	for field in ("index", "id", "name"):
		compare_exact(
			unexpected, agreements, path + "." + field,
			oracle_molecule.get(field), ferrum_molecule.get(field),
		)
	oracle_atoms = oracle_molecule["atoms"]
	ferrum_atoms = ferrum_molecule["atoms"]
	compare_exact(
		unexpected, agreements, path + ".atoms.length",
		len(oracle_atoms), len(ferrum_atoms),
	)
	for index, (oracle_atom, ferrum_atom) in enumerate(zip(oracle_atoms, ferrum_atoms)):
		compare_atom(
			oracle_atom, ferrum_atom, path + f".atoms[{index}]",
			agreements, classified, unexpected,
		)
	comparable_bonds, extra_bonds = atom_only_bonds(ferrum_molecule)
	oracle_bonds = oracle_molecule["bonds"]
	compare_exact(
		unexpected, agreements, path + ".atom_only_bonds.length",
		len(oracle_bonds), len(comparable_bonds),
	)
	for index, (oracle_bond, ferrum_bond) in enumerate(zip(oracle_bonds, comparable_bonds)):
		compare_bond(
			oracle_bond, ferrum_bond, document_version,
			path + f".atom_only_bonds[{index}]", agreements, classified, unexpected,
		)
	ferrum_extra_facts = [
		{field: bond[field] for field in SOURCE_BOND_FIELDS}
		for bond in extra_bonds
	]
	compare_exact(
		unexpected,
		agreements,
		path + ".typed_non_atom_bonds.source",
		source_molecule["typed_non_atom_bonds"],
		ferrum_extra_facts,
	)
	if extra_bonds:
		classify(
			classified,
			path + ".typed_non_atom_bonds",
			[],
			extra_bonds,
			"intended-core-scope",
			"direct CDML facts verify records that the historical chemistry reader drops",
		)
	for field in ("groups", "texts", "queries"):
		compare_exact(
			unexpected,
			agreements,
			path + "." + field + ".source",
			source_molecule[field],
			ferrum_molecule[field],
		)
		if ferrum_molecule[field]:
			classify(
				classified,
				path + "." + field,
				[],
				ferrum_molecule[field],
				"unverifiable-oracle-field",
				"direct CDML facts verify the vertex; the historical reader has no corresponding class",
			)


#============================================
def compare_projection(oracle: dict, ferrum: dict, source_molecules: list[dict]) -> dict:
	"""Return exact agreements, classified differences, and unexpected drift."""
	agreements = []
	classified = []
	unexpected = []
	for field in ("capability", "coordinate_unit", "corpus_path"):
		compare_exact(
			unexpected, agreements, "projection." + field,
			oracle.get(field), ferrum.get(field),
		)
	oracle_molecules = oracle["molecules"]
	ferrum_molecules = ferrum["molecules"]
	compare_exact(
		unexpected, agreements, "projection.molecules.length",
		len(oracle_molecules), len(ferrum_molecules),
	)
	compare_exact(
		unexpected, agreements, "projection.molecules.source_length",
		len(source_molecules), len(ferrum_molecules),
	)
	document_version = ferrum.get("document_version")
	for index, (oracle_molecule, ferrum_molecule, source_molecule) in enumerate(
			zip(oracle_molecules, ferrum_molecules, source_molecules),
			):
		compare_molecule(
			oracle_molecule, ferrum_molecule, document_version, source_molecule,
			f"projection.molecules[{index}]", agreements, classified, unexpected,
		)
	result = {
		"agreements": agreements,
		"classified_differences": classified,
		"unexpected_differences": unexpected,
	}
	return result


#============================================
def write_report(path: pathlib.Path, report: dict) -> None:
	"""Write one deterministic ASCII JSON report."""
	path.parent.mkdir(parents=True, exist_ok=True)
	report_text = json.dumps(report, indent=2, sort_keys=True) + "\n"
	path.write_text(report_text, encoding="ascii")


#============================================
def main() -> None:
	"""Run every corpus case, then emit its report and process exit status."""
	args = parse_args()
	mutation_kind = None
	if args.mutate_ferrum:
		mutation_kind = "atom-element"
	elif args.mutate_ferrum_non_atom:
		mutation_kind = "non-atom-removal"
	report = {
		"capability": CAPABILITY,
		"corpus": [path.relative_to(REPO_ROOT).as_posix() for path in corpus_paths()],
		"comparison_rule": (
			"Oracle-comparable facts and direct-CDML non-atom facts must agree; "
			"source-absence defaults, oracle-unrepresented fields, and higher-authority "
			"format corrections are classified separately."
		),
		"ferrum_mutation": mutation_kind,
		"oracle_setup": [
			"python3", "-m", "pip", "install", "-r", str(ORACLE_REQUIREMENTS),
		],
		"results": [],
	}
	if not ORACLE_PYTHON.is_file():
		report["status"] = "harness-error"
		report["error"] = (
			"isolated oracle Python was not found; create tests/e2e/oracle/.venv "
			"and install tests/e2e/oracle/pip_requirements.txt"
		)
		write_report(args.report_path, report)
		print(json.dumps(report, sort_keys=True))
		raise SystemExit(2)
	mutation_applied = False
	for corpus_path in corpus_paths():
		request_text = oracle_request(corpus_path)
		oracle_output = child_result(
			[str(ORACLE_PYTHON), "-I", "-B", str(ORACLE_CHILD)], request_text,
		)
		ferrum_output = child_result(rust_command(corpus_path))
		molecules = ferrum_output["projection"]["molecules"]
		if args.mutate_ferrum and not mutation_applied and molecules and molecules[0]["atoms"]:
			molecules[0]["atoms"][0]["element"] = "MUTATED"
			mutation_applied = True
		if (
				args.mutate_ferrum_non_atom
				and not mutation_applied
				and molecules
				and molecules[0]["groups"]
				):
			molecules[0]["groups"].pop(0)
			mutation_applied = True
		source_molecules = source_non_atom_facts(corpus_path)
		comparison = compare_projection(
			oracle_output["projection"], ferrum_output["projection"], source_molecules,
		)
		report["results"].append(
			{
				"corpus_path": corpus_path.relative_to(REPO_ROOT).as_posix(),
				"facts": {
					"oracle": oracle_output["facts"],
					"ferrum": ferrum_output["facts"],
				},
				"source_non_atom_facts": source_molecules,
				"normalized_outputs": {
					"oracle": oracle_output["projection"],
					"ferrum": ferrum_output["projection"],
				},
				**comparison,
			}
		)
	if mutation_kind is not None and not mutation_applied:
		raise RuntimeError(f"no corpus record accepted the requested {mutation_kind} mutation")
	unexpected_count = sum(
		len(result["unexpected_differences"]) for result in report["results"]
	)
	report["unexpected_difference_count"] = unexpected_count
	report["status"] = (
		"match-with-classified-differences" if unexpected_count == 0 else "divergence"
	)
	write_report(args.report_path, report)
	print(json.dumps(report, sort_keys=True))
	if unexpected_count:
		raise SystemExit(1)


if __name__ == "__main__":
	main()
