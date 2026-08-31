"""Installed-extension behavior for the closed direct-root Plus edit boundary."""

# PIP3 modules
import ferrum_chem
import pytest


def _plus(observation: object) -> object:
	"""Return the one projected Plus from an exact installed extension."""
	root, = observation.projection.presentation_stack.entries
	assert root.kind == "plus"
	return root.plus


def _plus_id(session: object) -> str:
	"""Return the current durable Plus target."""
	return _plus(session.observe(session.snapshot().revision)).target.document_object_id


def test_plus_properties_are_atomic_frozen_and_revision_bound() -> None:
	"""Apply one closed patch and preserve semantic state through history."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><plus id="p" font_size="14" color="#000" '
		'keep="yes"><point x="10" y="20"/><v:opaque/></plus></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentPlusPropertyChangeV1
	changes = (
		change_type.font_face_id("molecule_label"),
		change_type.font_size(18),
		change_type.color("#AbC"),
		change_type.background_color(None),
	)
	operation = ferrum_chem.DocumentOperationV1.set_plus_properties(_plus_id(session), changes)
	changed = session.apply_document_operation_v1(0, operation).observation
	plus = _plus(changed)
	assert changed.snapshot.revision == 1
	assert (plus.font.font_face_id, plus.font.size, plus.font.color) == (
		"molecule_label", 18.0, "#aabbcc",
	)
	assert plus.background.color is None
	assert 'keep="yes"' in changed.snapshot.cdml
	assert "opaque" in changed.snapshot.cdml
	with pytest.raises(AttributeError):
		changes[0].value = "other"
	assert _plus(session.undo(1).observation).font.font_face_id == "molecule_label"
	assert _plus(session.redo(2).observation).font.font_face_id == "molecule_label"


def test_plus_properties_reject_hostile_shapes_without_mutation() -> None:
	"""Reject malformed intent, tuple subclasses, excess work, and stale edits."""
	source = '<cdml xmlns="urn:ferrum:cdml"><plus id="p"><point x="1" y="2"/></plus></cdml>'
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentPlusPropertyChangeV1
	plus_id = _plus_id(session)
	before = session.observe(0).snapshot
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.font_size(True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.font_size(3)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_plus_properties(
			plus_id, tuple(change_type.font_size(18) for _ in range(5)),
		)

	class TupleSubclass(tuple):
		"""Hostile tuple subclass rejected before item extraction."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_plus_properties(
			plus_id, TupleSubclass((change_type.font_size(18),)),
		)
	assert session.observe(0).snapshot.digest == before.digest
	operation = ferrum_chem.DocumentOperationV1.set_plus_properties(
		plus_id, (change_type.font_size(18),),
	)
	session.apply_document_operation_v1(0, operation)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_document_operation_v1(0, operation)
	assert session.observe(1).projection.presentation_stack.entries[0].plus.font.size == 18.0
