"""Verified in-memory Telex resource for Ferrum render-plan painting."""

# Standard Library
import hashlib

# PIP3 modules
import PySide6.QtGui


TELEX_RESOURCE_ID = "ferrum-telex-regular-v1"
TELEX_BYTES = 38_940
TELEX_SHA256 = "eeaa2d17d105b6b46e5368ecd990f5b19c50131ff922dbf79bfb9bb45c249871"
TELEX_FAMILY = "Telex"
TELEX_STYLE = "Regular"


#============================================
class FerrumTelexError(ValueError):
	"""Raised when caller-supplied Telex bytes cannot meet Ferrum's contract."""


#============================================
class FerrumTelex:
	"""An immutable, byte-verified Telex face loaded without system fallback.

	The Rust boundary owns packaging and passes these already-vendored bytes to
	Qt.  This class checks the same public resource facts before it asks Qt to
	decode the in-memory font.  It never opens a path or registers a font with
	the process-wide font database.
	"""

	#============================================
	def __init__(self, data: bytes) -> None:
		"""Validate and retain the exact Ferrum Telex Regular bytes.

		Args:
			data: Immutable bytes supplied by the Rust package boundary.

		Raises:
			FerrumTelexError: If the resource facts or Qt face are invalid.
		"""
		if not isinstance(data, bytes):
			raise FerrumTelexError("Ferrum Telex bytes must be immutable bytes")
		if len(data) != TELEX_BYTES:
			raise FerrumTelexError("Ferrum Telex byte length does not match v1")
		digest = hashlib.sha256(data).hexdigest()
		if digest != TELEX_SHA256:
			raise FerrumTelexError("Ferrum Telex SHA-256 does not match v1")
		font = _load_raw_font(data, 1.0)
		if not font.isValid():
			raise FerrumTelexError("Ferrum Telex bytes did not load as a Qt raw font")
		if font.familyName() != TELEX_FAMILY or font.styleName() != TELEX_STYLE:
			raise FerrumTelexError("Ferrum Telex Qt family or style does not match v1")
		self._data = data

	#============================================
	def raw_font(self, pixel_size: float) -> PySide6.QtGui.QRawFont:
		"""Return a fresh in-memory raw font at one explicit positive pixel size.

		The returned font is used only for Telex outlines and glyph identifiers.
		No Qt family lookup, substitution, registration, or metrics API occurs.
		"""
		if not isinstance(pixel_size, (float, int)) or isinstance(pixel_size, bool):
			raise FerrumTelexError("Ferrum Telex pixel size must be numeric")
		pixel_size = float(pixel_size)
		if pixel_size <= 0.0 or not pixel_size < float("inf"):
			raise FerrumTelexError("Ferrum Telex pixel size must be finite and positive")
		font = _load_raw_font(self._data, pixel_size)
		if not font.isValid():
			raise FerrumTelexError("Ferrum Telex raw font reload failed")
		return font


#============================================
def from_verified_resource(resource: object) -> FerrumTelex:
	"""Create Telex only from the exact frozen resource published by ``ferrum_chem``."""
	try:
		import ferrum_chem
	except ImportError as error:
		raise FerrumTelexError("Ferrum Telex requires the installed ferrum_chem extension") from error
	if type(resource) is not ferrum_chem.VerifiedTelexRegularV1:
		raise FerrumTelexError("Ferrum Telex requires VerifiedTelexRegularV1")
	if (
		resource.resource_id != TELEX_RESOURCE_ID
		or resource.byte_length != TELEX_BYTES
		or resource.sha256 != TELEX_SHA256
		or resource.family != TELEX_FAMILY
		or resource.postscript_name != "Telex-Regular"
	):
		raise FerrumTelexError("Ferrum Telex resource metadata does not match v1")
	font = FerrumTelex(resource.data)
	return font


#============================================
def _load_raw_font(data: bytes, pixel_size: float) -> PySide6.QtGui.QRawFont:
	"""Load bytes with the v1 no-hinting policy and no font-database lookup."""
	font = PySide6.QtGui.QRawFont()
	font.loadFromData(
		data,
		pixel_size,
		PySide6.QtGui.QFont.HintingPreference.PreferNoHinting,
	)
	return font
