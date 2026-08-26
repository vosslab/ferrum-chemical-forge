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
	return session.observe(session.snapshot().revision).projection.molecules[0].atoms[0].document_object_id


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


def test_generic_methoxy_attachment_materializes_to_neutral_oxygen_first_topology() -> None:
	"""The Rust-issued Methoxy choice preserves its neutral oxygen-first chemistry."""
	session = _session()
	anchor = _anchor_id(session)
	methoxy = next(
		choice for choice in session._attached_compact_group_choices_v1()
		if choice.catalog_key == "methoxy")
	committed = session._commit_attach_compact_group_v1(_begin(session, methoxy.catalog_key))
	molecule = session.observe(committed.revision).projection.molecules[0]
	group = next(
		group for group in molecule.compact_groups
		if group.document_object_id == committed.compact_group_object_id)
	materialized = _materialize(session, molecule.document_object_id, group.document_object_id)
	materialized_molecule = materialized.mutation_result.observation.projection.molecules[0]
	atoms_by_id = {
		atom.document_object_id: atom
		for atom in materialized_molecule.atoms
	}
	anchor_bond = next(
		bond for bond in materialized_molecule.bonds
		if anchor in (bond.start.document_object_id, bond.end.document_object_id))
	oxygen_id = next(
		atom_id for atom_id in (
			anchor_bond.start.document_object_id,
			anchor_bond.end.document_object_id,
		)
		if atom_id != anchor)
	oxygen_bond = next(
		bond for bond in materialized_molecule.bonds
		if oxygen_id in (bond.start.document_object_id, bond.end.document_object_id)
		and anchor not in (bond.start.document_object_id, bond.end.document_object_id))
	carbon_id = next(
		atom_id for atom_id in (
			oxygen_bond.start.document_object_id,
			oxygen_bond.end.document_object_id,
		)
		if atom_id != oxygen_id)

	assert (
		atoms_by_id[oxygen_id].element,
		atoms_by_id[oxygen_id].formal_charge,
		anchor_bond.order,
		anchor_bond.style,
		atoms_by_id[carbon_id].element,
		atoms_by_id[carbon_id].formal_charge,
		oxygen_bond.order,
		oxygen_bond.style,
	) == (
		"O",
		None,
		ferrum_chem.DocumentBondOrderV1.single,
		ferrum_chem.DocumentBondStyleV1.normal,
		"C",
		None,
		ferrum_chem.DocumentBondOrderV1.single,
		ferrum_chem.DocumentBondStyleV1.normal,
	)


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


def test_attached_compact_group_refuses_unknown_key_without_mutation() -> None:
	"""An unknown catalog key is refused without changing the document."""
	session = _session()
	before = _snapshot_facts(session)

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as refused:
		_begin(session, "unknown")
	assert refused.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.invalid_catalog_key
	assert _snapshot_facts(session) == before


def test_generic_phenyl_attachment_materializes_with_typed_focus_receipt() -> None:
	"""Phenyl uses the same typed generic attach-and-materialize contract."""
	session = _session()
	committed = session._commit_attach_compact_group_v1(_begin(session, "phenyl"))
	molecule = session.observe(committed.revision).projection.molecules[0]
	group = next(
		group for group in molecule.compact_groups
		if group.document_object_id == committed.compact_group_object_id)
	materialized = _materialize(session, molecule.document_object_id, group.document_object_id)
	result = materialized.mutation_result

	assert isinstance(result, ferrum_chem.SessionOperationResultV1)
	outcome = result.outcome.compact_group_materialized
	assert result.outcome.kind == "compact_group_materialized_v1"
	assert outcome is not None
	post_molecule = result.observation.projection.molecules[0]
	assert outcome.replacement_focus_atom_identifier in {
		atom.document_object_id for atom in post_molecule.atoms
	}
	assert not post_molecule.compact_groups


def test_generic_methyl_attachment_previews_commits_and_consumes_once() -> None:
	"""A generic reviewed request keeps an opaque one-use lifecycle."""
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

	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as consumed:
		session._commit_attach_compact_group_v1(pending)
	assert consumed.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.consumed


def test_generic_ethyl_attachment_materializes_to_neutral_carbon_topology() -> None:
	"""One reviewed ethyl group materializes through the generic public lifecycle."""
	session = _session()
	anchor = _anchor_id(session)
	pending = _begin(session, "ethyl")

	assert session._preview_attach_compact_group_v1(pending).overlay is not None
	committed = session._commit_attach_compact_group_v1(pending)
	attached = session.observe(committed.revision)
	molecule = attached.projection.molecules[0]
	group = next(
		group for group in molecule.compact_groups
		if group.document_object_id == committed.compact_group_object_id)
	materialized = _materialize(session, molecule.document_object_id, group.document_object_id)
	result = materialized.mutation_result

	assert result is not None
	assert (materialized.source_revision, materialized.source_digest) == (
		committed.revision, bytes.fromhex(committed.digest))
	outcome = result.outcome.compact_group_materialized
	assert outcome is not None
	assert outcome.replacement_focus_atom_identifier
	post = result.observation
	assert session.observe(post.snapshot.revision).snapshot.digest == post.snapshot.digest
	materialized_molecule = post.projection.molecules[0]
	assert not materialized_molecule.compact_groups
	internal_bonds = [
		bond for bond in materialized_molecule.bonds
		if bond.start.document_object_id != anchor and bond.end.document_object_id != anchor]
	assert len(internal_bonds) == 1
	internal_bond = internal_bonds[0]
	assert (internal_bond.order, internal_bond.style) == (
		ferrum_chem.DocumentBondOrderV1.single,
		ferrum_chem.DocumentBondStyleV1.normal,
	)
	internal_atom_ids = {
		internal_bond.start.document_object_id,
		internal_bond.end.document_object_id,
	}
	materialized_atoms = [
		atom for atom in materialized_molecule.atoms
		if atom.document_object_id in internal_atom_ids]
	assert [(atom.element, atom.formal_charge) for atom in materialized_atoms] == [
		("C", None), ("C", None),
	]


def test_generic_nitro_attachment_preserves_projected_and_materialized_charge_chemistry() -> None:
	"""A reviewed nitro attachment reaches the ordinary materialization contract."""
	session = _session()
	committed = session._commit_attach_compact_group_v1(_begin(session, "nitro"))
	molecule = session.observe(session.snapshot().revision).projection.molecules[0]
	group = next(group for group in molecule.compact_groups if group.document_object_id == committed.compact_group_object_id)

	assert (group.catalog_key, group.label) == ("nitro", "NO2")
	materialized = _materialize(session, molecule.document_object_id, group.document_object_id)
	atoms = materialized.mutation_result.observation.projection.molecules[0].atoms
	charges = {atom.formal_charge for atom in atoms}
	assert 1 in charges
	assert -1 in charges


def test_generic_attachment_rejects_foreign_cancelled_and_stale_pending_handles() -> None:
	"""Session affinity, cancellation, and stale commits preserve durable state."""
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
	with pytest.raises(ferrum_chem.AttachedCompactGroupAttachmentError) as consumed:
		owner._preview_attach_compact_group_v1(pending)
	assert consumed.value.category == ferrum_chem.AttachedCompactGroupCategoryV1.consumed

	stale = _begin(owner, "methyl")
	owner._commit_attach_compact_group_v1(_begin(owner, "methyl", 30.0))
	stable = _snapshot_facts(owner)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		owner._commit_attach_compact_group_v1(stale)
	assert _snapshot_facts(owner) == stable
