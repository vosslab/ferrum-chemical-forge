"""Semantic Python coverage for Rust-owned presentation stack ordering."""

# PIP3 modules
import pytest

import ferrum_chem


_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor">'
	'<arrow id="a"><point x="0" y="0"/><point x="1" y="1"/></arrow>'
	'<v:opaque retained="yes"/><text id="t"><point x="2" y="2"/>'
	'<ftext>note</ftext></text><plus id="p"><point x="3" y="3"/></plus></cdml>'
)


#============================================
def _selector(identifier: str, kind: object) -> object:
	"""Build one exact frozen Rust selector."""
	return ferrum_chem.DocumentPresentationRootSelectorV1.create(identifier, kind)


#============================================
def _root_ids(session: object, revision: int) -> tuple[str, ...]:
	"""Return durable presentation IDs in projected source order."""
	identifiers = []
	for root in session.observe(revision).projection.presentation_stack.entries:
		payload = getattr(root, root.kind)
		identifiers.append(payload.target.document_object_id)
	return tuple(identifiers)


def _selector_for_entry(entry: object, kinds: object) -> object:
	"""Return one frozen durable selector for the observed presentation entry."""
	payload = getattr(entry, entry.kind)
	return _selector(payload.target.document_object_id, getattr(kinds, entry.kind))


#============================================
def test_stack_reorder_is_frozen_revisioned_and_history_owned() -> None:
	"""Closed selectors move exact roots while Rust retains content and history."""
	kinds = ferrum_chem.DocumentPresentationRootKindV1
	orders = ferrum_chem.DocumentPresentationStackOrderV1
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	a, t, _ = session.observe(0).projection.presentation_stack.entries
	a = _selector_for_entry(a, kinds)
	t = _selector_for_entry(t, kinds)
	initial_ids = _root_ids(session, 0)
	assert a.document_object_id and a.kind == kinds.arrow
	with pytest.raises(AttributeError):
		a.document_object_id = "changed"

	operation = ferrum_chem.DocumentOperationV1.reorder_presentation_roots(
		orders.bring_to_front, (a, t),
	)
	result = session.apply_document_operation_v1(0, operation)
	assert result.observation.snapshot.revision == 1
	assert _root_ids(session, 1) == (initial_ids[2], initial_ids[0], initial_ids[1])
	assert 'retained="yes"' in result.observation.snapshot.cdml
	session.undo(1)
	assert _root_ids(session, 2) == initial_ids


#============================================
def test_stack_reorder_rejects_forged_shape_wrong_kind_and_stale_revision() -> None:
	"""Malformed or stale ordering intent never changes the current Rust state."""
	kinds = ferrum_chem.DocumentPresentationRootKindV1
	orders = ferrum_chem.DocumentPresentationStackOrderV1
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	entry = session.observe(0).projection.presentation_stack.entries[0]
	selector = _selector_for_entry(entry, kinds)
	with pytest.raises(TypeError):
		ferrum_chem.DocumentOperationV1.reorder_presentation_roots(
			orders.bring_to_front, [selector],
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem.DocumentOperationV1.reorder_presentation_roots(
			orders.reverse_selected_slots, (selector,),
		)

	wrong = ferrum_chem.DocumentOperationV1.reorder_presentation_roots(
		orders.send_to_back, (_selector(selector.document_object_id, kinds.text),),
	)
	before = session.snapshot
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError):
		session.apply_document_operation_v1(0, wrong)
	assert session.snapshot == before

	changed = ferrum_chem.DocumentOperationV1.reorder_presentation_roots(
		orders.bring_to_front, (selector,),
	)
	session.apply_document_operation_v1(0, changed)
	after = session.snapshot
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_document_operation_v1(0, changed)
	assert session.snapshot == after


#============================================
def test_live_presentation_reorder_uses_durable_targets_and_current_fence() -> None:
	"""The live adapter lowers durable roots only after fence validation."""
	kinds = ferrum_chem.DocumentPresentationRootKindV1
	orders = ferrum_chem.DocumentPresentationStackOrderV1
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	snapshot = session.snapshot()
	roots = session.observe(snapshot.revision).projection.presentation_stack.entries
	targets = tuple(
		(getattr(root, root.kind).target.document_object_id, getattr(kinds, root.kind))
		for root in roots[:2]
	)
	result = session.apply_live_presentation_reorder_v1(
		snapshot.revision, snapshot.digest, orders.bring_to_front, targets,
	)
	assert result.observation.snapshot.revision == snapshot.revision + 1
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.apply_live_presentation_reorder_v1(
			snapshot.revision, snapshot.digest, orders.bring_to_front, targets,
		)
