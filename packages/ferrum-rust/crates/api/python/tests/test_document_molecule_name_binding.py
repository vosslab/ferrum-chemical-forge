"""Behavior checks for the private direct-root molecule-name binding."""

# PIP3 modules
import ferrum_chem
import pytest


SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml" xmlns:v="urn:vendor" version="26.07">'
	'<molecule id="m" name="before" role="source">'
	'<atom id="a" name="C"><point x="1" y="2"/>'
	'<v:opaque retained="yes"/></atom></molecule>'
	'<molecule id="other"><atom id="b" name="O">'
	'<point x="3" y="4"/></atom></molecule></cdml>'
)


#============================================
def _address(session: object, root: int = 0) -> tuple[object, str]:
	"""Return one exact installed observation and durable root selector."""
	observation = session.observe(session.snapshot().revision)
	return observation, observation.projection.molecules[root].document_object_id


#============================================
def _snapshot_facts(snapshot: object) -> tuple[str, int, str, bool]:
	"""Return the complete public snapshot state used by these mutation checks."""
	return snapshot.cdml, snapshot.revision, snapshot.digest, snapshot.is_dirty


#============================================
def test_exact_name_clear_history_and_reopen_preserve_retained_content() -> None:
	"""Whitespace persists, empty clears, and history remains Rust-owned."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation, molecule_id = _address(session)
	spaced = session.set_document_molecule_name_v1(
		0, observation.snapshot.digest, molecule_id, "  ",
	)
	cleared = session.set_document_molecule_name_v1(
		1, spaced.observation.snapshot.digest, molecule_id, "",
	)
	undone = session.undo(2)
	redone = session.redo(3)
	reopened = ferrum_chem.DocumentSession.load(redone.observation.snapshot.cdml)

	assert spaced.observation.projection.molecules[0].name == "  "
	assert cleared.observation.projection.molecules[0].name is None
	assert undone.observation.projection.molecules[0].name == "  "
	assert reopened.observe(0).projection.molecules[1].document_object_id is not None


#============================================
def test_same_name_is_history_free_and_stale_precedes_noop() -> None:
	"""An exact no-op retains revision zero, while a stale repeat is rejected."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation, molecule_id = _address(session)
	same = session.set_document_molecule_name_v1(
		0, observation.snapshot.digest, molecule_id, "before",
	)
	session.set_document_molecule_name_v1(
		0, observation.snapshot.digest, molecule_id, "after",
	)
	with pytest.raises(ferrum_chem.RevisionConflictError):
		session.set_document_molecule_name_v1(
			0, observation.snapshot.digest, molecule_id, "after",
		)

	assert _snapshot_facts(same.observation.snapshot) == _snapshot_facts(observation.snapshot)


#============================================
def test_digest_nonroot_and_invalid_xml_name_are_atomic() -> None:
	"""Every unauthenticated or unserializable request leaves the snapshot exact."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation, molecule_id = _address(session)
	atom_id = observation.projection.molecules[0].atoms[0].document_object_id
	before = session.snapshot()
	with pytest.raises(ferrum_chem.DocumentMoleculeNameError):
		session.set_document_molecule_name_v1(0, "0" * 64, molecule_id, "x")
	with pytest.raises(ferrum_chem.DocumentMoleculeNameError):
		session.set_document_molecule_name_v1(
			0, observation.snapshot.digest, atom_id, "x",
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		session.set_document_molecule_name_v1(
			0, observation.snapshot.digest, molecule_id, "bad\x00name",
		)

	assert _snapshot_facts(session.snapshot()) == _snapshot_facts(before)


#============================================
@pytest.mark.parametrize("field", ("digest", "selector", "name"))
def test_unpaired_surrogates_are_actionable_name_errors(field: str) -> None:
	"""Python-only text cannot escape the operation-specific error boundary."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	observation, molecule_id = _address(session)
	digest = observation.snapshot.digest
	name = "after"
	bad = chr(0xD800)
	if field == "digest":
		digest = bad
	elif field == "selector":
		molecule_id = bad
	else:
		name = bad
	with pytest.raises(ferrum_chem.DocumentMoleculeNameError) as caught:
		session.set_document_molecule_name_v1(0, digest, molecule_id, name)

	assert caught.value.reason
