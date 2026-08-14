"""Mach-O closure assembly and validation for Ferrum native wheels.

This module owns every interaction with ``otool`` and ``install_name_tool``.
It is deliberately independent from the source-build policy so the builder can
remain focused on acquiring, configuring, and recording the RDKit build.
"""

from __future__ import annotations

# Standard library imports.
import collections.abc
import re
import shutil
import subprocess
import sys
from pathlib import Path

# Local native-wheel profile.
from native_wheel_profile import (
	FERRUM_RDKIT_PROFILE,
	MACOS_ARM64_NATIVE_CLOSURE,
	RdkitCapabilityProfile,
)

SYSTEM_PREFIXES = ("/usr/lib/", "/System/Library/")
DEP_LINE = re.compile(r"^\s*(\S+) \(")
RPATH_LINE = re.compile(r"^\s*path (\S+) \(offset ")


class NativeMachoError(RuntimeError):
	"""An actionable failure in the packaged Mach-O closure."""


# otool parsing and inspection -------------------------------------------------

#============================================
def parse_otool_dependencies(output: str, identity: str) -> list[str]:
	"""Separate Mach-O load dependencies from the image's own install name."""
	records: list[str] = []
	for line in output.splitlines()[1:]:
		match = DEP_LINE.match(line)
		if match:
			records.append(match.group(1))
	if not records or records[0] != identity:
		raise NativeMachoError(
			f"otool -L did not begin with declared Mach-O identity {identity}: {records}"
		)
	return records[1:]


#============================================
def otool_identity(binary: Path) -> str:
	"""Return the one dylib install name advertised by ``binary``."""
	result = subprocess.run(
		["otool", "-D", str(binary)], text=True, capture_output=True, check=False
	)
	if result.returncode:
		raise NativeMachoError(f"otool -D failed for {binary}: {result.stderr.strip()}")
	identities = [line.strip() for line in result.stdout.splitlines()[1:] if line.strip()]
	if len(identities) != 1:
		raise NativeMachoError(
			f"expected exactly one dylib identity for {binary}, found {identities}"
		)
	return identities[0]


#============================================
def otool_dependencies(binary: Path) -> list[str]:
	"""Return non-identity entries from ``otool -L``."""
	result = subprocess.run(
		["otool", "-L", str(binary)], text=True, capture_output=True, check=False
	)
	if result.returncode:
		raise NativeMachoError(f"otool -L failed for {binary}: {result.stderr.strip()}")
	return parse_otool_dependencies(result.stdout, otool_identity(binary))


#============================================
def otool_rpaths(binary: Path) -> list[str]:
	"""Return every LC_RPATH entry, preserving multiplicity and order."""
	result = subprocess.run(
		["otool", "-l", str(binary)], text=True, capture_output=True, check=False
	)
	if result.returncode:
		raise NativeMachoError(f"otool -l failed for {binary}: {result.stderr.strip()}")
	paths: list[str] = []
	lines = iter(result.stdout.splitlines())
	for line in lines:
		if line.strip() != "cmd LC_RPATH":
			continue
		for detail in lines:
			match = RPATH_LINE.match(detail)
			if match:
				paths.append(match.group(1))
				break
	return paths


#============================================
def linked_names(libraries: list[Path]) -> set[str]:
	"""Collect library basenames and their Mach-O dependency basenames."""
	return {
		*(library.name for library in libraries),
		*(
			Path(dependency).name
			for library in libraries
			for dependency in otool_dependencies(library)
		),
	}


#============================================
def detect_variants_from_names(names: set[str]) -> dict[str, str]:
	"""Reject chemistry families outside the declared codec-capability profile."""
	boost = sorted(name for name in names if name.lower().startswith("libboost_"))
	if boost:
		raise NativeMachoError(
			f"Ferrum's header-only Boost profile forbids Boost dylibs: {boost}"
		)
	forbidden = sorted(
		name for name in names
		if any(fragment in name.lower() for fragment in ("coordgen", "maeparser"))
	)
	if forbidden:
		raise NativeMachoError(
			"codec profile retained disabled chemistry libraries: " f"{forbidden}"
		)
	return {"chemistry_scope": FERRUM_RDKIT_PROFILE.name}


#============================================
def detect_variants(rdkit_libraries: list[Path]) -> dict[str, str]:
	"""Classify dependency variants found in an installed RDKit library set."""
	return detect_variants_from_names(linked_names(rdkit_libraries))


# Closure assembly and install-name rewriting ----------------------------------

#============================================
def resolved_dependencies(binary: Path, search_directories: list[Path]) -> list[Path]:
	"""Resolve all non-system load commands from an allowed library search set."""
	result: list[Path] = []
	for dependency in otool_dependencies(binary):
		if dependency.startswith(SYSTEM_PREFIXES):
			continue
		if dependency.startswith("@loader_path/"):
			candidate = binary.parent / Path(dependency).name
		elif dependency.startswith("@rpath/"):
			candidate = next(
				(
					directory / Path(dependency).name
					for directory in search_directories
					if (directory / Path(dependency).name).is_file()
				),
				None,
			)
			if candidate is None:
				raise NativeMachoError(
					f"{binary} has an unresolved @rpath dependency: {dependency}"
				)
		elif dependency.startswith("@"):
			raise NativeMachoError(
				f"{binary} uses an unsupported loader reference: {dependency}"
			)
		else:
			candidate = Path(dependency)
		if not candidate.is_file():
			raise NativeMachoError(
				f"{binary} depends on missing non-system library: {dependency}"
			)
		result.append(candidate)
	return result


#============================================
def deduplicate_paths_by_identity(
	paths: list[Path], preferred_aliases: list[Path] | None = None,
) -> list[Path]:
	"""Keep an explicitly requested install-name alias per resolved library file."""
	preferred = {path.resolve(): path for path in preferred_aliases or []}
	result: list[Path] = []
	seen: set[Path] = set()
	for path in paths:
		identity = path.resolve()
		if identity in seen:
			continue
		seen.add(identity)
		result.append(preferred.get(identity, path))
	return result


#============================================
def closure(seed: list[Path]) -> list[Path]:
	"""Return the transitive, physical-library-deduplicated native closure."""
	pending = list(seed)
	seen: set[Path] = set()
	selected: list[Path] = []
	search_directories = sorted({library.parent.resolve() for library in seed})
	while pending:
		library = pending.pop()
		identity = library.resolve()
		if identity in seen:
			continue
		seen.add(identity)
		selected.append(library)
		dependencies = resolved_dependencies(library, search_directories)
		for dependency in dependencies:
			if dependency.parent not in search_directories:
				search_directories.append(dependency.parent)
		pending.extend(dependencies)
	return sorted(deduplicate_paths_by_identity(selected, seed))


#============================================
def install_name_tool(*arguments: str) -> None:
	"""Run the macOS install-name editor with the builder's stderr protocol."""
	command = ("install_name_tool", *arguments)
	print("+", " ".join(command), file=sys.stderr)
	try:
		subprocess.run(command, stdout=sys.stderr, check=True)
	except FileNotFoundError as error:
		raise NativeMachoError("required program is unavailable: install_name_tool") from error
	except subprocess.CalledProcessError as error:
		raise NativeMachoError(
			f"install_name_tool failed ({error.returncode}): {' '.join(command)}"
		) from error


#============================================
def replace_rpaths(binary: Path, expected: str) -> None:
	"""Replace every existing LC_RPATH with one intentional loader-relative path."""
	for rpath in otool_rpaths(binary):
		install_name_tool("-delete_rpath", rpath, str(binary))
	install_name_tool("-add_rpath", expected, str(binary))


#============================================
def copy_and_rewrite_closure(
	adapter: Path,
	rdkit_library: Path | None,
	package_libs: Path,
) -> None:
	"""Stage the native closure and make every internal load command relative."""
	package_libs.mkdir(parents=True, exist_ok=True)
	seeds = [adapter]
	if rdkit_library is not None:
		seeds.append(rdkit_library)
	for library in closure(seeds):
		destination = package_libs / library.name
		if destination.exists():
			raise NativeMachoError(
				f"duplicate dependency basename in closure: {library.name}"
			)
		shutil.copy2(library, destination)
	for library in package_libs.glob("*.dylib"):
		install_name_tool("-id", f"@loader_path/{library.name}", str(library))
		replace_rpaths(library, "@loader_path")
		for dependency in otool_dependencies(library):
			if dependency.startswith(SYSTEM_PREFIXES):
				continue
			name = Path(dependency).name
			if not (package_libs / name).is_file():
				raise NativeMachoError(
					f"unbundled non-system dependency remains: {dependency}"
				)
			if dependency != f"@loader_path/{name}":
				install_name_tool(
					"-change", dependency, f"@loader_path/{name}", str(library)
				)


# Packaged-wheel closure validation --------------------------------------------

#============================================
def validate_exact_rpaths(actual: list[str], expected: list[str], label: str) -> None:
	"""Require the complete LC_RPATH sequence, including multiplicity and order."""
	if actual != expected:
		raise NativeMachoError(
			f"unexpected LC_RPATH entries for {label}: expected {expected}, got {actual}"
		)


#============================================
def validate_packaged_dylib_closure(
	identity: str,
	dependencies: list[str],
	rpaths: list[str],
	name: str,
	packaged_names: set[str],
	allowed_non_system_names: frozenset[str],
) -> None:
	"""Validate one packaged dylib's identity, rpaths, and non-system loads."""
	if name not in allowed_non_system_names:
		raise NativeMachoError(
			f"packaged dylib is outside the Ferrum platform closure allowlist: {name}"
		)
	if identity != f"@loader_path/{name}":
		raise NativeMachoError(
			f"packaged dylib has a non-packaged identity: {name} -> {identity}"
		)
	validate_exact_rpaths(rpaths, ["@loader_path"], name)
	for dependency in dependencies:
		if dependency.startswith(SYSTEM_PREFIXES):
			continue
		expected = f"@loader_path/{Path(dependency).name}"
		dependency_name = Path(dependency).name
		if dependency_name not in allowed_non_system_names:
			raise NativeMachoError(
				f"packaged dylib has forbidden native dependency: {name} -> {dependency}"
			)
		if dependency != expected or dependency_name not in packaged_names:
			raise NativeMachoError(
				"packaged dylib has an unbundled or non-loader-relative dependency: "
				f"{name} -> {dependency}"
			)


#============================================
def validate_extension_closure(
	dependencies: list[str], rpaths: list[str], has_adapter: bool,
) -> None:
	"""Validate the extension boundary and its adjacent private closure rpath."""
	validate_exact_rpaths(rpaths, ["@loader_path/.dylibs"], "native extension")
	extension_dependencies = [
		dependency for dependency in dependencies if not dependency.startswith(SYSTEM_PREFIXES)
	]
	if extension_dependencies not in ([], ["@rpath/libferrum_chem.dylib"]):
		raise NativeMachoError(
			"extension must have no native dependency or only @rpath/libferrum_chem.dylib outside macOS "
			f"system libraries; got {extension_dependencies}"
		)
	if not has_adapter:
		raise NativeMachoError("extension rpath has no packaged libferrum_chem.dylib target")


#============================================
def assert_packaged_dylib(binary: Path, package_libs: Path) -> None:
	"""Inspect one packaged dylib against the frozen platform closure."""
	validate_packaged_dylib_closure(
		otool_identity(binary),
		otool_dependencies(binary),
		otool_rpaths(binary),
		binary.name,
		{library.name for library in package_libs.glob("*.dylib")},
		MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names,
	)


#============================================
def assert_packaged_library_closure(
	package_libs: Path,
	profile: RdkitCapabilityProfile = FERRUM_RDKIT_PROFILE,
) -> None:
	"""Reject a staged dylib closure that differs from Ferrum's platform policy."""
	packaged_names = {library.name for library in package_libs.glob("*.dylib")}
	if packaged_names != MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names:
		raise NativeMachoError(
			"packaged native closure differs from frozen Ferrum platform profile: "
			f"expected {sorted(MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names)}, "
			f"got {sorted(packaged_names)}"
		)
	for name in packaged_names:
		if any(fragment in name.lower() for fragment in profile.forbidden_native_fragments):
			raise NativeMachoError(f"forbidden native library in wheel closure: {name}")
	for library in sorted(package_libs.glob("*.dylib")):
		assert_packaged_dylib(library, package_libs)


#============================================
def assert_clean_closure(
	extension: Path,
	package_libs: Path,
	profile: RdkitCapabilityProfile = FERRUM_RDKIT_PROFILE,
) -> None:
	"""Reject a wheel closure that differs from Ferrum's frozen macOS profile."""
	assert_packaged_library_closure(package_libs, profile)
	validate_extension_closure(
		otool_dependencies(extension),
		otool_rpaths(extension),
		(package_libs / "libferrum_chem.dylib").is_file(),
	)


# Pure self-tests ---------------------------------------------------------------

#============================================
def self_test() -> None:
	"""Exercise Mach-O parser, variant, and rejection rules without native files."""
	def reject(action: collections.abc.Callable[[], object], label: str) -> None:
		try:
			action()
		except NativeMachoError:
			return
		raise NativeMachoError(f"Mach-O self-test accepted {label}")

	if detect_variants_from_names({
		"libRDKitGraphMol.1.dylib", "libRDKitDepictor.1.dylib",
		"libRDKitFileParsers.1.dylib",
	}) != {
		"chemistry_scope": FERRUM_RDKIT_PROFILE.name
	}:
		raise NativeMachoError("codec-capability variant self-test failed")
	reject(
		lambda: detect_variants_from_names(
			{"libRDKitGraphMol.1.dylib", "libboost_python312.dylib"}
		),
		"Boost.Python",
	)
	validate_extension_closure(
		["@rpath/libferrum_chem.dylib", "/usr/lib/libSystem.B.dylib"],
		["@loader_path/.dylibs"],
		True,
	)
	parsed = parse_otool_dependencies(
		"native.so:\n"
		"\t@rpath/native.so (compatibility version 0.0.0, current version 0.0.0)\n"
		"\t@rpath/libferrum_chem.dylib (compatibility version 0.0.0, current version 0.0.0)\n"
		"\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n",
		"@rpath/native.so",
	)
	if parsed != ["@rpath/libferrum_chem.dylib", "/usr/lib/libSystem.B.dylib"]:
		raise NativeMachoError(f"parser retained Mach-O image identity: {parsed}")
	duplicate_identity = parse_otool_dependencies(
		"native.so:\n"
		"\t@rpath/native.so (compatibility version 0.0.0, current version 0.0.0)\n"
		"\t@rpath/libferrum_chem.dylib (compatibility version 0.0.0, current version 0.0.0)\n"
		"\t@rpath/native.so (compatibility version 0.0.0, current version 0.0.0)\n",
		"@rpath/native.so",
	)
	if duplicate_identity != ["@rpath/libferrum_chem.dylib", "@rpath/native.so"]:
		raise NativeMachoError("parser hid a later self-load command")
	reject(
		lambda: validate_extension_closure(
			["@rpath/libferrum_chem.dylib"],
			["@loader_path/../.dylibs", "@loader_path/../.dylibs"],
			True,
		),
		"duplicate LC_RPATH entries",
	)
	validate_packaged_dylib_closure(
		"@loader_path/libferrum_chem.dylib",
		["@loader_path/libRDKitRDGeneral.1.dylib", "/usr/lib/libSystem.B.dylib"],
		["@loader_path"],
		"libferrum_chem.dylib",
		{"libferrum_chem.dylib", "libRDKitRDGeneral.1.dylib"},
		MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names,
	)
	reject(lambda: detect_variants_from_names({"libRDKitcoordgen.1.dylib"}), "CoordGen")
	reject(
		lambda: validate_packaged_dylib_closure(
			"@loader_path/libboost_python312.dylib",
			[],
			["@loader_path"],
			"libboost_python312.dylib",
			{"libboost_python312.dylib"},
			MACOS_ARM64_NATIVE_CLOSURE.allowed_non_system_names,
		),
		"unallowlisted native library",
	)
	for dependencies, rpaths, has_adapter in (
		(["/opt/homebrew/lib/libferrum_chem.dylib"], ["@loader_path/../.dylibs"], True),
		(["@rpath/libferrum_chem.dylib"], ["/checkout/output"], True),
		(["@rpath/libferrum_chem.dylib"], ["@loader_path/../.dylibs"], False),
	):
		reject(
			lambda dependencies=dependencies, rpaths=rpaths, has_adapter=has_adapter:
			validate_extension_closure(dependencies, rpaths, has_adapter),
			"host or unresolved loader path",
		)
