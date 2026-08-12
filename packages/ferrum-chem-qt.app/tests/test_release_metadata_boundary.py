"""Behavior and import-boundary coverage for Qt release metadata."""

# Standard Library
import ast
import importlib.metadata
import pathlib

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.bridge.release_metadata
import ferrum_qt.versioning


_QT_PACKAGE_ROOT = pathlib.Path(__file__).parents[1] / "ferrum_qt"
_ORDINARY_QT_MODULES = (
	"versioning.py",
	"app.py",
	"cli.py",
	"dialogs/about_dialog.py",
)


#============================================
def test_source_registry_produces_the_frontend_display_version(tmp_path: pathlib.Path) -> None:
	"""A recognized checkout obtains its user-facing label from root VERSION."""
	version_path = tmp_path / "VERSION"
	version_path.write_text("version = 26.07\n", encoding="utf-8")

	assert ferrum_qt.bridge.release_metadata.read_source_tree_display_version(version_path) == "26.07"


#============================================
def test_invalid_source_registry_reports_the_typed_boundary_failure(
		tmp_path: pathlib.Path,
		) -> None:
	"""A malformed checkout registry cannot become a guessed display label."""
	version_path = tmp_path / "VERSION"
	version_path.write_text("version = invalid\n", encoding="utf-8")

	with pytest.raises(
		ferrum_qt.bridge.release_metadata.ReleaseMetadataError,
		match="Unable to read VERSION file",
	):
		ferrum_qt.bridge.release_metadata.read_source_tree_display_version(version_path)


#============================================
def test_installed_metadata_normalizes_to_the_frontend_display_version(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Wheel metadata becomes the same zero-padded label used by the CLI and UI."""
	monkeypatch.setattr(
		ferrum_qt.bridge.release_metadata.importlib.metadata,
		"version", lambda _name: "26.7",
	)

	assert ferrum_qt.bridge.release_metadata.installed_display_version("ferrum-qt") == "26.07"


#============================================
def test_invalid_installed_metadata_reports_the_typed_boundary_failure(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""Unexpected wheel metadata fails explicitly instead of inventing a release label."""
	monkeypatch.setattr(
		ferrum_qt.bridge.release_metadata.importlib.metadata,
		"version", lambda _name: "not-a-ferrum-release",
	)

	with pytest.raises(
		ferrum_qt.bridge.release_metadata.ReleaseMetadataError,
		match="Unsupported installed Ferrum-Qt version metadata",
	):
		ferrum_qt.bridge.release_metadata.installed_display_version("ferrum-qt")


#============================================
def test_missing_installed_metadata_reports_the_typed_boundary_failure(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""An installed-layout lookup identifies absent package metadata clearly."""
	def missing_metadata(_name: str) -> str:
		raise importlib.metadata.PackageNotFoundError

	monkeypatch.setattr(
		ferrum_qt.bridge.release_metadata.importlib.metadata,
		"version", missing_metadata,
	)

	with pytest.raises(
		ferrum_qt.bridge.release_metadata.ReleaseMetadataError,
		match="Ferrum-Qt package metadata is unavailable",
	):
		ferrum_qt.bridge.release_metadata.installed_display_version("ferrum-qt")


#============================================
def test_installed_application_version_uses_the_ferrum_distribution_identity(
		monkeypatch: pytest.MonkeyPatch,
		) -> None:
	"""The retained Python namespace looks up the renamed wheel distribution."""
	distribution_names: list[str] = []
	monkeypatch.setattr(ferrum_qt.versioning, "_source_tree_version", lambda: None)
	monkeypatch.setattr(
		ferrum_qt.versioning.ferrum_qt.bridge.release_metadata,
		"installed_display_version",
		lambda name: distribution_names.append(name) or "26.08",
	)

	assert ferrum_qt.versioning.application_version() == "26.08"
	assert distribution_names == ["ferrum-qt"]


#============================================
def test_release_ui_modules_are_independent_of_the_historical_backend() -> None:
	"""Release UI and its metadata bridge retain no OASA import."""
	def oasa_imports(relative_path: str) -> tuple[str, ...]:
		tree = ast.parse(
			(_QT_PACKAGE_ROOT / relative_path).read_text(encoding="utf-8"),
			filename=relative_path,
		)
		imports: list[str] = []
		for node in ast.walk(tree):
			if isinstance(node, ast.Import):
				imports.extend(
					alias.name for alias in node.names
					if alias.name == "oasa" or alias.name.startswith("oasa.")
				)
			elif isinstance(node, ast.ImportFrom) and node.module is not None:
				if node.module == "oasa" or node.module.startswith("oasa."):
					imports.append(node.module)
		return tuple(imports)

	ordinary_imports = {
		relative_path: oasa_imports(relative_path)
		for relative_path in _ORDINARY_QT_MODULES
	}

	assert not any(ordinary_imports.values())
	assert not oasa_imports("bridge/release_metadata.py")
