"""Public immutable receipts for visible Ferrum document operations."""

# Standard Library
import dataclasses


SCHEMA = "ferrum-qt-operation-presentation-v1"
TERMINAL_KINDS = frozenset(("succeeded", "unavailable", "refused", "failed"))
DOCUMENT_EFFECTS = frozenset(("updated", "unchanged"))


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumOperationPresentationV1:
	"""One terminal visible state for a public document operation."""

	schema: str
	operation_kind: str
	terminal_kind: str
	document_effect: str
	source_revision: int
	source_digest_hex: str
	current_revision: int
	current_digest_hex: str

	#============================================
	def __post_init__(self) -> None:
		"""Reject values outside the compact public receipt vocabulary."""
		if self.schema != SCHEMA:
			raise ValueError("Ferrum operation presentation requires its V1 schema")
		if type(self.operation_kind) is not str or not self.operation_kind.startswith("document."):
			raise TypeError("Ferrum operation presentation requires a public document operation kind")
		if self.terminal_kind not in TERMINAL_KINDS:
			raise ValueError("Ferrum operation presentation requires a closed terminal kind")
		if self.document_effect not in DOCUMENT_EFFECTS:
			raise ValueError("Ferrum operation presentation requires a closed document effect")
		for revision in (self.source_revision, self.current_revision):
			if type(revision) is not int or revision < 0:
				raise TypeError("Ferrum operation presentation requires nonnegative revisions")
		for digest in (self.source_digest_hex, self.current_digest_hex):
			if type(digest) is not str or not digest:
				raise TypeError("Ferrum operation presentation requires document digests")
