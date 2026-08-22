"""Installed-extension checks for private native linear-form conversion."""

import pytest

import ferrum_chem


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="1.0"><molecule id="m">
 <atom id="late" name="C"><point x="40" y="5"/></atom>
 <atom id="early" name="O"><point x="10" y="5"/></atom>
 <bond id="b" start="late" end="early" type="n1"/>
</molecule><molecule id="other"><atom id="foreign" name="N">
 <point x="0" y="0"/></atom></molecule></cdml>
"""


def _address(session: object, root: int = 0) -> tuple[object, str, tuple[str, ...]]:
	"""Return one installed observation and its direct-root atom selectors."""
	observation = session.observe(session.snapshot().revision)
	molecule = observation.projection.molecules[root]
	atom_ids = (("late", "early"), ("foreign",))[root]
	return observation, molecule.id, atom_ids


def _facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return public session facts used to prove atomic refusal."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


def test_private_linear_form_converts_source_order() -> None:
	"""The installed binding returns the authoritative changed receipt."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, atom_ids = _address(session)
	changed = session.convert_linear_form_v1(
		0, observation.snapshot.digest, molecule_id, atom_ids,
	)
	changed_cdml = changed.observation.snapshot.cdml

	assert changed.observation.snapshot.revision == 1
	assert all(
		marker in changed_cdml
		for marker in (
			'id="late" name="C" show_hydrogens="on"',
			'id="early" name="O" show_hydrogens="on"',
			'type="linear_form"',
		)
	)


def test_private_linear_form_history_reopens_the_changed_document() -> None:
	"""Undo, redo, and reopen retain the binding's authoritative document."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	before_cdml = session.snapshot().cdml
	observation, molecule_id, atom_ids = _address(session)
	changed = session.convert_linear_form_v1(
		0, observation.snapshot.digest, molecule_id, atom_ids,
	)
	undone = session.undo(1)
	redone = session.redo(2)
	reopened = ferrum_chem.DocumentSession.load(redone.observation.snapshot.cdml)

	assert undone.observation.snapshot.cdml == before_cdml
	assert reopened.snapshot().cdml == changed.observation.snapshot.cdml


def test_private_linear_form_canonical_repeat_is_history_free() -> None:
	"""A canonical second request returns the current authoritative revision."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, atom_ids = _address(session)
	changed = session.convert_linear_form_v1(
		0, observation.snapshot.digest, molecule_id, atom_ids,
	)
	repeated = session.convert_linear_form_v1(
		1, changed.observation.snapshot.digest, molecule_id, atom_ids,
	)

	assert repeated.observation.snapshot.revision == changed.observation.snapshot.revision
	assert repeated.observation.snapshot.digest == changed.observation.snapshot.digest


def test_private_linear_form_binding_accepts_atom_selectors_not_bond_selectors() -> None:
	"""Qt expands selected bonds before this binding receives its atom-only tuple."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, _atom_ids = _address(session)
	bond_id = "b"

	with pytest.raises(ferrum_chem.DocumentLinearFormError) as caught:
		session.convert_linear_form_v1(
			0, observation.snapshot.digest, molecule_id, (bond_id,),
		)

	assert "not one direct atom" in caught.value.reason


def test_private_linear_form_refuses_unauthenticated_and_invalid_selection() -> None:
	"""Authentication and selection failures leave the authoritative snapshot unchanged."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, atom_ids = _address(session)
	foreign_observation, _foreign_root, foreign_atom_ids = _address(session, 1)
	before = session.snapshot()
	invalid_calls = (
		(1, observation.snapshot.digest, molecule_id, atom_ids, ferrum_chem.RevisionConflictError),
		(0, "0" * 64, molecule_id, atom_ids, ferrum_chem.DocumentLinearFormError),
		(0, observation.snapshot.digest, atom_ids[0], atom_ids, ferrum_chem.DocumentLinearFormError),
		(0, observation.snapshot.digest, molecule_id, (), ferrum_chem.DocumentLinearFormError),
		(0, observation.snapshot.digest, molecule_id, (atom_ids[0], atom_ids[0]), ferrum_chem.DocumentLinearFormError),
		(0, observation.snapshot.digest, molecule_id, foreign_atom_ids, ferrum_chem.DocumentLinearFormError),
	)
	assert foreign_observation.snapshot.digest == observation.snapshot.digest
	for revision, digest, root, atoms, error_type in invalid_calls:
		with pytest.raises(error_type):
			session.convert_linear_form_v1(revision, digest, root, atoms)

	assert _facts(session.snapshot()) == _facts(before)


@pytest.mark.parametrize("atoms", ([], ("late", 1), ("\ud800",)))
def test_private_linear_form_rejects_wrong_container_and_python_text(atoms: object) -> None:
	"""Only exact tuples of encodable built-in strings cross this boundary."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, _atom_ids = _address(session)

	with pytest.raises(ferrum_chem.DocumentLinearFormError) as caught:
		session.convert_linear_form_v1(0, observation.snapshot.digest, molecule_id, atoms)

	assert caught.value.reason

