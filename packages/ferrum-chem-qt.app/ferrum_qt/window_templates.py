"""Main application window for Ferrum-Qt."""

# Standard Library

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.config.geometry_units
import ferrum_qt.config.keybindings
import ferrum_qt.config.preferences
import ferrum_qt.widgets.status_bar
import ferrum_qt.widgets.zoom_controls
import ferrum_qt.widgets.icon_loader
import ferrum_qt.setup.canvas_setup
import ferrum_qt.setup.mode_setup
import ferrum_qt.setup.toolbar_setup
import ferrum_qt.actions.file_actions
import ferrum_qt.actions.options_actions
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.canvas.molecule_projection
import ferrum_qt.io.clipboard_manager
import ferrum_qt.io.import_capabilities
import ferrum_qt.io.user_template_catalog
import ferrum_qt.bridge.user_template_inspection
import ferrum_qt.dialogs.about_dialog
import ferrum_qt.dialogs.preferences_dialog
import ferrum_qt.dialogs.theme_chooser_dialog

import ferrum_qt.window_shared

_PendingSessionDeletion = ferrum_qt.window_shared._PendingSessionDeletion
ShutdownState = ferrum_qt.window_shared.ShutdownState


#============================================
class WindowTemplateMixin:
	"""Cohesive MainWindow behavior with no MainWindow import."""

	@property
	def user_template_catalog(self) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Return the immutable frontend-owned saved-template catalog snapshot."""
		return self._user_template_catalog
	def _scan_user_template_catalog(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Return one immutable scan of the explicitly configured directory."""
		if self._user_template_directory is None:
			return ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot((), ())
		return ferrum_qt.io.user_template_catalog.scan_user_template_catalog(
			self._user_template_directory,
		)
	def _show_user_template_catalog_status(
			self,
			snapshot: ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot,
			) -> None:
		"""Present one concise catalog outcome without hiding admitted neighbors."""
		if self._user_template_directory is None:
			self.statusBar().showMessage(self.tr("User template directory is not configured"), 3000)
			return
		if not snapshot.failures:
			self.statusBar().showMessage(
				self.tr("User templates refreshed: %d available") % len(snapshot.entries), 3000,
			)
			return
		first_failure = snapshot.failures[0]
		self.statusBar().showMessage(
			self.tr("User templates refreshed: %d available; skipped %s: %s") % (
				len(snapshot.entries), first_failure.source_name, first_failure.message,
			),
			5000,
		)
	def rescan_user_templates(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Replace the delivered saved-template catalog in every live session."""
		snapshot = self._scan_user_template_catalog()
		for session in tuple(self._sessions):
			if not session.is_disposed:
				session.replace_user_template_catalog(snapshot.entries)
		self._user_template_catalog = snapshot
		if self._active_mode_name() == "usertemplate":
			self._on_mode_changed("usertemplate")
		self._show_user_template_catalog_status(snapshot)
		return snapshot
	def refresh_user_templates(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Refresh saved templates through the visible File-action behavior."""
		return self._on_refresh_user_templates()
	def _on_refresh_user_templates(
			self,
			) -> ferrum_qt.io.user_template_catalog.UserTemplateCatalogSnapshot:
		"""Run one explicit catalog refresh and present all recoverable skips."""
		snapshot = self.rescan_user_templates()
		if snapshot.failures:
			details = "\n".join(
				"%s: %s" % (failure.source_name, failure.message)
				for failure in snapshot.failures
			)
			PySide6.QtWidgets.QMessageBox.information(
				self,
				self.tr("User Template Refresh"),
				self.tr("Some user templates were skipped.\n\n%s") % details,
			)
		return snapshot
