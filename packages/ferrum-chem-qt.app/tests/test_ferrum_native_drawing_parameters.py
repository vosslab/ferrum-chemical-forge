"""Behavior coverage for application-owned native drawing choices."""

# local repo modules
import ferrum_qt.native.ferrum_native_drawing_parameters


#============================================
class _ValueStore:
	"""Minimal value-store seam representing the product preference boundary."""

	#============================================
	def __init__(self) -> None:
		"""Start with no persisted application choices."""
		self.values = {}

	#============================================
	def value(self, key: str, default: object) -> object:
		"""Return one stored value or the caller's ordinary product default."""
		return self.values.get(key, default)

	#============================================
	def set_value(self, key: str, value: object) -> None:
		"""Persist one accepted preference value through the store seam."""
		self.values[key] = value


#============================================
def test_valid_next_drawing_choices_round_trip_through_application_store() -> None:
	"""A completed choice returns with conventional element spelling after recreation."""
	store = _ValueStore()
	parameters = ferrum_qt.native.ferrum_native_drawing_parameters.FerrumNativeDrawingParameters(
		store,
	)
	parameters.set_element("cL")
	parameters.set_order_name("triple")
	recreated = ferrum_qt.native.ferrum_native_drawing_parameters.FerrumNativeDrawingParameters(
		store,
	)
	assert recreated.snapshot() == (
		ferrum_qt.native.ferrum_native_drawing_parameters.
		FerrumNativeDrawingParametersSnapshot("Cl", "triple", "normal")
	)


#============================================
def test_invalid_next_drawing_choices_keep_last_effective_choice() -> None:
	"""An unfinished invalid edit leaves the next authoring operation unchanged."""
	parameters = ferrum_qt.native.ferrum_native_drawing_parameters.FerrumNativeDrawingParameters(
		_ValueStore(),
	)
	parameters.set_element("N")
	parameters.set_order_name("double")
	parameters.set_element("N2")
	parameters.set_order_name("aromatic")
	assert parameters.snapshot() == (
		ferrum_qt.native.ferrum_native_drawing_parameters.
		FerrumNativeDrawingParametersSnapshot("N", "double", "normal")
	)
