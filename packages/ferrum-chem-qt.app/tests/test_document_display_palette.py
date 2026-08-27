"""Test the YAML-owned semantic document-display palette contract."""

# Standard Library
import dataclasses

# PIP3 modules
import pytest
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.ferrum.graphics_view
import ferrum_qt.themes.document_display_palette
import ferrum_qt.themes.theme_loader


@dataclasses.dataclass(frozen=True)
class _Paint:
	"""Represent one frozen RenderPaintV3 fact at the Qt adapter boundary."""

	kind: str
	export_rgb: str
	role: str | None
	element: str | None


@dataclasses.dataclass
class _Refreshable(
		ferrum_qt.ferrum.document_display_refresh.DocumentDisplayRefreshableV1,
		):
	"""Record one palette delivery through the public refresh contract."""

	palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1 | None = None

	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Retain the exact delivered palette for this narrow lifecycle test."""
		self.palette = palette


#============================================
class _StructuralRefreshableLookAlike:
	"""Provide the method shape without membership in the nominal contract."""

	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Accept a palette only to model the rejected structural shape."""


#============================================
def _valid_document_display(theme_name: str) -> dict[str, object]:
	"""Return one complete inline palette map derived from a shipped theme."""
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette(theme_name)
	return {
		role.value: palette.color(role).name()
		for role in ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1
	} | {"elements": {}}


#============================================
@pytest.mark.parametrize("theme_name", ("light", "dark"))
def test_document_display_yaml_contract_meets_role_specific_contrast(
		theme_name: str,
		) -> None:
	"""Each complete YAML theme meets the approved display-role contrast floor."""
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette(theme_name)
	page_fill = palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PAGE_FILL
	)
	assert palette.element_symbols == ()
	for role in ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1:
		if role in (
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.
			CANVAS_SURROUND,
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.PAGE_FILL,
		):
			continue
		ratio = ferrum_qt.themes.document_display_palette.color_contrast_ratio(
			palette.color(role), page_fill,
		)
		assert ratio >= (
			ferrum_qt.themes.document_display_palette.
			document_display_minimum_contrast(role)
		), role.value


#============================================
@pytest.mark.parametrize(("removed_token", "replacement"), (
	("page_fill", None),
	(None, {"unexpected": "#ffffff"}),
	(None, {"atom_number": "invalid"}),
	(None, {"atom_number": "#11223380"}),
	(None, {"elements": {"C": "#ffffff"}}),
))
def test_document_display_palette_refuses_malformed_yaml_contract(
		removed_token: str | None,
		replacement: dict[str, object] | None,
		) -> None:
	"""Malformed YAML display maps refuse before a palette can be created."""
	document_display = _valid_document_display("light")
	if removed_token is not None:
		document_display.pop(removed_token)
	if replacement is not None:
		document_display.update(replacement)
	with pytest.raises(
		ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError
	):
		ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1.from_yaml(
			document_display
		)


#============================================
def test_document_display_palette_preserves_authored_rgb_exactly() -> None:
	"""An authored Rust RGB value bypasses theme adaptation unchanged."""
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	paint = _Paint("authored_rgb24", "1234ab", None, None)
	assert palette.resolve_render_paint(paint).name() == "#1234ab"


#============================================
def test_document_display_palette_resolves_closed_theme_roles() -> None:
	"""Rust default roles resolve only through their matching YAML display token."""
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	foreground = _Paint("theme_role", "000000", "document_foreground", None)
	atom_number = _Paint("theme_role", "0000c8", "atom_number", None)
	assert palette.resolve_render_paint(foreground).name() == palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.
		DOCUMENT_FOREGROUND
	).name()
	assert palette.resolve_render_paint(atom_number).name() == palette.color(
		ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.ATOM_NUMBER
	).name()


#============================================
@pytest.mark.parametrize("paint", (
	_Paint("unknown", "000000", None, None),
	_Paint("theme_role", "000000", None, None),
	_Paint("theme_role", "000000", "selection_outline", None),
	_Paint("theme_role", "000000", "unrecognized", None),
	_Paint("element_role", "000000", None, "O"),
	_Paint("authored_rgb24", "nothex", None, None),
))
def test_document_display_palette_refuses_malformed_or_unmapped_paint(
		paint: _Paint,
		) -> None:
	"""Malformed V3 tags and reserved unmapped element roles have typed refusal."""
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	with pytest.raises(
		ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteError
	):
		palette.resolve_render_paint(paint)


#============================================
def test_graphics_view_requires_and_uses_one_explicit_document_palette(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Canvas surround and cursor material come only from the selected palette."""
	with pytest.raises(TypeError):
		ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView()
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	view = ferrum_qt.ferrum.graphics_view.FerrumNativeGraphicsView(palette)
	try:
		assert view.backgroundBrush().color().name() == palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.
			CANVAS_SURROUND,
		).name()
		scene = PySide6.QtWidgets.QGraphicsScene()
		view.setScene(scene)
		view.set_keyboard_cursor_scene(PySide6.QtCore.QPointF(10.0, 20.0))
		assert view._keyboard_cursor_item is not None
		assert view._keyboard_cursor_item.pen().color().name() == palette.color(
			ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1.
			KEYBOARD_CURSOR,
		).name()
	finally:
		view.setScene(None)
		view.deleteLater()
		qapp.processEvents()


#============================================
def test_document_display_delegating_refreshable_forwards_shipped_palette() -> None:
	"""The delegating retained root forwards an exact palette through the registry."""
	class _RendererItem:
		"""Record the public renderer material refresh call."""

		def __init__(self) -> None:
			"""Start before the registry has delivered a palette."""
			self.palette: object | None = None

		def refresh_display_palette(self, palette: object) -> None:
			"""Retain the received palette for this isolated adapter proof."""
			self.palette = palette

	item = _RendererItem()
	refreshable = (
		ferrum_qt.ferrum.document_display_refresh.
		DocumentDisplayDelegatingRefreshableV1(item)
	)
	registry = (
		ferrum_qt.ferrum.document_display_refresh.
		DocumentDisplayPaletteRefreshRegistryV1()
	)
	palette = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	registry.register(refreshable)
	registry.refresh(palette)
	assert item.palette is palette


#============================================
def test_document_display_refresh_registry_releases_detached_objects() -> None:
	"""A tab-scoped registry refreshes attached objects and releases detached ones."""
	registry = (
		ferrum_qt.ferrum.document_display_refresh.
		DocumentDisplayPaletteRefreshRegistryV1()
	)
	refreshable = _Refreshable()
	light = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	dark = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	registry.register(refreshable)
	registry.refresh(light)
	registry.unregister(refreshable)
	registry.refresh(dark)
	assert refreshable.palette is light


#============================================
def test_document_display_refresh_registry_refuses_structural_lookalike() -> None:
	"""The registry requires nominal membership before accepting a refresh method."""
	registry = (
		ferrum_qt.ferrum.document_display_refresh.
		DocumentDisplayPaletteRefreshRegistryV1()
	)
	with pytest.raises(TypeError):
		registry.register(_StructuralRefreshableLookAlike())
