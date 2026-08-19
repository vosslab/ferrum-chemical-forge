"""Small, document-space keyboard cursor values for Ferrum authoring tools."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.config.geometry_units


KEYBOARD_CURSOR_GRID_INCREMENT_PT = (
	ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT
)
"""Normal Arrow-key cursor increment in document-space points."""

KEYBOARD_CURSOR_FINE_INCREMENT_PT = KEYBOARD_CURSOR_GRID_INCREMENT_PT / 4.0
"""Shift+Arrow increment in document-space points for fine placement."""


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class KeyboardCanvasCursor:
	"""One finite document-space location without document ownership."""

	x: float
	y: float

	#============================================
	def moved(self, dx: float, dy: float) -> "KeyboardCanvasCursor":
		"""Return an independently immutable cursor translated by one increment."""
		if type(dx) is not float or type(dy) is not float:
			raise TypeError("Ferrum keyboard cursor movement requires float increments")
		return KeyboardCanvasCursor(self.x + dx, self.y + dy)


#============================================
def keyboard_cursor_increment(shift_held: bool) -> float:
	"""Return normal Arrow movement or Shift's documented fine movement."""
	if type(shift_held) is not bool:
		raise TypeError("Ferrum keyboard cursor modifier must be a boolean")
	return (
		KEYBOARD_CURSOR_FINE_INCREMENT_PT
		if shift_held else KEYBOARD_CURSOR_GRID_INCREMENT_PT
	)
