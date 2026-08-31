"""Coverage contract for Ferrum's immutable V2 glyph-bond visual corpus."""

# Standard Library
import json
import pathlib

# PIP3 modules
import lxml.etree

# Local modules
from measure_stack.runner import FIXTURE_PATH, _read_fixtures


_RUST_CORPUS_PATH = pathlib.Path("packages/ferrum-rust/crates/document/tests/fixtures/atom_label_bond_alignment_cases_v1.json")
_STYLE_MAP = {
	"n1": "normal",
	"n2": "double",
	"n3": "triple",
	"w1": "solid-wedge",
	"h1": "hashed-wedge",
	"q1": "haworth-front-stroke",
	"b1": "bold",
	"d1": "dashed",
	"s1": "wavy",
}
_REQUIRED_ELEMENTS = {"C", "N", "O", "S", "P", "F", "Cl", "Br", "I"}
_REQUIRED_STYLES = {
	"normal", "double", "triple", "dashed", "bold", "wavy", "solid-wedge",
	"hashed-wedge", "haworth-front-stroke", "haworth-front-wedge",
}
_REQUIRED_NEGATIVES = {
	"must_reject_detached_gap", "must_reject_target_overlap",
	"must_reject_centerline_miss", "must_reject_style_topology",
	"must_reject_cropped_scene", "must_reject_orphaned_atom", "must_reject_collision",
}
_DIRECTION_IDS = {
	"bond_east", "bond_north_east", "bond_north", "bond_north_west",
	"bond_west", "bond_south_west", "bond_south", "bond_south_east",
}

# ASVS 1.5.1: disable DTDs, external entities, network access, recovery, and huge trees.
_XML_PARSER = lxml.etree.XMLParser(
	load_dtd=False,
	resolve_entities=False,
	no_network=True,
	recover=False,
	huge_tree=False,
)


# ============================================
def _mapping(value: object, label: str) -> dict[str, object]:
	"""Return one string-keyed fixture mapping or fail at the source boundary."""
	if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
		raise TypeError(f"{label} must be a string-keyed mapping")
	return value


# ============================================
def _mapping_rows(value: object, label: str) -> list[dict[str, object]]:
	"""Return one list of checked fixture mappings."""
	if not isinstance(value, list):
		raise TypeError(f"{label} must be a list")
	return [_mapping(row, label) for row in value]


# ============================================
def _text(value: object, label: str) -> str:
	"""Return one required fixture string."""
	if not isinstance(value, str):
		raise TypeError(f"{label} must be a string")
	return value


# ============================================
def _fixtures() -> list[dict[str, object]]:
	"""Return the closed authored fixture rows without runner-generated defaults."""
	value = _read_fixtures()
	return _mapping_rows(value["fixtures"], "fixture catalog")


# ============================================
def _by_id() -> dict[str, dict[str, object]]:
	"""Index the closed catalog by its durable fixture identity."""
	return {str(row["fixture_id"]): row for row in _fixtures()}


# ============================================
def test_fixture_corpus_is_ascii_and_every_fixture_cdml_owns_its_graph() -> None:
	"""Fixture sources are portable and graph IDs cannot drift from their CDML source."""
	text = FIXTURE_PATH.read_text(encoding="ascii")
	assert json.loads(text)["schema"] == "ferrum-measure-stack-fixtures-v2"
	for fixture in _fixtures():
		root = lxml.etree.fromstring(
			_text(fixture["fixture_cdml"], "fixture CDML").encode("utf-8"),
			parser=_XML_PARSER,
		)
		molecule = next(node for node in root if node.tag.endswith("molecule"))
		assert molecule.attrib["id"] == fixture["fixture_id"] or fixture["fixture_id"] in _source_render_cases()
		source_atoms = {node.attrib["id"] for node in molecule if node.tag.endswith("atom")}
		source_bonds = {node.attrib["id"] for node in molecule if node.tag.endswith("bond")}
		graph = _mapping(fixture["graph"], "fixture graph")
		atoms = _mapping_rows(graph["atoms"], "fixture atoms")
		bonds = _mapping_rows(graph["bonds"], "fixture bonds")
		assert source_atoms == {row["atom_id"] for row in atoms}
		assert source_bonds == {row["bond_id"] for row in bonds}


# ============================================
def _source_render_cases() -> dict[str, dict[str, object]]:
	"""Load exactly the renderable authority rows mirrored by the V2 catalog."""
	value = _mapping(
		json.loads(_RUST_CORPUS_PATH.read_text(encoding="utf-8")), "Rust corpus",
	)
	cases = _mapping_rows(value["cases"], "Rust corpus cases")
	return {str(row["name"]): row for row in cases if row["expected_outcome"] == "render"}


# ============================================
def test_v2_mirrors_every_renderable_rust_alignment_case_exactly() -> None:
	"""The V2 consumer catalog has no handwritten ID, CDML, or source-style mapping."""
	fixtures = _by_id()
	for name, source in _source_render_cases().items():
		fixture = fixtures[name]
		assert fixture["fixture_cdml"] == source["cdml"]
		graph = _mapping(fixture["graph"], "fixture graph")
		atoms = _mapping_rows(graph["atoms"], "fixture atoms")
		bonds = _mapping_rows(graph["bonds"], "fixture bonds")
		expected_bonds = []
		for bond in _mapping_rows(source["bonds"], "Rust corpus bonds"):
			style = _STYLE_MAP[bond["style"]]
			if bond["source_id"] == "wedge" and bond["display_layer"] == "haworth_front_wedge":
				style = "haworth-front-wedge"
			root = lxml.etree.fromstring(
				_text(source["cdml"], "Rust corpus CDML").encode("utf-8"),
				parser=_XML_PARSER,
			)
			endpoints = next(node for node in root.iter() if node.tag.endswith("bond") and node.attrib["id"] == bond["source_id"])
			expected_bonds.append({"bond_id": bond["source_id"], "start_atom_id": endpoints.attrib["start"], "end_atom_id": endpoints.attrib["end"], "style": style})
		assert atoms == [{"atom_id": atom["source_id"], "element": atom["core_run"]} for atom in _mapping_rows(source["atoms"], "Rust corpus atoms")]
		assert bonds == expected_bonds


# ============================================
def test_fixture_corpus_declares_chemical_directional_and_decoration_coverage() -> None:
	"""One durable corpus covers ordinary connected molecules and decorated labels."""
	fixtures = _by_id()
	assert {"bold_dashed_wavy_maximum_footprint", "ring_direction_cover"} <= set(fixtures)
	elements = {
		atom["element"] for fixture in _fixtures()
		for atom in _mapping_rows(_mapping(fixture["graph"], "fixture graph")["atoms"], "fixture atoms")
	}
	assert _REQUIRED_ELEMENTS <= elements
	compass = _mapping(fixtures["eight_endpoint_directions_visible_elements"]["graph"], "compass graph")
	assert {bond["bond_id"] for bond in _mapping_rows(compass["bonds"], "compass bonds")} == _DIRECTION_IDS
	for fixture_id in {"chlorine_normal_horizontal_mask", "isotope_carbon_hydrogen_charge", "bromine_hydrogen_charge_vertical", "phosphorus_isotope_hydrogen_charge_mask"}:
		assert fixture_id in fixtures


# ============================================
def test_fixture_corpus_declares_each_style_and_separate_haworth_front_forms() -> None:
	"""Distinct Haworth stroke and wedge identities cannot collapse into one style."""
	fixtures = _by_id()
	styles = {
		bond["style"] for fixture in _fixtures()
		for bond in _mapping_rows(_mapping(fixture["graph"], "fixture graph")["bonds"], "fixture bonds")
	}
	assert _REQUIRED_STYLES <= styles
	haworth = _mapping(fixtures["haworth_front_stroke_and_wedge"]["graph"], "Haworth graph")
	assert {bond["style"] for bond in _mapping_rows(haworth["bonds"], "Haworth bonds")} == {"haworth-front-stroke", "haworth-front-wedge"}


# ============================================
def test_fixture_corpus_declares_exact_adversarial_categories_and_crowded_near_miss() -> None:
	"""The catalog makes each visual defect a named rejection, not a vague bad case."""
	fixtures = _by_id()
	negatives = {
		row["expectation"] for fixture in _fixtures()
		for row in _mapping_rows(fixture["negative_cases"], "fixture negative cases")
	}
	assert negatives == _REQUIRED_NEGATIVES
	near_miss = _mapping_rows(
		fixtures["third_label_near_miss"]["expected_relations"], "near-miss relations",
	)
	assert {row["expectation"] for row in near_miss} >= {"normal_scale_connected", "clear_label"}
