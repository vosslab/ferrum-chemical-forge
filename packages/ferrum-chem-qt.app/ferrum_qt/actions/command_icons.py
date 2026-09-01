"""Canonical icon presentation for commands exposed by the Ferrum ribbon."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.declarative_resources
import ferrum_qt.widgets.icon_loader


_RESOURCE_ICONS = {
	"chemistry.compact_group.attach": "adder",
	"chemistry.compact_group.materialize": "repair",
	"chemistry.compact_group.place": "template",
	"chemistry.reaction.create": "adder",
	"chemistry.reaction.inspect": "interpret",
	"chemistry.template.catalog": "template",
	"draw.arrange.move_atom": "edit",
	"draw.arrange.move_complete_roots": "edit",
	"draw.arrange.rotate_selected_atoms": "rotate",
	"draw.arrow": "arrow",
	"draw.arrow.curved_electron": "electron",
	"draw.arrow.curved_equilibrium": "equilibrium2",
	"draw.arrow.curved_reaction": "arrow",
	"draw.arrow.curved_retro": "retro",
	"draw.arrow.equilibrium": "equilibrium2",
	"draw.atom_at_point": "atom",
	"draw.bond": "single",
	"draw.bond.connect_selected": "single",
	"draw.bond.hashed_wedge": "hashed",
	"draw.bond.solid_wedge": "wedge",
	"draw.bracket.rectangular": "rectangularbracket",
	"draw.bracket.round": "roundbracket",
	"draw.next_drawing": "draw",
	"draw.path.polygon": "polygon",
	"draw.path.polyline": "polyline",
	"draw.plus": "plus",
	"draw.ring.cyclohexane.attach": "cyclohexane",
	"draw.ring.cyclohexane.insert": "cyclohexane",
	"draw.ring.haworth.insert": "chair",
	"draw.ring.regular.c6": "benzene",
	"draw.selection.structure": "edit",
	"draw.text": "text",
	"draw.transform.roots.scale": "2d",
	"draw.vector.circle": "circle",
	"draw.vector.line": "vector",
	"draw.vector.oval": "oval",
	"draw.vector.rectangle": "rectangle",
	"draw.vector.square": "square",
	"draw.wavy": "wavyline",
	"edit.bond.change_order": "double",
	"edit.bond.reverse_wedge": "invertthrough",
	"edit.redo": "redo",
	"edit.undo": "undo",
	"view.command_palette": "interpret",
	"view.grid.snap": "fixed",
	"view.grid.visible": "benzene",
}

_STANDARD_ICONS = {
	"file.new": PySide6.QtWidgets.QStyle.StandardPixmap.SP_FileIcon,
	"file.open": PySide6.QtWidgets.QStyle.StandardPixmap.SP_DialogOpenButton,
	"file.save": PySide6.QtWidgets.QStyle.StandardPixmap.SP_DialogSaveButton,
	"view.zoom_100": PySide6.QtWidgets.QStyle.StandardPixmap.SP_BrowserReload,
	"view.zoom_in": PySide6.QtWidgets.QStyle.StandardPixmap.SP_ArrowUp,
	"view.zoom_out": PySide6.QtWidgets.QStyle.StandardPixmap.SP_ArrowDown,
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class CommandIconBinding:
	"""One resolved QAction and its package or platform icon source."""

	action_id: str
	action: PySide6.QtGui.QAction
	resource_name: str | None
	standard_pixmap: PySide6.QtWidgets.QStyle.StandardPixmap | None


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class CommandIconCatalog:
	"""Closed icon catalog for every command placed on the ribbon."""

	bindings: tuple[CommandIconBinding, ...]

	#============================================
	def apply(self, style: PySide6.QtWidgets.QStyle, theme_name: str) -> None:
		"""Resolve every theme-aware icon before updating shared QActions."""
		ferrum_qt.widgets.icon_loader.set_theme(theme_name)
		ferrum_qt.widgets.icon_loader.reload_icons()
		resolved: list[tuple[PySide6.QtGui.QAction, PySide6.QtGui.QIcon]] = []
		for binding in self.bindings:
			if binding.resource_name is not None:
				icon = ferrum_qt.widgets.icon_loader.get_icon(binding.resource_name)
			else:
				if binding.standard_pixmap is None:
					raise ferrum_qt.declarative_resources.DeclarativeResourceError(
						f"Ribbon command '{binding.action_id}' has no icon source.",
					)
				icon = style.standardIcon(binding.standard_pixmap)
			if icon.isNull():
				raise ferrum_qt.declarative_resources.DeclarativeResourceError(
					f"Ribbon command '{binding.action_id}' has no loadable icon.",
				)
			resolved.append((binding.action, icon))
		for action, icon in resolved:
			action.setIcon(icon)


#============================================
def build_command_icon_catalog(
		registry: object, required_action_ids: frozenset[str],
		) -> CommandIconCatalog:
	"""Resolve a complete, exact icon binding for the current ribbon contract."""
	catalog_action_ids = frozenset(_RESOURCE_ICONS) | frozenset(_STANDARD_ICONS)
	if catalog_action_ids != required_action_ids:
		missing = sorted(required_action_ids - catalog_action_ids)
		extra = sorted(catalog_action_ids - required_action_ids)
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			f"Ribbon command icon catalog mismatch; missing={missing}, extra={extra}.",
		)
	get_qt_action = getattr(registry, "get_qt_action", None)
	if not callable(get_qt_action):
		raise ferrum_qt.declarative_resources.DeclarativeResourceError(
			"Command icon catalog needs an action registry with get_qt_action().",
		)
	ferrum_qt.widgets.icon_loader.validate_icon_paths()
	bindings: list[CommandIconBinding] = []
	for action_id in sorted(required_action_ids):
		action = get_qt_action(action_id)
		if not isinstance(action, PySide6.QtGui.QAction):
			raise ferrum_qt.declarative_resources.DeclarativeResourceError(
				f"Command icon catalog references unbound QAction '{action_id}'.",
			)
		resource_name = _RESOURCE_ICONS.get(action_id)
		standard_pixmap = _STANDARD_ICONS.get(action_id)
		bindings.append(CommandIconBinding(
			action_id, action, resource_name, standard_pixmap,
		))
	return CommandIconCatalog(tuple(bindings))
