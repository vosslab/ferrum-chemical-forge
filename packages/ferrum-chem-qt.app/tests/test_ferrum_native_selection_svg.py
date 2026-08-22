"""Behavior coverage for Ferrum selected-root SVG clipboard publication."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import defusedxml.ElementTree
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.canvas.items.ferrum_plus_item
import ferrum_qt.io.clipboard_mime
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.selection_svg


_SOURCE = """\
<cdml xmlns="urn:ferrum:cdml" version="26.07"><plus id="p"><point x="40" y="20"/></plus>
<molecule id="near"><atom id="a" name="C"><point x="10" y="20"/></atom>
 <atom id="b" name="O"><point x="25" y="20"/></atom>
 <bond id="ab" start="a" end="b" type="n1"/></molecule>
<molecule id="far"><atom id="z" name="N"><point x="300" y="20"/></atom></molecule>
</cdml>
"""


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one reusable offscreen Qt application."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _window_with_selection() -> tuple[object, object]:
	"""Return one Ferrum window with atom and presentation roots selected."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		_SOURCE, "selection-svg.cdml",
	)
	window._register_native_tab(tab, activate=True)
	tab.select_atom("a")
	plus_items = tuple(
		item for item in tab.view.scene().items()
		if type(item) is ferrum_qt.canvas.items.ferrum_plus_item.FerrumPlusItem
	)
	if len(plus_items) != 1:
		raise RuntimeError("Ferrum Plus projection is unavailable")
	plus_items[0].setSelected(True)
	return window, tab


#============================================
def _action(window: object, label: str) -> PySide6.QtGui.QAction:
	"""Return one user-reachable action by its visible text."""
	matches = tuple(
		action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == label
	)
	if len(matches) != 1:
		raise RuntimeError("Ferrum action is unavailable: %s" % label)
	return matches[0]


#============================================
def _wait_for_svg(window: object, qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Wait for the action-created SVG worker and queued delivery."""
	workers = tuple(
		worker for worker in window.findChildren(
			ferrum_qt.ferrum.selection_svg.
			FerrumNativeSelectionSvgWorker,
		)
	)
	if len(workers) != 1 or not workers[0].wait(10000):
		raise RuntimeError("Ferrum selected SVG worker did not finish")
	qapp.processEvents()


#============================================
def test_copy_as_svg_publishes_fitted_native_roots_without_mutation(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The public action publishes selected roots from one unchanged observation."""
	window, tab = _window_with_selection()
	before = tab.current_snapshot
	try:
		_action(window, "Copy as SVG").trigger()
		_wait_for_svg(window, qapp)
		mime_data = qapp.clipboard().mimeData()
		svg = bytes(mime_data.data("image/svg+xml"))
		root = defusedxml.ElementTree.fromstring(svg)
		view_box = tuple(float(value) for value in root.attrib["viewBox"].split())

		assert (
			mime_data.hasFormat("image/svg+xml")
			and mime_data.property(
				ferrum_qt.io.clipboard_mime.FERRUM_OWNED_MIME_PROPERTY,
			) is True
			and mime_data.text() == svg.decode("utf-8")
			and view_box[0] + view_box[2] < 100.0
		)
		assert (
			tab.current_snapshot.revision,
			tab.current_snapshot.digest,
		) == (before.revision, before.digest)
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_copy_as_svg_failure_preserves_existing_clipboard(
		qapp: PySide6.QtWidgets.QApplication,
		monkeypatch: pytest.MonkeyPatch) -> None:
	"""A private renderer failure reaches the UI without replacing user data."""
	window, _tab = _window_with_selection()
	warnings = []
	qapp.clipboard().setText("clipboard-before-native-svg")
	monkeypatch.setattr(
		ferrum_chem, "render_document_selection_svg_v1",
		lambda *_arguments: (_ for _ in ()).throw(RuntimeError("injected SVG failure")),
	)
	window._show_edit_refusal = lambda request: warnings.append(request)
	try:
		_action(window, "Copy as SVG").trigger()
		_wait_for_svg(window, qapp)

		assert qapp.clipboard().text() == "clipboard-before-native-svg"
		assert len(warnings) == 1 and warnings[-1].outcome.value == "unavailable_operation" and warnings[-1].technical_details == "injected SVG failure"
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()
