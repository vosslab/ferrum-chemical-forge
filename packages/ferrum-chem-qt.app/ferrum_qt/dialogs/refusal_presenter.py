"""Plain-language presentation for actions Ferrum cannot complete."""

# Standard Library
import dataclasses
import enum


#============================================
class RefusalTaskContext(enum.Enum):
	"""The author task that needs a clear recovery explanation."""

	OPEN_DOCUMENT = "open_document"
	SAVE_DOCUMENT = "save_document"
	CLOSE_DOCUMENT = "close_document"
	USE_TOOL = "use_tool"
	EDIT_DOCUMENT = "edit_document"


#============================================
class RefusalOutcome(enum.Enum):
	"""Facts that must remain true when an action is refused."""

	INVALID_DOCUMENT = "invalid_document"
	UNSUPPORTED_DOCUMENT = "unsupported_document"
	SOURCE_NOT_ALLOWED = "source_not_allowed"
	DOCUMENT_DISPLAY_FAILED = "document_display_failed"
	UNSUPPORTED_SAVE_EXTENSION = "unsupported_save_extension"
	SAVE_NOT_STARTED = "save_not_started"
	SAVE_POSSIBLY_COMPLETED = "save_possibly_completed"
	SAVE_DISPLAY_FAILED = "save_display_failed"
	BUSY_CLOSE = "busy_close"
	STALE_TOOL = "stale_tool"
	UNRENDERABLE_MOLECULE = "unrenderable_molecule"
	UNAVAILABLE_OPERATION = "unavailable_operation"
	NO_UNDO = "no_undo"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RefusalRequest:
	"""Typed facts required to explain one refused author task."""

	context: RefusalTaskContext
	outcome: RefusalOutcome
	document_name: str | None = None
	technical_details: str | None = None
	primary_message: str | None = None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class RefusalPresentation:
	"""User language and separate optional diagnostic detail for a refusal."""

	title: str
	what_happened: str
	why: str
	what_next: str
	technical_details: str | None

	#============================================
	def ordinary_text(self) -> str:
		"""Return only the author-facing explanation for a status surface or dialog."""
		text = "What happened: " + self.what_happened
		text += "\n\nWhy: " + self.why
		text += "\n\nWhat to do now: " + self.what_next
		return text


#============================================
def present_refusal(request: RefusalRequest) -> RefusalPresentation:
	"""Render one truthful refusal without exposing implementation vocabulary."""
	_validate_request(request)
	name = _document_name(request.document_name)
	if request.outcome is RefusalOutcome.INVALID_DOCUMENT:
		return _invalid_document(request, name)
	if request.outcome is RefusalOutcome.UNSUPPORTED_DOCUMENT:
		return _unsupported_document(request, name)
	if request.outcome is RefusalOutcome.SOURCE_NOT_ALLOWED:
		return _source_not_allowed(request, name)
	if request.outcome is RefusalOutcome.DOCUMENT_DISPLAY_FAILED:
		return _document_display_failed(request, name)
	if request.outcome is RefusalOutcome.UNSUPPORTED_SAVE_EXTENSION:
		return _unsupported_save_extension(request, name)
	if request.outcome is RefusalOutcome.SAVE_NOT_STARTED:
		return _save_not_started(request, name)
	if request.outcome is RefusalOutcome.SAVE_POSSIBLY_COMPLETED:
		return _save_possibly_completed(request, name)
	if request.outcome is RefusalOutcome.SAVE_DISPLAY_FAILED:
		return _save_display_failed(request, name)
	if request.outcome is RefusalOutcome.BUSY_CLOSE:
		return _busy_close(request)
	if request.outcome is RefusalOutcome.STALE_TOOL:
		return _stale_tool(request)
	if request.outcome is RefusalOutcome.UNRENDERABLE_MOLECULE:
		return _unrenderable_molecule(request)
	if request.outcome is RefusalOutcome.UNAVAILABLE_OPERATION:
		return _unavailable_operation(request)
	if request.outcome is RefusalOutcome.NO_UNDO:
		return _no_undo(request)
	raise ValueError("unknown refusal outcome")


#============================================
def _validate_request(request: RefusalRequest) -> None:
	"""Reject an outcome presented for a different author task."""
	if request.primary_message is not None:
		if type(request.primary_message) is not str:
			raise TypeError("primary message must be a string or None")
		if not request.primary_message:
			raise ValueError("primary message must be nonempty")
		if request.outcome is not RefusalOutcome.UNAVAILABLE_OPERATION:
			raise ValueError("primary message needs an unavailable-operation refusal")
	if request.outcome in (
			RefusalOutcome.INVALID_DOCUMENT, RefusalOutcome.UNSUPPORTED_DOCUMENT,
			RefusalOutcome.SOURCE_NOT_ALLOWED,
			RefusalOutcome.DOCUMENT_DISPLAY_FAILED,
		) and request.context is not RefusalTaskContext.OPEN_DOCUMENT:
		raise ValueError("document-open refusal needs an open-document context")
	if request.outcome in (
			RefusalOutcome.UNSUPPORTED_SAVE_EXTENSION,
			RefusalOutcome.SAVE_NOT_STARTED,
			RefusalOutcome.SAVE_POSSIBLY_COMPLETED,
			RefusalOutcome.SAVE_DISPLAY_FAILED,
		) and request.context is not RefusalTaskContext.SAVE_DOCUMENT:
		raise ValueError("save refusal needs a save-document context")
	if request.outcome is RefusalOutcome.BUSY_CLOSE:
		if request.context is not RefusalTaskContext.CLOSE_DOCUMENT:
			raise ValueError("busy-close refusal needs a close-document context")
	if request.outcome is RefusalOutcome.STALE_TOOL:
		if request.context is not RefusalTaskContext.USE_TOOL:
			raise ValueError("stale-tool refusal needs a use-tool context")
	if request.outcome in (
			RefusalOutcome.UNRENDERABLE_MOLECULE, RefusalOutcome.UNAVAILABLE_OPERATION,
			RefusalOutcome.NO_UNDO,
		) and request.context is not RefusalTaskContext.EDIT_DOCUMENT:
		raise ValueError("editing refusal needs an edit-document context")


#============================================
def _document_name(document_name: str | None) -> str:
	"""Use a neutral subject when an action has no named document."""
	if document_name is None:
		name = "this document"
	else:
		name = document_name
	return name


#============================================
def _invalid_document(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""Explain that a document could not be read as a drawing."""
	return RefusalPresentation(
		"Could Not Open Document",
		f"Ferrum could not open {name}.",
		"The file does not contain a usable drawing.",
		"Choose another file or correct this file, then try again.",
		request.technical_details,
	)


#============================================
def _unsupported_document(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""Explain that the selected document kind is unsupported."""
	return RefusalPresentation(
		"Document Format Not Supported",
		f"Ferrum cannot open {name}.",
		"This kind of document is not supported here.",
		"Choose a supported document, then try again.",
		request.technical_details,
	)


#============================================
def _source_not_allowed(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""Explain a local-file safety refusal without implementation vocabulary."""
	return RefusalPresentation(
		"Cannot Open This File",
		f"Ferrum could not open {name}.",
		"Ferrum can open only a regular local file at this location.",
		"Choose the original file rather than a symbolic link or special file.",
		request.technical_details,
	)


#============================================
def _document_display_failed(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""Explain a failure after a document passed file checks."""
	return RefusalPresentation(
		"Could Not Display Document",
		f"Ferrum could not add {name} to the window.",
		"The file passed its checks, but the drawing could not be displayed.",
		"Your current tab is unchanged. Keep it open and try the file again.",
		request.technical_details,
	)


#============================================
def _unsupported_save_extension(
		request: RefusalRequest, name: str,
		) -> RefusalPresentation:
	"""Explain the only document format this save route accepts."""
	return RefusalPresentation(
		"Cannot Save in This Format",
		f"Ferrum cannot save {name} with that file extension.",
		"This drawing can be saved only as a .cdml file.",
		"Use a name ending in .cdml, then save again.",
		request.technical_details,
	)


#============================================
def _save_not_started(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""State the definite fact that writing did not begin."""
	return RefusalPresentation(
		"File Was Not Saved",
		f"Ferrum did not start saving {name}.",
		"The selected destination could not accept the file.",
		"Keep this tab open, choose another destination, then save again.",
		request.technical_details,
	)


#============================================
def _save_possibly_completed(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""Preserve the distinction between an unknown and failed save result."""
	return RefusalPresentation(
		"Could Not Confirm Save",
		f"Ferrum could not confirm that {name} was saved.",
		"The save may have completed, but its final result is unknown.",
		"Keep this tab open, check the destination, then save again if needed.",
		request.technical_details,
	)


#============================================
def _save_display_failed(request: RefusalRequest, name: str) -> RefusalPresentation:
	"""State that publication succeeded even though the visible tab did not update."""
	return RefusalPresentation(
		"Drawing Saved; Display Needs Attention",
		f"Ferrum saved {name}, but could not update this tab's display.",
		"The saved drawing and the current on-screen view are temporarily out of sync.",
		"Keep the tab open, then reopen the saved drawing to confirm its contents.",
		request.technical_details,
	)


#============================================
def _busy_close(request: RefusalRequest) -> RefusalPresentation:
	"""Prevent a close that would interrupt active work."""
	return RefusalPresentation(
		"Cannot Close Yet",
		"Ferrum is still working on this document.",
		"Closing now could interrupt the current task.",
		"Wait for it to finish, or cancel that task before closing.",
		request.technical_details,
	)


#============================================
def _stale_tool(request: RefusalRequest) -> RefusalPresentation:
	"""Explain that a tool intent no longer matches the drawing."""
	return RefusalPresentation(
		"Tool Needs to Be Chosen Again",
		"This tool was prepared for an earlier version of the drawing and was not used.",
		"The drawing changed before the tool could finish.",
		"Choose the tool again, then repeat the step.",
		request.technical_details,
	)


#============================================
def _unrenderable_molecule(request: RefusalRequest) -> RefusalPresentation:
	"""Explain why a structural edit cannot produce visible canvas feedback."""
	return RefusalPresentation(
		"Cannot Add an Atom Here",
		"Ferrum did not change the drawing.",
		"The selected molecule cannot currently be drawn on the Ferrum canvas.",
		"Choose another visible molecule. If this is the only molecule, use a "
		"Ferrum-supported element, hydrogen, and charge label or a supported style, "
		"then reopen the drawing.",
		request.technical_details,
	)


#============================================
def _unavailable_operation(request: RefusalRequest) -> RefusalPresentation:
	"""Explain why an action cannot be applied in the current state."""
	return RefusalPresentation(
		"Action Not Available",
		request.primary_message or "This action is not available for the current drawing.",
		"The needed selection or document state is not available.",
		"Select the required item or change the drawing, then try again.",
		request.technical_details,
	)


#============================================
def _no_undo(request: RefusalRequest) -> RefusalPresentation:
	"""Explain that the current drawing has no reversible earlier change."""
	return RefusalPresentation(
		"Nothing to Undo",
		"There is no earlier change to undo.",
		"The current drawing has no undoable change.",
		"Continue editing, or use Redo if you recently undid a change.",
		request.technical_details,
	)
