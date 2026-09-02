"""Closed declarative vocabulary shared by Ferrum ribbon presentation layers."""

# Standard Library
import dataclasses


ACCENTS = (
	"annotation",
	"drawing",
	"reaction",
	"selection",
	"structure",
	"utility",
	"view",
)

THEME_KEYS = frozenset({
	"accent_annotation", "accent_drawing", "accent_reaction", "accent_selection",
	"accent_structure", "accent_utility", "accent_view", "button_bg", "button_border",
	"button_checked", "button_checked_border", "button_disabled_bg", "button_disabled_fg",
	"button_fg", "button_hover", "caption_fg", "context_bg", "context_fg", "focus",
	"group_bg", "group_border", "header_bg", "header_fg", "header_muted", "shell", "surface",
	"tab_active_bg", "tab_active_fg", "tab_hover",
})


#============================================
@dataclasses.dataclass(frozen=True)
class RibbonMetrics:
	"""Own the small spacing and sizing vocabulary for every ribbon component."""

	page_margins: tuple[int, int, int, int] = (8, 5, 8, 5)
	group_spacing: int = 7
	group_margins: tuple[int, int, int, int] = (6, 6, 6, 3)
	group_label_spacing: int = 3
	action_spacing: int = 4
	compact_icon_size: int = 18
	compact_caption_icon_size: int = 15
	compact_caption_font_size: int = 7
	standard_icon_size: int = 14
	large_icon_size: int = 20
	compact_control_size: int = 32
	compact_grid_height: int = 68
	standard_control_width: int = 68
	standard_control_height: int = 32
	large_control_width: int = 68
	large_control_height: int = 68
	width_step: int = 32
	control_radius: int = 4
	group_radius: int = 3
	header_control_height: int = 30


METRICS = RibbonMetrics()


#============================================
def quantized_control_width(hint: int, minimum: int, maximum: int) -> int:
	"""Snap one live text/icon hint to the bounded ribbon width rhythm."""
	bounded = max(minimum, min(hint, maximum))
	return min(maximum, ((bounded + METRICS.width_step - 1) // METRICS.width_step)
		* METRICS.width_step)
