"""Authenticated Cut commit client for one Ferrum document tab."""

#============================================
class FerrumNativeClipboardCutTabMixin:
	"""Commit one prepared Cut and install its authoritative empty selection."""

	#============================================
	def apply_prepared_clipboard_cut(self, prepared: object,
			expected_revision: int, expected_digest: str) -> object:
		"""Commit one exact Rust Cut plan and clear its deleted selection."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(prepared) is not engine.DocumentClipboardCutPlanV1:
			raise TypeError("Ferrum Cut requires an exact frozen Ferrum plan")
		if type(expected_revision) is not int or type(expected_digest) is not str:
			raise TypeError("Ferrum Cut requires exact revision and digest facts")
		result = self._session.apply_clipboard_cut_v1(
			expected_revision, expected_digest, prepared,
		)
		self._install_mutation_result(result, ())
		return result
