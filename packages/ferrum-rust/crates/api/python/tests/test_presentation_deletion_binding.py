"""Semantic installed-extension checks for durable presentation deletion."""

import pytest

import ferrum_chem


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor"><text id="t"><point x="1" y="2"/>'
	'<ftext>label</ftext></text><v:opaque retained-id="t"/>'
	'<plus id="p"><point x="3" y="4"/></plus></cdml>'
)

BRACKET_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><polyline id="left" bracket_pair="left" bracket_side="left" spline="no">'
	'<point x="0" y="0"/><point x="1" y="1"/><point x="1" y="2"/>'
	'<point x="0" y="3"/></polyline><polyline id="right" bracket_pair="left" '
	'bracket_side="right" spline="no"><point x="4" y="0"/>'
	'<point x="3" y="1"/><point x="3" y="2"/><point x="4" y="3"/>'
	'</polyline></cdml>'
)


def test_presentation_deletion_is_exact_revisioned_and_preserves_opaque_content() -> None:
	kinds = ferrum_chem.DocumentPresentationRootKindV1
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.apply_document_operation_v1(
		0, ferrum_chem.DocumentOperationV1.delete_presentation_root("t", kinds.text),
	).observation

	assert changed.snapshot.revision == 1
	assert [root.kind for root in changed.projection.presentation_stack.roots] == ["plus"]
	assert '<v:opaque retained-id="t"' in changed.snapshot.cdml
	assert session.undo(1).observation.projection.presentation_stack.roots[0].kind == "text"
	assert session.redo(2).observation.projection.presentation_stack.roots[0].kind == "plus"
	assert type(kinds.text) is kinds
	assert kinds.__module__ == "ferrum_chem"
	with pytest.raises(AttributeError):
		kinds.text.value = "forged"


def test_presentation_deletion_rejects_wrong_kind_without_state_change() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()
	operation = ferrum_chem.DocumentOperationV1.delete_presentation_root(
		"t", ferrum_chem.DocumentPresentationRootKindV1.plus,
	)
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.apply_document_operation_v1(0, operation)
	assert caught.value.object_id == "t"
	after = session.snapshot()
	assert (after.revision, after.digest) == (before.revision, before.digest)


def test_complete_bracket_pair_deletes_atomically_through_frozen_selectors() -> None:
	kind = ferrum_chem.DocumentPresentationRootKindV1.polyline
	selector = ferrum_chem.DocumentPresentationRootSelectorV1.create
	session = ferrum_chem.DocumentSession.load(BRACKET_SOURCE)
	operation = ferrum_chem.DocumentOperationV1.delete_presentation_roots((
		selector("left", kind), selector("right", kind),
	))
	changed = session.apply_document_operation_v1(0, operation).observation
	assert not changed.projection.presentation_stack.roots
	restored = session.undo(1).observation.projection.presentation_stack
	assert restored.bracket_pairs[0].member_ids == ["left", "right"]


def test_presentation_deletion_set_rejects_empty_target_tuple() -> None:
	"""An atomic deletion must name at least one durable root."""
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.delete_presentation_roots(())


#============================================
def test_live_presentation_deletion_uses_durable_target_and_current_fence() -> None:
	"""The live adapter owns durable target lowering and rejects a stale fence."""
	kinds = ferrum_chem.DocumentPresentationRootKindV1
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	root = session.observe(snapshot.revision).projection.presentation_stack.roots[0]
	target = root.text.target
	result = session.apply_live_presentation_deletion_v1(
		snapshot.revision, snapshot.digest, ((target.id, kinds.text),),
	)
	assert result.observation.snapshot.revision == snapshot.revision + 1
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_live_presentation_deletion_v1(
			snapshot.revision, snapshot.digest, ((target.id, kinds.text),),
		)
