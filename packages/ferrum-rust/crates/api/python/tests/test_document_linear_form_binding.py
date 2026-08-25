"""Installed-binding checks for native linear-form conversion."""

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
	"""Return one observation with its durable molecule and atom IDs."""
	observation = session.observe(session.snapshot().revision)
	molecule = observation.projection.molecules[root]
	atom_ids = tuple(atom.id for atom in molecule.atoms)
	return observation, molecule.id, atom_ids


def _facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return public session facts used to prove atomic refusal."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


def test_linear_form_converts_observed_durable_atoms() -> None:
	"""The installed binding converts selected durable atom targets."""
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


def test_linear_form_history_reopens_the_changed_document() -> None:
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


def test_linear_form_canonical_repeat_is_history_free() -> None:
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


def test_linear_form_refuses_a_durable_bond_as_an_atom_target() -> None:
	"""The binding rejects a durable target of the wrong document-object kind."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, _atom_ids = _address(session)
	bond_id = observation.projection.molecules[0].bonds[0].id

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.convert_linear_form_v1(
			0, observation.snapshot.digest, molecule_id, (bond_id,),
		)

def test_linear_form_refuses_a_stale_revision_without_mutation() -> None:
	"""A stale revision fence leaves the authoritative snapshot unchanged."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, atom_ids = _address(session)
	before = session.snapshot()

	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.convert_linear_form_v1(1, observation.snapshot.digest, molecule_id, atom_ids)

	assert _facts(session.snapshot()) == _facts(before)


def test_linear_form_refuses_a_stale_digest_without_mutation() -> None:
	"""A stale digest fence leaves the authoritative snapshot unchanged."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, atom_ids = _address(session)
	before = session.snapshot()

	with pytest.raises(ferrum_chem.DocumentLinearFormError):
		session.convert_linear_form_v1(0, "0" * 64, molecule_id, atom_ids)

	assert _facts(session.snapshot()) == _facts(before)


def test_linear_form_refuses_a_durable_atom_as_the_molecule_target() -> None:
	"""The binding rejects a durable target of the wrong molecule kind."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, _molecule_id, atom_ids = _address(session)

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.convert_linear_form_v1(
			0, observation.snapshot.digest, atom_ids[0], atom_ids,
		)


def test_linear_form_refuses_an_atom_from_another_molecule() -> None:
	"""The binding rejects a durable atom target outside the molecule owner."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, _atom_ids = _address(session)
	_foreign_observation, _foreign_root, foreign_atom_ids = _address(session, 1)

	with pytest.raises(ferrum_chem.OperationValidationError):
		session.convert_linear_form_v1(
			0, observation.snapshot.digest, molecule_id, foreign_atom_ids,
		)


def test_linear_form_refuses_an_empty_selection() -> None:
	"""The binding rejects a request without selected durable atom targets."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, _atom_ids = _address(session)

	with pytest.raises(ferrum_chem.DocumentLinearFormError):
		session.convert_linear_form_v1(0, observation.snapshot.digest, molecule_id, ())


@pytest.mark.parametrize(
	("atoms", "error_type"),
	(
		([], ValueError),
		(("late", 1), ferrum_chem.InvalidDocumentObjectIdError),
		(("\ud800",), ValueError),
	),
)
def test_linear_form_rejects_invalid_python_input(
	atoms: object,
	error_type: type[ValueError] | type[ferrum_chem.InvalidDocumentObjectIdError],
) -> None:
	"""Only exact tuples of encodable built-in strings cross this boundary."""
	session = ferrum_chem.DocumentSession.load(_SOURCE)
	observation, molecule_id, _atom_ids = _address(session)

	with pytest.raises(error_type):
		session.convert_linear_form_v1(0, observation.snapshot.digest, molecule_id, atoms)
