"""One host-owned query for native canvas interaction containment."""


#============================================
def tab_has_active_native_canvas_interaction(window: object, tab: object) -> bool:
	"""Return whether a tab owns an unfinished native pointer interaction.

	The ordinary host owns atom insertion and the unified line intent. The latter
	covers bond, ring, bracket, move, rotation, and top-level translation tools.
	Callers use this truth instead of inferring activity from actions or previews.
	"""
	atom = getattr(window, "_atom_insertion_intent", None)
	line = getattr(window, "_line_gesture_intent", None)
	return (
		atom is not None and getattr(atom, "tab", None) is tab
	) or (
		line is not None and getattr(line, "tab", None) is tab
	)
