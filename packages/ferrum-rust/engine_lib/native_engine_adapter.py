"""RDKit native build, adapter configuration, and subprocess execution."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from engine_lib.native_engine_macho import otool_dependencies
from engine_lib.native_engine_policy import NativePolicyError, apple_sdk, audit_cmake_provenance, cmake_cxx_toolchain_options, cmake_toolchain_options, homebrew_cmake, homebrew_llvm, native_tool_environment, toolchain_receipt
from engine_lib.native_engine_profile import validate_rdkit_configuration as validate_profile_configuration, validate_resolved_rdkit_configuration as validate_profile_cache
from engine_lib.native_engine_model import ADAPTER_BUILD_TYPES, NativeBuildError, NATIVE_SOURCE, RdkitLayout, rdkit_layout_from_output_root
from engine_lib.native_engine_sources import install_pinned_inchi_source, materialize_boost_headers_config, materialize_retained_rdkit_inputs, minimal_rdkit_options, prepare_source, publish_native_input_manifest, stage_rdkit_inputs


def run(*command: str, cwd: Path | None = None, env: dict[str, str] | None = None) -> None:
	print("+", " ".join(command), file=sys.stderr)
	try:
		# Reserve stdout for one final machine result; route build progress to stderr.
		subprocess.run(command, cwd=cwd, env=env, stdout=sys.stderr, check=True)
	except FileNotFoundError as error:
		raise NativeBuildError(f"required program is unavailable: {command[0]}") from error
	except subprocess.CalledProcessError as error:
		raise NativeBuildError(f"command failed ({error.returncode}): {' '.join(command)}") from error
def build_rdkit(output_root: Path, archive_root: Path | None) -> RdkitLayout:
	source = prepare_source(output_root, archive_root)
	catch2_source, better_enums_source, boost_headers, inchi_source = materialize_retained_rdkit_inputs(
		output_root, archive_root
	)
	install_pinned_inchi_source(source, inchi_source)
	boost_config = materialize_boost_headers_config(output_root, boost_headers)
	build = output_root / "rdkit-build"
	install = output_root / "rdkit-install"
	if build.exists() or install.exists():
		if not (build.is_dir() and install.is_dir()):
			raise NativeBuildError("refusing to overwrite an incomplete RDKit build; choose a fresh output root")
		try:
			llvm_root = homebrew_llvm()
			cmake = homebrew_cmake()
			sdk_root = apple_sdk()
			validate_resolved_rdkit_configuration(build)
			provenance_audit = audit_cmake_provenance(build, output_root, llvm_root, cmake, sdk_root)
		except (NativePolicyError, ValueError) as error:
			raise NativeBuildError(str(error)) from error
		if not (output_root / "ferrum-native-inputs.json").is_file():
			stage_rdkit_inputs(output_root, source, build)
			publish_native_input_manifest(output_root)
		layout = rdkit_layout_from_output_root(output_root)
		return RdkitLayout(
			input_root=layout.input_root,
			lib_dir=layout.lib_dir,
			include_dir=layout.include_dir,
			boost_include_dir=layout.boost_include_dir,
			graphmol_library=layout.graphmol_library,
			rdgeneral_library=layout.rdgeneral_library,
			depictor_library=layout.depictor_library,
			smilesparse_library=layout.smilesparse_library,
			fileparsers_library=layout.fileparsers_library,
			rdinchi_library=layout.rdinchi_library,
			substructmatch_library=layout.substructmatch_library,
			cmake_options=tuple(minimal_rdkit_options(catch2_source, better_enums_source, boost_config)),
			toolchain=toolchain_receipt(llvm_root, cmake, sdk_root),
			provenance_audit=provenance_audit,
		)
	options = minimal_rdkit_options(catch2_source, better_enums_source, boost_config)
	options.append(f"-DCMAKE_INSTALL_PREFIX={install}")
	validate_rdkit_configuration(options)
	try:
		llvm_root = homebrew_llvm()
		cmake = homebrew_cmake()
		sdk_root = apple_sdk()
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	options.extend(cmake_toolchain_options(llvm_root, sdk_root))
	run(
		str(cmake), "-S", str(source), "-B", str(build),
		*options,
		env=native_tool_environment(llvm_root, cmake),
	)
	validate_resolved_rdkit_configuration(build)
	try:
		provenance_audit = audit_cmake_provenance(build, output_root, llvm_root, cmake, sdk_root)
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	run(
		str(cmake), "--build", str(build), "--target", "FileParsers", "RDInchiLib", "--parallel",
		env=native_tool_environment(llvm_root, cmake),
	)
	stage_rdkit_inputs(output_root, source, build)
	publish_native_input_manifest(output_root)
	layout = rdkit_layout_from_output_root(output_root)
	return RdkitLayout(
		input_root=layout.input_root,
		lib_dir=layout.lib_dir,
		include_dir=layout.include_dir,
		boost_include_dir=layout.boost_include_dir,
		graphmol_library=layout.graphmol_library,
		rdgeneral_library=layout.rdgeneral_library,
		depictor_library=layout.depictor_library,
		smilesparse_library=layout.smilesparse_library,
		fileparsers_library=layout.fileparsers_library,
		rdinchi_library=layout.rdinchi_library,
		substructmatch_library=layout.substructmatch_library,
		cmake_options=tuple(options),
		toolchain=toolchain_receipt(llvm_root, cmake, sdk_root),
		provenance_audit=provenance_audit,
	)

#============================================
def validate_rdkit_configuration(options: list[str]) -> None:
	"""Map command-policy validation into the builder's error contract."""
	try:
		validate_profile_configuration(options)
	except ValueError as error:
		raise NativeBuildError(str(error)) from error


#============================================
def validate_resolved_rdkit_configuration(build: Path) -> None:
	"""Map configured-cache validation into the builder's error contract."""
	try:
		validate_profile_cache(build)
	except ValueError as error:
		raise NativeBuildError(str(error)) from error

#============================================
def configure_adapter(
	output_root: Path,
	layout: RdkitLayout,
	build_type: str = "Release",
) -> Path:
	"""Build one real adapter against the declared private RDKit installation."""
	if build_type not in ADAPTER_BUILD_TYPES:
		raise NativeBuildError(f"unsupported adapter build type: {build_type}")
	build = output_root / "adapter-build"
	install = output_root / "adapter-install"
	if build.exists() or install.exists():
		raise NativeBuildError("refusing to overwrite existing adapter build output")
	try:
		llvm_root = homebrew_llvm()
		cmake = homebrew_cmake()
		sdk_root = apple_sdk()
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	command = [
		str(cmake), "-S", str(NATIVE_SOURCE), "-B", str(build),
		f"-DCMAKE_BUILD_TYPE={build_type}",
		f"-DCMAKE_INSTALL_PREFIX={install}",
		f"-DFERRUM_CHEM_RDKIT_INCLUDE_DIR={layout.include_dir}",
		f"-DFERRUM_CHEM_BOOST_INCLUDE_DIR={layout.boost_include_dir}",
		f"-DFERRUM_CHEM_RDKIT_GRAPHMOL={layout.graphmol_library}",
		f"-DFERRUM_CHEM_RDKIT_RDGENERAL={layout.rdgeneral_library}",
		f"-DFERRUM_CHEM_RDKIT_DEPICTOR={layout.depictor_library}",
		f"-DFERRUM_CHEM_RDKIT_SMILESPARSE={layout.smilesparse_library}",
		f"-DFERRUM_CHEM_RDKIT_FILEPARSERS={layout.fileparsers_library}",
		f"-DFERRUM_CHEM_RDKIT_RDINCHI={layout.rdinchi_library}",
		f"-DFERRUM_CHEM_RDKIT_SUBSTRUCTMATCH={layout.substructmatch_library}",
	]
	command.extend(cmake_cxx_toolchain_options(llvm_root, sdk_root))
	run(*command, env=native_tool_environment(llvm_root, cmake))
	try:
		audit_cmake_provenance(
			build, output_root, llvm_root, cmake, sdk_root,
			source_roots=(NATIVE_SOURCE, layout.input_root),
		)
	except NativePolicyError as error:
		raise NativeBuildError(str(error)) from error
	run(str(cmake), "--build", str(build), "--parallel", env=native_tool_environment(llvm_root, cmake))
	run(str(cmake), "--install", str(build), env=native_tool_environment(llvm_root, cmake))
	adapter = install / "lib" / "libferrum_chem.dylib"
	if not adapter.is_file():
		raise NativeBuildError(f"adapter build did not produce {adapter}")
	linked_names = {Path(item).name for item in otool_dependencies(adapter)}
	for library in (layout.graphmol_library, layout.rdgeneral_library,
			layout.depictor_library, layout.smilesparse_library, layout.fileparsers_library,
			layout.rdinchi_library, layout.substructmatch_library):
		if library.name not in linked_names:
			raise NativeBuildError(
				"adapter did not retain its declared RDKit loader dependency; "
				f"missing {library.name} from {sorted(linked_names)}"
			)
	return adapter
