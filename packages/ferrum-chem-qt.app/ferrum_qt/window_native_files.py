"""Rust-only CDML open and publication behavior for native document tabs."""

# Standard Library
import os
import pathlib

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab


_NATIVE_CDML_FILTER = "Ferrum CDML (*.cdml);;All Files (*)"


#============================================
class WindowNativeFileMixin:
	"""Route CDML file actions to native tabs without touching legacy sessions."""

	_native_cdml_default_open_enabled = True

	#============================================
	def can_save_authoritatively(self) -> bool:
		"""Permit Save whenever the selected page is a live Rust-native tab."""
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
		"""Open an explicitly chosen CDML path through the Rust-native route."""
		absolute_path = os.path.abspath(file_path)
		if pathlib.Path(absolute_path).suffix.lower() != ".cdml":
			self._show_native_file_warning(
				"Unsupported File Format", "Ferrum CDML files must use the .cdml extension.",
			)
			return False
		return self._open_native_cdml(absolute_path, replace_current=False)

	#============================================
	def _open_native_cdml(self, absolute_path: str, replace_current: bool) -> bool:
		"""Load one complete UTF-8 CDML source into a fresh Rust-owned page."""
		existing = self._native_tab_for_path(absolute_path)
		if existing is not None:
			self._tab_widget.setCurrentIndex(self._tab_widget.indexOf(existing))
			return True
		if replace_current:
			self._show_native_file_warning(
				"Open in Current Tab Unavailable",
				"Ferrum CDML currently opens in a new Rust-native tab. "
				"Replacing a legacy or native tab is not available yet.",
			)
			return False
		try:
			with open(absolute_path, encoding="utf-8") as source:
				cdml = source.read()
			tab = self._create_native_tab(cdml, pathlib.Path(absolute_path).name)
			tab._adopt_loaded_origin_path(absolute_path)
		except Exception as exc:
			self._show_native_file_warning(
				"File Read Error", "Could not open %s:\n%s" % (absolute_path, exc),
			)
			return False
		try:
			self._register_native_tab(tab, activate=True)
		except Exception as exc:
			tab.dispose()
			self._show_native_file_warning(
				"File Read Error", "Could not open %s:\n%s" % (absolute_path, exc),
			)
			return False
		self.statusBar().showMessage(self.tr("Loaded Rust CDML: %s") % absolute_path, 3000)
		return True

	#============================================
	def _create_native_tab(
			self, cdml: str, title: str,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Create the sole production native CDML page type."""
		return ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
			cdml, title,
		)

	#============================================
	def _native_tab_for_path(
			self, absolute_path: str,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab | None:
		"""Return a native page already loaded from the same canonical location."""
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
		"""Save an active Rust-native page without entering the legacy session path."""
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
		"""Prompt for a new CDML destination for the selected Rust-native page."""
		tab = self._active_native_tab()
		if tab is None:
			if getattr(self, "_neutral_native_shell", False):
				return False
			return super()._on_save_as()
		return self._prompt_native_save(tab, force_save_as=True)

	#============================================
	def _prompt_native_save(
			self,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			force_save_as: bool,
			) -> bool:
		"""Choose a CDML destination while preserving the originally active tab."""
		if self._active_native_tab() is not tab:
			return False
		initial = "" if force_save_as or tab.file_path is None else str(tab.file_path)
		path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Save Rust CDML File As"), initial,
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
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			file_path: str,
			) -> bool:
		"""Publish only through the selected tab's Rust atomic-save boundary."""
		if self._active_native_tab() is not tab:
			return False
		path = pathlib.Path(file_path)
		if not path.suffix:
			path = path.with_suffix(".cdml")
		elif path.suffix.lower() != ".cdml":
			self._show_native_file_warning(
				"Unsupported Save Format",
				"Ferrum-Qt Rust documents must use the .cdml extension.",
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
			self._show_native_file_warning(
				"Save Durability Unconfirmed",
				"The Rust snapshot may be present at %s, but directory-entry "
				"durability is unconfirmed. The tab remains dirty." % absolute_path,
			)
			return False
		index = self._tab_widget.indexOf(tab)
		if index >= 0:
			self._tab_widget.setTabText(index, tab.title)
		self.statusBar().showMessage(self.tr("Saved Rust CDML: %s") % absolute_path, 3000)
		return True

	#============================================
	def _native_save_destination_available(
			self,
			tab: ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab,
			absolute_path: str,
			) -> bool:
		"""Reject a destination already owned by another live tab of either kind."""
		existing_native = self._native_tab_for_path(absolute_path)
		if existing_native is not None and existing_native is not tab:
			self._show_native_file_warning(
				"Save Destination Already Open",
				"Another Rust-native tab already owns %s. Save to a different CDML "
				"path or close that tab first." % absolute_path,
			)
			return False
		candidate = os.path.normcase(os.path.realpath(absolute_path))
		for session in getattr(self, "_sessions", ()):
			origin_path = session.origin_path
			if origin_path is None:
				continue
			legacy_path = os.path.normcase(
				os.path.realpath(os.path.abspath(origin_path)),
			)
			if legacy_path == candidate:
				self._show_native_file_warning(
					"Save Destination Already Open",
					"A legacy tab already owns %s. Save to a different CDML path or "
					"close that tab first." % absolute_path,
				)
				return False
		return True

	#============================================
	def _report_native_save_error(self, absolute_path: str, exc: Exception) -> bool:
		"""Present Rust publication failure semantics without claiming a save state."""
		if type(exc) is (
				ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTabSavePresentationError
			):
			self._show_native_file_warning(
				"Save Completed; Display Refresh Failed",
				"Rust completed publication to %s, but the new display could not be "
				"installed. The tab retains its prior visible state." % absolute_path,
			)
			return False
		import ferrum_chem
		if type(exc) is ferrum_chem.PublicationPossiblyCompletedError:
			title = "Save Possibly Completed"
			message = (
				"Rust could not confirm whether publication completed at %s. "
				"The tab remains open and its saved state is unchanged.\n\n%s"
			) % (absolute_path, exc)
		elif type(exc) is ferrum_chem.PublicationNotStartedError:
			title = "Save Not Started"
			message = "Rust did not start publication to %s.\n\n%s" % (absolute_path, exc)
		else:
			title = "Save Error"
			message = "Could not save %s:\n%s" % (absolute_path, exc)
		self._show_native_file_warning(title, message)
		return False

	#============================================
	def _active_native_tab(
			self,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab | None:
		"""Return the exact current native page, never a legacy session alias."""
		return self._native_tabs_by_page.get(self._tab_widget.currentWidget())

	#============================================
	def _show_native_file_warning(self, title: str, message: str) -> None:
		"""Show one user-facing native file failure with no legacy fallback."""
		PySide6.QtWidgets.QMessageBox.warning(self, self.tr(title), self.tr(message))
