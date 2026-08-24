"""Public document-installation and terminal import-retirement contracts."""

# Standard Library
import pathlib

# PIP3 modules
import ferrum_chem

# local repo modules
import ferrum_qt.ferrum.document_installation
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


def _window(
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager,
		) -> ferrum_qt.main_window.MainWindow:
	"""Create the ordinary product window with isolated theme preferences."""
	return ferrum_qt.main_window.MainWindow(theme_manager)


#============================================
def _sdf(record_count: int) -> str:
	"""Return one or two valid Rust-generated records for one batch import."""
	records = (
		ferrum_chem.prepare_sdf_record(ferrum_chem.parse_smiles("CCO"), "ethanol", ()),
		ferrum_chem.prepare_sdf_record(ferrum_chem.parse_smiles("O"), "water", ()),
	)
	return ferrum_chem.records_to_sdf(
		records[:record_count],
		ferrum_chem.MolblockVersionV1.v2000,
	)


#============================================
def test_public_sdf_installation_receipt_precedes_terminal_import_retirement(
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager, qtbot: object,
		tmp_path: pathlib.Path,
		) -> None:
	"""A multi-record import publishes its receipt before ordinary retirement."""
	window = _window(theme_manager)
	qtbot.addWidget(window)
	path = tmp_path / "two-records.sdf"
	path.write_text(_sdf(2), encoding="utf-8")
	try:
		with qtbot.waitSignal(window.document_import_retired, timeout=10000):
			with qtbot.waitSignal(window.document_installation_completed, timeout=10000) as completed:
				assert window.start_sdf_import(str(path))
		receipt = completed.args[0]
		assert type(receipt) is ferrum_qt.ferrum.document_installation.FerrumDocumentInstallationV1
		assert (
			receipt.installation_kind,
			receipt.installed_record_count,
			receipt.accessible_summary,
		) == (
			"sdf_import", 2, "Ferrum installed 2 SDF records.",
		)
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_close_cancels_import_then_succeeds_after_terminal_retirement(
		theme_manager: ferrum_qt.themes.theme_manager.ThemeManager, qtbot: object,
		) -> None:
	"""An ordinary close waits for terminal import retirement before retrying."""
	window = _window(theme_manager)
	qtbot.addWidget(window)
	try:
		with qtbot.waitSignal(window.document_import_retired, timeout=10000):
			assert window.start_smiles_import("O")
			assert not window.close()
		assert window.close()
	finally:
		window.close()
