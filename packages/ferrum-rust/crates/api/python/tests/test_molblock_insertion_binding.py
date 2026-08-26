"""Installed-extension behavior for bounded native molfile insertion."""

# Standard Library
import json
import pathlib

# PIP3 modules
import pytest
import ferrum_chem


#============================================
def _placement() -> object:
	"""Return one exact finite insertion placement."""
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 100.0, 200.0)
	return placement


#============================================
def _ethanol_molblock() -> str:
	"""Produce one valid coordinate-bearing molblock through the packaged adapter."""
	molecule = ferrum_chem.parse_smiles("CCO")
	molblock = ferrum_chem.molecule_to_molblock(
		molecule,
		ferrum_chem.MolblockVersionV1.v2000,
	)
	return molblock


#============================================
def test_bounded_molfile_prepares_and_commits_one_owned_molecule(
		tmp_path: pathlib.Path) -> None:
	"""The file path crosses Rust once and commits only through the session."""
	path = tmp_path / "ethanol.mol"
	path.write_text(_ethanol_molblock(), encoding="utf-8")
	prepared = ferrum_chem.prepare_molblock_file_v1(str(path), _placement())
	session = ferrum_chem.DocumentSession.load('<cdml xmlns="urn:ferrum:cdml" version="1.0"/>')
	operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(prepared)
	pending = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(0))
	accepted = session.commit_session_operation_transition_v1(pending)
	molecule = accepted.observation.projection.molecules[0]

	assert tuple(atom.element for atom in molecule.atoms) == ("C", "C", "O")
	assert tuple(bond.source_type for bond in molecule.bonds) == ("n1", "n1")


#============================================
def test_native_molblock_stereo_reaches_the_durable_molecule_report() -> None:
	"""A native molblock coordinator retains tetrahedral source semantics."""
	molblock = ferrum_chem.molecule_to_molblock(
		ferrum_chem.parse_smiles("F[C@](Cl)(Br)I"),
		ferrum_chem.MolblockVersionV1.v2000,
	)
	molecule = ferrum_chem.prepare_molblock_molecule_v1(molblock, _placement())
	session = ferrum_chem.DocumentSession.load('<cdml xmlns="urn:ferrum:cdml"/>')
	operation = ferrum_chem.DocumentOperationV1.insert_molecule_v1(molecule)
	prepared = session.prepare_session_operation_transition_v1(
		operation.transition_request_v1(0))
	committed = session.commit_session_operation_transition_v1(prepared)
	snapshot = committed.observation.snapshot
	molecule_id = committed.observation.projection.molecules[0].document_object_id
	response = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "native-molblock-stereo",
		"operation": {
			"kind": "document.molecule.report.v1",
			"snapshot": {
				"cdml": snapshot.cdml,
				"revision": snapshot.revision,
				"digest_hex": snapshot.digest,
			},
			"molecule_ids": [molecule_id],
		},
	})))

	semantics = response["outcome"]["report"]["records"][0]["stereo_semantics"]
	assert semantics["tetrahedral"] == [{
		"center": 1,
		"ligands": [
			{"kind": "atom", "index": 0},
			{"kind": "atom", "index": 2},
			{"kind": "atom", "index": 3},
			{"kind": "atom", "index": 4},
		],
		"parity": "counter_clockwise",
	}]
	assert semantics["double_bonds"] == []


#============================================
def test_molfile_source_failures_are_typed_before_document_mutation(
		tmp_path: pathlib.Path) -> None:
	"""Non-UTF-8 input never reaches native parsing or a document session."""
	path = tmp_path / "not-utf8.mol"
	path.write_bytes(b"\xff")

	with pytest.raises(ferrum_chem.MolblockInputError) as caught:
		ferrum_chem.prepare_molblock_file_v1(str(path), _placement())

	assert caught.value.stage == "utf8"
	assert caught.value.path == str(path)
	assert caught.value.reason == f"molblock file is not UTF-8: {path}"


#============================================
def test_molfile_path_requires_an_exact_string(tmp_path: pathlib.Path) -> None:
	"""Path-like objects cannot smuggle alternate conversion behavior."""
	path = tmp_path / "ethanol.mol"
	path.write_text(_ethanol_molblock(), encoding="utf-8")

	with pytest.raises(TypeError):
		ferrum_chem.prepare_molblock_file_v1(path, _placement())


#============================================
def test_prepared_molfile_cannot_commit_to_a_newer_revision(tmp_path: pathlib.Path) -> None:
	"""A worker result cannot retarget a document changed after preparation."""
	path = tmp_path / "ethanol.mol"
	path.write_text(_ethanol_molblock(), encoding="utf-8")
	prepared = ferrum_chem.prepare_molblock_file_v1(str(path), _placement())
	session = ferrum_chem.DocumentSession.load('<cdml xmlns="urn:ferrum:cdml" version="1.0"/>')
	pending = session.prepare_session_operation_transition_v1(
		ferrum_chem.DocumentOperationV1.insert_molecule_v1(prepared)
		.transition_request_v1(0))
	other = ferrum_chem.prepare_smiles_molecule_v1("C", _placement())
	other_pending = session.prepare_session_operation_transition_v1(
		ferrum_chem.DocumentOperationV1.insert_molecule_v1(other)
		.transition_request_v1(0))
	session.commit_session_operation_transition_v1(other_pending)

	with pytest.raises(ferrum_chem.PreparedOperationStaleSnapshotError):
		session.commit_session_operation_transition_v1(pending)
