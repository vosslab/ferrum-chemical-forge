"""Coverage contract for Ferrum's immutable V2 glyph-bond visual corpus."""

# Standard Library
import json
import pathlib
import xml.etree.ElementTree as element_tree
from typing import cast

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


# ============================================
def _fixtures() -> list[dict[str, object]]:
	"""Return the closed authored fixture rows without runner-generated defaults."""
	value = _read_fixtures()
	return cast(list[dict[str, object]], value["fixtures"])


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
		root = element_tree.fromstring(cast(str, fixture["fixture_cdml"]))
		molecule = next(node for node in root if node.tag.endswith("molecule"))
		assert molecule.attrib["id"] == fixture["fixture_id"] or fixture["fixture_id"] in _source_render_cases()
		source_atoms = {node.attrib["id"] for node in molecule if node.tag.endswith("atom")}
		source_bonds = {node.attrib["id"] for node in molecule if node.tag.endswith("bond")}
		graph = cast(dict[str, list[dict[str, str]]], fixture["graph"])
		assert source_atoms == {row["atom_id"] for row in graph["atoms"]}
		assert source_bonds == {row["bond_id"] for row in graph["bonds"]}


# ============================================
def _source_render_cases() -> dict[str, dict[str, object]]:
	"""Load exactly the renderable authority rows mirrored by the V2 catalog."""
	value = json.loads(_RUST_CORPUS_PATH.read_text(encoding="utf-8"))
	return {row["name"]: row for row in value["cases"] if row["expected_outcome"] == "render"}


# ============================================
def test_v2_mirrors_every_renderable_rust_alignment_case_exactly() -> None:
	"""The V2 consumer catalog has no handwritten ID, CDML, or source-style mapping."""
	fixtures = _by_id()
	for name, source in _source_render_cases().items():
		fixture = fixtures[name]
		assert fixture["fixture_cdml"] == source["cdml"]
		graph = cast(dict[str, list[dict[str, str]]], fixture["graph"])
		assert graph["atoms"] == [{"atom_id": atom["source_id"], "element": atom["core_run"]} for atom in source["atoms"]]
		expected_bonds = []
		for bond in source["bonds"]:
			style = _STYLE_MAP[bond["style"]]
			if bond["source_id"] == "wedge" and bond["display_layer"] == "haworth_front_wedge":
				style = "haworth-front-wedge"
			root = element_tree.fromstring(source["cdml"])
			endpoints = next(node for node in root.iter() if node.tag.endswith("bond") and node.attrib["id"] == bond["source_id"])
			expected_bonds.append({"bond_id": bond["source_id"], "start_atom_id": endpoints.attrib["start"], "end_atom_id": endpoints.attrib["end"], "style": style})
		assert graph["bonds"] == expected_bonds


# ============================================
def test_fixture_corpus_declares_chemical_directional_and_decoration_coverage() -> None:
	"""One durable corpus covers ordinary connected molecules and decorated labels."""
	fixtures = _by_id()
	assert {"bold_dashed_wavy_maximum_footprint", "ring_direction_cover"} <= set(fixtures)
	elements = {atom["element"] for fixture in _fixtures() for atom in cast(dict[str, list[dict[str, str]]], fixture["graph"])["atoms"]}
	assert _REQUIRED_ELEMENTS <= elements
	compass = cast(dict[str, list[dict[str, str]]], fixtures["eight_endpoint_directions_visible_elements"]["graph"])
	assert {bond["bond_id"] for bond in compass["bonds"]} == _DIRECTION_IDS
	for fixture_id in {"chlorine_normal_horizontal_mask", "isotope_carbon_hydrogen_charge", "bromine_hydrogen_charge_vertical", "phosphorus_isotope_hydrogen_charge_mask"}:
		assert fixture_id in fixtures


# ============================================
def test_fixture_corpus_declares_each_style_and_separate_haworth_front_forms() -> None:
	"""Distinct Haworth stroke and wedge identities cannot collapse into one style."""
	fixtures = _by_id()
	styles = {bond["style"] for fixture in _fixtures() for bond in cast(dict[str, list[dict[str, str]]], fixture["graph"])["bonds"]}
	assert _REQUIRED_STYLES <= styles
	haworth = cast(dict[str, list[dict[str, str]]], fixtures["haworth_front_stroke_and_wedge"]["graph"])
	assert {bond["style"] for bond in haworth["bonds"]} == {"haworth-front-stroke", "haworth-front-wedge"}


# ============================================
def test_fixture_corpus_declares_exact_adversarial_categories_and_crowded_near_miss() -> None:
	"""The catalog makes each visual defect a named rejection, not a vague bad case."""
	fixtures = _by_id()
	negatives = {row["expectation"] for fixture in _fixtures() for row in cast(list[dict[str, str]], fixture["negative_cases"])}
	assert negatives == _REQUIRED_NEGATIVES
	near_miss = cast(list[dict[str, str]], fixtures["third_label_near_miss"]["expected_relations"])
	assert {row["expectation"] for row in near_miss} >= {"normal_scale_connected", "clear_label"}
