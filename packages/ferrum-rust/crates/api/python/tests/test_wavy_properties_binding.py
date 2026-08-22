"""Installed-extension behavior for Wavy appearance edits."""

# PIP3 modules
import ferrum_chem
import pytest


def _wavy(observation: object) -> object:
	"""Return the projected durable Wavy payload from an exact observation."""
	root = next(root for root in observation.projection.presentation_stack.roots
				if root.kind == "wavy")
	assert root.polyline.target.record_kind == "polyline"
	return root.polyline


def test_wavy_properties_preserve_authored_path_and_history() -> None:
	"""Apply one closed patch without regenerating stored Wavy geometry."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><polyline id="wave" style="wavy" '
		'color="#ABC" keep="yes"><point x="0" y="0"/>'
		'<point x="3" y="2"/><point x="6" y="0"/><v:opaque/>'
		'</polyline></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentWavyPropertyChangeV1
	operation = ferrum_chem.DocumentOperationV1.set_wavy_properties(
		"wave",
		(change_type.line_width(2.5), change_type.line_color("#445566")),
	)
	changed = session.submit(0, operation).observation
	wavy = _wavy(changed)
	assert [(point.x, point.y) for point in wavy.path.points] == [
		(0.0, 0.0), (3.0, 2.0), (6.0, 0.0),
	]
	assert (wavy.stroke.width, wavy.stroke.color) == (2.5, "#445566")
	assert 'keep="yes"' in changed.snapshot.cdml
	assert "opaque" in changed.snapshot.cdml
	assert _wavy(session.undo(1).observation).stroke.color == "#aabbcc"
	assert _wavy(session.redo(2).observation).stroke.color == "#445566"


def test_wavy_properties_reject_hostile_or_wrong_targets_atomically() -> None:
	"""Reject overlong, subclassed, ordinary, unknown, and stale requests."""
	source = (
		'<cdml xmlns="urn:ferrum:cdml"><polyline id="wave" style="wavy"><point x="0" y="0"/>'
		'<point x="2" y="2"/></polyline><polyline id="ordinary">'
		'<point x="0" y="0"/><point x="2" y="2"/></polyline></cdml>'
	)
	session = ferrum_chem.DocumentSession.load(source)
	change_type = ferrum_chem.DocumentWavyPropertyChangeV1
	before = session.observe(0).snapshot
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_width(True)
	with pytest.raises(ferrum_chem.OperationValidationError):
		change_type.line_color("#abc")
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_wavy_properties(
			"wave", tuple(change_type.line_width(1.0) for _ in range(3)),
		)

	class TupleSubclass(tuple):
		"""Hostile tuple subclass rejected before item extraction."""

	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.set_wavy_properties(
			"wave", TupleSubclass((change_type.line_width(1.0),)),
		)
	for identifier in ("ordinary", "missing"):
		operation = ferrum_chem.DocumentOperationV1.set_wavy_properties(
			identifier, (change_type.line_width(2.0),),
		)
		with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as error:
			session.submit(0, operation)
		assert error.value.object_id == identifier
		assert session.observe(0).snapshot.digest == before.digest


#============================================
def test_prepared_wavy_creation_exposes_rust_owned_identity_and_path() -> None:
	"""Commit one bounded prepared Wavy without Python-authored persistent geometry."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	prepared = session.prepare_create_wavy_v1(0, 0.0, 0.0, 48.0, 0.0)
	assert prepared.identifier == "ferrum-presentation-v1-0"
	assert session.observe(0).snapshot.revision == 0
	result = session.commit_create_wavy(0, prepared)
	root = next(
		root for root in result.observation.projection.presentation_stack.roots
		if root.kind == "wavy"
	)
	assert root.polyline.target.source_id == prepared.identifier
	assert len(root.polyline.path.points) == 5
	assert (root.polyline.path.points[0].x, root.polyline.path.points[0].y) == (0.0, 0.0)
	assert (root.polyline.path.points[-1].x, root.polyline.path.points[-1].y) == (48.0, 0.0)
	assert (root.polyline.stroke.width, root.polyline.stroke.color) == (1.5, "#000000")
