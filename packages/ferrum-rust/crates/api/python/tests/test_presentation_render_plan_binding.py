"""Installed-wheel contract for fenced renderer-owned presentation plans."""

from __future__ import annotations

import pytest

import ferrum_chem


SOURCE = (
	'<c:cdml xmlns:c="urn:ferrum:cdml"><c:arrow id="arrow" type="normal" width="1">'
	'<c:point x="0" y="0"/><c:point x="40" y="0"/></c:arrow></c:cdml>'
)


def _plan(session: object) -> object:
	"""Observe the current immutable renderer plan through its exact fence."""
	snapshot = session.snapshot()
	return session.observe_presentation_render_plan_v1(snapshot.revision, snapshot.digest)


def test_presentation_plan_publishes_the_current_document_fence() -> None:
	"""A current snapshot publishes one renderer plan with matching provenance."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	plan = _plan(session)

	assert type(plan) is ferrum_chem.PresentationRenderPlanV1
	assert (plan.revision, plan.digest) == (snapshot.revision, snapshot.digest)


def test_presentation_plan_refuses_stale_or_wrong_provenance_without_mutation() -> None:
	"""The public plan seam validates both revision and digest before renderer delivery."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	before = session.snapshot()

	with pytest.raises(ferrum_chem.RenderProvenanceError):
		session.observe_presentation_render_plan_v1(before.revision, "0" * 64)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.observe_presentation_render_plan_v1(before.revision + 1, before.digest)
	assert session.snapshot().digest == before.digest


def test_presentation_plan_publication_fences_live_smarts_after_document_mutation() -> None:
	"""Raw SMARTS requires a plan published for the current document fence."""
	session = ferrum_chem.DocumentSession.load(
		'<cdml xmlns="urn:ferrum:cdml"><molecule id="m"><atom id="a" name="C">'
		'<point x="1" y="2"/></atom></molecule></cdml>'
	)
	_plan(session)
	assert session._run_live_document_smarts_query_v1("[C]", 128, 256).molecules[0].match_count == 1

	changed = session.submit(
		0, ferrum_chem.DocumentOperationV1.set_atom_position("a", 3.0, 2.0, 0.0),
	).observation.snapshot
	with pytest.raises(ferrum_chem.LiveDocumentSmartsError) as caught:
		session._run_live_document_smarts_query_v1("[C]", 128, 256)
	assert (caught.value.category, caught.value.reason) == (
		ferrum_chem.LiveDocumentSmartsCategoryV1.stale,
		ferrum_chem.LiveDocumentSmartsReasonV1.stale_document,
	)

	plan = _plan(session)
	result = session._run_live_document_smarts_query_v1("[C]", 128, 256)
	assert (plan.revision, plan.digest) == (changed.revision, changed.digest)
	assert result.molecules[0].match_count == 1
