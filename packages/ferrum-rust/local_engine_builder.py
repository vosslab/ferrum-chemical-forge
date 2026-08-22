#!/usr/bin/env python3
"""Create Ferrum's build-local native engine runtime without publishing it."""

from __future__ import annotations

import argparse
import sys

from engine_lib.local_runtime import emit_runtime_result, runtime_root_path
from engine_lib.native_engine_model import NativeBuildError
from engine_lib.native_engine_macho import NativeMachoError


#============================================
def main() -> int:
	"""Build exactly one fresh repository-local native runtime."""
	parser = argparse.ArgumentParser(
		description="Build Ferrum's local native engine runtime below build/."
	)
	parser.add_argument("--runtime-root", required=True, type=runtime_root_path)
	arguments = parser.parse_args()
	try:
		emit_runtime_result(arguments.runtime_root)
		return 0
	except (NativeBuildError, NativeMachoError, ValueError) as error:
		print(f"local native-engine build error: {error}", file=sys.stderr)
		return 1


if __name__ == "__main__":
	raise SystemExit(main())
