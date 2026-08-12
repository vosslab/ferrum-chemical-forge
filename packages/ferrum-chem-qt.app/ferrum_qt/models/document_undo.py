"""Undo-state and terminal graphics-retirement helpers for Document."""

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
class DocumentUndoStack(PySide6.QtGui.QUndoStack):
	"""Keep terminal undo-history disposal under the owning Document.

	Qt retains graphics wrappers in structural commands while they remain
	undoable.  When a new command replaces an undone redo branch, however, Qt
	destroys those commands immediately.  This small history surface gives the
	Document a last explicit opportunity to retire the branch's already-detached
	graphics before Qt releases the commands themselves.
	"""

	#============================================
	def __init__(self, document: object) -> None:
		"""Create a QUndoStack whose terminal history transitions use ``document``."""
		super().__init__(document)

	#============================================
	def _owner(self) -> object:
		"""Return the still-live QObject parent that owns this history surface."""
		# The facade imports this module, so defer its runtime identity check.
		import ferrum_qt.models.document as document_module
		document = self.parent()
		if not isinstance(document, document_module.Document):
			raise RuntimeError("Document undo history has no live document owner")
		return document

	#============================================
	def push(self, command: PySide6.QtGui.QUndoCommand) -> None:
		"""Retire an obsolete redo branch before accepting its replacement."""
		self._owner()._retire_discarded_redo_graphics()
		super().push(command)

	#============================================
	def clear(self) -> None:
		"""Retire detached command graphics before Qt clears its history."""
		self._owner()._retire_all_history_graphics()
		super().clear()

	#============================================
	def setUndoLimit(self, limit: int) -> None:
		"""Keep graphics-retaining history unlimited until eviction is explicit.

		Qt may evict commands without a Python callback when a finite undo limit is
		configured.  The current history contract is unlimited, so rejecting a
		finite value makes a future eviction implementation choose and test its own
		terminal graphics handoff rather than silently dropping native wrappers.
		"""
		if limit != 0:
			raise ValueError(
				"Document undo history requires unlimited capacity until eviction "
				"owns graphics retirement",
			)
		super().setUndoLimit(limit)


#============================================
class DocumentUndoState:
	#============================================
	def set_graphics_retirement_reaper(
			self,
			reaper: object,
			) -> None:
		"""Assign this projection's session-owned terminal graphics reaper.

		The reaper is a frontend lifetime capability rather than a backend-facing
		contract.  A bare Document uses the process reaper; a live session supplies
		its own record so failed history retirement survives tab replacement and
		then transfers through the existing MainWindow chain.
		"""
		self._graphics_retirement_reaper = reaper

	#============================================
	def _retire_discarded_redo_graphics(self) -> None:
		"""Terminally retire detached graphics in the redo branch Qt will prune."""
		if self._undo_stack.index() >= self._undo_stack.count():
			return
		commands = [
			self._undo_stack.command(index)
			for index in range(self._undo_stack.index(), self._undo_stack.count())
		]
		self._retire_detached_history_graphics(commands)

	#============================================
	def _retire_all_history_graphics(self) -> None:
		"""Terminally retire every detached item no longer needed after clear."""
		commands = [
			self._undo_stack.command(index)
			for index in range(self._undo_stack.count())
		]
		self._retire_detached_history_graphics(commands)

	#============================================
	def _retire_detached_history_graphics(
			self, commands: list[PySide6.QtGui.QUndoCommand],
			) -> None:
		"""Retire only command trees already detached from the live scene.

		Applied commands may retain graphics still owned by the live scene.  Clear
		must leave those projections to their scene owner, while an undone redo
		branch has detached roots that lose their only future owner when Qt drops
		the commands.  Snapshot every candidate before the terminal coordinator
		changes any native parent relationship.
		"""
		reaper = self._effective_terminal_graphics_reaper()
		items = []
		seen_commands = set()
		seen_items = set()

		#============================================
		def visit(command: PySide6.QtGui.QUndoCommand) -> None:
			"""Collect graphics from one command and its macro children."""
			if id(command) in seen_commands:
				return
			seen_commands.add(id(command))
			graphics_items = getattr(command, "graphics_items", None)
			if callable(graphics_items):
				for item in graphics_items():
					if id(item) in seen_items:
						continue
					seen_items.add(id(item))
					# A failed terminal transition already has one durable owner.
					# This history scan must not inspect its scene or begin another
					# deletion attempt before the reaper's controlled resolution pass.
					if self._terminal_reaper_owns_graphics_root(item, reaper):
						continue
					from ferrum_qt.canvas.graphics_retirement import native_scene_for_item
					# This is the stable pre-retirement ownership check.  No
					# item is touched again after the coordinator begins deletion.
					if native_scene_for_item(item) is None:
						items.append(item)
			for index in range(command.childCount()):
				visit(command.child(index))

		for command in commands:
			visit(command)
		if not items:
			return
		from ferrum_qt.canvas.graphics_retirement import GraphicsRetirementCoordinator
		coordinator = GraphicsRetirementCoordinator()
		coordinator.retire_detached_projection_items(
			items, reaper,
		)

	#============================================
	def _effective_terminal_graphics_reaper(
			self,
		) -> object:
		"""Return the sole long-lived owner for failed terminal graphics.

		Live sessions install their own reaper so tab teardown can transfer every
		unresolved record to MainWindow.  A standalone Document has no session
		owner, so it deliberately uses the process reaper for both failed-root
		transfer and every later history ownership check.
		"""
		if self._graphics_retirement_reaper is not None:
			return self._graphics_retirement_reaper
		from ferrum_qt.canvas.graphics_retirement import (
			detached_graphics_retirement_reaper,
		)
		return detached_graphics_retirement_reaper

	#============================================
	def _terminal_reaper_owns_graphics_root(
			self,
			item: PySide6.QtWidgets.QGraphicsItem,
			reaper: object = None,
			) -> bool:
		"""Return whether an earlier terminal transition exclusively owns ``item``.

		The reaper uses Python identity only.  This check runs before history asks
		the wrapper for scene ownership, so a failed terminal deletion cannot be
		rediscovered by a later undo-stack clear.
		"""
		effective_reaper = reaper or self._effective_terminal_graphics_reaper()
		return (
			effective_reaper.owns_detached_root(item)
			or effective_reaper.owns_scene_projection_root(item)
		)
	@property
	def persistent_generation(self) -> int:
		"""Return this projection's monotonic persistent-mutation generation."""
		return self._persistent_generation

	#============================================
	def mark_clean(self) -> None:
		"""Mark the current undo-stack position as the saved document state."""
		self._direct_dirty = False
		self._undo_stack.setClean()
		self._sync_dirty_state()

	#============================================
	def mark_dirty(self) -> None:
		"""Mark a direct, non-command mutation as unsaved."""
		self._advance_persistent_generation()
		self._direct_dirty = True
		self._sync_dirty_state()
	#============================================
	def _on_undo_clean_changed(self, _clean: bool) -> None:
		"""Emit document modification changes from the undo stack clean state."""
		self._sync_dirty_state()

	#============================================
	def _on_undo_index_changed(self, _index: int) -> None:
		"""Record every command-stack position transition as persistent state."""
		self._advance_persistent_generation()

	#============================================
	def _advance_persistent_generation(self) -> None:
		"""Advance and publish Qt-local persistent mutation provenance."""
		self._persistent_generation += 1
		self.persistent_mutated.emit(self._persistent_generation)

	#============================================
	def _sync_dirty_state(self) -> None:
		"""Emit a transition when combined direct/undo dirty state changes."""
		dirty = self.dirty
		if dirty != self._dirty:
			self._dirty = dirty
			self.modified_changed.emit(dirty)
