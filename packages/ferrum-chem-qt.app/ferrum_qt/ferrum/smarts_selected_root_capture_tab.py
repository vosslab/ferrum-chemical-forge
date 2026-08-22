"""Private tab adapters for opaque live SMARTS selected-query tokens."""


#============================================
class FerrumNativeSmartsSelectedRootCaptureTabMixin:
	"""Keep selected-query capture/run beside the tab-owned live session."""

	#============================================
	def capture_live_smarts_selected_query(self, selection: object) -> object:
		"""Consume one renderer selection and return only Rust's opaque query token."""
		if self._disposed or self.requires_refresh:
			raise RuntimeError("Ferrum document is not ready for molecule capture")
		return self._session._capture_live_document_smarts_selected_query_v1(selection)

	#============================================
	def run_live_smarts_selected_query_token(self, token: object,
			per_molecule_limit: int, total_limit: int) -> object:
		"""Run a token already minted for this tab without consulting generic selection."""
		if self._disposed or self.requires_refresh:
			raise RuntimeError("Ferrum document is not ready for SMARTS query")
		return self._session._run_live_document_smarts_query_v1(
			token, per_molecule_limit, total_limit,
		)
