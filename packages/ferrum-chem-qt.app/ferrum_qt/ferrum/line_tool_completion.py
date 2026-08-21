"""Ferrum specialised line-gesture preview and commit handlers."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.bond_preview
import ferrum_qt.ferrum.direct_bond_preview
import ferrum_qt.ferrum.direct_root_preview
import ferrum_qt.ferrum.line_tool_intent
import ferrum_qt.ferrum.presentation_creation_preview
import ferrum_qt.ferrum.presentation_vector_preview
import ferrum_qt.ferrum.regular_ring
import ferrum_qt.ferrum.rotation
import ferrum_qt.ferrum.text_placement
import ferrum_qt.ferrum.text_placement_preview
import ferrum_qt.ferrum.top_level_transform
import ferrum_qt.ferrum.translation

_NativeLineTool = ferrum_qt.ferrum.line_tool_intent._NativeLineTool
_LineGestureIntent = ferrum_qt.ferrum.line_tool_intent._LineGestureIntent


class FerrumNativeLineToolCompletionMixin:
	"""Own specialised Rust preview and commit interactions."""
	def _update_direct_bond_gesture(self, intent: _LineGestureIntent,
			viewport_point: PySide6.QtCore.QPoint) -> None:
		"""Admit one Rust candidate and project only its copied receipt overlay."""
		if not self._line_gesture_is_current(intent):
			self._cancel_line_gesture()
			self._show_edit_refusal(self._unavailable_edit_refusal(
				"The document changed during the gesture; no operation was accepted.",
			))
			return
		gesture = intent.direct_bond_gesture
		if gesture is None:
			return
		import ferrum_qt.ferrum.direct_bond_gesture_tab as direct_bond_gesture_tab
		try:
			endpoint_pick = intent.tab._classify_direct_bond_endpoint_at_viewport_point(
				viewport_point,
			)
			if type(endpoint_pick) is direct_bond_gesture_tab._DirectBondAmbiguous:
				self._retire_line_preview(intent.preview)
				self._cancel_line_gesture(clear_status=False)
				self._show_edit_refusal(self._unavailable_edit_refusal(
					"Draw Bond is unchanged. Choose one atom clearly or an empty endpoint, then start again.",
					))
				return
			if type(endpoint_pick) is direct_bond_gesture_tab._DirectBondExistingAtom:
				endpoint = intent.tab.direct_bond_existing_endpoint(endpoint_pick.source_id)
			elif type(endpoint_pick) is direct_bond_gesture_tab._DirectBondEmptySpace:
				point = intent.tab.view.mapToScene(viewport_point)
				endpoint = intent.tab.direct_bond_new_endpoint(float(point.x()), float(point.y()))
			else:
				raise RuntimeError("Ferrum direct-bond endpoint classifier returned an unknown result")
			outcome = intent.tab.admit_direct_bond_candidate(gesture, endpoint)
			self._retire_line_preview(intent.preview)
			import ferrum_qt.ferrum.engine as engine
			if type(outcome) is engine.DirectBondAdmissionRefusalV1:
				self._cancel_line_gesture(clear_status=False)
				self._show_edit_refusal(self._unavailable_edit_refusal(
					self._direct_bond_refusal_message(outcome),
					))
				return
			if type(outcome) is not engine.DirectBondAdmissionV2:
				raise RuntimeError("Ferrum direct-bond admission returned an unknown result")
			overlay = ferrum_qt.ferrum.direct_bond_preview.create_overlay(
				intent.tab, outcome.overlay,
			)
		except Exception:
			self._retire_line_preview(intent.preview)
			self._cancel_line_gesture(clear_status=False)
			raise
		self._line_gesture_intent = dataclasses.replace(
			intent, preview=overlay, direct_bond_admission=outcome,
		)

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
			overlay = ferrum_qt.ferrum.presentation_creation_preview.create_straight_presentation_arrow_overlay(
				intent.tab, preview.overlay,
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
			if current.tool is _NativeLineTool.DRAW_ARROW:
				commit = current.tab.commit_straight_normal_arrow_gesture(
					current.presentation_gesture, current.presentation_preview,
				)
			else:
				commit = current.tab.commit_straight_equilibrium_arrow_gesture(
					current.presentation_gesture, current.presentation_preview,
				)
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
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = current.tab.observe_direct_root_interaction()
			selection = current.tab.select_direct_roots(
				observation, None, engine.RenderInteractionQueryV1.root(
					commit.root.identifier, engine.RenderInteractionModifierV1.replace,
				),
			)
		except Exception:
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
		self._replace_render_interaction_selection(selection, current.tab)
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
			prepared = current.tab.prepare_presentation_vector_gesture(
				current.vector_gesture, current.vector_preview,
			)
			self._reset_line_gesture_start()
			commit = current.tab.commit_presentation_vector_gesture(prepared)
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
		except Exception as exc:
			self._cancel_line_gesture()
			self._refresh_actions()
			self._show_edit_refusal(self._vector_gesture_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = current.tab.observe_direct_root_interaction()
			selection = current.tab.select_direct_roots(
				observation, None,
				engine.RenderInteractionQueryV1.root(commit.identifier),
			)
			self._replace_render_interaction_selection(selection, current.tab)
		except Exception:
			self._replace_render_interaction_selection(None, current.tab)
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
			commit = intent.tab.commit_plus_placement_gesture(
				intent.presentation_gesture, intent.presentation_preview,
			)
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
		except Exception as exc:
			self._cancel_line_gesture()
			self._show_edit_refusal(self._presentation_gesture_refusal(exc))
			return
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = intent.tab.observe_direct_root_interaction()
			selection = intent.tab.select_direct_roots(
				observation, None,
				engine.RenderInteractionQueryV1.root(commit.identifier),
			)
			self._replace_render_interaction_selection(selection, intent.tab)
		except Exception:
			self._replace_render_interaction_selection(None, intent.tab)
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
		try:
			import ferrum_qt.ferrum.engine as engine
			observation = intent.tab.observe_direct_root_interaction()
			selection = intent.tab.select_direct_roots(
				observation, None, engine.RenderInteractionQueryV1.root(commit.identifier),
			)
			self._replace_render_interaction_selection(selection, intent.tab)
		except Exception:
			self._replace_render_interaction_selection(None, intent.tab)
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
		if refusal.category in (
			engine.DirectBondGestureCategoryV1.self_loop,
			engine.DirectBondAdmissionCategoryV1.self_loop,
		):
			return "Choose a different atom or an empty endpoint, then start Draw Bond again."
		if refusal.category in (
			engine.DirectBondGestureCategoryV1.cross_molecule,
			engine.DirectBondAdmissionCategoryV1.cross_molecule,
		):
			return "Choose an atom in the same molecule, then start Draw Bond again."
		if refusal.category in (
			engine.DirectBondGestureCategoryV1.duplicate_bond,
			engine.DirectBondAdmissionCategoryV1.duplicate_bond,
		):
			return "Those atoms already have a bond. Choose another endpoint, then start Draw Bond again."
		return "Choose a different Draw Bond endpoint, then start the tool again."

	#============================================
	@staticmethod
	def _is_direct_bond_commit_refusal(error: Exception) -> bool:
		"""Accept only closed native receipt-redemption failures for presentation."""
		import ferrum_qt.ferrum.engine as engine
		if type(error) not in (
			engine.DirectBondGestureError,
			engine.RevisionConflictError,
		):
			return False
		return getattr(error, "category", None) in (
			engine.DirectBondCommitCategoryV1.foreign_session,
			engine.DirectBondCommitCategoryV1.stale_revision,
			engine.DirectBondCommitCategoryV1.stale_digest,
			engine.DirectBondCommitCategoryV1.identity_allocation_failed,
			engine.DirectBondCommitCategoryV1.provisional_token_unavailable,
			engine.DirectBondCommitCategoryV1.candidate_application_failed,
			engine.DirectBondCommitCategoryV1.revision_exhausted,
		)

	#============================================
