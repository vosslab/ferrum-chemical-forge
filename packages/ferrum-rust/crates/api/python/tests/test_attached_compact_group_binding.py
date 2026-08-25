"""Installed behavior for reviewed atom-anchored compact-group attachment."""

import json

import pytest

import ferrum_chem


def _snapshot_facts(session: object) -> tuple[str, int, str, bool]:
	"""Return durable facts that refused operations must preserve."""
	snapshot = session.snapshot()
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


def _session() -> object:
	"""Create one direct carbon through the supported molecule-insertion route."""
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	molecule = ferrum_chem.prepare_smiles_molecule_v1("C", placement)
	operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(molecule)
	pending = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(session.snapshot().revision))
	session.commit_session_operation_transition_v1(pending)
	return session


def _anchor_id(session: object) -> str:
	"""Return the direct atom's public Rust-issued durable object ID."""
	return session.observe(session.snapshot().revision).projection.molecules[0].atoms[0].id


def _begin(session: object, catalog_key: str, release_x: float = 20.0) -> object:
	"""Begin one generic attachment with the current document fence."""
	snapshot = session.snapshot()
	return session._begin_attach_compact_group_v1(
		snapshot.revision,
		snapshot.digest,
		_anchor_id(session),
		catalog_key,
		release_x,
		0.0,
	)


def _materialize(session: object, molecule_object_id: str, compact_group_object_id: str) -> object:
	"""Materialize a committed compact group through the supported live operation."""
	snapshot = session.snapshot()
	request = json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "attached-compact-group-materialization",
		"operation": {
			"kind": "document.compact-group.materialize.v1",
			"expected_revision": snapshot.revision,
			"expected_digest_hex": snapshot.digest,
			"molecule_object_id": molecule_object_id,
			"compact_group_object_id": compact_group_object_id,
		},
	})
	return session.apply_live_document_operation_v1(request)


def test_attached_compact_group_choices_are_rust_owned_public_facts() -> None:
	"""The private seam exposes reviewed catalog facts without a Python choice table."""
	session = _session()
	choices = {(choice.catalog_key, choice.label) for choice in session._attached_compact_group_choices_v1()}

	assert ("methyl", "Me") in choices
	assert ("nitro", "NO2") in choices


def test_attached_compact_group_availability_echoes_the_selected_choice() -> None:
	"""Read-only availability authenticates one reviewed choice and never mutates."""
	session = _session()
	before = _snapshot_facts(session)
	facts = session._attach_compact_group_availability_v1(
		before[1], before[2], _anchor_id(session), "methyl")

	assert facts.available is True
	assert facts.category == ferrum_chem.AttachedCompactGroupAvailabilityCategoryV1.available
	assert facts.catalog_key == "methyl"
	assert _snapshot_facts(session) == before


@pytest.mark.parametrize("catalog_key, category", [
	("unknown", ferrum_chem.AttachedCompactGroupCategoryV1.invalid_catalog_key),
	("ethyl", ferrum_chem.AttachedCompactGroupCategoryV1.unsupported_attachment_catalog_key),
])
def test_attached_compact_group_refuses_unapproved_keys_without_mutation(
		catalog_key: str, category: object) -> None:
	"""The boundary distinguishes unrecognized keys from known unreviewed choices."""
	session = _session()
	before = _snapshot_facts(session)

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as refused:
		_begin(session, catalog_key)
	assert refused.value.category == category
	assert _snapshot_facts(session) == before


def test_generic_methyl_attachment_previews_commits_and_retires_once() -> None:
	"""A generic reviewed request retains the opaque one-use lifecycle."""
	session = _session()
	before = session.snapshot()
	anchor = _anchor_id(session)
	pending = _begin(session, "methyl")

	assert session._preview_attach_compact_group_v1(pending).overlay is not None
	committed = session._commit_attach_compact_group_v1(pending)
	assert committed.revision == before.revision + 1
	assert committed.digest == session.snapshot().digest
	assert committed.is_dirty is True
	assert committed.focus_object_id == anchor
	assert committed.compact_group_object_id

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as replayed:
		session._commit_attach_compact_group_v1(pending)
	assert replayed.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.retired


def test_generic_nitro_attachment_preserves_projected_and_materialized_charge_chemistry() -> None:
	"""A reviewed nitro attachment reaches the ordinary materialization contract."""
	session = _session()
	committed = session._commit_attach_compact_group_v1(_begin(session, "nitro"))
	molecule = session.observe(session.snapshot().revision).projection.molecules[0]
	group = next(group for group in molecule.compact_groups if group.id == committed.compact_group_object_id)

	assert (group.catalog_key, group.label) == ("nitro", "NO2")
	materialized = _materialize(session, molecule.id, group.id)
	atoms = materialized.mutation_result.observation.projection.molecules[0].atoms
	charges = {atom.formal_charge for atom in atoms}
	assert 1 in charges
	assert -1 in charges


def test_generic_attachment_rejects_foreign_cancelled_and_stale_pending_handles() -> None:
	"""Session affinity, retirement, and stale commits preserve durable state."""
	owner = _session()
	foreign = _session()
	before_owner = _snapshot_facts(owner)
	before_foreign = _snapshot_facts(foreign)
	pending = _begin(owner, "methyl")

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as foreign_error:
		foreign._commit_attach_compact_group_v1(pending)
	assert foreign_error.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.foreign_session
	assert _snapshot_facts(owner) == before_owner
	assert _snapshot_facts(foreign) == before_foreign

	owner._cancel_attach_compact_group_v1(pending)
	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as retired:
		owner._preview_attach_compact_group_v1(pending)
	assert retired.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.retired

	stale = _begin(owner, "methyl")
	owner._commit_attach_compact_group_v1(_begin(owner, "methyl", 30.0))
	stable = _snapshot_facts(owner)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		owner._commit_attach_compact_group_v1(stale)
	assert _snapshot_facts(owner) == stable
