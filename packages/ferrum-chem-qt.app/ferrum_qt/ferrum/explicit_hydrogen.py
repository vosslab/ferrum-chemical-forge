"""Live Rust-owned explicit-hydrogen materialization action."""

# Standard Library
import json

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
_OPERATION_KIND = "document.molecule.hydrogen.materialize.v1"
_RESULT_SCHEMA = "ferrum-document-molecule-hydrogen-materialization-v1"
_ERROR_SCHEMA = "ferrum-operation-error-v1"
_UNAVAILABLE_RECOVERY = {
	"element_outside_profile": "Use a direct H/C/N/O molecule.",
	"nonzero_formal_charge": "Use a neutral molecule.",
	"nonzero_explicit_hydrogens": "Clear aggregate explicit-hydrogen counts first.",
	"unsupported_bond_or_radical": "Use non-radical single, double, or triple bonds.",
	"existing_hydrogen_topology": "Use a molecule without existing hydrogen vertices.",
	"valence_exceeded": "Repair the selected molecule's bond valence.",
	"unsupported_document": "Use one supported direct molecule.",
	"resource_limit": "Use a smaller selected molecule.",
	"unrenderable_candidate": "Adjust the molecular layout, then try again.",
	"oxidation_postcondition": "Use a neutral supported H/C/N/O molecule.",
	"render_preparation": "Refresh the document, then try again.",
}
_PROTOCOL_ERROR_RECOVERY = {
	"invalid_request": "Refresh the document and try again.",
	"unsupported_protocol_version": "Update Ferrum, then try again.",
	"document_admission_failed": "Refresh the document, then try again.",
	"document_invalid": "Repair the document, then try again.",
	"render_unsupported": "Use a renderable molecule, then try again.",
	"render_failed": "Refresh the document, then try again.",
	"chemistry_unavailable": "Restart Ferrum after its chemistry runtime is available.",
	"conversion_failed": "Refresh the document, then try again.",
	"conversion_unsupported": "Use a supported direct molecule.",
	"coordinate_generation_failed": "Adjust the molecular layout, then try again.",
	"stale_document": "Refresh the document, select one atom, then try again.",
	"atom_not_found": "Select one current atom, then try again.",
	"molecule_not_direct_root": "Select an atom in one direct molecule, then try again.",
	"atom_not_in_selected_molecule": "Select one current atom, then try again.",
	"unsupported_document": "Use one supported direct molecule.",
	"cancelled_before_dispatch": "Run Make Hydrogens Explicit again.",
	"internal_failure": "Refresh the document and try again.",
}
_RESOURCE_LIMIT_RECOVERY = {
	("response_size_exceeded", "reduce_requested_result"): (
		"Reduce the requested result, then try again."
	),
	("oxidation_root_atoms_exceeded", "use_smaller_root"): (
		"Use a smaller selected molecule."
	),
	("oxidation_root_bonds_exceeded", "use_smaller_root"): (
		"Use a smaller selected molecule."
	),
	("oxidation_root_components_exceeded", "use_smaller_root"): (
		"Use a smaller selected molecule."
	),
}


#============================================
def _request_json(address: object) -> str:
	"""Build one fenced public operation request from a durable atom address."""
	request = {
		"schema": "ferrum-operation-request-v1",
		"request_id": "qt-hydrogen-materialization",
		"operation": {
			"kind": _OPERATION_KIND,
			"document": {
				"cdml": address.document,
				"expected_revision": address.revision,
				"expected_digest_hex": address.digest,
			},
			"molecule_id": address.molecule_id,
			"anchor_atom_id": address.atom_id,
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
		or materialization.get("anchor_atom_id") != address.atom_id
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
	category = error.get("category")
	if category == "resource_limit":
		resource_limit = error.get("resource_limit")
		if type(resource_limit) is not dict:
			return None
		return _RESOURCE_LIMIT_RECOVERY.get((
			resource_limit.get("reason"), resource_limit.get("recovery"),
		))
	return _PROTOCOL_ERROR_RECOVERY.get(category)


#============================================
class FerrumNativeExplicitHydrogenWindowMixin:
	"""Expose one live Rust-owned materialization command in the Chemistry menu."""

	#============================================
	def _initialize_explicit_hydrogen(self) -> None:
		"""Initialize the action before the window constructs its menus."""
		self._explicit_hydrogen_action: PySide6.QtGui.QAction | None = None

	#============================================
	def _build_explicit_hydrogen_action(self, menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the accessible public Chemistry action."""
		action = PySide6.QtGui.QAction(self.tr("Make Hydrogens Explicit"), self)
		action.setObjectName("make-hydrogens-explicit-action")
		action.setIconText(self.tr("Make Hydrogens Explicit"))
		action.setStatusTip(self.tr(
			"Materialize hydrogens for one selected molecule with Ferrum Rust.",
		))
		action.setToolTip(self.tr(
			"Materialize hydrogen atoms for the selected molecule with Ferrum Rust.",
		))
		action.setWhatsThis(self.tr(
			"Materialize explicit hydrogen atoms for the molecule containing one selected atom. "
			"Ferrum Rust validates chemistry, identifiers, geometry, and rendering.",
		))
		action.triggered.connect(self._make_hydrogens_explicit)
		menu.addAction(action)
		self._explicit_hydrogen_action = action

	#============================================
	def _make_hydrogens_explicit(self) -> bool:
		"""Run the one generic materialization operation on the active live tab."""
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
			address = tab.selected_molecule_atom_address()
			receipt = tab._session.apply_live_document_operation_v1(_request_json(address))
		except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		except (TypeError, ValueError, RuntimeError) as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			self._refresh_actions()
			return False
		if self._native_tabs_by_page.get(tab) is not tab or tab.is_disposed:
			return False
		response = _response_from_receipt(receipt)
		materialization = _materialization_from_response(response, address)
		if materialization is None:
			recovery = _typed_protocol_recovery(response)
			if recovery is not None:
				self.statusBar().showMessage(self.tr(
					"Ferrum could not make hydrogens explicit: {0}".format(recovery),
				), 5000)
				self._refresh_actions()
				error = response.get("error") if type(response) is dict else None
				if type(error) is dict and error.get("category") != "cancelled_before_dispatch":
					self._publish_operation_presentation_v1(
						tab, _OPERATION_KIND, "refused", "unchanged",
						address.revision, address.digest,
					)
				return False
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum returned an unrecognized explicit-hydrogen operation result.",
			))
			self._refresh_actions()
			self._publish_operation_presentation_v1(
				tab, _OPERATION_KIND, "failed", "unchanged",
				address.revision, address.digest,
			)
			return False
		status = materialization.get("status")
		if status == "applied":
			result = receipt.mutation_result
			if result is None:
				self._show_edit_refusal(self._unavailable_edit_refusal(
					"Ferrum accepted hydrogen materialization without a live mutation receipt.",
				))
				self._refresh_actions()
				self._publish_operation_presentation_v1(
					tab, _OPERATION_KIND, "failed", "unchanged",
					address.revision, address.digest,
				)
				return False
			try:
				tab._install_mutation_result(result, (("atom", address.atom_id),))
			except native_document_tab_errors.FerrumNativeDocumentTabError as exc:
				self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
				self._refresh_actions()
				self._publish_operation_presentation_v1(
					tab, _OPERATION_KIND, "failed", "unchanged",
					address.revision, address.digest,
				)
				return False
			self.statusBar().showMessage(self.tr(
				"Made {0} hydrogens explicit with Ferrum Rust.".format(
					materialization.get("added_hydrogen_count", 0),
				),
			), 5000)
			self._refresh_actions()
			self._publish_operation_presentation_v1(
				tab, _OPERATION_KIND, "succeeded", "updated",
				address.revision, address.digest,
			)
			return True
		if status == "no_op":
			self.statusBar().showMessage(self.tr(
				"Hydrogens are already explicit for the selected molecule.",
			), 5000)
			self._refresh_actions()
			self._publish_operation_presentation_v1(
				tab, _OPERATION_KIND, "succeeded", "unchanged",
				address.revision, address.digest,
			)
			return True
		if status == "unavailable":
			reason = materialization.get("unavailable_reason")
			recovery = _UNAVAILABLE_RECOVERY.get(reason)
			if recovery is not None:
				self.statusBar().showMessage(self.tr(
					"Ferrum could not make hydrogens explicit: {0}".format(recovery),
				), 5000)
				self._refresh_actions()
				self._publish_operation_presentation_v1(
					tab, _OPERATION_KIND, "unavailable", "unchanged",
					address.revision, address.digest,
				)
				return False
		self._show_edit_refusal(self._unavailable_edit_refusal(
			"Ferrum returned an unrecognized explicit-hydrogen operation result.",
		))
		self._refresh_actions()
		self._publish_operation_presentation_v1(
			tab, _OPERATION_KIND, "failed", "unchanged",
			address.revision, address.digest,
		)
		return False

	#============================================
	def _refresh_explicit_hydrogen_action(self, active: bool, pending: bool,
			busy_elsewhere: bool) -> None:
		"""Enable only for a mutable live tab with exactly one durable atom selected."""
		action = self._explicit_hydrogen_action
		if action is None:
			return
		available = False
		if active and not pending and not busy_elsewhere:
			tab = self._active_native_tab()
			try:
				if tab is not None:
					tab.selected_molecule_atom_address()
			except native_document_tab_errors.FerrumNativeDocumentTabError:
				available = False
			else:
				available = tab is not None
		action.setEnabled(available)
