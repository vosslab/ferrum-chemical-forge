"""Installed-extension behavior for geometric presentation appearance edits."""

# PIP3 modules
import ferrum_chem
import pytest


def _rectangle(observation: object) -> object:
	"""Return the projected durable rectangle from an exact observation."""
	root = next(root for root in observation.projection.presentation_stack.entries
				if root.kind == "rectangle")
	return root.shape


def test_geometric_properties_are_atomic_frozen_and_history_aware() -> None:
	"""Apply one closed shape patch while preserving opaque retained content."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><rect id="shape" x1="1" y1="2" '
		'x2="3" y2="4" color="#ABC" background-color="#dEf" keep="yes">'
		'<v:opaque/></rect></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentGeometricPropertyChangeV1
	changes = (
		change_type.line_width(2.5),
		change_type.stroke_color("#445566"),
		change_type.fill_color(None),
	)
	operation = ferrum_chem.DocumentOperationV1.set_geometric_properties(
		"shape", changes,
	)
	changed = session.apply_document_operation_v1(0, operation).observation
	shape = _rectangle(changed)
	assert changed.snapshot.revision == 1
	assert (shape.stroke.width, shape.stroke.color) == (2.5, "#445566")
	assert shape.fill.color is None
	assert 'keep="yes"' in changed.snapshot.cdml
	assert "opaque" in changed.snapshot.cdml
	with pytest.raises(AttributeError):
		changes[0].value = 3.0
	assert _rectangle(session.undo(1).observation).stroke.color == "#aabbcc"
	assert _rectangle(session.redo(2).observation).stroke.color == "#445566"


def test_geometric_properties_reject_hostile_or_inapplicable_intent() -> None:
	"""Reject invalid Python shapes and wrong persistent targets without mutation."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml"><polyline id="line"><point x="0" y="0"/>'
		'<point x="2" y="2"/></polyline>'
		'<polyline id="wave" style="wavy"><point x="0" y="0"/>'
		'<point x="2" y="2"/></polyline></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentGeometricPropertyChangeV1
	before = session.observe(0).snapshot
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_width(True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.stroke_color("#abc")

	class StringSubclass(str):
		"""Hostile string subclass rejected at the exact DTO boundary."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.stroke_color(StringSubclass("#112233"))
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_geometric_properties(
			"line", tuple(change_type.line_width(1.0) for _ in range(4)),
		)

	class TupleSubclass(tuple):
		"""Hostile tuple subclass rejected before item extraction."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_geometric_properties(
			"line", TupleSubclass((change_type.line_width(1.0),)),
		)
	for identifier, change in (
		("line", change_type.fill_color(None)),
		("wave", change_type.line_width(2.0)),
	):
		operation = ferrum_chem.DocumentOperationV1.set_geometric_properties(
			identifier, (change,),
		)
		with pytest.raises(ferrum_chem.OperationValidationError):
			session.apply_document_operation_v1(0, operation)
		assert session.observe(0).snapshot.digest == before.digest
	unknown = ferrum_chem.DocumentOperationV1.set_geometric_properties(
		"ferrum-document-object-v1/00000000000000000000000000000000",
		(change_type.line_width(2.0),),
	)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.apply_document_operation_v1(0, unknown)
