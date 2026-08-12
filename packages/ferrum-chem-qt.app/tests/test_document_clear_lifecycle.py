"""Regression coverage for Document.clear graphics ownership."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.document_projection
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.models.atom_model
import ferrum_qt.models.document
import ferrum_qt.models.document_object
import ferrum_qt.models.molecule_model
import ferrum_qt.undo.commands


#============================================
def _drain_deferred_deletes(app: PySide6.QtWidgets.QApplication) -> None:
	"""Deliver deferred QObject destruction deterministically."""
	for _index in range(2):
		PySide6.QtCore.QCoreApplication.sendPostedEvents(
			None, PySide6.QtCore.QEvent.Type.DeferredDelete,
		)
		app.processEvents()


#============================================
def _presentation_model(object_id: str) -> ferrum_qt.models.document_object.PresentationObject:
	"""Return a minimal drawable presentation object with a stable ID."""
	return ferrum_qt.models.document_object.PresentationObject(
		"polyline",
		attributes={"id": object_id},
		points=[(10.0, 10.0, None), (40.0, 40.0, None)],
	)


#============================================
def _required_presentation_item(
		model: ferrum_qt.models.document_object.PresentationObject,
		) -> PySide6.QtWidgets.QGraphicsItem:
	"""Create the supported item required by this lifecycle test."""
	item = ferrum_qt.canvas.document_projection.create_presentation_item(model)
	assert item is not None
	return item


#============================================
def _required_mark_item(mark: ferrum_qt.models.document_object.AtomMarkModel,
		atom_item: ferrum_qt.canvas.items.atom_item.AtomItem,
		) -> PySide6.QtWidgets.QGraphicsItem:
	"""Create the supported mark item required by this lifecycle test."""
	item = ferrum_qt.canvas.document_projection.create_mark_item(mark, atom_item)
	assert item is not None
	return item


#============================================
def _binding_is_released(
		binding: ferrum_qt.canvas.document_projection._ProjectionBinding,
		) -> bool:
	"""Return whether a captured projection binding released its callback edges."""
	return (not binding._connected and binding._model is None
			and binding._item is None and binding._refresh_callback is None)


#============================================
def test_document_clear_disconnects_retained_projection_bindings(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Clear leaves retained graphics unable to react to model mutation."""
	document = ferrum_qt.models.document.Document()
	scene = PySide6.QtWidgets.QGraphicsScene()
	document.set_scene(scene)

	molecule = ferrum_qt.models.molecule_model.MoleculeModel()
	atom = ferrum_qt.models.atom_model.AtomModel()
	molecule.add_atom(atom)
	atom_item = ferrum_qt.canvas.items.atom_item.AtomItem(atom)
	document.add_molecule(molecule, mark_dirty=False)
	scene.addItem(atom_item)

	live_presentation = _presentation_model("live-presentation")
	live_presentation_item = _required_presentation_item(live_presentation)
	mark = ferrum_qt.models.document_object.AtomMarkModel(
		atom, {"type": "plus", "angle": "0"},
	)
	mark_item = _required_mark_item(mark, atom_item)
	document.undo_stack.push(ferrum_qt.undo.commands.AddPresentationObjectCommand(
		document, scene, live_presentation, live_presentation_item,
	))
	document.undo_stack.push(ferrum_qt.undo.commands.AddAtomMarkCommand(
		document, mark, mark_item, atom_item,
	))
	live_presentation_binding = live_presentation_item._projection_binding
	mark_binding = mark_item._projection_binding

	document.clear()
	_drain_deferred_deletes(qapp)

	assert _binding_is_released(live_presentation_binding)
	assert _binding_is_released(mark_binding)

	document.set_scene(None)
	scene.deleteLater()
	_drain_deferred_deletes(qapp)

	document.deleteLater()
	_drain_deferred_deletes(qapp)
