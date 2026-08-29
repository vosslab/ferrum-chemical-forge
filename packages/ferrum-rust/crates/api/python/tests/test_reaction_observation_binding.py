"""Installed-wheel contract for Rust-owned reaction observations and selections."""

from __future__ import annotations

import lxml.etree

import pytest

import ferrum_chem


CDML_NAMESPACE = "urn:ferrum:cdml"
STRICT_SOURCE = (
	'<c:cdml xmlns:c="' + CDML_NAMESPACE + '">'
	'<c:molecule id="left"><c:atom id="left-a" name="C">'
	'<c:point x="0" y="0"/></c:atom></c:molecule>'
	'<c:molecule id="right"><c:atom id="right-a" name="O">'
	'<c:point x="100" y="0"/></c:atom></c:molecule>'
	'<c:arrow id="arrow"><c:point x="25" y="0"/>'
	'<c:point x="75" y="0"/></c:arrow>'
	'<c:reaction id="strict"><c:reactant idref="left"/>'
	'<c:product idref="right"/><c:arrow idref="arrow"/></c:reaction></c:cdml>'
)
HOSTILE_SOURCE = (
	'<c:cdml xmlns:c="' + CDML_NAMESPACE + '" xmlns:v="urn:vendor">'
	'<c:molecule id="left"><c:atom id="left-a" name="C">'
	'<c:point x="0" y="0"/></c:atom></c:molecule>'
	'<c:molecule id="right"><c:atom id="right-a" name="O">'
	'<c:point x="100" y="0"/></c:atom></c:molecule>'
	'<c:arrow id="arrow"><c:point x="25" y="0"/>'
	'<c:point x="75" y="0"/></c:arrow>'
	'<c:reaction id="strict"><c:reactant idref="left"/>'
	'<c:product idref="right"/><c:arrow idref="arrow"/></c:reaction>'
	'<c:reaction id="display"><c:reactant id="display-reactant" v:idref="left"/>'
	'<v:product id="display-product" idref="right"/></c:reaction>'
	'<v:reaction id="foreign"><v:reactant idref="left"/></v:reaction>'
	'<c:molecule id="nested"><c:atom id="nested-a" name="N">'
	'<c:point x="200" y="0"/></c:atom><c:reaction id="nested-r">'
	'<c:reactant idref="left"/></c:reaction></c:molecule></c:cdml>'
)
_XML_PARSER = lxml.etree.XMLParser(
	load_dtd=False,
	resolve_entities=False,
	no_network=True,
	huge_tree=False,
)


def _list(session: object) -> object:
	"""Observe one exact Rust snapshot without reconstructing a fence in Python."""
	snapshot = session.snapshot()
	return session.observe_reaction_list_v1(snapshot.revision, snapshot.digest)


def test_reaction_observation_exposes_frozen_renderer_bounds_and_membership() -> None:
	"""A prefixed strict reaction retains renderer-issued aggregate geometry."""
	session = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	observation = _list(session)
	reaction = observation.reactions[0]
	selection = session.select_reaction_v1(observation, reaction.document_object_id)

	assert type(reaction) is ferrum_chem.ReactionObservationV1
	assert reaction.strict is True
	assert [(member.role, member.role_ordinal) for member in reaction.members] == [
		("reactant", 0), ("product", 0), ("arrow", 0),
	]
	assert [member.document_paint_order for member in reaction.members] == [0, 1, 2]
	assert all(member.document_object_id for member in reaction.members)
	session.validate_reaction_selection_v1(selection)


def test_hostile_reactions_remain_display_only_with_closed_diagnostics() -> None:
	"""Only direct CDML reaction roots become observations or opaque selections."""
	session = ferrum_chem.DocumentSession.load(HOSTILE_SOURCE)
	before = session.snapshot()
	observation = _list(session)
	strict, display = observation.reactions

	assert strict.strict is True
	assert display.strict is False
	assert {"missingidref", "unknownrolechild"} <= set(display.diagnostics)
	assert not display.members
	with pytest.raises(ferrum_chem.ReactionCommandError) as refused:
		session.select_reaction_v1(observation, display.document_object_id)
	assert refused.value.category is ferrum_chem.ReactionCommandRefusalCategoryV1.invalid_selection
	assert session.snapshot().digest == before.digest


def test_reaction_selection_refuses_foreign_and_stale_observations_without_mutation() -> None:
	"""Opaque selection is session and snapshot fenced even through PyO3."""
	owner = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	foreign = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	observation = _list(owner)
	selection = owner.select_reaction_v1(
		observation, observation.reactions[0].document_object_id,
	)
	foreign_before = foreign.snapshot()

	with pytest.raises(ferrum_chem.ReactionCommandError) as foreign_error:
		foreign.validate_reaction_selection_v1(selection)
	assert foreign_error.value.category is ferrum_chem.ReactionCommandRefusalCategoryV1.invalid_selection
	assert "different document session" in foreign_error.value.reason
	assert foreign.snapshot().digest == foreign_before.digest
	owner_before = owner.snapshot()
	owner.apply_document_operation_v1(
		owner_before.revision,
		ferrum_chem.DocumentOperationV1.set_atom_element("left-a", "N"),
	)
	stale_before = owner.snapshot()
	with pytest.raises(ferrum_chem.ReactionCommandError) as stale_error:
		owner.validate_reaction_selection_v1(selection)
	assert stale_error.value.category is ferrum_chem.ReactionCommandRefusalCategoryV1.invalid_selection
	assert "stale document revision" in stale_error.value.reason
	assert owner.snapshot().digest == stale_before.digest


def test_reaction_lifecycle_resolves_to_generic_transition_and_replays_no_commit() -> None:
	"""A selected strict reaction deletes through the sole generic receipt."""
	session = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	before = session.snapshot()
	observation = _list(session)
	selection = session.select_reaction_v1(
		observation, observation.reactions[0].document_object_id,
	)
	command = session.begin_delete_reaction_v1(selection)
	request = session.resolve_delete_reaction_command_v1(command)
	prepared = session.prepare_session_operation_transition_v1(request)
	commit = session.commit_session_operation_transition_v1(prepared)

	assert commit.outcome.kind == "reaction_definition_deleted_v1"
	assert commit.outcome.reaction_definition_deleted.reaction_document_object_id == (
		selection.reaction_document_object_id
	)
	assert commit.observation.snapshot.revision == before.revision + 1
	root = lxml.etree.fromstring(
		commit.observation.snapshot.cdml.encode("utf-8"), parser=_XML_PARSER,
	)
	assert not root.findall("{urn:ferrum:cdml}reaction")
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_session_operation_transition_v1(prepared)
