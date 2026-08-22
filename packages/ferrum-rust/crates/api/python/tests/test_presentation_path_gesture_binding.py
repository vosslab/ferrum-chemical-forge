"""Installed-extension contract for Rust-owned presentation path gestures."""

import pytest

import ferrum_chem


def test_presentation_path_binding_keeps_order_and_one_use_commit_contract() -> None:
	"""Publish ordered polygon points through one opaque prepared receipt."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_path_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationPathKindV1.polygon,
	)
	preview = session.preview_presentation_path_gesture_v1(
		gesture, ((0.0, 0.0), (20.0, 0.0), (0.0, 10.0)),
	)
	prepared = session.prepare_presentation_path_gesture_v1(gesture, preview)
	commit = session.commit_presentation_path_gesture_v1(prepared)
	assert preview.overlay.points == [(0.0, 0.0), (20.0, 0.0), (0.0, 10.0)]
	assert '<polygon id="' in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		session.commit_presentation_path_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.replayed_gesture


def test_presentation_path_invalid_geometry_is_typed_and_non_mutating() -> None:
	"""Refuse a collinear polygon before the immutable document can change."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_path_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationPathKindV1.polygon,
	)
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		session.preview_presentation_path_gesture_v1(
			gesture, ((0.0, 0.0), (10.0, 0.0), (20.0, 0.0)),
		)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.invalid_geometry
	assert session.snapshot().revision == snapshot.revision


def test_curved_electron_arrow_invalid_geometry_is_typed_and_non_mutating() -> None:
	"""Refuse a flat electron-arrow curve without changing the document."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_electron_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 10.0, 0.0,
	)
	with pytest.raises(ferrum_chem.CurvedElectronArrowGestureError) as captured:
		session.preview_curved_electron_arrow_gesture_v1(gesture, 20.0, 0.0)
	assert captured.value.category == ferrum_chem.CurvedElectronArrowGestureCategoryV1.control_too_near_chord
	assert captured.value.recovery == ferrum_chem.CurvedElectronArrowGestureRecoveryV1.change_geometry
	assert session.snapshot().revision == snapshot.revision
	assert session.snapshot().cdml == snapshot.cdml


def test_presentation_path_polyline_receipt_keeps_its_identity_after_a_polygon() -> None:
	"""Keep the second receipt's kind and generated identifier independent."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	polygon_snapshot = session.snapshot()
	polygon_gesture = session.begin_presentation_path_gesture_v1(
		polygon_snapshot.revision, polygon_snapshot.digest,
		ferrum_chem.PresentationPathKindV1.polygon,
	)
	polygon_preview = session.preview_presentation_path_gesture_v1(
		polygon_gesture, ((0.0, 0.0), (20.0, 0.0), (0.0, 10.0)),
	)
	polygon_prepared = session.prepare_presentation_path_gesture_v1(
		polygon_gesture, polygon_preview,
	)
	session.commit_presentation_path_gesture_v1(polygon_prepared)
	polyline_snapshot = session.snapshot()
	polyline_gesture = session.begin_presentation_path_gesture_v1(
		polyline_snapshot.revision, polyline_snapshot.digest,
		ferrum_chem.PresentationPathKindV1.polyline,
	)
	polyline_preview = session.preview_presentation_path_gesture_v1(
		polyline_gesture, ((30.0, 0.0), (50.0, 10.0)),
	)
	polyline_prepared = session.prepare_presentation_path_gesture_v1(
		polyline_gesture, polyline_preview,
	)
	commit = session.commit_presentation_path_gesture_v1(polyline_prepared)
	assert commit.kind == ferrum_chem.PresentationPathKindV1.polyline
	assert f'<polyline id="{commit.identifier}"' in commit.result.observation.snapshot.cdml
