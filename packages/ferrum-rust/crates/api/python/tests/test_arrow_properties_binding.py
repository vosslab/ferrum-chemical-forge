"""Installed-extension behavior for the closed direct-root Arrow edit boundary."""

# PIP3 modules
import ferrum_chem
import pytest


def _arrow(observation: object) -> object:
	"""Return the one projected normal Arrow from an exact installed extension."""
	root, = observation.projection.presentation_stack.roots
	assert root.kind == "arrow"
	return root.arrow


def test_arrow_properties_are_atomic_frozen_and_revision_bound() -> None:
	"""Apply one closed patch and preserve semantic state through history."""
	source = (
		'<cdml xmlns:v="urn:vendor"><arrow id="a" type="normal" start="no" '
		'end="yes" spline="no" width="1" color="#000" keep="yes">'
		'<point x="0" y="0"/><v:opaque/><point x="40" y="0"/>'
		'</arrow></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentArrowPropertyChangeV1
	changes = (
		change_type.start_head(True),
		change_type.end_head(False),
		change_type.spline(False),
		change_type.line_width(2.5),
		change_type.color("#AbC"),
	)
	operation = ferrum_chem.DocumentOperationV1.set_arrow_properties("a", changes)
	changed = session.submit(0, operation).observation
	arrow = _arrow(changed)
	assert changed.snapshot.revision == 1
	assert (arrow.start_head, arrow.end_head) == (True, False)
	assert (arrow.stroke.width, arrow.stroke.color) == (2.5, "#aabbcc")
	assert 'keep="yes"' in changed.snapshot.cdml
	assert "opaque" in changed.snapshot.cdml
	with pytest.raises(AttributeError):
		changes[0].value = False
	assert _arrow(session.undo(1).observation).start_head is False
	assert _arrow(session.redo(2).observation).start_head is True


def test_arrow_properties_reject_hostile_shapes_without_mutation() -> None:
	"""Reject malformed intent, tuple subclasses, excess work, and stale edits."""
	source = (
		'<cdml><arrow id="a" type="normal"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentArrowPropertyChangeV1
	before = session.observe(0).snapshot
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.start_head(1)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_width(True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_width(0.09)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_arrow_properties(
			"a", tuple(change_type.start_head(True) for _ in range(6)),
		)

	class TupleSubclass(tuple):
		"""Hostile tuple subclass rejected before item extraction."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_arrow_properties(
			"a", TupleSubclass((change_type.start_head(True),)),
		)
	assert session.observe(0).snapshot.digest == before.digest
	unknown = ferrum_chem.DocumentOperationV1.set_arrow_properties(
		"missing", (change_type.start_head(True),),
	)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as error:
		session.submit(0, unknown)
	assert error.value.object_id == "missing"
	assert session.observe(0).snapshot.digest == before.digest
	operation = ferrum_chem.DocumentOperationV1.set_arrow_properties(
		"a", (change_type.start_head(True),),
	)
	session.submit(0, operation)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.submit(0, operation)
	assert session.observe(1).projection.presentation_stack.roots[0].arrow.start_head
