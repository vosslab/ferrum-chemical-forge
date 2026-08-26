"""Recovery publication of the current Rust backend CDML without Save effects."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_tab


_RECOVERY_CDML_FILTER = "Ferrum CDML (*.cdml);;All Files (*)"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeRecoveryExportCapture:
	"""One frozen native-tab identity and exact backend snapshot provenance."""

	tab: "ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab"
	revision: int
	digest: str


#============================================
class FerrumNativeRecoveryExportWindowMixin:
	"""Own only the Ferrum Recovery Export action and its provenance fence."""

	#============================================
	def _build_recovery_export_action(self) -> None:
		"""Create and register the explicit backend-copy action."""
		action = PySide6.QtGui.QAction(self.tr("Recovery Export CDML..."), self)
		action.setToolTip(self.tr(
			"Copy the current CDML without changing this document's saved file or unsaved state",
		))
		action.triggered.connect(self._on_native_recovery_export)
		self._recovery_export_action = action
		self._action_registry.register_existing(
			"file.export.recovery_cdml", action,
			shortcut_exemption_reason="Available by its labelled File menu client.",
		)

	#============================================
	def _refresh_recovery_export_action(
			self, active: bool, _pending: bool, _busy: bool) -> None:
		"""Keep recovery reachable for every exact live registered Ferrum tab."""
		tab = self._active_native_tab() if active else None
		available = (
			tab is not None
			and self._native_tabs_by_page.get(tab) is tab
			and not tab.is_disposed
		)
		self._recovery_export_action.setEnabled(available)

	#============================================
	def _on_native_recovery_export(self) -> bool:
		"""Copy one frozen backend snapshot after a dialog-bound provenance check."""
		capture = self._capture_native_recovery_export()
		if capture is None:
			return False
		selected_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Recovery Export CDML"), "", self.tr(_RECOVERY_CDML_FILTER),
		)[0]
		if not selected_path:
			return False
		absolute_path = self._normalize_recovery_export_path(selected_path)
		if absolute_path is None:
			return False
		if not self._recovery_capture_is_current(capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("The active Rust backend snapshot changed while choosing a destination. "
				"Choose Recovery Export again to copy the current snapshot."))
			return False
		try:
			publication = capture.tab.recovery_export(absolute_path, capture.revision)
		except Exception as exc:
			return self._report_native_recovery_export_error(absolute_path, exc)
		if not self._recovery_receipt_matches_capture(publication, capture):
			self._show_edit_refusal(self._unavailable_edit_refusal("Rust returned publication provenance that does not match the selected "
				"backend snapshot. No successful export is being reported; inspect the "
				"destination because Rust may already have written it."))
			return False
		if publication.outcome.is_confirmed:
			self.statusBar().showMessage(
				self.tr("Recovery CDML exported: %s") % absolute_path, 5000,
			)
			return True
		self._show_edit_refusal(self._unavailable_edit_refusal("The exact Rust backend snapshot may be present at %s, but directory-entry "
			"durability needs verification. Inspect the destination before relying on it."
			% absolute_path))
		return False

	#============================================
	def _capture_native_recovery_export(self) -> FerrumNativeRecoveryExportCapture | None:
		"""Capture one exact active live backend snapshot before choosing a path."""
		tab = self._active_native_tab()
		if tab is None or self._native_tabs_by_page.get(tab) is not tab:
			return None
		try:
			snapshot = tab.backend_snapshot_for_recovery_export()
		except Exception as exc:
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return None
		return FerrumNativeRecoveryExportCapture(tab, snapshot.revision, snapshot.digest)

	#============================================
	def _normalize_recovery_export_path(self, selected_path: str) -> str | None:
		"""Apply Ferrum CDML suffix policy without reserving or adopting the path."""
		path = pathlib.Path(selected_path)
		if not path.suffix:
			path = path.with_suffix(".cdml")
		elif path.suffix.lower() != ".cdml":
			self._show_edit_refusal(self._unavailable_edit_refusal("Ferrum recovery exports must use the .cdml extension."))
			return None
		absolute_path = os.path.abspath(str(path))
		return absolute_path

	#============================================
	def _recovery_capture_is_current(self, capture: FerrumNativeRecoveryExportCapture) -> bool:
		"""Reauthenticate exact tab registration, liveness, and backend provenance."""
		if self._active_native_tab() is not capture.tab:
			return False
		if self._native_tabs_by_page.get(capture.tab) is not capture.tab:
			return False
		try:
			snapshot = capture.tab.backend_snapshot_for_recovery_export()
		except Exception:
			return False
		matches = snapshot.revision == capture.revision and snapshot.digest == capture.digest
		return matches

	#============================================
	def _recovery_receipt_matches_capture(
			self, publication: object, capture: FerrumNativeRecoveryExportCapture) -> bool:
		"""Require both public receipt snapshots to corroborate the frozen capture."""
		try:
			published = publication.published_snapshot
			snapshot = publication.snapshot
		except AttributeError:
			return False
		matches = (
			published.revision == capture.revision
			and published.digest == capture.digest
			and snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
		)
		return matches

	#============================================
	def _report_native_recovery_export_error(self, absolute_path: str, exc: Exception) -> bool:
		"""Describe each Rust publication uncertainty without claiming a saved state."""
		import ferrum_qt.ferrum.engine as engine
		if type(exc) is engine.PublicationPossiblyCompletedError:
			message = (
				"Rust could not confirm whether publication completed at %s. Verify the "
				"destination before relying on it.\n\n%s" % (absolute_path, exc)
			)
		elif type(exc) is engine.PublicationNotStartedError:
			message = "Rust did not start recovery publication to %s.\n\n%s" % (absolute_path, exc)
		elif type(exc) is engine.InvalidDestinationError:
			message = "Rust rejected %s. Choose a different destination.\n\n%s" % (
				absolute_path, exc,
			)
		else:
			message = "Could not export the Rust backend snapshot to %s:\n%s" % (
				absolute_path, exc,
			)
		self._show_edit_refusal(self._unavailable_edit_refusal(message))
		return False
