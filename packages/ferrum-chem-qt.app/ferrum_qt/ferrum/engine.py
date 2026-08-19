"""Single import boundary between Ferrum and the compiled Ferrum engine."""

# Standard Library
import importlib
import types


#============================================
def extension_module() -> types.ModuleType:
	"""Return the loaded extension for feature-owned DTO conversion and calls."""
	return importlib.import_module("ferrum_chem")


#============================================
def lower_round_bracket_presentation_path_v1(root: object) -> object:
	"""Return frozen Rust-issued replay commands for one round bracket root."""
	return extension_module().lower_round_bracket_presentation_path_v1(root)


#============================================
def __getattr__(name: str) -> object:
	"""Expose one extension symbol through the centralized engine boundary."""
	value = getattr(extension_module(), name)
	return value


#============================================
def __dir__() -> list[str]:
	"""List the extension surface for interactive inspection and diagnostics."""
	names = sorted(set(globals()) | set(dir(extension_module())))
	return names
