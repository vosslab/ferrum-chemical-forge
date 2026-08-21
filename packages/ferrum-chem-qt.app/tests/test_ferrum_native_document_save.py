"""Architecture coverage for isolated Ferrum document publication."""

# Standard Library
import pathlib


#============================================
def test_document_save_mixin_owns_no_document_admission() -> None:
	"""Publication support cannot acquire a second local-document ingress path."""
	path = pathlib.Path(__file__).parents[1] / "ferrum_qt" / "ferrum" / "document_save.py"
	source = path.read_text(encoding="utf-8")
	assert "class FerrumNativeDocumentSaveMixin" in source
	for forbidden in (
		"open_file_path", "open_native_cdml_path", "_open_native_cdml",
		"_prepare_local_cdml_admission", "prepare_local_cdml_file_v1",
	):
		assert forbidden not in source


#============================================
def test_main_window_has_one_generic_local_document_admission_owner() -> None:
	"""Normal File/Open resolves through the descriptor-dispatched local owner only."""
	package = pathlib.Path(__file__).parents[1] / "ferrum_qt" / "ferrum"
	main_window = (package / "main_window.py").read_text(encoding="utf-8")
	local_open = (package / "local_document_open.py").read_text(encoding="utf-8")
	assert "FerrumNativeLocalDocumentOpenMixin" in main_window
	assert "FerrumNativeDocumentSaveMixin" in main_window
	assert "window_native_files" not in main_window
	assert "def open_file_path(" in local_open
	assert "def open_native_cdml_path(" not in local_open
