"""Main application window for Ferrum-Qt."""

# Standard Library
import functools

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
class WindowSessionLifecycleMixin:
	"""One cohesive MainWindow session responsibility."""

	def _begin_import_request(self) -> int:
		"""Compatibility wrapper for the active session's import token."""
		return self._active_session.begin_import_request()
	def _invalidate_import_requests(self) -> None:
		"""Invalidate asynchronous imports targeting the active session."""
		if self._active_session is not None:
			self._active_session.invalidate_import_requests()
	def _import_request_is_current(self, token: int) -> bool:
		"""Return whether an import still targets the active live session."""
		return (
			not self._shutdown_prepared
			and self._active_session is not None
			and self._active_session.import_request_is_current(token)
		)
	def _track_import_worker(self, worker: PySide6.QtCore.QThread) -> None:
		"""Compatibility wrapper retaining a worker in the active session."""
		self._active_session.track_import_worker(worker)
	def _release_import_worker(self, worker: PySide6.QtCore.QThread) -> None:
		"""Release one finished worker without dereferencing a retired session.

		Queued ``QThread.finished`` slots can run after a closing tab has removed
		its session and released its Python-owned graph.  The window outlives those
		slots, so it is the terminal owner: it releases only workers still found in
		registered live sessions and otherwise retires the stopped worker directly.
		"""
		if worker in self._retired_import_workers:
			self._retired_import_workers.discard(worker)
			if not worker.isRunning():
				worker.deleteLater()
			self._emit_worker_retirement_drained()
			return
		for session in self._sessions:
			if worker in session._import_workers:
				session.release_import_worker(worker)
				return
		try:
			if not worker.isRunning():
				worker.deleteLater()
		except RuntimeError:
			# Session disposal may already have queued native worker deletion.
			pass
	def _retain_retiring_session_workers(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Transfer a result-delivering worker to the window's terminal owner.

		A same-tab replacement can be initiated by the worker's queued result
		slot before the native thread emits ``finished``.  The retiring session
		must therefore release ownership without joining that still-delivering
		thread; the window retains it until the relay observes ``finished``.
		"""
		for worker in session.retire_import_workers():
			self._retired_import_workers.add(worker)
	def _emit_worker_retirement_drained(self) -> None:
		"""Publish the terminal drain only after every adopted worker finished."""
		if self._shutdown_prepared and not self._retired_import_workers:
			self._shutdown_state = ShutdownState.READY
			self._complete_shutdown_session_disposal()
			self.worker_retirement_drained.emit()
	def _complete_shutdown_session_disposal(self) -> None:
		"""Queue detached session roots only after worker retirement drains."""
		sessions = tuple(self._shutdown_sessions_pending_disposal)
		self._shutdown_sessions_pending_disposal.clear()
		for session in sessions:
			self._dispose_session_later(session)
	def _stop_import_workers(self) -> None:
		"""Move all workers into delivery-cancelled window retirement."""
		for session in tuple(self._sessions):
			self._retain_retiring_session_workers(session)
		for worker in tuple(self._retired_import_workers):
			worker.requestInterruption()
	def _dispose_scene_items(
			self,
			session: ferrum_qt.models.document_session.DocumentSession | None = None,
			) -> None:
		"""Disconnect live and undo-retained graphics callbacks."""
		target = session if session is not None else self._active_session
		if target is None:
			return
		from ferrum_qt.canvas.graphics_retirement import GraphicsRetirementCoordinator
		coordinator = GraphicsRetirementCoordinator()
		coordinator.prepare_scene_retirement(target.scene, target.document.undo_stack)
		coordinator.raise_if_callback_failed(
			"Scene graphics callbacks were released after a disposal failure",
		)
	def _select_session(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> bool:
		"""Select and activate an existing live session."""
		if session.is_disposed or session not in self._sessions:
			return False
		index = self._tab_widget.indexOf(session.view)
		if index < 0:
			return False
		self._tab_widget.setCurrentIndex(index)
		if self._active_session is not session:
			self._activate_session(session)
		return True
	def _pristine_startup_session(
			self,
			) -> ferrum_qt.models.document_session.DocumentSession | None:
		"""Return the sole untouched startup tab, if it still exists."""
		if len(self._sessions) != 1:
			return None
		session = self._sessions[0]
		document = session.document
		if document is None:
			return None
		if (
				not document.objects
				and document.file_path is None
				and not document.dirty
				and session.origin_path is None
		):
			return session
		return None
	def _detach_tab_page(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			index: int,
			) -> None:
		"""Remove a tab page and transfer its native ownership from QTabWidget."""
		self._tab_widget.removeTab(index)
		session.view.hide()
		session.view.setParent(None)
	def _ensure_session_tab_attached(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			index: int,
			) -> None:
		"""Restore a registered session's page after a failed transition."""
		if self._tab_widget.indexOf(session.view) < 0:
			self._tab_widget.insertTab(index, session.view, session.title)
	def _unregister_session_without_disposal(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			title_connected: bool,
			) -> None:
		"""Remove a failed session while leaving its owner responsible to dispose it."""
		session.clear_projection_lifecycle_port()
		tab_index = self._tab_widget.indexOf(session.view)
		if tab_index >= 0:
			self._tab_widget.removeTab(tab_index)
			session.view.hide()
			session.view.setParent(None)
		self._sessions_by_view.pop(session.view, None)
		if session in self._sessions:
			self._sessions.remove(session)
		if title_connected:
			try:
				session.title_changed.disconnect(self._on_session_title_changed)
			except (RuntimeError, TypeError):
				pass
	def _restore_active_session(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			index: int,
			) -> None:
		"""Restore a live session after another session's activation failed."""
		current = self._active_session
		if current is not None and current is not session:
			if self._ui_signals_connected:
				self._disconnect_active_session_signals(current)
			current_mode = current.mode_manager.current_mode
			if current_mode is not None:
				current_mode.deactivate()
		if current is session and self._ui_signals_connected:
			self._disconnect_active_session_signals(session)
		self._set_active_session_aliases(session)
		self._bind_property_dock(session)
		if self._ui_signals_connected:
			self._connect_active_session_signals(session)
			active_mode = session.mode_manager.current_mode
			if active_mode is not None:
				active_mode.activate()
		if index >= 0:
			self._tab_widget.setCurrentIndex(index)
	def _dispose_session_later(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			) -> None:
		"""Retain a detached session until Qt confirms native destruction.

		``deleteLater()`` does not keep the Python wrapper strongly owned.
		Without this registry, a removed session can lose its final Python
		reference before Qt delivers its queued child-deletion events.
		"""
		session_key = id(session)
		document = session.document
		scene = session.scene
		view = session.view
		mode_manager = session.mode_manager
		# Keep only QObject roots while the session breaks its nested Python
		# reference graph.  Scene-owned QGraphicsItem wrappers must never cross
		# the synchronous scene-content disposal boundary into this reaper.
		retained_wrappers = [
			session,
			scene,
			view,
			mode_manager,
			*tuple(mode_manager._modes.values()),
		]
		if document is not None:
			retained_wrappers.extend((document, document.undo_stack))
		pending = _PendingSessionDeletion(retained_wrappers)
		self._pending_session_deletions[session_key] = pending
		session.destroyed.connect(functools.partial(
			self._release_disposed_session_later, session_key,
		))
		dispose_error = None
		try:
			session.dispose()
		except Exception as exc:
			dispose_error = exc
		if session._teardown_phase == "roots_queued":
			pending.retained_graphics_records = (
				session.take_retained_graphics_records()
			)
			session.release_python_references()
			session.setParent(None)
			session.deleteLater()
		else:
			raise RuntimeError(
				"Session roots remain retained because Qt teardown did not reach "
				"the queued terminal phase",
			) from dispose_error
		if dispose_error is not None:
			raise RuntimeError(
				"Session was queued after a disposal failure",
			) from dispose_error
	def _release_disposed_session_later(
			self, session_key: int, _destroyed_object: object = None,
			) -> None:
		"""Release an invalid session after its retained graphics are resolved."""
		pending = self._pending_session_deletions.get(session_key)
		if pending is None:
			return
		pending.session_destroyed = True
		if self._pending_session_graphics_are_resolved(pending):
			PySide6.QtCore.QTimer.singleShot(0, functools.partial(
				self._pending_session_deletions.pop, session_key, None,
			))
		else:
			self._schedule_pending_session_graphics_retry()
	def _schedule_pending_session_graphics_retry(self) -> None:
		"""Schedule one bounded ordinary retry for destroyed-session graphics.

		The destroyed callback is the first MainWindow-owned resolution pass.  A
		second zero-delay pass covers a transient native deletion failure after
		Qt has advanced normally.  Further failures stay retained for explicit
		shutdown draining so this path cannot create a busy event-loop retry.
		"""
		if self._shutdown_prepared or self._pending_session_graphics_retry_scheduled:
			return
		self._pending_session_graphics_retry_scheduled = True
		PySide6.QtCore.QTimer.singleShot(
			0, self._retry_pending_session_graphics_once,
		)
	def _retry_pending_session_graphics_once(self) -> None:
		"""Run the one queued retry through the normal MainWindow resolver."""
		self._pending_session_graphics_retry_scheduled = False
		if self._shutdown_prepared:
			return
		self._resolve_pending_session_graphics()
	def _pending_session_graphics_are_resolved(
			self, pending: _PendingSessionDeletion,
			) -> bool:
		"""Resolve retained graphics only through the coordinator's native boundary."""
		records = pending.retained_graphics_records
		if records is None or not records.unresolved:
			return True
		from ferrum_qt.canvas.graphics_retirement import DetachedGraphicsRetirementReaper
		reaper = DetachedGraphicsRetirementReaper()
		reaper.retain_graphics_records(records)
		reaper.drain()
		pending.retained_graphics_records = reaper.take_retained_graphics_records()
		return not pending.retained_graphics_records.unresolved
	def _resolve_pending_session_graphics(self) -> None:
		"""Advance terminal graphics records during the controlled reaper drain."""
		for session_key, pending in tuple(self._pending_session_deletions.items()):
			if not self._pending_session_graphics_are_resolved(pending):
				continue
			if pending.session_destroyed:
				self._pending_session_deletions.pop(session_key, None)
	def _remove_session(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> bool:
		"""Remove and deterministically dispose one session without prompting."""
		if session not in self._sessions:
			return False
		index = self._sessions.index(session)
		was_active = session is self._active_session
		if was_active:
			if self._ui_signals_connected:
				self._disconnect_active_session_signals(session)
			self._bind_property_dock(None)
			self._active_session = None
			self._document = None
			self._scene = None
			self._view = None
			self._mode_manager = None
		try:
			session.title_changed.disconnect(self._on_session_title_changed)
		except (RuntimeError, TypeError):
			pass
		session.clear_projection_lifecycle_port()

		previous_block = self._tab_widget.blockSignals(True)
		self._tab_change_blocked = True
		try:
			self._detach_tab_page(session, index)
			self._sessions.pop(index)
			self._sessions_by_view.pop(session.view, None)
			if was_active and self._sessions:
				next_index = min(index, len(self._sessions) - 1)
				self._tab_widget.setCurrentIndex(next_index)
		finally:
			self._tab_change_blocked = False
			self._tab_widget.blockSignals(previous_block)

		self._retain_retiring_session_workers(session)
		self._dispose_session_later(session)
		if was_active and self._sessions:
			next_view = self._tab_widget.currentWidget()
			next_session = self._sessions_by_view.get(next_view)
			if next_session is None:
				next_session = self._sessions[0]
				self._tab_widget.setCurrentIndex(0)
			self._activate_session(next_session)
		return True
	def _replace_with_prebuilt_session(
			self,
			session: ferrum_qt.models.document_session.DocumentSession,
			replacement: ferrum_qt.models.document_session.DocumentSession,
			*, activate: bool | None = None,
			) -> ferrum_qt.models.document_session.DocumentSession | None:
		"""Atomically swap a viable detached session into one registered tab."""
		if replacement.is_disposed or replacement in self._sessions:
			raise ValueError("Replacement session must be live and unregistered")
		if replacement.view in self._sessions_by_view:
			raise ValueError("Replacement view is already registered")
		if session not in self._sessions:
			self._dispose_session_later(replacement)
			return None
		index = self._sessions.index(session)
		was_active = session is self._active_session
		should_activate = was_active if activate is None else activate
		previous_index = self._tab_widget.currentIndex()
		active_target: ferrum_qt.models.document_session.DocumentSession | None = None
		if should_activate:
			active_target = replacement
		elif was_active:
			for candidate in self._sessions:
				if candidate is not session:
					active_target = candidate
					break
			if active_target is None:
				active_target = replacement
		replacement_registered = False
		old_title_disconnected = False

		previous_block = self._tab_widget.blockSignals(True)
		self._tab_change_blocked = True
		try:
			self._register_session(replacement, index=index, activate=False)
			replacement_registered = True
			if active_target is not None:
				active_index = self._tab_widget.indexOf(active_target.view)
				self._tab_widget.setCurrentIndex(active_index)
				self._activate_session(active_target)
			try:
				session.title_changed.disconnect(self._on_session_title_changed)
				old_title_disconnected = True
			except (RuntimeError, TypeError):
				pass
			self._detach_tab_page(session, index + 1)
			self._sessions.pop(index + 1)
			self._sessions_by_view.pop(session.view, None)
		except Exception:
			if session in self._sessions:
				self._ensure_session_tab_attached(session, index + 1)
				if old_title_disconnected:
					session.title_changed.connect(self._on_session_title_changed)
			if was_active and session in self._sessions:
				old_index = self._tab_widget.indexOf(session.view)
				self._restore_active_session(session, old_index)
			if replacement_registered:
				self._unregister_session_without_disposal(replacement, True)
				self._dispose_session_later(replacement)
			if session in self._sessions:
				old_index = self._tab_widget.indexOf(session.view)
				if old_index >= 0:
					self._tab_widget.setCurrentIndex(old_index)
			elif previous_index >= 0:
				self._tab_widget.setCurrentIndex(previous_index)
			raise
		finally:
			self._tab_change_blocked = False
			self._tab_widget.blockSignals(previous_block)

		self._retain_retiring_session_workers(session)
		self._dispose_session_later(session)
		return replacement
	def close_session_at(self, index: int) -> bool:
		"""Close one tab, or close the application when it is the final tab."""
		if index < 0 or index >= len(self._sessions):
			return False
		if len(self._sessions) == 1:
			return bool(self.close())
		session = self._sessions[index]
		if not self._confirm_save_if_dirty("closing this tab", session):
			self._select_session(session)
			return False
		return self._remove_session(session)
	def close_current_tab(self) -> bool:
		"""Close the currently selected tab through its save guard."""
		return self.close_session_at(self._tab_widget.currentIndex())
