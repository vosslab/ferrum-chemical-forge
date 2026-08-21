"""Installed-wheel contract for Ferrum-Chem's revisioned document boundary."""

from __future__ import annotations

import math
from pathlib import Path
import sys
import types

import pytest

import ferrum_chem


SOURCE = (
	"<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
	"<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)

BOND_SOURCE = (
	'<cdml version="26.08"><molecule id="m">'
	'<atom id="a" name="C"><point x="1" y="2"/></atom>'
	'<atom id="b" name="O"><point x="3" y="2"/></atom>'
	'</molecule></cdml>'
)

COORDINATE_SOURCE = (
	'<cdml version="26.08"><molecule id="m">'
	'<atom id="a" name="C"><point x="10" y="20"/></atom>'
	'<atom id="b" name="C"><point x="50" y="20"/></atom>'
	'<atom id="c" name="O"><point x="50" y="60"/></atom>'
	'<bond id="ab" start="a" end="b" type="n1"/>'
	'<bond id="bc" start="b" end="c" type="n1"/>'
	'</molecule></cdml>'
)

ATOM_PROPERTIES_SOURCE = (
	'<cdml xmlns:v="urn:vendor"><molecule id="m">'
	'<atom id="a" name="C" charge="2" valency="4" isotope="13" '
	'multiplicity="3" show="no" hydrogens="off" vendor_keep="yes">'
	'<point x="1" y="2"/><font family="Courier" size="11" '
	'vendor_keep="yes"/><ftext>keep</ftext><v:keep/></atom>'
	'</molecule><v:opaque id="retained"/></cdml>'
)

BOND_PROPERTIES_SOURCE = (
	'<cdml xmlns:v="urn:vendor"><molecule id="m">'
	'<atom id="a" name="C"><point x="1" y="2"/></atom>'
	'<atom id="b" name="O"><point x="20" y="2"/></atom>'
	'<bond id="ab" start="a" end="b" type="n1" line_width="1.5" '
	'bond_width="-2" wedge_width="3" color="#A0B1C2" vendor_keep="yes">'
	'<v:keep/></bond></molecule><v:opaque id="retained"/></cdml>'
)

HAWORTH_POSITION_SOURCE = (
	'<cdml><molecule id="m"><atom id="a" name="C"><point x="0" y="0"/></atom>'
	'<atom id="b" name="O"><point x="1" y="0"/></atom>'
	'<bond start="a" end="b" haworth_position="front"/>'
	'<bond start="b" end="a" haworth_position="back"/>'
	'<bond start="a" end="b" haworth_position="side"/></molecule></cdml>'
)


def set_atom(element: str) -> ferrum_chem.DocumentOperationV1:
	return ferrum_chem.DocumentOperationV1.set_atom_element("a", element)


def test_direct_bond_gesture_binding_commits_one_normal_bond() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	observation = session.observe(0)
	start = observation.projection.molecules[0].atoms[0].id
	end = observation.projection.molecules[0].atoms[1].id
	snap = ferrum_chem.DirectBondSnapPolicyV1()
	gesture = session.begin_direct_bond_gesture_v1(
		observation.snapshot.revision,
		observation.snapshot.digest,
		start,
		ferrum_chem.DocumentBondPresentationV1.normal_double,
		"C",
		snap,
	)
	preview = session.preview_direct_bond_gesture_v1(
		gesture, ferrum_chem.DirectBondEndIntentV1.existing_atom(end),
	)
	assert type(preview) is ferrum_chem.DirectBondPreviewV1
	commit = session.commit_direct_bond_gesture_v1(gesture, preview)
	assert commit.created_new_atom is False
	assert commit.result.observation.snapshot.revision == 1
	assert 'type="n2"' in commit.result.observation.snapshot.cdml


def test_presentation_vector_binding_keeps_frozen_failure_and_commit_contract() -> None:
	session = ferrum_chem.DocumentSession.load("<cdml><standard line_color='#123456' line_width='2'/></cdml>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_vector_gesture_v1(
		snapshot.revision,
		snapshot.digest,
		ferrum_chem.PresentationVectorKindV1.rectangle,
		10.0,
		20.0,
	)
	preview = session.preview_presentation_vector_gesture_v1(gesture, 30.0, 45.0)
	assert hasattr(session, "commit_presentation_vector_gesture_v1")
	assert not hasattr(session, "preflight_presentation_vector_gesture_v1")
	assert not hasattr(session, "commit_preflighted_presentation_vector_gesture_v1")
	prepared = session.prepare_presentation_vector_gesture_v1(gesture, preview)
	commit = session.commit_presentation_vector_gesture_v1(prepared)
	assert commit.result.observation.snapshot.revision == 1
	assert 'line_color="#123456"' in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.PresentationVectorGestureError) as captured:
		session.commit_presentation_vector_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.PresentationVectorGestureCategoryV1.replayed_gesture
	assert captured.value.recovery == ferrum_chem.PresentationVectorGestureRecoveryV1.refresh_and_restart


def test_reaction_creation_uses_only_the_renderer_preflighted_python_route() -> None:
	source = (
		'<cdml><molecule id="left"><atom id="left-a" name="C">'
		'<point x="0" y="0"/></atom></molecule><molecule id="product">'
		'<atom id="product-a" name="O"><point x="100" y="0"/></atom></molecule>'
		'<arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/>'
		'</arrow></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	commit = session.create_reaction_v1(0, ["left"], ["product"], "arrow", [], [])
	assert commit.reaction_id == "rxn-1"
	assert commit.result.observation.snapshot.revision == 1
	assert '<reaction id="rxn-1"' in commit.result.observation.snapshot.cdml
	assert not hasattr(session, "commit_raw_reaction_v1")
	assert not hasattr(session, "commit_preflighted_reaction_v1")
	with pytest.raises(ferrum_chem.ReactionGestureError) as captured:
		session.create_reaction_v1(1, ["missing"], ["product"], "arrow", [], [])
	assert captured.value.category == ferrum_chem.ReactionRefusalCategoryV1.missing_target
	assert captured.value.recovery == ferrum_chem.ReactionRefusalRecoveryV1.correct_selectors


def test_reaction_authoring_choices_are_renderer_fenced_and_non_mutating() -> None:
	source = (
		'<cdml><molecule id="left"><atom id="left-a" name="C">'
		'<point x="0" y="0"/></atom></molecule><molecule id="product">'
		'<atom id="product-a" name="O"><point x="100" y="0"/></atom></molecule>'
		'<arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/></arrow>'
		'<plus id="plus"><point x="20" y="-10"/></plus>'
		'<text id="condition"><point x="50" y="-20"/><ftext>heat</ftext></text>'
		'<rect id="annotation" x1="10" y1="10" x2="30" y2="30"/>'
		'<reaction id="rxn-old"><reactant idref="left"/></reaction>'
		'<plus><point x="0" y="20"/></plus>'
		'</cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	snapshot = session.snapshot()
	choices = session.observe_reaction_authoring_choices_v1(
		snapshot.revision, snapshot.digest,
	)
	assert choices.revision == snapshot.revision
	assert choices.digest == snapshot.digest
	assert [(item.identifier, item.kind, item.availability) for item in choices.choices] == [
		("left", ferrum_chem.ReactionAuthoringChoiceKindV1.molecule,
		 ferrum_chem.ReactionAuthoringChoiceAvailabilityV1.already_in_reaction),
		("product", ferrum_chem.ReactionAuthoringChoiceKindV1.molecule,
		 ferrum_chem.ReactionAuthoringChoiceAvailabilityV1.eligible),
		("arrow", ferrum_chem.ReactionAuthoringChoiceKindV1.arrow,
		 ferrum_chem.ReactionAuthoringChoiceAvailabilityV1.eligible),
		("plus", ferrum_chem.ReactionAuthoringChoiceKindV1.plus,
		 ferrum_chem.ReactionAuthoringChoiceAvailabilityV1.eligible),
		("condition", ferrum_chem.ReactionAuthoringChoiceKindV1.condition_text,
		 ferrum_chem.ReactionAuthoringChoiceAvailabilityV1.eligible),
	]
	annotation = next(
		item for item in choices.exclusions if item.diagnostic_key == "annotation"
	)
	assert annotation.reason == ferrum_chem.ReactionAuthoringExclusionReasonV1.display_only
	assert annotation.recovery == ferrum_chem.ReactionAuthoringExclusionRecoveryV1.choose_supported_member
	assert "<rect" not in annotation.label
	assert not hasattr(annotation, "cdml")
	session.validate_reaction_authoring_choices_v1(choices)
	commit = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_atom_element("left-a", "N"),
	)
	assert commit.observation.snapshot.revision == 1
	with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError) as captured:
		session.validate_reaction_authoring_choices_v1(choices)
	assert captured.value.category == ferrum_chem.ReactionAuthoringChoicesRefusalCategoryV1.stale_snapshot
	other = ferrum_chem.DocumentSession.load(source)
	with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError) as captured:
		other.validate_reaction_authoring_choices_v1(choices)
	assert captured.value.category == ferrum_chem.ReactionAuthoringChoicesRefusalCategoryV1.foreign_session


def test_presentation_vector_bridge_receipts_preflight_and_fence_every_python_path() -> None:
	standard = (
		'<cdml><standard line_color="#123456" line_width="2" '
		'area_color="#ABCDEF"/></cdml>'
	)
	first = ferrum_chem.DocumentSession.load(standard)
	second = ferrum_chem.DocumentSession.load(standard)
	first_snapshot = first.snapshot()
	second_snapshot = second.snapshot()
	first_gesture = first.begin_presentation_vector_gesture_v1(
		first_snapshot.revision,
		first_snapshot.digest,
		ferrum_chem.PresentationVectorKindV1.oval,
		10.0,
		20.0,
	)
	first_preview = first.preview_presentation_vector_gesture_v1(
		first_gesture, 40.0, 60.0,
	)
	assert first_preview.overlay.stroke_color == "#123456"
	assert first_preview.overlay.stroke_width == 2.0
	assert first_preview.overlay.fill_color == "#abcdef"

	# The former raw `(gesture, preview)` commit is not a client surface: only
	# one opaque prepared receipt is accepted by the Python bridge method.
	with pytest.raises(TypeError):
		first.commit_presentation_vector_gesture_v1(first_gesture, first_preview)
	assert not hasattr(first, "commit_raw_presentation_vector_gesture_v1")
	assert not hasattr(first, "commit_preflighted_presentation_vector_gesture_v1")

	with pytest.raises(ferrum_chem.PresentationVectorGestureError) as captured:
		second.prepare_presentation_vector_gesture_v1(first_gesture, first_preview)
	assert captured.value.category == ferrum_chem.PresentationVectorGestureCategoryV1.foreign_session
	assert second.snapshot().revision == second_snapshot.revision

	mismatch_gesture = first.begin_presentation_vector_gesture_v1(
		first_snapshot.revision,
		first_snapshot.digest,
		ferrum_chem.PresentationVectorKindV1.oval,
		10.0,
		20.0,
	)
	mismatch_preview = first.preview_presentation_vector_gesture_v1(mismatch_gesture, 40.0, 60.0)
	with pytest.raises(ferrum_chem.PresentationVectorGestureError) as captured:
		first.prepare_presentation_vector_gesture_v1(first_gesture, mismatch_preview)
	assert captured.value.category == ferrum_chem.PresentationVectorGestureCategoryV1.mismatched_preview
	assert first.snapshot().revision == first_snapshot.revision

	prepared = first.prepare_presentation_vector_gesture_v1(first_gesture, first_preview)
	commit = first.commit_presentation_vector_gesture_v1(prepared)
	assert commit.kind == ferrum_chem.PresentationVectorKindV1.oval
	assert commit.result.observation.snapshot.revision == 1
	assert 'line_color="#123456"' in commit.result.observation.snapshot.cdml
	assert 'width="2"' in commit.result.observation.snapshot.cdml
	assert 'area_color="#abcdef"' in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.PresentationVectorGestureError) as captured:
		first.commit_presentation_vector_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.PresentationVectorGestureCategoryV1.replayed_gesture

	excluded = ferrum_chem.DocumentSession.load(
		'<cdml><plus id="excluded"><point x="1" y="2"/>'
		'<font family="Arial"/></plus></cdml>',
	)
	excluded_snapshot = excluded.snapshot()
	excluded_gesture = excluded.begin_presentation_vector_gesture_v1(
		excluded_snapshot.revision,
		excluded_snapshot.digest,
		ferrum_chem.PresentationVectorKindV1.line,
		10.0,
		20.0,
	)
	excluded_preview = excluded.preview_presentation_vector_gesture_v1(
		excluded_gesture, 40.0, 60.0,
	)
	with pytest.raises(ferrum_chem.PresentationVectorGestureError) as captured:
		excluded.prepare_presentation_vector_gesture_v1(excluded_gesture, excluded_preview)
	assert captured.value.category == ferrum_chem.PresentationVectorGestureCategoryV1.render_preparation
	assert captured.value.recovery == ferrum_chem.PresentationVectorGestureRecoveryV1.document_unchanged
	assert excluded.snapshot().revision == excluded_snapshot.revision


def test_text_placement_binding_uses_renderer_overlay_and_one_commit() -> None:
	session = ferrum_chem.DocumentSession.load("<cdml><standard font_size='18' line_color='#123456'/></cdml>")
	snapshot = session.snapshot()
	gesture = session.begin_text_placement_gesture_v1(
		snapshot.revision, snapshot.digest, 10.0, 20.0,
	)
	defaults = session.text_placement_defaults_v1(gesture)
	assert defaults.font_size == 18.0
	assert defaults.color == "#123456"
	assert defaults.bold_supported is False
	run = ferrum_chem.DocumentTextEditRunV1.create
	style = ferrum_chem.DocumentTextEditStyleV1
	preview = session.preview_text_placement_gesture_v1(
		gesture, (run("H", ()), run("2", (style.subscript,)), run("O", ())), None, None,
	)
	assert preview.overlay.operation.size == 18.0
	commit = session.commit_text_placement_gesture_v1(gesture, preview)
	assert commit.result.observation.snapshot.revision == 1
	assert "<ftext>H&lt;sub&gt;2&lt;/sub&gt;O</ftext>" in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.TextPlacementError) as caught:
		session.commit_text_placement_gesture_v1(gesture, preview)
	assert caught.value.category == ferrum_chem.TextPlacementErrorCategoryV1.replayed_gesture
	assert ferrum_chem.TextPlacementErrorCategoryV1.unrenderable_standard
	assert ferrum_chem.TextPlacementRecoveryV1.repair_drawing_standard
	assert ferrum_chem.TextPlacementErrorCategoryV1.render_preparation
	assert ferrum_chem.TextPlacementRecoveryV1.recover_canvas


def test_text_placement_custom_standard_refuses_before_mutation() -> None:
	session = ferrum_chem.DocumentSession.load(
		'<cdml><standard font_family="No Such Face"/></cdml>',
	)
	snapshot = session.snapshot()
	gesture = session.begin_text_placement_gesture_v1(
		snapshot.revision, snapshot.digest, 1.0, 2.0,
	)
	run = ferrum_chem.DocumentTextEditRunV1.create("x", ())
	with pytest.raises(ferrum_chem.TextPlacementError) as caught:
		session.preview_text_placement_gesture_v1(gesture, (run,), None, None)
	assert caught.value.category == ferrum_chem.TextPlacementErrorCategoryV1.unrenderable_standard
	assert caught.value.recovery == ferrum_chem.TextPlacementRecoveryV1.repair_drawing_standard
	assert session.snapshot().revision == snapshot.revision


def test_direct_bond_gesture_preview_refusal_is_typed_and_non_mutating() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	observation = session.observe(0)
	start = observation.projection.molecules[0].atoms[0].id
	gesture = session.begin_direct_bond_gesture_v1(
		0,
		observation.snapshot.digest,
		start,
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	refusal = session.preview_direct_bond_gesture_v1(
		gesture, ferrum_chem.DirectBondEndIntentV1.existing_atom(start),
	)
	assert type(refusal) is ferrum_chem.DirectBondPreviewRefusalV1
	assert refusal.category == ferrum_chem.DirectBondGestureCategoryV1.self_loop
	assert refusal.recovery == ferrum_chem.DirectBondGestureRecoveryV1.adjust_endpoint
	assert session.snapshot().revision == 0


def test_direct_bond_gesture_binding_rejects_foreign_session_handles() -> None:
	first = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	second = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	first_observation = first.observe(0)
	second_observation = second.observe(0)
	first_start = first_observation.projection.molecules[0].atoms[0].id
	first_end = first_observation.projection.molecules[0].atoms[1].id
	second_start = second_observation.projection.molecules[0].atoms[0].id
	gesture = first.begin_direct_bond_gesture_v1(
		0,
		first_observation.snapshot.digest,
		first_start,
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	second_gesture = second.begin_direct_bond_gesture_v1(
		0,
		second_observation.snapshot.digest,
		second_start,
		ferrum_chem.DocumentBondPresentationV1.normal_single,
		"C",
		ferrum_chem.DirectBondSnapPolicyV1(),
	)
	preview = first.preview_direct_bond_gesture_v1(
		gesture, ferrum_chem.DirectBondEndIntentV1.existing_atom(first_end),
	)
	with pytest.raises(ferrum_chem.DirectBondGestureError) as captured:
		second.preview_direct_bond_gesture_v1(
			gesture, ferrum_chem.DirectBondEndIntentV1.existing_atom(first_end),
		)
	assert captured.value.category == ferrum_chem.DirectBondGestureCategoryV1.foreign_session
	with pytest.raises(ferrum_chem.DirectBondGestureError) as captured:
		second.commit_direct_bond_gesture_v1(second_gesture, preview)
	assert captured.value.recovery == ferrum_chem.DirectBondGestureRecoveryV1.refresh_and_restart


def test_structure_path_target_is_display_only_and_cannot_create_a_delete_handle() -> None:
	session = ferrum_chem.DocumentSession.load(
		"<cdml><molecule id=\"m\">"
		"<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>"
		"<atom id=\"b\" name=\"O\"><point x=\"30\" y=\"0\"/></atom>"
		"<bond id=\"ab\" type=\"w1\" start=\"a\" end=\"b\"/>"
		"</molecule></cdml>"
	)
	snapshot = session.snapshot()
	observation = session.observe_structure_interaction_v1(
		snapshot.revision, snapshot.digest
	)
	target = next(value for value in observation.targets if value.identifier == "ab")
	assert target.kind == ferrum_chem.StructureTargetKindV1.display_only
	with pytest.raises(ferrum_chem.RenderInteractionError) as caught:
		session.select_structure_interaction_v1(
			observation,
			None,
			ferrum_chem.StructureInteractionQueryV1.point(
				(target.bounds.left + target.bounds.right) / 2.0,
				(target.bounds.top + target.bounds.bottom) / 2.0,
				ferrum_chem.RenderInteractionModifierV1.replace,
			),
		)
	assert caught.value.category == ferrum_chem.RenderInteractionCategoryV1.display_only
	assert caught.value.recovery == ferrum_chem.RenderInteractionRecoveryV1.change_presentation


def test_smiles_input_precedes_native_availability_and_module_replacement() -> None:
	held_parse_smiles = ferrum_chem.parse_smiles

	for smiles in ("", "C\0O"):
		with pytest.raises(ferrum_chem.InvalidSmiles):
			held_parse_smiles(smiles)
	fake_module = types.ModuleType("ferrum_chem")
	fake_module.__file__ = "/private/tmp/not-shipping/fake_extension.so"
	original_module = sys.modules["ferrum_chem"]
	sys.modules["ferrum_chem"] = fake_module
	try:
		parsed = held_parse_smiles("C")
	finally:
		sys.modules["ferrum_chem"] = original_module

	assert parsed.canonical_smiles == "C"


def test_smiles_public_types_have_extension_module_provenance() -> None:
	public_types = (
		ferrum_chem.SmilesPoint2V1,
		ferrum_chem.SmilesAtomChiralityV1,
		ferrum_chem.SmilesBondOrderV1,
		ferrum_chem.SmilesBondStereoV1,
		ferrum_chem.SmilesBondDirectionV1,
		ferrum_chem.MolblockVersionV1,
		ferrum_chem.SmilesAtomV1,
		ferrum_chem.SmilesBondV1,
		ferrum_chem.SmilesMoleculeV1,
		ferrum_chem.SdfPropertyV1,
		ferrum_chem.SdfRecordV1,
		ferrum_chem.ImportedSdfRecordV1,
	)

	assert all(value.__module__ == "ferrum_chem" for value in public_types)
	assert (ferrum_chem.ChemistryError.__name__, ferrum_chem.ChemistryError.__module__) == (
		"ChemistryError",
		"ferrum_chem",
	)
	assert ferrum_chem.SmilesBondOrderV1.single == ferrum_chem.SmilesBondOrderV1.single


def test_smarts_export_uses_the_frozen_complete_molecule() -> None:
	molecule = ferrum_chem.parse_smiles("CCO")

	assert ferrum_chem.molecule_to_smarts(molecule) == "[#6]-[#6]-[#8]"
	with pytest.raises(TypeError):
		ferrum_chem.molecule_to_smarts(object())


def test_molblock_export_uses_explicit_syntax_and_frozen_coordinates() -> None:
	molecule = ferrum_chem.parse_smiles("CCO")
	v2000 = ferrum_chem.molecule_to_molblock(
		molecule, ferrum_chem.MolblockVersionV1.v2000,
	)
	v3000 = ferrum_chem.molecule_to_molblock(
		molecule, ferrum_chem.MolblockVersionV1.v3000,
	)

	assert "V2000" in v2000
	assert "M  V30 BEGIN CTAB" not in v2000
	assert "V3000" in v3000
	assert "M  V30 BEGIN CTAB" in v3000
	assert v2000.endswith("\n") and v3000.endswith("\n")
	with pytest.raises(TypeError):
		ferrum_chem.molecule_to_molblock(molecule, "v2000")


def test_molblock_import_owns_complete_v2000_and_v3000_molecules() -> None:
	molecule = ferrum_chem.parse_smiles("F/C=C/F")

	for version in (
		ferrum_chem.MolblockVersionV1.v2000,
		ferrum_chem.MolblockVersionV1.v3000,
	):
		molblock = ferrum_chem.molecule_to_molblock(molecule, version)
		imported = ferrum_chem.molblock_to_molecule(molblock)

		assert imported.canonical_smiles == "F/C=C/F"
		assert tuple(atom.atomic_number for atom in imported.atoms) == (9, 6, 6, 9)
		assert len(imported.bonds) == 3
		assert len(imported.coordinates) == 4
		assert all(
			math.isfinite(point.x) and math.isfinite(point.y)
			for point in imported.coordinates
		)

	for invalid in ("", "mol\0block", "not a molblock", "title\n\n\nV4000\nM  END\n"):
		with pytest.raises(ferrum_chem.InvalidMolblock):
			ferrum_chem.molblock_to_molecule(invalid)


def test_sdf_export_preserves_record_and_property_order_without_text_identity_gate() -> None:
	ethanol = ferrum_chem.parse_smiles("CCO")
	chloride = ferrum_chem.parse_smiles("[Cl-]")
	first = ferrum_chem.prepare_sdf_record(
		ethanol,
		"ethanol record",
		(("second", "line one\nline two"), ("first", "")),
	)
	second = ferrum_chem.prepare_sdf_record(chloride, "chloride record", ())
	sdf = ferrum_chem.records_to_sdf(
		(first, second), ferrum_chem.MolblockVersionV1.v3000,
	)

	assert sdf.count("$$$$\n") == 2
	assert sdf.count("M  V30 BEGIN CTAB") == 2
	assert sdf.startswith("ethanol record\n")
	assert sdf.find(">  <second>") < sdf.find(">  <first>")
	assert "line one\nline two\n\n" in sdf
	assert second.title == "chloride record"
	assert tuple((item.name, item.value) for item in first.properties) == (
		("second", "line one\nline two"),
		("first", ""),
	)
	with pytest.raises(ferrum_chem.ChemistryBoundary):
		ferrum_chem.prepare_sdf_record(
			ethanol, "record", (("same", "one"), ("same", "two")),
		)
	with pytest.raises(TypeError):
		ferrum_chem.records_to_sdf(
			[first], ferrum_chem.MolblockVersionV1.v3000,
		)


def test_sdf_import_copies_complete_records_and_preserves_duplicate_property_order() -> None:
	ethanol = ferrum_chem.parse_smiles("CCO")
	chloride = ferrum_chem.parse_smiles("[Cl-]")
	first = ferrum_chem.prepare_sdf_record(ethanol, "ethanol input", ())
	second = ferrum_chem.prepare_sdf_record(chloride, "chloride input", ())
	sdf = ferrum_chem.records_to_sdf(
		(first, second), ferrum_chem.MolblockVersionV1.v3000,
	)
	property_text = ">  <same>\none\n\n>  <same>\ntwo\n\n"
	sdf = sdf.replace("$$$$\n", property_text + "$$$$\n", 1)

	records = ferrum_chem.sdf_to_records(sdf)

	assert isinstance(records, tuple)
	assert tuple(record.title for record in records) == ("ethanol input", "chloride input")
	assert tuple(record.molecule.canonical_smiles for record in records) == ("CCO", "[Cl-]")
	assert tuple((item.name, item.value) for item in records[0].properties) == (
		("same", "one"),
		("same", "two"),
	)
	assert records[1].properties == ()
	with pytest.raises(AttributeError):
		records[0].title = "changed"
	for invalid in ("", "SDF\0"):
		with pytest.raises(ferrum_chem.InvalidSdf):
			ferrum_chem.sdf_to_records(invalid)


def test_smiles_molecule_preparation_is_frozen_and_one_atomic_document_edit() -> None:
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	molecule = ferrum_chem.prepare_smiles_molecule_v1("CCO", placement)
	session = ferrum_chem.DocumentSession.load("<cdml/>")
	prepared = session.prepare_insert_molecule_v1(0, molecule)
	committed = session.commit_create_molecule(0, prepared)
	projection = committed.observation.projection

	assert (molecule.atom_count, molecule.bond_count) == (3, 2)
	assert prepared.molecule_identifier.startswith("ferrum-molecule-v1-")
	assert committed.observation.snapshot.revision == 1
	assert tuple(atom.element for atom in projection.molecules[0].atoms) == ("C", "C", "O")
	assert len(projection.molecules[0].bonds) == 2
	with pytest.raises(AttributeError):
		molecule.atom_count = 9
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_create_molecule(1, prepared)
	assert session.undo(1).observation.projection.molecules == []
	assert len(session.redo(2).observation.projection.molecules) == 1


@pytest.mark.parametrize("smiles", (
	"F/C=C/F",
	"[C@H](F)(Cl)Br",
	"[CH3:1]O",
	"[CH2]",
	"[c:1]1ccccc1",
	"c1ccccc1/C=C/F",
	"c1ccccc1[CH2]",
))
def test_unproven_cdml_fact_mappings_are_rejected_instead_of_discarded(smiles: str) -> None:
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)

	with pytest.raises(
		ferrum_chem.UnsupportedMoleculeInsertionError,
		match="cannot encode yet",
	):
		ferrum_chem.prepare_smiles_molecule_v1(smiles, placement)


def test_periodic_display_catalog_is_closed_immutable_and_picker_scoped() -> None:
	"""The native picker catalog returns only typed Ferrum-owned display facts."""
	facts = ferrum_chem.periodic_display_facts_v1("Fe")
	entries = ferrum_chem.periodic_display_entries_v1()
	provenance = ferrum_chem.periodic_display_catalog_provenance_v1()

	assert (facts.symbol, facts.color, facts.category) == (
		"Fe", "#ffc0c0", ferrum_chem.ElementDisplayCategoryV1.transition_metal,
	)
	assert isinstance(entries, tuple) and len(entries) == 42
	assert tuple(entry.symbol for entry in entries) == tuple(dict.fromkeys(
		entry.symbol for entry in entries
	))
	assert (provenance.catalog_id, provenance.revision) == (
		"ferrum-periodic-display-v1", "2026-08-12",
	)
	assert "query pseudo-elements" in provenance.scope
	with pytest.raises(AttributeError):
		facts.color = "#000000"
	with pytest.raises(ferrum_chem.UnknownElementDisplaySymbolError) as caught:
		ferrum_chem.periodic_display_facts_v1("fe")

	assert caught.value.symbol == "fe"


def test_load_returns_an_immutable_authoritative_snapshot() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()

	assert "molecule" in snapshot.cdml
	assert snapshot.revision == 0
	assert len(snapshot.digest) == 64
	assert snapshot.is_dirty is False
	with pytest.raises(AttributeError):
		snapshot.cdml = "<cdml/>"


def test_malformed_cdml_maps_to_the_public_load_error() -> None:
	with pytest.raises(ferrum_chem.DocumentLoadError):
		ferrum_chem.DocumentSession.load("<cdml><molecule></cdml>")


def test_observation_and_stale_revision_conflict_are_typed() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observed = session.observe(0)

	assert observed.snapshot.revision == 0
	assert observed.projection.revision == observed.snapshot.revision
	assert observed.projection.digest == observed.snapshot.digest
	assert observed.projection.is_dirty == observed.snapshot.is_dirty
	assert observed.projection.molecules[0].atoms[0].position.x == 1.0
	with pytest.raises(AttributeError):
		observed.projection.molecules = []
	changed = session.submit(0, set_atom("N")).observation.snapshot
	assert changed.revision == 1

	with pytest.raises(ferrum_chem.RevisionConflictError) as caught:
		session.observe(0)

	assert caught.value.expected == 0
	assert caught.value.actual == 1


def test_render_observation_is_one_frozen_api_owned_plan_with_exact_glyphs() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation = session.observe_render(0)
	entry = observation.molecule_plans[0]
	plan = entry.plan
	batch = plan.batches[0]
	operation = batch.operations[-1]
	assert (observation.document.snapshot.revision, observation.document.snapshot.digest) == (
		plan.provenance.revision,
		plan.provenance.digest,
	)
	assert isinstance(observation.molecule_plans, tuple)
	assert (plan.schema, type(plan), type(batch), type(operation)) == ("ferrum-render-plan-v2",
		ferrum_chem.RenderPlanV2, ferrum_chem.RenderBatchV2, ferrum_chem.RenderOperationV2)
	assert (entry.molecule.source_id, entry.molecule.source_order) == ("m", 0)
	assert batch.target.record_id.kind == "Atom"
	assert operation.kind == "text"
	assert operation.operation.runs[0].glyphs[0].glyph_index > 0
	with pytest.raises(AttributeError):
		plan.provenance.revision = 1


def test_render_observation_preserves_typed_stale_and_closed_telex_contracts() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	resource = ferrum_chem.verified_telex_regular()
	assert isinstance(resource.data, bytes)
	assert (resource.resource_id, resource.byte_length, resource.family) == (
		"ferrum-telex-regular-v1",
		len(resource.data),
		"Telex",
	)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.observe_render(1)


def test_direct_text_projection_and_render_keep_closed_runs_and_exact_glyphs() -> None:
	session = ferrum_chem.DocumentSession.load(
		'<cdml><text id="label"><point x="10" y="20"/>'
		'<font size="18" color="#123456"/>'
		'<ftext>Line one\nH&lt;sub&gt;2&lt;/sub&gt;O</ftext></text></cdml>',
	)
	observation = session.observe_render(0)
	root = observation.document.projection.presentation_stack.roots[0]
	render = observation.text_renders[0]

	assert root.kind == "text"
	assert isinstance(root.text.runs, tuple)
	assert [(run.text, run.styles) for run in root.text.runs] == [
		("Line one\nH", ()),
		("2", ("subscript",)),
		("O", ()),
	]
	assert isinstance(render.source_runs, tuple)
	assert [run.script for run in render.source_runs] == [
		"baseline", "subscript", "baseline",
	]
	assert isinstance(render.operation.runs, tuple)
	assert all(glyph.glyph_index > 0 for run in render.operation.runs for glyph in run.glyphs)
	assert (render.target.source_id, render.anchor.x, render.anchor.y) == ("label", 10.0, 20.0)
	with pytest.raises(AttributeError):
		render.operation.paint = "000000"


def test_presentation_polyline_is_frozen_revision_bound_and_source_ordered() -> None:
	session = ferrum_chem.DocumentSession.load(
		"<cdml><polyline id=\"line\" spline=\"no\" line_color=\"#AbC\" width=\"2px\">"
		"<point x=\"1cm\" y=\"2\"/><point x=\"3\" y=\"4\"/></polyline></cdml>"
	)
	observation = session.observe(0)
	stack = observation.projection.presentation_stack
	root = stack.roots[0]

	assert (stack.revision, stack.digest) == (
		observation.snapshot.revision,
		observation.snapshot.digest,
	)
	assert (root.kind, root.polyline.path.points[0].x, root.polyline.path.points[1].y) == (
		"polyline",
		72.0 / 2.54,
		4.0,
	)
	assert (root.polyline.stroke.color, root.polyline.stroke.width) == ("#aabbcc", 2.0)
	assert (root.polyline.stroke.color_provenance, root.polyline.stroke.width_provenance) == (
		"root",
		"root",
	)
	assert (root.polyline.target.id is not None, root.polyline.target.source_id, root.polyline.target.source_order) == (
		True,
		"line",
		0,
	)
	with pytest.raises(AttributeError):
		root.polyline.stroke.width = 3.0


def test_presentation_polyline_idless_targets_and_invalid_geometry_remain_explicit() -> None:
	session = ferrum_chem.DocumentSession.load(
		"<cdml><polyline><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline>"
		"<polyline id=\"bad\"><point x=\"NaN\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline>"
		"<polyline><point x=\"2\" y=\"2\"/><point x=\"3\" y=\"3\"/></polyline></cdml>"
	)
	stack = session.observe(0).projection.presentation_stack

	assert stack.roots[0].polyline.target.id is None
	assert stack.roots[0].polyline.target.projection_key != stack.roots[1].polyline.target.projection_key
	assert [root.polyline.target.source_order for root in stack.roots] == [0, 2]
	assert (stack.issues[0].code, stack.issues[0].target.source_id) == (
		"invalid_polyline_geometry",
		"bad",
	)


def test_noop_and_mutation_follow_the_revision_and_dirty_contract() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	baseline = session.snapshot()
	no_change = session.submit(0, set_atom("C")).observation.snapshot

	assert no_change.revision == baseline.revision
	assert no_change.digest == baseline.digest
	assert no_change.is_dirty is False

	changed = session.submit(no_change.revision, set_atom("N")).observation.snapshot
	assert changed.revision == 1
	assert changed.is_dirty is True
	assert 'name="N"' in changed.cdml


def test_undo_and_redo_create_monotonic_revisions() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(0, set_atom("N")).observation.snapshot
	undone = session.undo(changed.revision).observation.snapshot
	redone = session.redo(undone.revision).observation.snapshot

	assert undone.revision == 2
	assert 'name="C"' in undone.cdml
	assert redone.revision == 3
	assert 'name="N"' in redone.cdml
	assert redone.is_dirty is True


def test_atom_position_operation_is_finite_revisioned_and_undoable() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	operation = ferrum_chem.DocumentOperationV1.set_atom_position("a", 7.5, 8.25, 0.0)
	moved = session.submit(0, operation).observation

	assert ferrum_chem.DocumentOperationV1.__module__ == "ferrum_chem"
	assert moved.snapshot.revision == 1
	assert (
		moved.projection.molecules[0].atoms[0].position.x,
		moved.projection.molecules[0].atoms[0].position.y,
	) == (7.5, 8.25)
	undone = session.undo(1).observation.projection.molecules[0].atoms[0].position
	assert (undone.x, undone.y) == (1.0, 2.0)
	with pytest.raises(ferrum_chem.ProjectionError):
		ferrum_chem.DocumentOperationV1.set_atom_position("a", float("nan"), 0.0, 0.0)


def test_atom_properties_are_one_frozen_atomic_edit_with_history(
	tmp_path: Path,
) -> None:
	change_type = ferrum_chem.DocumentAtomPropertyChangeV1
	changes = (
		change_type.element("O"),
		change_type.formal_charge(-1),
		change_type.valence(2),
		change_type.isotope(18),
		change_type.multiplicity(2),
		change_type.show(True),
		change_type.show_hydrogens(True),
		change_type.font_size(15.0),
		change_type.label_color("#A0B1c2"),
	)
	session = ferrum_chem.DocumentSession.load(ATOM_PROPERTIES_SOURCE)
	operation = ferrum_chem.DocumentOperationV1.set_atom_properties("a", changes)
	changed = session.submit(0, operation).observation
	atom = changed.projection.molecules[0].atoms[0]

	assert change_type.__module__ == "ferrum_chem"
	assert (
		changed.snapshot.revision, atom.element, atom.formal_charge, atom.valence,
		atom.isotope, atom.multiplicity, atom.show, atom.show_hydrogens,
	) == (1, "O", -1, 2, 18, 2, True, True)
	assert (atom.label_font.size, atom.label_font.color) == (15.0, "#a0b1c2")
	assert "vendor_keep=\"yes\"" in changed.snapshot.cdml and "<v:opaque" in changed.snapshot.cdml
	assert session.undo(1).observation.projection.molecules[0].atoms[0].element == "C"
	assert session.redo(2).observation.projection.molecules[0].atoms[0].element == "O"
	published = session.save_atomic(
		tmp_path / "atom-properties.cdml", session.snapshot().revision,
	)
	reopened = ferrum_chem.DocumentSession.load(published.published_snapshot.cdml)
	reopened_atom = reopened.observe(0).projection.molecules[0].atoms[0]
	assert (
		reopened_atom.element, reopened_atom.formal_charge, reopened_atom.valence,
		reopened_atom.isotope, reopened_atom.multiplicity, reopened_atom.show,
		reopened_atom.show_hydrogens,
	) == ("O", -1, 2, 18, 2, True, True)
	assert (reopened_atom.label_font.size, reopened_atom.label_font.color) == (15.0, "#a0b1c2")


def test_atom_properties_reject_ambiguous_or_invalid_python_intent() -> None:
	class TupleSubclass(tuple):
		pass

	change_type = ferrum_chem.DocumentAtomPropertyChangeV1
	charge = change_type.formal_charge(1)
	session = ferrum_chem.DocumentSession.load(ATOM_PROPERTIES_SOURCE)

	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.set_atom_properties("a", [charge])
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.set_atom_properties("a", (object(),))
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_atom_properties("a", (charge, charge))
	before = session.snapshot()
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_atom_properties("a", TupleSubclass((charge,)))
	after_subclass = session.snapshot()
	assert (after_subclass.revision, after_subclass.digest) == (before.revision, before.digest)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_atom_properties("a", (charge,) * 10)
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.isotope(0)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.element("C<")
	with pytest.raises(AttributeError):
		charge.value = 2

	no_change = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_atom_properties("a", ()),
	).observation.snapshot
	assert no_change.revision == 0 and no_change.is_dirty is False


def test_atom_number_is_frozen_revisioned_and_exactly_typed() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	assigned = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_atom_number("m", "a", 17, False),
	).observation
	atom = assigned.projection.molecules[0].atoms[0]
	assert (assigned.snapshot.revision, atom.number, atom.show_number) == (1, 17, False)
	assert 'number="17" show_number="no"' in assigned.snapshot.cdml
	assert session.undo(1).observation.projection.molecules[0].atoms[0].number is None
	assert session.redo(2).observation.projection.molecules[0].atoms[0].number == 17
	cleared = session.submit(
		3, ferrum_chem.DocumentOperationV1.clear_atom_number("m", "a"),
	).observation
	assert (
		cleared.snapshot.revision,
		cleared.projection.molecules[0].atoms[0].number,
		cleared.projection.molecules[0].atoms[0].show_number,
	) == (4, None, None)

	for number, show_number in [(True, True), (0, True), (-1, True), (1, 1)]:
		with pytest.raises(ferrum_chem.OperationValidationError):
			ferrum_chem.DocumentOperationV1.set_atom_number(
				"m", "a", number, show_number,
			)


def test_bond_properties_are_one_frozen_atomic_edit_with_history(
	tmp_path: Path,
) -> None:
	change_type = ferrum_chem.DocumentBondPropertyChangeV1
	changes = (
		change_type.order(ferrum_chem.DocumentBondOrderV1.double),
		change_type.style(ferrum_chem.DocumentBondStyleV1.dashed),
		change_type.center(True),
		change_type.line_width(2.5),
		change_type.bond_width(-4.0),
		change_type.wedge_width(5.0),
		change_type.color("#aBc"),
	)
	session = ferrum_chem.DocumentSession.load(BOND_PROPERTIES_SOURCE)
	changed = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", changes),
	).observation
	bond = changed.projection.molecules[0].bonds[0]

	assert change_type.__module__ == "ferrum_chem"
	assert ferrum_chem.DocumentBondStyleV1.__module__ == "ferrum_chem"
	assert changed.snapshot.revision == 1
	assert (
		bond.source_type, bond.order, bond.style, bond.center, bond.line_width,
		bond.bond_width, bond.wedge_width, bond.color,
	) == (
		"d2", ferrum_chem.DocumentBondOrderV1.double,
		ferrum_chem.DocumentBondStyleV1.dashed, True, 2.5, -4.0, 5.0, "#aabbcc",
	)
	assert "vendor_keep=\"yes\"" in changed.snapshot.cdml and "<v:keep" in changed.snapshot.cdml
	assert session.undo(1).observation.projection.molecules[0].bonds[0].source_type == "n1"
	assert session.redo(2).observation.projection.molecules[0].bonds[0].source_type == "d2"
	published = session.save_atomic(
		tmp_path / "bond-properties.cdml", session.snapshot().revision,
	)
	reopened = ferrum_chem.DocumentSession.load(published.published_snapshot.cdml)
	reopened_bond = reopened.observe(0).projection.molecules[0].bonds[0]
	assert (
		reopened_bond.source_type, reopened_bond.center, reopened_bond.line_width,
		reopened_bond.bond_width, reopened_bond.wedge_width, reopened_bond.color,
	) == ("d2", True, 2.5, -4.0, 5.0, "#aabbcc")


def test_haworth_position_projection_preserves_closed_depth_and_reports_malformed() -> None:
	projection = ferrum_chem.DocumentSession.load(HAWORTH_POSITION_SOURCE).observe(0).projection
	bonds = projection.molecules[0].bonds

	assert (bonds[0].haworth_position, bonds[1].haworth_position) == (
		ferrum_chem.DocumentHaworthPositionV1.front,
		ferrum_chem.DocumentHaworthPositionV1.back,
	)
	assert bonds[2].haworth_position is None
	assert [issue.code for issue in projection.issues].count("invalid_presentation_fact") == 1


def test_bond_properties_reject_hostile_python_intent_without_mutation() -> None:
	class TupleSubclass(tuple):
		pass

	change_type = ferrum_chem.DocumentBondPropertyChangeV1
	order = change_type.order(ferrum_chem.DocumentBondOrderV1.double)
	session = ferrum_chem.DocumentSession.load(BOND_PROPERTIES_SOURCE)
	before = session.snapshot()

	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.set_bond_properties("ab", [order])
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (object(),))
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_bond_properties(
			"ab", TupleSubclass((order,)),
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,) * 8)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order, order))
	for invalid in (0.0, float("nan"), float("inf")):
		with pytest.raises(ferrum_chem.OperationValidationError):
			change_type.bond_width(invalid)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_width(-1.0)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_width(True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.color("not-a-color")
	with pytest.raises(AttributeError):
		order.value = 2
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.submit(
			1, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,)),
		)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.submit(
			0, ferrum_chem.DocumentOperationV1.set_bond_properties("missing", (order,)),
		)
	unsupported = ferrum_chem.DocumentSession.load(
		BOND_PROPERTIES_SOURCE.replace('type="n1"', 'type="l1"'),
	)
	unsupported_before = unsupported.snapshot()
	with pytest.raises(ferrum_chem.OperationValidationError):
		unsupported.submit(
			0, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", (order,)),
		)
	unsupported_after = unsupported.snapshot()
	assert (unsupported_after.revision, unsupported_after.digest) == (
		unsupported_before.revision, unsupported_before.digest,
	)
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)

	no_change = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_bond_properties("ab", ()),
	).observation.snapshot
	assert no_change.revision == 0 and no_change.is_dirty is False


def test_atom_deletion_removes_incident_bonds_and_uses_one_history_entry() -> None:
	source = (
		'<cdml><molecule id="m">'
		'<atom id="a" name="C"><point x="1" y="2"/></atom>'
		'<atom id="b" name="O"><point x="3" y="2"/></atom>'
		'<bond id="ab" type="n1" start="a" end="b"/>'
		'</molecule></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	deleted = session.submit(
		0, ferrum_chem.DocumentOperationV1.delete_atom("b"),
	).observation

	assert tuple(atom.source_id for atom in deleted.projection.molecules[0].atoms) == ("a",)
	assert not deleted.projection.molecules[0].bonds
	restored = session.undo(1).observation.projection.molecules[0]
	assert len(restored.atoms) == 2 and len(restored.bonds) == 1
	redone = session.redo(2).observation.projection.molecules[0]
	assert len(redone.atoms) == 1 and not redone.bonds


def test_bond_deletion_preserves_atoms_and_uses_one_history_entry() -> None:
	source = (
		'<cdml><molecule id="m">'
		'<atom id="a" name="C"><point x="1" y="2"/></atom>'
		'<atom id="b" name="O"><point x="3" y="2"/></atom>'
		'<bond id="ab" type="n1" start="a" end="b"/>'
		'</molecule></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	deleted = session.submit(
		0, ferrum_chem.DocumentOperationV1.delete_bond("ab"),
	).observation

	assert len(deleted.projection.molecules[0].atoms) == 2
	assert not deleted.projection.molecules[0].bonds
	assert len(session.undo(1).observation.projection.molecules[0].bonds) == 1
	assert not session.redo(2).observation.projection.molecules[0].bonds
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.submit(3, ferrum_chem.DocumentOperationV1.delete_bond("missing"))
	assert caught.value.object_id == "missing"
	assert session.snapshot().revision == 3


def test_bond_order_change_is_typed_noop_aware_and_undoable() -> None:
	source = (
		'<cdml><molecule id="m">'
		'<atom id="a" name="C"><point x="1" y="2"/></atom>'
		'<atom id="b" name="O"><point x="20" y="2"/></atom>'
		'<bond id="ab" type="n1" start="a" end="b" retained="yes"/>'
		'</molecule></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	no_change = session.submit(
		0,
		ferrum_chem.DocumentOperationV1.set_bond_order(
			"ab", ferrum_chem.DocumentBondOrderV1.single,
		),
	).observation
	assert no_change.snapshot.revision == 0

	changed = session.submit(
		0,
		ferrum_chem.DocumentOperationV1.set_bond_order(
			"ab", ferrum_chem.DocumentBondOrderV1.double,
		),
	).observation
	assert changed.snapshot.revision == 1
	assert changed.projection.molecules[0].bonds[0].source_type == "n2"
	assert 'retained="yes"' in changed.snapshot.cdml
	assert session.undo(1).observation.projection.molecules[0].bonds[0].source_type == "n1"
	assert session.redo(2).observation.projection.molecules[0].bonds[0].source_type == "n2"
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.submit(
			3,
			ferrum_chem.DocumentOperationV1.set_bond_order(
				"missing", ferrum_chem.DocumentBondOrderV1.triple,
			),
		)
	assert caught.value.object_id == "missing"
	assert session.snapshot().revision == 3


def test_operation_validation_errors_are_specific_and_structured() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)

	with pytest.raises(ferrum_chem.InvalidAtomElementError):
		session.submit(0, set_atom("2"))
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.submit(
			0,
			ferrum_chem.DocumentOperationV1.set_atom_element("missing", "N"),
		)

	assert caught.value.object_id == "missing"
	assert session.snapshot().revision == 0


def test_prepared_atom_insertion_is_revision_bound_and_one_use() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	molecule_object_id = session.observe(0).projection.molecules[0].id
	prepared = session.prepare_create_atom_v1(
		0, molecule_object_id, "O", 3.0, 4.0, 0.0,
	)

	assert prepared.identifier.startswith("ferrum-atom-v1-")
	committed = session.commit_create_atom(0, prepared).observation.snapshot
	assert committed.revision == 1
	assert f'id="{prepared.identifier}"' in committed.cdml

	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_create_atom(1, prepared)


def test_prepared_bond_insertion_preserves_directed_presentation_and_is_one_use() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	projection = session.observe(0).projection
	start, end = (atom.id for atom in projection.molecules[0].atoms)
	prepared = session.prepare_create_bond_v2(
		0, start, end, ferrum_chem.DocumentBondPresentationV1.solid_wedge,
	)

	assert prepared.identifier == "ferrum-bond-v1-0"
	committed = session.commit_create_bond(0, prepared).observation
	assert committed.snapshot.revision == 1
	bond = committed.projection.molecules[0].bonds[0]
	assert bond.source_type == "w1"
	assert (bond.start.object_id, bond.end.object_id) == (start, end)
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_create_bond(1, prepared)


def test_bond_insertion_rejects_self_and_duplicate_edges_without_state_change() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	start, end = (atom.id for atom in session.observe(0).projection.molecules[0].atoms)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.prepare_create_bond_v2(
			0, start, start, ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
		)
	assert session.snapshot().revision == 0

	prepared = session.prepare_create_bond_v2(
		0, start, end, ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
	)
	session.commit_create_bond(0, prepared)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.prepare_create_bond_v2(
			1, end, start, ferrum_chem.DocumentBondPresentationV1.normal_double,
		)
	assert session.snapshot().revision == 1


def test_bonded_atom_insertion_is_one_frozen_rust_operation_and_one_undo() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	start = session.observe(0).projection.molecules[0].atoms[0].id
	prepared = session.prepare_create_bonded_atom_v2(
		0, start, "O", 8.0, 9.0, 0.0,
		ferrum_chem.DocumentBondPresentationV1.hashed_wedge,
	)

	assert prepared.__class__.__module__ == "ferrum_chem"
	assert prepared.atom_identifier == "ferrum-atom-v1-0"
	assert prepared.bond_identifier == "ferrum-bond-v1-0"
	assert session.snapshot().revision == 0
	committed = session.commit_create_bonded_atom(0, prepared).observation
	molecule = committed.projection.molecules[0]
	assert committed.snapshot.revision == 1
	assert len(molecule.atoms) == 2 and len(molecule.bonds) == 1
	assert molecule.atoms[1].source_id == prepared.atom_identifier
	assert (molecule.atoms[1].position.x, molecule.atoms[1].position.y) == (8.0, 9.0)
	assert molecule.bonds[0].source_type == "h1"
	assert molecule.bonds[0].end.source_id == prepared.atom_identifier
	undone = session.undo(1).observation.projection.molecules[0]
	assert len(undone.atoms) == 1 and not undone.bonds
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_create_bonded_atom(2, prepared)


def test_prepared_atom_requires_finite_explicit_coordinates() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	molecule_object_id = session.observe(0).projection.molecules[0].id

	with pytest.raises(ferrum_chem.ProjectionError) as caught:
		session.prepare_create_atom_v1(
			0, molecule_object_id, "O", float("nan"), 0.0, 0.0,
		)

	assert "not finite" in caught.value.reason
	assert session.snapshot().revision == 0


def test_molecule_coordinate_preparation_is_frozen_revision_bound_and_undoable() -> None:
	session = ferrum_chem.DocumentSession.load(COORDINATE_SOURCE)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].id
	prepared = ferrum_chem.prepare_molecule_coordinates_v1(observation, molecule_id)

	assert type(prepared) is ferrum_chem.PreparedMoleculeCoordinatesV1
	assert prepared.__class__.__module__ == "ferrum_chem"
	assert prepared.source_revision == observation.snapshot.revision
	assert prepared.source_digest == observation.snapshot.digest
	assert prepared.molecule_id == molecule_id and prepared.atom_count == 3
	with pytest.raises(AttributeError):
		prepared.atom_count = 4

	changed = session.apply_molecule_coordinates_v1(0, prepared).observation
	assert changed.snapshot.revision == 1 and changed.snapshot.is_dirty
	assert session.undo(1).observation.snapshot.cdml == observation.snapshot.cdml
	assert session.redo(2).observation.snapshot.digest == changed.snapshot.digest


def test_stale_molecule_coordinate_result_does_not_change_current_state() -> None:
	session = ferrum_chem.DocumentSession.load(COORDINATE_SOURCE)
	observation = session.observe(0)
	molecule_id = observation.projection.molecules[0].id
	prepared = ferrum_chem.prepare_molecule_coordinates_v1(observation, molecule_id)
	changed = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_atom_element("a", "N"),
	).observation.snapshot

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.apply_molecule_coordinates_v1(changed.revision, prepared)
	assert session.snapshot().digest == changed.digest


def test_foreign_session_rejection_keeps_prepared_atom_retryable_by_its_owner() -> None:
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	foreign = ferrum_chem.DocumentSession.load(SOURCE)
	molecule_object_id = owner.observe(0).projection.molecules[0].id
	prepared = owner.prepare_create_atom_v1(
		0, molecule_object_id, "O", 3.0, 4.0, 0.0,
	)

	with pytest.raises(ferrum_chem.PreparedOperationForeignSessionError):
		foreign.commit_create_atom(0, prepared)

	accepted = owner.commit_create_atom(0, prepared).observation.snapshot
	assert accepted.revision == 1
	assert f'id="{prepared.identifier}"' in accepted.cdml


def test_create_atom_requires_a_durable_typed_molecule_selector() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	atom_object_id = session.observe(0).projection.molecules[0].atoms[0].id

	with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
		session.prepare_create_atom_v1(0, "m", "O", 3.0, 4.0, 0.0)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.prepare_create_atom_v1(0, atom_object_id, "O", 3.0, 4.0, 0.0)


def test_confirmed_save_or_unconfirmed_outcome_preserves_exact_contract(
	tmp_path: Path,
) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(0, set_atom("N")).observation.snapshot
	published = session.save_atomic(tmp_path / "saved.cdml", changed.revision)

	revisions = (published.published_snapshot.revision, published.snapshot.revision)
	assert ((tmp_path / "saved.cdml").read_text(), revisions) == (
		published.published_snapshot.cdml, (changed.revision, changed.revision),
	)
	assert published.snapshot.is_dirty is (not published.outcome.is_confirmed)


def test_recovery_export_never_changes_the_session_state(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(0, set_atom("N")).observation.snapshot
	exported = session.recovery_export(tmp_path / "recovery.cdml", changed.revision)
	current = session.snapshot()

	assert (exported.snapshot.revision, exported.snapshot.is_dirty) == (changed.revision, True)
	assert (current.revision, current.digest, current.is_dirty) == (
		changed.revision, changed.digest, True,
	)


def test_invalid_destination_keeps_its_structured_public_fields(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)

	with pytest.raises(ferrum_chem.InvalidDestinationError) as caught:
		session.save_atomic(tmp_path, 0)

	assert caught.value.path == str(tmp_path)
	assert caught.value.reason == "destination exists but is not a regular file"


def test_publication_errors_share_the_documented_shape(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	target = tmp_path / "missing-parent" / "saved.cdml"

	with pytest.raises(ferrum_chem.PublicationNotStartedError) as caught:
		session.save_atomic(target, 0)

	assert (caught.value.path, bool(caught.value.reason)) == (str(target), True)


def test_render_interaction_binding_uses_render_plan_authority() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	initial = session.snapshot()
	observation = session.observe_render_interaction_v1(
		initial.revision, initial.digest,
	)
	assert [root.identifier for root in observation.roots] == ["m"]
	selection = session.select_render_interaction_roots_v1(
		observation, None,
		ferrum_chem.RenderInteractionQueryV1.point(
			1.0, 2.0, ferrum_chem.RenderInteractionModifierV1.replace,
		),
	)
	gesture = session.begin_render_interaction_translation_v1(
		selection, 1.0, 2.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	preview = session.preview_render_interaction_translation_v1(gesture, 6.0, 0.0)
	committed = session.commit_render_interaction_translation_v1(gesture, preview)
	assert committed.changed is True
	assert committed.result.observation.snapshot.revision == 1
	assert session.undo(1).observation.snapshot.revision == 2

	unsupported = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="blocked"><atom id="a" name="C">'
		'<point x="1" y="2"/><ftext><b>rich</b></ftext>'
		'</atom></molecule></cdml>'
	)
	blocked_snapshot = unsupported.snapshot()
	blocked = unsupported.observe_render_interaction_v1(
		blocked_snapshot.revision, blocked_snapshot.digest,
	)
	assert blocked.roots == ()
	assert [(entry.identifier, entry.reason) for entry in blocked.exclusions] == [
		("blocked", ferrum_chem.RenderInteractionExclusionReasonV1.unrenderable_depiction),
	]
	with pytest.raises(ferrum_chem.RenderInteractionError) as blocked_error:
		unsupported.select_render_interaction_roots_v1(
			blocked, None,
			ferrum_chem.RenderInteractionQueryV1.root("blocked"),
		)
	assert blocked_error.value.category == ferrum_chem.RenderInteractionCategoryV1.unrenderable_depiction
	display_only = ferrum_chem.DocumentSession.load(
		'<cdml><plus><point x="4" y="5"/></plus></cdml>',
	)
	display_snapshot = display_only.snapshot()
	display_observation = display_only.observe_render_interaction_v1(
		display_snapshot.revision, display_snapshot.digest,
	)
	assert display_observation.roots == ()
	assert display_observation.exclusions[0].reason == (
		ferrum_chem.RenderInteractionExclusionReasonV1.display_only
	)
	with pytest.raises(ferrum_chem.RenderInteractionError) as display_error:
		display_only.select_render_interaction_roots_v1(
			display_observation, None,
			ferrum_chem.RenderInteractionQueryV1.root(
				display_observation.exclusions[0].identifier,
			),
		)
	assert display_error.value.category == ferrum_chem.RenderInteractionCategoryV1.display_only
	fragment_reference = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="m"><atom id="a" name="C">'
		'<point x="0" y="0"/></atom><fragment><bond id="m"/>'
		'</fragment></molecule></cdml>',
	)
	fragment_snapshot = fragment_reference.snapshot()
	fragment_observation = fragment_reference.observe_render_interaction_v1(
		fragment_snapshot.revision, fragment_snapshot.digest,
	)
	assert [root.identifier for root in fragment_observation.roots] == ["m"]
	fragment_selection = fragment_reference.select_render_interaction_roots_v1(
		fragment_observation, None,
		ferrum_chem.RenderInteractionQueryV1.root("m"),
	)
	fragment_gesture = fragment_reference.begin_render_interaction_translation_v1(
		fragment_selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	fragment_preview = fragment_reference.preview_render_interaction_translation_v1(
		fragment_gesture, 3.0, 0.0,
	)
	assert fragment_reference.commit_render_interaction_translation_v1(
		fragment_gesture, fragment_preview,
	).result.observation.snapshot.revision == 1
	foreign = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	with pytest.raises(ferrum_chem.RenderInteractionError) as foreign_error:
		foreign.select_render_interaction_roots_v1(
			observation, None, ferrum_chem.RenderInteractionQueryV1.clear(),
		)
	assert foreign_error.value.category == ferrum_chem.RenderInteractionCategoryV1.foreign_session
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.select_render_interaction_roots_v1(
			observation, None, ferrum_chem.RenderInteractionQueryV1.clear(),
		)


def test_render_interaction_binding_moves_molecule_and_plus_atomically() -> None:
	session = ferrum_chem.DocumentSession.load(
		'<cdml><molecule id="m"><atom id="a" name="C">'
		'<point x="0" y="0"/></atom></molecule><plus id="p">'
		'<point x="40" y="0"/></plus></cdml>',
	)
	initial = session.snapshot()
	observation = session.observe_render_interaction_v1(
		initial.revision, initial.digest,
	)
	assert [root.identifier for root in observation.roots] == ["m", "p"]
	molecule = session.select_render_interaction_roots_v1(
		observation, None,
		ferrum_chem.RenderInteractionQueryV1.point(
			0.0, 0.0, ferrum_chem.RenderInteractionModifierV1.replace,
		),
	)
	mixed = session.select_render_interaction_roots_v1(
		observation, molecule,
		ferrum_chem.RenderInteractionQueryV1.point(
			40.0, 0.0, ferrum_chem.RenderInteractionModifierV1.toggle,
		),
	)
	assert [root.identifier for root in mixed.roots] == ["m", "p"]
	gesture = session.begin_render_interaction_translation_v1(
		mixed, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	preview = session.preview_render_interaction_translation_v1(gesture, 7.0, 4.0)
	committed = session.commit_render_interaction_translation_v1(gesture, preview)
	assert (committed.changed, committed.result.observation.snapshot.revision) == (True, 1)
	assert session.undo(1).observation.snapshot.revision == 2


def test_render_interaction_binding_captures_raw_or_view_hex_grid_snap() -> None:
	session = ferrum_chem.DocumentSession.load(BOND_SOURCE)
	initial = session.snapshot()
	observation = session.observe_render_interaction_v1(initial.revision, initial.digest)
	selection = session.select_render_interaction_roots_v1(
		observation, None, ferrum_chem.RenderInteractionQueryV1.root("m"),
	)
	raw = session.begin_render_interaction_translation_v1(
		selection, 0.0, 0.0, ferrum_chem.RenderInteractionSnapV1.free(),
	)
	grid = session.begin_render_interaction_translation_v1(
		selection, 0.0, 0.0,
		ferrum_chem.RenderInteractionSnapV1.with_grid_policy(
			ferrum_chem.RenderInteractionAxisV1.free,
			ferrum_chem.RenderInteractionGridSnapPolicyV1.view_hex_grid,
		),
	)
	raw_preview = session.preview_render_interaction_translation_v1(raw, 38.0, 18.0)
	grid_preview = session.preview_render_interaction_translation_v1(grid, 38.0, 18.0)
	assert (raw_preview.dx, raw_preview.dy) == (38.0, 18.0)
	assert (grid_preview.dx, grid_preview.dy) != (raw_preview.dx, raw_preview.dy)


def test_presentation_creation_gesture_binding_owns_preview_and_canonical_arrow() -> None:
	session = ferrum_chem.DocumentSession.load("<cdml/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_creation_gesture_v1(
		snapshot.revision, snapshot.digest,
		ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
		0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
		ferrum_chem.PresentationGestureSnapPolicyV1(
			angle_increment_degrees=45, fixed_length_pt=20,
		),
	)
	preview = session.preview_presentation_creation_gesture_v1(gesture, 8.0, 9.0)
	assert preview.overlay.color == "#000000"
	assert preview.overlay.width == 1.0
	assert len(preview.overlay.head_vertices) == 3
	assert preview.overlay.right > preview.overlay.end_x
	assert abs(preview.overlay.end_x - 14.142) < 0.01
	assert session.snapshot().revision == 0
	commit = session.commit_presentation_creation_gesture_v1(gesture, preview)
	assert commit.root.kind == ferrum_chem.PresentationGestureRootKindV1.arrow
	assert commit.root.identifier.startswith("ferrum-presentation-v1-")
	assert commit.result.observation.snapshot.revision == 1
	cdml = commit.result.observation.snapshot.cdml
	assert 'width="1.0"' in cdml
	assert 'color="#000000"' in cdml
	assert "cm" in cdml


def test_presentation_creation_gesture_binding_rejects_bad_handles_and_geometry() -> None:
	first = ferrum_chem.DocumentSession.load("<cdml/>")
	second = ferrum_chem.DocumentSession.load("<cdml/>")
	first_snapshot = first.snapshot()
	second_snapshot = second.snapshot()
	first_gesture = first.begin_presentation_creation_gesture_v1(
		first_snapshot.revision, first_snapshot.digest,
		ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
		0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
		ferrum_chem.PresentationGestureSnapPolicyV1(),
	)
	second_gesture = second.begin_presentation_creation_gesture_v1(
		second_snapshot.revision, second_snapshot.digest,
		ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
		0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
		ferrum_chem.PresentationGestureSnapPolicyV1(),
	)
	preview = first.preview_presentation_creation_gesture_v1(first_gesture, 10.0, 0.0)
	with pytest.raises(ferrum_chem.PresentationGestureError) as foreign:
		second.preview_presentation_creation_gesture_v1(first_gesture, 10.0, 0.0)
	assert foreign.value.category == ferrum_chem.PresentationGestureCategoryV1.foreign_session
	with pytest.raises(ferrum_chem.PresentationGestureError) as mixed:
		second.commit_presentation_creation_gesture_v1(second_gesture, preview)
	assert mixed.value.category == ferrum_chem.PresentationGestureCategoryV1.foreign_session
	assert second.snapshot().revision == 0
	with pytest.raises(ferrum_chem.PresentationGestureError) as short:
		first.preview_presentation_creation_gesture_v1(first_gesture, 1.0, 0.0)
	assert short.value.category == ferrum_chem.PresentationGestureCategoryV1.below_minimum_length
	with pytest.raises(ferrum_chem.PresentationGestureError) as long:
		first.preview_presentation_creation_gesture_v1(first_gesture, 20_001.0, 0.0)
	assert long.value.category == ferrum_chem.PresentationGestureCategoryV1.exceeds_geometry_limit
	assert first.snapshot().revision == 0


def test_dedicated_plus_placement_facade_commits_standard_plus() -> None:
	"""Only the renderer-backed Plus facade persists one canonical root."""
	session = ferrum_chem.DocumentSession.load("<cdml><standard font_size='18'/></cdml>")
	snapshot = session.snapshot()
	assert not hasattr(ferrum_chem.PresentationGestureKindV1, "plus")
	gesture = session.begin_plus_placement_gesture_v1(
		snapshot.revision, snapshot.digest, 72.0, 36.0,
	)
	preview = session.preview_plus_placement_gesture_v1(gesture)
	assert preview.overlay.text == "+"
	assert preview.overlay.font_size == 18.0
	assert session.snapshot().revision == 0
	commit = session.commit_plus_placement_gesture_v1(gesture, preview)
	assert commit.identifier.startswith("ferrum-presentation-v1-")
	assert '<plus' in commit.result.observation.snapshot.cdml
	plus = commit.result.observation.snapshot.cdml.split('<plus', 1)[1].split('</plus>', 1)[0]
	assert 'font_size' not in plus


def test_dedicated_plus_preview_equals_the_committed_renderer() -> None:
	"""The dedicated facade publishes only exact current renderer facts."""
	session = ferrum_chem.DocumentSession.load(
		"<cdml><standard font_size='18' line_color='#123456'/></cdml>",
	)
	snapshot = session.snapshot()
	gesture = session.begin_plus_placement_gesture_v1(
		snapshot.revision, snapshot.digest, 72.0, 36.0,
	)
	preview = session.preview_plus_placement_gesture_v1(gesture)
	assert preview.overlay.color == '123456'
	commit = session.commit_plus_placement_gesture_v1(gesture, preview)
	plus = commit.result.observation.projection.presentation_stack.roots[0].plus
	rendered = session.observe_render(1).plus_renders[0]
	assert (plus.font.size, plus.font.color) == (18.0, '#123456')
	assert preview.overlay.color == rendered.operation.paint
	assert (preview.overlay.left - 72.0, preview.overlay.top - 36.0) == pytest.approx((
		rendered.bounds.left, rendered.bounds.top,
	))
	assert (preview.overlay.right - 72.0, preview.overlay.bottom - 36.0) == pytest.approx((
		rendered.bounds.right, rendered.bounds.bottom,
	))


def test_dedicated_plus_facade_rejects_foreign_and_replayed_handles() -> None:
	"""The Plus facade keeps both opaque halves bound to one session snapshot."""
	first = ferrum_chem.DocumentSession.load("<cdml/>")
	second = ferrum_chem.DocumentSession.load("<cdml/>")
	first_snapshot = first.snapshot()
	gesture = first.begin_plus_placement_gesture_v1(
		first_snapshot.revision, first_snapshot.digest, 72.0, 36.0,
	)
	preview = first.preview_plus_placement_gesture_v1(gesture)
	with pytest.raises(ferrum_chem.PresentationGestureError) as foreign:
		second.commit_plus_placement_gesture_v1(gesture, preview)
	assert foreign.value.category == ferrum_chem.PresentationGestureCategoryV1.foreign_session
	first.commit_plus_placement_gesture_v1(gesture, preview)
	with pytest.raises(ferrum_chem.PresentationGestureError) as replay:
		first.commit_plus_placement_gesture_v1(gesture, preview)
	assert replay.value.category == ferrum_chem.PresentationGestureCategoryV1.stale_revision


def test_presentation_creation_gesture_binding_rejects_bool_and_replay_without_mutation() -> None:
	for kwargs in ({"angle_increment_degrees": True}, {"fixed_length_pt": True}):
		with pytest.raises(ferrum_chem.PresentationGestureError) as invalid:
			ferrum_chem.PresentationGestureSnapPolicyV1(**kwargs)
		assert invalid.value.category == ferrum_chem.PresentationGestureCategoryV1.invalid_snap_policy
	session = ferrum_chem.DocumentSession.load("<cdml/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_creation_gesture_v1(
		snapshot.revision, snapshot.digest,
		ferrum_chem.PresentationGestureKindV1.straight_normal_arrow,
		0.0, 0.0, ferrum_chem.ArrowGestureStyleV1(),
		ferrum_chem.PresentationGestureSnapPolicyV1(),
	)
	preview = session.preview_presentation_creation_gesture_v1(gesture, 10.0, 0.0)
	session.commit_presentation_creation_gesture_v1(gesture, preview)
	after = session.snapshot()
	with pytest.raises(ferrum_chem.PresentationGestureError) as replay:
		session.commit_presentation_creation_gesture_v1(gesture, preview)
	assert replay.value.category == ferrum_chem.PresentationGestureCategoryV1.stale_revision
	assert session.snapshot().revision == after.revision
	assert session.snapshot().cdml == after.cdml
