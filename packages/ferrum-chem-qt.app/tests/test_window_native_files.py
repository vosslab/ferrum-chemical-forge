"""Behavioral coverage for the isolated Rust-native CDML file route."""

# Standard Library
import dataclasses
import os
import pathlib


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.window_native_files


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	"""Small immutable snapshot used only by this route's owned-value seam."""

	revision: int
	digest: str
	is_dirty: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _DocumentObservation:
	"""Fixture document envelope with one snapshot."""

	snapshot: _Snapshot


@dataclasses.dataclass(frozen=True, slots=True)
class _RenderObservation:
	"""Fixture render envelope with durable snapshot provenance."""

	document: _DocumentObservation


@dataclasses.dataclass(frozen=True, slots=True)
class _Outcome:
	"""Fixture publication confirmation fact."""

	is_confirmed: bool


@dataclasses.dataclass(frozen=True, slots=True)
class _Publication:
	"""Fixture publication result."""

	snapshot: _Snapshot
	outcome: _Outcome


class _Session:
	"""One native-session seam with intentional save outcome control."""

	#============================================
	def __init__(self, current: _Snapshot, saved: _Snapshot,
			confirmed: bool) -> None:
		"""Retain the current and publication snapshots."""
		self._current = current
		self._saved = saved
		self._confirmed = confirmed
		self._published = False

	#============================================
	def snapshot(self) -> _Snapshot:
		"""Return the current authoritative fixture snapshot."""
		return self._current

	#============================================
	def observe_render(self, revision: int) -> _RenderObservation:
		"""Return an observation only for the requested current revision."""
		snapshot = self._saved if self._published else self._current
		if revision != snapshot.revision:
			raise ValueError("unexpected native fixture revision")
		return _RenderObservation(_DocumentObservation(snapshot))

	#============================================
	def save_atomic(self, _path: object, revision: int) -> _Publication:
		"""Publish only the snapshot that the tab actually observed."""
		if revision != self._current.revision:
			raise ValueError("unexpected native fixture save revision")
		self._published = self._confirmed
		return _Publication(self._saved, _Outcome(self._confirmed))


class _Controller:
	"""Small accepting projection owner for the exact tab fixture seam."""

	#============================================
	def __init__(self, acceptances: tuple[bool, ...] = (True, True)) -> None:
		"""Create one accepting projection generation."""
		self.generation = 0
		self.disposed = False
		self._acceptances = iter(acceptances)

	#============================================
	def replace(self, observation: _RenderObservation, latch: object) -> bool:
		"""Accept only a matching current observation."""
		matching = (
			not self.disposed
			and latch.generation == self.generation
			and latch.revision == observation.document.snapshot.revision
			and latch.digest == observation.document.snapshot.digest
		)
		if not matching:
			return False
		return next(self._acceptances)

	#============================================
	def dispose(self) -> None:
		"""Record projection retirement."""
		self.disposed = True


class _StatusBar:
	"""Collect semantic status updates without constructing a legacy window."""

	#============================================
	def __init__(self) -> None:
		"""Start with no reported message."""
		self.message = ""

	#============================================
	def showMessage(self, message: str, _timeout: int) -> None:
		"""Retain the user-visible status message."""
		self.message = message


class _NativeFileHost(ferrum_qt.window_native_files.WindowNativeFileMixin):
	"""Minimal isolated host exposing only the native file controller contract."""

	#============================================
	def __init__(self) -> None:
		"""Own a native-page registry and a common Qt tab widget."""
		self._tab_widget = PySide6.QtWidgets.QTabWidget()
		self._native_tabs_by_page = {}
		self._status_bar = _StatusBar()
		self.warnings = []
		self.loaded_paths = []
		self.next_confirmed = True
		self.next_replacements = (True, True)

	#============================================
	def tr(self, text: str) -> str:
		"""Use stable English strings in this isolated controller proof."""
		return text

	#============================================
	def statusBar(self) -> _StatusBar:
		"""Return the host-owned status surface."""
		return self._status_bar

	#============================================
	def _prepare_local_cdml_admission(
			self, absolute_path: str,
			) -> tuple[_Session, object]:
		"""Record one profile-owned compatibility admission without reading in Python."""
		self.loaded_paths.append(absolute_path)
		current = _Snapshot(9, "a" * 64, True)
		saved = _Snapshot(9, "b" * 64, False)
		return _Session(current, saved, self.next_confirmed), object()

	#============================================
	def _create_native_tab_from_admission(
			self, admission: tuple[object, object], title: str,
			) -> ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
		"""Use the exact tab type through its explicitly private fixture seam."""
		session, _observation = admission
		tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab._from_fixture(
			title, session, _Controller(self.next_replacements),
		)
		return tab

	#============================================
	def _register_native_tab(self, tab: object, *, activate: bool) -> object:
		"""Install one exact page using the minimal common-tab contract."""
		if type(tab) is not ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab:
			raise TypeError("native file route created the wrong tab type")
		index = self._tab_widget.addTab(tab, tab.title)
		self._native_tabs_by_page[tab] = tab
		if activate:
			self._tab_widget.setCurrentIndex(index)
		return tab

	#============================================
	def _show_native_file_warning(self, title: str, message: str) -> None:
		"""Record failure text instead of opening a modal dialog in the test."""
		self.warnings.append((title, message))


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide an offscreen application without importing the legacy host."""
	app = PySide6.QtWidgets.QApplication.instance()
	if app is None:
		app = PySide6.QtWidgets.QApplication([])
	return app


#============================================
def test_native_cdml_open_uses_one_profile_admission_and_sets_origin(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A CDML source becomes one Rust-native tab with its true origin location."""
	del qapp
	source = tmp_path / "ethanol.cdml"
	cdml = "<svg><molecule id=\"m1\"><opaque-root keep=\"yes\"/></molecule></svg>"
	source.write_text(cdml, encoding="utf-8")
	host = _NativeFileHost()

	assert host.open_file_path(str(source))
	tab = host._active_native_tab()
	assert tab is not None and tab.file_path == source and tab.title == source.name
	assert host.loaded_paths == [str(source.resolve())]
	assert host._status_bar.message.endswith(str(source))
	tab.dispose()


#============================================
def test_native_cdml_duplicate_activates_existing_page_without_reloading(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The canonical source path owns one page instead of a duplicate Rust session."""
	del qapp
	source = tmp_path / "one.cdml"
	source.write_text("<svg/>", encoding="utf-8")
	host = _NativeFileHost()
	assert host.open_file_path(str(source))
	first = host._active_native_tab()
	assert host.open_file_path(str(source))
	assert host._active_native_tab() is first
	assert host.loaded_paths == [str(source.resolve())]
	first.dispose()


#============================================
def test_native_cdml_refuses_same_tab_replacement_before_reading_source(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""The clean boundary rejects ownership replacement instead of bridging sessions."""
	del qapp
	source = tmp_path / "replacement.cdml"
	source.write_text("<svg/>", encoding="utf-8")
	host = _NativeFileHost()
	assert not host.open_file_path(str(source), replace_current=True)
	assert not host.loaded_paths
	assert host.warnings[-1][0] == "Open in Current Tab Unavailable"


#============================================
def test_native_save_confirms_title_and_clean_state_only_after_rust_publication(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Confirmed Save updates the tab from Rust while an unconfirmed save does not."""
	del qapp
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	first.write_text("<svg/>", encoding="utf-8")
	host = _NativeFileHost()
	assert host.open_file_path(str(first))
	tab = host._active_native_tab()
	assert tab is not None
	assert host._save_native_tab_to_path(tab, str(second))
	assert tab.file_path == second and tab.title == second.name and not tab.is_dirty
	assert host._tab_widget.tabText(host._tab_widget.indexOf(tab)) == second.name
	tab.dispose()


#============================================
def test_native_save_reports_unconfirmed_directory_entry_without_changing_tab_truth(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Directory-entry uncertainty leaves the Rust tab dirty at its known origin."""
	del qapp
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	first.write_text("<svg/>", encoding="utf-8")
	host = _NativeFileHost()
	host.next_confirmed = False
	assert host.open_file_path(str(first))
	tab = host._active_native_tab()
	assert tab is not None
	assert not host._save_native_tab_to_path(tab, str(second))
	assert tab.file_path == first and tab.is_dirty
	assert host.warnings[-1][0] == "Save Durability Unconfirmed"
	tab.dispose()


#============================================
def test_native_save_reports_completed_publication_when_display_refresh_fails(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""A confirmed Rust save never becomes a false generic failure after painting fails."""
	del qapp
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	first.write_text("<svg/>", encoding="utf-8")
	host = _NativeFileHost()
	host.next_replacements = (True, False)
	assert host.open_file_path(str(first))
	tab = host._active_native_tab()
	assert tab is not None
	assert not host._save_native_tab_to_path(tab, str(second))
	assert tab.file_path == first and tab.is_dirty
	assert host.warnings[-1][0] == "Save Completed; Display Refresh Failed"
	tab.dispose()


#============================================
def test_native_save_rejects_a_symlink_alias_owned_by_another_native_tab(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Canonical aliases cannot let two Rust-native tabs claim one CDML destination."""
	del qapp
	first = tmp_path / "first.cdml"
	second = tmp_path / "second.cdml"
	alias = tmp_path / "first-alias.cdml"
	first.write_text("<svg/>", encoding="utf-8")
	second.write_text("<svg/>", encoding="utf-8")
	alias.symlink_to(first)
	host = _NativeFileHost()
	assert host.open_file_path(str(first))
	first_tab = host._active_native_tab()
	assert host.open_file_path(str(second))
	second_tab = host._active_native_tab()
	assert first_tab is not None and second_tab is not None
	assert not host._save_native_tab_to_path(second_tab, str(alias))
	assert second_tab.file_path == second and second_tab.is_dirty
	assert host.warnings[-1][0] == "Save Destination Already Open"
	first_tab.dispose()
	second_tab.dispose()
