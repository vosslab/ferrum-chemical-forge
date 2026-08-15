"""Rust-native catalog, publication, and placement for user templates."""

# Standard Library
import dataclasses
import os
import pathlib

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.bridge.user_template_inspection
import ferrum_qt.io.user_template_catalog


_TEMPLATE_FILTER = "Ferrum CDML Template (*.cdml)"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeUserTemplatePlacementIntent:
	"""One catalog plan and exact native-tab provenance awaiting a scene click."""

	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	catalog_key: str
	prepared: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeUserTemplateSaveCapture:
	"""One eligible backend snapshot frozen before choosing a destination."""

	tab: object
	revision: int
	digest: str
	cdml: str


#============================================
class FerrumNativeUserTemplateTabMixin:
	"""Commit one exact prepared template through the tab's Rust session."""

	#============================================
	def insert_user_template(self, prepared: object, x: float, y: float) -> object:
		"""Place one Rust-admitted molecule and install its authoritative observation."""
		self._require_mutable()
		import ferrum_chem
		if type(prepared) is not ferrum_chem.DocumentUserTemplatePlanV1:
			raise TypeError("native user-template insertion requires an exact Ferrum plan")
		if type(x) is not float or type(y) is not float:
			raise TypeError("native user-template anchor coordinates must be floats")
		snapshot = self.current_snapshot
		result = self._session.apply_user_template_v1(
			snapshot.revision, snapshot.digest, prepared, x, y,
		)
		self._install_mutation_result(result.operation)
		return result


#============================================
class FerrumNativeUserTemplateWindowMixin:
	"""Own the native template catalog, actions, one-click intent, and save flow."""

	#============================================
	def _initialize_native_user_templates(
			self, directory: str | pathlib.Path | None) -> None:
		"""Install one explicit application-owned catalog without creating it."""
		if directory is not None and not isinstance(directory, (str, pathlib.Path)):
			raise TypeError("native user-template directory must be a path or None")
		self._user_template_directory = (
			pathlib.Path(directory) if directory is not None else None
		)
		self._user_template_placement_intent: (
			FerrumNativeUserTemplatePlacementIntent | None
		) = None
		self._user_template_catalog = self._scan_native_user_template_catalog()

	#============================================
	@property
	def user_template_catalog(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Return the current immutable application-owned catalog projection."""
		return self._user_template_catalog

	#============================================
	def _scan_native_user_template_catalog(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Scan only the configured directory through secure Rust-backed admission."""
		if self._user_template_directory is None:
			return ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot((), ())
		return ferrum_qt.io.user_template_catalog.scan_user_template_catalog(
			self._user_template_directory,
		)

	#============================================
	def _build_native_user_template_file_actions(
			self, file_menu: PySide6.QtWidgets.QMenu) -> None:
		"""Expose template publication and explicit catalog refresh in File."""
		self._save_as_user_template_action = PySide6.QtGui.QAction(
			self.tr("Save As User Template..."), self,
		)
		self._save_as_user_template_action.setToolTip(self.tr(
			"Publish one eligible Rust document into the Ferrum template catalog",
		))
		self._save_as_user_template_action.triggered.connect(
			self._on_save_as_user_template,
		)
		file_menu.addAction(self._save_as_user_template_action)
		self._refresh_user_templates_action = PySide6.QtGui.QAction(
			self.tr("Refresh User Templates"), self,
		)
		self._refresh_user_templates_action.triggered.connect(
			self._on_refresh_native_user_templates,
		)
		file_menu.addAction(self._refresh_user_templates_action)

	#============================================
	def _build_native_user_template_place_action(
			self, chemistry_menu: PySide6.QtWidgets.QMenu) -> None:
		"""Expose one checked placement action with an explicit catalog choice."""
		self._place_user_template_action = PySide6.QtGui.QAction(
			self.tr("Place User Template..."), self,
		)
		self._place_user_template_action.setCheckable(True)
		self._place_user_template_action.setToolTip(self.tr(
			"Choose reusable chemical content, then click once to place its atom centroid",
		))
		self._place_user_template_action.triggered.connect(
			self._on_place_user_template,
		)
		chemistry_menu.addAction(self._place_user_template_action)

	#============================================
	def refresh_user_templates(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Rescan the configured catalog and report every skipped neighbor."""
		return self._on_refresh_native_user_templates()

	#============================================
	def _on_refresh_native_user_templates(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Replace one catalog projection while keeping admission failures visible."""
		self._cancel_user_template_placement(clear_status=False)
		snapshot = self._scan_native_user_template_catalog()
		self._user_template_catalog = snapshot
		self._show_native_user_template_catalog_status(snapshot)
		if snapshot.failures:
			details = "\n".join(
				"%s: %s" % (failure.source_name, failure.message)
				for failure in snapshot.failures
			)
			PySide6.QtWidgets.QMessageBox.information(
				self, self.tr("User Template Refresh"),
				self.tr("Some user templates were skipped.\n\n%s") % details,
			)
		self._refresh_actions()
		return snapshot

	#============================================
	def _show_native_user_template_catalog_status(
			self,
			snapshot: ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot,
			) -> None:
		"""Present available chemistry content and the first recoverable failure."""
		if self._user_template_directory is None:
			self.statusBar().showMessage(
				self.tr("User template directory is not configured"), 3000,
			)
			return
		if not snapshot.failures:
			self.statusBar().showMessage(
				self.tr("User templates refreshed: %d available") % len(snapshot.entries),
				3000,
			)
			return
		failure = snapshot.failures[0]
		self.statusBar().showMessage(
			self.tr("User templates refreshed: %d available; skipped %s: %s") % (
				len(snapshot.entries), failure.source_name, failure.message,
			),
			5000,
		)

	#============================================
	def _on_place_user_template(self, checked: bool) -> None:
		"""Choose one current entry and capture a single-click placement intent."""
		if not checked:
			self._cancel_user_template_placement()
			return
		entries = self._user_template_catalog.entries
		if not entries:
			self._cancel_user_template_placement()
			self._show_native_file_warning(
				"No User Templates",
				"Save or add an eligible .cdml template, then refresh the catalog.",
			)
			return
		labels = self._template_choice_labels(entries)
		selected, accepted = PySide6.QtWidgets.QInputDialog.getItem(
			self, self.tr("Place User Template"), self.tr("Template:"),
			labels, 0, False,
		)
		if not accepted:
			self._cancel_user_template_placement()
			return
		entry = entries[labels.index(selected)]
		if not self.start_user_template_placement(entry.catalog_key):
			self._cancel_user_template_placement()

	#============================================
	def _template_choice_labels(self, entries: tuple[object, ...]) -> tuple[str, ...]:
		"""Return readable unique choices without changing stored display names."""
		counts = {}
		for entry in entries:
			counts[entry.label] = counts.get(entry.label, 0) + 1
		labels = []
		for entry in entries:
			if counts[entry.label] == 1:
				labels.append(entry.label)
			else:
				source_name = entry.source_name or entry.catalog_key
				labels.append("%s - %s" % (entry.label, source_name))
		return tuple(labels)

	#============================================
	def start_user_template_placement(self, catalog_key: str) -> bool:
		"""Capture one exact catalog plan and active-tab provenance for one click."""
		if type(catalog_key) is not str or not catalog_key:
			raise TypeError("native user-template placement requires a catalog key")
		if self._user_template_placement_intent is not None:
			return False
		if self._native_user_template_other_work_busy():
			return False
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		entry = next(
			(entry for entry in self._user_template_catalog.entries
			 if entry.catalog_key == catalog_key),
			None,
		)
		if entry is None:
			return False
		prepared = entry.native_plan
		if prepared is None:
			prepared = ferrum_qt.bridge.user_template_inspection.prepare_user_template(
				entry.template_cdml,
			)
		import ferrum_chem
		if type(prepared) is not ferrum_chem.DocumentUserTemplatePlanV1:
			raise TypeError("catalog entry does not contain an exact Ferrum template plan")
		self._cancel_atom_insertion()
		self._cancel_line_gesture()
		snapshot = tab.current_snapshot
		viewport = tab.view.viewport()
		self._user_template_placement_intent = FerrumNativeUserTemplatePlacementIntent(
			tab, viewport, snapshot.revision, snapshot.digest, catalog_key, prepared,
		)
		viewport.installEventFilter(self)
		viewport.setCursor(PySide6.QtCore.Qt.CursorShape.CrossCursor)
		viewport.setFocus()
		self._place_user_template_action.setChecked(True)
		self.statusBar().showMessage(self.tr(
			"Click once to place the template molecule; press Esc to cancel.",
		))
		self._refresh_actions()
		return True

	#============================================
	def _native_user_template_other_work_busy(self) -> bool:
		"""Return whether another native workflow currently owns interaction."""
		return (
			self._molecule_import_busy()
			or self._molecule_export_busy()
			or self._molecule_inspection_busy()
			or self._clipboard_busy()
			or self._coordinate_generation_intent is not None
			or self._atom_insertion_intent is not None
			or self._line_gesture_intent is not None
		)

	#============================================
	def eventFilter(self, watched: PySide6.QtCore.QObject,
			event: PySide6.QtCore.QEvent) -> bool:
		"""Capture the template placement click before other native gesture owners."""
		intent = self._user_template_placement_intent
		if intent is None or watched is not intent.viewport:
			return super().eventFilter(watched, event)
		if event.type() == PySide6.QtCore.QEvent.Type.KeyPress:
			if event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
				self._cancel_user_template_placement()
				return True
			return False
		if event.type() != PySide6.QtCore.QEvent.Type.MouseButtonPress:
			return False
		if event.button() != PySide6.QtCore.Qt.MouseButton.LeftButton:
			return False
		self._complete_user_template_placement(event)
		return True

	#============================================
	def _complete_user_template_placement(self, event: PySide6.QtGui.QMouseEvent) -> None:
		"""Reauthenticate the exact intent and submit one finite scene anchor."""
		intent = self._user_template_placement_intent
		if intent is None:
			return
		if not self._user_template_placement_is_current(intent):
			self._cancel_user_template_placement()
			self._show_native_file_warning(
				"User Template Placement Stale",
				"The document or catalog changed; choose the template again.",
			)
			return
		try:
			point = intent.tab.view.snap_authored_scene_point(
				intent.tab.view.mapToScene(event.position().toPoint()),
			)
			self._cancel_user_template_placement(clear_status=False)
			intent.tab.insert_user_template(
				intent.prepared, float(point.x()), float(point.y()),
			)
		except Exception as error:
			self._cancel_user_template_placement(clear_status=False)
			self._show_native_file_warning("User Template Placement Failed", str(error))
			self._refresh_actions()
			return
		self.statusBar().showMessage(self.tr("Placed one Rust-native user template."), 5000)
		self._refresh_actions()

	#============================================
	def _user_template_placement_is_current(
			self, intent: FerrumNativeUserTemplatePlacementIntent) -> bool:
		"""Require current tab, backend provenance, and catalog plan identity."""
		if self._active_native_tab() is not intent.tab:
			return False
		if self._native_tabs_by_page.get(intent.tab) is not intent.tab:
			return False
		try:
			snapshot = intent.tab.current_snapshot
		except Exception:
			return False
		if snapshot.revision != intent.revision or snapshot.digest != intent.digest:
			return False
		return any(
			entry.catalog_key == intent.catalog_key
			for entry in self._user_template_catalog.entries
		)

	#============================================
	def _cancel_user_template_placement(self, clear_status: bool = True) -> None:
		"""Release one pending click intent without mutating the document."""
		intent = self._user_template_placement_intent
		self._user_template_placement_intent = None
		if intent is not None:
			intent.viewport.removeEventFilter(self)
			intent.viewport.unsetCursor()
		if hasattr(self, "_place_user_template_action"):
			self._place_user_template_action.setChecked(False)
		if clear_status:
			self.statusBar().clearMessage()

	#============================================
	def _user_template_placement_blocks_tab_close(self, tab: object) -> bool:
		"""Cancel an intent for the closing tab and require a fresh close request."""
		intent = self._user_template_placement_intent
		if intent is None or intent.tab is not tab:
			return False
		self._cancel_user_template_placement()
		return True

	#============================================
	def _refresh_native_user_template_actions(
			self, active: bool, pending: bool, other_busy: bool) -> None:
		"""Keep save, refresh, and one-click placement honest about current state."""
		configured = self._user_template_directory is not None
		placing = self._user_template_placement_intent is not None
		self._save_as_user_template_action.setEnabled(
			configured and active and not pending and not other_busy and not placing,
		)
		self._refresh_user_templates_action.setEnabled(
			configured and not other_busy and not placing,
		)
		self._place_user_template_action.setEnabled(
			active and not pending and not other_busy and (placing or bool(
				self._user_template_catalog.entries
			)),
		)
		self._place_user_template_action.setChecked(placing)

	#============================================
	def _on_save_as_user_template(self) -> bool:
		"""Validate one snapshot, choose a direct catalog child, and publish it."""
		capture = self._capture_native_user_template_save()
		if capture is None:
			return False
		try:
			self._user_template_directory.mkdir(parents=True, exist_ok=True)
		except OSError as error:
			self._show_native_file_warning("User Template Directory Failed", str(error))
			return False
		selected = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Save As User Template"),
			str(self._user_template_directory), self.tr(_TEMPLATE_FILTER),
		)[0]
		if not selected:
			return False
		return self._publish_native_user_template_capture(capture, pathlib.Path(selected))

	#============================================
	def save_active_as_user_template_to_path(self, path: str | pathlib.Path) -> bool:
		"""Publish an eligible active snapshot to one explicit direct catalog child."""
		capture = self._capture_native_user_template_save()
		if capture is None:
			return False
		return self._publish_native_user_template_capture(capture, pathlib.Path(path))

	#============================================
	def _capture_native_user_template_save(
			self,
			) -> FerrumNativeUserTemplateSaveCapture | None:
		"""Freeze one current eligible backend snapshot before path selection."""
		if self._user_template_directory is None:
			return None
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return None
		try:
			snapshot = tab.backend_snapshot_for_recovery_export()
			ferrum_qt.bridge.user_template_inspection.prepare_user_template(snapshot.cdml)
		except Exception as error:
			self._show_native_file_warning("Document Is Not a User Template", str(error))
			return None
		return FerrumNativeUserTemplateSaveCapture(
			tab, snapshot.revision, snapshot.digest, snapshot.cdml,
		)

	#============================================
	def _publish_native_user_template_capture(
			self, capture: FerrumNativeUserTemplateSaveCapture,
			selected: pathlib.Path) -> bool:
		"""Reauthenticate and safely publish one frozen eligible snapshot."""
		try:
			self._user_template_directory.mkdir(parents=True, exist_ok=True)
		except OSError as error:
			self._show_native_file_warning("User Template Directory Failed", str(error))
			return False
		directory = pathlib.Path(os.path.abspath(self._user_template_directory))
		candidate = pathlib.Path(os.path.abspath(selected))
		if candidate.suffix != ".cdml":
			self._show_native_file_warning(
				"User Template Destination Rejected",
				"Ferrum user templates use the lowercase .cdml extension.",
			)
			return False
		if candidate.parent != directory:
			self._show_native_file_warning(
				"User Template Destination Rejected",
				"Save templates directly in the configured Ferrum template directory.",
			)
			return False
		if not self._native_user_template_save_is_current(capture):
			self._show_native_file_warning(
				"User Template Save Stale",
				"The active Rust document changed; choose Save As User Template again.",
			)
			return False
		try:
			publication = capture.tab.recovery_export(candidate, capture.revision)
		except Exception as error:
			self._show_native_file_warning("User Template Publication Failed", str(error))
			return False
		published = publication.published_snapshot
		if (
			published.revision != capture.revision
			or published.digest != capture.digest
			or not publication.outcome.is_confirmed
		):
			self._show_native_file_warning(
				"User Template Durability Unconfirmed",
				"Inspect the destination before relying on this template.",
			)
			return False
		self._user_template_catalog = self._scan_native_user_template_catalog()
		self._show_native_user_template_catalog_status(self._user_template_catalog)
		self._refresh_actions()
		return True

	#============================================
	def _native_user_template_save_is_current(
			self, capture: FerrumNativeUserTemplateSaveCapture) -> bool:
		"""Require the same registered active backend snapshot selected for save."""
		if self._active_native_tab() is not capture.tab:
			return False
		if self._native_tabs_by_page.get(capture.tab) is not capture.tab:
			return False
		try:
			snapshot = capture.tab.backend_snapshot_for_recovery_export()
		except Exception:
			return False
		return (
			snapshot.revision == capture.revision
			and snapshot.digest == capture.digest
			and snapshot.cdml == capture.cdml
		)
