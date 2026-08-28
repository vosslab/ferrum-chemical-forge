"""Prove installed PyO3 SMARTS receipts remain local, finite, and one-use."""

# Standard Library
import math

# local repo modules
import ferrum_chem


SOURCE = (
	"<cdml xmlns='urn:ferrum:cdml'><molecule id='m'><atom id='a' name='C'>"
	"<point x='10' y='20'/></atom></molecule></cdml>"
)


#============================================
class InstalledSmartsReceiptE2eError(RuntimeError):
	"""Report an installed SMARTS receipt contract failure."""


#============================================
def _require(condition: bool, message: str) -> None:
	"""Raise one explicit E2E failure when a durable contract is absent."""
	if not condition:
		raise InstalledSmartsReceiptE2eError(message)


#============================================
def _publish_plan(session: object) -> None:
	"""Publish the sole current renderer plan accepted by the live query seam."""
	snapshot = session.snapshot()
	session.observe_presentation_render_plan_v1(snapshot.revision, snapshot.digest)


#============================================
def _raw_result(session: object) -> object:
	"""Issue one raw SMARTS result through the installed native engine."""
	_publish_plan(session)
	return session._run_live_document_smarts_query_v1("[C]", 32, 32)


#============================================
def _selected_token(session: object) -> object:
	"""Mint the sole selected-root token from the public render selection seam."""
	snapshot = session.snapshot()
	interaction = session.observe_render_interaction_v1(snapshot.revision, snapshot.digest)
	root = interaction.roots[0]
	selection = session.select_render_interaction_roots_v1(
		interaction,
		None,
		ferrum_chem.RenderInteractionQueryV1.root(root.document_object_id),
	)
	return session._capture_live_document_smarts_selected_query_v1(selection)


#============================================
def _expect_live_failure(action: object, message: str) -> object:
	"""Require the typed PyO3 SMARTS error rather than a Python-side fallback."""
	try:
		action()
	except ferrum_chem.LiveDocumentSmartsError as error:
		return error
	raise InstalledSmartsReceiptE2eError(message)


#============================================
def _move_atom(session: object) -> None:
	"""Advance the authoritative document fence without replacing its owner."""
	snapshot = session.snapshot()
	session.apply_document_operation_v1(
		snapshot.revision,
		ferrum_chem.DocumentOperationV1.set_atom_position("a", 12.0, 20.0, 0.0),
	)


#============================================
def _test_raw_receipt_is_opaque_finite_and_one_use() -> None:
	"""A raw receipt yields finite geometry once and never exposes identity."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	result = _raw_result(session)
	_require(result.molecules[0].match_count == 1, "raw SMARTS did not find the installed carbon")
	paint = session._show_live_document_smarts_match_v1(result.receipt, 0)
	_require(
		all(math.isfinite(value) for bounds in paint.atom_bounds for value in bounds),
		"installed SMARTS paint included a non-finite bound",
	)
	_require(
		not hasattr(paint, "document_object_id") and not hasattr(paint, "molecule_id"),
		"installed SMARTS paint exposed document identity",
	)
	_expect_live_failure(
		lambda: session._show_live_document_smarts_match_v1(result.receipt, 0),
		"installed SMARTS receipt replay unexpectedly succeeded",
	)


#============================================
def _test_selected_receipt_is_opaque_finite_and_one_use() -> None:
	"""An owner-selected SMARTS request has the same receipt-only presentation seam."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	_publish_plan(session)
	selected = _selected_token(session)
	result = session._run_live_document_smarts_query_v1(selected, 32, 32)
	_require(
		result.molecules[0].match_count == 1,
		"selected SMARTS did not find the installed owner molecule",
	)
	paint = session._show_live_document_smarts_match_v1(result.receipt, 0)
	_require(
		all(math.isfinite(value) for bounds in paint.atom_bounds for value in bounds),
		"selected installed SMARTS paint included a non-finite bound",
	)
	_require(
		not hasattr(paint, "document_object_id") and not hasattr(paint, "molecule_id"),
		"selected installed SMARTS paint exposed document identity",
	)
	_expect_live_failure(
		lambda: session._show_live_document_smarts_match_v1(result.receipt, 0),
		"selected installed SMARTS receipt replay unexpectedly succeeded",
	)


#============================================
def _test_public_module_hides_private_smarts_capabilities() -> None:
	"""PyO3 keeps receipt plumbing private while retaining public closed error facts."""
	private_names = (
		"_LiveDocumentSmartsReceiptV1",
		"_LiveDocumentSmartsSelectedQueryV1",
		"_LiveDocumentSmartsSelectedReadinessV1",
		"_LiveDocumentSmartsMoleculeSummaryV1",
		"_LiveDocumentSmartsRunSummaryV1",
		"_LiveDocumentSmartsPaintV1",
		"_LiveDocumentSmartsReceipt",
		"_LiveDocumentSmartsSelectedQuery",
		"_LiveDocumentSmartsSelectedReadiness",
		"_LiveDocumentSmartsMoleculeSummary",
		"_LiveDocumentSmartsRunSummary",
		"_LiveDocumentSmartsPaint",
	)
	_require(
		not any(hasattr(ferrum_chem, name) for name in private_names),
		"installed ferrum_chem exposed private SMARTS capability plumbing",
	)
	_require(
		hasattr(ferrum_chem, "LiveDocumentSmartsError")
		and hasattr(ferrum_chem, "LiveDocumentSmartsCategoryV1")
		and hasattr(ferrum_chem, "LiveDocumentSmartsReasonV1")
		and hasattr(ferrum_chem, "LiveDocumentSmartsRecoveryV1"),
		"installed ferrum_chem omitted a public SMARTS error contract",
	)


#============================================
def _test_foreign_and_cleared_capabilities_are_refused() -> None:
	"""Receipts and selected tokens remain session-local and clearable."""
	owner = ferrum_chem.DocumentSession.load(SOURCE)
	foreign = ferrum_chem.DocumentSession.load(SOURCE)
	result = _raw_result(owner)
	_expect_live_failure(
		lambda: foreign._show_live_document_smarts_match_v1(result.receipt, 0),
		"foreign session redeemed an installed SMARTS receipt",
	)
	selected = _selected_token(owner)
	_publish_plan(foreign)
	_expect_live_failure(
		lambda: foreign._run_live_document_smarts_query_v1(selected, 32, 32),
		"foreign session used an installed selected SMARTS token",
	)
	owner._clear_live_document_smarts_receipts_v1()
	_expect_live_failure(
		lambda: owner._show_live_document_smarts_match_v1(result.receipt, 0),
		"cleared installed SMARTS receipt remained redeemable",
	)


#============================================
def _test_document_mutation_invalidates_receipts_and_selected_tokens() -> None:
	"""Current-document mutation refuses both prior kinds of installed capability."""
	session = ferrum_chem.DocumentSession.load(SOURCE)
	result = _raw_result(session)
	selected = _selected_token(session)
	_move_atom(session)
	_expect_live_failure(
		lambda: session._show_live_document_smarts_match_v1(result.receipt, 0),
		"stale installed SMARTS receipt remained redeemable",
	)
	_publish_plan(session)
	_expect_live_failure(
		lambda: session._run_live_document_smarts_query_v1(selected, 32, 32),
		"stale installed selected SMARTS token remained usable",
	)


#============================================
def main() -> int:
	"""Exercise installed raw and selected SMARTS capability ownership."""
	_test_public_module_hides_private_smarts_capabilities()
	_test_raw_receipt_is_opaque_finite_and_one_use()
	_test_selected_receipt_is_opaque_finite_and_one_use()
	_test_foreign_and_cleared_capabilities_are_refused()
	_test_document_mutation_invalidates_receipts_and_selected_tokens()
	print("installed PyO3 SMARTS receipt E2E passed")
	return 0


if __name__ == "__main__":
	main()
