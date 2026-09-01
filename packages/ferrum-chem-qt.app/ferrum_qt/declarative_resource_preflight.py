"""Acyclic, failure-atomic preflight for window declarative resources."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.actions.command_icons
import ferrum_qt.declarative_resources
import ferrum_qt.ferrum.authoring_ribbon_layout


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class WindowDeclarativeResources:
	"""Fully resolved resources safe to hand to visible window clients."""

	ribbon: ferrum_qt.ferrum.authoring_ribbon_layout.RibbonLayout
	command_icons: ferrum_qt.actions.command_icons.CommandIconCatalog


#============================================
def preflight_window_resources(registry: object) -> WindowDeclarativeResources:
	"""Resolve menu, ribbon, and command icons before visible surface mutation."""
	ferrum_qt.declarative_resources.preflight_menu_declarations(registry)
	ribbon = ferrum_qt.ferrum.authoring_ribbon_layout.load_ribbon_layout(registry)
	command_icons = ferrum_qt.actions.command_icons.build_command_icon_catalog(
		registry, ribbon.action_ids(),
	)
	return WindowDeclarativeResources(ribbon, command_icons)
