"""Shared error types for the Rust-owned native document tab."""


#============================================
class FerrumNativeDocumentTabError(RuntimeError):
	"""Raised when a native tab cannot install its authoritative observation."""


#============================================
class FerrumNativeDocumentTabMutationPresentationError(FerrumNativeDocumentTabError):
	"""A Rust-accepted edit whose authoritative render is pending refresh."""

	#============================================
	def __init__(self, result: object) -> None:
		"""Retain the accepted Rust result without pretending the old scene is current."""
		self.result = result
		super().__init__(
			"Rust accepted the native edit, but its authoritative render could not be "
			"installed; refresh before saving or editing again",
		)
