"""Installed private binding behavior for atom-anchored methyl compact groups."""

import pytest

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
	'<point x="0" y="0"/></atom></molecule></cdml>'
)

SATURATED_CARBON_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="a" name="C"><point x="0" y="0"/></atom>'
	'<atom id="h1" name="H"><point x="1" y="0"/></atom>'
	'<atom id="h2" name="H"><point x="-1" y="0"/></atom>'
	'<atom id="h3" name="H"><point x="0" y="1"/></atom>'
	'<atom id="h4" name="H"><point x="0" y="-1"/></atom>'
	'<bond id="b1" start="a" end="h1" type="n1"/>'
	'<bond id="b2" start="a" end="h2" type="n1"/>'
	'<bond id="b3" start="a" end="h3" type="n1"/>'
	'<bond id="b4" start="a" end="h4" type="n1"/>'
	'</molecule></cdml>'
)

ATTACHED_GROUP_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="a" name="C"><point x="0" y="0"/></atom>'
	'<compact-group id="g" version="1" catalog-key="methyl" attachment-index="0" '
	'orientation-degrees="0"><point x="20" y="0"/></compact-group>'
	'<bond id="b" start="a" end="g" type="n1"/></molecule></cdml>'
)

INVALID_TOPOLOGY_GROUP_SOURCE = ATTACHED_GROUP_SOURCE.replace(
	'<bond id="b" start="a" end="g" type="n1"/>', "")


def _snapshot_facts(session: object) -> tuple[str, int, str, bool]:
	"""Return the durable facts that refused private operations must preserve."""
	snapshot = session.snapshot()
	return (snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty)


def _anchor_id(session: object) -> str:
	"""Return the direct atom's Rust-issued durable object ID."""
	return session.observe(session.snapshot().revision).projection.molecules[0].atoms[0].id


def _commit_attachment(session: object, release_x: float = 20.0) -> object:
	"""Commit one private attachment and return its authoritative typed facts."""
	snapshot = session.snapshot()
	pending = session._begin_attach_methyl_compact_group_v1(
		snapshot.revision, snapshot.digest, _anchor_id(session), release_x, 0.0)
	return session._commit_attach_methyl_compact_group_v1(pending)


def _select_structure_target(session: object, x: float, y: float, previous: object | None = None,
		modifier: object | None = None) -> object:
	"""Select one renderer-issued structural target through the public opaque bridge."""
	snapshot = session.snapshot()
	observation = session.observe_structure_interaction_v1(snapshot.revision, snapshot.digest)
	if modifier is None:
		modifier = ferrum_chem.RenderInteractionModifierV1.replace
	query = ferrum_chem.StructureInteractionQueryV1.point(x, y, modifier)
	return session.select_structure_interaction_v1(observation, previous, query)


def test_private_methyl_availability_is_frozen_and_advisory() -> None:
	"""Availability returns current enablement facts without issuing a candidate."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = _snapshot_facts(session)
	anchor = _anchor_id(session)
	facts = session._attach_methyl_compact_group_availability_v1(
		before[1], before[2], anchor)

	assert facts.available is True
	assert facts.category == ferrum_chem.AttachedCompactGroupAvailabilityCategoryV1.available
	assert (facts.revision, facts.digest, facts.anchor_object_id) == (before[1], before[2], anchor)
	assert _snapshot_facts(session) == before


def test_private_methyl_availability_reports_capacity_without_mutation() -> None:
	"""A saturated direct carbon is a typed unavailable action target."""
	session = ferrum_chem.DocumentSession.load(SATURATED_CARBON_SOURCE)
	before = _snapshot_facts(session)
	facts = session._attach_methyl_compact_group_availability_v1(
		before[1], before[2], _anchor_id(session))

	assert facts.available is False
	assert facts.category == ferrum_chem.AttachedCompactGroupAvailabilityCategoryV1.candidate_admission
	assert _snapshot_facts(session) == before


def test_private_methyl_attachment_commits_authoritative_durable_facts() -> None:
	"""One pending methyl attachment previews and commits exactly once."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	anchor = _anchor_id(session)
	pending = session._begin_attach_methyl_compact_group_v1(
		before.revision, before.digest, anchor, 20.0, 0.0)

	assert session._preview_attach_methyl_compact_group_v1(pending).overlay is not None
	assert _snapshot_facts(session) == (before.cdml, before.revision, before.digest, before.is_dirty)

	committed = session._commit_attach_methyl_compact_group_v1(pending)
	assert (committed.revision, committed.digest, committed.is_dirty) == (
		before.revision + 1, session.snapshot().digest, True)
	assert committed.focus_object_id == anchor
	assert committed.compact_group_object_id
	assert committed.compact_group_object_id != committed.focus_object_id

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as replayed:
		session._commit_attach_methyl_compact_group_v1(pending)
	assert replayed.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.retired


@pytest.mark.parametrize("begin", [False, True])
def test_private_methyl_attachment_rejects_malformed_digests_with_a_typed_category(
	begin: bool,
) -> None:
	"""Availability and begin expose one stable malformed-digest refusal category."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as refused:
		if begin:
			session._begin_attach_methyl_compact_group_v1(
				snapshot.revision, "malformed", _anchor_id(session), 20.0, 0.0)
		else:
			session._attach_methyl_compact_group_availability_v1(
				snapshot.revision, "malformed", _anchor_id(session))
	assert refused.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.invalid_digest


def test_private_methyl_attachment_rejects_foreign_and_stale_pending_handles() -> None:
	"""Session affinity and stale fences refuse before either session mutates."""
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	foreign = ferrum_chem.DocumentSession.load(SOURCE)
	before_owner = _snapshot_facts(owner)
	before_foreign = _snapshot_facts(foreign)
	owner_snapshot = owner.snapshot()
	pending = owner._begin_attach_methyl_compact_group_v1(
		owner_snapshot.revision, owner_snapshot.digest, _anchor_id(owner), 20.0, 0.0)

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as foreign_error:
		foreign._commit_attach_methyl_compact_group_v1(pending)
	assert foreign_error.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.foreign_session
	assert _snapshot_facts(owner) == before_owner
	assert _snapshot_facts(foreign) == before_foreign

	owner._cancel_attach_methyl_compact_group_v1(pending)
	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as retired:
		owner._preview_attach_methyl_compact_group_v1(pending)
	assert retired.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.retired

	stale = owner._begin_attach_methyl_compact_group_v1(
		owner_snapshot.revision, owner_snapshot.digest, _anchor_id(owner), 20.0, 0.0)
	accepted = owner._begin_attach_methyl_compact_group_v1(
		owner_snapshot.revision, owner_snapshot.digest, _anchor_id(owner), 30.0, 0.0)
	owner._commit_attach_methyl_compact_group_v1(accepted)
	stable = _snapshot_facts(owner)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		owner._commit_attach_methyl_compact_group_v1(stale)
	assert _snapshot_facts(owner) == stable


def test_structure_deletion_returns_the_compact_group_receipt() -> None:
	"""One selected compact group uses the ordinary deletion receipt shape."""
	session = ferrum_chem.DocumentSession.load(ATTACHED_GROUP_SOURCE)
	commit = session.commit_structure_deletion_v1(_select_structure_target(session, 20.0, 0.0))

	assert (
		commit.removed_atom_count,
		commit.removed_bond_count,
		commit.removed_compact_group_count,
	) == (0, 1, 1)
	assert "compact-group" not in session.snapshot().cdml


def test_structure_deletion_refuses_mixed_compact_group_selection_before_prepare() -> None:
	"""A compact group cannot be combined with an atom or bond deletion target."""
	session = ferrum_chem.DocumentSession.load(ATTACHED_GROUP_SOURCE)
	before = _snapshot_facts(session)
	group = _select_structure_target(session, 20.0, 0.0)
	mixed = _select_structure_target(
		session, 0.0, 0.0, group, ferrum_chem.RenderInteractionModifierV1.toggle,
	)

	with pytest.raises(ferrum_chem.RenderInteractionError) as refused:
		session.commit_structure_deletion_v1(mixed)
	assert refused.value.category == (
		ferrum_chem.RenderInteractionCategoryV1.invalid_compact_group_deletion_selection
	)
	assert _snapshot_facts(session) == before


def test_structure_deletion_refuses_invalid_compact_topology_with_document_repair() -> None:
	"""Topology-only compact refusal stays redacted and gives the repair recovery."""
	session = ferrum_chem.DocumentSession.load(INVALID_TOPOLOGY_GROUP_SOURCE)
	before = _snapshot_facts(session)

	with pytest.raises(ferrum_chem.RenderInteractionError) as refused:
		session.commit_structure_deletion_v1(_select_structure_target(session, 20.0, 0.0))
	assert refused.value.category == (
		ferrum_chem.RenderInteractionCategoryV1.invalid_compact_group_deletion_topology
	)
	assert refused.value.recovery == ferrum_chem.RenderInteractionRecoveryV1.repair_document
	assert str(refused.value) == (
		"the compact group deletion topology requires document repair before retry"
	)
	assert _snapshot_facts(session) == before
