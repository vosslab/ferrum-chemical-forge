"""Self-contained release-metadata boundary for the Ferrum-Qt frontend."""

# Standard Library
import importlib.metadata
import pathlib
import re


CALVER_PATTERN = re.compile(
	r"^(?P<year>\d{2})\.(?P<month>\d{1,2})"
	r"(?P<suffix>(?:\.\d+)?(?:(?:a|b|rc)\d+)?)$"
)
ASSIGNMENT_PATTERN = re.compile(
	r"^(?:__version__|VERSION|version)\s*=\s*(?P<version>.+)$"
)


class ReleaseMetadataError(RuntimeError):
	"""Report missing or invalid frontend release metadata."""


#============================================
def _display_version(raw_value: str) -> str:
	"""Normalize one supported CalVer spelling without losing its suffix."""
	value = raw_value.strip()
	assignment = ASSIGNMENT_PATTERN.fullmatch(value)
	if assignment is not None:
		value = assignment.group("version").strip()
		if len(value) >= 2 and value[0] == value[-1] and value[0] in "'\"":
			value = value[1:-1]
	match = CALVER_PATTERN.fullmatch(value)
	if match is None:
		raise ValueError(f"unsupported Ferrum CalVer value: {value!r}")
	month = int(match.group("month"))
	if not 1 <= month <= 12:
		raise ValueError(f"Ferrum CalVer month is outside 1 through 12: {value!r}")
	return f"{match.group('year')}.{month:02d}{match.group('suffix')}"


#============================================
def read_source_tree_display_version(version_path: str | pathlib.Path) -> str:
	"""Return one canonical display version from the supplied source registry.

	Args:
		version_path: Explicit root ``VERSION`` file for a recognized checkout.

	Raises:
		ReleaseMetadataError: If the registry cannot provide one valid version.
	"""
	try:
		registry_text = pathlib.Path(version_path).read_text(encoding="utf-8")
	except OSError as error:
		raise ReleaseMetadataError(f"Unable to read VERSION file: {error}") from error
	try:
		return _display_version(registry_text)
	except ValueError as error:
		raise ReleaseMetadataError(f"Unable to read VERSION file: {error}") from error


#============================================
def installed_display_version(distribution_name: str) -> str:
	"""Return canonical display CalVer from one installed distribution name.

	Args:
		distribution_name: Installed package name resolved through Python metadata.

	Raises:
		ReleaseMetadataError: If metadata is absent or outside Ferrum's CalVer
			profile.
	"""
	try:
		installed_version = importlib.metadata.version(distribution_name)
	except importlib.metadata.PackageNotFoundError as error:
		raise ReleaseMetadataError("Ferrum-Qt package metadata is unavailable") from error
	try:
		return _display_version(installed_version)
	except ValueError as error:
		raise ReleaseMetadataError(
			f"Unsupported installed Ferrum-Qt version metadata: {error}"
		) from error
