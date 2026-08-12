"""Main application window for Ferrum-Qt."""

# Standard Library
import os
import pathlib

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
class WindowFileMixin:
	"""Cohesive MainWindow behavior with no MainWindow import."""

	def _registered_recovery_export_session(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			*, require_active: bool,
			) -> ferrum_qt.models.document_session.DocumentSession | None:
		"""Return one still-registered exportable session without retargeting it."""
		if require_active and self._active_session is not session:
			return None
		if session.is_disposed or not any(item is session for item in self._sessions):
			return None
		if not session.can_recovery_export:
			return None
		return session
	def _active_recovery_export_session(
			self,
			) -> ferrum_qt.models.document_session.DocumentSession | None:
		"""Return the exact active registered session eligible for Recovery Export."""
		session = self._active_session
		if session is None:
			return None
		return self._registered_recovery_export_session(session, require_active=True)
	def can_recovery_export(self) -> bool:
		"""Return the total File-action predicate for Recovery Export."""
		return self._active_recovery_export_session() is not None
	def can_save_authoritatively(self) -> bool:
		"""Return whether the active document may use ordinary Save or Save As."""
		session = self._active_session
		return bool(
			session is not None
			and session in self._sessions
			and session.can_write_authoritative_snapshot
		)
	def can_save_as_template(self) -> bool:
		"""Return whether Template export has one current backend snapshot to publish."""
		return self._user_template_directory is not None and self.can_recovery_export()
	def can_refresh_user_templates(self) -> bool:
		"""Return whether this window has an explicit user-template directory."""
		return self._user_template_directory is not None
	def _export_captured_backend_snapshot(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			*, require_active: bool, dialog_title: str,
			) -> bool:
		"""Prompt and publish only the session captured before the prompt."""
		file_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self,
			self.tr(dialog_title),
			"",
			self.tr(ferrum_qt.actions.file_actions.CDML_FILTER),
		)[0]
		if not file_path:
			return False
		path = pathlib.Path(file_path)
		if not path.suffix:
			path = path.with_suffix(".cdml")
		elif path.suffix.lower() != ".cdml":
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Unsupported Recovery Export Format"),
				self.tr("Recovery Export writes Ferrum CDML files with a .cdml extension."),
			)
			return False
		captured = self._registered_recovery_export_session(
			session, require_active=require_active,
		)
		if captured is not session:
			return False
		absolute_path = os.path.abspath(str(path))
		try:
			session.export_backend_snapshot(absolute_path)
		except ferrum_qt.models.document_session.BackendSnapshotPublicationError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Recovery Export Durability Unconfirmed"),
				self.tr(
					"The exact canonical snapshot may be present at %s, but export "
					"durability is unconfirmed. No session state changed; the tab "
					"remains open.\n\n%s"
				) % (absolute_path, exc),
			)
			return False
		except Exception as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Recovery Export Error"),
				self.tr("Could not export backend CDML to %s:\n%s") % (
					absolute_path, exc,
				),
			)
			return False
		self.statusBar().showMessage(
			self.tr("Backend snapshot exported: %s") % absolute_path,
			3000,
		)
		return True
	def _recovery_export_close_choice(self, message: str) -> str:
		"""Run the Recovery Export close prompt and dispose it before returning."""
		dialog = PySide6.QtWidgets.QMessageBox(self)
		try:
			dialog.setWindowTitle(self.tr("Unsaved Backend Changes"))
			dialog.setText(message)
			export_button = dialog.addButton(
				self.tr("Recovery Export"),
				PySide6.QtWidgets.QMessageBox.ButtonRole.ActionRole,
			)
			discard_button = dialog.addButton(
				PySide6.QtWidgets.QMessageBox.StandardButton.Discard,
			)
			dialog.addButton(PySide6.QtWidgets.QMessageBox.StandardButton.Cancel)
			dialog.exec()
			if dialog.clickedButton() is export_button:
				choice = "export"
			elif dialog.clickedButton() is discard_button:
				choice = "discard"
			else:
				choice = "cancel"
			return choice
		finally:
			dialog.deleteLater()
	def _on_recovery_export(self) -> bool:
		"""Export only the exact active backend session captured before the dialog."""
		session = self._active_recovery_export_session()
		if session is None:
			return False
		return self._export_captured_backend_snapshot(
			session, require_active=True, dialog_title="Recovery Export Backend CDML",
		)
	def _confirm_recovery_export_or_discard(
			self, operation: str,
			session: ferrum_qt.models.document_session.DocumentSession,
			state: ferrum_qt.models.document_session.CloseState,
			) -> bool:
		"""Offer Recovery Export only when ordinary authoritative Save is unsafe."""
		message = self.tr(
			"The current backend document cannot be saved authoritatively before %s."
		) % operation
		if state.legacy_local_pending:
			message += "\n\n" + self.tr(
				"Recovery Export saves the backend document only; Qt-local edits are excluded.",
			)
		elif state.backend_unseen:
			message += "\n\n" + self.tr(
				"The saved backend document cannot currently be shown in the Qt projection.",
			)
		choice = self._recovery_export_close_choice(message)
		if choice == "export":
			return self._export_captured_backend_snapshot(
				session, require_active=False, dialog_title="Recovery Export Backend CDML",
			)
		if choice == "discard":
			return True
		return False
	def _confirm_save_if_dirty(
			self, operation: str,
			session: ferrum_qt.models.document_session.DocumentSession | None = None,
			) -> bool:
		"""Return whether a destructive operation may continue for one tab."""
		target = session if session is not None else self._active_session
		if target is None:
			return True
		if target.is_disposed:
			return False
		try:
			state = target.close_state()
		except RuntimeError:
			return False
		if not state.needs_confirmation:
			return True
		if state.uses_recovery_export:
			return self._confirm_recovery_export_or_discard(operation, target, state)
		reply = PySide6.QtWidgets.QMessageBox.question(
			self,
			self.tr("Unsaved Changes"),
			self.tr("Save changes before %s?") % operation,
			(PySide6.QtWidgets.QMessageBox.StandardButton.Save
				| PySide6.QtWidgets.QMessageBox.StandardButton.Discard
				| PySide6.QtWidgets.QMessageBox.StandardButton.Cancel),
			PySide6.QtWidgets.QMessageBox.StandardButton.Save,
		)
		if reply == PySide6.QtWidgets.QMessageBox.StandardButton.Cancel:
			return False
		if reply == PySide6.QtWidgets.QMessageBox.StandardButton.Save:
			return self._save_session(target)
		return True
	def _on_new(self) -> bool:
		"""Create and activate a new independent document tab."""
		self._create_session(activate=True)
		return True
	def _on_open(self) -> bool:
		"""Open a file in a new document tab."""
		file_path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self,
			self.tr("Open Chemistry File"),
			"",
			self.tr(ferrum_qt.actions.file_actions.CHEMISTRY_FILTER),
		)[0]
		if not file_path:
			return False
		return self.open_file_path(file_path)
	def _on_open_same_tab(self) -> bool:
		"""Open a file by deliberately replacing the current tab."""
		file_path = PySide6.QtWidgets.QFileDialog.getOpenFileName(
			self,
			self.tr("Open Chemistry File in Current Tab"),
			"",
			self.tr(ferrum_qt.actions.file_actions.CHEMISTRY_FILTER),
		)[0]
		if not file_path:
			return False
		return self.open_file_path(file_path, replace_current=True)
	def _open_path_replacing_current(self, file_path: str) -> bool:
		"""Compatibility wrapper for deliberate same-tab opening."""
		return self.open_file_path(file_path, replace_current=True)
	def open_file_path(
			self, file_path: str, replace_current: bool = False,
			) -> bool:
		"""Open a path in a new tab or deliberately replace the active tab."""
		absolute_path = os.path.abspath(file_path)
		canonical_path = os.path.normcase(os.path.realpath(absolute_path))
		for session in self._sessions:
			origin_path = session.origin_path
			if origin_path is None:
				continue
			existing_path = os.path.normcase(
				os.path.realpath(os.path.abspath(origin_path))
			)
			if existing_path == canonical_path:
				return self._select_session(session)

		extension = os.path.splitext(absolute_path)[1].lower()
		try:
			capability = (
				ferrum_qt.io.import_capabilities.capability_for_extension(
					extension,
				)
			)
		except ValueError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("File Read Error"),
				self.tr("Could not open %s:\n%s") % (absolute_path, exc),
			)
			return False
		if capability.route == "worker":
			return self._start_async_import(
				capability.codec_name,
				absolute_path,
				replace_current,
			)
		if capability.route != "native":
			raise RuntimeError(
				"Qt import capability '%s' has no loading route."
				% capability.codec_name
			)

		try:
			with open(absolute_path, encoding="utf-8") as source:
				cdml_text = source.read()
			prepared_native_cdml = (
				ferrum_qt.models.document_session.DocumentSession.prepare_native_cdml(
					cdml_text,
				)
			)
		except Exception as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("File Read Error"),
				self.tr("Could not open %s:\n%s") % (absolute_path, exc),
			)
			return False
		target = self._active_session if replace_current else None
		if target is not None and not self._confirm_save_if_dirty(
				"opening another file", target,
		):
			return False
		return self._install_prepared_native_cdml(
			absolute_path,
			prepared_native_cdml,
			replace_session=target,
		)
	def _start_async_import(
			self, codec_name: str, file_path: str, replace_current: bool,
			) -> bool:
		"""Start one session-owned non-CDML import."""
		startup_session = None
		if replace_current:
			target = self._active_session
		else:
			startup_session = self._pristine_startup_session()
			target = self._create_session(
				activate=True,
				display_name=self.tr(
					"Loading %s..." % os.path.basename(file_path)
				),
				origin_path=file_path,
			)
		request_token = target.begin_import_request()
		ferrum_qt.actions.file_actions._load_with_worker(
			self,
			codec_name,
			file_path,
			on_loaded=lambda prepared_cdml: self._complete_async_import(
				target,
				request_token,
				file_path,
				prepared_cdml,
				replace_current,
				startup_session,
			),
			should_deliver=lambda: (
				not self._shutdown_prepared
				and target in self._sessions
				and target.import_request_is_current(request_token)
			),
			worker_owner=target,
			on_error=lambda message: self._handle_async_import_error(
				target, request_token, file_path, message, replace_current,
			),
		)
		return True
	def _complete_async_import(
			self,
			target: ferrum_qt.models.document_session.DocumentSession,
			request_token: int,
			file_path: str,
			prepared_cdml: ferrum_qt.bridge.worker.PreparedCompleteCDML,
			replace_current: bool,
			startup_session: (
				ferrum_qt.models.document_session.DocumentSession | None
			),
			) -> bool:
		"""Install a prepared worker result only into its originating tab."""
		if (
				self._shutdown_prepared
				or target not in self._sessions
				or not target.import_request_is_current(request_token)
		):
			return False
		if not isinstance(prepared_cdml, ferrum_qt.bridge.worker.PreparedCompleteCDML):
			self._handle_async_import_error(
				target,
				request_token,
				file_path,
				self.tr("No molecules found"),
				replace_current,
			)
			return False
		try:
			prepared_imported_cdml = (
				ferrum_qt.models.document_session.DocumentSession.prepare_imported_cdml(
					prepared_cdml.complete_cdml,
				)
			)
		except Exception as exc:
			self._handle_async_import_error(
				target, request_token, file_path, str(exc), replace_current,
			)
			return False
		if replace_current:
			if not self._confirm_save_if_dirty(
				"opening another file", target,
			):
				return False
			return self._install_prepared_imported_cdml(
				file_path, prepared_imported_cdml, replace_session=target,
			)

		installed = self._install_prepared_imported_cdml(
			file_path, prepared_imported_cdml, replace_session=target,
		)
		if (
			installed
			and startup_session is not None
			and startup_session in self._sessions
			and self._pristine_startup_session() is None
			and not startup_session.document.objects
			and not startup_session.document.dirty
			and startup_session.origin_path is None
		):
			self._remove_session(startup_session)
		return installed
	def _handle_async_import_error(
			self,
			target: ferrum_qt.models.document_session.DocumentSession,
			request_token: int,
			file_path: str,
			message: str,
			replace_current: bool,
			) -> None:
		"""Report a current import error and remove only a loading tab."""
		if (
				target not in self._sessions
				or not target.import_request_is_current(request_token)
			):
			return
		PySide6.QtWidgets.QMessageBox.warning(
			self,
			self.tr("File Read Error"),
			self.tr("Could not open %s:\n%s") % (file_path, message),
		)
		if replace_current:
			return
		if len(self._sessions) == 1:
			replacement = self._construct_session()
			self._replace_with_prebuilt_session(target, replacement, activate=True)
		else:
			self._remove_session(target)
	def _install_prepared_native_cdml(
			self, file_path: str,
			prepared_native_cdml: (
				ferrum_qt.models.document_session.PreparedNativeCDML
			), *,
			replace_session: (
				ferrum_qt.models.document_session.DocumentSession | None
			) = None,
			) -> bool:
		"""Install an OASA-staged native projection after it is fully viable."""
		absolute_path = os.path.abspath(file_path)
		startup_session = None
		session: ferrum_qt.models.document_session.DocumentSession | None = None
		if replace_session is None:
			startup_session = self._pristine_startup_session()
		try:
			session = self._construct_session(
				file_path=absolute_path,
				origin_path=absolute_path,
				prepared_native_cdml=prepared_native_cdml,
			)
			molecule_projections = ferrum_qt.canvas.molecule_projection.project_molecules_to_scene(
				session.scene, session.document.molecules,
			)
			presentation_projections = ferrum_qt.canvas.document_projection.project_document_presentation(
				session.document, session.scene,
			)
			session.document.register_current_projection_items(
				tuple(
					item for _molecule, items in molecule_projections for item in items
				) + tuple(presentation_projections["presentation"].values())
				+ tuple(presentation_projections["marks"].values()),
			)
		except Exception as exc:
			if session is not None:
				self._dispose_session_later(session)
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("File Read Error"),
				self.tr("Could not open %s:\n%s") % (absolute_path, exc),
			)
			return False

		try:
			if replace_session is None:
				self._register_session(session, activate=True)
			else:
				session = self._replace_with_prebuilt_session(
					replace_session, session,
					activate=replace_session is self._active_session,
				)
				if session is None:
					raise RuntimeError("The target tab is no longer available")
		except Exception as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("File Read Error"),
				self.tr("Could not open %s:\n%s") % (absolute_path, exc),
			)
			return False

		ferrum_qt.actions.file_actions._record_recent_file(self, absolute_path)
		self._warn_unsupported_content(session, absolute_path)
		self.statusBar().showMessage(
			self.tr("Loaded %d molecule(s), %d drawing object(s)") % (
				len(session.document.molecules),
				len(session.document.presentation_objects),
			),
			3000,
		)
		if (
			startup_session is not None
			and startup_session is not session
			and startup_session in self._sessions
		):
			self._remove_session(startup_session)
		return True
	def _install_prepared_imported_cdml(
			self, file_path: str,
			prepared_imported_cdml: (
				ferrum_qt.models.document_session.PreparedImportedCDML
			), *,
			replace_session: (
				ferrum_qt.models.document_session.DocumentSession | None
			) = None,
			) -> bool:
		"""Install a fully staged external document without adopting its source path."""
		absolute_path = os.path.abspath(file_path)
		startup_session = None
		session = None
		if replace_session is None:
			startup_session = self._pristine_startup_session()
		try:
			session = self._construct_session(
				display_name=os.path.basename(absolute_path),
				origin_path=absolute_path,
				prepared_imported_cdml=prepared_imported_cdml,
			)
			molecule_projections = ferrum_qt.canvas.molecule_projection.project_molecules_to_scene(
				session.scene, session.document.molecules,
			)
			presentation_projections = ferrum_qt.canvas.document_projection.project_document_presentation(
				session.document, session.scene,
			)
			session.document.register_current_projection_items(
				tuple(
					item for _molecule, items in molecule_projections for item in items
				) + tuple(presentation_projections["presentation"].values())
				+ tuple(presentation_projections["marks"].values()),
			)
		except Exception as exc:
			if session is not None:
				self._dispose_session_later(session)
			PySide6.QtWidgets.QMessageBox.warning(
				self, self.tr("File Read Error"),
				self.tr("Could not open %s:\n%s") % (absolute_path, exc),
			)
			return False
		try:
			if replace_session is None:
				self._register_session(session, activate=True)
			else:
				session = self._replace_with_prebuilt_session(
					replace_session, session,
					activate=replace_session is self._active_session,
				)
				if session is None:
					raise RuntimeError("The target tab is no longer available")
		except Exception as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self, self.tr("File Read Error"),
				self.tr("Could not open %s:\n%s") % (absolute_path, exc),
			)
			return False
		ferrum_qt.actions.file_actions._record_recent_file(self, absolute_path)
		self._warn_unsupported_content(session, absolute_path)
		self.statusBar().showMessage(
			self.tr("Imported %d molecule(s); save as CDML to publish") % (
				len(session.document.molecules),
			), 3000,
		)
		if (
				startup_session is not None
				and startup_session is not session
				and startup_session in self._sessions
			):
			self._remove_session(startup_session)
		return True
	def _warn_unsupported_content(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			file_path: str,
			) -> None:
		"""Report retained CDML content that the Qt canvas cannot edit yet."""
		warnings = session.document.unsupported_content
		if not warnings:
			return
		details = []
		for warning in warnings:
			label = warning.tag
			if warning.object_id:
				label += f" id={warning.object_id}"
			details.append(f"{warning.path}: {label} - {warning.reason}")
		message = self.tr(
			"Some content in %s is not editable in the PySide6 frontend yet. "
			"It will be preserved when the document is saved.\n\n%s"
		) % (file_path, "\n".join(details))
		PySide6.QtWidgets.QMessageBox.warning(
			self, self.tr("Unsupported CDML Content"), message,
		)
	def _save_session_to_path(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			file_path: str,
			) -> bool:
		"""Authoritatively save one explicit session to CDML and establish a clean point."""
		if not session.can_write_authoritative_snapshot:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Authoritative Save Unavailable"),
				self.tr(
					"This document cannot be saved while its Qt projection is not an "
					"exact current backend snapshot. Use Recovery Export Backend CDML "
					"to publish the current backend snapshot without changing this "
					"document's saved state.",
				),
			)
			return False
		path = pathlib.Path(file_path)
		if not path.suffix:
			path = path.with_suffix(".cdml")
		elif path.suffix.lower() != ".cdml":
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Unsupported Save Format"),
				self.tr("Ferrum-Qt native documents must use the .cdml extension."),
			)
			return False
		absolute_path = os.path.abspath(str(path))
		try:
			session.write_backend_snapshot(absolute_path)
		except ferrum_qt.models.document_session.BackendSnapshotPublicationError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Save Durability Unconfirmed"),
				self.tr(
					"The exact canonical snapshot may be present at %s, but Save "
					"durability is unconfirmed. The backend saved state was not updated."
				) % absolute_path + "\n\n" + str(exc),
			)
			return False
		except Exception as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Save Error"),
				self.tr("Could not save %s:\n%s") % (absolute_path, exc),
			)
			return False
		self._record_successful_save_bookkeeping(
			session, absolute_path,
		)
		return True
	def _record_successful_save_bookkeeping(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			absolute_path: str,
			) -> None:
		"""Apply nonessential post-save presentation updates without falsifying Save.

		The file has already been published and OASA has updated its saved baseline.
		Title/status/recent-file errors
		therefore cannot turn a completed persistence operation into a false Save
		failure.
		"""
		try:
			session.set_file_path(absolute_path)
		except Exception:
			pass
		try:
			message = self.tr("Saved: %s") % absolute_path
			self.statusBar().showMessage(message, 3000)
		except Exception:
			pass
		try:
			ferrum_qt.actions.file_actions._record_recent_file(
				self, absolute_path,
			)
		except Exception:
			pass
	def _save_document_to_path(self, file_path: str) -> bool:
		"""Compatibility wrapper saving the active document session."""
		return self._save_session_to_path(self._active_session, file_path)
	def _save_session(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			force_save_as: bool = False,
			) -> bool:
		"""Save one session, prompting when it has no native CDML path."""
		if not session.can_write_authoritative_snapshot:
			return self._save_session_to_path(session, "")
		file_path = None if force_save_as else session.document.file_path
		if file_path is None:
			file_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
				self,
				self.tr(
					"Save CDML File As"
					if force_save_as
					else "Save CDML File"
				),
				session.document.file_path or "",
				self.tr("CDML Files (*.cdml);;All Files (*)"),
			)[0]
			if not file_path:
				return False
		return self._save_session_to_path(session, file_path)
	def _on_save(self) -> bool:
		"""Save the active document to its native CDML path."""
		return self._save_session(self._active_session)
	def _on_save_as(self) -> bool:
		"""Save the active document under a newly selected CDML path."""
		return self._save_session(self._active_session, force_save_as=True)
	def _save_template_session_to_path(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			file_path: str,
			) -> bool:
		"""Publish canonical backend CDML as a template without saving the session."""
		if not session.can_recovery_export:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Export Unavailable"),
				self.tr("No readable backend snapshot is available for this template."),
			)
			return False
		try:
			session.export_backend_snapshot(file_path)
		except ferrum_qt.models.document_session.BackendSnapshotPublicationError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Export Durability Unconfirmed"),
				self.tr("The canonical template may be present, but durability is unconfirmed.\n\n%s") % exc,
			)
			return False
		except Exception as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Export Error"),
				self.tr("Could not export template CDML to %s:\n%s") % (file_path, exc),
			)
			return False
		self.statusBar().showMessage(self.tr("Template saved: %s") % file_path, 3000)
		return True
	def _on_save_as_template(self) -> bool:
		"""Prompt for a template destination and publish the current backend snapshot."""
		session = self._active_recovery_export_session()
		if session is None:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Export Unavailable"),
				self.tr("No active backend snapshot is available for template export."),
			)
			return False
		snapshot = session.backend_snapshot
		try:
			ferrum_qt.bridge.user_template_inspection.inspect_user_template_display_name(
				snapshot.cdml,
			)
		except ferrum_qt.bridge.user_template_inspection.UserTemplateInspectionError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Not Eligible"),
				self.tr("Save As Template accepts one detached molecule with valid geometry.\n\n%s") % exc,
			)
			return False
		if self._user_template_directory is None:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Directory Unavailable"),
				self.tr("This embedded Ferrum window has no user template directory."),
			)
			return False
		try:
			self._user_template_directory.mkdir(parents=True, exist_ok=True)
		except OSError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Directory Unavailable"),
				self.tr("Could not create the user template directory:\n%s") % exc,
			)
			return False
		file_path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Save As Template"), str(self._user_template_directory),
			self.tr("CDML Template (*.cdml);;All Files (*)"),
		)[0]
		if not file_path:
			return False
		if self._registered_recovery_export_session(session, require_active=True) is None:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Export Unavailable"),
				self.tr("The active document changed before template publication."),
			)
			return False
		current_snapshot = session.backend_snapshot
		if current_snapshot != snapshot:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Export Unavailable"),
				self.tr("The document changed before template publication. Please try again."),
			)
			return False
		try:
			ferrum_qt.bridge.user_template_inspection.inspect_user_template_display_name(
				current_snapshot.cdml,
			)
		except ferrum_qt.bridge.user_template_inspection.UserTemplateInspectionError as exc:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Not Eligible"),
				self.tr("The document changed to an ineligible template.\n\n%s") % exc,
			)
			return False
		path = pathlib.Path(file_path)
		if not path.suffix:
			path = path.with_suffix(".cdml")
		elif path.suffix != ".cdml":
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Unsupported Template Format"),
				self.tr("Ferrum templates must use the lowercase .cdml extension."),
			)
			return False
		template_directory = self._user_template_directory.resolve()
		candidate = path.resolve()
		if candidate.parent != template_directory:
			PySide6.QtWidgets.QMessageBox.warning(
				self,
				self.tr("Template Destination Outside Catalog"),
				self.tr("Save templates directly in the configured user template directory."),
			)
			return False
		if not self._save_template_session_to_path(session, str(candidate)):
			return False
		self.rescan_user_templates()
		return True
	def save_as_template(self) -> bool:
		"""Publish one eligible active backend snapshot through File behavior."""
		return self._on_save_as_template()
	def _export_snapshot_to_path(self, format_name: str, path: str) -> bool:
		"""Render the active backend snapshot to one selected artifact path."""
		session = self._active_session
		if session is None or session not in self._sessions:
			self.statusBar().showMessage(self.tr("Visual export unavailable"), 3000)
			return False
		result = ferrum_qt.io.export.write_session_snapshot_artifact(
			session, format_name, path,
		)
		if not result.succeeded:
			self.statusBar().showMessage(result.message, 5000)
			return False
		message = self.tr("Exported %s") % path
		if result.warnings:
			message += self.tr(" (%d unsupported persistent object(s) omitted)") % len(result.warnings)
		self.statusBar().showMessage(message, 5000)
		return True
	def _on_export_svg(self) -> None:
		"""Export the active backend snapshot to SVG."""
		path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export SVG"), "", self.tr("SVG Files (*.svg)")
		)[0]
		if path:
			self._export_snapshot_to_path("svg", path)
	def _on_export_png(self) -> None:
		"""Export the active backend snapshot to PNG."""
		path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export PNG"), "", self.tr("PNG Files (*.png)")
		)[0]
		if path:
			self._export_snapshot_to_path("png", path)
	def _on_export_pdf(self) -> None:
		"""Export the active backend snapshot to PDF."""
		path = PySide6.QtWidgets.QFileDialog.getSaveFileName(
			self, self.tr("Export PDF"), "", self.tr("PDF Files (*.pdf)")
		)[0]
		if path:
			self._export_snapshot_to_path("pdf", path)
