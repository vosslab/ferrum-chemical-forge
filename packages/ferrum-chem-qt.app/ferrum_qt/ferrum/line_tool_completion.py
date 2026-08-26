"""Ferrum specialised line-gesture preview and commit handlers."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.direct_bond_gesture_tab
import ferrum_qt.ferrum.direct_bond_overlay
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.curved_equilibrium_arrow
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.ferrum.presentation_creation_preview
import ferrum_qt.ferrum.presentation_vector_preview
import ferrum_qt.ferrum.presentation_path_preview
import ferrum_qt.ferrum.regular_ring
import ferrum_qt.ferrum.rotation
import ferrum_qt.ferrum.text_placement
import ferrum_qt.ferrum.text_placement_preview
import ferrum_qt.ferrum.terminal_arrow

_NativeLineTool = ferrum_qt.ferrum.line_tool_intent._NativeLineTool
_LineGestureIntent = ferrum_qt.ferrum.line_tool_intent._LineGestureIntent


class FerrumNativeLineToolCompletionMixin:
	"""Own specialised Rust preview and commit interactions."""
	#============================================
	def _update_direct_bond_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Replace the preview with one exact Rust-admitted direct-bond candidate."""
		start_probe = intent.direct_bond_start_probe
		gesture = intent.direct_bond_gesture
		if start_probe is None or gesture is None or intent.drawing is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during Draw Bond; refresh and start again.",
			))
			return
		import ferrum_qt.ferrum.engine as engine
		try:
			end_probe = intent.tab.direct_bond_pointer_probe_at_viewport_point(viewport_point)
			request = intent.tab.resolve_direct_bond_end(gesture, end_probe)
			prepared = intent.tab.prepare_session_operation_transition_v1(request)
		except engine.DirectBondPointerProbeErrorV3 as exc:
			self._cancel_line_gesture()
			self._show_direct_bond_refusal(exc)
			return
		except engine.DirectBondAdmissionRefusalV3 as exc:
			self._cancel_line_gesture()
			self._show_direct_bond_refusal(exc)
			return
		except engine.OperationValidationError as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(str(exc)))
			return
		if type(prepared) is not engine.PreparedSessionTransitionV1:
			self._cancel_line_gesture()
			raise RuntimeError("Ferrum direct-bond preparation returned an unknown transition")
		try:
			next_gesture = intent.tab.begin_direct_bond_gesture(
				start_probe,
				intent.drawing.bond_presentation(intent.direct_bond_presentation),
				intent.direct_bond_snap_enabled,
			)
		except Exception as exc:
			self._cancel_line_gesture()
			if self._is_direct_bond_begin_refusal(exc):
				self._show_direct_bond_refusal(exc)
				return
			raise
		try:
			presentation = prepared.presentation_v1()
			overlay_contract = presentation.precommit_overlay
			if overlay_contract is None:
				raise RuntimeError("Ferrum direct-bond transition has no precommit overlay")
			overlay = ferrum_qt.ferrum.direct_bond_overlay.create_overlay(
				intent.tab, overlay_contract,
			)
		except Exception:
			self._cancel_line_gesture()
			raise
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, direct_bond_gesture=next_gesture,
			prepared_transition=prepared,
		)
	#============================================
	def _commit_direct_bond_transition(self, tab: object, prepared: object) -> object:
		"""Redeem a generic transition and apply Draw Bond's caller-owned selection."""
		result = tab.commit_session_operation_transition_v1(prepared)
		outcome = result.outcome
		if outcome.kind != "direct_bond_v1" or outcome.direct_bond is None:
			raise RuntimeError("Ferrum direct-bond transition returned an unknown outcome")
		bond_document_object_id = outcome.direct_bond.bond_document_object_id
		if type(bond_document_object_id) is not str or not bond_document_object_id:
			raise RuntimeError("Ferrum direct-bond transition returned an invalid bond document ID")
		tab._install_mutation_result(result, (bond_document_object_id,))
		return result

	#============================================
	def _commit_created_presentation_root_transition(self, tab: object,
			request: object,
			expected_root_kind: "ferrum_qt.ferrum.engine.CreatedPresentationRootKindV1",
			recovery_message: str,
			) -> tuple[object, str] | None:
		"""Redeem one visual request through the generic authority and install Rust truth."""
		import ferrum_qt.ferrum.engine as engine
		if type(expected_root_kind) is not engine.CreatedPresentationRootKindV1:
			raise TypeError("Ferrum visual transitions require a typed expected root kind")
		try:
			prepared = tab.prepare_session_operation_transition_v1(request)
			result = tab.commit_session_operation_transition_v1(prepared)
		except (engine.OperationValidationError, engine.PreparedOperationError):
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._unavailable_edit_refusal(recovery_message))
			return None
		outcome = result.outcome
		created = outcome.created_presentation_root
		if outcome.kind != "created_presentation_root_v1" or created is None:
			raise RuntimeError("Ferrum visual transition returned an unknown operation outcome")
		if created.kind != expected_root_kind:
			raise RuntimeError("Ferrum visual transition returned an unexpected root kind")
		root_document_object_id = created.document_object_id
		if type(root_document_object_id) is not str or not root_document_object_id:
			raise RuntimeError("Ferrum visual transition returned an invalid root document ID")
		tab._install_mutation_result(result, (root_document_object_id,))
		return result, root_document_object_id

	#============================================
	def _restore_created_presentation_root_selection(self, tab: object,
			root_document_object_id: str) -> bool:
		"""Select one committed root unless Rust reports the fresh interaction is stale."""
		import ferrum_qt.ferrum.engine as engine
		from ferrum_qt.ferrum.document_tab_errors import FerrumNativeDocumentTabMutationPresentationError
		try:
			observation = tab.observe_direct_root_interaction()
			selection = tab.select_direct_roots(
				observation, None, engine.RenderInteractionQueryV1.root(
					root_document_object_id, engine.RenderInteractionModifierV1.replace,
				),
			)
			self._replace_render_interaction_selection(selection, tab)
		except (
			engine.RevisionConflictError,
			engine.RenderInteractionError,
			FerrumNativeDocumentTabMutationPresentationError,
		):
			self._replace_render_interaction_selection(None, tab)
			return False
		return True
	#============================================
	def _update_terminal_arrow_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Paint one closed Rust-issued terminal arrow after start/control capture."""
		state = intent.terminal_arrow
		if state is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				f"The document changed during {state.kind.description} drawing; refresh and start again.",
			))
			return
		if intent.presentation_gesture is None or len(state.points) < 2:
			return
		end = state.points[2] if len(state.points) == 3 else None
		if end is None:
			point = intent.tab.view.mapToScene(viewport_point)
			end = (float(point.x()), float(point.y()))
		try:
			preview = intent.tab.preview_terminal_arrow_gesture(
				state.kind, intent.presentation_gesture, end,
			)
			overlay = ferrum_qt.ferrum.presentation_creation_preview.create_arrow_preview(
				intent.tab, preview.plan,
			)
		except Exception as exc:
			if ferrum_qt.ferrum.terminal_arrow.needs_endpoint(state, exc):
				self._retire_line_preview(intent.preview)
				self._show_terminal_arrow_point_guidance(state.kind, 2)
				return
			self._cancel_line_gesture(clear_status=False)
			if ferrum_qt.ferrum.terminal_arrow.is_native_error(state.kind, exc):
				self._show_edit_refusal(self._terminal_arrow_refusal(state.kind, exc))
				return
			raise
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, presentation_preview=preview,
		)

	#============================================
	def _complete_terminal_arrow_gesture(self, intent: _LineGestureIntent) -> None:
		"""Prepare, commit, and select one complete opaque terminal-arrow receipt."""
		state = intent.terminal_arrow
		if state is None:
			return
		if len(state.points) < 3:
			self._show_terminal_arrow_point_guidance(state.kind, len(state.points))
			return
		self._update_terminal_arrow_gesture(intent, PySide6.QtCore.QPoint())
		current = self._line_gesture_intent
		if current is None or current.presentation_gesture is None or current.presentation_preview is None:
			return
		current_state = current.terminal_arrow
		if current_state is None:
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			request = current.tab.resolve_terminal_arrow_gesture(
				current_state.kind, current.presentation_gesture, current.presentation_preview,
			)
			self._reset_line_gesture_start()
			committed = self._commit_created_presentation_root_transition(
				current.tab, request, engine.CreatedPresentationRootKindV1.curved_terminal_arrow,
				f"{current_state.kind.action_name} is unchanged. Refresh the Rust view and start the tool again.",
			)
			if committed is None:
				return
			_result, root_document_object_id = committed
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, current.tab)
			current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				f"Added one Ferrum {current_state.kind.description}; the display was refreshed after installation recovery.",
			))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			if ferrum_qt.ferrum.terminal_arrow.is_native_error(current_state.kind, exc):
				self._show_edit_refusal(self._terminal_arrow_refusal(current_state.kind, exc))
				return
			raise
		self._restore_created_presentation_root_selection(current.tab, root_document_object_id)
		self._finish_line_gesture(current, self.tr(
			f"Added one Ferrum {current_state.kind.description}; click a new start point or press Esc.",
		))

	#============================================
	def _show_terminal_arrow_point_guidance(self,
			kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind, point_count: int) -> None:
		"""Explain the next finite point in one closed three-click arrow contract."""
		remaining = 3 - point_count
		if remaining > 0:
			point_word = "point" if remaining == 1 else "points"
			self.statusBar().showMessage(self.tr(
				"{0} needs {1} more {2}; click start, bend, and endpoint in order."
			).format(kind.action_name, remaining, point_word), 5000)

	#============================================
	def _update_curved_equilibrium_arrow_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Paint only the two cubic lanes and heads issued by Rust."""
		state = intent.curved_equilibrium_arrow
		if state is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during curved equilibrium arrow drawing; refresh and start again.",
			))
			return
		if intent.presentation_gesture is None or len(state.points) < 2:
			return
		end = state.points[2] if len(state.points) == 3 else None
		if end is None:
			point = intent.tab.view.mapToScene(viewport_point)
			end = (float(point.x()), float(point.y()))
		try:
			preview = intent.tab.preview_curved_equilibrium_arrow_gesture(
				intent.presentation_gesture, end,
			)
		except Exception as exc:
			if not ferrum_qt.ferrum.curved_equilibrium_arrow.is_native_error(exc):
				raise
			self._cancel_line_gesture(clear_status=False)
			self._show_curved_equilibrium_arrow_refusal(exc)
			return
		overlay = ferrum_qt.ferrum.presentation_creation_preview.create_arrow_preview(
			intent.tab, preview.plan,
		)
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, presentation_preview=preview,
		)

	#============================================
	def _complete_curved_equilibrium_arrow_gesture(self, intent: _LineGestureIntent) -> None:
		"""Prepare, commit, and select one opaque curved-equilibrium receipt."""
		state = intent.curved_equilibrium_arrow
		if state is None:
			return
		if len(state.points) < 3:
			self._show_curved_equilibrium_arrow_point_guidance(len(state.points))
			return
		self._update_curved_equilibrium_arrow_gesture(intent, PySide6.QtCore.QPoint())
		current = self._line_gesture_intent
		if current is None or current.presentation_gesture is None or current.presentation_preview is None:
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			request = current.tab.resolve_curved_equilibrium_arrow_gesture(
				current.presentation_gesture, current.presentation_preview,
			)
			self._reset_line_gesture_start()
			committed = self._commit_created_presentation_root_transition(
				current.tab, request, engine.CreatedPresentationRootKindV1.curved_equilibrium_arrow,
				"Curved equilibrium arrow is unchanged. Refresh the Rust view and start the tool again.",
			)
			if committed is None:
				return
			_result, root_document_object_id = committed
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, current.tab)
			recovered = current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum curved equilibrium arrow; the display was refreshed after installation recovery."
				if recovered else
				"Added one Ferrum curved equilibrium arrow. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The curved equilibrium arrow was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The curved equilibrium arrow was added, but its authoritative display still needs recovery; refresh before saving or editing.",
			))
			return
		except Exception as exc:
			if not ferrum_qt.ferrum.curved_equilibrium_arrow.is_native_error(exc):
				raise
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_curved_equilibrium_arrow_refusal(exc)
			return
		if not self._restore_created_presentation_root_selection(
				current.tab, root_document_object_id,
		):
			# The accepted Rust mutation remains durable, while selection is a separate
			# authoritative observation. Recover the installed scene before describing success.
			self._replace_render_interaction_selection(None, current.tab)
			recovered = current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum curved equilibrium arrow; the display was refreshed after selection recovery."
				if recovered else
				"Added one Ferrum curved equilibrium arrow. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The curved equilibrium arrow was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The curved equilibrium arrow was added, but its durable selection could not be restored; refresh or reopen before selecting or moving it.",
			))
			return
		self._finish_line_gesture(current, self.tr(
			"Added one Ferrum curved equilibrium arrow; click a new start point or press Esc.",
		))

	#============================================
	def _show_curved_equilibrium_arrow_point_guidance(self, point_count: int) -> None:
		"""Explain the remaining points in the dedicated three-click contract."""
		remaining = 3 - point_count
		if remaining <= 0:
			return
		point_word = "point" if remaining == 1 else "points"
		self.statusBar().showMessage(self.tr(
			"Draw Curved Equilibrium Arrow needs {0} more {1}; click start, bend, and endpoint in order."
		).format(remaining, point_word), 5000)

	#============================================
	def _show_presentation_path_point_guidance(self,
			tool: _NativeLineTool, progress: object) -> None:
		"""State the remaining clicks for one Rust-owned open or closed path."""
		remaining = progress.minimum_point_count - progress.accepted_point_count
		if remaining <= 0:
			return
		point_word = "point" if remaining == 1 else "points"
		path_name = "Polygon" if tool is _NativeLineTool.DRAW_POLYGON else "Polyline"
		self.statusBar().showMessage(self.tr(
			"Draw {0} needs {1} more {2}; click ordered points, then press Enter or double-click."
		).format(path_name, remaining, point_word), 5000)

	#============================================
	def _update_presentation_path_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint | None) -> None:
		"""Project only the exact Rust-issued open or closed path preview."""
		if intent.path_gesture is None or intent.path_progress is None:
			return
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during path drawing; refresh and start the tool again.",
			))
			return
		hover = None
		if viewport_point is not None:
			point = intent.tab.view.mapToScene(viewport_point)
			hover = (float(point.x()), float(point.y()))
		import ferrum_qt.ferrum.engine as engine
		try:
			preview = intent.tab.preview_presentation_path_gesture(
				intent.path_gesture, hover,
			)
			overlay = ferrum_qt.ferrum.presentation_path_preview.create_overlay(
				intent.tab, preview,
			)
		except engine.PresentationPathGestureError as exc:
			self._retire_line_preview(intent.preview)
			self._line_gesture_intent = dataclasses.replace(
				intent, preview=None, path_preview=None,
			)
			self._show_presentation_path_refusal(exc)
			return
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, path_preview=preview,
		)

	#============================================
	def _complete_presentation_path_gesture(self, intent: _LineGestureIntent) -> None:
		"""Preflight and commit one complete Rust-owned Polyline or Polygon."""
		if intent.path_progress is None:
			return
		if not intent.path_progress.can_complete:
			self._show_presentation_path_point_guidance(intent.tool, intent.path_progress)
			return
		self._update_presentation_path_gesture(
			intent, None,
		)
		current = self._line_gesture_intent
		if current is None or current.path_gesture is None or current.path_preview is None:
			return
		import ferrum_qt.ferrum.engine as engine
		try:
			request = current.tab.resolve_presentation_path_gesture(
				current.path_gesture, current.path_preview,
			)
			self._reset_line_gesture_start()
			committed = self._commit_created_presentation_root_transition(
				current.tab, request, engine.CreatedPresentationRootKindV1.path,
				"Path drawing is unchanged. Refresh the Rust view and start the tool again.",
			)
			if committed is None:
				return
			_result, root_document_object_id = committed
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, current.tab)
			current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum path; the display was refreshed after installation recovery.",
			))
			return
		except engine.PresentationPathGestureError as exc:
			if not self._presentation_path_refusal_allows_continuation(exc):
				self._cancel_line_gesture()
				self._refresh_actions()
			self._show_presentation_path_refusal(exc)
			return
		self._restore_created_presentation_root_selection(current.tab, root_document_object_id)
		self._finish_line_gesture(current, self.tr(
			"Added one Ferrum {0}; click a new start point or press Esc."
		).format("Polygon" if current.tool is _NativeLineTool.DRAW_POLYGON else "Polyline"))

	#============================================
	def _show_presentation_path_refusal(self, error: Exception) -> None:
		"""Show a typed path refusal without interrupting canvas authoring."""
		self.statusBar().showMessage(self.tr(self._presentation_path_refusal(error)), 5000)

	#============================================
	def _presentation_path_refusal(self, error: Exception) -> str:
		"""Translate typed native path facts into concise nonmodal guidance."""
		import ferrum_qt.ferrum.engine as engine
		if type(error) is engine.PresentationPathGestureError:
			if error.category == engine.PresentationPathGestureCategoryV1.incomplete:
				return "Ferrum needs more ordered points before this path can be completed."
			if error.recovery == engine.PresentationPathGestureRecoveryV1.change_geometry:
				return "Ferrum rejected that point. Choose a different path point."
			if error.recovery == engine.PresentationPathGestureRecoveryV1.refresh_and_restart:
				return "The document changed during path drawing. Refresh and start the path again."
			if error.recovery == engine.PresentationPathGestureRecoveryV1.document_unchanged:
				return "Ferrum left the document unchanged. Start a new path when ready."
		return "Ferrum could not complete this path."

	#============================================
	@staticmethod
	def _presentation_path_refusal_allows_continuation(error: Exception) -> bool:
		"""Keep the tool armed only when Rust requests a geometry correction."""
		import ferrum_qt.ferrum.engine as engine
		return (
			type(error) is engine.PresentationPathGestureError
			and error.recovery == engine.PresentationPathGestureRecoveryV1.change_geometry
		)

	#============================================
	def _terminal_arrow_refusal(self, kind: ferrum_qt.ferrum.terminal_arrow.TerminalArrowKind,
			error: Exception) -> object:
		"""Present a closed native terminal-arrow refusal without parsing error text."""
		return self._unavailable_edit_refusal(
			ferrum_qt.ferrum.terminal_arrow.refusal_message(kind, error),
		)

	#============================================
	def _show_curved_equilibrium_arrow_refusal(self, error: Exception) -> None:
		"""Show one typed geometry correction without interrupting canvas authoring."""
		message = ferrum_qt.ferrum.curved_equilibrium_arrow.refusal_message(error)
		self.statusBar().showMessage(self.tr(message), 5000)

	#============================================
	def _update_presentation_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Request and paint exactly one backend-issued Arrow overlay."""
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				f"The document changed during {self._presentation_arrow_tool_name(intent.tool)}; start the tool again.",
			))
			return
		gesture = intent.presentation_gesture
		if gesture is None:
			return
		point = intent.tab.view.mapToScene(viewport_point)
		try:
			preview = intent.tab.preview_straight_presentation_arrow_gesture(
				gesture, float(point.x()), float(point.y()),
			)
			overlay = ferrum_qt.ferrum.presentation_creation_preview.create_arrow_preview(
				intent.tab, preview.plan,
			)
		except Exception as exc:
			self._cancel_line_gesture(clear_status=False)
			self._show_edit_refusal(self._presentation_gesture_refusal(exc, intent.tool))
			return
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, presentation_preview=preview,
		)

	#============================================
	def _complete_presentation_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Commit only the exact Arrow preview and then restore Rust selection."""
		self._update_presentation_gesture(intent, viewport_point)
		current = self._line_gesture_intent
		if current is None or current.presentation_gesture is None or current.presentation_preview is None:
			return
		self._reset_line_gesture_start()
		try:
			import ferrum_qt.ferrum.engine as engine
			request = current.tab.resolve_presentation_creation_gesture(
				current.presentation_gesture, current.presentation_preview,
			)
			expected_root_kind = (
				engine.CreatedPresentationRootKindV1.straight_normal_arrow
				if current.tool is _NativeLineTool.DRAW_ARROW
				else engine.CreatedPresentationRootKindV1.straight_equilibrium_arrow
			)
			committed = self._commit_created_presentation_root_transition(
				current.tab, request, expected_root_kind,
				f"{self._presentation_arrow_tool_name(current.tool)} is unchanged. Refresh the Rust view and start the tool again.",
			)
			if committed is None:
				return
			_result, root_document_object_id = committed
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			# Rust accepted the Arrow and the tab retained its exact pending snapshot.
			# Reproject from that authority; do not reuse the preview or call it refused.
			self._replace_render_interaction_selection(None, current.tab)
			recovered = current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				f"Added one Ferrum {self._presentation_arrow_description(current.tool)}; the display was refreshed after installation recovery."
				if recovered else
				f"Added one Ferrum {self._presentation_arrow_description(current.tool)}. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				f"The {self._presentation_arrow_description(current.tool)} was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				f"The {self._presentation_arrow_description(current.tool)} was added, but its authoritative display still needs recovery; refresh before saving or editing.",
			))
			return
		except engine.PresentationGestureError as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		if not self._restore_created_presentation_root_selection(
				current.tab, root_document_object_id,
		):
			# Commit already installed its accepted Rust snapshot. Selection recovery
			# is secondary and must never describe this persisted Arrow as unchanged.
			self._replace_render_interaction_selection(None, current.tab)
			self._finish_line_gesture(current, self.tr(
				f"Added one Ferrum {self._presentation_arrow_description(current.tool)}. Selection was unavailable; refresh the Rust view before moving it.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				f"The {self._presentation_arrow_description(current.tool)} was added. Its selection could not be restored; refresh the Rust view before selecting or moving it.",
			))
			return
		self._finish_line_gesture(current, self.tr(
			f"Added one Ferrum {self._presentation_arrow_description(current.tool)}; drag again or press Esc.",
		))

	#============================================
	def _update_vector_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Request and paint only one Rust-issued ordinary vector overlay."""
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during vector drawing; refresh the Rust view and start again.",
			))
			return
		if intent.vector_gesture is None:
			return
		point = intent.tab.view.mapToScene(viewport_point)
		try:
			preview = intent.tab.preview_presentation_vector_gesture(
				intent.vector_gesture, float(point.x()), float(point.y()),
			)
			overlay = ferrum_qt.ferrum.presentation_vector_preview.create_overlay(
				intent.tab, preview.overlay,
			)
		except Exception as exc:
			self._cancel_line_gesture(clear_status=False)
			self._show_edit_refusal(self._vector_gesture_refusal(exc))
			return
		self._retire_line_preview(intent.preview)
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, vector_preview=preview,
		)

	#============================================
	def _complete_vector_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Preflight then commit exactly the opaque Rust vector receipt."""
		self._update_vector_gesture(intent, viewport_point)
		current = self._line_gesture_intent
		if current is None or current.vector_gesture is None or current.vector_preview is None:
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			request = current.tab.resolve_presentation_vector_gesture(
				current.vector_gesture, current.vector_preview,
			)
			self._reset_line_gesture_start()
			committed = self._commit_created_presentation_root_transition(
				current.tab, request, engine.CreatedPresentationRootKindV1.vector,
				"Vector drawing is unchanged. Refresh the Rust view and start the tool again.",
			)
			if committed is None:
				return
			_result, root_document_object_id = committed
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, current.tab)
			recovered = current.tab.refresh_authoritative()
			self._finish_line_gesture(current, self.tr(
				"Added one Ferrum vector; the display was refreshed after installation recovery."
				if recovered else
				"Added one Ferrum vector. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The vector was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The vector was added, but its authoritative display still needs recovery; refresh before saving or editing."
			))
			return
		except engine.PresentationVectorGestureError as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._vector_gesture_refusal(exc))
			return
		if not self._restore_created_presentation_root_selection(
				current.tab, root_document_object_id,
		):
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The vector was added, but selection could not be restored; refresh before moving it.",
			))
		self._finish_line_gesture(current, self.tr(
			"Added one Ferrum {0}; drag again or press Esc."
		).format(self._draw_vector_actions[current.tool].text()[5:].lower()))

	#============================================
	def _complete_plus_gesture(self, intent: _LineGestureIntent) -> None:
		"""Commit one backend-owned Plus click and restore durable selection."""
		if (
			not self._line_gesture_is_current(intent)
			or intent.presentation_gesture is None
			or intent.presentation_preview is None
		):
			self._cancel_line_gesture()
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			request = intent.tab.resolve_presentation_creation_gesture(
				intent.presentation_gesture, intent.presentation_preview,
			)
			committed = self._commit_created_presentation_root_transition(
				intent.tab, request, engine.CreatedPresentationRootKindV1.plus,
				"Plus placement is unchanged. Refresh the Rust view and start the tool again.",
			)
			if committed is None:
				return
			_result, root_document_object_id = committed
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, intent.tab)
			recovered = intent.tab.refresh_authoritative()
			self._finish_line_gesture(intent, self.tr(
				"Added one Ferrum Plus; the display was refreshed after installation recovery."
				if recovered else
				"Added one Ferrum Plus. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The Plus was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"The Plus was added, but its authoritative display still needs recovery; refresh or reopen before saving or editing.",
			))
			return
		except engine.PresentationGestureError as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		self._restore_created_presentation_root_selection(intent.tab, root_document_object_id)
		self._finish_line_gesture(intent, self.tr(
			"Added one Ferrum Plus; click again or press Esc.",
		))

	#============================================
	def _complete_text_placement_gesture(self) -> None:
		"""Commit one exact Text preview, then select its durable Rust root."""
		intent = self._line_gesture_intent
		if (
			intent is None or intent.tool is not _NativeLineTool.INSERT_TEXT
			or not self._line_gesture_is_current(intent)
			or intent.text_gesture is None or intent.text_preview is None
		):
			return
		try:
			commit = intent.tab.commit_text_placement_gesture(
				intent.text_gesture, intent.text_preview,
			)
		except ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTabMutationPresentationError:
			self._replace_render_interaction_selection(None, intent.tab)
			recovered = intent.tab.refresh_authoritative()
			self._finish_line_gesture(intent, self.tr(
				"Text added; the display was refreshed after installation recovery."
				if recovered else
				"Text added. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Text was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"Text was added, but its authoritative display still needs recovery; refresh or reopen before saving or editing.",
			))
			return
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._text_placement_refusal(exc))
			return
		if not self._restore_created_presentation_root_selection(
				intent.tab, commit.document_object_id,
		):
			recovered = intent.tab.refresh_authoritative()
			self._finish_line_gesture(intent, self.tr(
				"Text added; the display was refreshed after selection recovery."
				if recovered else
				"Text added. Display recovery is required before further editing.",
			))
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"Text was added. Ferrum refreshed the authoritative Rust display; select it again before moving it."
				if recovered else
				"Text was added, but its durable selection could not be restored; refresh or reopen before selecting or moving it.",
			))
			return
		self._finish_line_gesture(intent, self.tr(
			"Text added; click another page location or press Esc.",
		))

	#============================================
	def _text_placement_refusal(self, error: Exception) -> object:
		"""Present closed Rust Text failures without parsing error prose for policy."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(error, "category", None)
		if category is engine.TextPlacementErrorCategoryV1.unrenderable_standard:
			message = "Text insertion is unchanged. Repair the drawing standard, then start the tool again."
		elif category in (
			engine.TextPlacementErrorCategoryV1.blank_content,
			engine.TextPlacementErrorCategoryV1.unsupported_style,
			engine.TextPlacementErrorCategoryV1.invalid_font_override,
		):
			message = "Text insertion is unchanged. Correct the text or supported formatting and try again."
		elif category in (
			engine.TextPlacementErrorCategoryV1.stale_snapshot,
			engine.TextPlacementErrorCategoryV1.session_conflict,
		):
			message = "Text insertion is unchanged. Refresh the Rust view and start the tool again."
		else:
			message = "Text insertion is unchanged. Choose another location or restart the tool."
		return self._unavailable_edit_refusal(message)

	#============================================
	def _presentation_gesture_refusal(self, error: Exception,
			tool: _NativeLineTool = _NativeLineTool.DRAW_ARROW) -> object:
		"""Map closed Rust Arrow refusal categories to actionable recovery text."""
		import ferrum_qt.ferrum.engine as engine
		tool_name = self._presentation_arrow_tool_name(tool)
		category = getattr(error, "category", None)
		if category in (
			engine.PresentationGestureCategoryV1.collapsed_endpoint,
			engine.PresentationGestureCategoryV1.below_minimum_length,
		):
			message = f"{tool_name} is unchanged. Drag to a clearly different endpoint and try again."
		elif category in (
			engine.PresentationGestureCategoryV1.stale_revision,
			engine.PresentationGestureCategoryV1.stale_digest,
			engine.PresentationGestureCategoryV1.session_conflict,
		):
			message = f"{tool_name} is unchanged. Refresh the Rust view and start the tool again."
		else:
			message = f"{tool_name} is unchanged. Adjust the endpoint or tool style and try again."
		return self._unavailable_edit_refusal(message)

	#============================================
	@staticmethod
	def _presentation_arrow_tool_name(tool: _NativeLineTool) -> str:
		"""Return the user-facing closed presentation-arrow tool name."""
		if tool is _NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
			return "Draw Equilibrium Arrow"
		return "Draw Arrow"

	#============================================
	@staticmethod
	def _presentation_arrow_description(tool: _NativeLineTool) -> str:
		"""Return the user-facing closed presentation-arrow result description."""
		if tool is _NativeLineTool.DRAW_EQUILIBRIUM_ARROW:
			return "equilibrium reaction arrow"
		return "reaction arrow"

	#============================================
	def _vector_gesture_refusal(self, error: Exception) -> object:
		"""Present closed render-bridge vector failures with their recovery class."""
		import ferrum_qt.ferrum.engine as engine
		category = getattr(error, "category", None)
		if category in (
			engine.PresentationVectorGestureCategoryV1.degenerate_geometry,
			engine.PresentationVectorGestureCategoryV1.invalid_point,
		):
			message = "Vector drawing is unchanged. Drag to a clearly different finite endpoint and try again."
		elif category is engine.PresentationVectorGestureCategoryV1.unrenderable_standard:
			message = "Vector drawing is unchanged. Choose a supported drawing appearance, then try again."
		elif category in (
			engine.PresentationVectorGestureCategoryV1.stale_snapshot,
			engine.PresentationVectorGestureCategoryV1.session_conflict,
			engine.PresentationVectorGestureCategoryV1.replayed_gesture,
		):
			message = "Vector drawing is unchanged. Refresh the Rust view and start the tool again."
		else:
			message = "Vector drawing is unchanged. Adjust the shape or drawing appearance and try again."
		return self._unavailable_edit_refusal(message)

	#============================================
	@staticmethod
	def _direct_bond_refusal_message(refusal: object) -> str:
		"""Explain a typed ordinary Rust endpoint refusal without parsing strings."""
		import ferrum_qt.ferrum.engine as engine
		if type(refusal) is engine.DirectBondPointerProbeErrorV3:
			if refusal.recovery == engine.DirectBondPointerProbeRecoveryV3.refresh_and_restart:
				return "Refresh the Rust view and start Draw Bond again."
			if refusal.category == engine.DirectBondPointerProbeCategoryV3.ambiguous_atom:
				return "Choose one atom clearly or an empty endpoint, then start Draw Bond again."
			if refusal.recovery == engine.DirectBondPointerProbeRecoveryV3.adjust_endpoint:
				return "Choose a different atom or empty endpoint, then start Draw Bond again."
			return "Choose a finite pointer position and start Draw Bond again."
		if type(refusal) is engine.DirectBondAdmissionRefusalV3:
			if (
				refusal.category in (
					engine.DirectBondAdmissionCategoryV3.self_loop,
					engine.DirectBondAdmissionCategoryV3.unknown_start_atom,
					engine.DirectBondAdmissionCategoryV3.unknown_end_atom,
					engine.DirectBondAdmissionCategoryV3.invalid_endpoint_input,
					engine.DirectBondAdmissionCategoryV3.collapsed_endpoint,
					engine.DirectBondAdmissionCategoryV3.cross_molecule,
					engine.DirectBondAdmissionCategoryV3.duplicate_bond,
					engine.DirectBondAdmissionCategoryV3.exceeds_chemistry_capacity,
					engine.DirectBondAdmissionCategoryV3.unsupported_chemistry_admission,
				)
				and refusal.recovery == engine.DirectBondAdmissionRecoveryV3.adjust_endpoint
			):
				return "Choose a different atom or empty endpoint, then start Draw Bond again."
			if (
				refusal.category in (
					engine.DirectBondAdmissionCategoryV3.foreign_session,
					engine.DirectBondAdmissionCategoryV3.stale_revision,
					engine.DirectBondAdmissionCategoryV3.stale_digest,
				)
				and refusal.recovery == engine.DirectBondAdmissionRecoveryV3.refresh_and_restart
			):
				return "Refresh the Rust view and start Draw Bond again."
			if (
				refusal.category in (
					engine.DirectBondAdmissionCategoryV3.unsupported_presentation,
					engine.DirectBondAdmissionCategoryV3.unrenderable_candidate,
				)
				and refusal.recovery == engine.DirectBondAdmissionRecoveryV3.change_presentation
			):
				return "Choose a supported bond appearance and start Draw Bond again."
			raise RuntimeError("Ferrum direct-bond admission refusal has an unknown contract pair")
	#============================================
	def _show_direct_bond_refusal(self, refusal: object) -> None:
		"""Publish a typed bond refusal without blocking the canvas event loop."""
		self.statusBar().showMessage(self.tr(
			"Draw Bond refused: {0}"
		).format(self._direct_bond_refusal_message(refusal)), 5000)

	#============================================
