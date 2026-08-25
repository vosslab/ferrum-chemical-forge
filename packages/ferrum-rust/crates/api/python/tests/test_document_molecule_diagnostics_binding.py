"""Installed-extension behavior for private fenced molecule diagnostics."""

import pytest

import ferrum_chem


_DIAGNOSTIC_CDML = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="c" name="C"><point x="0" y="0"/></atom>'
	'<atom id="o" name="O"><point x="1" y="0"/></atom>'
	'<bond id="zero" start="c" end="o" type="n0"/>'
	'</molecule></cdml>'
)


def _address(source: str = _DIAGNOSTIC_CDML) -> tuple[str, int, str, str]:
	"""Return owned snapshot facts and one durable root ID."""
	session = ferrum_chem.DocumentSession.load(source)
	snapshot = session.snapshot()
	molecule_id = session.observe(snapshot.revision).projection.molecules[0].id
	return source, snapshot.revision, snapshot.digest, molecule_id


def test_owned_diagnostics_returns_a_reachable_closed_finding() -> None:
	"""One supported diagnostic category crosses the owned frozen boundary."""
	cdml, source_revision, source_digest, molecule_id = _address()
	receipt = ferrum_chem._document_molecule_diagnostics_from_snapshot_v1(
		cdml, source_revision, source_digest, (molecule_id,),
	)
	findings = {
		finding.code: finding
		for record in receipt.records
		for finding in record.findings
	}
	zero = findings["zero_order_bond"]

	assert receipt.source_revision == source_revision and receipt.source_digest == source_digest
	assert zero.severity == "warning" and zero.recovery == "correct_chemical_facts"
	assert zero.location.kind == "bond" and zero.location.identifier == "zero"


def test_owned_diagnostics_refuses_an_invalid_digest() -> None:
	"""The detached boundary rejects an owned digest that does not match CDML."""
	cdml, source_revision, _source_digest, molecule_id = _address()
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem._document_molecule_diagnostics_from_snapshot_v1(
			cdml, source_revision, "0" * 64, (molecule_id,),
		)


def test_owned_diagnostics_refuses_nonroot_and_missing_durable_ids() -> None:
	"""Only current direct molecule roots can enter the frozen diagnostics route."""
	cdml, source_revision, source_digest, molecule_id = _address()
	session = ferrum_chem.DocumentSession.load(cdml)
	atom_id = session.observe(source_revision).projection.molecules[0].atoms[0].id
	_other_cdml, _other_revision, _other_digest, missing_id = _address(
		_DIAGNOSTIC_CDML.replace('molecule id="m"', 'molecule id="other"'),
	)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem._document_molecule_diagnostics_from_snapshot_v1(
			cdml, source_revision, source_digest, (atom_id,),
		)
	with pytest.raises(ferrum_chem.OperationValidationError):
		ferrum_chem._document_molecule_diagnostics_from_snapshot_v1(
			cdml, source_revision, source_digest, (missing_id,),
		)


def test_owned_diagnostics_returns_immutable_owned_facts() -> None:
	"""Receipt and nested records are immutable copies, not session-owned aliases."""
	cdml, source_revision, source_digest, molecule_id = _address()
	receipt = ferrum_chem._document_molecule_diagnostics_from_snapshot_v1(
		cdml, source_revision, source_digest, (molecule_id,),
	)
	record = next(record for record in receipt.records if record.molecule_id == molecule_id)
	finding = next(finding for finding in record.findings if finding.code == "zero_order_bond")
	with pytest.raises(AttributeError):
		receipt.source_digest = "changed"
	with pytest.raises(AttributeError):
		record.molecule_id = "changed"
	with pytest.raises(AttributeError):
		finding.location.kind = "changed"

	assert isinstance(receipt.records, tuple) and isinstance(record.findings, tuple)
