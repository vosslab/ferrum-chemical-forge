"""Build Qt paths from Ferrum-issued molecule-label glyph identifiers and origins."""

# Standard Library
import math
import unicodedata

# PIP3 modules
import PySide6.QtCore
import PySide6.QtGui


#============================================
class MoleculeLabelGlyphOutlineError(ValueError):
	"""A supplied glyph run cannot be painted without frontend interpretation."""


#============================================
def path_from_runs(runs: tuple[object, ...], origin: PySide6.QtCore.QPointF,
		font: PySide6.QtGui.QRawFont) -> PySide6.QtGui.QPainterPath:
	"""Build outlines from exact glyph IDs and origins, without shaping or advances."""
	if not isinstance(runs, tuple) or not runs:
		raise MoleculeLabelGlyphOutlineError("Ferrum render text requires a frozen run tuple")
	if not isinstance(origin, PySide6.QtCore.QPointF) or not font.isValid():
		raise MoleculeLabelGlyphOutlineError(
			"Ferrum render text requires a valid origin and molecule-label face"
		)
	path = PySide6.QtGui.QPainterPath()
	for run in runs:
		if not isinstance(run.text, str) or not run.text.strip():
			raise MoleculeLabelGlyphOutlineError("Ferrum render text run must contain visible text")
		if any(unicodedata.category(character) == "Cc" for character in run.text):
			raise MoleculeLabelGlyphOutlineError("Ferrum render text run cannot contain control characters")
		if run.script not in {"baseline", "subscript", "superscript"}:
			raise MoleculeLabelGlyphOutlineError("Ferrum render text run has an unknown script")
		glyphs = run.glyphs
		if not isinstance(glyphs, tuple) or len(glyphs) != len(run.text):
			raise MoleculeLabelGlyphOutlineError(
				"Ferrum render run requires one frozen glyph per Unicode scalar",
			)
		run_origin = _point(run.origin, "text run origin")
		scale = _positive(run.scale, "text run scale")
		for glyph in glyphs:
			if (
				type(glyph.glyph_index) is not int
				or glyph.glyph_index <= 0
				or glyph.glyph_index >= 2**32
			):
				raise MoleculeLabelGlyphOutlineError(
					"Ferrum render glyph index must be a nonzero u32 integer",
				)
			glyph_path = font.pathForGlyph(glyph.glyph_index)
			if glyph_path.isEmpty():
				raise MoleculeLabelGlyphOutlineError(
					"Ferrum Atkinson Hyperlegible Next returned an empty required glyph outline",
				)
			glyph_origin = _point(glyph.origin, "text glyph origin")
			transform = PySide6.QtGui.QTransform()
			transform.translate(
				origin.x() + run_origin.x() + glyph_origin.x(),
				origin.y() + run_origin.y() + glyph_origin.y(),
			)
			transform.scale(scale, scale)
			path.addPath(transform.map(glyph_path))
	return path


#============================================
def path_from_presentation_runs(runs: tuple[object, ...],
		font: PySide6.QtGui.QRawFont) -> PySide6.QtGui.QPainterPath:
	"""Build direct-root Text outlines while retaining whitespace advances from Rust."""
	if not isinstance(runs, tuple) or not runs:
		raise MoleculeLabelGlyphOutlineError("Ferrum presentation Text requires a frozen run tuple")
	if not font.isValid():
		raise MoleculeLabelGlyphOutlineError(
			"Ferrum presentation Text requires the verified molecule-label face"
		)
	path = PySide6.QtGui.QPainterPath()
	for run in runs:
		if not isinstance(run.text, str) or not run.text:
			raise MoleculeLabelGlyphOutlineError("Ferrum presentation Text run must not be empty")
		if any(unicodedata.category(character) == "Cc" for character in run.text):
			raise MoleculeLabelGlyphOutlineError("Ferrum presentation Text run cannot contain controls")
		if run.script not in {"baseline", "subscript", "superscript"}:
			raise MoleculeLabelGlyphOutlineError("Ferrum presentation Text run has an unknown script")
		glyphs = run.glyphs
		if not isinstance(glyphs, tuple) or len(glyphs) != len(run.text):
			raise MoleculeLabelGlyphOutlineError(
				"Ferrum presentation Text requires one glyph per Unicode scalar",
			)
		run_origin = _point(run.origin, "presentation Text run origin")
		scale = _positive(run.scale, "presentation Text run scale")
		for character, glyph in zip(run.text, glyphs, strict=True):
			if (
				type(glyph.glyph_index) is not int
				or glyph.glyph_index <= 0
				or glyph.glyph_index >= 2**32
			):
				raise MoleculeLabelGlyphOutlineError(
					"Ferrum presentation Text glyph index must be a nonzero u32 integer",
				)
			glyph_path = font.pathForGlyph(glyph.glyph_index)
			if glyph_path.isEmpty():
				if not character.isspace():
					raise MoleculeLabelGlyphOutlineError(
						"Ferrum Atkinson Hyperlegible Next returned an empty visible glyph outline",
					)
				continue
			glyph_origin = _point(glyph.origin, "presentation Text glyph origin")
			transform = PySide6.QtGui.QTransform()
			transform.translate(
				run_origin.x() + glyph_origin.x(),
				run_origin.y() + glyph_origin.y(),
			)
			transform.scale(scale, scale)
			path.addPath(transform.map(glyph_path))
	if path.isEmpty():
		raise MoleculeLabelGlyphOutlineError("Ferrum presentation Text has no visible glyph outline")
	return path


#============================================
def _point(value: object, description: str) -> PySide6.QtCore.QPointF:
	"""Copy one finite renderer point without accepting a sequence or mapping."""
	x = getattr(value, "x", None)
	y = getattr(value, "y", None)
	if type(x) not in (int, float) or type(y) not in (int, float):
		raise MoleculeLabelGlyphOutlineError(f"Ferrum {description} must be numeric")
	if not math.isfinite(x) or not math.isfinite(y):
		raise MoleculeLabelGlyphOutlineError(f"Ferrum {description} must be finite")
	return PySide6.QtCore.QPointF(float(x), float(y))


#============================================
def _positive(value: object, description: str) -> float:
	"""Return one finite positive renderer scalar."""
	if type(value) not in (int, float) or not math.isfinite(value) or value <= 0.0:
		raise MoleculeLabelGlyphOutlineError(f"Ferrum {description} must be finite and positive")
	return float(value)
