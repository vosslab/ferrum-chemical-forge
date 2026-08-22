"""Read the public Ferrum chemistry adapter ABI authority."""

from __future__ import annotations

# Standard library imports.
import re
from pathlib import Path


ADAPTER_ABI_PATTERN = re.compile(
	r"^\s*#define\s+FERRUM_CHEM_ADAPTER_ABI_VERSION\s+([1-9][0-9]*)U\s*$",
	flags=re.MULTILINE,
)


def adapter_abi_version_from_header(header_path: Path) -> int:
	"""Return the sole positive ABI macro declared by the public C header."""
	try:
		header = header_path.read_text(encoding="utf-8")
	except OSError as error:
		raise RuntimeError(f"cannot read Ferrum-Chem ABI header: {header_path}") from error
	versions = ADAPTER_ABI_PATTERN.findall(header)
	if len(versions) != 1:
		raise RuntimeError(
			"Ferrum-Chem ABI header must define exactly one positive "
			"FERRUM_CHEM_ADAPTER_ABI_VERSION macro"
		)
	return int(versions[0])
