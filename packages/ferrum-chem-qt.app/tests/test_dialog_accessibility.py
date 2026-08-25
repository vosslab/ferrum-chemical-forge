"""Offscreen accessibility contracts shared by constructible Ferrum dialogs."""

# Standard Library
import collections.abc
import pathlib

# PIP3 modules
import pytest
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.dialogs.accessibility
import ferrum_qt.dialogs.about_dialog
import ferrum_qt.dialogs.arrow_dialog
import ferrum_qt.dialogs.geometric_properties_dialog
import ferrum_qt.dialogs.plus_dialog
import ferrum_qt.dialogs.preferences_dialog
import ferrum_qt.dialogs.scale_dialog
import ferrum_qt.dialogs.theme_chooser_dialog
import ferrum_qt.dialogs.wavy_dialog
import ferrum_qt.ferrum.atom_number
import ferrum_qt.ferrum.drawing_standard
import ferrum_qt.themes.theme_loader


#============================================
DialogFactory = collections.abc.Callable[[], PySide6.QtWidgets.QDialog]


# These are standalone dialogs whose inputs are plain UI values.  Dialogs that
# require a live Rust receipt have feature-specific tests with that smallest
# authoritative fixture; this test intentionally never invents a receipt.
DIALOG_FACTORY_METADATA: dict[str, DialogFactory] = {
	"AboutDialog": ferrum_qt.dialogs.about_dialog.AboutDialog,
	"ArrowDialog": ferrum_qt.dialogs.arrow_dialog.ArrowDialog,
	"GeometricPropertiesDialog": lambda: (
		ferrum_qt.dialogs.geometric_properties_dialog.GeometricPropertiesDialog(
			"Line", 1.0, "#000000", None, False,
		)
	),
	"PlusDialog": lambda: ferrum_qt.dialogs.plus_dialog.PlusDialog(12, "#000000"),
	"PreferencesDialog": lambda: ferrum_qt.dialogs.preferences_dialog.PreferencesDialog(
		ferrum_qt.dialogs.preferences_dialog.PreferencesDialogResult(
			ferrum_qt.themes.theme_loader.get_theme_names()[0], False, True, False,
		),
	),
	"ScaleDialog": ferrum_qt.dialogs.scale_dialog.ScaleDialog,
	"ThemeChooserDialog": lambda: (
		ferrum_qt.dialogs.theme_chooser_dialog.ThemeChooserDialog(
			ferrum_qt.themes.theme_loader.get_theme_names()[0],
		)
	),
	"WavyDialog": lambda: ferrum_qt.dialogs.wavy_dialog.WavyDialog(1.0, "#000000"),
	"FerrumNativeAtomNumberDialog": lambda: (
		ferrum_qt.ferrum.atom_number.FerrumNativeAtomNumberDialog(7, True)
	),
	"FerrumNativeDrawingStandardDialog": lambda: (
		ferrum_qt.ferrum.drawing_standard.FerrumNativeDrawingStandardDialog(
			ferrum_qt.ferrum.drawing_standard.FerrumNativeDrawingStandardModel(
				1.0, 12, "#000000", "#ffffff", 6.0, 8.0, True,
			),
		)
	),
}

# Dialogs outside the plain-value set need a live Rust observation, source
# model, or main-window action fixture.  Their owning feature tests construct
# them with that real boundary data.  Keeping the reason adjacent to this
# factory registry prevents a convenient fake object from becoming a second
# document model merely to satisfy a visual test.
DIALOG_FACTORY_EXEMPTIONS = {
	"AtomDialog": "requires a durable atom presentation model",
	"BondDialog": "requires a durable bond presentation model",
	"FerrumNativeBondCapacityDialog": "requires a Rust diagnostic receipt",
	"FerrumNativeDrawingParametersDialog": "requires next-drawing model and action",
	"FerrumNativePreferencesDialog": "requires application settings model",
	"PaperPropertiesDialog": "requires a paper-properties presentation model",
	"RichTextDialog": "requires rich-text run fixture",
	"_CreateExplicitFragmentDialog": "covered by explicit-fragment workflow",
	"_DirectGlycosidicHaworthDialog": "covered by direct-glycosidic workflow",
	"_ExplicitFragmentViewDialog": "requires a Rust fragment observation",
}

DIALOG_SUBCLASS_SOURCES = {
	"about_dialog.py": ("AboutDialog",),
	"arrow_dialog.py": ("ArrowDialog",),
	"atom_dialog.py": ("AtomDialog",),
	"bond_dialog.py": ("BondDialog",),
	"geometric_properties_dialog.py": ("GeometricPropertiesDialog",),
	"paper_properties_dialog.py": ("PaperPropertiesDialog",),
	"plus_dialog.py": ("PlusDialog",),
	"preferences_dialog.py": ("PreferencesDialog",),
	"rich_text_dialog.py": ("RichTextDialog",),
	"scale_dialog.py": ("ScaleDialog",),
	"theme_chooser_dialog.py": ("ThemeChooserDialog",),
	"wavy_dialog.py": ("WavyDialog",),
	"../ferrum/atom_number.py": ("FerrumNativeAtomNumberDialog",),
	"../ferrum/bond_capacity.py": ("FerrumNativeBondCapacityDialog",),
	"../ferrum/direct_glycosidic_haworth_tool.py": ("_DirectGlycosidicHaworthDialog",),
	"../ferrum/drawing_parameters_client.py": ("FerrumNativeDrawingParametersDialog",),
	"../ferrum/drawing_standard.py": ("FerrumNativeDrawingStandardDialog",),
	"../ferrum/explicit_fragments.py": (
		"_CreateExplicitFragmentDialog", "_ExplicitFragmentViewDialog",
	),
	"../ferrum/preferences.py": ("FerrumNativePreferencesDialog",),
}


#============================================
@pytest.mark.parametrize("dialog_name", tuple(DIALOG_FACTORY_METADATA))
def test_live_dialogs_publish_focus_and_semantic_metadata(
		dialog_name: str, qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Every plain-value dialog has an ordered live keyboard contract."""
	dialog = DIALOG_FACTORY_METADATA[dialog_name]()
	dialog.show()
	qapp.processEvents()
	try:
		metadata = ferrum_qt.dialogs.accessibility.DIALOG_ACCESSIBILITY_METADATA[
			dialog_name
		]
		assert dialog.accessibleName()
		assert metadata.initial_focus and metadata.tab_order
		assert dialog.focusWidget() is not None
		for control in dialog.findChildren(PySide6.QtWidgets.QWidget):
			if (
				control.focusPolicy() != PySide6.QtCore.Qt.FocusPolicy.NoFocus
				and control.window() is dialog
			):
				assert control.accessibleName()
		PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Escape)
		assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Rejected
	finally:
		dialog.deleteLater()


#============================================
def test_every_dialog_subclass_uses_the_shared_accessibility_base() -> None:
	"""New dialogs cannot silently opt out of the shared keyboard policy."""
	package_root = pathlib.Path(ferrum_qt.dialogs.__file__).parent
	all_dialog_names = set(DIALOG_FACTORY_METADATA) | set(DIALOG_FACTORY_EXEMPTIONS)
	assert all_dialog_names == {
		name for names in DIALOG_SUBCLASS_SOURCES.values() for name in names
	}
	for relative_path, class_names in DIALOG_SUBCLASS_SOURCES.items():
		contents = (package_root / relative_path).read_text(encoding="utf-8")
		for class_name in class_names:
			assert f"class {class_name}(FerrumAccessibleDialog):" in contents
