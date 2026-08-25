"""Behavior checks for plain-language refusal presentation."""

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.dialogs.refusal_presenter


#============================================
def test_uncertain_save_preserves_the_unknown_file_outcome() -> None:
	"""A potentially completed save must not be described as definitely failed."""
	request = ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
		ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
		ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_POSSIBLY_COMPLETED,
		"ethanol.cdml",
	)
	presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(request)

	assert "could not confirm" in presentation.ordinary_text().lower()
	assert "not saved" not in presentation.ordinary_text().lower()


#============================================
def test_save_not_started_preserves_the_definite_file_outcome() -> None:
	"""A save that never started must not be presented as an unknown result."""
	request = ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
		ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
		ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_NOT_STARTED,
		"ethanol.cdml",
	)
	presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(request)

	assert "did not start saving" in presentation.ordinary_text().lower()
	assert "could not confirm" not in presentation.ordinary_text().lower()


#============================================
def test_refusal_text_keeps_diagnostics_out_of_the_author_explanation() -> None:
	"""Technical diagnostics remain available without leaking into ordinary text."""
	request = ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
		ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
		ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.INVALID_DOCUMENT,
		"broken.cdml",
		"Rust parser rejected native admission",
	)
	presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(request)

	assert presentation.technical_details == "Rust parser rejected native admission"
	assert "Rust" not in presentation.ordinary_text()


#============================================
def test_unavailable_edit_uses_generic_or_supplied_primary_message() -> None:
	"""Unavailable edits retain generic copy unless a caller supplies a primary fact."""
	default_request = ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
		ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
		ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
	)
	default = ferrum_qt.dialogs.refusal_presenter.present_refusal(default_request)
	primary_message = (
		"Me cannot attach to the selected atom. Select another atom and try again."
	)
	custom_request = ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
		ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
		ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
		primary_message=primary_message,
	)
	custom = ferrum_qt.dialogs.refusal_presenter.present_refusal(custom_request)

	assert default.title == "Action Not Available"
	assert default.what_happened == "This action is not available for the current drawing."
	assert custom.title == "Action Not Available"
	assert custom.what_happened == primary_message
	assert primary_message in custom.ordinary_text()
	invalid_requests = (
		(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
				primary_message="",
			),
			ValueError,
			"nonempty",
		),
		(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
				primary_message=7,  # type: ignore[arg-type]
			),
			TypeError,
			"string or None",
		),
		(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.NO_UNDO,
				primary_message=primary_message,
			),
			ValueError,
			"unavailable-operation",
		),
	)
	for invalid_request, error_type, message in invalid_requests:
		with pytest.raises(error_type, match=message):
			ferrum_qt.dialogs.refusal_presenter.present_refusal(invalid_request)


#============================================
def test_refusal_requires_its_matching_task_context() -> None:
	"""A close warning cannot accidentally be shown as an editing refusal."""
	request = ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
		ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
		ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.BUSY_CLOSE,
	)

	with pytest.raises(ValueError, match="close-document context"):
		ferrum_qt.dialogs.refusal_presenter.present_refusal(request)


#============================================
def test_every_refusal_has_recovery_guidance_without_implementation_vocabulary() -> None:
	"""Every core refusal says what happened, why, and a useful next step."""
	requests = (
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.INVALID_DOCUMENT,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNSUPPORTED_DOCUMENT,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SOURCE_NOT_ALLOWED,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.DOCUMENT_DISPLAY_FAILED,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNSUPPORTED_SAVE_EXTENSION,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.CLOSE_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.BUSY_CLOSE,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.USE_TOOL,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.STALE_TOOL,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
		),
		ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
			ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
			ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.NO_UNDO,
		),
	)
	for request in requests:
		presentation = ferrum_qt.dialogs.refusal_presenter.present_refusal(request)
		ordinary = presentation.ordinary_text().lower()
		assert "what to do now:" in ordinary and presentation.what_next
		assert not any(word in ordinary for word in (
			"rust", "native", "admission", "authoritative", "typed cdml", "publication",
		))
