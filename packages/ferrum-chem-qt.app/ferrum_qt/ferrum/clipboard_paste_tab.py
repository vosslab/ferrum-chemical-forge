"""Native-tab installation boundary for a worker-prepared clipboard Paste."""

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
def _presentation_target(root: object) -> object:
	"""Return the durable target carried by one supported clipboard root."""
	if root.kind == "arrow":
		return root.arrow.target
	if root.kind == "plus":
		return root.plus.target
	if root.kind == "text":
		return root.text.target
	if root.kind == "polyline":
		return root.polyline.target
	if root.kind in ("rectangle", "square", "oval", "circle"):
		return root.shape.target
	if root.kind == "polygon":
		return root.polygon.target
	raise native_document_tab_errors.FerrumNativeDocumentTabError(
		"Ferrum Paste projection contains an unsupported presentation root",
	)


#============================================
def _clipboard_paste_selection(projection: object,
		pasted_roots: object) -> tuple[tuple[str, str], ...]:
	"""Resolve inserted source IDs to current projection-owned durable selectors."""
	if type(pasted_roots) is not tuple or not pasted_roots:
		raise native_document_tab_errors.FerrumNativeDocumentTabError(
			"Ferrum Paste returned no inserted roots",
		)
	selection = []
	seen_roots = set()
	for receipt in pasted_roots:
		if (
			type(receipt) is not tuple
			or len(receipt) != 2
			or type(receipt[0]) is not str
			or type(receipt[1]) is not str
			or not receipt[1]
			or receipt in seen_roots
		):
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum Paste returned invalid inserted-root identity",
			)
		seen_roots.add(receipt)
		kind, source_id = receipt
		if kind == "molecule":
			matches = tuple(
				molecule for molecule in projection.molecules
				if molecule.source_id == source_id
			)
			if len(matches) != 1:
				raise native_document_tab_errors.FerrumNativeDocumentTabError(
					"Ferrum Paste molecule is absent from its committed projection",
				)
			for atom in matches[0].atoms:
				if type(atom.id) is not str or not atom.id:
					raise native_document_tab_errors.FerrumNativeDocumentTabError(
						"Ferrum Paste atom has no durable committed identity",
					)
				selection.append(("atom", atom.id))
			for bond in matches[0].bonds:
				if type(bond.id) is not str or not bond.id:
					raise native_document_tab_errors.FerrumNativeDocumentTabError(
						"Ferrum Paste bond has no durable committed identity",
					)
				selection.append(("bond", bond.id))
			continue
		matches = tuple(
			_presentation_target(root) for root in projection.presentation_stack.roots
			if root.kind == kind and _presentation_target(root).source_id == source_id
		)
		if len(matches) != 1 or matches[0].id is None:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum Paste presentation root is absent from its committed projection",
			)
		selection.append((kind, matches[0].id))
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
