"""Immutable Qt data used by the Ferrum documentation screenshot workflow."""

# Standard Library
import collections.abc
import dataclasses
import pathlib

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.themes.theme_manager


EMPTY_CDML = "<cdml xmlns='urn:ferrum:cdml' version='26.08'/>"
CARBON_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<molecule id='demo-molecule'>
  <atom id='carbon' name='C'><point x='300' y='360'/></atom>
</molecule>
</cdml>"""
PAIR_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<molecule id='demo-molecule' name='Carbonyl fragment'>
  <atom id='carbon' name='C'><point x='300' y='360'/></atom>
  <atom id='oxygen' name='O'><point x='520' y='360'/></atom>
  <bond id='carbonyl' start='carbon' end='oxygen' type='n2'/>
</molecule>
</cdml>"""
CDXML = (
	'<?xml version="1.0" encoding="UTF-8"?>'
	'<!DOCTYPE CDXML SYSTEM "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd">'
	'<CDXML CreationProgram="ChemDraw 23.0"><page HeightPages="1">'
	'<fragment id="source-fragment"><n id="source-carbon" p="240 360"/>'
	'<n id="source-oxygen" p="440 360" Element="8"/>'
	'<n id="source-nitrogen" p="640 360" Element="7"/>'
	'<n id="source-fluorine" p="840 360" Element="9"/>'
	'<b id="source-wavy" B="source-carbon" E="source-oxygen" Display="Wavy"/>'
	'<b id="source-bold" B="source-oxygen" E="source-nitrogen" Display="Bold"/>'
	'<b id="source-dashed" B="source-nitrogen" E="source-fluorine" Display="Dash"/>'
	'</fragment></page></CDXML>'
)
CATALOG_QUERY = "furan"
DOCUMENTATION_PROPERTY_DOCK_WIDTH = 190


#============================================
@dataclasses.dataclass(frozen=True)
class Scene:
	"""One named screenshot and its completed-state authoring workflow."""

	name: str
	caption: str
	create: collections.abc.Callable[
		[
			PySide6.QtWidgets.QApplication,
			ferrum_qt.themes.theme_manager.ThemeManager,
			pathlib.Path,
		],
		PySide6.QtWidgets.QMainWindow,
	]
	post_prepare: collections.abc.Callable[
		[PySide6.QtWidgets.QMainWindow, PySide6.QtWidgets.QApplication], None
	] | None = None
	overlay_capture: collections.abc.Callable[
		[PySide6.QtWidgets.QMainWindow, pathlib.Path], None
	] | None = None
