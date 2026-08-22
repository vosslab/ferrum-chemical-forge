"""Verify the receipt for Ferrum's build-local runtime."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from engine_lib.local_runtime_receipt import (
	LocalRuntimeReceiptError,
	local_extension_path,
	validate_local_runtime_receipt,
	write_local_runtime_receipt,
)


#============================================
def main() -> int:
	"""Expose local-runtime receipt operations without building or installing."""
	parser = argparse.ArgumentParser(description="Manage a Ferrum local runtime receipt.")
	parser.add_argument("command", choices=("extension-path", "write", "validate"))
	parser.add_argument("--runtime-root", required=True, type=Path)
	arguments = parser.parse_args()
	try:
		if arguments.command == "extension-path":
			print(local_extension_path(arguments.runtime_root))
		elif arguments.command == "write":
			write_local_runtime_receipt(arguments.runtime_root)
		else:
			validate_local_runtime_receipt(arguments.runtime_root)
	except LocalRuntimeReceiptError as error:
		print(f"local runtime receipt error: {error}", file=sys.stderr)
		return 1
	return 0


if __name__ == "__main__":
	raise SystemExit(main())
