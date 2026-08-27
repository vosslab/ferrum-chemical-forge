"""Installed-extension behavior for the closed direct-root Arrow edit boundary."""

# PIP3 modules
import ferrum_chem
import pytest


def _arrow(observation: object) -> object:
	"""Return the one projected normal Arrow from an exact installed extension."""
	root, = observation.projection.presentation_stack.entries
	assert root.kind == "arrow"
	return root.arrow


def test_arrow_properties_edit_updates_semantics_and_history() -> None:
	"""Apply one Arrow edit through the binding and preserve it through history."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal" start="no" '
		'end="yes" spline="no" width="1" color="#000">'
		'<point x="0" y="0"/><point x="40" y="0"/>'
		'</arrow></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	arrow_object_id = _arrow(session.observe(0)).target.document_object_id
	change_type = ferrum_chem.DocumentArrowPropertyChangeV1
	changes = (
		change_type.start_head(True),
		change_type.end_head(False),
		change_type.spline(False),
		change_type.line_width(2.5),
		change_type.color("#AbC"),
	)
	operation = ferrum_chem.DocumentOperationV1.set_arrow_properties(arrow_object_id, changes)
	changed = session.apply_document_operation_v1(0, operation).observation
	arrow = _arrow(changed)
	assert changed.snapshot.revision == 1
	assert arrow.target.document_object_id == changed.projection.direct_roots[0].document_object_id
	assert (arrow.kind.kind, arrow.kind.start_head, arrow.kind.end_head) == ("normal", True, False)
	assert (arrow.stroke.width, arrow.stroke.color) == (2.5, "#aabbcc")
	undone = _arrow(session.undo(1).observation)
	assert (undone.kind.start_head, undone.kind.end_head) == (False, True)
	assert (undone.stroke.width, undone.stroke.color) == (1.0, "#000000")
	redone = _arrow(session.redo(2).observation)
	assert (redone.kind.start_head, redone.kind.end_head) == (True, False)
	assert (redone.stroke.width, redone.stroke.color) == (2.5, "#aabbcc")


def test_arrow_properties_reject_hostile_shapes_without_mutation() -> None:
	"""Reject malformed intent, tuple subclasses, excess work, and stale edits."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml"><arrow id="a" type="normal"><point x="0" y="0"/>'
		'<point x="40" y="0"/></arrow></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	arrow_object_id = _arrow(session.observe(0)).target.document_object_id
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
			arrow_object_id, tuple(change_type.start_head(True) for _ in range(6)),
		)

	class TupleSubclass(tuple):
		"""Hostile tuple subclass rejected before item extraction."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_arrow_properties(
			arrow_object_id, TupleSubclass((change_type.start_head(True),)),
		)
	assert session.observe(0).snapshot.digest == before.digest
	unknown = ferrum_chem.DocumentOperationV1.set_arrow_properties(
		"ferrum-document-object-v1/00000000000000000000000000000001",
		(change_type.start_head(True),),
	)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.apply_document_operation_v1(0, unknown)
	assert session.observe(0).snapshot.digest == before.digest
	operation = ferrum_chem.DocumentOperationV1.set_arrow_properties(
		arrow_object_id, (change_type.start_head(True),),
	)
	session.apply_document_operation_v1(0, operation)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_document_operation_v1(0, operation)
	arrow = session.observe(1).projection.presentation_stack.entries[0].arrow
	assert (arrow.kind.kind, arrow.kind.start_head) == ("normal", True)
