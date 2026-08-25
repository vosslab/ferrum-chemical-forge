"""Installed-wheel contract for Ferrum-Chem's revisioned document boundary."""

from __future__ import annotations

import math
import json
from pathlib import Path
import sys
import types

import pytest

import ferrum_chem


SOURCE = (
    "<cdml xmlns='urn:ferrum:cdml'><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
    "<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)

COORDINATE_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="m">'
    '<atom id="a" name="C"><point x="10" y="20"/></atom>'
    '<atom id="b" name="C"><point x="50" y="20"/></atom>'
    '<atom id="c" name="O"><point x="50" y="60"/></atom>'
    '<bond id="ab" start="a" end="b" type="n1"/>'
    '<bond id="bc" start="b" end="c" type="n1"/>'
    '</molecule></cdml>'
)

ATOM_PROPERTIES_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><molecule id="m">'
    '<atom id="a" name="C" charge="2" valency="4" isotope="13" '
    'multiplicity="3" show="no" hydrogens="off" vendor_keep="yes">'
    '<point x="1" y="2"/><font family="Courier" size="11" '
    'vendor_keep="yes"/><ftext>keep</ftext><v:keep/></atom>'
    '</molecule><v:opaque id="retained"/></cdml>'
)

BOND_PROPERTIES_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><molecule id="m">'
    '<atom id="a" name="C"><point x="1" y="2"/></atom>'
    '<atom id="b" name="O"><point x="20" y="2"/></atom>'
    '<bond id="ab" start="a" end="b" type="n1" line_width="1.5" '
    'bond_width="-2" wedge_width="3" color="#A0B1C2" vendor_keep="yes">'
    '<v:keep/></bond></molecule><v:opaque id="retained"/></cdml>'
)

HAWORTH_POSITION_SOURCE = (
    '<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C"><point x="0" y="0"/></atom>'
    '<atom id="b" name="O"><point x="1" y="0"/></atom>'
    '<bond start="a" end="b" haworth_position="front"/>'
    '<bond start="b" end="a" haworth_position="back"/>'
    '<bond start="a" end="b" haworth_position="side"/></molecule></cdml>'
)


def set_atom(element: str) -> ferrum_chem.DocumentOperationV1:
    return ferrum_chem.DocumentOperationV1.set_atom_element("a", element)


def test_reaction_creation_resolves_to_the_generic_transition_route() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="left"><atom id="left-a" name="C">'
        '<point x="0" y="0"/></atom></molecule><molecule id="product">'
        '<atom id="product-a" name="O"><point x="100" y="0"/></atom></molecule>'
        '<arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/>'
        '</arrow></cdml>'
    )
    session = ferrum_chem.DocumentSession.load(source)
    snapshot = session.snapshot()
    gesture = session.begin_reaction_gesture_v1(
        snapshot.revision, snapshot.digest, ["left"], ["product"], "arrow", [], [],
    )
    request = session.resolve_reaction_gesture_v1(gesture)
    prepared = session.prepare_session_operation_transition_v1(request)
    commit = session.commit_session_operation_transition_v1(prepared)
    assert commit.outcome.kind == "reaction_created_v1"
    assert commit.outcome.reaction_created.reaction_id == "rxn-1"
    assert commit.observation.snapshot.revision == 1
    assert '<reaction id="rxn-1"' in commit.observation.snapshot.cdml


def test_reaction_authoring_choices_are_renderer_fenced_and_non_mutating() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="left"><atom id="left-a" name="C">'
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
    with pytest.raises(ferrum_chem.OperationValidationError):
        session.apply_document_operation_v1(
            0, ferrum_chem.DocumentOperationV1.set_atom_element("left-a", "N"),
        )
    rejected = session.snapshot()
    assert (rejected.revision, rejected.digest) == (snapshot.revision, snapshot.digest)

    renderable = ferrum_chem.DocumentSession.load(SOURCE)
    renderable_snapshot = renderable.snapshot()
    renderable_choices = renderable.observe_reaction_authoring_choices_v1(
        renderable_snapshot.revision, renderable_snapshot.digest,
    )
    commit = renderable.apply_document_operation_v1(
        0, ferrum_chem.DocumentOperationV1.set_atom_element("a", "N"),
    )
    assert commit.observation.snapshot.revision == 1
    with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError) as captured:
        renderable.validate_reaction_authoring_choices_v1(renderable_choices)
    assert captured.value.category == ferrum_chem.ReactionAuthoringChoicesRefusalCategoryV1.stale_snapshot
    other = ferrum_chem.DocumentSession.load(SOURCE)
    with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError) as captured:
        other.validate_reaction_authoring_choices_v1(renderable_choices)
    assert captured.value.category == ferrum_chem.ReactionAuthoringChoicesRefusalCategoryV1.foreign_session


def test_text_placement_binding_uses_renderer_overlay_and_one_commit() -> None:
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'><standard font_size='18' line_color='#123456'/></cdml>")
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
        '<cdml xmlns="urn:ferrum:cdml"><standard font_family="No Such Face"/></cdml>',
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


def test_structure_path_target_is_display_only_and_cannot_create_a_delete_handle() -> None:
    session = ferrum_chem.DocumentSession.load(
        "<cdml xmlns='urn:ferrum:cdml'><molecule id=\"m\">"
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
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(molecule)
    prepared = session.prepare_session_operation_transition_v1(
        operation.transition_request_v1(0))
    committed = session.commit_session_operation_transition_v1(prepared)
    projection = committed.observation.projection

    assert (molecule.atom_count, molecule.bond_count) == (3, 2)
    assert committed.outcome.molecule_inserted.molecule_identifier.startswith(
        "ferrum-molecule-v1-")
    assert committed.observation.snapshot.revision == 1
    assert tuple(atom.element for atom in projection.molecules[0].atoms) == ("C", "C", "O")
    assert len(projection.molecules[0].bonds) == 2
    with pytest.raises(AttributeError):
        molecule.atom_count = 9
    with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
        session.commit_session_operation_transition_v1(prepared)
    assert session.undo(1).observation.projection.molecules == []
    assert len(session.redo(2).observation.projection.molecules) == 1


@pytest.mark.parametrize("smiles", (
    "[CH3:1]O",
    "[CH2]",
))
def test_unproven_cdml_fact_mappings_are_rejected_instead_of_discarded(smiles: str) -> None:
    placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)

    with pytest.raises(ferrum_chem.UnsupportedMoleculeInsertionError):
        ferrum_chem.prepare_smiles_molecule_v1(smiles, placement)


@pytest.mark.parametrize(("smiles", "expected_semantics"), (
    (
        "[C@H](F)(Cl)Br",
        {
            "tetrahedral": [{
                "center": 0,
                "ligands": [
                    {"kind": "atom", "index": 1},
                    {"kind": "atom", "index": 2},
                    {"kind": "atom", "index": 3},
                    {"kind": "explicit_hydrogen"},
                ],
                "parity": "clockwise",
            }],
            "double_bonds": [],
        },
    ),
))
def test_native_smiles_stereo_reaches_the_durable_molecule_report(
        smiles: str, expected_semantics: dict[str, object]) -> None:
    """Native P0 stereo facts cross the source coordinator into the report."""
    placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
    molecule = ferrum_chem.prepare_smiles_molecule_v1(smiles, placement)
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(molecule)
    prepared = session.prepare_session_operation_transition_v1(
        operation.transition_request_v1(0))
    committed = session.commit_session_operation_transition_v1(prepared)
    snapshot = committed.observation.snapshot
    molecule_id = committed.observation.projection.molecules[0].id
    response = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
        "schema": "ferrum-operation-request-v1",
        "request_id": "native-smiles-stereo",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {
                "cdml": snapshot.cdml,
                "revision": snapshot.revision,
                "digest_hex": snapshot.digest,
            },
            "molecule_ids": [molecule_id],
        },
    })))

    record = response["outcome"]["report"]["records"][0]
    assert record["stereo_semantics"] == expected_semantics


def test_native_inchi_stereo_reaches_the_durable_molecule_report() -> None:
    """Public InChI preparation retains durable E/Z semantics through commit."""
    placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
    molecule = ferrum_chem.prepare_inchi_molecule_v1(
        "InChI=1S/C4H8/c1-3-4-2/h3-4H,1-2H3/b4-3+", placement)
    session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
    operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(molecule)
    prepared = session.prepare_session_operation_transition_v1(
        operation.transition_request_v1(0))
    committed = session.commit_session_operation_transition_v1(prepared)
    snapshot = committed.observation.snapshot
    molecule_id = committed.observation.projection.molecules[0].id
    response = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
        "schema": "ferrum-operation-request-v1",
        "request_id": "native-inchi-stereo",
        "operation": {
            "kind": "document.molecule.report.v1",
            "snapshot": {
                "cdml": snapshot.cdml,
                "revision": snapshot.revision,
                "digest_hex": snapshot.digest,
            },
            "molecule_ids": [molecule_id],
        },
    })))

    semantics = response["outcome"]["report"]["records"][0]["stereo_semantics"]
    assert semantics["tetrahedral"] == []
    assert len(semantics["double_bonds"]) == 1
    assert semantics["double_bonds"][0]["configuration"] == "e"


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
        snapshot.cdml = "<cdml xmlns='urn:ferrum:cdml'/>"


def test_malformed_cdml_maps_to_the_public_load_error() -> None:
    with pytest.raises(ferrum_chem.DocumentLoadError):
        ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'><molecule></cdml>")


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
    changed = session.apply_document_operation_v1(0, set_atom("N")).observation.snapshot
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
    assert batch.target.kind == "Atom"
    assert operation.kind == "text"
    assert operation.operation.runs[0].glyphs[0].glyph_index > 0
    with pytest.raises(AttributeError):
        plan.provenance.revision = 1


def test_render_targets_publish_visual_and_durable_document_identities() -> None:
    source = (
        '<cdml xmlns="urn:ferrum:cdml"><molecule id="molecule">'
        '<atom id="atom" name="C"><point x="0" y="0"/></atom>'
        '<compact-group id="group" version="1" catalog-key="methyl" attachment-index="0" '
        'orientation-degrees="0"><point x="20" y="0"/></compact-group>'
        '<bond id="bond" start="atom" end="group" type="n1"/>'
        '</molecule></cdml>'
    )
    entry = ferrum_chem.DocumentSession.load(source).observe_render(0).molecule_plans[0]
    targets = {batch.target.kind: batch.target for batch in entry.plan.batches}

    for kind in ("Atom", "Bond", "Group"):
        target = targets[kind]
        assert target.render_identifier is not None
        assert target.durable_object_id is not None
        assert target.durable_molecule_object_id == entry.molecule.id


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
        '<cdml xmlns="urn:ferrum:cdml"><text id="label"><point x="10" y="20"/>'
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
    assert (render.target.render_identifier, render.anchor.x, render.anchor.y) == (
        "label",
        10.0,
        20.0,
    )
    with pytest.raises(AttributeError):
        render.operation.paint = "000000"


def test_presentation_polyline_is_frozen_revision_bound_and_source_ordered() -> None:
    session = ferrum_chem.DocumentSession.load(
        "<cdml xmlns='urn:ferrum:cdml'><polyline id=\"line\" spline=\"no\" line_color=\"#AbC\" width=\"2px\">"
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
        "<cdml xmlns='urn:ferrum:cdml'><polyline><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline>"
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
    no_change = session.apply_document_operation_v1(0, set_atom("C")).observation.snapshot

    assert no_change.revision == baseline.revision
    assert no_change.digest == baseline.digest
    assert no_change.is_dirty is False

    changed = session.apply_document_operation_v1(no_change.revision, set_atom("N")).observation.snapshot
    assert changed.revision == 1
    assert changed.is_dirty is True
    assert 'name="N"' in changed.cdml


def test_undo_and_redo_create_monotonic_revisions() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    changed = session.apply_document_operation_v1(0, set_atom("N")).observation.snapshot
    undone = session.undo(changed.revision).observation.snapshot
    redone = session.redo(undone.revision).observation.snapshot

    assert undone.revision == 2
    assert 'name="C"' in undone.cdml
    assert redone.revision == 3
    assert 'name="N"' in redone.cdml
    assert redone.is_dirty is True


def test_history_availability_getters_follow_cursor_and_discarded_branch() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    assert session.can_undo is False and session.can_redo is False

    changed = session.apply_document_operation_v1(0, set_atom("N")).observation.snapshot
    assert session.can_undo is True and session.can_redo is False
    undone = session.undo(changed.revision).observation.snapshot
    assert session.can_undo is False and session.can_redo is True
    session.apply_document_operation_v1(undone.revision, set_atom("O"))
    assert session.can_undo is True and session.can_redo is False


def test_atom_position_operation_is_finite_revisioned_and_undoable() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    operation = ferrum_chem.DocumentOperationV1.set_atom_position("a", 7.5, 8.25, 0.0)
    moved = session.apply_document_operation_v1(0, operation).observation

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
    loaded_atom = session.observe(0).projection.molecules[0].atoms[0]
    assert (loaded_atom.label_font.family, loaded_atom.label_font.size) == ("Courier", 11.0)
    operation = ferrum_chem.DocumentOperationV1.set_atom_properties("a", changes)
    changed = session.apply_document_operation_v1(0, operation).observation
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

    no_change = session.apply_document_operation_v1(
        0, ferrum_chem.DocumentOperationV1.set_atom_properties("a", ()),
    ).observation.snapshot
    assert no_change.revision == 0 and no_change.is_dirty is False


def test_atom_number_is_frozen_revisioned_and_exactly_typed() -> None:
    session = ferrum_chem.DocumentSession.load(SOURCE)
    assigned = session.apply_document_operation_v1(
        0, ferrum_chem.DocumentOperationV1.set_atom_number("m", "a", 17, False),
    ).observation
    atom = assigned.projection.molecules[0].atoms[0]
    assert (assigned.snapshot.revision, atom.number, atom.show_number) == (1, 17, False)
    assert 'number="17" show_number="no"' in assigned.snapshot.cdml
    assert session.undo(1).observation.projection.molecules[0].atoms[0].number is None
    assert session.redo(2).observation.projection.molecules[0].atoms[0].number == 17
    cleared = session.apply_document_operation_v1(
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
    changed = session.apply_document_operation_v1(
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
