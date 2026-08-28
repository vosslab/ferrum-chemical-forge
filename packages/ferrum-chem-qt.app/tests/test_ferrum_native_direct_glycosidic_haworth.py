"""Visible ordinary-native behavior for direct-glycosidic Haworth insertion."""

import os
import unittest.mock

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets

import ferrum_chem
import ferrum_qt.canvas.graphics_disposal
import ferrum_qt.ferrum.close_decision
import ferrum_qt.ferrum.document_display_refresh
import ferrum_qt.themes.theme_loader
import ferrum_qt.themes.theme_manager
import ferrum_qt.main_window
import ferrum_qt.ferrum.document_tab


SMILES = "O1CCCC1OC2CCCCO2"


def _close_test_window(
		qapp: PySide6.QtWidgets.QApplication,
		window: ferrum_qt.main_window.MainWindow,
		) -> None:
	"""Discard owned tabs before ordinary window shutdown can present a prompt."""
	for tab in tuple(window._native_tabs_by_page.values()):
		index = window._tab_widget.indexOf(tab)
		result = window._close_native_tab_at(
			index, ferrum_qt.ferrum.close_decision.CloseDecision.DISCARD,
		)
		assert result is ferrum_qt.ferrum.close_decision.CloseResult.CLOSED
	assert not window._native_tabs_by_page
	window.close()
	window.deleteLater()
	qapp.processEvents()


def _is_direct_glycosidic_profile(molecule: object) -> bool:
	"""Recognize durable C/O bridge and front-depth facts without XML details."""
	atoms = {atom.document_object_id: atom for atom in molecule.atoms}
	if any(atom.element not in {"C", "O"} for atom in atoms.values()):
		return False
	adjacency = {identifier: [] for identifier in atoms}
	for bond in molecule.bonds:
		start = bond.start.document_object_id
		end = bond.end.document_object_id
		if start not in adjacency or end not in adjacency:
			return False
		adjacency[start].append((end, bond))
		adjacency[end].append((start, bond))
	bridges = [
		identifier for identifier, neighbors in adjacency.items()
		if atoms[identifier].element == "O" and len(neighbors) == 2
		and all(atoms[neighbor].element == "C" for neighbor, _ in neighbors)
		and all(
			bond.source_type == "n1" and bond.haworth_position is None
			for _, bond in neighbors
		)
	]
	if len(bridges) != 1:
		return False
	bridge = bridges[0]
	if any(
		bond.source_type != "n1" or bond.haworth_position is not None
		for _, bond in adjacency[bridge]
	):
		return False
	bridge_bond_ids = {bond.document_object_id for _, bond in adjacency[bridge]}
	remaining = set(atoms) - {bridge}
	components: list[set[str]] = []
	while remaining:
		component = {remaining.pop()}
		frontier = list(component)
		while frontier:
			current = frontier.pop()
			for neighbor, bond in adjacency[current]:
				if neighbor in remaining and bond.document_object_id not in bridge_bond_ids:
					remaining.remove(neighbor)
					component.add(neighbor)
					frontier.append(neighbor)
		components.append(component)
	if len(components) != 2:
		return False
	for component in components:
		ring_bonds = [
			bond for atom_id in component for neighbor, bond in adjacency[atom_id]
			if neighbor in component and atom_id < neighbor
		]
		if (
			len(component) not in {5, 6}
			or sum(atoms[atom_id].element == "O" for atom_id in component) != 1
			or len(ring_bonds) != len(component)
		):
			return False
		front_strokes = [bond for bond in ring_bonds if (
			bond.source_type == "q1"
			and bond.haworth_position == ferrum_chem.DocumentHaworthPositionV1.front
		)]
		front_wedges = [bond for bond in ring_bonds if (
			bond.source_type == "w1"
			and bond.haworth_position == ferrum_chem.DocumentHaworthPositionV1.front
		)]
		if len(front_strokes) != 1 or len(front_wedges) != 2:
			return False
		if any(
			bond.source_type != "n1"
			or bond.haworth_position != ferrum_chem.DocumentHaworthPositionV1.back
			for bond in ring_bonds if bond not in front_strokes and bond not in front_wedges
		):
			return False
	return True


def _start_placement(window: PySide6.QtWidgets.QMainWindow,
		qapp: PySide6.QtWidgets.QApplication, exercise_blank_error: bool = False) -> None:
	"""Use the visible Ferrum action and real modal field to arm one request."""
	def accept_dialog() -> None:
		dialog = PySide6.QtWidgets.QApplication.activeModalWidget()
		if dialog is None:
			raise AssertionError("Direct-Glycosidic Haworth action did not open its dialog")
		if exercise_blank_error:
			PySide6.QtTest.QTest.mouseClick(
				dialog.start_button, PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				dialog.start_button.rect().center(),
			)
			PySide6.QtCore.QTimer.singleShot(0, lambda: recover_from_blank(dialog))
			return
		dialog.smiles_edit.setText(SMILES)
		PySide6.QtTest.QTest.mouseClick(
			dialog.start_button, PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			dialog.start_button.rect().center(),
		)

	def recover_from_blank(dialog: PySide6.QtWidgets.QDialog) -> None:
		if "Enter a structural SMILES." not in dialog.smiles_edit.accessibleDescription():
			raise AssertionError("blank request did not reach the field recovery description")
		dialog.smiles_edit.setText(SMILES)
		PySide6.QtTest.QTest.mouseClick(
			dialog.start_button, PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
			dialog.start_button.rect().center(),
		)

	PySide6.QtCore.QTimer.singleShot(0, accept_dialog)
	window._insert_direct_glycosidic_haworth_action.trigger()
	qapp.processEvents()


def test_direct_glycosidic_action_arms_a_real_dialog_and_commits_on_empty_page(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""One visible request reaches Rust's ordinary projection only after a page click."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "direct-haworth.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before = tab.current_snapshot
		_start_placement(window, qapp, exercise_blank_error=True)
		assert window._direct_glycosidic_haworth_intent is not None
		assert tab.current_snapshot == before
		PySide6.QtTest.QTest.mouseClick(
			tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
			PySide6.QtCore.Qt.KeyboardModifier.NoModifier, PySide6.QtCore.QPoint(600, 400),
		)
		assert window._direct_glycosidic_haworth_intent is None
		assert any(
			_is_direct_glycosidic_profile(molecule)
			for molecule in tab.current_document_observation().projection.molecules
		)
	finally:
		_close_test_window(qapp, window)


def test_direct_glycosidic_escape_preserves_the_uncommitted_document(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""Escape cancels the captured receipt instead of creating or redirecting a drawing."""
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "cancel-direct-haworth.cdml",
	ferrum_qt.themes.theme_loader.get_document_display_palette("light"))
	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		before = tab.current_snapshot
		_start_placement(window, qapp)
		PySide6.QtTest.QTest.keyClick(
			tab.view.viewport(), PySide6.QtCore.Qt.Key.Key_Escape,
		)
		assert window._direct_glycosidic_haworth_intent is None
		assert tab.current_snapshot == before
	finally:
		_close_test_window(qapp, window)


def test_direct_glycosidic_preview_releases_palette_before_graphics_disposal(
		qapp: PySide6.QtWidgets.QApplication) -> None:
	"""The direct route ends palette ownership before it removes its real overlay."""
	light = ferrum_qt.themes.theme_loader.get_document_display_palette("light")
	dark = ferrum_qt.themes.theme_loader.get_document_display_palette("dark")
	window = ferrum_qt.main_window.MainWindow(
		ferrum_qt.themes.theme_manager.ThemeManager(qapp),
	)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		"<cdml xmlns='urn:ferrum:cdml'/>", "direct-haworth-palette.cdml", light,
	)
	lifecycle: list[str] = []
	refreshes: list[object] = []
	preview: object | None = None
	register = ferrum_qt.ferrum.document_display_refresh.register_attached_document_display_refreshable
	unregister = ferrum_qt.ferrum.document_display_refresh.unregister_attached_document_display_refreshable
	dispose = ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator.dispose_scene_projection_items

	def record_registration(tab_owner: object, item: object, refreshable: object) -> None:
		nonlocal preview
		preview = item
		refresh = refreshable.refresh_document_display_palette

		def record_refresh(palette: object) -> None:
			refreshes.append(palette)
			refresh(palette)

		refreshable.refresh_document_display_palette = record_refresh
		lifecycle.append("register")
		register(tab_owner, item, refreshable)

	def record_unregistration(item: object | None) -> None:
		if preview is not None and item is preview:
			lifecycle.append("unregister")
		unregister(item)

	def record_disposal(
			coordinator: object, scene: object, items: list[object],
			) -> object:
		if preview is not None and tuple(items) == (preview,):
			lifecycle.append("dispose")
		return dispose(coordinator, scene, items)

	try:
		window._register_native_tab(tab, activate=True)
		window.show()
		qapp.processEvents()
		with (
			unittest.mock.patch.object(
				ferrum_qt.ferrum.document_display_refresh,
				"register_attached_document_display_refreshable",
				side_effect=record_registration,
			),
			unittest.mock.patch.object(
				ferrum_qt.ferrum.document_display_refresh,
				"unregister_attached_document_display_refreshable",
				side_effect=record_unregistration,
			),
			unittest.mock.patch.object(
				ferrum_qt.canvas.graphics_disposal.GraphicsDisposalCoordinator,
				"dispose_scene_projection_items",
				autospec=True,
				side_effect=record_disposal,
			),
		):
			_start_placement(window, qapp)
			PySide6.QtTest.QTest.mouseClick(
				tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton,
				PySide6.QtCore.Qt.KeyboardModifier.NoModifier,
				PySide6.QtCore.QPoint(600, 400),
			)
		assert lifecycle == ["register", "unregister", "dispose"]
		tab.apply_theme_change(ferrum_qt.themes.theme_manager.ThemeChangeV1("dark", dark))
		assert refreshes == []
	finally:
		_close_test_window(qapp, window)
