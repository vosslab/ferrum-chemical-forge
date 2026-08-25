"""Static guard for the author-facing refusal wiring owned by this package."""

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.ferrum.main_window


#============================================
def test_refusal_presentation_rejects_string_input() -> None:
	"""A caller cannot silently turn a label and detail into a generic refusal."""
	with pytest.raises(TypeError, match="exact RefusalRequest"):
		ferrum_qt.ferrum.main_window.FerrumNativeMainWindow._show_edit_refusal(
			object(),
			"legacy title",
		)
