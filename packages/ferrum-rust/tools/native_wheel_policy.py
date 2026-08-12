"""Host-independent policy helpers for Ferrum's native-wheel build.

This module owns the native toolchain environment and CMake provenance gate so
the wheel orchestrator remains focused on source materialization and packaging.
"""

from __future__ import annotations

# Standard library
import os
import re
import subprocess
from collections.abc import Mapping
from pathlib import Path
from tempfile import TemporaryDirectory


# ============================================================================
# Exceptions

class NativePolicyError(RuntimeError):
	"""A native build selected content outside Ferrum's declared boundary."""


# CMake emits both paths and language syntax in its generated files.  In
# particular, an expression ending ``/)`` leaves a standalone ``/`` token.
# The filesystem root is a separator, not a dependency location, so require a
# component after the leading slash and discard candidates that normalize to
# the root before handing them to the provenance audit.
_PATH_TOKEN = re.compile(r"/(?:[^\s\"';()]+)")
_REMOVED_ENVIRONMENT = (
	"CMAKE_PREFIX_PATH", "CMAKE_FRAMEWORK_PATH", "CMAKE_INCLUDE_PATH",
	"CMAKE_LIBRARY_PATH", "CMAKE_PROGRAM_PATH", "CMAKE_MODULE_PATH",
	"PKG_CONFIG_PATH", "PKG_CONFIG_LIBDIR", "CPATH", "C_INCLUDE_PATH",
	"CPLUS_INCLUDE_PATH", "LIBRARY_PATH", "DYLD_LIBRARY_PATH",
	"DYLD_FALLBACK_LIBRARY_PATH", "PYTHONPATH", "PYTHONHOME",
	"CC", "CXX", "CPP", "CFLAGS", "CXXFLAGS", "CPPFLAGS", "LDFLAGS",
	"AR", "AS", "LD", "NM", "RANLIB", "STRIP", "SDKROOT",
	"MACOSX_DEPLOYMENT_TARGET", "ARCHFLAGS", "CMAKE_ARGS", "CMAKE_GENERATOR",
)
_REMOVED_ENVIRONMENT_PREFIXES = (
	"CMAKE_",
	"RDBASE", "RDKIT", "RDK_", "BOOST_", "Boost_", "Inchi_", "INCHI_",
	"coordgen", "COORDGEN_", "maeparser", "MAEPARSER_",
)
_SYSTEM_TOOL_DIRECTORIES = (
	Path("/usr/bin"), Path("/bin"), Path("/usr/sbin"), Path("/sbin"),
)


# ============================================================================
# Tool discovery and environment construction

def command_output(*command: str) -> str:
	result = subprocess.run(command, text=True, capture_output=True, check=False)
	if result.returncode:
		raise NativePolicyError(
			f"required tool failed: {' '.join(command)}: {result.stderr.strip()}"
		)
	return result.stdout.strip()


def homebrew_llvm() -> Path:
	path = Path(command_output("brew", "--prefix", "llvm")).resolve()
	if not (path / "bin/clang").is_file() or not (path / "bin/clang++").is_file():
		raise NativePolicyError(f"Homebrew LLVM is incomplete at {path}; install brew 'llvm'")
	return path


def homebrew_cmake() -> Path:
	"""Resolve the one CMake executable declared by Ferrum's toolchain."""
	path = (Path(command_output("brew", "--prefix", "cmake")) / "bin/cmake").resolve()
	if not path.is_file():
		raise NativePolicyError(f"Homebrew CMake is incomplete at {path}; install brew 'cmake'")
	return path


def homebrew_rustup() -> Path:
	"""Resolve the Rust toolchain launcher declared by Ferrum's Brewfile."""
	path = (Path(command_output("brew", "--prefix", "rustup")) / "bin").resolve()
	if not (path / "cargo").is_file() or not (path / "rustc").is_file():
		raise NativePolicyError(f"Homebrew Rustup is incomplete at {path}; install brew 'rustup'")
	return path


def cmake_installation_root(cmake: Path) -> Path:
	"""Return the resolved, versioned CMake installation containing ``cmake``."""
	root = cmake.resolve().parent.parent
	if not (root / "share").is_dir():
		raise NativePolicyError(f"CMake installation lacks support files below {root}")
	return root


def apple_sdk() -> Path:
	path = Path(command_output("xcrun", "--sdk", "macosx", "--show-sdk-path")).resolve()
	if not path.is_dir():
		raise NativePolicyError(f"xcrun returned a missing macOS SDK: {path}")
	return path


def controlled_environment(baseline: Mapping[str, str] | None = None) -> dict[str, str]:
	"""Remove ambient native-build injection from native builds.

	Every inherited ``CMAKE_*`` value is untrusted configuration input.  The
	builder supplies the few deliberate CMake settings as command-line options,
	so this prevents both known and future CMake environment controls from
	changing a Ferrum native build.
	"""
	environment = dict(os.environ if baseline is None else baseline)
	for name in _REMOVED_ENVIRONMENT:
		environment.pop(name, None)
	for name in tuple(environment):
		if name.startswith(_REMOVED_ENVIRONMENT_PREFIXES):
			environment.pop(name, None)
	return environment


def declared_tool_environment(*tool_directories: Path) -> dict[str, str]:
	"""Return a scrubbed environment with only declared tool directories on PATH.

	CMake records the programs it discovers.  An inherited PATH therefore turns
	otherwise invisible shell preferences into native-build inputs.  This helper
	keeps program discovery reproducible without allowing all of Homebrew.
	"""
	environment = controlled_environment()
	directories = (*tool_directories, *_SYSTEM_TOOL_DIRECTORIES)
	resolved: list[Path] = []
	for directory in directories:
		path = directory.resolve()
		if not path.is_dir():
			raise NativePolicyError(f"declared tool directory is unavailable: {path}")
		if path not in resolved:
			resolved.append(path)
	environment["PATH"] = os.pathsep.join(str(path) for path in resolved)
	return environment


def native_tool_environment(llvm_root: Path, cmake: Path) -> dict[str, str]:
	"""Constrain CMake's native program discovery to Ferrum's declared tools."""
	return declared_tool_environment(llvm_root / "bin", cmake.resolve().parent)


def rust_tool_environment(llvm_root: Path) -> dict[str, str]:
	"""Constrain Maturin/Cargo lookup without inheriting shell-installed tools."""
	return declared_tool_environment(homebrew_rustup(), llvm_root / "bin")


def rust_toolchain_receipt() -> dict[str, str]:
	"""Record the declared Rust launcher and resolved compiler tools."""
	rustup_bin = homebrew_rustup()
	cargo = rustup_bin / "cargo"
	rustc = rustup_bin / "rustc"
	return {
		"rustup_bin": str(rustup_bin),
		"cargo": str(cargo),
		"cargo_version": command_output(str(cargo), "--version"),
		"rustc": str(rustc),
		"rustc_version": command_output(str(rustc), "--version"),
	}


def cmake_toolchain_options(llvm_root: Path, sdk_root: Path) -> list[str]:
	return [
		f"-DCMAKE_C_COMPILER={llvm_root / 'bin/clang'}",
		f"-DCMAKE_CXX_COMPILER={llvm_root / 'bin/clang++'}",
		f"-DCMAKE_OSX_SYSROOT={sdk_root}",
	]


def toolchain_receipt(llvm_root: Path, cmake: Path, sdk_root: Path) -> dict[str, str]:
	"""Describe the intentional FOSS compiler plus unavoidable macOS boundary."""
	return {
		"compiler_family": "Homebrew LLVM/Clang",
		"llvm_root": str(llvm_root),
		"c_compiler": str(llvm_root / "bin/clang"),
		"cxx_compiler": str(llvm_root / "bin/clang++"),
		"cmake": str(cmake),
		"cmake_installation_root": str(cmake_installation_root(cmake)),
		"cmake_version": command_output(str(cmake), "--version").splitlines()[0],
		"sdk": str(sdk_root),
		"linker_boundary": (
			"Apple SDK and system linker are platform inputs, not packaged dependencies"
		),
	}


# ============================================================================
# CMake provenance policy

def declared_provenance_roots(
	output_root: Path,
	llvm_root: Path,
	cmake: Path,
	sdk_root: Path,
	source_roots: tuple[Path, ...] = (),
) -> tuple[Path, ...]:
	"""Return every intentionally permitted native-build path root.

	This is deliberately a short allowlist rather than a Homebrew denylist.
	The only checkout source accepted by an adapter configuration is passed as an
	exact source root; the RDKit source is already under ``output_root``.
	"""
	roots = (
		output_root, llvm_root, cmake_installation_root(cmake), sdk_root,
		Path("/usr/lib"), Path("/System/Library"), Path("/usr/bin"),
		Path("/bin"),
		Path("/Applications/Xcode.app"), Path("/Library/Developer/CommandLineTools"),
		*source_roots,
	)
	return tuple(dict.fromkeys(root.resolve() for root in roots))


def _allowed_path(path: Path, allowed_roots: tuple[Path, ...]) -> bool:
	return any(path.is_relative_to(root) for root in allowed_roots)


def _absolute_path_candidates(text: str) -> tuple[Path, ...]:
	"""Extract meaningful absolute paths from one configured CMake value."""
	candidates: list[Path] = []
	for token in _PATH_TOKEN.findall(text):
		candidate = Path(token.rstrip(".,:")).expanduser()
		if candidate == Path("/"):
			continue
		candidates.append(candidate)
	return tuple(candidates)


def _configured_path_values(path: Path, text: str) -> tuple[Path, ...]:
	"""Read CMake's configured values, never its exploratory search transcript.

	``CMakeConfigureLog.yaml`` records every directory CMake *considered* while
	searching.  Those candidates are not dependencies.  The cache and generated
	CMake files contain the selected values, while YAML ``found:`` entries record
	selected programs that are not always cached.  Auditing those concrete values
	preserves fail-closed provenance without treating macOS's fixed search list as
	an undeclared package dependency.
	"""
	values: list[str] = []
	if path.name == "CMakeCache.txt":
		values.extend(
			line.partition("=")[2]
			for line in text.splitlines()
			if "=" in line and not line.startswith(("//", "#"))
		)
	elif path.suffix == ".cmake":
		values.extend(
			line
			for line in text.splitlines()
			if line.lstrip().startswith(("set(", "include(", "file("))
		)
	elif path.name == "CMakeConfigureLog.yaml":
		values.extend(
			match.group(1)
			for match in re.finditer(
				r'^\s*found:\s*"([^"]+)"\s*$', text, re.MULTILINE,
			)
		)
	return tuple(candidate for value in values for candidate in _absolute_path_candidates(value))


def audit_cmake_provenance(
	build_root: Path,
	output_root: Path,
	llvm_root: Path,
	cmake: Path,
	sdk_root: Path,
	source_roots: tuple[Path, ...] = (),
) -> dict[str, object]:
	"""Reject retained host dependency paths from CMake's generated evidence."""
	if not build_root.is_dir():
		raise NativePolicyError(f"CMake did not create build directory: {build_root}")
	files = [
		path for path in build_root.rglob("*")
		if path.is_file()
		and (
			path.name in {"CMakeCache.txt", "CMakeConfigureLog.yaml"}
			or path.suffix == ".cmake"
		)
	]
	allowed_roots = declared_provenance_roots(
		output_root, llvm_root, cmake, sdk_root, source_roots,
	)
	findings: list[str] = []
	for path in files:
		try:
			text = path.read_text(encoding="utf-8", errors="ignore")
		except OSError as error:
			raise NativePolicyError(f"could not audit CMake evidence {path}: {error}") from error
		if "OTHER_REPOS" in text:
			findings.append(f"{path}: OTHER_REPOS")
		for candidate in _configured_path_values(path, text):
			if not candidate.is_absolute() or not candidate.exists():
				continue
			resolved = candidate.resolve()
			if resolved == Path("/"):
				# CMake punctuation can also spell the root as ``//`` or ``/.``.
				# Neither form names a dependency, so do not turn it into one.
				continue
			if not _allowed_path(resolved, allowed_roots):
				findings.append(f"{path}: undeclared host dependency {resolved}")
	if findings:
		raise NativePolicyError(
			"CMake provenance gate rejected host/reference content: "
			+ "; ".join(findings[:8])
		)
	return {
		"status": "passed",
		"audited_files": len(files),
		"allowed_roots": [str(root) for root in allowed_roots],
	}


# ============================================================================
# Deterministic policy verification

def self_test() -> None:
	"""Exercise provenance rejection without configuring or compiling native code."""
	with TemporaryDirectory() as temporary:
		root = Path(temporary) / "output-native"
		build = root / "build"
		build.mkdir(parents=True)
		cmake = root / "cmake/4.0.0/bin/cmake"
		cmake.parent.mkdir(parents=True)
		cmake.touch()
		(cmake.parent.parent / "share").mkdir(parents=True)
		output_library = root / "lib/libferrum_chem.dylib"
		output_library.parent.mkdir()
		output_library.touch()
		(build / "CMakeCache.txt").write_text(
			f"CMAKE_COMMAND:INTERNAL={cmake}\n"
			f"CMAKE_ROOT:INTERNAL={cmake.parent.parent / 'share/cmake-4.0'}\n"
			"CMAKE_SYNTAX:INTERNAL=if(DEFINED //)\n"
			"CMAKE_SYNTAX_DOT:INTERNAL=if(DEFINED /.)\n"
			"OUTPUT_LIBRARY:FILEPATH=" + str(output_library) + "\n"
			"SYSTEM_LIBRARY:FILEPATH=/usr/lib\n"
			"SYSTEM_FRAMEWORK:FILEPATH=/System/Library\n",
			encoding="utf-8",
		)
		audit_cmake_provenance(build, root, Path("/opt/homebrew/opt/llvm"), cmake, Path("/sdk"))
		(build / "CMakeCache.txt").write_text(
			"FREETYPE_LIBRARY:FILEPATH=/opt/homebrew/lib/libfreetype.dylib\n",
			encoding="utf-8",
		)
		try:
			audit_cmake_provenance(build, root, Path("/opt/homebrew/opt/llvm"), cmake, Path("/sdk"))
		except NativePolicyError:
			pass
		else:
			raise NativePolicyError("provenance self-test accepted a host FreeType cache entry")
		for label, host_path in (
			("usr-local", Path("/usr/local")),
			("home-local", Path(temporary) / "home-local"),
		):
			if label == "home-local":
				host_path.mkdir()
			(build / "CMakeCache.txt").write_text(
				f"HOST_LIBRARY:FILEPATH={host_path}\n", encoding="utf-8",
			)
			try:
				audit_cmake_provenance(build, root, Path("/opt/homebrew/opt/llvm"), cmake, Path("/sdk"))
			except NativePolicyError:
				pass
			else:
				raise NativePolicyError(f"provenance self-test accepted {label} host path")
		(build / "CMakeCache.txt").write_text(
			f"SYSTEM_LIBRARY:FILEPATH=/usr/lib\nCMAKE_COMMAND:INTERNAL={cmake}\n",
			encoding="utf-8",
		)
		audit_cmake_provenance(build, root, Path("/opt/homebrew/opt/llvm"), cmake, Path("/sdk"))
		host_path = Path(temporary) / "ambient-program"
		host_path.touch()
		(build / "CMakeConfigureLog.yaml").write_text(
			"candidate_directories:\n"
			f"  - \"{host_path}\"\n"
			f"found: \"{cmake}\"\n",
			encoding="utf-8",
		)
		audit_cmake_provenance(build, root, Path("/opt/homebrew/opt/llvm"), cmake, Path("/sdk"))
		(build / "CMakeConfigureLog.yaml").write_text(
			f"found: \"{host_path}\"\n", encoding="utf-8",
		)
		try:
			audit_cmake_provenance(build, root, Path("/opt/homebrew/opt/llvm"), cmake, Path("/sdk"))
		except NativePolicyError:
			pass
		else:
			raise NativePolicyError("provenance self-test accepted an ambient selected program")
		declared_tools = root / "declared-tools"
		declared_tools.mkdir()
		clean_path = declared_tool_environment(declared_tools)["PATH"].split(os.pathsep)
		if clean_path[0] != str(declared_tools.resolve()) or str(host_path) in clean_path:
			raise NativePolicyError("tool environment self-test retained an ambient PATH entry")
	seed = {
		"PATH": "/usr/bin", "CC": "host-cc", "CFLAGS": "-I/host", "RDBASE": "/host",
		"CMAKE_TOOLCHAIN_FILE": "/host/toolchain.cmake",
		"CMAKE_PROJECT_TOP_LEVEL_INCLUDES": "/host/inject.cmake",
		"CMAKE_UNKNOWN_INJECTION": "/host/future-injection",
		"RDKIT_ROOT": "/host", "BOOST_ROOT": "/host", "Boost_ROOT": "/host",
		"INCHI_ROOT": "/host", "Inchi_ROOT": "/host", "COORDGEN_ROOT": "/host",
		"coordgen_ROOT": "/host", "MAEPARSER_ROOT": "/host", "maeparser_ROOT": "/host",
	}
	clean = controlled_environment(seed)
	if clean != {"PATH": "/usr/bin"}:
		raise NativePolicyError(f"environment self-test retained ambient native input: {clean}")
