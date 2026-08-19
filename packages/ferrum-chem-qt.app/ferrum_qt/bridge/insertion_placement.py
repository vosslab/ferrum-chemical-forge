"""Qt-session capture for plain backend molecule-insertion placement data."""

# local repo modules
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.config.geometry_units


DEFAULT_INSERTION_ANCHOR = (2000.0, 1500.0)


#============================================
def capture_insertion_placement_v1(target: object) -> object:
	"""Snapshot scene scale and paper center as one frozen Ferrum value."""
	bond_length_pt = ferrum_qt.config.geometry_units.DEFAULT_BOND_LENGTH_PT
	anchor = DEFAULT_INSERTION_ANCHOR
	scene = getattr(target, "scene", None)
	if scene is not None and hasattr(scene, "grid_spacing_pt"):
		bond_length_pt = float(scene.grid_spacing_pt)
	if scene is not None and hasattr(scene, "paper_rect"):
		center = scene.paper_rect.center()
		anchor = (float(center.x()), float(center.y()))
	return engine.validate_insertion_placement_v1(
		bond_length_pt, anchor[0], anchor[1],
	)


#============================================
def capture_insertion_placement(target: object) -> tuple[float, tuple[float, float]]:
	"""Snapshot scene scale and paper center as built-in worker values only."""
	validated = capture_insertion_placement_v1(target)
	return validated.bond_length_pt, (validated.anchor_x, validated.anchor_y)
