"""Regression coverage for the retired Qt user-template scanner boundary."""

# Standard Library
import importlib.util


#============================================
def test_qt_has_no_user_template_catalog_scanner_authority() -> None:
	"""Template discovery is issued by Rust, never reconstructed by Qt."""
	assert importlib.util.find_spec("ferrum_qt.io.user_template_catalog") is None
