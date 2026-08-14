"""Public behavior coverage for Rust-native document drawing defaults."""

# Standard Library
import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui
import PySide6.QtWidgets
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.native.ferrum_native_document_tab
import ferrum_qt.native.ferrum_native_drawing_standard as native_drawing_standard
import ferrum_qt.native.ferrum_native_main_window


SOURCE = (
	'<cdml xmlns:v="urn:vendor"><molecule id="m" v:keep="yes">'
	'<atom id="a" name="O"><point x="10" y="20"/></atom>'
	'</molecule><v:opaque retained="yes"/></cdml>'
)


#============================================
@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide one reusable offscreen Qt application."""
	application = PySide6.QtWidgets.QApplication.instance()
	if application is None:
		application = PySide6.QtWidgets.QApplication([])
	return application


#============================================
def _window_with_source() -> tuple[object, object]:
	"""Return one native product host with a clean selected document."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		SOURCE, "drawing-defaults.cdml",
	)
	window._register_native_tab(tab, activate=True)
	return window, tab


#============================================
def _action(window: object, label: str) -> PySide6.QtGui.QAction:
	"""Find one user-reachable action through the public QObject tree."""
	matches = tuple(
		action for action in window.findChildren(PySide6.QtGui.QAction)
		if action.text() == label
	)
	assert len(matches) == 1
	return matches[0]


#============================================
def _field(dialog: object, field_type: type, accessible_name: str) -> object:
	"""Find one form control through its user-facing accessible name."""
	for field in dialog.findChildren(field_type):
		if field.accessibleName() == accessible_name:
			return field
	raise AssertionError(f"Drawing Defaults is missing {accessible_name!r}.")


#============================================
def test_projection_model_uses_exact_ferrum_fallbacks_and_authored_values() -> None:
	"""Absent fields use product defaults while authored fields remain exact."""
	default = native_drawing_standard.model_from_projection(None)
	session = ferrum_chem.DocumentSession.load(
		'<cdml><standard line_width="2" font_size="18" line_color="#123456" '
		'area_color="#abcdef"><bond width="7" wedge-width="8" '
		'double-ratio="0.4"/><atom show_hydrogens="yes"/></standard></cdml>',
	)
	authored = native_drawing_standard.model_from_projection(
		session.observe(0).projection.drawing_standard,
	)

	assert default == native_drawing_standard.FerrumNativeDrawingStandardModel(
		1.0, 12, "#000000", "", 6.0, 5.0, False,
	)
	assert authored == native_drawing_standard.FerrumNativeDrawingStandardModel(
		2.0, 18, "#123456", "#abcdef", 7.0, 8.0, True,
	)


#============================================
def test_dialog_mapping_rejects_defaults_the_renderer_does_not_honor() -> None:
	"""Font-family and double-ratio edits cannot masquerade as visible changes."""
	with pytest.raises(ValueError, match="unsupported field"):
		native_drawing_standard.changes_from_dialog((("font_family", "Fira Sans"),))
	with pytest.raises(ValueError, match="unsupported field"):
		native_drawing_standard.changes_from_dialog((("double_ratio", 0.6),))


#============================================
def test_public_action_commits_rendered_defaults_and_preserves_selection(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The real menu action accepts one dialog and installs one Rust revision."""
	window, tab = _window_with_source()
	tab.select_atom("a")
	before = tab.current_snapshot
	seen_dialogs = []

	def edit_and_accept() -> None:
		"""Drive the active real modal form through accessible public controls."""
		dialog = qapp.activeModalWidget()
		assert type(dialog) is native_drawing_standard.FerrumNativeDrawingStandardDialog
		seen_dialogs.append(dialog)
		_field(dialog, PySide6.QtWidgets.QDoubleSpinBox, "Default line width").setValue(2.5)
		_field(dialog, PySide6.QtWidgets.QLineEdit, "Default line and text color").setText(
			"#123456",
		)
		_field(
			dialog, PySide6.QtWidgets.QCheckBox,
			"Show heteroatom hydrogens by default",
		).setChecked(True)
		dialog.accept()

	try:
		action = _action(window, "Document Drawing Defaults...")
		assert action.isEnabled()
		PySide6.QtCore.QTimer.singleShot(0, edit_and_accept)
		action.trigger()
		standard = tab.drawing_standard_projection()

		assert seen_dialogs
		assert tab.current_snapshot.revision == before.revision + 1
		assert standard.line_width == 2.5
		assert standard.line_color == "#123456"
		assert standard.show_hydrogens is True
		assert standard.font_size is None and standard.double_ratio is None
		assert tab.selected_atom_projection().source_id == "a"
		assert 'v:keep="yes"' in tab.current_snapshot.cdml
		assert '<v:opaque retained="yes"/>' in tab.current_snapshot.cdml
		_action(window, "Undo").trigger()
		assert tab.current_snapshot.digest == before.digest
	finally:
		_action(window, "Close Tab").trigger()
		window.deleteLater()


#============================================
def test_action_and_tab_adapter_refuse_unavailable_or_wrong_state() -> None:
	"""No active page and non-Ferrum change values cannot enter the operation."""
	window = ferrum_qt.native.ferrum_native_main_window.FerrumNativeMainWindow()
	try:
		assert not _action(window, "Document Drawing Defaults...").isEnabled()
		tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
			SOURCE, "drawing-defaults.cdml",
		)
		window._register_native_tab(tab, activate=True)
		before = tab.current_snapshot
		with pytest.raises(TypeError, match="frozen Ferrum values"):
			tab.apply_drawing_standard((object(),))
		assert tab.current_snapshot == before
	finally:
		while window.centralWidget().count():
			window._close_tab_at(0)
		window.deleteLater()

