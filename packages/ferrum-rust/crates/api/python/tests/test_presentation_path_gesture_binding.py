"""Python behavior coverage for generic visual authoring transitions."""

from __future__ import annotations

import pytest

import ferrum_chem


SOURCE = '<cdml xmlns="urn:ferrum:cdml"><standard/></cdml>'


def commit_transition(
	session: ferrum_chem.DocumentSession,
	request: ferrum_chem.SessionOperationTransitionRequestV1,
) -> ferrum_chem.SessionOperationResultV1:
	prepared = session.prepare_session_operation_transition_v1(request)
	return session.commit_session_operation_transition_v1(prepared)


def test_path_authoring_commits_a_created_root_through_generic_transition() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	gesture = session.begin_presentation_path_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationPathKindV1.polyline,
	)
	for point in ((0.0, 0.0), (30.0, 20.0)):
		session.add_presentation_path_gesture_point_v1(gesture, *point)
	overlay = session.preview_presentation_path_gesture_v1(gesture, None)
	assert (overlay.stroke_paint.kind, overlay.stroke_paint.role, overlay.stroke_paint.element) == (
		"theme_role", "document_foreground", None,
	)
	assert overlay.fill_paint is None
	request = session.resolve_presentation_path_gesture_v1(gesture, overlay)
	result = commit_transition(session, request)

	assert result.outcome.kind == "created_presentation_root_v1"
	assert result.outcome.created_presentation_root.document_object_id
	assert result.outcome.created_presentation_root.kind == ferrum_chem.CreatedPresentationRootKindV1.path
	assert result.observation.snapshot.revision == 1
	assert "<polyline id=\"" in result.observation.snapshot.cdml


def test_vector_preview_publishes_a_tagged_semantic_paint() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	gesture = session.begin_presentation_vector_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationVectorKindV1.rectangle, 0.0, 0.0,
	)
	preview = session.preview_presentation_vector_gesture_v1(gesture, 30.0, 20.0)
	overlay = preview.overlay

	assert (overlay.stroke_paint.kind, overlay.stroke_paint.role, overlay.stroke_paint.element) == (
		"theme_role", "document_foreground", None,
	)
	assert overlay.fill_paint is None


@pytest.mark.parametrize(
	("begin", "preview", "resolve", "expected_type"),
	(
		(
			"begin_curved_electron_arrow_gesture_v1",
			"preview_curved_electron_arrow_gesture_v1",
			"resolve_curved_electron_arrow_gesture_v1",
			"electron",
		),
		(
			"begin_curved_retro_arrow_gesture_v1",
			"preview_curved_retro_arrow_gesture_v1",
			"resolve_curved_retro_arrow_gesture_v1",
			"retro",
		),
		(
			"begin_curved_normal_reaction_arrow_gesture_v1",
			"preview_curved_normal_reaction_arrow_gesture_v1",
			"resolve_curved_normal_reaction_arrow_gesture_v1",
			"curved-normal",
		),
	),
)
def test_terminal_arrow_authoring_uses_generic_transition(
	begin: str,
	preview: str,
	resolve: str,
	expected_type: str,
) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	gesture = getattr(session, begin)(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
	)
	paint = getattr(session, preview)(gesture, 40.0, 0.0)
	request = getattr(session, resolve)(gesture, paint)
	result = commit_transition(session, request)

	assert result.outcome.kind == "created_presentation_root_v1"
	assert f'type="{expected_type}"' in result.observation.snapshot.cdml


def test_equilibrium_arrow_refuses_invalid_geometry_without_mutation() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()
	gesture = session.begin_curved_equilibrium_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 0.0,
	)
	with pytest.raises(ferrum_chem.CurvedEquilibriumArrowGestureError) as captured:
		session.preview_curved_equilibrium_arrow_gesture_v1(gesture, 40.0, 0.0)

	assert captured.value.category == ferrum_chem.CurvedEquilibriumArrowGestureCategoryV1.control_too_near_chord
	assert session.snapshot().revision == snapshot.revision
