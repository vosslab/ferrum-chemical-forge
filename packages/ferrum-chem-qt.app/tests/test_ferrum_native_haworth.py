"""Visible ordinary-native behavior for standalone D-glucose Haworth insertion."""

import os

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_qt.main_window
import ferrum_qt.native.ferrum_native_document_tab


def _open_haworth_chooser(window: PySide6.QtWidgets.QMainWindow,
		label: str, qapp: PySide6.QtWidgets.QApplication) -> PySide6.QtWidgets.QDialog:
	"""Open the labelled Haworth action through the visible Edit menu."""
	for menu_action in window.menuBar().actions():
		menu = menu_action.menu()
		if menu is not None and menu.title().replace("&", "") == "Edit":
			for candidate in menu.actions():
				if candidate.text().replace("&", "") == label:
					PySide6.QtTest.QTest.mouseClick(
						window.menuBar(), PySide6.QtCore.Qt.MouseButton.LeftButton,
						PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
						window.menuBar().actionGeometry(menu_action).center(),
					)
					qapp.processEvents()
					PySide6.QtTest.QTest.mouseClick(
						menu, PySide6.QtCore.Qt.MouseButton.LeftButton,
						PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
						menu.actionGeometry(candidate).center(),
					)
					qapp.processEvents()
					dialog = PySide6.QtWidgets.QApplication.activeModalWidget()
					if isinstance(dialog, PySide6.QtWidgets.QDialog):
						return dialog
					raise AssertionError("Insert Haworth Ring did not open its chooser")
	raise AssertionError(f"No visible Edit action is labelled {label!r}")


def _choose_haworth_recipe(
		window: PySide6.QtWidgets.QMainWindow, form: str, anomer: str,
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Open the retained chooser normally and choose one visible recipe."""
	dialog = _open_haworth_chooser(window, "Insert Haworth Ring...", qapp)
	for button in dialog.findChildren(PySide6.QtWidgets.QRadioButton):
		if button.text() in (form, anomer):
			PySide6.QtTest.QTest.mouseClick(
				button, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier, button.rect().center(),
			)
	buttons = dialog.findChild(PySide6.QtWidgets.QDialogButtonBox)
	if buttons is None:
		raise AssertionError("Insert Haworth Ring has no confirmation control")
	accept = buttons.button(PySide6.QtWidgets.QDialogButtonBox.StandardButton.Ok)
	PySide6.QtTest.QTest.mouseClick(
		accept, PySide6.QtCore.Qt.MouseButton.LeftButton,
		PySide6.QtCore.Qt.KeyboardModifier.NoModifier, accept.rect().center(),
	)
	PySide6.QtTest.QTest.qWait(1)


def _ring_size(molecule: object) -> int:
	"""Find the heavy-atom Haworth cycle after hydroxyl and chain leaves are stripped."""
	adjacency = {atom.source_id: set() for atom in molecule.atoms}
	for bond in molecule.bonds:
		adjacency[bond.start.source_id].add(bond.end.source_id)
		adjacency[bond.end.source_id].add(bond.start.source_id)
	changed = True
	while changed:
		leaves = [identifier for identifier, neighbors in adjacency.items() if len(neighbors) < 2]
		changed = bool(leaves)
		for identifier in leaves:
			for neighbor in adjacency.pop(identifier):
				adjacency[neighbor].remove(identifier)
	return len(adjacency)


def _is_haworth_projection(molecule: object) -> bool:
	"""Recognize durable C6O6 single-bond Haworth depiction facts."""
	return (
		sum(atom.element == "C" for atom in molecule.atoms) == 6
		and sum(atom.element == "O" for atom in molecule.atoms) == 6
		and len(molecule.bonds) == 12
		and {bond.source_type for bond in molecule.bonds} == {"n1", "q1", "w1"}
	)


def test_insert_haworth_ring_uses_visible_chooser_and_one_shot_rust_placement(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""Pyranose and furanose choices commit their distinct Rust-owned cycles once."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		"<cdml/>", "haworth.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		_choose_haworth_recipe(
			window, "Six-membered pyranose", "alpha", qapp,
		)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPoint(143, 91),
		)
		first_snapshot = tab.current_snapshot
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPoint(318, 207),
		)
		assert tab.current_snapshot == first_snapshot

		_choose_haworth_recipe(
			window, "Five-membered furanose", "beta", qapp,
		)
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPoint(453, 241),
		)
		molecules = tab.current_document_observation().projection.molecules
		selected = tab._controller.projection.selected_durable_targets()
		latest_atoms = {atom.source_id for atom in molecules[-1].atoms}

		assert (
			all(_is_haworth_projection(molecule) for molecule in molecules)
			and sorted(_ring_size(molecule) for molecule in molecules) == [5, 6]
			and selected
			and all(target.kind == "atom" and target.identifier in latest_atoms for target in selected)
		)
	finally:
		window.close()
		window.deleteLater()
		qapp.processEvents()


def test_insert_haworth_ring_preserves_an_occupied_document_and_selection(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A detached Haworth action leaves an occupied page and selection untouched."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'>"
		"<point x='10' y='20'/></atom></molecule></cdml>",
		"occupied-haworth.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab._controller.projection.select_durable((("atom", "a"),))
		before_snapshot = tab.current_snapshot
		before_selection = tab._controller.projection.selected_durable_targets()
		_choose_haworth_recipe(
			window, "Six-membered pyranose", "alpha", qapp,
		)
		occupied = tab.view.mapFromScene(PySide6.QtCore.QPointF(10.0, 20.0))
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, occupied,
		)

		assert (
			tab.current_snapshot == before_snapshot
			and tab._controller.projection.selected_durable_targets() == before_selection
		)
	finally:
		window.close()
		window.deleteLater()
		qapp.processEvents()


def test_insert_haworth_ring_refuses_a_bond_and_keeps_its_intent_armed(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""A bond hit leaves state alone, then the same choice accepts an empty page."""
	window = ferrum_qt.main_window.MainWindow(object())
	window.resize(1400, 900)
	tab = ferrum_qt.native.ferrum_native_document_tab.FerrumNativeDocumentTab(
		"<cdml><molecule id='m'><atom id='a' name='C'><point x='10' y='20'/></atom>"
		"<atom id='b' name='C'><point x='50' y='20'/></atom>"
		"<bond id='ab' start='a' end='b' type='n1'/></molecule></cdml>",
		"occupied-haworth-bond.cdml",
	)
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		tab.select_atom("a")
		before_snapshot = tab.current_snapshot
		before_selection = tab.selected_molecule_information_targets()
		_choose_haworth_recipe(window, "Six-membered pyranose", "alpha", qapp)
		occupied = tab.view.mapFromScene(PySide6.QtCore.QPointF(30.0, 20.0))
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, occupied,
		)

		assert (
			tab.current_snapshot == before_snapshot
			and tab.selected_molecule_information_targets() == before_selection
		)

		empty = tab.view.mapFromScene(PySide6.QtCore.QPointF(300.0, 300.0))
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, empty,
		)
		assert any(
			_is_haworth_projection(molecule)
			for molecule in tab.current_document_observation().projection.molecules
		)
	finally:
		window.close()
		window.deleteLater()
		qapp.processEvents()
