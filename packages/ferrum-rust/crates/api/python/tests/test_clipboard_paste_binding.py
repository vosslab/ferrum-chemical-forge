"""Installed-extension checks for private bounded native clipboard Paste."""

import pytest

import ferrum_chem


_FRAGMENT = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><molecule id="m">
 <atom id="a" name="C"><point x="1" y="2"/></atom>
 <atom id="b" name="O"><point x="41" y="2"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/>
</molecule><plus id="p"><point x="31" y="42"/></plus></cdml>
"""


def _facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return durable facts needed to prove atomic refusal."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


def _direct_root_receipt(observation: object) -> tuple[tuple[str, str], ...]:
	"""Return the durable direct-root receipt represented by an observation."""
	direct_roots = observation.projection.direct_roots
	receipt = tuple(
		(root.kind, root.document_object_id)
		for root in direct_roots
	)
	return receipt


def test_private_paste_prepares_off_session_and_commits_one_translated_edit() -> None:
	"""The closed plan remaps identities and moves every inserted root once."""
	plan = ferrum_chem.prepare_clipboard_paste_v1(_FRAGMENT)
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	before = session.snapshot()
	result = session.apply_clipboard_paste_v1(0, before.digest, plan)
	observation = result.operation.observation
	molecule = observation.projection.molecules[0]
	plus = observation.projection.presentation_stack.entries[0].plus

	assert tuple(root[0] for root in result.pasted_roots) == ("molecule", "plus")
	assert isinstance(
		observation.projection.direct_roots[0], ferrum_chem.DocumentDirectRootV1,
	)
	assert result.pasted_roots == tuple(
		(root.kind, root.document_object_id)
		for root in observation.projection.direct_roots
	)
	assert [
		observation.snapshot.revision,
		molecule.atoms[0].position.x,
		molecule.atoms[0].position.y,
		plus.anchor.x,
		plus.anchor.y,
	] == pytest.approx([1.0, 21.0, 22.0, 51.0, 62.0], abs=0.02)


def test_private_paste_is_one_history_step_and_plan_is_reusable() -> None:
	"""Undo/Redo restores exact states and reuse gets collision-free fresh roots."""
	plan = ferrum_chem.prepare_clipboard_paste_v1(_FRAGMENT)
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	baseline = session.snapshot()
	first = session.apply_clipboard_paste_v1(0, baseline.digest, plan)
	first_ids = tuple(root[1] for root in first.pasted_roots)
	undone = session.undo(1)
	redone = session.redo(2)
	redone_root_ids = frozenset(root[1] for root in _direct_root_receipt(redone.observation))
	second = session.apply_clipboard_paste_v1(
		3, redone.observation.snapshot.digest, plan,
	)
	second_ids = tuple(root[1] for root in second.pasted_roots)
	second_new_roots = tuple(
		root
		for root in _direct_root_receipt(second.operation.observation)
		if root[1] not in redone_root_ids
	)

	assert first.pasted_roots == tuple(
		(root.kind, root.document_object_id)
		for root in first.operation.observation.projection.direct_roots
	)
	assert second.pasted_roots == second_new_roots
	assert (
		undone.observation.snapshot.cdml,
		redone.observation.snapshot.cdml,
	) == (baseline.cdml, first.operation.observation.snapshot.cdml)
	assert set(first_ids).isdisjoint(second_ids) and second.operation.observation.snapshot.revision == 4


@pytest.mark.parametrize(
	"source, reason",
	(
		(b"<cdml xmlns='urn:ferrum:cdml'/>", "exact built-in string"),
		("\ud800", "valid UTF-8 text"),
		("<cdml xmlns='urn:ferrum:cdml'>", "invalid CDML"),
		("<cdml xmlns='urn:ferrum:cdml'><paper id='paper'/></cdml>", "unsupported direct-root"),
		("<cdml xmlns='urn:ferrum:cdml'/>/or empty", "invalid CDML"),
	),
)
def test_private_paste_rejects_invalid_external_fragments(
		source: object, reason: str,
		) -> None:
	"""Wrong Python containers and invalid fragment grammar stay privately typed."""
	with pytest.raises(ferrum_chem.DocumentClipboardPasteError) as caught:
		ferrum_chem.prepare_clipboard_paste_v1(source)
	assert reason in caught.value.reason


def test_private_paste_reauthenticates_before_mutation() -> None:
	"""Stale revision or digest never changes the target document."""
	plan = ferrum_chem.prepare_clipboard_paste_v1(_FRAGMENT)
	session = ferrum_chem.DocumentSession.create_empty_document_v1()
	before = session.snapshot()
	for revision, digest in ((1, before.digest), (0, "0" * 64)):
		with pytest.raises(ferrum_chem.DocumentClipboardPasteError) as caught:
			session.apply_clipboard_paste_v1(revision, digest, plan)
		assert caught.value.reason
	assert _facts(session.snapshot()) == _facts(before)
