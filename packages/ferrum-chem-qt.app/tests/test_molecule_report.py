"""Receipt authentication coverage for the native Molecule Report client."""

# local repo modules
import ferrum_qt.ferrum.engine
import ferrum_qt.ferrum.molecule_inspection
import ferrum_qt.ferrum.molecule_report


#============================================
def _durable_molecule_ids() -> tuple[str, str]:
	"""Observe two Rust-issued durable IDs from one minimal local document."""
	session = ferrum_qt.ferrum.engine.extension_module().DocumentSession.load(
		"<cdml xmlns='urn:ferrum:cdml'><molecule id='first'>"
		"<atom id='first-atom' name='C'><point x='0' y='0'/></atom>"
		"</molecule><molecule id='second'>"
		"<atom id='second-atom' name='O'><point x='10' y='0'/></atom>"
		"</molecule></cdml>",
	)
	molecules = session.observe(0).projection.molecules
	return tuple(molecule.document_object_id for molecule in molecules)


#============================================
def _intent(*molecule_ids: str) -> object:
	"""Build one frozen report intent with only durable molecule identities."""
	addresses = tuple(
		ferrum_qt.ferrum.molecule_inspection.FerrumNativeMoleculeInspectionAddress(molecule_id)
		for molecule_id in molecule_ids
	)
	return ferrum_qt.ferrum.molecule_report._MoleculeReportIntent(
		object(), 7, "digest", addresses, object(),
	)


#============================================
def _record(molecule_id: str, document_paint_order: int) -> dict:
	"""Build the smallest valid public record with Rust-owned report facts."""
	return {
		"molecule_id": molecule_id,
		"document_paint_order": document_paint_order,
		"authored_name": None,
		"atom_count": 1,
		"bond_count": 0,
		"authored_charge": None,
		"authored_elements": [{"symbol": "C", "atom_count": 1}],
		"composition": None,
		"neutral_bond_capacity": "within_capacity",
		"stereo_semantics": None,
		"stereo_depiction": None,
		"findings": [],
	}


#============================================
def _response(records: list[dict]) -> dict:
	"""Build one valid public Molecule Report response envelope."""
	return {
		"schema": "ferrum-operation-response-v1",
		"request_id": "qt-molecule-report",
		"outcome": {
			"kind": "document.molecule.report.v1",
			"report": {
				"schema": "ferrum-document-molecule-report-v1",
				"source_revision": 7,
				"source_digest_hex": "digest",
				"records": records,
				"aggregate": {
					"kind": "omitted",
					"reason": "fewer_than_two_selected",
					"recovery": "none",
				},
			},
		},
	}


#============================================
def _authenticated_report(intent: object, response: object) -> dict | None:
	"""Use the callback authentication seam without constructing a Qt window."""
	return ferrum_qt.ferrum.molecule_report.FerrumNativeMoleculeReportMixin._report_from_current_intent(
		object(), intent, response,
	)


#============================================
def test_molecule_report_accepts_a_single_durable_molecule_id() -> None:
	"""A one-root callback accepts source facts not retained in the Qt address."""
	molecule_id, _unused_molecule_id = _durable_molecule_ids()
	report = _authenticated_report(
		_intent(molecule_id),
		_response([_record(molecule_id, 0)]),
	)
	assert report is not None
	assert report["records"][0]["molecule_id"] == molecule_id


#============================================
def test_molecule_report_matches_durable_ids_without_changing_rust_order() -> None:
	"""Response records authenticate by durable ID while preserving Rust report order."""
	first_molecule_id, second_molecule_id = _durable_molecule_ids()
	report = _authenticated_report(
		_intent(first_molecule_id, second_molecule_id),
		_response([
			_record(second_molecule_id, 0),
			_record(first_molecule_id, 1),
		]),
	)
	assert report is not None
	assert [record["molecule_id"] for record in report["records"]] == [
		second_molecule_id, first_molecule_id,
	]
