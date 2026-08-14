"""Shared construction helpers for Rust-owned native document tabs."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.native.ferrum_native_graphics_view


#============================================
def create_document_tab_from_session(
		tab_class: type,
		session: object,
		title: str,
		) -> object:
	"""Construct one tab and derive its initial observation on this thread."""
	_validate_session_and_title(session, title)
	tab = _construct_tab(tab_class, session, title)
	try:
		tab._refresh_from_current_revision()
	except Exception:
		tab._retire_partial_resources()
		raise
	return tab


#============================================
def create_admitted_local_document_tab(
		tab_class: type,
		session: object,
		title: str,
		observation: object,
		error_type: type[Exception],
		) -> object:
	"""Construct one tab without repeating worker-owned Rust observation."""
	import ferrum_chem
	_validate_session_and_title(session, title)
	if type(observation) is not ferrum_chem.RenderObservationV1:
		raise TypeError(
			"local CDML Open requires exact Ferrum session and observation values",
		)
	_validate_admitted_provenance(session, observation, error_type)
	tab = _construct_tab(tab_class, session, title)
	try:
		if not tab._install_observation(observation):
			raise error_type("native tab could not install its admitted render observation")
	except Exception:
		tab._retire_partial_resources()
		raise
	return tab


#============================================
def _validate_admitted_provenance(
		session: object,
		observation: object,
		error_type: type[Exception],
		) -> None:
	"""Authenticate the worker-prepared observation against its exact session."""
	session_snapshot = session.snapshot()
	observation_snapshot = observation.document.snapshot
	if (
		observation_snapshot.revision != session_snapshot.revision
		or observation_snapshot.digest != session_snapshot.digest
		):
		raise error_type(
			"local CDML Open observation does not match its admitted session",
		)


#============================================
def _validate_session_and_title(session: object, title: str) -> None:
	"""Require the exact public Rust session and a presentation title."""
	import ferrum_chem
	if type(session) is not ferrum_chem.DocumentSession or type(title) is not str:
		raise TypeError("native document tab requires a Rust session and title string")


#============================================
def _construct_tab(tab_class: type, session: object, title: str) -> object:
	"""Install the shared Qt ownership graph without deriving document facts."""
	import ferrum_chem
	tab = tab_class.__new__(tab_class)
	PySide6.QtWidgets.QWidget.__init__(tab)
	try:
		view = ferrum_qt.native.ferrum_native_graphics_view.FerrumNativeGraphicsView(tab)
		resource = ferrum_chem.verified_telex_regular()
		controller = (
			ferrum_qt.canvas.ferrum_render_projection.
			FerrumRenderProjectionController(view, resource)
		)
		tab._initialize(title, session, view, controller)
	except Exception:
		tab._retire_partial_resources()
		raise
	return tab
