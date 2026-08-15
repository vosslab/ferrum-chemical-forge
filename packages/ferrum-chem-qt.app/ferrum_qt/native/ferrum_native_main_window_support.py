"""Small native-main-window support types kept out of the composition root."""


#============================================
class NativeOnlyFileFallback:
	"""Terminate native-file controller delegation at the native boundary."""

	#============================================
	def open_file_path(self, file_path: str, replace_current: bool = False) -> bool:
		"""Reject unsupported formats when the native controller delegates to us."""
		if replace_current:
			self._show_native_file_warning(
				"Open in Current Tab Unavailable",
				"Ferrum CDML opens in a new Rust-native tab.",
			)
			return False
		self._show_native_file_warning(
			"Unsupported File Format",
			"Ferrum-native bounded editor currently opens only .cdml files.",
		)
		return False

	#============================================
	def can_save_authoritatively(self) -> bool:
		"""Return false when no native page is selected for the mixin fallback."""
		return False

	#============================================
	def _on_save(self) -> bool:
		"""Provide the native-file mixin's no-tab fallback."""
		return False

	#============================================
	def _on_save_as(self) -> bool:
		"""Provide the native-file mixin's no-tab fallback."""
		return False
