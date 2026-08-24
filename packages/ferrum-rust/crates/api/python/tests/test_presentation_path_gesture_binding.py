"""Installed-extension contract for Rust-owned presentation path gestures."""

import pytest

import ferrum_chem


def test_curved_terminal_projection_uses_one_closed_python_payload() -> None:
	"""Expose all terminal families through one typed Rust projection payload."""
	session = ferrum_chem.DocumentSession.load(
		"<cdml xmlns='urn:ferrum:cdml' version='26.07'>"
		"<arrow id='electron' type='electron'><point x='0' y='0'/><point x='10' y='10'/><point x='20' y='0'/></arrow>"
		"<arrow id='retro' type='retro'><point x='30' y='0'/><point x='40' y='10'/><point x='50' y='0'/></arrow>"
		"<arrow id='normal' type='curved-normal'><point x='60' y='0'/><point x='70' y='10'/><point x='80' y='0'/></arrow>"
		"</cdml>",
	)
	observation = session.observe(0)
	arrows = [root.arrow for root in observation.projection.presentation_stack.roots]
	assert [arrow.kind.kind for arrow in arrows] == ["curved_terminal"] * 3
	assert [arrow.kind.terminal_kind for arrow in arrows] == ["electron", "retro", "normal"]
	assert [len(arrow.source_path.points) for arrow in arrows] == [3, 3, 3]
	plan = session.observe_presentation_render_plan_v1(
		observation.snapshot.revision, observation.snapshot.digest,
	)
	assert [root.target.source_id for root in plan.roots] == ["electron", "retro", "normal"]
	assert [[operation.kind for operation in root.vector_operations] for root in plan.roots] == [
		["path", "path"], ["path", "path"], ["path", "path"],
	]


@pytest.mark.parametrize(
	("kind", "points"),
	[
		(ferrum_chem.PresentationPathKindV1.polyline, ((0.0, 0.0), (20.0, 10.0))),
		(ferrum_chem.PresentationPathKindV1.polygon, ((0.0, 0.0), (20.0, 0.0), (0.0, 10.0))),
	],
)
def test_presentation_path_incremental_receipts_keep_order_and_exclude_hover(
	kind: object, points: tuple[tuple[float, float], ...],
) -> None:
	"""Keep ordered accepted points separate from an uncommitted hover point."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_path_gesture_v1(snapshot.revision, snapshot.digest, kind)
	session.add_presentation_path_gesture_point_v1(gesture, *points[0])
	for point in points[1:]:
		session.add_presentation_path_gesture_point_v1(gesture, *point)
	hover = (97.0, 89.0)
	overlay = session.preview_presentation_path_gesture_v1(gesture, hover)
	assert overlay.accepted_points == list(points)
	assert overlay.points == [*points, hover]


@pytest.mark.parametrize(
	("kind", "points", "element"),
	[
		(ferrum_chem.PresentationPathKindV1.polyline, ((0.0, 0.0), (20.0, 10.0)), "polyline"),
		(ferrum_chem.PresentationPathKindV1.polygon, ((0.0, 0.0), (20.0, 0.0), (0.0, 10.0)), "polygon"),
	],
)
def test_presentation_path_prepared_commit_is_atomic_and_single_use(
		kind: object, points: tuple[tuple[float, float], ...], element: str,
		) -> None:
	"""Persist each supported path once without committing its hover point."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_path_gesture_v1(snapshot.revision, snapshot.digest, kind)
	for point in points:
		session.add_presentation_path_gesture_point_v1(gesture, *point)
	overlay = session.preview_presentation_path_gesture_v1(gesture, (97.0, 89.0))
	prepared = session.prepare_presentation_path_gesture_v1(gesture, overlay)
	commit = session.commit_presentation_path_gesture_v1(prepared)
	assert f"<{element} id=\"" in commit.result.observation.snapshot.cdml
	assert 'x="97" y="89"' not in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		session.commit_presentation_path_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.replayed_gesture


def test_presentation_path_incremental_refusals_are_typed_and_non_mutating() -> None:
	"""Keep incomplete, repeated, stale, and foreign candidates outside document mutation."""
	owner = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	foreign = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = owner.snapshot()
	gesture = owner.begin_presentation_path_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationPathKindV1.polygon,
	)
	owner.add_presentation_path_gesture_point_v1(gesture, 0.0, 0.0)
	overlay = owner.preview_presentation_path_gesture_v1(gesture, None)
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		owner.prepare_presentation_path_gesture_v1(gesture, overlay)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.incomplete
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		owner.add_presentation_path_gesture_point_v1(gesture, 0.0, 0.0)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.invalid_geometry
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		foreign.preview_presentation_path_gesture_v1(gesture, None)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.foreign_session
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		owner.begin_presentation_path_gesture_v1(
			snapshot.revision + 1, snapshot.digest, ferrum_chem.PresentationPathKindV1.polyline,
		)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.stale_snapshot
	assert owner.snapshot().revision == snapshot.revision
	assert foreign.snapshot().revision == 0


def test_presentation_path_cancel_consumes_the_candidate_without_mutation() -> None:
	"""Report cancellation separately and preserve the immutable document receipt."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_presentation_path_gesture_v1(
		snapshot.revision, snapshot.digest, ferrum_chem.PresentationPathKindV1.polyline,
	)
	first_overlay = session.preview_presentation_path_gesture_v1(gesture, (5.0, 5.0))
	session.add_presentation_path_gesture_point_v1(gesture, 0.0, 0.0)
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		session.cancel_presentation_path_gesture_v1(gesture)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.cancelled
	assert captured.value.recovery == ferrum_chem.PresentationPathGestureRecoveryV1.document_unchanged
	assert first_overlay.points == [(5.0, 5.0)]
	with pytest.raises(ferrum_chem.PresentationPathGestureError) as captured:
		session.preview_presentation_path_gesture_v1(gesture, None)
	assert captured.value.category == ferrum_chem.PresentationPathGestureCategoryV1.replayed_gesture
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


@pytest.mark.parametrize(
	("begin", "preview", "prepare", "commit", "error_type", "category", "recovery", "replayed"),
	[
		(
			lambda session, snapshot: session.begin_curved_electron_arrow_gesture_v1(
				snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
			),
			lambda session, gesture: session.preview_curved_electron_arrow_gesture_v1(
				gesture, 40.0, 0.0,
			),
			lambda session, gesture, value: session.prepare_curved_electron_arrow_gesture_v1(
				gesture, value,
			),
			lambda session, value: session.commit_curved_electron_arrow_gesture_v1(value),
			ferrum_chem.CurvedElectronArrowGestureError,
			ferrum_chem.CurvedElectronArrowGestureCategoryV1.foreign_session,
			ferrum_chem.CurvedElectronArrowGestureRecoveryV1.refresh_and_restart,
			ferrum_chem.CurvedElectronArrowGestureCategoryV1.replayed_gesture,
		),
		(
			lambda session, snapshot: session.begin_curved_retro_arrow_gesture_v1(
				snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
			),
			lambda session, gesture: session.preview_curved_retro_arrow_gesture_v1(
				gesture, 40.0, 0.0,
			),
			lambda session, gesture, value: session.prepare_curved_retro_arrow_gesture_v1(
				gesture, value,
			),
			lambda session, value: session.commit_curved_retro_arrow_gesture_v1(value),
			ferrum_chem.CurvedRetroArrowGestureError,
			ferrum_chem.CurvedRetroArrowGestureCategoryV1.foreign_session,
			ferrum_chem.CurvedRetroArrowGestureRecoveryV1.refresh_and_restart,
			ferrum_chem.CurvedRetroArrowGestureCategoryV1.replayed_gesture,
		),
		(
			lambda session, snapshot: session.begin_curved_normal_reaction_arrow_gesture_v1(
				snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
			),
			lambda session, gesture: session.preview_curved_normal_reaction_arrow_gesture_v1(
				gesture, 40.0, 0.0,
			),
			lambda session, gesture, value: session.prepare_curved_normal_reaction_arrow_gesture_v1(
				gesture, value,
			),
			lambda session, value: session.commit_curved_normal_reaction_arrow_gesture_v1(value),
			ferrum_chem.CurvedNormalReactionArrowGestureError,
			ferrum_chem.CurvedNormalReactionArrowGestureCategoryV1.foreign_session,
			ferrum_chem.CurvedNormalReactionArrowGestureRecoveryV1.refresh_and_restart,
			ferrum_chem.CurvedNormalReactionArrowGestureCategoryV1.replayed_gesture,
		),
	],
)
def test_terminal_arrow_foreign_commit_keeps_owner_receipt_redeemable(
	begin: object,
	preview: object,
	prepare: object,
	commit: object,
	error_type: object,
	category: object,
	recovery: object,
	replayed: object,
) -> None:
	"""Keep a foreign commit from consuming any terminal-arrow receipt."""
	owner = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	foreign = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	gesture = begin(owner, owner.snapshot())
	value = preview(owner, gesture)
	assert type(value.plan) is ferrum_chem.PresentationRenderPlanV1
	assert len(value.plan.roots[0].vector_operations) == 2
	prepared = prepare(owner, gesture, value)
	with pytest.raises(error_type) as captured:
		commit(foreign, prepared)
	assert (captured.value.category, captured.value.recovery) == (category, recovery)
	assert (owner.snapshot().revision, foreign.snapshot().revision) == (0, 0)
	commit(owner, prepared)
	assert owner.snapshot().revision == 1
	with pytest.raises(error_type) as captured:
		commit(owner, prepared)
	assert captured.value.category == replayed
	assert owner.snapshot().revision == 1


@pytest.mark.parametrize(
	("begin", "preview", "error_type", "category", "recovery"),
	[
		(
			lambda session, snapshot: session.begin_curved_electron_arrow_gesture_v1(
				snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
			),
			lambda session, gesture: session.preview_curved_electron_arrow_gesture_v1(
				gesture, 40.0, 0.0,
			),
			ferrum_chem.CurvedElectronArrowGestureError,
			ferrum_chem.CurvedElectronArrowGestureCategoryV1.foreign_session,
			ferrum_chem.CurvedElectronArrowGestureRecoveryV1.refresh_and_restart,
		),
		(
			lambda session, snapshot: session.begin_curved_retro_arrow_gesture_v1(
				snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
			),
			lambda session, gesture: session.preview_curved_retro_arrow_gesture_v1(
				gesture, 40.0, 0.0,
			),
			ferrum_chem.CurvedRetroArrowGestureError,
			ferrum_chem.CurvedRetroArrowGestureCategoryV1.foreign_session,
			ferrum_chem.CurvedRetroArrowGestureRecoveryV1.refresh_and_restart,
		),
		(
			lambda session, snapshot: session.begin_curved_normal_reaction_arrow_gesture_v1(
				snapshot.revision, snapshot.digest, 0.0, 0.0, 20.0, 20.0,
			),
			lambda session, gesture: session.preview_curved_normal_reaction_arrow_gesture_v1(
				gesture, 40.0, 0.0,
			),
			ferrum_chem.CurvedNormalReactionArrowGestureError,
			ferrum_chem.CurvedNormalReactionArrowGestureCategoryV1.foreign_session,
			ferrum_chem.CurvedNormalReactionArrowGestureRecoveryV1.refresh_and_restart,
		),
	],
)
def test_terminal_arrow_foreign_session_errors_request_restart(
	begin: object, preview: object, error_type: object, category: object, recovery: object,
) -> None:
	"""Expose the same restartable foreign-session contract for each terminal binding."""
	owner = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	foreign = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	gesture = begin(owner, owner.snapshot())
	with pytest.raises(error_type) as captured:
		preview(foreign, gesture)
	assert (captured.value.category, captured.value.recovery) == (category, recovery)
	assert preview(owner, gesture) is not None


def test_curved_retro_arrow_invalid_geometry_is_typed_and_non_mutating() -> None:
	"""Refuse a flat retro curve with its typed recovery and no document change."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_retro_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 10.0, 0.0,
	)
	with pytest.raises(ferrum_chem.CurvedRetroArrowGestureError) as captured:
		session.preview_curved_retro_arrow_gesture_v1(gesture, 20.0, 0.0)
	assert captured.value.category == ferrum_chem.CurvedRetroArrowGestureCategoryV1.control_too_near_chord
	assert captured.value.recovery == ferrum_chem.CurvedRetroArrowGestureRecoveryV1.change_geometry
	assert session.snapshot().revision == snapshot.revision
	assert session.snapshot().cdml == snapshot.cdml


def test_curved_retro_arrow_binding_persists_only_the_closed_three_point_type() -> None:
	"""Commit one Rust-owned retro curve with an opaque root selector once."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_retro_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 10.0, 10.0,
	)
	preview = session.preview_curved_retro_arrow_gesture_v1(gesture, 20.0, 0.0)
	assert type(preview.plan) is ferrum_chem.PresentationRenderPlanV1
	assert len(preview.plan.roots[0].vector_operations) == 2
	prepared = session.prepare_curved_retro_arrow_gesture_v1(gesture, preview)
	commit = session.commit_curved_retro_arrow_gesture_v1(prepared)
	assert isinstance(commit.root, ferrum_chem.PresentationGestureRootSelectorV1)
	assert commit.root.kind == ferrum_chem.PresentationGestureRootKindV1.arrow
	assert '<arrow id="' in commit.result.observation.snapshot.cdml
	assert f'id="{commit.root.identifier}"' in commit.result.observation.snapshot.cdml
	assert 'type="retro"' in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.CurvedRetroArrowGestureError) as captured:
		session.commit_curved_retro_arrow_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.CurvedRetroArrowGestureCategoryV1.replayed_gesture


def test_curved_retro_arrow_error_keeps_shared_typed_facts_without_electron_copy() -> None:
	"""Expose the shared native error taxonomy with family-neutral public text."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_retro_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 10.0, 0.0,
	)
	with pytest.raises(ferrum_chem.CurvedRetroArrowGestureError) as captured:
		session.preview_curved_retro_arrow_gesture_v1(gesture, 20.0, 0.0)
	assert str(captured.value) == "curved terminal-arrow control point is too close to its chord"
	assert captured.value.category == ferrum_chem.CurvedRetroArrowGestureCategoryV1.control_too_near_chord
	assert captured.value.recovery == ferrum_chem.CurvedRetroArrowGestureRecoveryV1.change_geometry


def test_curved_normal_reaction_arrow_lifecycle_persists_closed_renderer_plan() -> None:
	"""Commit one curved normal arrow through its public renderer-owned plan."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_normal_reaction_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 10.0, 10.0,
	)
	preview = session.preview_curved_normal_reaction_arrow_gesture_v1(gesture, 20.0, 0.0)
	assert type(preview.plan) is ferrum_chem.PresentationRenderPlanV1
	assert len(preview.plan.roots[0].vector_operations) == 2
	prepared = session.prepare_curved_normal_reaction_arrow_gesture_v1(gesture, preview)
	commit = session.commit_curved_normal_reaction_arrow_gesture_v1(prepared)
	assert '<arrow id="' in commit.result.observation.snapshot.cdml
	assert 'type="curved-normal"' in commit.result.observation.snapshot.cdml
	with pytest.raises(ferrum_chem.CurvedNormalReactionArrowGestureError) as captured:
		session.commit_curved_normal_reaction_arrow_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.CurvedNormalReactionArrowGestureCategoryV1.replayed_gesture


def test_curved_normal_reaction_arrow_refusals_are_typed_and_non_mutating() -> None:
	"""Reject stale and flat normal-arrow candidates before document mutation."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_normal_reaction_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 10.0, 0.0,
	)
	with pytest.raises(ferrum_chem.CurvedNormalReactionArrowGestureError) as captured:
		session.preview_curved_normal_reaction_arrow_gesture_v1(gesture, 20.0, 0.0)
	assert captured.value.category == ferrum_chem.CurvedNormalReactionArrowGestureCategoryV1.control_too_near_chord
	assert session.snapshot().revision == snapshot.revision
	with pytest.raises(ferrum_chem.CurvedNormalReactionArrowGestureError) as captured:
		session.begin_curved_normal_reaction_arrow_gesture_v1(
			snapshot.revision + 1, snapshot.digest, 0.0, 0.0, 10.0, 10.0,
		)
	assert captured.value.category == ferrum_chem.CurvedNormalReactionArrowGestureCategoryV1.stale_snapshot


def test_curved_equilibrium_arrow_lifecycle_issues_one_renderer_plan() -> None:
	"""Commit one curved-equilibrium arrow through its opaque native receipt."""
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	snapshot = session.snapshot()
	gesture = session.begin_curved_equilibrium_arrow_gesture_v1(
		snapshot.revision, snapshot.digest, 0.0, 0.0, 40.0, 20.0,
	)
	preview = session.preview_curved_equilibrium_arrow_gesture_v1(gesture, 80.0, 0.0)
	assert type(preview.plan) is ferrum_chem.PresentationRenderPlanV1
	assert len(preview.plan.roots[0].vector_operations) == 3
	prepared = session.prepare_curved_equilibrium_arrow_gesture_v1(gesture, preview)
	commit = session.commit_curved_equilibrium_arrow_gesture_v1(prepared)
	observation = commit.result.observation
	arrow = observation.projection.presentation_stack.roots[0].arrow
	assert (arrow.kind.kind, len(arrow.source_path.points)) == ("curved_equilibrium", 3)
	assert isinstance(commit.root, ferrum_chem.PresentationGestureRootSelectorV1)
	assert commit.root.kind == ferrum_chem.PresentationGestureRootKindV1.arrow
	assert f'id="{commit.root.identifier}"' in commit.result.observation.snapshot.cdml
	assert 'type="curved-equilibrium"' in commit.result.observation.snapshot.cdml
	plan = session.observe_presentation_render_plan_v1(
		observation.snapshot.revision, observation.snapshot.digest,
	)
	root, = plan.roots
	assert root.target.source_id == commit.root.identifier
	assert [operation.kind for operation in root.vector_operations] == ["path"] * 3
	with pytest.raises(ferrum_chem.CurvedEquilibriumArrowGestureError) as captured:
		session.commit_curved_equilibrium_arrow_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.CurvedEquilibriumArrowGestureCategoryV1.replayed_gesture


def test_curved_equilibrium_arrow_refuses_invalid_and_foreign_capabilities_without_mutation() -> None:
	"""Keep curved-equilibrium gesture capabilities bound to one native session."""
	owner = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	foreign = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml' version='26.07'/>")
	owner_snapshot = owner.snapshot()
	foreign_snapshot = foreign.snapshot()
	gesture = owner.begin_curved_equilibrium_arrow_gesture_v1(
		owner_snapshot.revision, owner_snapshot.digest, 0.0, 0.0, 40.0, 20.0,
	)
	with pytest.raises(ferrum_chem.CurvedEquilibriumArrowGestureError) as captured:
		foreign.preview_curved_equilibrium_arrow_gesture_v1(gesture, 80.0, 0.0)
	assert captured.value.category == ferrum_chem.CurvedEquilibriumArrowGestureCategoryV1.foreign_session
	assert captured.value.recovery == ferrum_chem.CurvedEquilibriumArrowGestureRecoveryV1.refresh_and_restart
	assert foreign.snapshot().revision == foreign_snapshot.revision
	assert foreign.snapshot().cdml == foreign_snapshot.cdml

	preview = owner.preview_curved_equilibrium_arrow_gesture_v1(gesture, 80.0, 0.0)
	prepared = owner.prepare_curved_equilibrium_arrow_gesture_v1(gesture, preview)
	with pytest.raises(ferrum_chem.CurvedEquilibriumArrowGestureError) as captured:
		foreign.commit_curved_equilibrium_arrow_gesture_v1(prepared)
	assert captured.value.category == ferrum_chem.CurvedEquilibriumArrowGestureCategoryV1.foreign_session
	assert foreign.snapshot().revision == foreign_snapshot.revision
	assert foreign.snapshot().cdml == foreign_snapshot.cdml
	owner.commit_curved_equilibrium_arrow_gesture_v1(prepared)

	invalid_snapshot = owner.snapshot()
	invalid = owner.begin_curved_equilibrium_arrow_gesture_v1(
		invalid_snapshot.revision, invalid_snapshot.digest, 0.0, 0.0, -10.0, 0.0,
	)
	with pytest.raises(ferrum_chem.CurvedEquilibriumArrowGestureError) as captured:
		owner.preview_curved_equilibrium_arrow_gesture_v1(invalid, 80.0, 0.0)
	assert captured.value.category == ferrum_chem.CurvedEquilibriumArrowGestureCategoryV1.control_too_near_chord
	assert owner.snapshot().revision == invalid_snapshot.revision
