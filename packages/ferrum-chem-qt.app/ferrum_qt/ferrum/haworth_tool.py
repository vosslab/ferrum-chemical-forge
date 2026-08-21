"""Ordinary Ferrum chooser and one-shot placement for closed D-glucose Haworth recipes."""

import dataclasses

import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
from ferrum_qt.dialogs.accessibility import finalize_dialog_accessibility

import ferrum_qt.canvas.graphics_retirement
import ferrum_qt.ferrum.haworth


@dataclasses.dataclass
class _HaworthIntent:
	tab: object
	viewport: PySide6.QtWidgets.QWidget
	revision: int
	digest: str
	recipe: str


class FerrumNativeHaworthToolMixin:
	"""Own only the Ferrum client lifecycle; Rust owns recipe and durable drawing facts."""

	def _build_line_tool_actions(self, edit_menu: PySide6.QtWidgets.QMenu) -> None:
		super()._build_line_tool_actions(edit_menu)
		self._insert_haworth_ring_action = PySide6.QtGui.QAction(
			self.tr("Insert Haworth Ring..."), self,
		)
		self._insert_haworth_ring_action.setToolTip(self.tr(
			"Choose a D-glucose Haworth form, then click an empty page location.",
		))
		self._connect_interaction_action_v1(
			self._insert_haworth_ring_action, self._choose_haworth_recipe,
		)
		edit_menu.addAction(self._insert_haworth_ring_action)
		self._haworth_intent: _HaworthIntent | None = None

	def _refresh_line_tool_actions(self, enabled: bool) -> None:
		super()._refresh_line_tool_actions(enabled)
		self._insert_haworth_ring_action.setEnabled(enabled)

	def _refresh_cancel_tool_action(self) -> None:
		super()._refresh_cancel_tool_action()
		if getattr(self, "_haworth_intent", None) is not None:
			self._cancel_tool_action.setEnabled(True)

	def _on_cancel_tool(self) -> None:
		self._cancel_haworth_intent()
		super()._on_cancel_tool()

	def _choose_haworth_recipe(self) -> None:
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return
		dialog = PySide6.QtWidgets.QDialog(self)
		dialog.setWindowTitle(self.tr("Insert Haworth Ring"))
		dialog.setAccessibleName(self.tr("Insert D-glucose Haworth ring"))
		layout = PySide6.QtWidgets.QVBoxLayout(dialog)
		form = PySide6.QtWidgets.QGroupBox(self.tr("Ring form"), dialog)
		form_layout = PySide6.QtWidgets.QVBoxLayout(form)
		pyranose = PySide6.QtWidgets.QRadioButton(self.tr("Six-membered pyranose"), form)
		furanose = PySide6.QtWidgets.QRadioButton(self.tr("Five-membered furanose"), form)
		pyranose.setChecked(True)
		form_layout.addWidget(pyranose); form_layout.addWidget(furanose); layout.addWidget(form)
		anomer = PySide6.QtWidgets.QGroupBox(self.tr("Anomer"), dialog)
		anomer_layout = PySide6.QtWidgets.QVBoxLayout(anomer)
		alpha = PySide6.QtWidgets.QRadioButton(self.tr("alpha"), anomer)
		beta = PySide6.QtWidgets.QRadioButton(self.tr("beta"), anomer)
		alpha.setChecked(True)
		anomer_layout.addWidget(alpha); anomer_layout.addWidget(beta); layout.addWidget(anomer)
		summary = PySide6.QtWidgets.QLabel(dialog)
		summary.setAccessibleName(self.tr("Selected D-glucose Haworth structure"))
		def update_summary() -> None:
			name = "beta" if beta.isChecked() else "alpha"
			form_name = "glucofuranose" if furanose.isChecked() else "glucopyranose"
			summary.setText(f"{name}-D-{form_name}")
		for control in (pyranose, furanose, alpha, beta): control.toggled.connect(update_summary)
		update_summary(); layout.addWidget(summary)
		buttons = PySide6.QtWidgets.QDialogButtonBox(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok | PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel, dialog,
		)
		buttons.accepted.connect(dialog.accept); buttons.rejected.connect(dialog.reject); layout.addWidget(buttons)
		finalize_dialog_accessibility(dialog)
		dialog.finished.connect(
			lambda result: self._finish_haworth_choice(
				dialog, summary.text(), result,
			),
		)
		self._haworth_chooser = dialog
		dialog.open()

	def _finish_haworth_choice(self, dialog: PySide6.QtWidgets.QDialog,
			recipe: str, result: int) -> None:
		"""Arm one placement only after this parented modal chooser accepts."""
		if getattr(self, "_haworth_chooser", None) is dialog:
			self._haworth_chooser = None
		if result != PySide6.QtWidgets.QDialog.DialogCode.Accepted:
			return
		tab = self._active_native_tab()
		if tab is None or tab.requires_refresh:
			return
		self._cancel_haworth_intent()
		snapshot = tab.current_snapshot
		intent = _HaworthIntent(tab, tab.view.viewport(), snapshot.revision, snapshot.digest, recipe)
		self._haworth_intent = intent
		intent.viewport.installEventFilter(self); intent.viewport.setFocus()
		self._refresh_cancel_tool_action()
		self.statusBar().showMessage(self.tr("Click an empty page location to insert {0}; Escape cancels.").format(recipe))

	def eventFilter(self, watched: object, event: PySide6.QtCore.QEvent) -> bool:
		intent = self._haworth_intent
		if intent is not None and watched is intent.viewport:
			if event.type() == PySide6.QtCore.QEvent.Type.KeyPress and event.key() == PySide6.QtCore.Qt.Key.Key_Escape:
				self._cancel_haworth_intent(); return True
			if event.type() == PySide6.QtCore.QEvent.Type.MouseButtonPress and event.button() == PySide6.QtCore.Qt.MouseButton.LeftButton:
				self._place_haworth(intent, event); return True
		return super().eventFilter(watched, event)

	def _place_haworth(self, intent: _HaworthIntent, event: PySide6.QtGui.QMouseEvent) -> None:
		if self._haworth_intent is not intent or intent.tab.current_snapshot.revision != intent.revision or intent.tab.current_snapshot.digest != intent.digest:
			self._cancel_haworth_intent(); self._show_edit_refusal(self._unavailable_edit_refusal("The document changed; choose the Haworth drawing again.")); return
		point = event.position().toPoint(); raw = intent.tab.view.mapToScene(point)
		anchor = intent.tab.view.snap_authored_scene_point(raw)
		if (
			intent.tab.durable_structure_at_viewport_point(point) is not None
			or intent.tab.durable_structure_at_viewport_point(
				intent.tab.view.mapFromScene(anchor),
			) is not None
		):
			self.statusBar().showMessage(self.tr("Choose an empty page location to insert a separate Haworth drawing."), 5000); return
		try:
			prepared = ferrum_qt.ferrum.haworth.prepare_recipe(intent.tab, intent.recipe, anchor)
			preview = ferrum_qt.ferrum.haworth.create_preview(intent.tab, prepared)
			# Session installation replaces the scene. Retire this disposable item
			# while its current scene still owns it, before authoritative repaint.
			self._retire_haworth_preview(preview)
			intent.tab.commit_standalone_haworth(prepared)
		except Exception as exc:
			self._cancel_haworth_intent(); self._show_edit_refusal(self._unavailable_edit_refusal(str(exc))); return
		self._cancel_haworth_intent()
		self.statusBar().showMessage(self.tr("Inserted {0}; use Undo to remove it.").format(intent.recipe), 4000)
		self._refresh_actions()

	def _retire_haworth_preview(self, preview: PySide6.QtWidgets.QGraphicsItem) -> None:
		"""Retire a transient preview through the shared scene-ownership boundary."""
		scene = ferrum_qt.canvas.graphics_retirement.native_scene_for_item(preview)
		if scene is not None:
			coordinator = ferrum_qt.canvas.graphics_retirement.GraphicsRetirementCoordinator()
			coordinator.retire_scene_projection_items(scene, [preview])

	def _cancel_haworth_intent(self) -> None:
		intent = getattr(self, "_haworth_intent", None)
		if intent is not None:
			intent.viewport.removeEventFilter(self)
		self._haworth_intent = None
		if hasattr(self, "_cancel_tool_action"):
			self._refresh_cancel_tool_action()
