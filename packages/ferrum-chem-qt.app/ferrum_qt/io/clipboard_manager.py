"""Clipboard support for complete Ferrum top-level CDML objects."""

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.io.cdml_fragment_builder
import ferrum_qt.io.clipboard_mime


# Ferrum publishes its own type and the historical type for clipboard compatibility.
CDML_MIME_TYPE = "application/x-ferrum-cdml"
LEGACY_CDML_MIME_TYPE = "application/x-bkchem-cdml"

#============================================
class ClipboardManager:
	"""Copy selection and read raw CDML through the Qt clipboard adapter."""

	#============================================
	def copy_selection(self, document: object) -> int:
		"""Copy selected molecules and presentation objects in document order.

		Selection is resolved by ``Document.selected_top_level_objects`` so a
		selected atom, bond, or atom-attached mark means copy its entire owning
		molecule.  The serializer reads original models only; it never reparents
		them into a temporary document.
		"""
		objects = document.selected_top_level_objects
		if not objects:
			return 0
		cdml_text = ferrum_qt.io.cdml_fragment_builder.build_top_level_fragment(
			document, objects,
		)
		object_count = len(objects)
		# ``setMimeData`` can synchronously notify application observers.  Preserve
		# only the completed plain fragment across that native callback boundary so
		# it cannot retain a projection document or selected model wrappers.
		del objects
		del document
		self.publish_fragment(cdml_text)
		return object_count

	#============================================
	def publish_fragment(self, cdml_text: str) -> None:
		"""Publish already-authoritative raw CDML without inspecting a projection."""
		if not isinstance(cdml_text, str):
			raise TypeError("Clipboard CDML fragment must be text")
		clipboard = PySide6.QtWidgets.QApplication.clipboard()
		mime_data = PySide6.QtCore.QMimeData()
		encoded = PySide6.QtCore.QByteArray(cdml_text.encode("utf-8"))
		mime_data.setData(CDML_MIME_TYPE, encoded)
		mime_data.setData(LEGACY_CDML_MIME_TYPE, encoded)
		mime_data.setText(cdml_text)
		mime_data.setProperty(
			ferrum_qt.io.clipboard_mime.FERRUM_OWNED_MIME_PROPERTY, True,
		)
		clipboard.setMimeData(mime_data)

	#============================================
	def read_fragment(self) -> tuple[str, str | None]:
		"""Read raw CDML once, leaving all validation to the backend."""
		return _read_cdml_from_clipboard()

	#============================================
	def can_paste(self) -> bool:
		"""Return whether the system clipboard appears to contain CDML content."""
		clipboard = PySide6.QtWidgets.QApplication.clipboard()
		mime_data = clipboard.mimeData()
		if mime_data is None:
			return False
		if any(
			mime_data.hasFormat(mime_type)
			for mime_type in (CDML_MIME_TYPE, LEGACY_CDML_MIME_TYPE)
		):
			return True
		if mime_data.hasText():
			text = mime_data.text()
			return "<cdml" in text or "<molecule" in text
		return False


#============================================
def _read_cdml_from_clipboard() -> tuple[str, str | None]:
	"""Read custom CDML data first, then a plain-text CDML fallback."""
	clipboard = PySide6.QtWidgets.QApplication.clipboard()
	mime_data = clipboard.mimeData()
	if mime_data is None:
		return ("no_data", None)
	for mime_type in (CDML_MIME_TYPE, LEGACY_CDML_MIME_TYPE):
		if not mime_data.hasFormat(mime_type):
			continue
		raw = mime_data.data(mime_type)
		try:
			cdml_text = bytes(raw).decode("utf-8")
		except UnicodeDecodeError:
			return ("decode_error", None)
		return ("ok", cdml_text)
	if mime_data.hasText():
		text = mime_data.text()
		if "<cdml" in text or "<molecule" in text:
			return ("ok", text)
	return ("no_data", None)


#============================================
