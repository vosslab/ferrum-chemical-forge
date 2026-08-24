"""Installed-wheel contract for Rust-owned reaction observations and selections."""

from __future__ import annotations

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
	'<c:reaction id="display"><c:reactant v:idref="left"/>'
	'<v:product idref="right"/></c:reaction>'
	'<v:reaction id="foreign"><v:reactant idref="left"/></v:reaction>'
	'<c:molecule id="nested"><c:atom id="nested-a" name="N">'
	'<c:point x="200" y="0"/></c:atom><c:reaction id="nested-r">'
	'<c:reactant idref="left"/></c:reaction></c:molecule></c:cdml>'
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
	selection = session.select_reaction_v1(observation, "strict")

	assert type(reaction) is ferrum_chem.ReactionObservationV1
	assert reaction.disposition is ferrum_chem.ReactionDefinitionDispositionV1.strict
	assert [(member.identifier, member.role) for member in reaction.members] == [
		("left", "reactant"), ("right", "product"), ("arrow", "arrow"),
	]
	assert reaction.union_bounds.left <= min(member.bounds.left for member in reaction.members)
	assert reaction.union_bounds.right >= max(member.bounds.right for member in reaction.members)
	session.validate_reaction_selection_v1(selection)
	with pytest.raises(AttributeError):
		reaction.union_bounds.left = 7.0
	for forbidden in ("cdml", "dom", "candidate", "render_plan", "roots"):
		assert not hasattr(reaction, forbidden)


def test_hostile_reactions_remain_display_only_with_closed_diagnostics() -> None:
	"""Only direct CDML reaction roots become observations or opaque selections."""
	session = ferrum_chem.DocumentSession.load(HOSTILE_SOURCE)
	before = session.snapshot()
	observation = _list(session)
	by_id = {reaction.reaction_id: reaction for reaction in observation.reactions}
	display = by_id["display"]

	assert set(by_id) == {"strict", "display"}
	assert display.union_bounds is None
	assert {(item.reason, item.recovery, item.selector_source) for item in display.diagnostics} >= {
		(
			ferrum_chem.ReactionDiagnosticReasonV1.missing_idref,
			ferrum_chem.ReactionDiagnosticRecoveryV1.repair_document,
			ferrum_chem.ReactionDiagnosticSelectorSourceV1.direct_cdml_semantic_index,
		),
		(
			ferrum_chem.ReactionDiagnosticReasonV1.unknown_role_child,
			ferrum_chem.ReactionDiagnosticRecoveryV1.repair_document,
			ferrum_chem.ReactionDiagnosticSelectorSourceV1.direct_cdml_semantic_index,
		),
	}
	assert not display.members
	with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError):
		session.select_reaction_v1(observation, "display")
	assert session.snapshot().digest == before.digest


def test_reaction_selection_refuses_foreign_and_stale_observations_without_mutation() -> None:
	"""Opaque selection is session and snapshot fenced even through PyO3."""
	owner = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	foreign = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	observation = _list(owner)
	selection = owner.select_reaction_v1(observation, "strict")
	foreign_before = foreign.snapshot()

	with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError) as foreign_error:
		foreign.validate_reaction_selection_v1(selection)
	assert foreign_error.value.category is ferrum_chem.ReactionAuthoringChoicesRefusalCategoryV1.foreign_session
	assert foreign.snapshot().digest == foreign_before.digest
	owner_before = owner.snapshot()
	owner.apply_document_operation_v1(
		owner_before.revision,
		ferrum_chem.DocumentOperationV1.set_atom_element("left-a", "N"),
	)
	stale_before = owner.snapshot()
	with pytest.raises(ferrum_chem.ReactionAuthoringChoicesError) as stale_error:
		owner.validate_reaction_selection_v1(selection)
	assert stale_error.value.category == ferrum_chem.ReactionAuthoringChoicesRefusalCategoryV1.stale_snapshot
	assert owner.snapshot().digest == stale_before.digest


def test_reaction_lifecycle_resolves_to_generic_transition_and_replays_no_commit() -> None:
	"""A selected strict reaction deletes through the sole generic receipt."""
	session = ferrum_chem.DocumentSession.load(STRICT_SOURCE)
	before = session.snapshot()
	selection = session.select_reaction_v1(_list(session), "strict")
	gesture = session.begin_reaction_definition_delete_v1(selection)
	request = session.resolve_reaction_lifecycle_v1(gesture)
	prepared = session.prepare_session_operation_transition_v1(request)
	commit = session.commit_session_operation_transition_v1(prepared)

	assert commit.outcome.kind == "reaction_definition_deleted_v1"
	assert commit.outcome.reaction_definition_deleted.reaction_id == "strict"
	assert commit.observation.snapshot.revision == before.revision + 1
	assert '<c:reaction id="strict"' not in commit.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_session_operation_transition_v1(prepared)
