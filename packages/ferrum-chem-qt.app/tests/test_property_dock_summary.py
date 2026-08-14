"""Behavior checks for the visible PropertyDock document summary."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.main_window
import ferrum_qt.qt_lifecycle
import ferrum_qt.legacy.compatibility_lifecycle
import ferrum_qt.models.document_object
import ferrum_qt.models.molecule_model


#============================================
def test_visible_document_summary_tracks_object_addition_and_removal(
		main_window: ferrum_qt.main_window.MainWindow,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""The visible dock updates from public document object signals."""
	document = main_window.document
	dock = main_window._property_dock
	molecule = ferrum_qt.models.molecule_model.MoleculeModel()
	drawing = ferrum_qt.models.document_object.PresentationObject("arrow")
	dock.show()
	qapp.processEvents()
	try:
		assert dock.summary_text == "No drawable objects"
		document.add_molecule(molecule, mark_dirty=False)
		assert dock.summary_text == "1 molecule, 0 atoms"
		document.add_presentation_object(drawing, mark_dirty=False)
		assert dock.summary_text == "1 molecule, 0 atoms, 1 drawing object"
		document.remove_molecule(molecule, mark_dirty=False)
		assert dock.summary_text == "1 drawing object"
		document.remove_presentation_object(drawing, mark_dirty=False)
		assert dock.summary_text == "No drawable objects"
	finally:
		if molecule in document.molecules:
			document.remove_molecule(molecule, mark_dirty=False)
		if drawing in document.presentation_objects:
			document.remove_presentation_object(drawing, mark_dirty=False)
		assert ferrum_qt.qt_lifecycle.delete_qobject_and_wait(qapp, molecule)
		assert ferrum_qt.qt_lifecycle.delete_qobject_and_wait(qapp, drawing)
