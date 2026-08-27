"""Own typed refresh lifetimes for retained document-display materials."""

# Standard Library
import typing

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui

# local repo modules
import ferrum_qt.themes.document_display_palette


@typing.runtime_checkable
class DocumentDisplayRefreshableV1(typing.Protocol):
	"""Replace only retained Qt material from one immutable display palette."""

	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Refresh retained display material without deriving document state."""


#============================================
class DocumentDisplayPaletteRefreshRegistryV1:
	"""Retain live document-display refreshables for one document tab."""

	def __init__(self) -> None:
		"""Start with no attached transient display objects."""
		self._refreshables: list[DocumentDisplayRefreshableV1] = []

	#============================================
	def register(self, refreshable: DocumentDisplayRefreshableV1) -> None:
		"""Retain one attached refreshable by identity until it is released."""
		self._require_refreshable(refreshable)
		if not any(known is refreshable for known in self._refreshables):
			self._refreshables.append(refreshable)

	#============================================
	def unregister(self, refreshable: DocumentDisplayRefreshableV1) -> None:
		"""Release one detached refreshable without affecting other objects."""
		self._require_refreshable(refreshable)
		self._refreshables = [
			known for known in self._refreshables if known is not refreshable
		]

	#============================================
	def refresh(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Refresh the currently attached objects from one validated palette."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum document display refresh requires an exact palette")
		for refreshable in tuple(self._refreshables):
			refreshable.refresh_document_display_palette(palette)

	#============================================
	def clear(self) -> None:
		"""Release retained object references when their owning tab is disposed."""
		self._refreshables.clear()

	#============================================
	@staticmethod
	def _require_refreshable(refreshable: DocumentDisplayRefreshableV1) -> None:
		"""Require the explicit refresh contract instead of arbitrary callbacks."""
		if not isinstance(refreshable, DocumentDisplayRefreshableV1):
			raise TypeError("Ferrum document display refresh requires a refreshable")


#============================================
class DocumentDisplayRoleMaterialRefreshableV1:
	"""Refresh retained Qt item material from named document-display roles."""

	def __init__(self, items: tuple[object, ...], outline_role: object,
			fill_role: object | None, width: float, style: PySide6.QtCore.Qt.PenStyle,
			) -> None:
		"""Retain only attached item references and their closed material recipe."""
		if not items or type(width) is not float or width <= 0.0:
			raise TypeError("Ferrum display material requires items and a positive width")
		if fill_role is not None and not isinstance(fill_role, ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1):
			raise TypeError("Ferrum display material requires a document display fill role")
		if not isinstance(outline_role, ferrum_qt.themes.document_display_palette.DocumentDisplayRoleV1):
			raise TypeError("Ferrum display material requires a document display outline role")
		self._items = items
		self._outline_role = outline_role
		self._fill_role = fill_role
		self._width = width
		self._style = style

	#============================================
	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Replace pens and brushes while retaining each item's geometry and state."""
		if type(palette) is not ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1:
			raise TypeError("Ferrum display material requires an exact palette")
		pen = PySide6.QtGui.QPen(palette.color(self._outline_role))
		pen.setWidthF(self._width)
		pen.setStyle(self._style)
		pen.setCosmetic(False)
		brush = None if self._fill_role is None else PySide6.QtGui.QBrush(
			palette.color(self._fill_role),
		)
		for item in self._items:
			set_pen = getattr(item, "setPen", None)
			if not callable(set_pen):
				raise TypeError("Ferrum display material item cannot accept a pen")
			set_pen(pen)
			if brush is not None:
				set_brush = getattr(item, "setBrush", None)
				if not callable(set_brush):
					raise TypeError("Ferrum display material item cannot accept a brush")
				set_brush(brush)


#============================================
class DocumentDisplayDelegatingRefreshableV1:
	"""Adapt one retained renderer-owned item to the tab refresh protocol."""

	def __init__(self, item: object) -> None:
		"""Require the item's typed retained-material refresh contract."""
		refresh = getattr(item, "refresh_display_palette", None)
		if not callable(refresh):
			raise TypeError("Ferrum retained item has no display palette refresh contract")
		self._item = item

	#============================================
	def refresh_document_display_palette(
			self,
			palette: ferrum_qt.themes.document_display_palette.DocumentDisplayPaletteV1,
			) -> None:
		"""Delegate material replacement without changing retained renderer geometry."""
		self._item.refresh_display_palette(palette)


#============================================
def register_attached_document_display_refreshable(
		tab: object, item: object, refreshable: DocumentDisplayRefreshableV1,
		) -> None:
	"""Bind one attached preview item's refresh lifetime to its owning tab."""
	if not isinstance(refreshable, DocumentDisplayRefreshableV1):
		raise TypeError("Ferrum attached display item requires a refreshable")
	register = getattr(tab, "register_document_display_refreshable", None)
	if not callable(register):
		raise TypeError("Ferrum attached display item requires a document tab owner")
	register(refreshable)
	setattr(item, "_ferrum_document_display_refreshable_v1", refreshable)
	setattr(item, "_ferrum_document_display_refreshable_tab_v1", tab)


#============================================
def unregister_attached_document_display_refreshable(item: object | None) -> None:
	"""Release a detached preview before its shared graphics disposal begins."""
	if item is None:
		return
	refreshable = getattr(item, "_ferrum_document_display_refreshable_v1", None)
	tab = getattr(item, "_ferrum_document_display_refreshable_tab_v1", None)
	if refreshable is None and tab is None:
		return
	if not isinstance(refreshable, DocumentDisplayRefreshableV1):
		raise TypeError("Ferrum attached display item has an invalid refreshable")
	unregister = getattr(tab, "unregister_document_display_refreshable", None)
	if not callable(unregister):
		raise TypeError("Ferrum attached display item has an invalid tab owner")
	unregister(refreshable)
	delattr(item, "_ferrum_document_display_refreshable_v1")
	delattr(item, "_ferrum_document_display_refreshable_tab_v1")
