#!/usr/bin/env python3
"""Import ordered SDF records through the staged offscreen Ferrum application."""

# Standard Library
import json
import os
import pathlib
import sys
import tempfile


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.document_installation
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


_TWO_RECORD_SDF = """ethanol
  Ferrum

  3  2  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    1.5000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0
    3.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
  1  2  1  0
  2  3  1  0
M  END
$$$$
water
  Ferrum

  1  0  0  0  0  0  0  0  0  0999 V2000
    0.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0
M  END
$$$$
"""


#============================================
class SdfImportE2eError(RuntimeError):
	"""Report one failed public staged SDF-import workflow assertion."""


#============================================
def _active_canvas_tab(window: PySide6.QtWidgets.QMainWindow) -> object:
	"""Return the current document through the product's visible tab control."""
	tabs = window.centralWidget()
	if not isinstance(tabs, PySide6.QtWidgets.QTabWidget):
		raise SdfImportE2eError("Ferrum did not expose its public document tabs")
	tab = tabs.currentWidget()
	if tab is None:
		raise SdfImportE2eError("Ferrum did not select its initial document")
	return tab


#============================================
def _molecule_names(tab: object) -> tuple[str, ...]:
	"""Return the installed molecule names in source order from the Rust observation."""
	projection = tab.current_document_observation().projection
	return tuple(molecule.name for molecule in projection.molecules)


#============================================
def main() -> int:
	"""Run one complete receipt-bounded SDF import against the staged product."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	receipts: list[object] = []
	completion_loop = PySide6.QtCore.QEventLoop()

	def receive_installation(receipt: object) -> None:
		receipts.append(receipt)
		completion_loop.quit()

	window.document_installation_completed.connect(receive_installation)
	try:
		with tempfile.TemporaryDirectory(prefix="ferrum-sdf-import-") as temporary:
			path = pathlib.Path(temporary) / "records.sdf"
			path.write_text(_TWO_RECORD_SDF, encoding="utf-8")
			window.show()
			app.processEvents()
			tab = _active_canvas_tab(window)
			initial_revision = tab.current_snapshot.revision
			if window._sdf_import_route_handle() is None:
				raise SdfImportE2eError("Ferrum did not retain the SDF ingress route handle")
			if not window.start_sdf_import(str(path)):
				raise SdfImportE2eError("Ferrum did not start the supplied SDF import")
			completion_loop.exec()
			if len(receipts) != 1:
				raise SdfImportE2eError("SDF import did not publish one installation receipt")
			receipt = receipts[0]
			if type(receipt) is not ferrum_qt.ferrum.document_installation.FerrumDocumentInstallationV1:
				raise SdfImportE2eError("SDF import did not publish its typed installation receipt")
			if (
				receipt.installation_kind != "sdf_import"
				or receipt.installed_record_count != 2
				or receipt.source_revision != initial_revision
				or receipt.current_revision != initial_revision + 1
			):
				raise SdfImportE2eError("SDF installation receipt did not describe one committed batch")
			if _molecule_names(tab) != ("ethanol", "water"):
				raise SdfImportE2eError("SDF import did not preserve source record order and names")
			print(json.dumps({"schema": "ferrum-sdf-import-e2e-v1", "status": "ok"}))
			return 0
	finally:
		window.deleteLater()


if __name__ == "__main__":
	try:
		raise SystemExit(main())
	except SdfImportE2eError as exc:
		print(f"e2e_sdf_import: {exc}", file=sys.stderr)
		raise SystemExit(1)
