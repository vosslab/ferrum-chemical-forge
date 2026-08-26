"""Acyclic, failure-atomic preflight for window declarative resources."""

# local repo modules
import ferrum_qt.declarative_resources
import ferrum_qt.ferrum.authoring_ribbon_layout


#============================================
def preflight_window_resources(registry: object) -> None:
	"""Resolve menu and ribbon clients before either visible surface mutates."""
	ferrum_qt.declarative_resources.preflight_menu_declarations(registry)
	ferrum_qt.ferrum.authoring_ribbon_layout.load_ribbon_layout(registry)
