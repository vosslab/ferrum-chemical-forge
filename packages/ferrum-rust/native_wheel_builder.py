#!/usr/bin/env python3
"""Execute one source-verified Ferrum native wheel operation."""

from __future__ import annotations

import sys

import wheel_lib.native_wheel_builder_cli as native_wheel_builder_cli
from wheel_lib.native_wheel_macho import NativeMachoError
from wheel_lib.native_wheel_packaging import NativePackagingError
from wheel_lib.native_wheel_builder_commands import command_adapter, command_build, command_parse_artifact_result, command_publish_publication, command_record_qt_worktree_source_closure, command_stage_qt_source_tree, command_validate_publication
from wheel_lib.native_wheel_builder_model import NativeBuildError, archive_root_path, engine_bundle_path, output_path
from wheel_lib.native_wheel_builder_self_test_command import command_self_test


#============================================
def main() -> int:
	"""Parse one command and return its process exit status."""
	try:
		arguments = native_wheel_builder_cli.parser(
			command_build,
			command_adapter,
			command_self_test,
			command_validate_publication,
			command_publish_publication,
			command_parse_artifact_result,
			command_record_qt_worktree_source_closure,
			command_stage_qt_source_tree,
			output_path,
			engine_bundle_path,
			archive_root_path,
		).parse_args()
		arguments.handler(arguments)
		return 0
	except (NativeBuildError, NativeMachoError, NativePackagingError) as error:
		print(f"initial native-wheel build error: {error}", file=sys.stderr)
		return 1


if __name__ == "__main__":
	raise SystemExit(main())
