"""Verified in-memory Atkinson Hyperlegible Next resource for Ferrum render-plan painting."""

# Standard Library
import hashlib

# PIP3 modules
import PySide6.QtGui


#============================================
class MoleculeLabelFontError(ValueError):
	"""Raised when caller-supplied Atkinson Hyperlegible Next bytes cannot meet Ferrum's contract."""


#============================================
class MoleculeLabelFont:
	"""An immutable, byte-verified Atkinson Hyperlegible Next face loaded without system fallback.

	The Rust boundary owns packaging and passes these already-vendored bytes to
	Qt.  This class checks the same public resource facts before it asks Qt to
	decode the in-memory font.  It never opens a path or registers a font with
	the process-wide font database.
	"""

	#============================================
	def __init__(self, resource_id: str, data: bytes, byte_length: int,
			sha256: str, family: str) -> None:
		"""Validate and retain the exact Ferrum Atkinson Hyperlegible Next Regular bytes.

		Args:
			resource_id: Immutable identity issued by the Rust render contract.
			data: Immutable bytes supplied by the Rust package boundary.
			byte_length: Exact byte length issued by the Rust resource.
			sha256: Exact lowercase digest issued by the Rust resource.
			family: Font family metadata verified by Rust.

		Raises:
			MoleculeLabelFontError: If the resource facts or Qt face are invalid.
		"""
		if not isinstance(data, bytes):
			raise MoleculeLabelFontError("Ferrum Atkinson Hyperlegible Next bytes must be immutable bytes")
		if len(data) != byte_length:
			raise MoleculeLabelFontError(
				"Ferrum molecule-label byte length does not match the verified resource"
			)
		digest = hashlib.sha256(data).hexdigest()
		if digest != sha256:
			raise MoleculeLabelFontError(
				"Ferrum molecule-label SHA-256 does not match the verified resource"
			)
		font = _load_raw_font(data, 1.0)
		if not font.isValid():
			raise MoleculeLabelFontError("Ferrum molecule-label bytes did not load as a Qt raw font")
		if font.familyName() != family:
			raise MoleculeLabelFontError(
				"Ferrum molecule-label Qt family does not match the verified resource"
			)
		self._resource_id = resource_id
		self._data = data

	#============================================
	@property
	def resource_id(self) -> str:
		"""Return the authenticated Rust-issued resource identity."""
		return self._resource_id

	#============================================
	def raw_font(self, pixel_size: float) -> PySide6.QtGui.QRawFont:
		"""Return a fresh in-memory raw font at one explicit positive pixel size.

		The returned font is used only for Atkinson Hyperlegible Next outlines and
		glyph identifiers.
		No Qt family lookup, substitution, registration, or metrics API occurs.
		"""
		if not isinstance(pixel_size, (float, int)) or isinstance(pixel_size, bool):
			raise MoleculeLabelFontError("Ferrum Atkinson Hyperlegible Next pixel size must be numeric")
		pixel_size = float(pixel_size)
		if pixel_size <= 0.0 or not pixel_size < float("inf"):
			raise MoleculeLabelFontError(
				"Ferrum molecule-label pixel size must be finite and positive"
			)
		font = _load_raw_font(self._data, pixel_size)
		if not font.isValid():
			raise MoleculeLabelFontError("Ferrum Atkinson Hyperlegible Next raw font reload failed")
		return font


#============================================
def from_verified_resource(resource: object) -> MoleculeLabelFont:
	"""Create a font only from the exact frozen resource published by ``ferrum_chem``."""
	try:
		import ferrum_qt.ferrum.engine as engine
	except ImportError as error:
		raise MoleculeLabelFontError(
			"Ferrum molecule-label loading requires the installed ferrum_chem extension"
		) from error
	if type(resource) is not engine.VerifiedMoleculeLabelFont:
		raise MoleculeLabelFontError("Ferrum molecule-label loading requires VerifiedMoleculeLabelFont")
	font = MoleculeLabelFont(
		resource.resource_id,
		resource.data,
		resource.byte_length,
		resource.sha256,
		resource.family,
	)
	return font


#============================================
def _load_raw_font(data: bytes, pixel_size: float) -> PySide6.QtGui.QRawFont:
	"""Load bytes with the closed no-hinting policy and no font-database lookup."""
	font = PySide6.QtGui.QRawFont()
	font.loadFromData(
		data,
		pixel_size,
		PySide6.QtGui.QFont.HintingPreference.PreferNoHinting,
	)
	return font
