"""Behavior coverage for isolated Ferrum document publication."""

# PIP3 modules
import ferrum_chem

# local repo modules
import ferrum_qt.ferrum.document_save

#============================================
def test_invalid_destination_save_error_reports_not_started() -> None:
	"""A native pre-write destination refusal cannot be presented as completion."""
	requests = []

	class SaveErrorWindow(ferrum_qt.ferrum.document_save.FerrumNativeDocumentSaveMixin):
		"""Capture the public refusal request without a Qt modal."""

		def _show_refusal(self, request: object) -> None:
			"""Record the typed request sent to the presentation boundary."""
			requests.append(request)

	window = SaveErrorWindow()
	error = ferrum_chem.InvalidDestinationError("/private/tmp/arrow.cdml", "rejected")
	assert not window._report_native_save_error("/private/tmp/arrow.cdml", error)
	assert requests[-1].outcome.value == "save_not_started"
