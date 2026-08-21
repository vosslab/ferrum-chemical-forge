#!/usr/bin/env python3
"""Offscreen P0.3 atom/bond structural deletion workflow."""

import os
import pathlib

os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

import ferrum_chem
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window

_CDML = """<cdml version='26.08'><molecule id='m'><atom id='a' name='C'><point x='0' y='0'/></atom><atom id='b' name='C'><point x='40' y='0'/></atom><atom id='c' name='O'><point x='80' y='0'/></atom><bond id='ab' start='a' end='b' type='n1'/><bond id='bc' start='b' end='c' type='n1'/></molecule></cdml>"""

def point(tab, identifier):
	observation = tab.observe_structure_interaction()
	target = next(value for value in observation.targets if value.identifier == identifier)
	return tab.view.mapFromScene(PySide6.QtCore.QPointF((target.bounds.left + target.bounds.right) / 2.0, (target.bounds.top + target.bounds.bottom) / 2.0))

def main():
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow()
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(_CDML, "structure.cdml")
	window._register_native_tab(tab, activate=True)
	window.show(); app.processEvents()
	try:
		window._select_structure_action.trigger()
		PySide6.QtTest.QTest.mouseClick(tab.view.viewport(), PySide6.QtCore.Qt.MouseButton.LeftButton, PySide6.QtCore.Qt.KeyboardModifier.NoModifier, point(tab, "b"))
		window._commit_structure_deletion()
		if len(tab.current_document_observation().projection.molecules) != 2:
			raise RuntimeError("structural delete did not split molecule roots")
		moved = tab.current_snapshot.revision
		if tab.undo().observation.snapshot.revision <= moved:
			raise RuntimeError("structural delete undo did not create a Rust revision")
		path = pathlib.Path("/private/tmp/ferrum-p0-structure-delete-e2e.cdml")
		tab.save_atomic(path)
		if ferrum_chem.DocumentSession.load(path.read_text()).snapshot().digest != tab.current_snapshot.digest:
			raise RuntimeError("structural delete save/reopen changed Rust state")
		print('{"schema":"ferrum-p0-structure-delete-e2e-v1","status":"ok"}')
	finally:
		tab.dispose(); window.deleteLater()

if __name__ == "__main__":
	raise SystemExit(main())
