"""Resolve Ferrum's YAML-owned semantic document-display colors."""

# Standard Library
import dataclasses
import enum
import math

# PySide6 modules
import PySide6.QtGui


class DocumentDisplayPaletteError(ValueError):
	"""Report malformed or incomplete document-display palette facts."""


#============================================
class DocumentDisplayRoleV1(enum.StrEnum):
	"""Closed display-only roles consumed by the Qt document surface."""

	CANVAS_SURROUND = "canvas_surround"
	PAGE_FILL = "page_fill"
	PAGE_OUTLINE = "page_outline"
	DOCUMENT_FOREGROUND = "document_foreground"
	ATOM_NUMBER = "atom_number"
	SELECTION_OUTLINE = "selection_outline"
	HOVER_OUTLINE = "hover_outline"
	PREVIEW_OUTLINE = "preview_outline"
	PREVIEW_FILL = "preview_fill"
	KEYBOARD_CURSOR = "keyboard_cursor"
	GRID_LINE = "grid_line"
	GRID_DOT_OUTLINE = "grid_dot_outline"
	GRID_DOT_FILL = "grid_dot_fill"


_THIN_CONTENT_ROLES = frozenset((
	DocumentDisplayRoleV1.DOCUMENT_FOREGROUND,
	DocumentDisplayRoleV1.ATOM_NUMBER,
	DocumentDisplayRoleV1.SELECTION_OUTLINE,
	DocumentDisplayRoleV1.HOVER_OUTLINE,
	DocumentDisplayRoleV1.PREVIEW_OUTLINE,
	DocumentDisplayRoleV1.KEYBOARD_CURSOR,
))

_CONTRAST_FREE_ROLES = frozenset((
	DocumentDisplayRoleV1.CANVAS_SURROUND,
	DocumentDisplayRoleV1.PAGE_FILL,
))

_REQUIRED_ROLES = frozenset(DocumentDisplayRoleV1)

_RENDER_THEME_ROLE_TO_DISPLAY_ROLE = {
	"document_foreground": DocumentDisplayRoleV1.DOCUMENT_FOREGROUND,
	"atom_number": DocumentDisplayRoleV1.ATOM_NUMBER,
}


#============================================
def document_display_minimum_contrast(role: DocumentDisplayRoleV1) -> float:
	"""Return the approved contrast threshold for one document-display role."""
	if type(role) is not DocumentDisplayRoleV1:
		msg = f"Unknown document display role: {role!r}"
		raise DocumentDisplayPaletteError(msg)
	if role in _CONTRAST_FREE_ROLES:
		return 0.0
	if role in _THIN_CONTENT_ROLES:
		return 4.5
	return 3.0


#============================================
def color_contrast_ratio(
		foreground: PySide6.QtGui.QColor,
		background: PySide6.QtGui.QColor,
		) -> float:
	"""Return the WCAG contrast ratio for two fully opaque Qt colors."""
	return (
		_max_relative_luminance(foreground, background) + 0.05
	) / (
		_min_relative_luminance(foreground, background) + 0.05)


#============================================
def _max_relative_luminance(
		first: PySide6.QtGui.QColor,
		second: PySide6.QtGui.QColor,
		) -> float:
	"""Return the larger relative luminance without leaking tuple arithmetic."""
	return max(_relative_luminance(first), _relative_luminance(second))


#============================================
def _min_relative_luminance(
		first: PySide6.QtGui.QColor,
		second: PySide6.QtGui.QColor,
		) -> float:
	"""Return the smaller relative luminance without leaking tuple arithmetic."""
	return min(_relative_luminance(first), _relative_luminance(second))


#============================================
def _relative_luminance(color: PySide6.QtGui.QColor) -> float:
	"""Return WCAG relative luminance for one validated opaque color."""
	red = _linear_channel(color.red())
	green = _linear_channel(color.green())
	blue = _linear_channel(color.blue())
	return (0.2126 * red) + (0.7152 * green) + (0.0722 * blue)


#============================================
def _linear_channel(channel: int) -> float:
	"""Convert one eight-bit sRGB channel to linear light."""
	value = channel / 255.0
	if value <= 0.04045:
		return value / 12.92
	return math.pow((value + 0.055) / 1.055, 2.4)


#============================================
@dataclasses.dataclass(frozen=True)
class DocumentDisplayPaletteV1:
	"""Immutable semantic display colors issued exclusively from one YAML theme."""

	_tokens: tuple[tuple[DocumentDisplayRoleV1, str], ...]
	_elements: tuple[tuple[str, str], ...]

	#============================================
	@classmethod
	def from_yaml(cls, document_display: object) -> "DocumentDisplayPaletteV1":
		"""Build and validate one complete document-display palette from YAML."""
		if type(document_display) is not dict:
			msg = "document_display must be a YAML mapping"
			raise DocumentDisplayPaletteError(msg)
		allowed_keys = {role.value for role in DocumentDisplayRoleV1} | {"elements"}
		actual_keys = set(document_display)
		unknown_keys = actual_keys - allowed_keys
		if unknown_keys:
			msg = f"document_display has unknown token(s): {sorted(unknown_keys)!r}"
			raise DocumentDisplayPaletteError(msg)
		missing_keys = {role.value for role in _REQUIRED_ROLES} - actual_keys
		if missing_keys:
			msg = f"document_display is missing token(s): {sorted(missing_keys)!r}"
			raise DocumentDisplayPaletteError(msg)
		if "elements" not in document_display:
			msg = "document_display is missing its elements map"
			raise DocumentDisplayPaletteError(msg)
		elements = document_display["elements"]
		if type(elements) is not dict or elements:
			msg = "document_display.elements must be an empty mapping in V1"
			raise DocumentDisplayPaletteError(msg)
		tokens = tuple(
			(role, _validated_color_token(role.value, document_display[role.value]))
			for role in DocumentDisplayRoleV1
		)
		palette = cls(tokens, ())
		palette._validate_contrast()
		return palette

	#============================================
	@property
	def element_symbols(self) -> tuple[str, ...]:
		"""Return the validated element-role names supplied by this palette."""
		return tuple(symbol for symbol, _color in self._elements)

	#============================================
	def color(self, role: DocumentDisplayRoleV1) -> PySide6.QtGui.QColor:
		"""Return a fresh Qt color for one closed display role."""
		if type(role) is not DocumentDisplayRoleV1:
			msg = f"Unknown document display role: {role!r}"
			raise DocumentDisplayPaletteError(msg)
		for known_role, token in self._tokens:
			if known_role is role:
				return PySide6.QtGui.QColor(token)
		msg = f"Document display role is unmapped: {role.value}"
		raise DocumentDisplayPaletteError(msg)

	#============================================
	def resolve_render_paint(self, paint: object) -> PySide6.QtGui.QColor:
		"""Resolve one frozen Rust V3 paint at the sole Qt display boundary."""
		kind = _paint_string_field(paint, "kind")
		export_rgb = _paint_rgb_field(paint)
		role = _paint_optional_string_field(paint, "role")
		element = _paint_optional_string_field(paint, "element")
		if kind == "authored_rgb24":
			if role is not None or element is not None:
				_raise_malformed_paint(kind)
			return PySide6.QtGui.QColor(f"#{export_rgb}")
		if kind == "theme_role":
			if element is not None or role is None:
				_raise_malformed_paint(kind)
			return self.color(_theme_display_role(role))
		if kind == "element_role":
			if role is not None or element is None:
				_raise_malformed_paint(kind)
			return self._element_color(element)
		msg = f"Unknown Rust render paint kind: {kind!r}"
		raise DocumentDisplayPaletteError(msg)

	#============================================
	def _element_color(self, element: str) -> PySide6.QtGui.QColor:
		"""Resolve a Rust-issued element role only when YAML explicitly maps it."""
		for symbol, token in self._elements:
			if symbol == element:
				return PySide6.QtGui.QColor(token)
		msg = f"Document display element role is unmapped: {element!r}"
		raise DocumentDisplayPaletteError(msg)

	#============================================
	def _validate_contrast(self) -> None:
		"""Reject a theme whose complete display palette misses an approved ratio."""
		page_fill = self.color(DocumentDisplayRoleV1.PAGE_FILL)
		for role in DocumentDisplayRoleV1:
			if role in _CONTRAST_FREE_ROLES:
				continue
			ratio = color_contrast_ratio(self.color(role), page_fill)
			minimum = document_display_minimum_contrast(role)
			if ratio < minimum:
				msg = (
					f"document_display.{role.value} contrast {ratio:.2f}:1 "
					f"is below {minimum:.1f}:1 against page_fill"
				)
				raise DocumentDisplayPaletteError(msg)


#============================================
def _validated_color_token(name: str, value: object) -> str:
	"""Return one opaque YAML color as a normalized RGB token."""
	if type(value) is not str:
		msg = f"document_display.{name} must be a color string"
		raise DocumentDisplayPaletteError(msg)
	color = PySide6.QtGui.QColor(value)
	if not color.isValid() or color.alpha() != 255:
		msg = f"document_display.{name} must be an opaque valid color"
		raise DocumentDisplayPaletteError(msg)
	return color.name()


#============================================
def _paint_string_field(paint: object, name: str) -> str:
	"""Return one required nonempty string field from a frozen paint DTO."""
	try:
		value = getattr(paint, name)
	except AttributeError as error:
		msg = f"Rust render paint is missing {name!r}"
		raise DocumentDisplayPaletteError(msg) from error
	if type(value) is not str or not value:
		msg = f"Rust render paint has malformed {name!r}"
		raise DocumentDisplayPaletteError(msg)
	return value


#============================================
def _paint_optional_string_field(paint: object, name: str) -> str | None:
	"""Return one optional string field from a frozen paint DTO."""
	try:
		value = getattr(paint, name)
	except AttributeError as error:
		msg = f"Rust render paint is missing {name!r}"
		raise DocumentDisplayPaletteError(msg) from error
	if value is None:
		return None
	if type(value) is not str or not value:
		msg = f"Rust render paint has malformed {name!r}"
		raise DocumentDisplayPaletteError(msg)
	return value


#============================================
def _paint_rgb_field(paint: object) -> str:
	"""Return one six-digit Rust export RGB field after structural validation."""
	value = _paint_string_field(paint, "export_rgb")
	if len(value) != 6 or any(character not in "0123456789abcdef" for character in value):
		msg = "Rust render paint has malformed 'export_rgb'"
		raise DocumentDisplayPaletteError(msg)
	return value


#============================================
def _theme_display_role(role: str) -> DocumentDisplayRoleV1:
	"""Map one closed Rust semantic role to its YAML display role."""
	try:
		return _RENDER_THEME_ROLE_TO_DISPLAY_ROLE[role]
	except KeyError as error:
		msg = f"Unknown Rust theme paint role: {role!r}"
		raise DocumentDisplayPaletteError(msg) from error


#============================================
def _raise_malformed_paint(kind: str) -> None:
	"""Raise the typed refusal for fields that disagree with one V3 tag."""
	msg = f"Rust render paint fields are malformed for {kind!r}"
	raise DocumentDisplayPaletteError(msg)
