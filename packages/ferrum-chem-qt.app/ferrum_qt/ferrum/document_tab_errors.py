"""Shared error types for the Rust-owned Ferrum document tab."""


#============================================
class FerrumNativeDocumentTabError(RuntimeError):
	"""Raised when a Ferrum tab cannot install its authoritative observation."""


#============================================
class FerrumNativeDocumentTabUnrenderableMoleculeError(FerrumNativeDocumentTabError):
	"""A canvas edit target lacks a plan in the installed Rust render evidence."""

	#============================================
	def __init__(self, molecule_object_id: str) -> None:
		"""Retain the durable molecule identity that the canvas cannot author."""
		self.molecule_object_id = molecule_object_id
		super().__init__(
			"the current Rust render observation contains no canvas plan for molecule "
			f"{molecule_object_id!r}; no edit was applied",
		)


#============================================
class FerrumNativeDocumentTabMutationPresentationError(FerrumNativeDocumentTabError):
	"""A Rust-accepted edit whose authoritative render is pending refresh."""

	#============================================
	def __init__(self, result: object) -> None:
		"""Retain the accepted Rust result without pretending the old scene is current."""
		self.result = result
		super().__init__(
			"Rust accepted the Ferrum edit, but its authoritative render could not be "
			"installed; refresh before saving or editing again",
		)
