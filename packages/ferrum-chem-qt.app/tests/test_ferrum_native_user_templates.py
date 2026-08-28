"""Native action coverage for Rust-owned saved-template publication."""

# Standard Library
import pathlib

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets
import pytest

# local repo modules
import ferrum_qt.ferrum.main_window
import ferrum_qt.ferrum.user_templates


@pytest.fixture
def qapp() -> PySide6.QtWidgets.QApplication:
	"""Provide the normal offscreen Qt application host."""
	app = PySide6.QtWidgets.QApplication.instance()
	return PySide6.QtWidgets.QApplication([]) if app is None else app


#============================================
def test_template_actions_have_one_catalog_route_and_no_retired_placement_action(
		qapp: PySide6.QtWidgets.QApplication, tmp_path: pathlib.Path,
		) -> None:
	"""Saved templates publish and browse through the unified catalog workflow."""
	window = ferrum_qt.ferrum.main_window.FerrumNativeMainWindow(
		user_template_directory=tmp_path / "templates",
	)
	try:
		actions = tuple(window.findChildren(PySide6.QtGui.QAction))
		labels = {action.text() for action in actions}
		assert "Template Catalog..." in labels
		assert "Save Current as Template..." in labels
		assert "Refresh Templates" in labels
		assert "Place User Template..." not in labels
		assert window._action_registry.get_qt_action("chemistry.template.catalog") is not None
		assert window._action_registry.get_qt_action("chemistry.template.place") is None
	finally:
		window.close()
		window.deleteLater()


#============================================
def test_template_save_uses_one_opaque_rust_receipt_then_refreshes(
		tmp_path: pathlib.Path,
		) -> None:
	"""Qt neither serializes CDML nor reparses it before saved-template publication."""
	class Receipt:
		revision = 7
		digest = "a" * 64

	class PublishedSnapshot:
		revision = 7
		digest = "a" * 64

	class Outcome:
		is_confirmed = True

	class Publication:
		published_snapshot = PublishedSnapshot()
		outcome = Outcome()

	class Tab:
		requires_refresh = False
		is_disposed = False

		def __init__(self) -> None:
			self.receipt = Receipt()
			self.calls: list[tuple[object, str]] = []

		def prepare_user_template_publication_v1(self) -> Receipt:
			return self.receipt

		def publish_user_template_v1(self, receipt: object, path: str) -> Publication:
			self.calls.append((receipt, path))
			return Publication()

	class Window(ferrum_qt.ferrum.user_templates.FerrumNativeUserTemplateWindowMixin):
		def __init__(self, tab: Tab) -> None:
			self._user_template_directory = tmp_path
			self._tab = tab
			self._native_tabs_by_page = {tab: tab}
			self.refreshes = 0
			self.refusals: list[object] = []
			self.refresh_result: object | None = object()

		def _active_native_tab(self) -> Tab:
			return self._tab

		def _on_refresh_native_user_templates(self) -> object:
			self.refreshes += 1
			return self.refresh_result

		def _show_edit_refusal(self, refusal: object) -> None:
			self.refusals.append(refusal)

		def _unavailable_edit_refusal(self, message: str) -> str:
			return message

	tab = Tab()
	window = Window(tab)
	destination = tmp_path / "reusable.cdml"

	assert window.save_active_as_user_template_to_path(destination)
	assert tab.calls == [(tab.receipt, str(destination.resolve()))]
	assert window.refreshes == 1
	assert not window.refusals


#============================================
def test_template_save_reports_refresh_uncertainty_without_claiming_success(
		tmp_path: pathlib.Path,
		) -> None:
	"""A completed publication does not imply the catalog successfully refreshed."""
	# The focused publication-path double above is deliberately reused through
	# its local test surface so this assertion stays behavioral rather than GUI-pixel based.
	class Receipt:
		revision = 7
		digest = "a" * 64

	class PublishedSnapshot:
		revision = 7
		digest = "a" * 64

	class Outcome:
		is_confirmed = True

	class Publication:
		published_snapshot = PublishedSnapshot()
		outcome = Outcome()

	class Tab:
		requires_refresh = False

		def prepare_user_template_publication_v1(self) -> Receipt:
			return Receipt()

		def publish_user_template_v1(self, receipt: object, path: str) -> Publication:
			return Publication()

	class Window(ferrum_qt.ferrum.user_templates.FerrumNativeUserTemplateWindowMixin):
		def __init__(self) -> None:
			self._user_template_directory = tmp_path
			self._tab = Tab()
			self.refusals: list[str] = []

		def _active_native_tab(self) -> Tab:
			return self._tab

		def _on_refresh_native_user_templates(self) -> None:
			return None

		def _show_edit_refusal(self, refusal: str) -> None:
			self.refusals.append(refusal)

		def _unavailable_edit_refusal(self, message: str) -> str:
			return message

	window = Window()

	assert not window.save_active_as_user_template_to_path(tmp_path / "reusable.cdml")
	assert any("saved, but the catalog" in message for message in window.refusals)
