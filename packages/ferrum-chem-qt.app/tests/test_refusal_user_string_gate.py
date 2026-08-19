"""Static guard for the author-facing refusal wiring owned by this package."""

# Standard Library
import pathlib
import re

# PIP3 modules
import ast
import pytest

# local repo modules
import ferrum_qt.ferrum.main_window


_FORBIDDEN_AUTHOR_VOCABULARY = (
	"native", "rust", "admitted", "admission", "authoritative", "typed cdml",
	"publication",
)

# These source fragments are implementation diagnostics or identifiers.  They never
# reach the ordinary status or warning surface, and remain inspectable separately.
_NON_UI_EXEMPTIONS = (
	"_NATIVE_CDML_FILTER",
	"_native_tab_for_path",
	"prepare_local_cdml_admission",
	"PublicationPossiblyCompletedError",
	"PublicationNotStartedError",
)


#============================================
def test_file_route_author_strings_use_plain_document_language() -> None:
	"""The new file route must not reintroduce implementation vocabulary to authors."""
	assert _NON_UI_EXEMPTIONS
	package = pathlib.Path(__file__).parents[1] / "ferrum_qt"
	for path in (
			package / "window_native_files.py",
			package / "ferrum" / "main_window_support.py",
		):
		for line in path.read_text(encoding="utf-8").splitlines():
			if not any(marker in line for marker in (
				"showMessage(", "_show_native_file_warning(", "self.tr(\"",
			)):
				continue
			for literal in re.findall(r'"([^"\\]*(?:\\.[^"\\]*)*)"', line):
				ordinary = literal.lower()
				assert not any(word in ordinary for word in _FORBIDDEN_AUTHOR_VOCABULARY), line


#============================================
def test_no_production_title_message_warning_bridge_remains() -> None:
	"""Every product refusal enters the typed presenter instead of a title bridge."""
	package = pathlib.Path(__file__).parents[1] / "ferrum_qt"
	legacy = tuple(package.rglob("*.py"))
	for path in legacy:
		assert "_show_native_file_warning(" not in path.read_text(encoding="utf-8")


#============================================
def test_production_refusal_calls_pass_one_explicit_request_expression() -> None:
	"""No presentation boundary can retain a `(title, message)` call shape."""
	package = pathlib.Path(__file__).parents[1] / "ferrum_qt"
	for path in package.rglob("*.py"):
		tree = ast.parse(path.read_text(encoding="utf-8"))
		for node in ast.walk(tree):
			if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Attribute):
				continue
			if node.func.attr not in ("_show_refusal", "_show_edit_refusal"):
				continue
			assert len(node.args) == 1 and not node.keywords, path


#============================================
def test_refusal_presentation_rejects_string_input() -> None:
	"""A caller cannot silently turn a label and detail into a generic refusal."""
	with pytest.raises(TypeError, match="exact RefusalRequest"):
		ferrum_qt.ferrum.main_window.FerrumNativeMainWindow._show_edit_refusal(
			object(),
			"legacy title",
		)
