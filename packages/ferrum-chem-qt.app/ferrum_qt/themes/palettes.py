"""YAML-driven color palettes and QSS stylesheets for Ferrum themes.

All colors are loaded from the shared YAML theme files via theme_loader.
No hardcoded hex color values exist in this module.
"""

# PySide6 modules
import PySide6.QtGui

# local repo modules
import ferrum_qt.themes.theme_loader
import ferrum_qt.ribbon_contract


#============================================
def build_palette(theme_name: str) -> PySide6.QtGui.QPalette:
	"""Build a QPalette from the YAML theme gui section.

	Reads all gui color keys from the theme YAML file and maps them
	to the appropriate QPalette color roles for both active and
	disabled color groups.

	Args:
		theme_name: Theme name ('dark' or 'light'), matching YAML filename.

	Returns:
		QPalette configured with colors from the YAML theme file.
	"""
	gui = ferrum_qt.themes.theme_loader.get_gui_colors(theme_name)

	palette = PySide6.QtGui.QPalette()

	# resolve YAML gui keys to QColor objects
	background_color = PySide6.QtGui.QColor(gui["background"])
	toolbar_fg_color = PySide6.QtGui.QColor(gui["toolbar_fg"])
	toolbar_color = PySide6.QtGui.QColor(gui["toolbar"])
	entry_bg_color = PySide6.QtGui.QColor(gui["entry_bg"])
	entry_fg_color = PySide6.QtGui.QColor(gui["entry_fg"])
	active_mode_color = PySide6.QtGui.QColor(gui["active_mode"])
	active_mode_fg_color = PySide6.QtGui.QColor(gui["active_mode_fg"])
	highlight_color = PySide6.QtGui.QColor(gui["active_mode_highlight"])
	disabled_fg_color = PySide6.QtGui.QColor(gui["entry_disabled_fg"])

	# active group: window and general backgrounds
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.Window, background_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.WindowText, toolbar_fg_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.Button, toolbar_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.ButtonText, toolbar_fg_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.Base, entry_bg_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.AlternateBase, background_color
	)

	# active group: tooltip colors
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.ToolTipBase, entry_bg_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.ToolTipText, entry_fg_color
	)

	# active group: text and highlight colors
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.Text, entry_fg_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.BrightText, entry_fg_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.Link, highlight_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.Highlight, active_mode_color
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.HighlightedText,
		active_mode_fg_color,
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorRole.PlaceholderText, disabled_fg_color
	)

	# disabled group: dimmed text variants with reduced alpha
	disabled_text = PySide6.QtGui.QColor(gui["entry_disabled_fg"])
	disabled_text.setAlpha(120)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorGroup.Disabled,
		PySide6.QtGui.QPalette.ColorRole.WindowText,
		disabled_text,
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorGroup.Disabled,
		PySide6.QtGui.QPalette.ColorRole.Text,
		disabled_text,
	)
	palette.setColor(
		PySide6.QtGui.QPalette.ColorGroup.Disabled,
		PySide6.QtGui.QPalette.ColorRole.ButtonText,
		disabled_text,
	)
	# disabled highlight uses toolbar background
	palette.setColor(
		PySide6.QtGui.QPalette.ColorGroup.Disabled,
		PySide6.QtGui.QPalette.ColorRole.Highlight,
		toolbar_color,
	)
	# disabled highlighted text uses dimmed color
	palette.setColor(
		PySide6.QtGui.QPalette.ColorGroup.Disabled,
		PySide6.QtGui.QPalette.ColorRole.HighlightedText,
		disabled_text,
	)

	return palette


#============================================
def build_qss(theme_name: str) -> str:
	"""Build a QSS stylesheet from the YAML theme gui section.

	Reads all gui color keys from the theme YAML file and constructs
	a complete QSS string for the Qt application. Uses string
	concatenation for readability.

	Args:
		theme_name: Theme name ('dark' or 'light'), matching YAML filename.

	Returns:
		QSS stylesheet string with all widget selectors.
	"""
	gui = ferrum_qt.themes.theme_loader.get_gui_colors(theme_name)
	ribbon = ferrum_qt.themes.theme_loader.get_ribbon_colors(theme_name)
	metrics = ferrum_qt.ribbon_contract.METRICS

	# main window background
	qss = "QMainWindow {"
	qss += f"  background-color: {gui['background']};"
	qss += "}"

	# menu bar
	qss += "QMenuBar {"
	qss += f"  background-color: {gui['toolbar']};"
	qss += f"  color: {gui['toolbar_fg']};"
	qss += "  padding: 2px;"
	qss += "}"
	# menu bar selected item
	qss += "QMenuBar::item:selected {"
	qss += f"  background-color: {gui['active_mode']};"
	qss += f"  color: {gui['active_mode_fg']};"
	qss += "}"

	# dropdown menu
	qss += "QMenu {"
	qss += f"  background-color: {gui['entry_bg']};"
	qss += f"  color: {gui['entry_fg']};"
	qss += f"  border: 1px solid {gui['separator']};"
	qss += "}"
	# dropdown menu selected item
	qss += "QMenu::item:selected {"
	qss += f"  background-color: {gui['active_mode']};"
	qss += f"  color: {gui['active_mode_fg']};"
	qss += "}"

	# toolbar
	qss += "QToolBar {"
	qss += f"  background-color: {gui['toolbar']};"
	qss += f"  border-bottom: 1px solid {gui['separator']};"
	qss += "  spacing: 4px;"
	qss += "  padding: 2px;"
	qss += "}"

	# toolbar button checked state (active mode highlight)
	qss += "QToolBar QToolButton:checked {"
	qss += f"  background-color: {gui['active_mode']};"
	qss += f"  color: {gui['active_mode_fg']};"
	qss += f"  border: 2px solid {gui['active_mode_highlight']};"
	qss += "  border-radius: 4px;"
	qss += "}"
	# toolbar button hover state
	qss += "QToolBar QToolButton:hover {"
	qss += f"  background-color: {gui['hover']};"
	qss += "}"

	# status bar
	qss += "QStatusBar {"
	qss += f"  background-color: {gui['toolbar']};"
	qss += f"  color: {gui['group_label_fg']};"
	qss += "}"

	# tab widget pane (container)
	qss += "QTabWidget::pane {"
	qss += f"  border: 1px solid {gui['separator']};"
	qss += f"  background-color: {gui['background']};"
	qss += "}"
	# individual tabs (inactive)
	qss += "QTabBar::tab {"
	qss += f"  background-color: {gui['toolbar']};"
	qss += f"  color: {gui['inactive_tab_fg']};"
	qss += "  padding: 6px 16px;"
	qss += "  margin-right: 2px;"
	qss += "}"
	# selected tab
	qss += "QTabBar::tab:selected {"
	qss += f"  background-color: {gui['active_tab_bg']};"
	qss += f"  color: {gui['active_tab_fg']};"
	qss += "}"
	# hovered tab
	qss += "QTabBar::tab:hover {"
	qss += f"  background-color: {gui['hover']};"
	qss += f"  color: {gui['active_tab_fg']};"
	qss += "}"

	# push buttons
	qss += "QPushButton {"
	qss += f"  background-color: {gui['toolbar']};"
	qss += f"  color: {gui['toolbar_fg']};"
	qss += f"  border: 1px solid {gui['separator']};"
	qss += "  border-radius: 4px;"
	qss += "  padding: 4px 12px;"
	qss += "}"
	# hovered push button
	qss += "QPushButton:hover {"
	qss += f"  background-color: {gui['hover']};"
	qss += f"  border-color: {gui['hover']};"
	qss += "}"
	# pressed push button
	qss += "QPushButton:pressed {"
	qss += f"  background-color: {gui['active_mode']};"
	qss += "}"
	# checked push button (submode buttons)
	qss += "QPushButton:checked {"
	qss += f"  background-color: {gui['grid_selected']};"
	qss += f"  color: {gui['active_mode_fg']};"
	qss += f"  border: 1px solid {gui['active_mode_highlight']};"
	qss += "}"

	# labels
	qss += "QLabel {"
	qss += f"  color: {gui['toolbar_fg']};"
	qss += "}"

	# line edits (text input fields)
	qss += "QLineEdit {"
	qss += f"  background-color: {gui['entry_bg']};"
	qss += f"  color: {gui['entry_fg']};"
	qss += f"  border: 1px solid {gui['separator']};"
	qss += "  border-radius: 4px;"
	qss += "  padding: 4px;"
	qss += "}"
	# focused line edit
	qss += "QLineEdit:focus {"
	qss += f"  border-color: {gui['active_mode_highlight']};"
	qss += "}"

	# graphics view (canvas) -- borderless
	qss += "QGraphicsView {"
	qss += "  border: none;"
	qss += "}"

	# Ferrum's primary command surface uses a dedicated, closed theme layer.
	qss += "QToolBar#ferrum-authoring-ribbon {"
	qss += f"  background-color: {ribbon['shell']};"
	qss += "  border: none;"
	qss += "  padding: 0px;"
	qss += "  spacing: 0px;"
	qss += "}"
	qss += "QWidget#authoring-ribbon-header {"
	qss += f"  background-color: {ribbon['header_bg']};"
	qss += f"  color: {ribbon['header_fg']};"
	qss += "  border: none;"
	qss += "}"
	qss += "QLabel#authoring-ribbon-brand {"
	qss += f"  color: {ribbon['header_fg']};"
	qss += "  font-weight: 700;"
	qss += "  letter-spacing: 1px;"
	qss += "  padding: 0px 10px 0px 2px;"
	qss += "}"
	qss += "QFrame#authoring-ribbon-header-separator {"
	qss += f"  color: {ribbon['header_muted']};"
	qss += "  margin: 5px 7px;"
	qss += "}"
	qss += "QWidget#authoring-ribbon-header QToolButton {"
	qss += f"  color: {ribbon['header_fg']};"
	qss += f"  background-color: {ribbon['tab_hover']};"
	qss += f"  border: 1px solid {ribbon['header_muted']};"
	qss += f"  border-radius: {metrics.control_radius}px;"
	qss += "  padding: 5px 7px;"
	qss += "}"
	qss += "QWidget#authoring-ribbon-header QToolButton:hover {"
	qss += f"  background-color: {ribbon['tab_hover']};"
	qss += f"  border-color: {ribbon['header_muted']};"
	qss += "}"
	qss += "QWidget#authoring-ribbon-header QToolButton:focus {"
	qss += f"  border: 2px solid {ribbon['focus']};"
	qss += "}"
	qss += "QToolButton[ribbonHeaderRole='global'] {"
	qss += f"  background-color: {ribbon['tab_hover']};"
	qss += "  font-weight: 600;"
	qss += "}"
	qss += "QTabBar#authoring-ribbon-tabs {"
	qss += f"  background-color: {ribbon['header_bg']};"
	qss += "}"
	qss += "QTabBar#authoring-ribbon-tabs::tab {"
	qss += f"  background-color: {ribbon['header_bg']};"
	qss += f"  color: {ribbon['header_muted']};"
	qss += "  border: none;"
	qss += f"  border-radius: {metrics.control_radius}px;"
	qss += "  padding: 8px 15px;"
	qss += "  margin: 0px 1px;"
	qss += "  font-weight: 600;"
	qss += "}"
	qss += "QTabBar#authoring-ribbon-tabs::tab:hover {"
	qss += f"  background-color: {ribbon['tab_hover']};"
	qss += f"  color: {ribbon['header_fg']};"
	qss += "}"
	qss += "QTabBar#authoring-ribbon-tabs::tab:selected {"
	qss += f"  background-color: {ribbon['tab_active_bg']};"
	qss += f"  color: {ribbon['tab_active_fg']};"
	qss += "}"
	qss += "QStackedWidget#authoring-ribbon-pages {"
	qss += f"  background-color: {ribbon['surface']};"
	qss += "  border: none;"
	qss += "}"
	qss += "QWidget[ribbonGroup='true'] {"
	qss += f"  background-color: {ribbon['group_bg']};"
	qss += f"  border: 1px solid {ribbon['group_border']};"
	qss += f"  border-radius: {metrics.group_radius}px;"
	qss += "}"
	for accent in ferrum_qt.ribbon_contract.ACCENTS:
		qss += f"QWidget[ribbonGroup='true'][ribbonAccent='{accent}'] {{"
		qss += f"  border-top: 3px solid {ribbon[f'accent_{accent}']};"
		qss += "}"
	qss += "QWidget#authoring-ribbon-pages QToolButton {"
	qss += f"  color: {ribbon['button_fg']};"
	qss += f"  background-color: {ribbon['button_bg']};"
	qss += f"  border: 1px solid {ribbon['button_border']};"
	qss += f"  border-radius: {metrics.control_radius}px;"
	qss += "  padding: 4px 6px;"
	qss += "}"
	qss += "QWidget#authoring-ribbon-pages QToolButton:hover {"
	qss += f"  background-color: {ribbon['button_hover']};"
	qss += "}"
	for accent in ferrum_qt.ribbon_contract.ACCENTS:
		qss += f"QToolButton[ribbonAccent='{accent}']:hover {{"
		qss += f"  border-color: {ribbon[f'accent_{accent}']};"
		qss += "}"
	qss += "QWidget#authoring-ribbon-pages QToolButton:checked {"
	qss += f"  background-color: {ribbon['button_checked']};"
	qss += f"  border: 2px solid {ribbon['button_checked_border']};"
	qss += "}"
	qss += "QWidget#authoring-ribbon-pages QToolButton:focus {"
	qss += f"  border: 2px solid {ribbon['focus']};"
	qss += "}"
	qss += "QWidget#authoring-ribbon-pages QToolButton:disabled {"
	qss += f"  color: {ribbon['button_disabled_fg']};"
	qss += f"  background-color: {ribbon['button_disabled_bg']};"
	qss += f"  border-color: {ribbon['group_border']};"
	qss += "}"
	qss += "QToolButton[ribbonRole='primary'] {"
	qss += "  font-weight: 600;"
	qss += "  padding: 5px 8px;"
	qss += "}"
	qss += "QToolButton[ribbonRole='supporting'] {"
	qss += "  padding: 4px 7px;"
	qss += "  text-align: left;"
	qss += "}"
	qss += "QToolButton[ribbonRole='overflow']::menu-indicator {"
	qss += "  image: none;"
	qss += "}"
	qss += "QLabel[ribbonCaption='true'] {"
	qss += f"  color: {ribbon['caption_fg']};"
	qss += "  font-size: 11px;"
	qss += "  font-weight: 600;"
	qss += "  padding-top: 1px;"
	qss += "}"
	qss += "QWidget#authoring-context-row {"
	qss += f"  background-color: {ribbon['context_bg']};"
	qss += f"  color: {ribbon['context_fg']};"
	qss += f"  border-top: 1px solid {ribbon['group_border']};"
	qss += "}"
	qss += "QWidget#authoring-context-row QLabel {"
	qss += f"  color: {ribbon['context_fg']};"
	qss += "  font-weight: 600;"
	qss += "}"

	return qss
