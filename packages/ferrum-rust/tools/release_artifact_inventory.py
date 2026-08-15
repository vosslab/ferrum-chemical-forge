#!/usr/bin/env python3
"""Classify final Ferrum release artifacts after the M20 target-native proof.

This maintainer-only command consumes the two first-party wheels, one source
archive, and the completed M20 package receipt.  It checks the declared M22
artifact roles for the supported macOS arm64 / CPython 3.12 release boundary.
It does not make a legal decision, build artifacts, run an installer, inspect
historical documentation, or treat digest equality as a release gate.
"""

from __future__ import annotations

# Standard Library
import argparse
import email.parser
import json
import pathlib
import re
import sys
import tarfile
import zipfile


RESULT_SCHEMA = "ferrum-release-artifact-inventory-v1"
RECEIPT_SCHEMA = "ferrum-release-package-receipt-v1"
TARGET = {"platform": "macos", "architecture": "arm64", "python": "3.12"}
NATIVE_NOTICE_ROLES = {
	"FERRUM-CHEM-LGPL-3.0.txt",
	"RDKIT-BSD-3-CLAUSE.txt",
	"INCHI-MIT.txt",
	"TELEX-OFL-1.1.txt",
	"THIRD_PARTY_NOTICES.md",
}
FORBIDDEN_DISTRIBUTIONS = {"oasa", "rdkit", "tk", "tcl", "tkinter"}
FORBIDDEN_MODULES = {"oasa", "rdkit", "tk", "tcl", "tkinter"}


class InventoryError(RuntimeError):
	"""One release artifact cannot establish its required M22 predicate."""


#============================================
def canonical_name(value: str) -> str:
	"""Normalize a Python distribution or module name for semantic matching.

	Args:
		value: Name text from packaging metadata or a member path.

	Returns:
		Lowercase name with punctuation runs normalized to one dash.
	"""
	result = re.sub(r"[-_.]+", "-", value).lower()
	return result


#============================================
def normalized_version(value: str) -> tuple[int | str, ...]:
	"""Return a small PEP-440-like comparison key for known release versions.

	Args:
		value: Declared source, wheel, or receipt version.

	Returns:
		Comparable segments that intentionally treat ``26.08`` and ``26.8.0`` alike.

	Raises:
		InventoryError: When a release identity contains an unsupported version form.
	"""
	match = re.fullmatch(r"([0-9]+)(?:\.([0-9]+))?(?:\.([0-9]+))?", value)
	if match is None:
		raise InventoryError(f"identity phase found unsupported release version: {value}")
	segments = [int(part) for part in match.groups(default="0")]
	while len(segments) > 1 and segments[-1] == 0:
		segments.pop()
	result = tuple(segments)
	return result


#============================================
def require_file(value: str, label: str) -> pathlib.Path:
	"""Resolve one explicit regular-file input.

	Args:
		value: Command-line file text.
		label: Human-readable input role.

	Returns:
		Resolved regular path.

	Raises:
		argparse.ArgumentTypeError: When the supplied path is not a regular file.
	"""
	path = pathlib.Path(value).expanduser().resolve()
	if not path.is_file():
		raise argparse.ArgumentTypeError(f"{label} must be an existing regular file: {path}")
	return path


#============================================
def read_json_object(path: pathlib.Path, phase: str) -> dict:
	"""Read one required JSON object with a stable release phase label.

	Args:
		path: JSON receipt path.
		phase: Human-readable release phase.

	Returns:
		Decoded object.

	Raises:
		InventoryError: When the receipt is unreadable or not object-shaped.
	"""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
		raise InventoryError(f"{phase} phase cannot read JSON record: {path}") from error
	if not isinstance(value, dict):
		raise InventoryError(f"{phase} phase needs a JSON object: {path}")
	return value


#============================================
def wheel_members(path: pathlib.Path, label: str) -> tuple[list[str], dict[str, bytes]]:
	"""Read regular wheel members without assigning meaning to their order.

	Args:
		path: Wheel archive path.
		label: Artifact role used in failures.

	Returns:
		Member names and their contents indexed by member name.

	Raises:
		InventoryError: When the supplied archive is unreadable or ambiguous.
	"""
	try:
		with zipfile.ZipFile(path) as archive:
			members = []
			contents = {}
			for info in archive.infolist():
				if info.is_dir():
					continue
				if info.filename in contents:
					raise InventoryError(
						f"{label} phase found duplicate wheel member: {info.filename}"
					)
				members.append(info.filename)
				contents[info.filename] = archive.read(info.filename)
	except (OSError, zipfile.BadZipFile) as error:
		raise InventoryError(f"{label} phase cannot read wheel: {path}") from error
	return members, contents


#============================================
def member_with_suffix(members: list[str], suffix: str, label: str) -> str:
	"""Find one semantic wheel member role rather than a member-count contract.

	Args:
		members: Archive member paths.
		suffix: Required role suffix.
		label: Human-readable role.

	Returns:
		The one matching member path.

	Raises:
		InventoryError: When the role is absent or ambiguous.
	"""
	candidates = [member for member in members if member.endswith(suffix)]
	if not candidates:
		raise InventoryError(f"wheel structure phase lacks {label}")
	if len(candidates) != 1:
		raise InventoryError(f"wheel structure phase has ambiguous {label}: {candidates}")
	result = candidates[0]
	return result


#============================================
def parse_metadata(
		contents: dict[str, bytes], metadata_member: str, label: str,
		) -> email.message.Message:
	"""Parse one wheel METADATA role.

	Args:
		contents: Wheel member bytes indexed by path.
		metadata_member: Selected metadata member path.
		label: Artifact role used in failures.

	Returns:
		Parsed RFC 822 metadata.

	Raises:
		InventoryError: When metadata cannot identify its distribution and version.
	"""
	try:
		text = contents[metadata_member].decode("utf-8")
	except UnicodeDecodeError as error:
		raise InventoryError(f"metadata phase found non-UTF-8 {label} METADATA") from error
	metadata = email.parser.Parser().parsestr(text)
	if metadata["Name"] is None or metadata["Version"] is None:
		raise InventoryError(f"metadata phase found incomplete {label} METADATA")
	return metadata


#============================================
def requirement_name(value: str) -> str:
	"""Return the distribution token from one Requires-Dist declaration.

	Args:
		value: Complete core-metadata requirement string.

	Returns:
		Canonical distribution name.

	Raises:
		InventoryError: When a requirement lacks a usable distribution token.
	"""
	match = re.match(r"\s*([A-Za-z0-9_.-]+)", value)
	if match is None:
		raise InventoryError(f"dependency phase found malformed Requires-Dist: {value}")
	result = canonical_name(match.group(1))
	return result


#============================================
def require_distribution(metadata: email.message.Message, expected: str, label: str) -> str:
	"""Validate one wheel's distribution identity and return its declared version.

	Args:
		metadata: Parsed wheel metadata.
		expected: Expected normalized distribution name.
		label: Human-readable artifact role.

	Returns:
		The declared wheel version.

	Raises:
		InventoryError: When the wheel identity does not match its release role.
	"""
	actual = canonical_name(metadata["Name"])
	if actual != expected:
		raise InventoryError(f"identity phase expected {expected}, found {actual} in {label} wheel")
	version = metadata["Version"]
	normalized_version(version)
	return version


#============================================
def require_license(metadata: email.message.Message, token: str, label: str) -> None:
	"""Require one semantic license expression from wheel metadata.

	Args:
		metadata: Parsed wheel metadata.
		token: Expected lowercase license expression fragment.
		label: Artifact role used in failures.

	Raises:
		InventoryError: When no declared metadata license expresses the role.
	"""
	values = metadata.get_all("License", []) + metadata.get_all("License-Expression", [])
	if not any(token in value.lower() for value in values):
		raise InventoryError(f"notice phase {label} metadata does not declare {token}")


#============================================
def inspect_dependency_boundary(metadata: email.message.Message, label: str) -> set[str]:
	"""Reject forbidden production requirement names and return declared names.

	Args:
		metadata: Parsed wheel metadata.
		label: Artifact role used in failures.

	Returns:
		Normalized declared requirement names.

	Raises:
		InventoryError: When metadata requests a forbidden production dependency.
	"""
	requirements = {requirement_name(value) for value in metadata.get_all("Requires-Dist", [])}
	for forbidden in FORBIDDEN_DISTRIBUTIONS:
		if forbidden in requirements:
			raise InventoryError(
				f"dependency phase {label} wheel requires prohibited {forbidden} distribution"
			)
	return requirements


#============================================
def inspect_importable_boundary(members: list[str], label: str) -> None:
	"""Reject importable OASA, Python-RDKit, and Tk/Tcl module payloads.

	Args:
		members: Wheel member names.
		label: Artifact role used in failures.

	Raises:
		InventoryError: When an importable forbidden module is packaged.
	"""
	for member in members:
		first = member.split("/", 1)[0]
		module = canonical_name(first.split(".", 1)[0])
		if module in FORBIDDEN_MODULES:
			raise InventoryError(
				f"runtime boundary phase {label} wheel exposes prohibited {module} module payload"
			)


#============================================
def inspect_wheel_tag(contents: dict[str, bytes], members: list[str]) -> None:
	"""Confirm the native wheel names the admitted interpreter and platform.

	Args:
		contents: Wheel member bytes indexed by path.
		members: Wheel member paths.

	Raises:
		InventoryError: When WHEEL does not contain an admitted target tag.
	"""
	wheel_member = member_with_suffix(members, ".dist-info/WHEEL", "WHEEL metadata")
	try:
		text = contents[wheel_member].decode("utf-8")
	except UnicodeDecodeError as error:
		raise InventoryError("runtime boundary phase found non-UTF-8 native WHEEL metadata") from error
	tags = [line.removeprefix("Tag: ") for line in text.splitlines() if line.startswith("Tag: ")]
	if not any("cp312" in tag and "macosx" in tag and "arm64" in tag for tag in tags):
		raise InventoryError("runtime boundary phase native wheel lacks a CPython 3.12 macOS arm64 tag")


#============================================
def inspect_native_wheel(path: pathlib.Path) -> dict:
	"""Classify Ferrum-Chem packaging roles and its intentional native closure.

	Args:
		path: Final Ferrum-Chem wheel.

	Returns:
		Small machine-readable classification record.

	Raises:
		InventoryError: When required native wheel roles are absent or forbidden.
	"""
	members, contents = wheel_members(path, "native wheel")
	metadata_member = member_with_suffix(members, ".dist-info/METADATA", "native METADATA")
	metadata = parse_metadata(contents, metadata_member, "native")
	version = require_distribution(metadata, "ferrum-chem", "native")
	require_license(metadata, "lgpl-3.0", "native")
	requirements = inspect_dependency_boundary(metadata, "native")
	inspect_importable_boundary(members, "native")
	inspect_wheel_tag(contents, members)
	if not any(re.fullmatch(r"ferrum_chem[^/]*\.so", member) for member in members):
		raise InventoryError("runtime boundary phase native wheel lacks root Ferrum extension")
	if "ferrum-operation-v1.schema.json" not in contents:
		raise InventoryError("runtime boundary phase native wheel lacks protocol schema resource")
	if not any(member == ".dylibs/libferrum_chem.dylib" for member in members):
		raise InventoryError("runtime boundary phase native wheel lacks package-relative Ferrum loader")
	if not any(
			member.startswith(".dylibs/libRDKit") and member.endswith(".dylib")
			for member in members
		):
		raise InventoryError("runtime boundary phase native wheel lacks intentional RDKit closure")
	notice_prefix = metadata_member.rsplit("/", 1)[0] + "/licenses/"
	notices = {
		member.removeprefix(notice_prefix) for member in members if member.startswith(notice_prefix)
	}
	missing_notices = NATIVE_NOTICE_ROLES.difference(notices)
	if missing_notices:
		raise InventoryError(
			"notice phase native wheel lacks required notice role(s): " + ", ".join(sorted(missing_notices))
		)
	declared_notices = set(metadata.get_all("License-File", []))
	expected_declarations = {"licenses/" + name for name in NATIVE_NOTICE_ROLES}
	missing_declarations = expected_declarations.difference(declared_notices)
	if missing_declarations:
		raise InventoryError(
			"notice phase native metadata lacks license-file role(s): "
			+ ", ".join(sorted(missing_declarations))
		)
	result = {
		"distribution": "ferrum-chem",
		"version": version,
		"requirements": sorted(requirements),
		"licenses": sorted(NATIVE_NOTICE_ROLES),
		"additional_notice_roles": sorted(notices.difference(NATIVE_NOTICE_ROLES)),
		"native_rdkit": "intentional_package_relative_closure",
	}
	return result


#============================================
def inspect_qt_wheel(path: pathlib.Path) -> dict:
	"""Classify Ferrum-Qt package, dependency, resource, and notice roles.

	Args:
		path: Final Ferrum-Qt wheel.

	Returns:
		Small machine-readable classification record.

	Raises:
		InventoryError: When required Qt wheel roles are absent or forbidden.
	"""
	members, contents = wheel_members(path, "Qt wheel")
	metadata_member = member_with_suffix(members, ".dist-info/METADATA", "Qt METADATA")
	metadata = parse_metadata(contents, metadata_member, "Qt")
	version = require_distribution(metadata, "ferrum-qt", "Qt")
	require_license(metadata, "agpl-3.0", "Qt")
	requirements = inspect_dependency_boundary(metadata, "Qt")
	if "ferrum-chem" not in requirements:
		raise InventoryError("dependency phase Qt wheel does not declare Ferrum-Chem")
	inspect_importable_boundary(members, "Qt")
	entry_member = member_with_suffix(members, ".dist-info/entry_points.txt", "Qt entry points")
	try:
		entry_text = contents[entry_member].decode("utf-8")
	except UnicodeDecodeError as error:
		raise InventoryError("runtime boundary phase found non-UTF-8 Qt entry points") from error
	if not re.search(r"(?m)^ferrum-qt\s*=\s*ferrum_qt\.cli:main\s*$", entry_text):
		raise InventoryError("runtime boundary phase Qt wheel lacks ferrum-qt console entry point")
	required_resources = {"ferrum_qt/resources/app_icon.svg", "ferrum_qt/resources/themes/light.yaml"}
	missing_resources = required_resources.difference(contents)
	if missing_resources:
		raise InventoryError(
			"runtime boundary phase Qt wheel lacks package resource(s): "
			+ ", ".join(sorted(missing_resources))
		)
	notice_prefix = metadata_member.rsplit("/", 1)[0] + "/licenses/"
	if not any(
			member.startswith(notice_prefix) and member.endswith("/LICENSE") for member in members
		):
		raise InventoryError("notice phase Qt wheel lacks its distributable AGPL license")
	result = {
		"distribution": "ferrum-qt",
		"version": version,
		"requirements": sorted(requirements),
		"license": "AGPL-3.0-only",
		"entry_point": "ferrum-qt",
	}
	return result


#============================================
def archive_members(path: pathlib.Path) -> set[str]:
	"""List regular source archive members without extracting untrusted paths.

	Args:
		path: Source zip or gzip-compressed tar archive.

	Returns:
		Normalized member paths with their one archive-root prefix removed.

	Raises:
		InventoryError: When the archive is unreadable or lacks one shared root.
	"""
	try:
		if path.name.endswith(".zip"):
			with zipfile.ZipFile(path) as archive:
				members = [info.filename for info in archive.infolist() if not info.is_dir()]
		else:
			with tarfile.open(path, "r:*") as archive:
				members = [info.name for info in archive.getmembers() if info.isfile()]
	except (OSError, tarfile.TarError, zipfile.BadZipFile) as error:
		raise InventoryError(f"source archive phase cannot read archive: {path}") from error
	if not members:
		raise InventoryError("source archive phase found no regular members")
	first_parts = {pathlib.PurePosixPath(member).parts[0] for member in members}
	if len(first_parts) != 1:
		raise InventoryError("source archive phase needs one release-root directory")
	root = next(iter(first_parts))
	result = set()
	for member in members:
		parts = pathlib.PurePosixPath(member).parts
		if not parts or parts[0] != root or ".." in parts:
			raise InventoryError(f"source archive phase found unsafe member path: {member}")
		if len(parts) > 1:
			result.add("/".join(parts[1:]))
	return result


#============================================
def archive_member_bytes(path: pathlib.Path, expected: str) -> bytes:
	"""Read one source member role after verifying its release-root position.

	Args:
		path: Source zip or gzip-compressed tar archive.
		expected: Repo-relative member role.

	Returns:
		Raw selected member bytes.

	Raises:
		InventoryError: When the role is absent or ambiguous in the source archive.
	"""
	try:
		if path.name.endswith(".zip"):
			with zipfile.ZipFile(path) as archive:
				candidates = [
					info for info in archive.infolist() if info.filename.endswith("/" + expected)
				]
				if len(candidates) != 1:
					raise InventoryError(
						f"source archive phase has ambiguous or absent {expected} role"
					)
				return archive.read(candidates[0])
		with tarfile.open(path, "r:*") as archive:
			candidates = [
				info for info in archive.getmembers()
				if info.isfile() and info.name.endswith("/" + expected)
			]
			if len(candidates) != 1:
				raise InventoryError(f"source archive phase has ambiguous or absent {expected} role")
			member = archive.extractfile(candidates[0])
			if member is None:
				raise InventoryError(f"source archive phase cannot read regular {expected} role")
			contents = member.read()
	except (OSError, tarfile.TarError, zipfile.BadZipFile) as error:
		raise InventoryError(f"source archive phase cannot read member {expected}") from error
	return contents


#============================================
def source_version(path: pathlib.Path, member: str, pattern: str) -> str:
	"""Read one declared source-release version using its owned manifest role.

	Args:
		path: Source archive path.
		member: Repo-relative manifest role.
		pattern: Regex with one version capture group.

	Returns:
		Declared manifest version.

	Raises:
		InventoryError: When the required source version is unreadable or absent.
	"""
	try:
		text = archive_member_bytes(path, member).decode("utf-8")
	except UnicodeDecodeError as error:
		raise InventoryError(f"identity phase found non-UTF-8 source {member}") from error
	match = re.search(pattern, text, flags=re.MULTILINE)
	if match is None:
		raise InventoryError(f"identity phase cannot find declared version in source {member}")
	version = match.group(1)
	normalized_version(version)
	return version


#============================================
def inspect_source_archive(path: pathlib.Path) -> dict:
	"""Classify source-release legal and provenance roles.

	Args:
		path: Final committed source archive.

	Returns:
		Small machine-readable classification record.

	Raises:
		InventoryError: When the source route lacks required legal/provenance roles.
	"""
	members = archive_members(path)
	required = {
		"LICENSE.AGPL-3.0.md",
		"LICENSE.LGPL-3.0.md",
		"VERSION",
		"Cargo.toml",
		"crates/api/python/pyproject.toml",
		"packages/ferrum-chem-qt.app/pyproject.toml",
		"packages/ferrum-chem-qt.app/LICENSE",
		"crates/render/assets/licenses/Telex-OFL-1.1.txt",
		"docs/PROVENANCE.md",
	}
	missing = required.difference(members)
	if missing:
		raise InventoryError(
			"source archive phase lacks required license/provenance role(s): "
			+ ", ".join(sorted(missing))
		)
	versions = {
		"VERSION": source_version(path, "VERSION", r"^\s*([0-9]+(?:\.[0-9]+)*)\s*$"),
		"Cargo.toml": source_version(
			path, "Cargo.toml", r"^version\s*=\s*\"([0-9]+(?:\.[0-9]+)*)\"\s*$"
		),
		"ferrum-chem": source_version(
			path,
			"crates/api/python/pyproject.toml",
			r"^version\s*=\s*\"([0-9]+(?:\.[0-9]+)*)\"\s*$",
		),
		"ferrum-qt": source_version(
			path,
			"packages/ferrum-chem-qt.app/pyproject.toml",
			r"^version\s*=\s*\"([0-9]+(?:\.[0-9]+)*)\"\s*$",
		),
	}
	version_keys = {normalized_version(value) for value in versions.values()}
	if len(version_keys) != 1:
		raise InventoryError("identity phase source release versions are not packaging-equivalent")
	result = {
		"licenses": ["AGPL-3.0-only", "LGPL-3.0-only"],
		"provenance": "present",
		"version": versions["VERSION"],
	}
	return result


#============================================
def require_receipt_identity(
		receipt: dict, chemistry: dict, qt: dict, source: dict, paths: dict[str, pathlib.Path],
		) -> dict:
	"""Require M20's completed receipt to identify these release-role artifacts.

	Args:
		receipt: Decoded M20 package receipt.
		chemistry: Native wheel classification.
		qt: Qt wheel classification.
		source: Source archive classification.
		paths: Final wheel paths keyed by normalized distribution name.

	Returns:
		Receipt identity classification.

	Raises:
		InventoryError: When receipt schema, target, semantic artifact identity, or proof fails.
	"""
	if receipt.get("schema") != RECEIPT_SCHEMA:
		raise InventoryError("receipt identity phase needs ferrum-release-package-receipt-v1")
	if receipt.get("target") != TARGET:
		raise InventoryError("receipt identity phase needs macOS arm64 CPython 3.12 M20 evidence")
	artifacts = receipt.get("artifacts")
	source_versions = receipt.get("source_versions")
	validation = receipt.get("validation")
	if not isinstance(artifacts, dict) or not isinstance(source_versions, dict):
		raise InventoryError("receipt identity phase lacks selected artifact or source-version records")
	if not isinstance(validation, dict) or validation.get("outcome") != "success":
		raise InventoryError("receipt identity phase lacks successful M20 validation")
	if not isinstance(validation.get("observations"), dict):
		raise InventoryError("receipt identity phase lacks M20 installed-behavior observations")
	for name, observed in (("ferrum-chem", chemistry), ("ferrum-qt", qt)):
		recorded = artifacts.get(name)
		if not isinstance(recorded, dict):
			raise InventoryError(f"receipt identity phase lacks {name} artifact record")
		if recorded.get("filename") != paths[name].name:
			raise InventoryError(f"receipt identity phase names a different {name} wheel")
		recorded_version = recorded.get("version")
		source_version = source_versions.get(name)
		if not isinstance(recorded_version, str) or not isinstance(source_version, str):
			raise InventoryError(f"receipt identity phase lacks {name} version identity")
		if normalized_version(recorded_version) != normalized_version(observed["version"]):
			raise InventoryError(f"receipt identity phase version differs for {name}")
		if normalized_version(source_version) != normalized_version(observed["version"]):
			raise InventoryError(f"receipt identity phase source version differs for {name}")
		if normalized_version(source["version"]) != normalized_version(observed["version"]):
			raise InventoryError(f"receipt identity phase archive version differs for {name}")
	result = {"schema": RECEIPT_SCHEMA, "target": TARGET, "validation": "success"}
	return result


#============================================
def parse_args() -> argparse.Namespace:
	"""Parse the four explicit final release artifacts.

	Returns:
		Parsed command arguments.
	"""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument(
		"--chem-wheel", required=True, type=lambda value: require_file(value, "chem wheel")
	)
	parser.add_argument(
		"--qt-wheel", required=True, type=lambda value: require_file(value, "Qt wheel")
	)
	parser.add_argument(
		"--source-archive", required=True, type=lambda value: require_file(value, "source archive")
	)
	parser.add_argument(
		"--receipt", required=True, type=lambda value: require_file(value, "M20 receipt")
	)
	args = parser.parse_args()
	return args


#============================================
def main() -> int:
	"""Classify final artifacts and publish JSON only after every predicate passes.

	Returns:
		Zero after a successful M22 inventory, otherwise one after a categorized failure.
	"""
	try:
		args = parse_args()
		chemistry = inspect_native_wheel(args.chem_wheel)
		qt = inspect_qt_wheel(args.qt_wheel)
		source = inspect_source_archive(args.source_archive)
		receipt = read_json_object(args.receipt, "receipt identity")
		identity = require_receipt_identity(
			receipt,
			chemistry,
			qt,
			source,
			{"ferrum-chem": args.chem_wheel, "ferrum-qt": args.qt_wheel},
		)
		result = {
			"schema": RESULT_SCHEMA,
			"target": TARGET,
			"artifacts": {"ferrum-chem": chemistry, "ferrum-qt": qt},
			"source_archive": source,
			"receipt": identity,
			"human_legal_review": "required_before_publication",
		}
		print(json.dumps(result, sort_keys=True))
		return 0
	except InventoryError as error:
		print(f"artifact inventory error: {error}", file=sys.stderr)
		return 1


if __name__ == "__main__":
	raise SystemExit(main())
