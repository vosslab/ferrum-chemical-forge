"""Preference storage identity checks for Ferrum-Qt."""

# Standard Library
import pathlib

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.app
import ferrum_qt.config.preferences


#============================================
def test_preferences_use_the_ferrum_product_identity() -> None:
	"""QSettings uses the displayed Ferrum product identity."""
	preferences = ferrum_qt.config.preferences.Preferences()

	assert preferences._settings.organizationName() == ferrum_qt.config.preferences.SETTINGS_ORGANIZATION
	assert preferences._settings.applicationName() == ferrum_qt.config.preferences.SETTINGS_APPLICATION


#============================================
def test_user_templates_use_the_ferrum_application_directory(
		monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""New user templates live under Ferrum's product directory."""
	monkeypatch.setattr(pathlib.Path, "home", lambda: tmp_path)

	directory = ferrum_qt.app.default_user_template_directory()

	assert directory == tmp_path / ".ferrum" / "templates"
