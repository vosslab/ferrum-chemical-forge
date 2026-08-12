"""Transient drag completion for :class:`EditMode`."""

# Standard Library
import math

# local repo modules
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.items.atom_item


#============================================
class EditDragMixin:
	"""Own drag previews and their durable completion boundaries."""

	def _capture_drag_start_state(
			self, clicked_item: object | None = None,
			) -> None:
		"""Capture atom start positions and choose a snap anchor.

		Args:
			clicked_item: Item under the cursor when drag starts.
		"""
		self._drag_atom_start_positions = {}
		self._drag_presentation_start_geometry = {}
		self._drag_anchor_item = None
		self._drag_anchor_start = None
		self._drag_presentation_authority = "local"
		self._drag_presentation_operation = None
		self._drag_presentation_revision = None
		self._drag_presentation_state = "none"
		self._drag_selection_authority = "local"
		self._drag_selection_operation = None
		self._drag_selection_revision = None
		self._drag_selection_state = "none"
		for item in self._moved_items:
			if not isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				if self._is_presentation_item(item):
					model = item.document_object_model
					self._drag_presentation_start_geometry[id(model)] = (
						model, self._presentation_geometry(model),
					)
				continue
			model = item.atom_model
			self._drag_atom_start_positions[id(item)] = (model.x, model.y)
			if self._drag_anchor_item is None:
				self._drag_anchor_item = item
		if (
			isinstance(clicked_item, ferrum_qt.canvas.items.atom_item.AtomItem)
			and clicked_item in self._moved_items
		):
			self._drag_anchor_item = clicked_item
		if self._drag_anchor_item is not None:
			anchor_model = self._drag_anchor_item.atom_model
			self._drag_anchor_start = (anchor_model.x, anchor_model.y)
		self._capture_presentation_drag_context()
		self._capture_selection_translate_context()

	#============================================
	def _capture_selection_translate_context(self) -> None:
		"""Freeze one mixed drag's origin capability before its preview starts."""
		items = tuple(self._moved_items)
		if not items:
			return
		has_atom = any(
			isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem)
			for item in items
		)
		# A selected presentation lookalike must enter the mixed eligibility gate
		# too.  Otherwise the atom-only route could silently commit while a
		# foreign graphics item remains part of the user's visible selection.
		has_presentation = any(
			getattr(item, "document_object_model", None) is not None
			for item in items
		)
		if not has_atom or not has_presentation:
			return
		document = self._env.document
		if (
			document is None
			or ferrum_qt.canvas.document_projection.selection_translate_targets_for_items(
				document, items,
			) is None
		):
			self._drag_selection_authority = "unavailable"
			self._drag_selection_state = "ineligible"
			return
		self._drag_selection_state = "eligible"
		if self._selection_translate_context is None:
			return
		context = self._selection_translate_context()
		if (
			type(context) is not tuple or len(context) != 2
			or context[0] not in ("backend", "local", "unavailable")
		):
			raise ValueError("Selection translation context returned an unknown state")
		authority, revision = context
		if authority == "backend":
			if type(revision) is not int or self._selection_translate_operation is None:
				raise ValueError("Backend selection translation requires a captured revision")
			self._drag_selection_operation = self._selection_translate_operation
			self._drag_selection_revision = revision
		elif revision is not None:
			raise ValueError("Non-backend selection translation must not capture a revision")
		self._drag_selection_authority = authority

	#============================================
	def _capture_presentation_drag_context(self) -> None:
		"""Freeze one presentation drag's authority, revision, and callback."""
		if not self._moved_items or not all(
			self._is_presentation_item(item) for item in self._moved_items
		):
			return
		document = self._env.document
		if document is None or not all(
			self._is_current_supported_presentation_item(document, item)
			for item in self._moved_items
		):
			self._drag_presentation_authority = "unavailable"
			self._drag_presentation_state = "ineligible"
			return
		self._drag_presentation_state = "eligible"
		if self._presentation_translate_context is None:
			return
		context = self._presentation_translate_context()
		if (
			type(context) is not tuple or len(context) != 2
			or context[0] not in ("backend", "local", "unavailable")
		):
			raise ValueError("Presentation translation context returned an unknown state")
		authority, revision = context
		if authority == "backend":
			if type(revision) is not int or self._presentation_translate_operation is None:
				raise ValueError("Backend presentation translation requires a captured revision")
			self._drag_presentation_operation = self._presentation_translate_operation
			self._drag_presentation_revision = revision
		elif revision is not None:
			raise ValueError("Non-backend presentation translation must not capture a revision")
		self._drag_presentation_authority = authority

	#============================================
	def _is_current_supported_presentation_item(self, document: object, item: object) -> bool:
		"""Return whether one drag item is a durable current presentation binding."""
		model = getattr(item, "document_object_model", None)
		return (
			document.is_current_projection_item(item)
			and getattr(model, "supported", False)
			and getattr(model, "editable", False)
			and model in document.presentation_objects
			and model in document.objects
			and type(getattr(model, "object_id", None)) is str
			and bool(model.object_id)
			and ferrum_qt.canvas.document_projection.is_bound_presentation_projection(item, model)
		)

	#============================================
	def _presentation_only_drag_request(
			self, presentation_changes: list,
			) -> tuple[tuple[tuple[str, str], ...], tuple[float, float]] | None:
		"""Capture a durable presentation-only translation request at release."""
		if self._drag_presentation_state != "eligible" or not presentation_changes:
			return None
		document = self._env.document
		if document is None:
			return None
		items = tuple(self._moved_items)
		root_keys = ferrum_qt.canvas.document_projection.top_level_presentation_keys_for_items(
			document, items,
		)
		if not root_keys:
			return None
		deltas = []
		for _model, before, after in presentation_changes:
			delta = self._presentation_geometry_delta(before, after)
			if delta is None:
				return None
			deltas.append(delta)
		if len(deltas) != len(items):
			return None
		first_delta = deltas[0]
		if any(
			abs(delta[0] - first_delta[0]) >= 1e-6
			or abs(delta[1] - first_delta[1]) >= 1e-6
			for delta in deltas[1:]
		):
			return None
		return root_keys, first_delta

	#============================================
	def _presentation_geometry_delta(
			self, before: tuple, after: tuple,
			) -> tuple[float, float] | None:
		"""Return one exact shared translation when geometry contains no reshape."""
		before_points, before_bounds = before
		after_points, after_bounds = after
		deltas = []
		if len(before_points) != len(after_points):
			return None
		for before_point, after_point in zip(before_points, after_points, strict=True):
			before_x, before_y, before_z = before_point
			after_x, after_y, after_z = after_point
			if before_z != after_z:
				return None
			deltas.append((after_x - before_x, after_y - before_y))
		if before_bounds is None or after_bounds is None:
			if before_bounds != after_bounds:
				return None
		else:
			before_x, before_y, before_width, before_height = before_bounds
			after_x, after_y, after_width, after_height = after_bounds
			if before_width != after_width or before_height != after_height:
				return None
			deltas.append((after_x - before_x, after_y - before_y))
		if not deltas:
			return None
		first_delta = deltas[0]
		if any(
			abs(delta[0] - first_delta[0]) >= 1e-6
			or abs(delta[1] - first_delta[1]) >= 1e-6
			for delta in deltas[1:]
		):
			return None
		return first_delta

	#============================================
	def _atom_only_drag_request(
			self, items_and_offsets: list, presentation_changes: list,
			) -> tuple[tuple[tuple[str, str], ...], tuple[float, float]] | None:
		"""Capture one durable atom-only drag request from the live preview.

		The caller must restore the preview before submitting this result because
		an accepted callback can retire the entire current projection.
		"""
		if presentation_changes or not items_and_offsets:
			return None
		if not self._moved_items or any(
			not isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem)
			for item in self._moved_items
		):
			return None
		if len(items_and_offsets) != len(self._moved_items):
			return None
		first_delta = (items_and_offsets[0][1], items_and_offsets[0][2])
		if any(
			abs(dx - first_delta[0]) >= 1e-6 or abs(dy - first_delta[1]) >= 1e-6
			for _item, dx, dy in items_and_offsets[1:]
		):
			return None
		targets = []
		for item, _dx, _dy in items_and_offsets:
			model = item.atom_model
			atom_id = getattr(model, "backend_durable_id", None)
			molecule = getattr(model, "_molecule_model", None)
			molecule_id = getattr(molecule, "mol_id", None)
			if not isinstance(atom_id, str) or not atom_id:
				return None
			if not isinstance(molecule_id, str) or not molecule_id:
				return None
			targets.append((molecule_id, atom_id))
		return tuple(targets), first_delta

	#============================================
	def _atom_drag_offsets(self) -> list:
		"""Return local atom-wrapper offsets after authority routing is settled."""
		items_and_offsets = []
		for item in self._moved_items:
			if not isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				continue
			start_pos = self._drag_atom_start_positions.get(id(item))
			if start_pos is None:
				continue
			model = item.atom_model
			dx = model.x - start_pos[0]
			dy = model.y - start_pos[1]
			if abs(dx) < 1e-6 and abs(dy) < 1e-6:
				continue
			items_and_offsets.append((item, dx, dy))
		return items_and_offsets

	#============================================
	def _mixed_selection_drag_plan(
			self,
			) -> tuple[
				tuple[tuple[str, str], ...], tuple[tuple[str, str], ...], tuple[float, float],
				] | None:
		"""Return a plain mixed-drag request while all graphics stay frame-local.

		This is deliberately a distinct stack frame: accepted session submission
		can synchronously replace the projection, so its caller must retain only
		durable IDs and scalar deltas after this method returns.
		"""
		if self._drag_selection_state != "eligible":
			return None
		items_and_offsets = self._atom_drag_offsets()
		presentation_changes = self._presentation_drag_changes()
		if not items_and_offsets or not presentation_changes:
			return None
		document = self._env.document
		if document is None:
			return None
		targets = ferrum_qt.canvas.document_projection.selection_translate_targets_for_items(
			document, tuple(self._moved_items),
		)
		if targets is None:
			return None
		atom_targets, presentation_keys = targets
		atom_items = [
			item for item in self._moved_items
			if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem)
		]
		if len(items_and_offsets) != len(atom_items):
			return None
		if len(presentation_changes) != len(presentation_keys):
			return None
		deltas = [(dx, dy) for _item, dx, dy in items_and_offsets]
		for _model, before, after in presentation_changes:
			delta = self._presentation_geometry_delta(before, after)
			if delta is None:
				return None
			deltas.append(delta)
		first_delta = deltas[0]
		if not all(math.isfinite(value) for value in first_delta):
			return None
		if any(
			abs(delta[0] - first_delta[0]) >= 1e-6
			or abs(delta[1] - first_delta[1]) >= 1e-6
			or not all(math.isfinite(value) for value in delta)
			for delta in deltas[1:]
		):
			return None
		return atom_targets, presentation_keys, first_delta

	#============================================
	def _is_atom_only_drag(self, presentation_changes: list) -> bool:
		"""Return whether the current drag selection contains only atom projections."""
		return bool(self._moved_items) and not presentation_changes and all(
			isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem)
			for item in self._moved_items
		)

	#============================================
	def _drag_authority(self) -> str:
		"""Return the current session's explicit atom-drag authority state."""
		if self._atom_translate_authority is None:
			return "local"
		authority = self._atom_translate_authority()
		if authority in ("backend", "local", "unavailable"):
			return authority
		raise ValueError("Atom translation authority returned an unknown state")

	#============================================
	def _restore_drag_preview(self) -> None:
		"""Restore every transient atom and presentation geometry preview."""
		for item in self._moved_items:
			if not isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
				continue
			start = self._drag_atom_start_positions.get(id(item))
			if start is None:
				continue
			model = item.atom_model
			model.x, model.y = start
		for object_model, geometry in self._drag_presentation_start_geometry.values():
			points, bounds = geometry
			object_model.set_points(points)
			object_model.set_bounds(bounds)

	#============================================
	def _submit_atom_drag(
			self, targets: tuple[tuple[str, str], ...], delta: tuple[float, float],
			) -> None:
		"""Submit a plain captured atom drag through its originating session seam."""
		if self._atom_translate_operation is None:
			self.status_message.emit("Move unavailable for this document")
			return
		outcome = self._atom_translate_operation(targets, delta)
		self.status_message.emit(outcome.message)

	#============================================
	def _submit_presentation_drag(
			self, operation: object, revision: object,
			root_keys: tuple[tuple[str, str], ...], delta: tuple[float, float],
			) -> None:
		"""Submit one captured durable presentation move through its origin seam."""
		if not callable(operation) or type(revision) is not int:
			self.status_message.emit("Move unavailable for this document")
			return
		outcome = operation(revision, "translate", root_keys, delta=delta)
		self.status_message.emit(outcome.message)

	#============================================
	def _submit_selection_drag(
			self, operation: object, revision: object,
			atom_targets: tuple[tuple[str, str], ...],
			presentation_keys: tuple[tuple[str, str], ...], delta: tuple[float, float],
			) -> None:
		"""Submit one frozen mixed drag through its originating session seam."""
		if not callable(operation) or type(revision) is not int:
			self.status_message.emit("Move unavailable for this document")
			return
		outcome = operation(revision, atom_targets, presentation_keys, delta)
		self.status_message.emit(outcome.message)

	#============================================
	def _reset_drag_state(self) -> None:
		"""Drop all transient drag wrappers after local completion or submission."""
		self._dragging = False
		self._drag_start = None
		self._drag_last = None
		self._drag_anchor_item = None
		self._drag_anchor_start = None
		self._drag_atom_start_positions = {}
		self._drag_presentation_start_geometry = {}
		self._drag_presentation_authority = "local"
		self._drag_presentation_operation = None
		self._drag_presentation_revision = None
		self._drag_presentation_state = "none"
		self._drag_selection_authority = "local"
		self._drag_selection_operation = None
		self._drag_selection_revision = None
		self._drag_selection_state = "none"
		self._moved_items = []
		self._rubber_band_origin = None
		self._cancel_rubber_band()

	#============================================
