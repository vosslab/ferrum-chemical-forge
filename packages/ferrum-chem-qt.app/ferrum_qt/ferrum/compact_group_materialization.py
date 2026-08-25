"""Live Rust-owned compact-group materialization action."""

# Standard Library
import dataclasses
import functools
import json

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
_OPERATION_KIND = "document.compact-group.materialize.v1"
_RESULT_SCHEMA = "ferrum-document-compact-group-materialization-v1"
_ERROR_SCHEMA = "ferrum-operation-error-v1"
_REFUSAL_RECOVERY = {
	"stale_document_fence": "Refresh the selected compact group and try again.",
	"unknown_or_foreign_target": "Refresh and select a current compact group.",
	"ineligible_target": "Select an eligible attached compact group.",
	"renderer_preparation_refused": "Refresh the document, then try again.",
	"session_conflict_or_replayed_preparation": "Refresh and restart materialization.",
}


#============================================
def _request_json(address: object) -> str:
	"""Build one fenced public operation request from a durable group address."""
	request = {
		"schema": "ferrum-operation-request-v1",
		"request_id": "qt-compact-group-materialization",
		"operation": {
			"kind": _OPERATION_KIND,
			"document": {
				"cdml": address.document,
				"expected_revision": address.revision,
				"expected_digest_hex": address.digest,
			},
			"molecule_id": address.molecule_id,
			"compact_group_id": address.compact_group_id,
		},
	}
	return json.dumps(request, separators=(",", ":"), ensure_ascii=True)


#============================================
def _response_from_receipt(receipt: object) -> dict | None:
	"""Decode one JSON object from the public live-operation receipt."""
	if type(receipt.response_json) is not str:
		return None
	try:
		response = json.loads(receipt.response_json)
	except json.JSONDecodeError:
		return None
	return response if type(response) is dict else None


#============================================
def _materialization_from_response(response: object, address: object) -> dict | None:
	"""Decode one exact successful response matching its source-fenced selection."""
	if type(response) is not dict or response.get("schema") != "ferrum-operation-response-v1":
		return None
	outcome = response.get("outcome")
	if type(outcome) is not dict or outcome.get("kind") != _OPERATION_KIND:
		return None
	materialization = outcome.get("materialization")
	if (
		type(materialization) is not dict
		or materialization.get("schema") != _RESULT_SCHEMA
		or materialization.get("source_revision") != address.revision
		or materialization.get("source_digest_hex") != address.digest
		or materialization.get("molecule_id") != address.molecule_id
		or materialization.get("compact_group_id") != address.compact_group_id
		or type(materialization.get("replacement_focus_atom_id")) is not str
		or not materialization["replacement_focus_atom_id"]
	):
		return None
	return materialization


#============================================
def _typed_protocol_recovery(response: object) -> str | None:
	"""Return closed recovery text for this operation's typed error envelope."""
	if type(response) is not dict or response.get("schema") != _ERROR_SCHEMA:
		return None
	error = response.get("error")
	if type(error) is not dict or error.get("operation") != _OPERATION_KIND:
		return None
	return _REFUSAL_RECOVERY.get(error.get("category"))


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _CompactGroupMaterializationIntent:
	"""One queued owner-thread compact-group mutation at a fixed source fence."""

	tab: object
	address: object


#============================================
class FerrumNativeCompactGroupMaterializationWindowMixin:
	"""Expose one live Rust-owned compact-group action in the Chemistry menu."""

	#============================================
	def _initialize_compact_group_materialization(self) -> None:
		"""Initialize the action before the window constructs its menus."""
		self._compact_group_materialization_action: PySide6.QtGui.QAction | None = None
		self._compact_group_materialization_intent: _CompactGroupMaterializationIntent | None = None

	#============================================
	def _build_compact_group_materialization_action(
			self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the accessible public Chemistry action."""
		action = PySide6.QtGui.QAction(self.tr("Materialize Selected Compact Group"), self)
		action.setObjectName("materialize-selected-compact-group-action")
		action.setIconText(self.tr("Materialize Selected Compact Group"))
		action.setStatusTip(self.tr(
			"Replace one selected compact group with its Ferrum Rust materialization.",
		))
		action.setToolTip(self.tr(
			"Materialize the selected compact group with Ferrum Rust.",
		))
		action.setWhatsThis(self.tr(
			"Replace one selected attached compact group with ordinary atoms and bonds. "
			"Ferrum Rust validates chemistry, identifiers, geometry, and rendering.",
		))
		action.triggered.connect(self._materialize_selected_compact_group)
		menu.addAction(action)
		self._compact_group_materialization_action = action

	#============================================
	def _materialize_selected_compact_group(self) -> bool:
		"""Run one fenced compact operation on the owning Qt/Python thread."""
		if self._compact_group_materialization_intent is not None:
			return False
		tab = self._active_native_tab()
		if tab is None or self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			return False
		if self._atom_insertion_intent is not None:
			self._cancel_atom_insertion()
		if self._line_gesture_intent is not None:
			self._cancel_line_gesture()
		if self._structure_tab is tab:
			self._cancel_structure_selection()
		try:
			address = tab.selected_molecule_compact_group_address()
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		except (TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		self._compact_group_materialization_intent = _CompactGroupMaterializationIntent(
			tab, address,
		)
		self._run_compact_group_materialization()
		return True

	#============================================
	def _run_compact_group_materialization(self) -> None:
		"""Apply the queued operation only on the tab's owning Qt/Python thread."""
		intent = self._compact_group_materialization_intent
		if intent is None:
			return
		tab = intent.tab
		address = intent.address
		if self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			self._compact_group_materialization_intent = None
			self._refresh_actions()
			return
		try:
			receipt = tab._session.apply_live_document_operation_v1(_request_json(address))
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._finish_compact_group_materialization("failed", "unchanged")
			return
		except (TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._finish_compact_group_materialization("failed", "unchanged")
			return
		response = _response_from_receipt(receipt)
		materialization = _materialization_from_response(response, address)
		if materialization is None:
			recovery = _typed_protocol_recovery(response)
			if recovery is not None:
				self.statusBar().showMessage(self.tr(
					"Ferrum could not materialize the compact group: {0}".format(recovery),
				), 5000)
				self._finish_compact_group_materialization("refused", "unchanged")
				return
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an unrecognized compact-group operation result.",
			))
			self._finish_compact_group_materialization("failed", "unchanged")
			return
		result = receipt.mutation_result
		if result is None:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum accepted compact-group materialization without a live mutation receipt.",
			))
			self._finish_compact_group_materialization("failed", "unchanged")
			return
		focus_atom_id = materialization["replacement_focus_atom_id"]
		try:
			tab._install_mutation_result(result, (("atom", focus_atom_id),))
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._finish_compact_group_materialization("failed", "unchanged")
			return
		self.statusBar().showMessage(self.tr(
			"Materialized the selected compact group with Ferrum Rust.",
		), 5000)
		self._finish_compact_group_materialization("succeeded", "updated")

	#============================================
	def _finish_compact_group_materialization(self, terminal_kind: str,
			document_effect: str) -> None:
		"""Clear one owner-thread intent and publish exactly one terminal receipt."""
		intent = self._compact_group_materialization_intent
		if intent is None:
			return
		self._compact_group_materialization_intent = None
		self._refresh_actions()
		PySide6.QtCore.QTimer.singleShot(0, functools.partial(
			self._queue_operation_presentation_v1,
			intent.tab, _OPERATION_KIND, terminal_kind, document_effect,
			intent.address.revision, intent.address.digest,
		))

	#============================================
	def _refresh_compact_group_materialization_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Enable only for one current typed compact-group selection."""
		action = self._compact_group_materialization_action
		if action is None:
			return
		available = False
		if (
			active and not pending and not busy_elsewhere
		):
			tab = self._active_native_tab()
			try:
				if tab is not None:
					tab.selected_molecule_compact_group_address()
			except native_document_tab_errors.FerrumNativeDocumentTabError:
				available = False
			else:
				available = tab is not None
		action.setEnabled(available)
