"""Shared error types for the Rust-owned native document tab."""


#============================================
class FerrumNativeDocumentTabError(RuntimeError):
	"""Raised when a native tab cannot install its authoritative observation."""
