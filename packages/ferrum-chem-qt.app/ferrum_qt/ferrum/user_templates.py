"""Rust-backed publication actions for authors' reusable templates."""

import dataclasses
import os
import pathlib

import PySide6.QtGui
import PySide6.QtWidgets

_TEMPLATE_FILTER = "Ferrum CDML Template (*.cdml)"


@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeUserTemplateSaveCapture:
	"""One opaque Rust receipt frozen before choosing a destination."""

	tab: object
	receipt: object


class FerrumNativeUserTemplateWindowMixin:
	"""Own publication only; Rust owns catalog scanning, identity, and placement."""

	def _initialize_native_user_templates(self, directory: str | pathlib.Path | None) -> None:
		if directory is not None and not isinstance(directory, (str, pathlib.Path)):
			raise TypeError("Ferrum user-template directory must be a path or None")
		self._user_template_directory = pathlib.Path(directory) if directory is not None else None

	def _build_native_user_template_file_actions(self) -> None:
		self._save_as_user_template_action = PySide6.QtGui.QAction(
			self.tr("Save Current as Template..."), self,
		)
		self._save_as_user_template_action.setToolTip(self.tr(
			"Publish the eligible Rust document, then refresh the template catalog",
		))
		self._save_as_user_template_action.triggered.connect(self._on_save_as_user_template)
		self._refresh_user_templates_action = PySide6.QtGui.QAction(self.tr("Refresh Templates"), self)
		self._refresh_user_templates_action.setToolTip(self.tr("Refresh the Rust-owned template catalog"))
		self._refresh_user_templates_action.triggered.connect(self._on_refresh_native_user_templates)
		for action_id, action in (
			("file.template.save_as", self._save_as_user_template_action),
			("file.template.refresh", self._refresh_user_templates_action),
		):
			self._action_registry.register_existing(
				action_id, action,
				shortcut_exemption_reason="Available by its labelled File menu client.",
			)

	def _on_refresh_native_user_templates(self) -> object | None:
		"""Request one replacement immutable catalog snapshot from Rust."""
		import ferrum_qt.ferrum.engine as engine
		try:
			snapshot = self._template_catalog_controller.refresh_snapshot()
		except engine.TemplateCatalogError as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				self._template_catalog_controller.error_message(error),
			))
			return None
		self.statusBar().showMessage(self.tr("Template catalog refreshed."), 3000)
		self._refresh_actions()
		return snapshot

	def _refresh_native_user_template_actions(
			self, active: bool, pending: bool, other_busy: bool,
			) -> None:
		configured = self._user_template_directory is not None
		self._save_as_user_template_action.setEnabled(
			configured and active and not pending and not other_busy,
		)
		self._refresh_user_templates_action.setEnabled(not other_busy)

	def _on_save_as_user_template(self) -> bool:
		capture = self._capture_native_user_template_save()
		if capture is None:
			return False
		try:
			self._user_template_directory.mkdir(parents=True, exist_ok=True)
		except OSError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum could not prepare the configured template directory.",
			))
			return False
		selected = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Save Current as Template"), str(self._user_template_directory),
			self.tr(_TEMPLATE_FILTER),
		)[0]
		if not selected:
			return False
		return self._publish_native_user_template_capture(capture, pathlib.Path(selected))

	def save_active_as_user_template_to_path(self, path: str | pathlib.Path) -> bool:
		capture = self._capture_native_user_template_save()
		if capture is None:
			return False
		return self._publish_native_user_template_capture(capture, pathlib.Path(path))

	def _capture_native_user_template_save(self) -> FerrumNativeUserTemplateSaveCapture | None:
		if self._user_template_directory is None:
			return None
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return None
		import ferrum_qt.ferrum.engine as engine
		try:
			receipt = tab.prepare_user_template_publication_v1()
		except engine.UserTemplatePublicationError as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				self._user_template_publication_refusal_message(error),
			))
			return None
		return FerrumNativeUserTemplateSaveCapture(tab, receipt)

	def _publish_native_user_template_capture(
			self, capture: FerrumNativeUserTemplateSaveCapture,
			selected: pathlib.Path,
			) -> bool:
		try:
			self._user_template_directory.mkdir(parents=True, exist_ok=True)
		except OSError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum could not prepare the configured template directory.",
			))
			return False
		directory = pathlib.Path(os.path.abspath(self._user_template_directory))
		candidate = pathlib.Path(os.path.abspath(selected))
		if candidate.suffix != ".cdml":
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum templates use the lowercase .cdml extension.",
			))
			return False
		if candidate.parent != directory:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Save templates directly in the configured Ferrum template directory.",
			))
			return False
		import ferrum_qt.ferrum.engine as engine
		try:
			publication = capture.tab.publish_user_template_v1(capture.receipt, str(candidate))
		except engine.RevisionConflictError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The active Rust document changed; choose Save Current as Template again.",
			))
			return False
		except engine.UserTemplatePublicationError as error:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				self._user_template_publication_refusal_message(error),
			))
			return False
		except engine.InvalidDestinationError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum could not use that template destination.",
			))
			return False
		except engine.PublicationNotStartedError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum could not publish the template. No completed save was confirmed.",
			))
			return False
		except engine.PublicationPossiblyCompletedError:
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Ferrum may have published the template. Inspect the destination before retrying.",
			))
			return False
		published = publication.published_snapshot
		if (
			published.revision != capture.receipt.revision
			or published.digest != capture.receipt.digest
			or not publication.outcome.is_confirmed
		):
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Inspect the destination before relying on this template.",
			))
			return False
		if self._on_refresh_native_user_templates() is not None:
			return True
		self._show_edit_refusal(self._unavailable_edit_refusal(
			"Template saved, but the catalog still needs refresh before it can be placed.",
		))
		return False

	def _user_template_publication_refusal_message(self, error: object) -> str:
		"""Map Rust's closed receipt refusal categories to actionable recovery text."""
		reason = getattr(error, "reason", "")
		if reason == "ineligible":
			return self.tr("This document is not eligible to become a reusable template.")
		if reason == "consumed":
			return self.tr("Choose Save Current as Template again before saving another copy.")
		if reason == "foreign_session":
			return self.tr("The template save request belongs to a different document tab.")
		return self.tr("Ferrum could not prepare this document as a reusable template.")
