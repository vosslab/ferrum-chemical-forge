"""Text-import dialogs plus guarded asynchronous proposal delivery."""

import PySide6.QtCore
import PySide6.QtWidgets

import ferrum_qt.bridge.chemistry_preparation
import ferrum_qt.bridge.insertion_placement
import ferrum_qt.models.document_session

def _read_smiles(app: object) -> None:
	"""Prompt for a SMILES string and import as a molecule.

	Parses the SMILES via OASA, generates 2D coordinates, converts
	to a MoleculeModel, and adds it to the scene.

	Args:
		app: MainWindow instance.
	"""
	text, ok = PySide6.QtWidgets.QInputDialog.getText(
		app, "Import SMILES", "Enter SMILES string:"
	)
	if not ok or not text.strip():
		return
	smiles_string = text.strip()
	_start_text_import(
		app,
		"smiles",
		smiles_string,
		"SMILES",
		"Imported SMILES molecule",
		_show_smiles_import_error,
	)


#============================================
class _MoleculeInsertionResultRelay(PySide6.QtCore.QObject):
	"""Deliver one plain molecule proposal only to its live source tab."""

	#============================================
	def __init__(
			self, target: object, worker: PySide6.QtCore.QThread,
			delivery: "MoleculeInsertionDelivery",
			) -> None:
		"""Retain frontend lifecycle facts until the worker has stopped."""
		super().__init__(delivery.app)
		self._target = target
		self._worker = worker
		self._delivery = delivery

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_result(self, prepared: object) -> None:
		"""Submit one immutable proposal through the public session operation seam."""
		self._delivery.deliver(prepared)

	#============================================
	@PySide6.QtCore.Slot(object)
	def on_error(self, message: object) -> None:
		"""Show preparation failure only while the source request remains current."""
		self._delivery.report_error(message)

	#============================================
	@PySide6.QtCore.Slot()
	def on_thread_finished(self) -> None:
		"""Release through the window's terminal-safe worker owner."""
		self._delivery.app._release_import_worker(self._worker)
		self.deleteLater()


#============================================
def _start_text_import(
		app: object, codec_name: str, source_text: str, source_label: str,
		success_message: str, error_handler: object,
		) -> None:
	"""Start one backend-authoritative text molecule insertion.

	The source tab, revision, and scalar placement are captured before worker
	startup.  The worker returns a frozen CDML proposal; delivery can therefore
	commit only through the session's authoritative molecule-insertion route.
	"""
	target = app._active_session
	try:
		target_mean_bond_length, insertion_anchor = (
			ferrum_qt.bridge.insertion_placement.capture_insertion_placement(target)
		)
	except ValueError as error:
		error_handler(app, error)
		return
	request_token = target.begin_import_request()
	expected_revision = target.backend_snapshot.revision
	token_stem = "%s-r%s-i%s" % (codec_name, expected_revision, request_token)
	worker = ferrum_qt.bridge.chemistry_preparation.create_text_molecule_insertion_worker(
		codec_name, source_text, expected_revision, token_stem,
		target_mean_bond_length, insertion_anchor, success_message,
	)
	delivery = MoleculeInsertionDelivery(
		app, target, request_token, expected_revision, source_label,
		success_message, error_handler,
	)
	relay = _MoleculeInsertionResultRelay(target, worker, delivery)
	worker._result_relay = relay
	connection_type = PySide6.QtCore.Qt.ConnectionType.QueuedConnection
	worker.result.connect(relay.on_result, connection_type)
	worker.error.connect(relay.on_error, connection_type)
	worker.finished.connect(relay.on_thread_finished, connection_type)
	target.track_import_worker(worker)
	app.statusBar().showMessage("Loading %s..." % source_label, 0)
	worker.start()


#============================================
class MoleculeInsertionDelivery:
	"""Deliver one text-derived proposal to its captured backend session.

	This frontend-local controller owns the source-tab fence and user feedback.
	It delegates request construction to the named bridge and exposes only
	persistent-operation outcomes, never backend handles or OASA graph objects.
	"""

	#============================================
	def __init__(
			self, app: object, target: object, request_token: int,
			expected_revision: int, source_label: str, success_message: str,
			error_handler: object,
			) -> None:
		"""Capture one source tab and immutable preparation generation."""
		self.app = app
		self._target = target
		self._request_token = request_token
		self._expected_revision = expected_revision
		self._source_label = source_label
		self._success_message = success_message
		self._error_handler = error_handler

	#============================================
	def is_current(self) -> bool:
		"""Return whether this request may still affect its source tab."""
		return (
			self._target in self.app.sessions
			and self._target.import_request_is_current(self._request_token)
		)

	#============================================
	def _discarded_outcome(
			self, message: str,
			) -> ferrum_qt.models.document_session.PersistentActionOutcome:
		"""Return a uniform inert result for a stale source request."""
		return ferrum_qt.models.document_session.PersistentActionOutcome(
			"discarded", message, None, False,
		)

	#============================================
	def deliver(
			self, prepared: object,
			) -> ferrum_qt.models.document_session.PersistentActionOutcome:
		"""Submit one current plain proposal through the persistent action seam."""
		if not self.is_current():
			return self._discarded_outcome(
				"%s import request is no longer current" % self._source_label,
			)
		proposal = ferrum_qt.bridge.chemistry_preparation.molecule_insertion_proposal(
			prepared,
		)
		if proposal is None:
			message = "%s preparation returned invalid data" % self._source_label
			self._error_handler(self.app, message)
			return ferrum_qt.models.document_session.PersistentActionOutcome(
				"rejected", message, None, False,
			)
		if proposal.expected_revision != self._expected_revision:
			message = "%s preparation revision changed" % self._source_label
			self._error_handler(self.app, message)
			return ferrum_qt.models.document_session.PersistentActionOutcome(
				"rejected", message, None, False,
			)
		request = ferrum_qt.bridge.chemistry_preparation.build_molecule_insertion_request(
			proposal, self._success_message,
		)
		outcome = self._target.submit_persistent_operation(request)
		if outcome.status == "accepted":
			self.app.statusBar().showMessage(outcome.message, 3000)
		elif outcome.submitted:
			self.app.statusBar().showMessage(outcome.message, 5000)
		elif outcome.status == "rejected":
			self._error_handler(self.app, outcome.message)
		return outcome

	#============================================
	def report_error(self, message: object) -> bool:
		"""Show one worker error only while its source request remains current."""
		if not self.is_current():
			return False
		self._error_handler(self.app, message)
		return True


#============================================
def _show_smiles_import_error(app: object, message: object) -> None:
	"""Report a current worker failure through the SMILES dialog vocabulary."""
	PySide6.QtWidgets.QMessageBox.warning(
		app, "SMILES Error", f"Failed to parse SMILES:\n{message}",
	)


#============================================
def _show_inchi_import_error(app: object, message: object) -> None:
	"""Report an InChI preparation error with the legacy dialog vocabulary."""
	stage, detail = _text_import_error_stage(message)
	if stage == "coordinates":
		title = "Coordinate Error"
		body = "Failed to generate coordinates:\n%s" % detail
	else:
		title = "InChI Error"
		body = "Failed to parse InChI:\n%s" % detail
	PySide6.QtWidgets.QMessageBox.warning(app, title, body)


#============================================
def _show_peptide_import_error(app: object, message: object) -> None:
	"""Report peptide preparation errors with parser-specific labels."""
	stage, detail = _text_import_error_stage(message)
	if stage == "coordinates":
		title = "Coordinate Error"
		body = "Failed to generate coordinates:\n%s" % detail
	elif stage == "peptide-smiles":
		title = "SMILES Error"
		body = "Failed to parse peptide SMILES:\n%s" % detail
	elif stage == "peptide":
		title = "Peptide Sequence Error"
		body = "Failed to convert peptide sequence:\n%s" % detail
	elif stage == "peptide-validation":
		title = "Peptide Sequence Error"
		body = detail
	else:
		title = "Peptide Sequence Error"
		body = "Failed to import peptide sequence:\n%s" % detail
	PySide6.QtWidgets.QMessageBox.warning(app, title, body)


#============================================
def _text_import_error_stage(message: object) -> tuple[str, str]:
	"""Recover a preparation stage carried by a worker exception string."""
	facts = ferrum_qt.bridge.chemistry_preparation.text_import_failure_facts(message)
	return facts.stage, facts.message


#============================================
def _read_inchi(app: object) -> None:
	"""Prompt for an InChI string and import as a molecule.

	Parses the InChI via OASA, generates 2D coordinates, converts
	to a MoleculeModel, and adds it to the scene.

	Args:
		app: MainWindow instance.
	"""
	text, ok = PySide6.QtWidgets.QInputDialog.getText(
		app, "Import InChI", "Enter InChI string:"
	)
	if not ok or not text.strip():
		return
	_start_text_import(
		app,
		"inchi",
		text.strip(),
		"InChI",
		"Imported InChI molecule",
		_show_inchi_import_error,
	)


#============================================
def _read_peptide(app: object) -> None:
	"""Prompt for a peptide sequence and import as a molecule.

	Validates single-letter amino acid codes, converts the sequence
	to SMILES via OASA, generates 2D coordinates, converts to a
	MoleculeModel, and adds it to the scene.

	Args:
		app: MainWindow instance.
	"""
	# build prompt listing supported amino acid codes
	supported = ferrum_qt.bridge.chemistry_preparation.supported_peptide_codes()
	supported_str = ", ".join(supported)
	prompt_text = (
		"Enter a single-letter amino acid sequence (e.g. ANKLE):\n"
		f"Supported: {supported_str}"
	)
	text, ok = PySide6.QtWidgets.QInputDialog.getText(
		app, "Import Peptide Sequence", prompt_text
	)
	if not ok or not text.strip():
		return
	# Normalization is UI-only; validation, conversion, parsing, and layout run
	# together in the session-owned OASA worker.
	sequence = text.strip().upper()
	_start_text_import(
		app,
		"peptide",
		sequence,
		"Peptide Sequence",
		f"Imported peptide sequence '{sequence}'",
		_show_peptide_import_error,
	)


#============================================
