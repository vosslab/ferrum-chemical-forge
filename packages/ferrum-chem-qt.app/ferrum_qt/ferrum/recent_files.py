"""Personal Ferrum Recent Files state and File-menu projection."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.config.preferences


_RECENT_FILES_SCHEMA_VERSION = 1
_RECENT_FILES_CAPACITY = 12


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeRecentFilesV1:
	"""Versioned personal display paths retained outside every document contract."""

	paths: tuple[str, ...]

	#============================================
	def to_settings_value(self) -> dict[str, object]:
		"""Return the compact QSettings value for this one schema version."""
		return {
			"version": _RECENT_FILES_SCHEMA_VERSION,
			"paths": list(self.paths),
		}


#============================================
class FerrumNativeRecentFiles:
	"""Own ordinary-native recent paths and their small Qt menu client.

	Twelve entries give a useful working set without turning File into a long
	scrolling menu. This is presentation policy rather than a document contract.
	"""

	#============================================
	def __init__(
			self, window: object, preferences: object, capacity: int = _RECENT_FILES_CAPACITY,
			) -> None:
		"""Bind personal storage to one ordinary-native Qt window."""
		if type(preferences) is not ferrum_qt.config.preferences.Preferences:
			raise TypeError("Ferrum recent files require the shared Preferences owner")
		if type(capacity) is not int or capacity < 1:
			raise ValueError("Ferrum recent-file capacity must be a positive integer")
		self._window = window
		self._preferences = preferences
		self._capacity = capacity
		self._menu: PySide6.QtWidgets.QMenu | None = None

	#============================================
	def create_menu(self) -> PySide6.QtWidgets.QMenu:
		"""Create the dynamic menu; declarative YAML owns its placement."""
		menu = PySide6.QtWidgets.QMenu(self._window.tr("Recent Files"), self._window)
		menu.setToolTip(self._window.tr("Open a recently used Ferrum drawing."))
		menu.setStatusTip(self._window.tr("Open a recently used Ferrum drawing."))
		menu.aboutToShow.connect(self.rebuild_menu)
		self._menu = menu
		self.rebuild_menu()
		return menu

	#============================================
	def record_confirmed_path(self, path: str | pathlib.Path) -> None:
		"""Promote one path only after Ferrum installation or publication succeeded."""
		display_path = self._normalize_display_path(path)
		stored = self._load().paths
		key = self._display_key(display_path)
		paths = [item for item in stored if self._display_key(item) != key]
		paths.insert(0, display_path)
		self._store(FerrumNativeRecentFilesV1(tuple(paths[:self._capacity])))
		self.rebuild_menu()

	#============================================
	def remove_path(self, path: str | pathlib.Path) -> None:
		"""Remove one explicit stale path without altering any document state."""
		key = self._display_key(self._normalize_display_path(path))
		paths = tuple(item for item in self._load().paths if self._display_key(item) != key)
		self._store(FerrumNativeRecentFilesV1(paths))
		self.rebuild_menu()

	#============================================
	def clear(self) -> None:
		"""Clear only the personal QSettings list and report the visible result."""
		self._store(FerrumNativeRecentFilesV1(()))
		self.rebuild_menu()
		self._window.statusBar().showMessage(self._window.tr("Recent Files cleared."), 3000)

	#============================================
	def rebuild_menu(self) -> None:
		"""Project the latest personal paths into actions when the menu is used."""
		if self._menu is None:
			return
		self._menu.clear()
		paths = self._load().paths
		labels = self._labels(paths)
		for path, label in zip(paths, labels, strict=True):
			action = PySide6.QtGui.QAction(label, self._menu)
			description = self._window.tr("Open recent file: %s") % path
			action.setToolTip(path)
			action.setStatusTip(description)
			action.setWhatsThis(description)
			action.triggered.connect(
			lambda _checked=False, selected_path=path: self._open_recent(selected_path),
			)
			self._menu.addAction(action)
		if paths:
			self._menu.addSeparator()
			clear_action = PySide6.QtGui.QAction(self._window.tr("Clear Recent Files"), self._menu)
			clear_action.setToolTip(self._window.tr("Remove recently used paths from this application"))
			clear_action.triggered.connect(self.clear)
			self._menu.addAction(clear_action)

	#============================================
	def handle_failed_recent_open(self, path: str, failure: object) -> bool:
		"""Offer explicit removal after Rust confirms an unavailable recent source."""
		if not self._is_stale_source_failure(failure):
			return False
		guidance = self._stale_source_guidance(failure)
		message = self._window.tr(
			"Ferrum cannot use this recent file:\n%s\n\n"
			"%s\n\nKeep it if the file may become available again, or remove this stale entry.",
		) % (path, guidance)
		dialog = PySide6.QtWidgets.QMessageBox(
			PySide6.QtWidgets.QMessageBox.Icon.Warning,
			self._window.tr("File Not Available"), message,
			parent=self._window,
		)
		keep = dialog.addButton(self._window.tr("Keep"), PySide6.QtWidgets.QMessageBox.ButtonRole.AcceptRole)
		remove = dialog.addButton(
			self._window.tr("Remove from Recent Files"),
			PySide6.QtWidgets.QMessageBox.ButtonRole.DestructiveRole,
		)
		dialog.setDefaultButton(keep)
		dialog.exec()
		if dialog.clickedButton() is remove:
			self.remove_path(path)
		return True

	#============================================
	def _open_recent(self, path: str) -> None:
		"""Submit a recent selection through the immutable ordinary NewTab route."""
		self._window.open_recent_native_document_path(path)

	#============================================
	def _load(self) -> FerrumNativeRecentFilesV1:
		"""Read only the current versioned DTO; malformed personal state starts empty."""
		value = self._preferences.value(ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES)
		if type(value) is not dict or value.get("version") != _RECENT_FILES_SCHEMA_VERSION:
			return FerrumNativeRecentFilesV1(())
		stored_paths = value.get("paths")
		if type(stored_paths) is not list:
			return FerrumNativeRecentFilesV1(())
		paths: list[str] = []
		keys: set[str] = set()
		for stored_path in stored_paths:
			if type(stored_path) is not str or not stored_path:
				continue
			normalized = self._normalize_display_path(stored_path)
			key = self._display_key(normalized)
			if key not in keys:
				paths.append(normalized)
				keys.add(key)
		return FerrumNativeRecentFilesV1(tuple(paths[:self._capacity]))

	#============================================
	def _store(self, recent: FerrumNativeRecentFilesV1) -> None:
		"""Persist the whole versioned DTO as personal application state."""
		self._preferences.set_value(
			ferrum_qt.config.preferences.Preferences.KEY_RECENT_FILES,
			recent.to_settings_value(),
		)

	#============================================
	def _normalize_display_path(self, path: str | pathlib.Path) -> str:
		"""Use Qt-platform display spelling without resolving a source identity."""
		if isinstance(path, pathlib.Path):
			path = str(path)
		if type(path) is not str or not path:
			raise TypeError("Ferrum recent paths require a nonempty path string")
		normalized = PySide6.QtCore.QDir.cleanPath(
			PySide6.QtCore.QFileInfo(path).absoluteFilePath(),
		)
		return normalized

	#============================================
	def _display_key(self, display_path: str) -> str:
		"""Return the platform-appropriate lexical deduplication key."""
		return os.path.normcase(display_path)

	#============================================
	def _labels(self, paths: tuple[str, ...]) -> tuple[str, ...]:
		"""Add parent context only where a basename would be ambiguous."""
		basenames = [pathlib.Path(path).name for path in paths]
		labels: list[str] = []
		for path, basename in zip(paths, basenames, strict=True):
			if basenames.count(basename) == 1:
				labels.append(basename)
			else:
				labels.append(
					f"{basename} \N{EM DASH} {pathlib.Path(path).parent.name}",
				)
		return tuple(labels)

	#============================================
	def _is_stale_source_failure(self, failure: object) -> bool:
		"""Limit recovery to Rust-confirmed unavailable or nonregular paths."""
		return (
			getattr(failure, "error_type", None) == "DocumentInputError"
			and (
				getattr(failure, "stage", None) in {"read", "source_policy"}
				or getattr(failure, "category", None) == "source_rejected"
			)
		)

	#============================================
	def _stale_source_guidance(self, failure: object) -> str:
		"""Keep the typed Rust category visible in the single recovery flow."""
		if (
			getattr(failure, "stage", None) == "source_policy"
			or getattr(failure, "category", None) == "source_rejected"
		):
			return self._window.tr("Choose a regular, non-symlink local drawing file.")
		return self._window.tr("Ferrum could not read this path as a local drawing file.")
