"""Behavioral Qt coverage for the Rust-owned Template Catalog dialog."""

# Standard Library
import dataclasses
import os


os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")

# PIP3 modules
import PySide6.QtCore
import PySide6.QtTest
import PySide6.QtWidgets
import pytest

# local repo modules
from ferrum_qt.ferrum.template_catalog_dialog import FerrumTemplateCatalogDialog


@dataclasses.dataclass(frozen=True, slots=True)
class _Entry:
	key: str
	label: str
	source_kind: str
	family: str | None
	category: str | None
	family_label: str = ""
	category_label: str = ""
	search_terms: tuple[str, ...] = ()
	provenance_source_kind: str = "shipped_recipe"
	provenance_source_id: str = "recipe:benzene"
	provenance_license_spdx: str | None = "CC0-1.0"
	provenance_reviewed_on: str | None = "2026-08-27"
	provenance_chemistry_scope: str | None = "organic"
	content_identity_algorithm: str = "sha256"
	content_identity: str = "a" * 64
	compatibility_profile: str = "ferrum-v1"
	compatibility_format: str = "cdml"


@dataclasses.dataclass(frozen=True, slots=True)
class _Refusal:
	basename: str | None
	category: str
	recovery: str
	occurrences: int = 1


@dataclasses.dataclass(frozen=True, slots=True)
class _Snapshot:
	entries: tuple[_Entry, ...]
	refusals: tuple[_Refusal, ...] = ()
	limits_max_entries: int = 100
	limits_max_candidates: int = 200
	limits_max_refusals: int = 20
	limits_max_file_bytes: int = 4_096
	limits_max_total_bytes: int = 16_384


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the normal offscreen native application host."""
	app = PySide6.QtWidgets.QApplication.instance()
	return PySide6.QtWidgets.QApplication([]) if app is None else app


def _snapshot(*, include_user: bool = True) -> _Snapshot:
	"""Issue deterministic Rust-shaped DTO facts without filesystem discovery."""
	entries = [
		_Entry(
			"built-in:v1:benzene", "Benzene", "shipped", "aromatic", "ring",
			"Aromatic", "Ring", ("benzene", "aromatic ring"),
		),
	]
	if include_user:
		entries.append(_Entry(
			"user:v1:thioether", "Saved sulfur linker", "user_directory", None, None,
			search_terms=("sulfur", "thioether", "sulfide"),
			provenance_source_kind="user_directory",
			provenance_source_id="opaque-user-content",
		))
	return _Snapshot(tuple(entries), (
		_Refusal("broken.cdml", "document_admission", "fix_file"),
		_Refusal(None, "catalog_limit_exceeded", "fix_file", occurrences=3),
	))


#============================================
def test_catalog_labels_have_accessible_filter_buddies(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""Accessible labels identify the three catalog filter controls."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot())
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	dialog.show()
	dialog.activateWindow()
	qapp.processEvents()
	assert dialog.source_label.buddy() is dialog.source
	assert dialog.family_label.buddy() is dialog.family
	assert dialog.category_label.buddy() is dialog.category


#============================================
def test_catalog_projects_native_identity_and_limits(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""Selected entries expose Rust-issued identity and catalog safety facts."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot())
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	assert "Identity: sha256" in dialog.details.text()
	assert "100 entries, 200 candidates, 20 refusals" in dialog.details.text()


#============================================
def test_catalog_searches_native_terms_and_switches_source_facets(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""Alias search and the My templates source use native projected fields."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot())
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	dialog.search.setText("aromatic")
	assert dialog.results.count() == 1
	dialog.search.clear()
	dialog.source.setCurrentIndex(dialog.source.findData("user_directory"))
	assert dialog.results.count() == 1
	assert not dialog.family.isVisible() and not dialog.category.isVisible()
	dialog.search.setText("sulfide")
	assert dialog.results.currentItem().text() == "Saved sulfur linker"
	assert not dialog.family.isVisible() and not dialog.category.isVisible()


#============================================
def test_partial_refusal_explains_aggregate_recovery(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""Admitted saved templates remain available beside aggregate refusal details."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot())
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	dialog.show()
	dialog.source.setCurrentIndex(dialog.source.findData("user_directory"))
	assert "Some neighboring templates" in dialog.state.text()
	assert dialog.refusal_toggle.isVisible()
	dialog.refusal_toggle.setChecked(True)
	assert "broken.cdml (1)" in dialog.refusal_details.toPlainText()
	assert "Template (3)" in dialog.refusal_details.toPlainText()


#============================================
def test_refresh_preserves_source_search_and_selection(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""A fresh native snapshot retains viable user browse choices."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot())
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	dialog.show()
	dialog.source.setCurrentIndex(dialog.source.findData("user_directory"))
	dialog.search.setText("sulfur")
	chosen = dialog.selected_key()
	dialog.replace_snapshot(_snapshot())
	assert dialog.source.currentData() == "user_directory"
	assert dialog.search.text() == "sulfur"
	assert dialog.selected_key() == chosen


#============================================
def test_unavailable_catalog_disables_placement_and_restores_search_focus(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""An unavailable snapshot gives an actionable, keyboard-safe recovery state."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot())
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	dialog.show()
	dialog.activateWindow()
	qapp.processEvents()
	dialog.set_unavailable("Directory needs attention. Refresh.")
	qapp.processEvents()
	assert dialog.results.count() == 0
	assert not dialog.place_button.isEnabled()
	assert dialog.search.hasFocus()


#============================================
def test_keyboard_defaults_and_cancel_are_safe_and_predictable(
		qapp: PySide6.QtWidgets.QApplication, qtbot: object,
		) -> None:
	"""Enter only accepts an admitted result and Escape closes without a placement."""
	parent = PySide6.QtWidgets.QWidget()
	dialog = FerrumTemplateCatalogDialog(parent, _snapshot(include_user=False))
	qtbot.addWidget(parent)
	qtbot.addWidget(dialog)
	dialog.show()
	assert dialog.place_button.isDefault()
	assert not dialog.save_button.isDefault()
	assert not dialog.refresh_button.isDefault()
	PySide6.QtTest.QTest.keyClick(dialog.results, PySide6.QtCore.Qt.Key.Key_Return)
	assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Accepted
	dialog.show()
	PySide6.QtTest.QTest.keyClick(dialog, PySide6.QtCore.Qt.Key.Key_Escape)
	assert dialog.result() == PySide6.QtWidgets.QDialog.DialogCode.Rejected
