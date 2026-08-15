"""Frontend-neutral Ferrum display facts for picker colors and canvas geometry."""

# local repo modules
import ferrum_chem


#============================================
def element_category_color(symbol: str) -> str:
	"""Return Ferrum's exact V1 category color for one picker element.

	Args:
		symbol: Chemical element symbol.

	Returns:
		A lowercase ``#rrggbb`` color string.

	Raises:
		ferrum_chem.UnknownElementDisplaySymbolError: If ``symbol`` is not one
		exact supported picker symbol.
	"""
	return ferrum_chem.periodic_display_facts_v1(symbol).color


#============================================
def points_per_cm() -> float:
	"""Return Ferrum's exact V1 CDML point scale as one plain finite number."""
	return ferrum_chem.cdml_points_per_cm_v1()


#============================================
def cm_to_points(cm: float) -> float:
	"""Convert one finite CDML centimetre coordinate to display points."""
	return ferrum_chem.cm_to_points_v1(cm)


#============================================
def points_to_cm(points: float) -> float:
	"""Convert one finite display-point coordinate to CDML centimetres."""
	return ferrum_chem.points_to_cm_v1(points)


#============================================
def hex_grid_points(
		x_min: float, y_min: float, x_max: float, y_max: float, spacing: float,
		) -> tuple[tuple[float, float], ...]:
	"""Return immutable finite lattice vertices for one display rectangle.

	An oversized grid is intentionally represented by an empty tuple.  Qt uses
	that same result as an empty decorative overlay rather than retaining a
	local list or a backend sentinel.
	"""
	return ferrum_chem.hex_grid_points_v1(x_min, y_min, x_max, y_max, spacing)


#============================================
def hex_grid_edges(
		x_min: float, y_min: float, x_max: float, y_max: float, spacing: float,
		) -> tuple[tuple[tuple[float, float], tuple[float, float]], ...]:
	"""Return immutable finite honeycomb edges for one display rectangle."""
	return ferrum_chem.hex_grid_edges_v1(x_min, y_min, x_max, y_max, spacing)


#============================================
def snap_to_hex_grid(x: float, y: float, spacing: float) -> tuple[float, float]:
	"""Return the nearest finite hex-grid vertex for finite display scalars.

	Raises:
		ValueError: If a coordinate or spacing is non-finite, or spacing is not
			positive.
	"""
	return ferrum_chem.snap_to_hex_grid_v1(x, y, spacing)


#============================================
def normalize_hex_grid_spacing(value: float) -> float:
	"""Return one finite positive grid spacing as a built-in float.

	This small request/result operation lets a frontend validate a proposed
	spacing before it changes any disposable Qt projection state.
	"""
	return ferrum_chem.normalize_hex_grid_spacing_v1(value)
