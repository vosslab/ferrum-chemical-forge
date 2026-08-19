"""Rust-only CDML open and publication behavior for Ferrum document tabs."""

# Standard Library
import os
import pathlib

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.refusal_presenter
import ferrum_qt.ferrum.document_tab


_NATIVE_CDML_FILTER = "Ferrum CDML (*.cdml);;All Files (*)"


#============================================
class WindowNativeFileMixin:
	"""Route CDML file actions to Ferrum tabs."""

	_native_cdml_default_open_enabled = True

	#============================================
	def can_save_authoritatively(self) -> bool:
		"""Permit Save whenever the selected page is a live Ferrum tab."""
		if self._active_native_tab() is not None:
			return True
		if getattr(self, "_neutral_native_shell", False):
			return False
		return super().can_save_authoritatively()

	#============================================
	def open_file_path(self, file_path: str, replace_current: bool = False) -> bool:
		"""Open CDML through Rust and leave every non-CDML route unchanged."""
		absolute_path = os.path.abspath(file_path)
		if (
			pathlib.Path(absolute_path).suffix.lower() != ".cdml"
			or not self._native_cdml_default_open_enabled
		):
			return super().open_file_path(file_path, replace_current)
		return self._open_native_cdml(absolute_path, replace_current)

	#============================================
	def open_native_cdml_path(self, file_path: str) -> bool:
		"""Open an explicitly chosen CDML path through the Ferrum route."""
		absolute_path = os.path.abspath(file_path)
		if pathlib.Path(absolute_path).suffix.lower() != ".cdml":
			self._show_refusal(
				ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
					ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
					ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNSUPPORTED_DOCUMENT,
					pathlib.Path(absolute_path).name,
				),
			)
			return False
		return self._open_native_cdml(absolute_path, replace_current=False)

	#============================================
	def _open_native_cdml(self, absolute_path: str, replace_current: bool) -> bool:
		"""Load one local CDML file through Rust's named V1 resource profile."""
		existing = self._native_tab_for_path(absolute_path)
		if existing is not None:
			self._tab_widget.setCurrentIndex(self._tab_widget.indexOf(existing))
			return True
		if replace_current:
			self._show_refusal(ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.EDIT_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNAVAILABLE_OPERATION,
				technical_details=(
					"This drawing opens in a new tab. Replacing the current tab is not "
					"available yet."
				),
			))
			return False
		try:
			admission = self._prepare_local_cdml_admission(absolute_path)
			tab = self._create_native_tab_from_admission(
				admission, pathlib.Path(absolute_path).name,
			)
			tab._adopt_loaded_origin_path(absolute_path)
		except Exception as exc:
			self._show_refusal(
				ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
					ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
					ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.INVALID_DOCUMENT,
					pathlib.Path(absolute_path).name, str(exc),
				),
			)
			return False
		try:
			self._register_native_tab(tab, activate=True)
		except Exception as exc:
			tab.dispose()
			self._show_refusal(
				ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
					ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.OPEN_DOCUMENT,
					ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.INVALID_DOCUMENT,
					pathlib.Path(absolute_path).name, str(exc),
				),
			)
			return False
		self.statusBar().showMessage(self.tr("Opened drawing: %s") % absolute_path, 3000)
		return True

	#============================================
	def _prepare_local_cdml_admission(self, absolute_path: str) -> tuple[object, object]:
		"""Synchronously consume one complete Rust-owned local-CDML admission."""
		import ferrum_qt.ferrum.engine as engine
		prepared = engine.DocumentSession.prepare_local_cdml_file_v1(absolute_path)
		session, observation, _origin_token, source_kind = prepared.take_admission_v1()
		if source_kind != "cdml":
			raise RuntimeError("local-CDML admission returned another source kind")
		return session, observation

	#============================================
	def _create_native_tab_from_admission(
			self, admission: tuple[object, object], title: str,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab:
		"""Create the sole Ferrum page type without repeating Rust observation."""
		session, observation = admission
		return (
			ferrum_qt.ferrum.document_tab.
			FerrumNativeDocumentTab.from_admitted_local_open(
				session, title, observation,
			)
		)

	#============================================
	def _native_tab_for_path(
			self, absolute_path: str,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None:
		"""Return a Ferrum page already loaded from the same canonical location."""
		candidate = os.path.normcase(os.path.realpath(absolute_path))
		for tab in self._native_tabs_by_page.values():
			if tab.file_path is None:
				continue
			loaded = os.path.normcase(os.path.realpath(os.path.abspath(tab.file_path)))
			if loaded == candidate:
				return tab
		return None

	#============================================
	def _on_save(self) -> bool:
		"""Save an active Ferrum page through the Ferrum document flow."""
		tab = self._active_native_tab()
		if tab is None:
			if getattr(self, "_neutral_native_shell", False):
				return False
			return super()._on_save()
		if tab.file_path is None:
			return self._prompt_native_save(tab, force_save_as=False)
		return self._save_native_tab_to_path(tab, str(tab.file_path))

	#============================================
	def _on_save_as(self) -> bool:
		"""Prompt for a new CDML destination for the selected Ferrum page."""
		tab = self._active_native_tab()
		if tab is None:
			if getattr(self, "_neutral_native_shell", False):
				return False
			return super()._on_save_as()
		return self._prompt_native_save(tab, force_save_as=True)

	#============================================
	def _prompt_native_save(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			force_save_as: bool,
			) -> bool:
		"""Choose a CDML destination while preserving the originally active tab."""
		if self._active_native_tab() is not tab:
			return False
		initial = "" if force_save_as or tab.file_path is None else str(tab.file_path)
		path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Save Drawing As"), initial,
			self.tr(_NATIVE_CDML_FILTER),
		)[0]
		if not path:
			return False
		if self._active_native_tab() is not tab:
			return False
		return self._save_native_tab_to_path(tab, path)

	#============================================
	def _save_native_tab_to_path(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			file_path: str,
			) -> bool:
		"""Publish only through the selected tab's Rust atomic-save boundary."""
		if self._active_native_tab() is not tab:
			return False
		path = pathlib.Path(file_path)
		if not path.suffix:
			path = path.with_suffix(".cdml")
		elif path.suffix.lower() != ".cdml":
			self._show_refusal(
				ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
					ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
					ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.UNSUPPORTED_SAVE_EXTENSION,
					path.name,
				),
			)
			return False
		absolute_path = os.path.abspath(str(path))
		if not self._native_save_destination_available(tab, absolute_path):
			return False
		try:
			publication = tab.save_atomic(absolute_path)
		except Exception as exc:
			return self._report_native_save_error(absolute_path, exc)
		if not publication.outcome.is_confirmed:
			self._show_refusal(
				ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
					ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
					ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_POSSIBLY_COMPLETED,
					path.name,
				),
			)
			return False
		index = self._tab_widget.indexOf(tab)
		if index >= 0:
			self._tab_widget.setTabText(index, tab.title)
			self._tab_widget.setTabToolTip(
				index, tab.local_document_source_description or "",
			)
		self.statusBar().showMessage(self.tr("Saved drawing: %s") % absolute_path, 3000)
		return True

	#============================================
	def _native_save_destination_available(
			self,
			tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
			absolute_path: str,
			) -> bool:
		"""Reject a destination already owned by another live tab."""
		existing_native = self._native_tab_for_path(absolute_path)
		if existing_native is not None and existing_native is not tab:
			self._show_refusal(
				ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
					ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
					ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_NOT_STARTED,
					pathlib.Path(absolute_path).name,
					"Another open tab already uses this destination.",
				),
			)
			return False
		return True

	#============================================
	def _report_native_save_error(self, absolute_path: str, exc: Exception) -> bool:
		"""Present save failure semantics without claiming a file state."""
		if type(exc) is (
				ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabSavePresentationError
			):
			self._show_refusal(ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
				ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_DISPLAY_FAILED,
				pathlib.Path(absolute_path).name,
				"The drawing was saved, but Ferrum could not update this tab. Keep the "
				"tab open and reopen the file to confirm the saved drawing.",
			))
			return False
		import ferrum_qt.ferrum.engine as engine
		if type(exc) is engine.PublicationPossiblyCompletedError:
			outcome = ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_POSSIBLY_COMPLETED
		elif type(exc) is engine.PublicationNotStartedError:
			outcome = ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_NOT_STARTED
		else:
			outcome = ferrum_qt.dialogs.refusal_presenter.RefusalOutcome.SAVE_POSSIBLY_COMPLETED
		self._show_refusal(
			ferrum_qt.dialogs.refusal_presenter.RefusalRequest(
				ferrum_qt.dialogs.refusal_presenter.RefusalTaskContext.SAVE_DOCUMENT,
				outcome, pathlib.Path(absolute_path).name, str(exc),
			),
		)
		return False

	#============================================
	def _active_native_tab(
			self,
			) -> ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab | None:
		"""Return the exact current Ferrum page."""
		return self._native_tabs_by_page.get(self._tab_widget.currentWidget())

	#============================================
	def _show_refusal(
			self, request: ferrum_qt.dialogs.refusal_presenter.RefusalRequest,
			) -> None:
		"""Forward the typed refusal intact to the product presentation seam."""
		self._show_edit_refusal(request)
