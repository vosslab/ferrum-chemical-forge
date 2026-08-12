"""Installed-wheel contract for Ferrum-Chem's revisioned document boundary."""

from __future__ import annotations

from pathlib import Path

import pytest

import ferrum_chem


SOURCE = (
	"<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\">"
	"<point x=\"1\" y=\"2\"/></atom></molecule></cdml>"
)


def set_atom(element: str) -> ferrum_chem.DocumentOperationV1:
	return ferrum_chem.DocumentOperationV1.set_atom_element("a", element)


def test_load_returns_an_immutable_authoritative_snapshot() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	snapshot = session.snapshot()

	assert "molecule" in snapshot.cdml
	assert snapshot.revision == 0
	assert len(snapshot.digest) == 64
	assert snapshot.is_dirty is False
	with pytest.raises(AttributeError):
		snapshot.cdml = "<cdml/>"


def test_malformed_cdml_maps_to_the_public_load_error() -> None:
	with pytest.raises(ferrum_chem.DocumentLoadError):
		ferrum_chem.DocumentSession.load("<cdml><molecule></cdml>")


def test_observation_and_stale_revision_conflict_are_typed() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observed = session.observe(0)

	assert observed.snapshot.revision == 0
	changed = session.submit(0, set_atom("N"))
	assert changed.revision == 1

	with pytest.raises(ferrum_chem.RevisionConflictError) as caught:
		session.observe(0)

	assert caught.value.expected == 0
	assert caught.value.actual == 1


def test_noop_and_mutation_follow_the_revision_and_dirty_contract() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	baseline = session.snapshot()
	no_change = session.submit(0, set_atom("C"))

	assert no_change.revision == baseline.revision
	assert no_change.digest == baseline.digest
	assert no_change.is_dirty is False

	changed = session.submit(no_change.revision, set_atom("N"))
	assert changed.revision == 1
	assert changed.is_dirty is True
	assert 'name="N"' in changed.cdml


def test_undo_and_redo_create_monotonic_revisions() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(0, set_atom("N"))
	undone = session.undo(changed.revision)
	redone = session.redo(undone.revision)

	assert undone.revision == 2
	assert 'name="C"' in undone.cdml
	assert redone.revision == 3
	assert 'name="N"' in redone.cdml
	assert redone.is_dirty is True


def test_operation_validation_errors_are_specific_and_structured() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)

	with pytest.raises(ferrum_chem.InvalidAtomElementError):
		session.submit(0, set_atom("2"))
	with pytest.raises(ferrum_chem.UnknownDocumentObjectError) as caught:
		session.submit(
			0,
			ferrum_chem.DocumentOperationV1.set_atom_element("missing", "N"),
		)

	assert caught.value.object_id == "missing"
	assert session.snapshot().revision == 0


def test_prepared_atom_insertion_is_revision_bound_and_one_use() -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	prepared = session.prepare_create_atom(0, "m", "created", "O")

	assert prepared.identifier == "created"
	committed = session.commit_create_atom(0, prepared)
	assert committed.revision == 1
	assert 'id="created"' in committed.cdml

	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		session.commit_create_atom(1, prepared)


def test_confirmed_save_or_unconfirmed_outcome_preserves_exact_contract(
	tmp_path: Path,
) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(0, set_atom("N"))
	published = session.save_atomic(tmp_path / "saved.cdml", changed.revision)

	assert (tmp_path / "saved.cdml").read_text() == published.published_snapshot.cdml
	assert published.published_snapshot.revision == changed.revision
	assert published.snapshot.revision == changed.revision
	if published.outcome == "confirmed":
		assert published.snapshot.is_dirty is False
	else:
		assert published.outcome == "directory_entry_unconfirmed"
		assert published.snapshot.is_dirty is True


def test_recovery_export_never_changes_the_session_state(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	changed = session.submit(0, set_atom("N"))
	exported = session.recovery_export(tmp_path / "recovery.cdml", changed.revision)
	current = session.snapshot()

	assert exported.snapshot.revision == changed.revision
	assert exported.snapshot.is_dirty is True
	assert current.revision == changed.revision
	assert current.digest == changed.digest
	assert current.is_dirty is True


def test_invalid_destination_keeps_its_structured_public_fields(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)

	with pytest.raises(ferrum_chem.InvalidDestinationError) as caught:
		session.save_atomic(tmp_path, 0)

	assert caught.value.path == str(tmp_path)
	assert caught.value.reason == "destination exists but is not a regular file"


def test_public_module_is_the_compiled_extension() -> None:
	assert ferrum_chem.__name__ == "ferrum_chem"
	assert not hasattr(ferrum_chem, "_bindings")


def test_publication_errors_share_the_documented_shape(tmp_path: Path) -> None:
	session = ferrum_chem.DocumentSession.load(SOURCE)
	target = tmp_path / "missing-parent" / "saved.cdml"

	with pytest.raises(ferrum_chem.PublicationNotStartedError) as caught:
		session.save_atomic(target, 0)

	assert isinstance(caught.value, ferrum_chem.PublicationError)
	assert isinstance(caught.value, ferrum_chem.FerrumError)
	assert caught.value.path == str(target)
	assert caught.value.reason
	assert issubclass(
		ferrum_chem.PublicationPossiblyCompletedError,
		ferrum_chem.PublicationError,
	)
