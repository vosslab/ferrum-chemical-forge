"""Ordinary Qt request and one-shot placement for Ferrum direct-glycosidic Haworth."""

import dataclasses

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_qt.ferrum.engine as engine
from ferrum_qt.dialogs.accessibility import FerrumAccessibleDialog

import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.direct_glycosidic_haworth
from ferrum_qt.ferrum.document_tab_errors import (
	FerrumNativeDocumentTabMutationPresentationError,
)


@dataclasses.dataclass
class _DirectGlycosidicHaworthIntent:
	"""One source-bound, unanchored Rust request awaiting a canvas location."""

	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	source: object


class _DirectGlycosidicHaworthDialog(FerrumAccessibleDialog):
	"""Collect structural text without suggesting chemical identity or a preset."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget) -> None:
		super().__init__(parent)
		self.setWindowTitle(self.tr("Insert Direct-Glycosidic Haworth"))
		self.setAccessibleName(self.tr("Insert Direct-Glycosidic Haworth"))
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		description = PySide6.QtWidgets.QLabel(self.tr(
			"Create one detached two-ring Haworth drawing from a supported structural "
			"SMILES. This tool checks a limited C/O ring-and-bridge profile. It does "
			"not identify a sugar, infer stereochemistry, or name a glycosidic linkage.",
		), self)
		description.setWordWrap(True)
		description.setAccessibleName(self.tr("Direct-glycosidic Haworth description"))
		layout.addWidget(description)
		label = PySide6.QtWidgets.QLabel(self.tr("Structural SMILES:"), self)
		self.smiles_edit = PySide6.QtWidgets.QLineEdit(self)
		self.smiles_edit.setAccessibleName(self.tr("Structural SMILES"))
		self._smiles_accessible_description = description.text()
		self.smiles_edit.setAccessibleDescription(self._smiles_accessible_description)
		label.setBuddy(self.smiles_edit)
		layout.addWidget(label)
		layout.addWidget(self.smiles_edit)
		profile = PySide6.QtWidgets.QLabel(self.tr(
			"Two five- or six-member C/O rings joined by one exterior oxygen; neutral "
			"single bonds only.",
		), self)
		profile.setWordWrap(True)
		profile.setAccessibleName(self.tr("Supported structural profile"))
		layout.addWidget(profile)
		self.error = PySide6.QtWidgets.QLabel(self)
		self.error.setWordWrap(True)
		self.error.setAccessibleName(self.tr("Structural SMILES request error"))
		self.error.setVisible(False)
		layout.addWidget(self.error)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self.start_button = buttons.addButton(
			self.tr("Start placement"), PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole,
		)
		buttons.addButton(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel)
		self.start_button.clicked.connect(self.accept)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)
		self.smiles_edit.setFocus()

	def show_error(self, message: str) -> None:
		"""Retain the request and give the field an accessible recovery surface."""
		self.error.setText(message)
		self.error.setVisible(True)
		self.smiles_edit.setAccessibleDescription(
			f"{self._smiles_accessible_description} {message}",
		)
		self.smiles_edit.setFocus()

	def clear_error(self) -> None:
		"""Restore the field's stable description before another request."""
		self.error.clear()
		self.error.setVisible(False)
		self.smiles_edit.setAccessibleDescription(self._smiles_accessible_description)


class FerrumNativeDirectGlycosidicHaworthWindowMixin:
	"""Own Qt lifecycle only; Rust owns SMILES, receipt, graph, and history."""

	def _build_direct_glycosidic_haworth_action(self,
			menu: PySide6.QtWidgets.QMenu) -> None:
		"""Add the one scoped ordinary-product Chemistry action."""
		self._insert_direct_glycosidic_haworth_action = PySide6.QtGui.QAction(
			self.tr("Insert Direct-Glycosidic Haworth..."), self,
		)
		message = self.tr(
			"Validate a supported two-ring structural SMILES, then place one detached "
			"Haworth drawing.",
		)
		self._insert_direct_glycosidic_haworth_action.setToolTip(message)
		self._insert_direct_glycosidic_haworth_action.setStatusTip(message)
		self._connect_interaction_action_v1(
			self._insert_direct_glycosidic_haworth_action,
			self._on_insert_direct_glycosidic_haworth,
		)
		menu.addAction(self._insert_direct_glycosidic_haworth_action)
		self._direct_glycosidic_haworth_intent: _DirectGlycosidicHaworthIntent | None = None

	def _refresh_direct_glycosidic_haworth_action(self, active: bool,
			pending: bool, busy: bool) -> None:
		"""Retire a source-bound request once its Ferrum source is no longer current."""
		intent = self._direct_glycosidic_haworth_intent
		if intent is not None and (
			not active or pending or busy or self._active_native_tab() is not intent.tab
			or not self._direct_glycosidic_haworth_is_current(intent)
		):
			self._cancel_direct_glycosidic_haworth_intent()
		self._insert_direct_glycosidic_haworth_action.setEnabled(active and not pending and not busy)

	def _refresh_cancel_tool_action(self) -> None:
		super()._refresh_cancel_tool_action()
		if self._direct_glycosidic_haworth_intent is not None:
			self._cancel_tool_action.setEnabled(True)

	def _on_cancel_tool(self) -> None:
		self._cancel_direct_glycosidic_haworth_intent()
		super()._on_cancel_tool()

	def _on_insert_direct_glycosidic_haworth(self, _checked: bool = False) -> bool:
		"""Keep typed Rust refusals in an accessible retry-preserving dialog."""
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return False
		if self._atom_insertion_intent is not None or self._line_gesture_intent is not None:
			# Reuse the host's ordinary Cancel Tool path so an older event filter
			# cannot resume after this dialog or its later one-click placement ends.
			super()._on_cancel_tool()
		dialog = _DirectGlycosidicHaworthDialog(self)
		while dialog.exec() == PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			dialog.clear_error()
			smiles = dialog.smiles_edit.text()
			if not smiles.strip():
				dialog.show_error(self.tr("Enter a structural SMILES."))
				continue
			if self._active_native_tab() is not tab or tab.requires_refresh:
				dialog.reject()
				self._direct_glycosidic_haworth_changed_before_placement()
				return False
			try:
				source = tab.prepare_direct_glycosidic_haworth_source(smiles)
			except engine.DirectHaworthError as error:
				dialog.show_error(self._direct_glycosidic_haworth_request_error(error))
				continue
			snapshot = tab.current_snapshot
			self._cancel_direct_glycosidic_haworth_intent()
			intent = _DirectGlycosidicHaworthIntent(
				tab, tab.view.viewport(), snapshot.revision, snapshot.digest, source,
			)
			self._direct_glycosidic_haworth_intent = intent
			intent.viewport.installEventFilter(self)
			intent.viewport.setFocus()
			self.statusBar().showMessage(self.tr(
				"Click an empty page location to place the two-ring drawing; Escape cancels.",
			))
			self._refresh_cancel_tool_action()
			return True
		return False

	def eventFilter(self, watched: object, event: PySide6.QtCore.QEvent) -> bool:
		intent = self._direct_glycosidic_haworth_intent
		if intent is not None and watched is intent.viewport:
			if (
				event.type() == PySide6.QtCore.QEvent.Type.KeyPress
				and event.key() == PySide6.QtCore.Qt.Key.Key_Escape
			):
				self._cancel_direct_glycosidic_haworth_intent()
				return True
			if (
				event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress
				and event.button() == PySide6.QtCore.Qt.MouseButton.LeftButton
			):
				self._place_direct_glycosidic_haworth(intent, event)
				return True
		return super().eventFilter(watched, event)

	def _place_direct_glycosidic_haworth(self,
			intent: _DirectGlycosidicHaworthIntent,
			event: PySide6.QtGui.QMouseEvent) -> None:
		"""Use one snap and an opaque receipt; retain intent after occupancy refusal."""
		if not self._direct_glycosidic_haworth_is_current(intent):
			self._cancel_direct_glycosidic_haworth_intent()
			self._direct_glycosidic_haworth_changed_before_placement()
			return
		viewport_point = event.position().toPoint()
		raw_anchor = intent.tab.view.mapToScene(viewport_point)
		anchor = intent.tab.view.snap_authored_scene_point(raw_anchor)
		if (
			intent.tab.durable_structure_at_viewport_point(viewport_point) is not None
			or intent.tab.durable_structure_at_viewport_point(
				intent.tab.view.mapFromScene(anchor),
			) is not None
		):
			self.statusBar().showMessage(self.tr(
				"Choose an empty page location to insert a separate two-ring drawing.",
			), 5000)
			return
		try:
			prepared = intent.tab.prepare_direct_glycosidic_haworth_placement(
				intent.source, float(anchor.x()), float(anchor.y()),
			)
		except engine.DirectHaworthError:
			self._cancel_direct_glycosidic_haworth_intent()
			self._show_edit_refusal(self._unavailable_edit_refusal(self.tr(
				"The drawing could not be placed. Choose Insert Direct-Glycosidic Haworth "
				"again and try another empty page location.",
			)))
			return
		try:
			preview = ferrum_qt.ferrum.direct_glycosidic_haworth.create_preview(
				intent.tab, prepared,
			)
		except ValueError:
			self._cancel_direct_glycosidic_haworth_intent()
			self._show_edit_refusal(self._unavailable_edit_refusal(self.tr(
				"The drawing preview could not be prepared. Choose Insert Direct-Glycosidic "
				"Haworth again and try another empty page location.",
			)))
			self._refresh_actions()
			return
		self._retire_direct_glycosidic_haworth_preview(preview)
		try:
			intent.tab.commit_direct_glycosidic_haworth(prepared)
		except FerrumNativeDocumentTabMutationPresentationError:
			self._cancel_direct_glycosidic_haworth_intent()
			self._show_edit_refusal(self._unavailable_edit_refusal(self.tr(
				"The drawing was accepted, but the authoritative view needs Refresh before "
				"saving or editing again.",
			)))
			self._refresh_actions()
			return
		except engine.DirectHaworthError:
			self._cancel_direct_glycosidic_haworth_intent()
			self._show_edit_refusal(self._unavailable_edit_refusal(self.tr(
				"The drawing could not be placed. Choose Insert Direct-Glycosidic Haworth "
				"again and try another empty page location.",
			)))
			return
		self._cancel_direct_glycosidic_haworth_intent()
		self.statusBar().showMessage(self.tr(
			"Inserted the two-ring Haworth drawing; use Undo to remove it.",
		), 4000)
		self._refresh_actions()

	def _retire_direct_glycosidic_haworth_preview(
			self, preview: PySide6.QtWidgets.QGraphicsItem) -> None:
		"""Release a disposable preview before authoritative scene replacement."""
		scene = ferrum_qt.canvas.graphics_retirement.native_scene_for_item(preview)
		if scene is not None:
			coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
			coordinator.retire_scene_projection_items(scene, [preview])

	def _direct_glycosidic_haworth_is_current(
			self, intent: _DirectGlycosidicHaworthIntent) -> bool:
		"""Reject redirected, closed, busy, or revision-stale source intent."""
		tab = intent.tab
		if (
			self._native_tabs_by_page.get(tab) is not tab or tab._disposed
			or tab.requires_refresh or self._active_native_tab() is not tab
		):
			return False
		snapshot = tab.current_snapshot
		return snapshot.revision == intent.revision and snapshot.digest == intent.digest

	def _direct_glycosidic_haworth_request_error(self, error: object) -> str:
		"""Keep recovery copy stable without exposing a Rust or Python exception."""
		category = error.reason
		return self.tr(
			"Cannot create this direct-glycosidic Haworth drawing: {0}. Use a neutral, "
			"single-bond C/O two-ring structure with one exterior oxygen bridge, or cancel.",
		).format(category)

	def _direct_glycosidic_haworth_changed_before_placement(self) -> None:
		"""Explain that the captured source tab changed and no receipt was committed."""
		self.statusBar().showMessage(self.tr(
			"The document changed before placement. Choose Insert Direct-Glycosidic Haworth again.",
		), 5000)
		self._refresh_actions()

	def _cancel_direct_glycosidic_haworth_intent(self) -> None:
		"""Disarm without retaining source receipt or viewport ownership."""
		intent = self._direct_glycosidic_haworth_intent
		if intent is not None:
			intent.viewport.removeEventFilter(self)
		self._direct_glycosidic_haworth_intent = None
		if hasattr(self, "_cancel_tool_action"):
			self._refresh_cancel_tool_action()
