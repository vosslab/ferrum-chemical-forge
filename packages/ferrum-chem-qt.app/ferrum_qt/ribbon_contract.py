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

	page_margins: tuple[int, int, int, int] = (8, 6, 8, 6)
	group_spacing: int = 8
	group_margins: tuple[int, int, int, int] = (8, 8, 8, 6)
	group_label_spacing: int = 4
	action_spacing: int = 8
	supporting_row_spacing: int = 4
	action_height: int = 72
	supporting_row_height: int = 34
	primary_minimum_width: int = 96
	primary_maximum_width: int = 192
	supporting_minimum_width: int = 128
	supporting_maximum_width: int = 288
	popup_width: int = 96
	width_step: int = 32
	control_radius: int = 6
	group_radius: int = 8
	header_control_height: int = 30


METRICS = RibbonMetrics()


#============================================
def quantized_control_width(hint: int, minimum: int, maximum: int) -> int:
	"""Snap one live text/icon hint to the bounded ribbon width rhythm."""
	bounded = max(minimum, min(hint, maximum))
	return min(maximum, ((bounded + METRICS.width_step - 1) // METRICS.width_step)
		* METRICS.width_step)
