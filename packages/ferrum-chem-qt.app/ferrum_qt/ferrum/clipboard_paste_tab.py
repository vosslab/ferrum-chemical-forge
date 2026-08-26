"""Native-tab installation boundary for a worker-prepared clipboard Paste."""

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


_PASTE_ROOT_KINDS = frozenset((
	"molecule", "arrow", "plus", "text", "rectangle", "square", "oval",
	"circle", "polygon", "polyline",
))


#============================================
def _clipboard_paste_selection(projection: object,
		pasted_roots: object) -> tuple[str, ...]:
	"""Select committed paste roots through their Rust-issued durable identities."""
	if type(pasted_roots) is not tuple or not pasted_roots:
		raise native_document_tab_errors.FerrumNativeDocumentTabError(
			"Ferrum Paste returned no inserted roots",
		)
	selection = []
	seen_roots = set()
	seen_object_ids = set()
	seen_selection = set()
	for receipt in pasted_roots:
		if (
			type(receipt) is not tuple
			or len(receipt) != 2
			or type(receipt[0]) is not str
			or type(receipt[1]) is not str
			or not receipt[1]
			or receipt[0] not in _PASTE_ROOT_KINDS
			or receipt in seen_roots
			or receipt[1] in seen_object_ids
		):
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum Paste returned invalid inserted-root identity",
			)
		seen_roots.add(receipt)
		kind, document_object_id = receipt
		seen_object_ids.add(document_object_id)
		root_matches = tuple(
			root for root in projection.direct_roots
			if (
				getattr(root, "kind", None) == kind
				and getattr(root, "document_object_id", None) == document_object_id
			)
		)
		if len(root_matches) != 1:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum Paste root is absent from its committed projection",
			)
		if kind == "molecule":
			matches = tuple(
				molecule for molecule in projection.molecules
				if getattr(molecule, "document_object_id", None) == document_object_id
			)
			if len(matches) != 1:
				raise native_document_tab_errors.FerrumNativeDocumentTabError(
					"Ferrum Paste molecule is absent from its committed projection",
				)
			selection_start = len(selection)
			for atom in matches[0].atoms:
				atom_object_id = getattr(atom, "document_object_id", None)
				if (
					type(atom_object_id) is not str
					or not atom_object_id
					or atom_object_id in seen_selection
				):
					raise native_document_tab_errors.FerrumNativeDocumentTabError(
						"Ferrum Paste atom has no durable committed identity",
					)
				seen_selection.add(atom_object_id)
				selection.append(atom_object_id)
			for bond in matches[0].bonds:
				bond_object_id = getattr(bond, "document_object_id", None)
				if (
					type(bond_object_id) is not str
					or not bond_object_id
					or bond_object_id in seen_selection
				):
					raise native_document_tab_errors.FerrumNativeDocumentTabError(
						"Ferrum Paste bond has no durable committed identity",
					)
				seen_selection.add(bond_object_id)
				selection.append(bond_object_id)
			if len(selection) == selection_start:
				raise native_document_tab_errors.FerrumNativeDocumentTabError(
					"Ferrum Paste molecule has no committed selectable members",
				)
			continue
		if document_object_id in seen_selection:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum Paste returned duplicate committed selection identity",
			)
		seen_selection.add(document_object_id)
		selection.append(document_object_id)
	return tuple(selection)


#============================================
class FerrumNativeClipboardPasteTabMixin:
	"""Commit a prepared Paste and restore selection through durable receipts."""

	#============================================
	def apply_prepared_clipboard_paste(self, prepared: object,
			expected_revision: int, expected_digest: str) -> object:
		"""Commit one worker-prepared fragment and select every inserted root."""
		self._require_mutable()
		import ferrum_qt.ferrum.engine as engine
		if type(prepared) is not engine.DocumentClipboardPastePlanV1:
			raise TypeError("Ferrum Paste requires an exact frozen Ferrum plan")
		if type(expected_revision) is not int or type(expected_digest) is not str:
			raise TypeError("Ferrum Paste requires exact revision and digest facts")
		result = self._session.apply_clipboard_paste_v1(
			expected_revision, expected_digest, prepared,
		)
		try:
			selection = _clipboard_paste_selection(
				result.operation.observation.projection, result.pasted_roots,
			)
		except Exception as exc:
			self._pending_result = result.operation
			self._pending_snapshot = result.operation.observation.snapshot
			self._pending_durable_selection = None
			raise (
				native_document_tab_errors.
				FerrumNativeDocumentTabMutationPresentationError(result.operation)
			) from exc
		self._install_mutation_result(result.operation, selection)
		return result
