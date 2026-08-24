"""Transient Qt adapter for Rust-owned standalone Text placement."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.dialogs.rich_text_dialog


_STYLE_TO_DIALOG = {
	"subscript": "sub",
	"superscript": "sup",
}
_STYLE_TO_RUST = {
	"sub": "subscript",
	"sup": "superscript",
}


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumTextPlacementDialogModel:
	"""Immutable presentation of Rust-issued Text defaults for one modal dialog."""

	runs: tuple[tuple[str, tuple[str, ...]], ...]
	font_size: int
	color: str


#============================================
def dialog_model_from_defaults(defaults: object) -> FerrumTextPlacementDialogModel:
	"""Copy only the closed Rust values representable by the P1 Text dialog."""
	import ferrum_qt.ferrum.engine as engine
	if type(defaults) is not engine.TextPlacementDefaultsV1:
		raise TypeError("Ferrum Text placement requires exact Rust defaults")
	if defaults.bold_supported or defaults.italic_supported or defaults.font_family_supported:
		raise ValueError("Ferrum Text placement received an unsupported dialog capability")
	if type(defaults.font_size) not in (int, float) or not float(defaults.font_size).is_integer():
		raise ValueError("Ferrum Text placement default size is not an integer")
	font_size = int(defaults.font_size)
	if not 4 <= font_size <= 144 or type(defaults.color) is not str:
		raise ValueError("Ferrum Text placement defaults are not representable")
	runs = []
	for run in defaults.runs:
		if type(run) is not engine.DocumentTextEditRunV1:
			raise TypeError("Ferrum Text placement has an invalid Rust run")
		styles = []
		for style in run.styles:
			name = next((name for name in _STYLE_TO_DIALOG if style == getattr(
				engine.DocumentTextEditStyleV1, name,
			)), None)
			if name is None:
				raise ValueError("Text placement supports baseline, subscript, and superscript")
			styles.append(_STYLE_TO_DIALOG[name])
		runs.append((run.text, tuple(styles)))
	return FerrumTextPlacementDialogModel(tuple(runs), font_size, defaults.color)


#============================================
def runs_from_dialog(runs: tuple[tuple[str, tuple[str, ...]], ...]) -> tuple[object, ...]:
	"""Convert immutable plain dialog runs into exact Rust run DTOs."""
	if type(runs) is not tuple:
		raise TypeError("Ferrum Text placement requires immutable dialog runs")
	import ferrum_qt.ferrum.engine as engine
	converted = []
	for run in runs:
		if type(run) is not tuple or len(run) != 2:
			raise TypeError("Ferrum Text placement runs require text/style pairs")
		text, styles = run
		if type(text) is not str or type(styles) is not tuple:
			raise TypeError("Ferrum Text placement runs must be immutable plain values")
		converted_styles = []
		for style in styles:
			name = _STYLE_TO_RUST.get(style)
			if name is None:
				raise ValueError("Text placement supports baseline, subscript, and superscript")
			converted_styles.append(getattr(engine.DocumentTextEditStyleV1, name))
		converted.append(engine.DocumentTextEditRunV1.create(text, tuple(converted_styles)))
	return tuple(converted)


#============================================
def dialog_for_placement(model: FerrumTextPlacementDialogModel, parent: object) -> object:
	"""Build the constrained dialog without making it a persistent text authority."""
	if type(model) is not FerrumTextPlacementDialogModel:
		raise TypeError("Ferrum Text placement requires the closed dialog model")
	capabilities = ferrum_qt.dialogs.rich_text_dialog.RichTextDialogCapabilities(
		bold=False,
		italic=False,
		font_family=False,
		disabled_reason=(
			"The verified Ferrum renderer currently uses its regular Telex face only."
		),
	)
	return ferrum_qt.dialogs.rich_text_dialog.RichTextDialog(
		model.runs, model.font_size, model.color, parent,
		capabilities=capabilities, initial_text_selected=True,
	)
