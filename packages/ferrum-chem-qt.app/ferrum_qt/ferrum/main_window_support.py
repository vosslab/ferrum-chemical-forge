"""Small native-main-window support types kept out of the composition root."""


#============================================
class NativeOnlyFileFallback:
	"""Terminate native-file controller delegation at the Ferrum boundary."""

	#============================================
	def open_file_path(self, file_path: str, replace_current: bool = False) -> bool:
		"""Reject unsupported formats when the Ferrum controller delegates to us."""
		if replace_current:
			self._show_edit_refusal(self._unavailable_edit_refusal("This drawing opens in a new tab."))
			return False
		self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum currently opens only .cdml drawing files."))
		return False

	#============================================
	def can_save_authoritatively(self) -> bool:
		"""Return false when no Ferrum page is selected for the mixin fallback."""
		return False

	#============================================
	def _on_save(self) -> bool:
		"""Provide the native-file mixin's no-tab fallback."""
		return False

	#============================================
	def _on_save_as(self) -> bool:
		"""Provide the native-file mixin's no-tab fallback."""
		return False
