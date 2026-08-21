"""Shared construction helpers for Rust-owned Ferrum document tabs."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.ferrum.graphics_view


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
	import ferrum_qt.ferrum.engine as engine
	_validate_session_and_title(session, title)
	if type(observation) is not engine.RenderObservationV1:
		raise TypeError(
			"local CDML Open requires exact Ferrum session and observation values",
		)
	_validate_admitted_provenance(session, observation, error_type)
	tab = _construct_tab(tab_class, session, title)
	try:
		live_observation = tab._publish_live_render_plan_v1(
			observation.document.snapshot.revision,
		)
		if not tab._install_observation(live_observation):
			raise error_type("Ferrum tab could not install its admitted render observation")
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
	import ferrum_qt.ferrum.engine as engine
	if type(session) is not engine.DocumentSession or type(title) is not str:
		raise TypeError("Ferrum document tab requires a Rust session and title string")


#============================================
def _construct_tab(tab_class: type, session: object, title: str) -> object:
	"""Install the shared Qt ownership graph without deriving document facts."""
	import ferrum_qt.ferrum.engine as engine
	tab = tab_class.__new__(tab_class)
	PySide6.QtWidgets.QWidget.__init__(tab)
	try:
		view = ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView(tab)
		resource = engine.verified_telex_regular()
		controller = (
			ferrum_qt.canvas.ferrum_render_projection.
			FerrumRenderProjectionController(view, resource)
		)
		tab._initialize(title, session, view, controller)
	except Exception:
		tab._retire_partial_resources()
		raise
	return tab
