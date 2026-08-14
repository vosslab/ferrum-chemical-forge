"""Behavior coverage for authoritative interactive InChI and peptide insertion."""

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.actions.chemistry_actions
import ferrum_qt.bridge.worker
import oasa.cdml


#============================================
def _formula_from_proposal(proposal_cdml: str) -> str:
	"""Return the formula carried by one validated insertion proposal."""
	molecule = next(iter(oasa.cdml.read_cdml(proposal_cdml)))
	return str(molecule.get_formula_dict())


#============================================
@pytest.mark.parametrize(
	("codec_name", "source_text", "label", "expected_formula"),
	[
		("inchi", "InChI=1S/CH4/h1H4", "Imported InChI molecule", "CH4"),
		("peptide", "AN", "Imported peptide sequence 'AN'", "C7H13N3O4"),
	],
)
def test_text_insertion_preparation_returns_a_plain_backend_proposal(
		codec_name: str, source_text: str, label: str, expected_formula: str,
		) -> None:
	"""Both supported text codecs produce the same immutable insertion shape."""
	prepared = ferrum_qt.bridge.worker._prepare_text_molecule_insertion(
		codec_name, source_text, 4, "text-import", 35.0, (120.0, 240.0), label,
	)

	assert prepared.label == label
	assert _formula_from_proposal(prepared.proposal_cdml) == expected_formula


#============================================
def test_peptide_text_validation_carries_its_dialog_stage() -> None:
	"""Invalid peptide input keeps the parser-specific UI failure vocabulary."""
	with pytest.raises(ferrum_qt.bridge.worker.TextImportPreparationError) as error:
		ferrum_qt.bridge.worker._prepare_text_molecule_insertion(
			"peptide", "A?", 0, "invalid", 35.0, (0.0, 0.0), "Import peptide",
		)

	assert error.value.stage == "peptide-validation"


#============================================
@pytest.mark.parametrize(
	("codec_name", "source_text", "label"),
	[
		("inchi", "InChI=1S/CH4/h1H4", "Imported InChI molecule"),
		("peptide", "AN", "Imported peptide sequence 'AN'"),
	],
)
def test_text_insertion_commits_through_backend_history_only(
		main_window: object, codec_name: str, source_text: str, label: str,
		) -> None:
	"""Accepted text insertion changes backend state and remains backend-undoable."""
	target = main_window._active_session
	request_token = target.begin_import_request()
	prepared = ferrum_qt.bridge.worker._prepare_text_molecule_insertion(
		codec_name, source_text, target.backend_snapshot.revision,
		"accepted-%s" % codec_name,
		35.0, (120.0, 240.0), label,
	)
	delivery = ferrum_qt.actions.chemistry_actions.MoleculeInsertionDelivery(
		main_window, target, request_token, prepared.expected_revision,
		codec_name, label, lambda _app, _message: None,
	)
	accepted = delivery.deliver(prepared)
	qt_undo_available = target.document.undo_stack.canUndo()
	undone = target.undo_backend().status
	redone = target.redo_backend().status
	# Return the fixture session to its clean backend baseline.
	target.undo_backend()

	assert accepted.status == "accepted", accepted.message
	assert not qt_undo_available
	assert (undone, redone) == ("accepted", "accepted")


#============================================
def test_stale_text_insertion_leaves_the_backend_snapshot_unchanged(
		main_window: object,
		) -> None:
	"""A stale text proposal is discarded before an authoritative commit begins."""
	target = main_window._active_session
	prepared = ferrum_qt.bridge.worker._prepare_text_molecule_insertion(
		"inchi", "InChI=1S/CH4/h1H4", target.backend_snapshot.revision,
		"stale", 35.0, (120.0, 240.0), "Imported InChI molecule",
	)
	request_token = target.begin_import_request()
	delivery = ferrum_qt.actions.chemistry_actions.MoleculeInsertionDelivery(
		main_window, target, request_token, prepared.expected_revision, "InChI",
		prepared.label, lambda _app, _message: None,
	)
	target.invalidate_import_requests()
	before = target.backend_snapshot
	outcome = delivery.deliver(prepared)

	assert outcome.status == "discarded"
	assert target.backend_snapshot == before


#============================================
@pytest.mark.parametrize(
	("codec_name", "source_text", "label"),
	[
		("inchi", "InChI=1S/CH4/h1H4", "Imported InChI molecule"),
		("peptide", "AN", "Imported peptide sequence 'AN'"),
	],
)
def test_mismatched_text_revision_is_rejected_before_backend_mutation(
		main_window: object, codec_name: str, source_text: str, label: str,
		) -> None:
	"""A delayed text proposal is never rebased onto a later session revision."""
	target = main_window._active_session
	prepared = ferrum_qt.bridge.worker._prepare_text_molecule_insertion(
		codec_name, source_text, target.backend_snapshot.revision,
		"mismatched-%s" % codec_name, 35.0, (120.0, 240.0), label,
	)
	delivery = ferrum_qt.actions.chemistry_actions.MoleculeInsertionDelivery(
		main_window, target, target.begin_import_request(),
		prepared.expected_revision + 1, codec_name, label, lambda _app, _message: None,
	)
	before = target.backend_snapshot
	outcome = delivery.deliver(prepared)

	assert outcome.status == "rejected"
	assert target.backend_snapshot == before
