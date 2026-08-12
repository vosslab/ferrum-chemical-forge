"""Document model holding molecules and providing undo support."""

# Standard Library
import os

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.models.molecule_model
import ferrum_qt.models.document_object
import ferrum_qt.models.document_presentation
import ferrum_qt.models.document_selection
import ferrum_qt.models.document_identity
import ferrum_qt.models.document_undo

#============================================
DocumentUndoStack = ferrum_qt.models.document_undo.DocumentUndoStack


#============================================
class Document(
		ferrum_qt.models.document_identity.DocumentIdentity,
		ferrum_qt.models.document_undo.DocumentUndoState,
		ferrum_qt.models.document_selection.DocumentSelection,
		ferrum_qt.models.document_presentation.DocumentPresentation,
		PySide6.QtCore.QObject,
		):
	"""Top-level document that holds molecules, file state, and undo stack.

	The Document is the Qt projection's owner for the object stack, undo
	history, and local file state.  Persistent CDML authority remains in the
	backend session; this object records only the live frontend projection.
	Selection state lives in the QGraphicsScene but Document provides query
	helpers that give modes chemistry-aware access to the selection.

	Emits ``modified_changed`` whenever the dirty flag transitions so the
	window title can show an unsaved-changes indicator. Emits
	``selection_changed`` after selection queries detect a change.

	Args:
		parent: Optional parent QObject.
	"""

	# emitted when the dirty flag changes
	modified_changed = PySide6.QtCore.Signal(bool)
	# emitted when the selection changes (forwarded from scene)
	selection_changed = PySide6.QtCore.Signal()
	# emitted when top-level CDML content is inserted or removed
	object_added = PySide6.QtCore.Signal(object)
	object_removed = PySide6.QtCore.Signal(object)
	# emitted when a mark is inserted or removed
	mark_added = PySide6.QtCore.Signal(object)
	mark_removed = PySide6.QtCore.Signal(object)
	# emitted after paper or viewport state changes
	paper_changed = PySide6.QtCore.Signal(object)
	# emitted after every Qt-local persistent mutation boundary advances
	persistent_mutated = PySide6.QtCore.Signal(int)

	#============================================
	def __init__(self, parent: PySide6.QtCore.QObject | None = None) -> None:
		"""Initialize an empty document.

		Args:
			parent: Optional parent QObject.
		"""
		super().__init__(parent)
		self._molecules = []
		self._object_stack = []
		self._presentation_objects = []
		# This document is the explicit frontend owner of the wrappers in its
		# current disposable projection.  A QGraphicsScene owns native items, but
		# PySide does not promise it owns their Python wrappers.  Keep those
		# wrappers alive until the retirement coordinator has detached them.
		self._projection_item_refs = {}
		self._marks = []
		self._paper = ferrum_qt.models.document_object.PaperModel()
		self._cdml_envelope = ferrum_qt.models.document_object.CdmlEnvelope()
		self._unsupported_content = []
		self._file_path = None
		self._undo_stack = DocumentUndoStack(self)
		self._graphics_retirement_reaper = None
		self._persistent_generation = 0
		# Undoable edits use QUndoStack's clean point. Direct structural
		# mutations remain supported while older actions are migrated.
		self._direct_dirty = False
		self._dirty = False
		self._undo_stack.cleanChanged.connect(self._on_undo_clean_changed)
		self._undo_stack.indexChanged.connect(self._on_undo_index_changed)
		# scene reference for selection queries (set by MainWindow)
		self._scene = None

	# ------------------------------------------------------------------
	# Properties
	# ------------------------------------------------------------------

	#============================================
	@property
	def molecules(self) -> list:
		"""Return the list of MoleculeModel instances in this document.

		Returns:
			List of MoleculeModel objects.
		"""
		return list(self._molecules)

	#============================================
	@property
	def objects(self) -> list:
		"""Return the ordered top-level molecule and presentation stack."""
		return list(self._object_stack)

	#============================================
	@property
	def presentation_objects(self) -> list:
		"""Return non-molecule drawable CDML objects."""
		return list(self._presentation_objects)

	#============================================
	@property
	def marks(self) -> list:
		"""Return atom-attached CDML mark models."""
		return list(self._marks)

	#============================================
	@property
	def paper(self) -> ferrum_qt.models.document_object.PaperModel:
		"""Return preserved paper and viewport state."""
		return self._paper

	#============================================
	@property
	def cdml_envelope(self) -> ferrum_qt.models.document_object.CdmlEnvelope:
		"""Return preserved document-level CDML content."""
		return self._cdml_envelope

	#============================================
	@property
	def unsupported_content(self) -> list:
		"""Return warnings for persistent content not projected by the Qt UI."""
		return list(self._unsupported_content)

	#============================================
	@property
	def file_path(self) -> str | None:
		"""Absolute path to the saved file, or None if unsaved.

		Returns:
			str or None.
		"""
		return self._file_path

	#============================================
	@file_path.setter
	def file_path(self, value: str | None) -> None:
		self._file_path = value

	#============================================
	@property
	def dirty(self) -> bool:
		"""Whether the document has unsaved changes."""
		return self._direct_dirty or not self._undo_stack.isClean()

	#============================================
	@dirty.setter
	def dirty(self, value: bool) -> None:
		"""Set or clear the document's compatibility dirty state.

		New persistent edits should use undo commands. The true branch remains
		available for older direct-mutation paths so they cannot bypass close
		guards while those paths are migrated.
		"""
		if value:
			self.mark_dirty()
			return
		self.mark_clean()

	#============================================
	@property
	def undo_stack(self) -> PySide6.QtGui.QUndoStack:
		"""The QUndoStack for undo/redo operations.

		Returns:
			QUndoStack instance owned by this document.
		"""
		return self._undo_stack

	#============================================
	def set_graphics_retirement_reaper(
			self,
			reaper: "ferrum_qt.canvas.graphics_retirement.DetachedGraphicsRetirementReaper | None",
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

	# ------------------------------------------------------------------
	# File info
	# ------------------------------------------------------------------

	#============================================
	def title(self) -> str:
		"""Return a display title for the document.

		Uses the filename from ``file_path`` if available, otherwise
		returns 'Untitled'.

		Returns:
			Title string.
		"""
		if self._file_path:
			basename = os.path.basename(self._file_path)
			return basename
		return "Untitled"

	#============================================
	def __repr__(self) -> str:
		"""Return a developer-friendly string representation."""
		n_mols = len(self._molecules)
		title = self.title()
		return f"Document('{title}', {n_mols} molecules)"
