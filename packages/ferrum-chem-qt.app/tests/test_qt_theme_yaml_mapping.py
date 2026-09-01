"""Test that Qt theme palette/QSS is driven entirely by YAML theme files.

Verifies that build_palette and build_qss read from YAML and do not
contain old hardcoded color values.

Usage:
	source source_me.sh && python -m pytest packages/ferrum-qt.app/tests/test_qt_theme_yaml_mapping.py -v
"""

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.themes.palettes

# expected gui keys present in both dark.yaml and light.yaml
_EXPECTED_GUI_KEYS = [
	'background', 'toolbar', 'toolbar_fg', 'separator',
	'hover', 'active_mode', 'active_mode_fg',
	'active_mode_highlight', 'active_tab_bg', 'active_tab_fg',
	'inactive_tab_fg', 'button_active_bg', 'group_separator',
	'grid_selected', 'grid_deselected', 'group_label_fg',
	'entry_bg', 'entry_fg', 'entry_disabled_fg',
	'entry_insert_bg', 'canvas_surround',
]


#============================================
def _relative_luminance(color: str) -> float:
	"""Return WCAG relative luminance for one validated hex theme token."""
	channels = tuple(int(color[index:index + 2], 16) / 255.0 for index in (1, 3, 5))
	linear = tuple(channel / 12.92 if channel <= 0.04045
		else ((channel + 0.055) / 1.055) ** 2.4 for channel in channels)
	return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]


#============================================
def _contrast(first: str, second: str) -> float:
	"""Return WCAG contrast ratio for two validated hex theme tokens."""
	lighter, darker = sorted(
		(_relative_luminance(first), _relative_luminance(second)), reverse=True,
	)
	return (lighter + 0.05) / (darker + 0.05)


#============================================
def test_ribbon_semantic_palette_meets_text_and_control_contrast() -> None:
	"""Ribbon text, focus, state, and category rails meet their WCAG thresholds."""
	for theme_name in ("dark", "light"):
		ribbon = ferrum_qt.themes.theme_loader.get_ribbon_colors(theme_name)
		text_pairs = (
			("header_fg", "header_bg"),
			("tab_active_fg", "tab_active_bg"),
			("button_fg", "button_bg"),
			("button_disabled_fg", "button_disabled_bg"),
			("caption_fg", "group_bg"),
			("context_fg", "context_bg"),
		)
		for foreground, background in text_pairs:
			assert _contrast(ribbon[foreground], ribbon[background]) >= 4.5
		control_pairs = [
			("button_border", "button_bg"),
			("button_checked_border", "button_checked"),
			("focus", "group_bg"),
			("focus", "header_bg"),
		]
		control_pairs.extend((key, "group_bg") for key in ribbon
			if key.startswith("accent_"))
		for foreground, background in control_pairs:
			assert _contrast(ribbon[foreground], ribbon[background]) >= 3.0


#============================================
def test_dark_yaml_gui_keys_exist() -> None:
	"""Verify all expected gui keys exist in dark.yaml via get_gui_colors."""
	gui = ferrum_qt.themes.theme_loader.get_gui_colors('dark')
	for key in _EXPECTED_GUI_KEYS:
		assert key in gui, f'Missing gui key in dark.yaml: {key}'
	assert len(_EXPECTED_GUI_KEYS) >= 19, 'Expected at least 19 gui keys'


#============================================
def test_light_yaml_gui_keys_exist() -> None:
	"""Verify all expected gui keys exist in light.yaml via get_gui_colors."""
	gui = ferrum_qt.themes.theme_loader.get_gui_colors('light')
	for key in _EXPECTED_GUI_KEYS:
		assert key in gui, f'Missing gui key in light.yaml: {key}'
	assert len(_EXPECTED_GUI_KEYS) >= 19, 'Expected at least 19 gui keys'


#============================================
def test_dark_qss_uses_yaml_values() -> None:
	"""Verify dark QSS contains YAML values, not old hardcoded values."""
	qss = ferrum_qt.themes.palettes.build_qss('dark')
	assert '#2b2b2b' in qss, 'Expected YAML dark background'
	assert '#1e1e2e' not in qss, 'Found old hardcoded dark background'


#============================================
def test_light_qss_uses_yaml_values() -> None:
	"""Verify light QSS contains YAML values, not old hardcoded values."""
	qss = ferrum_qt.themes.palettes.build_qss('light')
	assert '#eaeaea' in qss, 'Expected YAML light background'
	assert '#f8fafc' not in qss, 'Found old hardcoded light background'


#============================================
def test_dark_palette_returns_qpalette(qapp: object) -> None:
	"""Verify build_palette('dark') returns a QPalette instance."""
	import PySide6.QtGui
	palette = ferrum_qt.themes.palettes.build_palette('dark')
	assert isinstance(palette, PySide6.QtGui.QPalette), (
		f'Expected QPalette, got {type(palette)}'
	)


#============================================
def test_light_palette_returns_qpalette(qapp: object) -> None:
	"""Verify build_palette('light') returns a QPalette instance."""
	import PySide6.QtGui
	palette = ferrum_qt.themes.palettes.build_palette('light')
	assert isinstance(palette, PySide6.QtGui.QPalette), (
		f'Expected QPalette, got {type(palette)}'
	)


# -- canvas section tests --

_EXPECTED_CANVAS_KEYS = ['selection', 'hover', 'preview']


#============================================
def test_dark_yaml_canvas_keys_exist() -> None:
	"""Verify canvas section exists in dark.yaml with required keys."""
	colors = ferrum_qt.themes.theme_loader.get_canvas_colors('dark')
	for key in _EXPECTED_CANVAS_KEYS:
		assert key in colors, f'Missing canvas key in dark.yaml: {key}'
		assert colors[key].startswith('#'), f'Expected hex color for canvas.{key}'


#============================================
def test_light_yaml_canvas_keys_exist() -> None:
	"""Verify canvas section exists in light.yaml with required keys."""
	colors = ferrum_qt.themes.theme_loader.get_canvas_colors('light')
	for key in _EXPECTED_CANVAS_KEYS:
		assert key in colors, f'Missing canvas key in light.yaml: {key}'
		assert colors[key].startswith('#'), f'Expected hex color for canvas.{key}'


# -- chemistry charge color tests --

#============================================
def test_dark_yaml_charge_colors_exist() -> None:
	"""Verify chemistry section has charge_plus and charge_minus in dark."""
	chem = ferrum_qt.themes.theme_loader.get_chemistry_colors('dark')
	assert 'charge_plus' in chem, 'Missing charge_plus in dark chemistry'
	assert 'charge_minus' in chem, 'Missing charge_minus in dark chemistry'


#============================================
def test_light_yaml_charge_colors_exist() -> None:
	"""Verify chemistry section has charge_plus and charge_minus in light."""
	chem = ferrum_qt.themes.theme_loader.get_chemistry_colors('light')
	assert 'charge_plus' in chem, 'Missing charge_plus in light chemistry'
	assert 'charge_minus' in chem, 'Missing charge_minus in light chemistry'


# -- gui tooltip and high_contrast tests --

_EXPECTED_NEW_GUI_KEYS = [
	'high_contrast_1', 'high_contrast_2', 'tooltip_bg', 'tooltip_fg',
]


#============================================
def test_dark_yaml_new_gui_keys_exist() -> None:
	"""Verify tooltip and high_contrast keys exist in dark.yaml gui."""
	gui = ferrum_qt.themes.theme_loader.get_gui_colors('dark')
	for key in _EXPECTED_NEW_GUI_KEYS:
		assert key in gui, f'Missing gui key in dark.yaml: {key}'


#============================================
def test_light_yaml_new_gui_keys_exist() -> None:
	"""Verify tooltip and high_contrast keys exist in light.yaml gui."""
	gui = ferrum_qt.themes.theme_loader.get_gui_colors('light')
	for key in _EXPECTED_NEW_GUI_KEYS:
		assert key in gui, f'Missing gui key in light.yaml: {key}'
