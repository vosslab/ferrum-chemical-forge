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
class WindowPropertiesMixin:
	"""Cohesive MainWindow behavior with no MainWindow import."""

	@property
	def shutdown_state(self) -> ShutdownState:
		"""Return the observable application retirement state."""
		return self._shutdown_state
	@property
	def retiring_worker_count(self) -> int:
		"""Return the number of adopted workers still awaiting ``finished``."""
		return len(self._retired_import_workers)
	@property
	def document(self) -> ferrum_qt.models.document.Document:
		"""The active document."""
		return self._document
	@property
	def scene(self) -> PySide6.QtWidgets.QGraphicsScene:
		"""The active graphics scene."""
		return self._scene
	@property
	def view(self) -> PySide6.QtWidgets.QGraphicsView:
		"""The active graphics view."""
		return self._view
	@property
	def sessions(self) -> list:
		"""Return the open document sessions in tab order."""
		return list(self._sessions)
	def persistent_operation_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze a non-mode operation capability onto one exact registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Persistent operation capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Persistent operation capability requires a live registered session")
		def submit(
				request: ferrum_qt.models.document_session.PersistentOperationRequest,
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the captured session remains live and registered."""
			if (
				not isinstance(
					request, ferrum_qt.models.document_session.PersistentOperationRequest,
				)
			):
				raise TypeError("Persistent operations require PersistentOperationRequest")
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_persistent_operation(request)
		return submit
	def bond_properties_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze one narrow direct-bond patch capability onto a registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Bond properties capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Bond properties capability requires a live registered session")
		def submit(
				expected_revision: int, molecule_id: str, bond_id: str,
				changes: tuple[tuple[str, object], ...],
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the exact captured session remains registered."""
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_bond_properties_patch(
				expected_revision, molecule_id, bond_id, changes,
			)
		return submit
	def bond_properties_capability_for_view(self, view: object) -> object | None:
		"""Return one frozen bond-patch capability for this registered view.

		Interaction surfaces provide the view that owned their selected item.  The
		window resolves that view once, so a dialog or retained callback cannot be
		redirected merely because another tab later becomes active.
		"""
		session = self._sessions_by_view.get(view)
		if session is None or session.is_disposed:
			return None
		return self.bond_properties_capability_for(session)
	def capture_bond_properties_for_view(
			self, view: object, molecule_id: str, bond_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and exact-tab bond patch callback for one intent."""
		session = self._sessions_by_view.get(view)
		return self.capture_bond_properties_for(session, molecule_id, bond_id)
	def capture_bond_properties_for(
			self, session: object, molecule_id: str, bond_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and callback for a known registered bond session."""
		if (
			not isinstance(session, ferrum_qt.models.document_session.DocumentSession)
			or session.is_disposed or session not in self._sessions
			or session.document is None or not session.can_commit_persistent_action
			or not isinstance(molecule_id, str) or not molecule_id
			or not isinstance(bond_id, str) or not bond_id
		):
			return None
		return session.backend_snapshot.revision, self.bond_properties_capability_for(session)
	def atom_properties_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze one narrow direct-atom patch capability onto a registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Atom properties capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Atom properties capability requires a live registered session")
		def submit(
				expected_revision: int, molecule_id: str, atom_id: str,
				changes: tuple[tuple[str, object], ...],
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the exact captured session remains registered."""
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_atom_properties_patch(
				expected_revision, molecule_id, atom_id, changes,
			)
		return submit
	def atom_properties_capability_for_view(self, view: object) -> object | None:
		"""Return one frozen atom-patch capability for this registered view."""
		session = self._sessions_by_view.get(view)
		if session is None or session.is_disposed:
			return None
		return self.atom_properties_capability_for(session)
	def capture_atom_properties_for_view(
			self, view: object, molecule_id: str, atom_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and exact-tab atom patch callback for one intent."""
		session = self._sessions_by_view.get(view)
		return self.capture_atom_properties_for(session, molecule_id, atom_id)
	def capture_atom_properties_for(
			self, session: object, molecule_id: str, atom_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and callback for a known registered atom session."""
		if (
			not isinstance(session, ferrum_qt.models.document_session.DocumentSession)
			or session.is_disposed or session not in self._sessions
			or session.document is None or not session.can_commit_persistent_action
			or not isinstance(molecule_id, str) or not molecule_id
			or not isinstance(atom_id, str) or not atom_id
		):
			return None
		return session.backend_snapshot.revision, self.atom_properties_capability_for(session)
	def text_properties_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze one narrow direct-root plain Text patch onto a registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Text properties capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Text properties capability requires a live registered session")
		def submit(
				expected_revision: int, text_id: str,
				changes: tuple[tuple[str, object], ...],
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the exact captured session remains registered."""
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_text_properties_patch(
				expected_revision, text_id, changes,
			)
		return submit
	def rich_text_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze one authored rich-Text patch capability onto a registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Rich Text capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Rich Text capability requires a live registered session")
		def submit(
				expected_revision: int, text_id: str,
			runs: tuple[tuple[str, tuple[str, ...]], ...],
			changes: tuple[tuple[str, object], ...] = (),
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the exact captured session remains registered."""
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_rich_text_patch(expected_revision, text_id, runs, changes)
		return submit
	def capture_rich_text_for_view(
			self, view: object, text_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and exact-tab rich Text callback for one dialog."""
		session = self._sessions_by_view.get(view)
		if (
			session is None or session.is_disposed or session.document is None
			or not session.can_commit_persistent_action
			or not isinstance(text_id, str) or not text_id
		):
			return None
		return session.backend_snapshot.revision, self.rich_text_capability_for(session)
	def capture_text_properties_for_view(
			self, view: object, text_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and exact-tab Text patch callback for one intent."""
		session = self._sessions_by_view.get(view)
		if (
			session is None or session.is_disposed or session.document is None
			or not session.can_commit_persistent_action
			or not isinstance(text_id, str) or not text_id
		):
			return None
		return session.backend_snapshot.revision, self.text_properties_capability_for(session)
	def plus_properties_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze one narrow direct-root plain Plus patch onto a registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Plus properties capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Plus properties capability requires a live registered session")
		def submit(
				expected_revision: int, plus_id: str,
				changes: tuple[tuple[str, object], ...],
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the exact captured session remains registered."""
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_plus_properties_patch(
				expected_revision, plus_id, changes,
			)
		return submit
	def capture_plus_properties_for_view(
			self, view: object, plus_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and exact-tab Plus patch callback for one intent."""
		session = self._sessions_by_view.get(view)
		if (
			session is None or session.is_disposed or session.document is None
			or not session.can_commit_persistent_action
			or not isinstance(plus_id, str) or not plus_id
		):
			return None
		return session.backend_snapshot.revision, self.plus_properties_capability_for(session)
	def wavy_properties_capability_for(
			self, session: ferrum_qt.models.document_session.DocumentSession,
			) -> object:
		"""Freeze one narrow direct-root plain Wavy patch onto a registered tab."""
		if not isinstance(session, ferrum_qt.models.document_session.DocumentSession):
			raise TypeError("Wavy properties capability requires a DocumentSession")
		if session.is_disposed or session not in self._sessions:
			raise ValueError("Wavy properties capability requires a live registered session")
		def submit(
				expected_revision: int, wavy_id: str,
				changes: tuple[tuple[str, object], ...],
				) -> ferrum_qt.models.document_session.PersistentActionOutcome:
			"""Submit only while the exact captured session remains registered."""
			if session.is_disposed or session not in self._sessions:
				return ferrum_qt.models.document_session.PersistentActionOutcome(
					"unavailable", "Document cannot accept a persistent edit", None, False,
				)
			return session.submit_wavy_properties_patch(
				expected_revision, wavy_id, changes,
			)
		return submit
	def capture_wavy_properties_for_view(
			self, view: object, wavy_id: str,
			) -> tuple[int, object] | None:
		"""Capture one revision and exact-tab Wavy patch callback for one intent."""
		session = self._sessions_by_view.get(view)
		if (
			session is None or session.is_disposed or session.document is None
			or not session.can_commit_persistent_action
			or not isinstance(wavy_id, str) or not wavy_id
		):
			return None
		return session.backend_snapshot.revision, self.wavy_properties_capability_for(session)
	def _bind_property_dock(
			self,
			session: ferrum_qt.models.document_session.DocumentSession | None,
			) -> None:
		"""Bind the dock to one live projection and its exact session callbacks.

		This is the sole MainWindow binding seam for the disposable projection:
		all dock callbacks close over the supplied session rather than active
		window aliases, so tab activation and recovery cannot redirect an edit.
		"""
		if not hasattr(self, "_property_dock"):
			return
		if (
			session is None or session.is_disposed or session not in self._sessions
			or session.document is None
		):
			self._property_dock.set_document(None)
			return
		def capture_bond(molecule_id: str, bond_id: str) -> tuple[int, object] | None:
			"""Capture one dock bond intent for the bound session."""
			return self.capture_bond_properties_for(session, molecule_id, bond_id)

		def capture_atom(molecule_id: str, atom_id: str) -> tuple[int, object] | None:
			"""Capture one dock atom intent for the bound session."""
			return self.capture_atom_properties_for(session, molecule_id, atom_id)

		self._property_dock.set_document(
			session.document,
			bond_properties_capture=capture_bond,
			atom_properties_capture=capture_atom,
		)
