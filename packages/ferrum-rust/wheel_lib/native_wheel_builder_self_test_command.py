"""Self-test command handler for the native-wheel builder executable."""

from __future__ import annotations

import argparse

import wheel_lib.native_wheel_builder_self_test as native_wheel_builder_self_test


#============================================
def command_self_test(_: argparse.Namespace) -> None:
	"""Run every deterministic native-wheel helper fixture without a native build."""
	native_wheel_builder_self_test.run()
	print("native wheel pure helper checks passed")
