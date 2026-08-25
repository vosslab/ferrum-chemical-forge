"""Receipt authentication coverage for the native Molecule Report client."""

# local repo modules
import ferrum_qt.ferrum.molecule_inspection
import ferrum_qt.ferrum.molecule_report


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
def _record(molecule_id: str, source_id: str, document_root_order: int) -> dict:
	"""Build the smallest valid public record with Rust-owned report facts."""
	return {
		"molecule_id": molecule_id,
		"source_id": source_id,
		"document_root_order": document_root_order,
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
	report = _authenticated_report(
		_intent("molecule-a"),
		_response([_record("molecule-a", "rust-source-a", 3)]),
	)
	assert report is not None
	assert report["records"][0]["molecule_id"] == "molecule-a"


#============================================
def test_molecule_report_matches_durable_ids_without_changing_rust_order() -> None:
	"""Response records authenticate by durable ID while preserving Rust report order."""
	report = _authenticated_report(
		_intent("molecule-first", "molecule-second"),
		_response([
			_record("molecule-second", "rust-source-second", 9),
			_record("molecule-first", "rust-source-first", 2),
		]),
	)
	assert report is not None
	assert [record["molecule_id"] for record in report["records"]] == [
		"molecule-second", "molecule-first",
	]
