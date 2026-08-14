"""Installed-extension behavior for the closed direct-root Plus edit boundary."""

# PIP3 modules
import ferrum_chem
import pytest


def _plus(observation: object) -> object:
	"""Return the one projected Plus from an exact installed extension."""
	root, = observation.projection.presentation_stack.roots
	assert root.kind == "plus"
	return root.plus


def test_plus_properties_are_atomic_frozen_and_revision_bound() -> None:
	"""Apply one closed patch and preserve semantic state through history."""
	source = (
		'<cdml xmlns:v="urn:vendor"><plus id="p" font_size="14" color="#000" '
		'keep="yes"><point x="10" y="20"/><v:opaque/></plus></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentPlusPropertyChangeV1
	changes = (
		change_type.font_family(" Telex "),
		change_type.font_size(18),
		change_type.color("#AbC"),
		change_type.background_color(None),
	)
	operation = ferrum_chem.DocumentOperationV1.set_plus_properties("p", changes)
	changed = session.submit(0, operation).observation
	plus = _plus(changed)
	assert changed.snapshot.revision == 1
	assert (plus.font.family, plus.font.size, plus.font.color) == (
		"Telex", 18.0, "#aabbcc",
	)
	assert plus.background.color is None
	assert 'keep="yes"' in changed.snapshot.cdml
	assert "opaque" in changed.snapshot.cdml
	with pytest.raises(AttributeError):
		changes[0].value = "other"
	assert _plus(session.undo(1).observation).font.family is None
	assert _plus(session.redo(2).observation).font.family == "Telex"


def test_plus_properties_reject_hostile_shapes_without_mutation() -> None:
	"""Reject malformed intent, tuple subclasses, excess work, and stale edits."""
	source = '<cdml><plus id="p"><point x="1" y="2"/></plus></cdml>'
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentPlusPropertyChangeV1
	before = session.observe(0).snapshot
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.font_size(True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.font_size(3)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_plus_properties(
			"p", tuple(change_type.font_size(18) for _ in range(5)),
		)

	class TupleSubclass(tuple):
		"""Hostile tuple subclass rejected before item extraction."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_plus_properties(
			"p", TupleSubclass((change_type.font_size(18),)),
		)
	assert session.observe(0).snapshot.digest == before.digest
	operation = ferrum_chem.DocumentOperationV1.set_plus_properties(
		"p", (change_type.font_size(18),),
	)
	session.submit(0, operation)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.submit(0, operation)
	assert session.observe(1).projection.presentation_stack.roots[0].plus.font.size == 18.0
