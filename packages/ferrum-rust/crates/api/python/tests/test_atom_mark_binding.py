"""Installed-extension behavior for the closed Rust atom-mark boundary."""

import ferrum_chem
import pytest


SOURCE = (
	"<cdml xmlns='urn:ferrum:cdml'><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
	"<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)

LIVE_TARGET_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="first">'
	'<atom id="first-c" name="C"><point x="1" y="2"/></atom>'
	'<atom id="first-o" name="O"><point x="3" y="2"/></atom>'
	'<bond id="first-bond" start="first-c" end="first-o" type="n1"/>'
	'</molecule><molecule id="second">'
	'<atom id="second-n" name="N"><point x="5" y="2"/></atom>'
	'</molecule></cdml>'
)

LIVE_DELETE_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="c" name="C"><point x="1" y="2"/></atom>'
	'<atom id="o" name="O"><point x="3" y="2"/></atom>'
	'<bond id="bond" start="c" end="o" type="n1"/>'
	'</molecule></cdml>'
)


def test_atom_mark_changes_the_public_atom_semantics_and_undo_restores_it() -> None:
	"""One accepted mark changes formal charge and undo restores the original atom."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	added = session.apply_document_operation_v1(
		0,
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", ferrum_chem.AtomMarkActionV1.add,
			ferrum_chem.AtomMarkKindV1.plus, None,
		),
	).observation
	atom = added.projection.molecules[0].atoms[0]

	assert atom.formal_charge == 1
	mark = atom.marks[0]
	assert mark.kind == ferrum_chem.AtomMarkKindV1.plus
	assert mark.radial_offset > 0.0

	removed = session.apply_document_operation_v1(
		1,
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", ferrum_chem.AtomMarkActionV1.remove,
			ferrum_chem.AtomMarkKindV1.plus, 0,
		),
	).observation
	assert removed.projection.molecules[0].atoms[0].formal_charge is None
	assert session.undo(2).observation.projection.molecules[0].atoms[0].formal_charge == 1


def test_atom_mark_intent_rejects_hostile_selectors_before_mutation() -> None:
	"""Malformed Python selectors never reach revisioned document mutation."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()

	for invalid in (True, -1, 2**64):
		with pytest.raises(ferrum_chem.OperationValidationError):
			ferrum_chem.DocumentOperationV1.apply_atom_mark(
				"m", "a", ferrum_chem.AtomMarkActionV1.remove,
				ferrum_chem.AtomMarkKindV1.radical, invalid,
			)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", ferrum_chem.AtomMarkActionV1.add,
			ferrum_chem.AtomMarkKindV1.radical, 0,
		)
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.apply_atom_mark(
			"m", "a", "add", ferrum_chem.AtomMarkKindV1.radical, None,
		)
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)


def test_live_atom_mark_uses_the_rust_issued_durable_owner_address() -> None:
	"""A live mutation rejects source selectors and accepts the current durable pair."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	molecule = session.observe(snapshot.revision).projection.molecules[0]
	atom = molecule.atoms[0]

	with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
		session.apply_atom_mark_v1(
			snapshot.revision,
			snapshot.digest,
			"m",
			"a",
			ferrum_chem.AtomMarkActionV1.add,
			ferrum_chem.AtomMarkKindV1.plus,
			None,
		)
	assert session.snapshot().revision == snapshot.revision

	result = session.apply_atom_mark_v1(
		snapshot.revision,
		snapshot.digest,
		molecule.document_object_id,
		atom.document_object_id,
		ferrum_chem.AtomMarkActionV1.add,
		ferrum_chem.AtomMarkKindV1.plus,
		None,
	)
	assert result.observation.projection.molecules[0].atoms[0].formal_charge == 1


def test_live_atom_property_and_number_use_the_rust_issued_durable_address() -> None:
	"""Live atom edits accept the projection address and retain their public facts."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	molecule = session.observe(snapshot.revision).projection.molecules[0]
	atom = molecule.atoms[0]

	changed = session.set_atom_properties_v1(
		snapshot.revision,
		snapshot.digest,
		molecule.document_object_id,
		atom.document_object_id,
		(ferrum_chem.DocumentAtomPropertyChangeV1.element("N"),),
	)
	changed_atom = changed.observation.projection.molecules[0].atoms[0]
	assert changed_atom.element == "N"

	numbered = session.set_atom_number_v1(
		changed.observation.snapshot.revision,
		changed.observation.snapshot.digest,
		molecule.document_object_id,
		atom.document_object_id,
		7,
		True,
	)
	numbered_atom = numbered.observation.projection.molecules[0].atoms[0]
	assert (numbered_atom.number, numbered_atom.show_number) == (7, True)


def test_live_atom_element_position_and_deletion_lower_only_valid_durable_targets() -> None:
	"""Live element, position, and deletion each lower the current durable owner pair."""
	session = ferrum_chem.DocumentSession.load(LIVE_DELETE_SOURCE)
	snapshot = session.snapshot()
	molecule = session.observe(snapshot.revision).projection.molecules[0]
	atom = molecule.atoms[0]

	with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
		session.set_atom_element_v1(
			snapshot.revision, snapshot.digest, "m", "a", "N",
		)

	changed = session.set_atom_element_v1(
		snapshot.revision, snapshot.digest, molecule.document_object_id, atom.document_object_id, "N",
	)
	moved = session.set_atom_position_v1(
		changed.observation.snapshot.revision,
		changed.observation.snapshot.digest,
		molecule.document_object_id,
		atom.document_object_id,
		3.0,
		4.0,
		0.0,
	)
	moved_atom = moved.observation.projection.molecules[0].atoms[0]
	assert moved_atom.element == "N"
	assert (moved_atom.position.x, moved_atom.position.y) == (3.0, 4.0)

	deleted = session.delete_atom_v1(
		moved.observation.snapshot.revision,
		moved.observation.snapshot.digest,
		molecule.document_object_id,
		atom.document_object_id,
	)
	assert deleted.observation.projection.molecules[0].atoms[0].element == "O"


def test_live_atom_adapters_reject_stale_revision_and_digest_without_mutation() -> None:
	"""Each adapter exposes stale fences as the typed revision-conflict contract."""
	session = ferrum_chem.DocumentSession.load(LIVE_TARGET_SOURCE)
	before = session.snapshot()
	molecule = session.observe(before.revision).projection.molecules[0]
	atom = molecule.atoms[0]

	with pytest.raises(ferrum_chem.RevisionConflictError) as revision_error:
		session.set_atom_element_v1(
			before.revision + 1, before.digest, molecule.document_object_id, atom.document_object_id, "N",
		)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.set_atom_position_v1(
			before.revision + 1, before.digest, molecule.document_object_id, atom.document_object_id, 3.0, 4.0, 0.0,
		)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.delete_atom_v1(
			before.revision + 1, before.digest, molecule.document_object_id, atom.document_object_id,
		)
	assert (revision_error.value.expected, revision_error.value.actual) == (
		before.revision + 1, before.revision,
	)

	with pytest.raises(ferrum_chem.RevisionConflictError) as digest_error:
		session.set_atom_element_v1(
			before.revision, "0" * 64, molecule.document_object_id, atom.document_object_id, "N",
		)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.set_atom_position_v1(
			before.revision, "0" * 64, molecule.document_object_id, atom.document_object_id, 3.0, 4.0, 0.0,
		)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.delete_atom_v1(
			before.revision, "0" * 64, molecule.document_object_id, atom.document_object_id,
		)
	assert digest_error.value.reason == "expected digest does not match the live document"
	assert (session.snapshot().revision, session.snapshot().digest, session.snapshot().cdml) == (
		before.revision, before.digest, before.cdml,
	)


def test_live_atom_adapters_refuse_invalid_durable_targets_without_mutation() -> None:
	"""Only a current durable molecule-owned atom can reach every live adapter."""
	session = ferrum_chem.DocumentSession.load(LIVE_TARGET_SOURCE)
	before = session.snapshot()
	first, second = session.observe(before.revision).projection.molecules
	foreign_atom = second.atoms[0]
	bond = first.bonds[0]

	with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
		session.set_atom_element_v1(before.revision, before.digest, "first", "first-c", "N")
	with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
		session.set_atom_position_v1(
			before.revision, before.digest, "first", "first-c", 3.0, 4.0, 0.0,
		)
	with pytest.raises(ferrum_chem.InvalidDocumentObjectIdError):
		session.delete_atom_v1(before.revision, before.digest, "first", "first-c")

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.set_atom_element_v1(before.revision, before.digest, first.document_object_id, foreign_atom.document_object_id, "N")
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.set_atom_position_v1(
			before.revision, before.digest, first.document_object_id, foreign_atom.document_object_id, 3.0, 4.0, 0.0,
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.delete_atom_v1(before.revision, before.digest, first.document_object_id, foreign_atom.document_object_id)

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.set_atom_element_v1(before.revision, before.digest, first.document_object_id, bond.document_object_id, "N")
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.set_atom_position_v1(
			before.revision, before.digest, first.document_object_id, bond.document_object_id, 3.0, 4.0, 0.0,
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.delete_atom_v1(before.revision, before.digest, first.document_object_id, bond.document_object_id)
	assert (session.snapshot().revision, session.snapshot().digest, session.snapshot().cdml) == (
		before.revision, before.digest, before.cdml,
	)


def test_live_atom_adapters_refuse_missing_and_invalid_operation_input() -> None:
	"""Known-invalid targets and operation input preserve the current document exactly."""
	session = ferrum_chem.DocumentSession.load(LIVE_TARGET_SOURCE)
	before = session.snapshot()
	first = session.observe(before.revision).projection.molecules[0]
	atom = first.atoms[0]

	with pytest.raises(ferrum_chem.InvalidAtomElementError):
		session.set_atom_element_v1(before.revision, before.digest, first.document_object_id, atom.document_object_id, "Xx")
	with pytest.raises(ferrum_chem.ProjectionError):
		session.set_atom_position_v1(
			before.revision, before.digest, first.document_object_id, atom.document_object_id, float("nan"), 4.0, 0.0,
		)
	assert (session.snapshot().revision, session.snapshot().digest, session.snapshot().cdml) == (
		before.revision, before.digest, before.cdml,
	)

	deleted = session.delete_atom_v1(
		before.revision, before.digest, first.document_object_id, atom.document_object_id,
	)
	current = deleted.observation.snapshot
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.set_atom_element_v1(current.revision, current.digest, first.document_object_id, atom.document_object_id, "N")
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.set_atom_position_v1(
			current.revision, current.digest, first.document_object_id, atom.document_object_id, 3.0, 4.0, 0.0,
		)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.delete_atom_v1(current.revision, current.digest, first.document_object_id, atom.document_object_id)
	assert (session.snapshot().revision, session.snapshot().digest, session.snapshot().cdml) == (
		current.revision, current.digest, current.cdml,
	)
