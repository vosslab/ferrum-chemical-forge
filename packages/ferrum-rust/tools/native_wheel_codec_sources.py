"""Materialize sealed codec sources into an RDKit source tree."""

from __future__ import annotations

# Standard library imports.
import shutil
from pathlib import Path


def stage_codec_sources(source: Path, inchi: Path, coordgen: Path) -> None:
	"""Stage local codec inputs and remove CoordGen's unused MAE fetch path."""
	for input_source, destination in (
		(inchi, source / "External" / "INCHI-API" / "src"),
		(coordgen, source / "External" / "CoordGen" / "coordgen"),
	):
		if not input_source.is_dir() or destination.exists():
			raise ValueError(f"codec source staging needs an empty destination: {destination}")
		destination.parent.mkdir(parents=True, exist_ok=True)
		shutil.copytree(input_source, destination)
	coordgen_cmake = source / "External" / "CoordGen" / "CMakeLists.txt"
	contents = coordgen_cmake.read_text(encoding="utf-8")
	needle = "if(RDK_BUILD_MAEPARSER_SUPPORT OR RDK_BUILD_COORDGEN_SUPPORT)"
	if contents.count(needle) != 1:
		raise ValueError("RDKit CoordGen source no longer has the expected MAE conditional")
	coordgen_cmake.write_text(contents.replace(needle, "if(RDK_BUILD_MAEPARSER_SUPPORT)"), encoding="utf-8")
	inchi_cmake = source / "External" / "INCHI-API" / "CMakeLists.txt"
	contents = inchi_cmake.read_text(encoding="utf-8")
	needle = "rdkit_library(RDInchiLib inchi.cpp SHARED LINK_LIBRARIES ${INCHI_LIBRARIES}"
	if contents.count(needle) != 1:
		raise ValueError("RDKit InChI source no longer has the expected native link declaration")
	inchi_cmake.write_text(contents.replace(needle, "rdkit_library(RDInchiLib inchi.cpp SHARED LINK_LIBRARIES Inchi"), encoding="utf-8")
