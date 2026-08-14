"""Exact Qt copy of Ferrum's closed CDML paper-size catalog."""

# Standard Library
import math

# PIP3 modules
import ferrum_chem


#============================================
def paper_catalog_v1() -> dict[str, list[float] | None]:
	"""Return a fresh plain-data catalog copied from exact frozen Rust DTOs."""
	entries = ferrum_chem.paper_size_catalog_v1()
	if type(entries) is not tuple:
		raise TypeError("Ferrum paper-size catalog must be an exact tuple")
	result: dict[str, list[float] | None] = {}
	for entry in entries:
		if type(entry) is not ferrum_chem.PaperSizeV1:
			raise TypeError("Ferrum paper-size catalog has the wrong DTO type")
		name = entry.name
		if type(name) is not str or not name or name in result:
			raise ValueError("Ferrum paper-size catalog has an invalid name")
		dimensions = entry.dimensions
		if dimensions is None:
			if name != "custom":
				raise ValueError("only custom paper may omit fixed dimensions")
			result[name] = None
			continue
		if type(dimensions) is not ferrum_chem.PaperDimensionsMmV1:
			raise TypeError("Ferrum paper dimensions have the wrong DTO type")
		width = dimensions.width
		height = dimensions.height
		if any(
			type(value) is not float or not math.isfinite(value) or value <= 0.0
			for value in (width, height)
		):
			raise ValueError("Ferrum paper dimensions must be finite and positive")
		result[name] = [width, height]
	if "custom" not in result or result["custom"] is not None:
		raise ValueError("Ferrum paper-size catalog must contain custom paper")
	return result
