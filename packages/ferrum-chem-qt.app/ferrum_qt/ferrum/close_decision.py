"""Typed decisions and outcomes for closing one Ferrum document tab."""

# Standard Library
import enum


#============================================
class CloseDecision(enum.Enum):
	"""State the caller's explicit disposition for unsaved document changes."""

	KEEP_OPEN = "keep_open"
	SAVE = "save"
	DISCARD = "discard"


#============================================
class CloseResult(enum.Enum):
	"""Report the one lifecycle outcome of an attempted Ferrum tab close."""

	CLOSED = "closed"
	NO_TAB = "no_tab"
	LOCAL_DOCUMENT_OPEN_CANCELLATION_REQUESTED = "local_document_open_cancellation_requested"
	MOLECULE_IMPORT_BLOCKED = "molecule_import_blocked"
	MOLECULE_EXPORT_BLOCKED = "molecule_export_blocked"
	SNAPSHOT_EXPORT_BLOCKED = "snapshot_export_blocked"
	MOLECULE_INSPECTION_BLOCKED = "molecule_inspection_blocked"
	MOLECULE_DIAGNOSTICS_BLOCKED = "molecule_diagnostics_blocked"
	ATOM_OXIDATION_BLOCKED = "atom_oxidation_blocked"
	CLIPBOARD_OPERATION_BLOCKED = "clipboard_operation_blocked"
	COORDINATE_GENERATION_BLOCKED = "coordinate_generation_blocked"
	OPERATION_CANCELLATION_FAILED = "operation_cancellation_failed"
	REFRESH_REQUIRED = "refresh_required"
	DIRTY_REQUIRES_DECISION = "dirty_requires_decision"
	SAVE_FAILED = "save_failed"
	LINE_GESTURE_CANCELLATION_FAILED = "line_gesture_cancellation_failed"
	DISPOSAL_FAILED = "disposal_failed"
