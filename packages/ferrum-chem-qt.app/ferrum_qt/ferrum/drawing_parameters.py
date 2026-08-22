"""Application-owned next-operation choices for Ferrum drawing tools."""

# Standard Library
import dataclasses

# PIP3 modules
import PySide6.QtCore


_DEFAULT_ELEMENT = "C"
_DEFAULT_ORDER = "single"
_ELEMENT_KEY = "drawing/next_atom_element"
_ORDER_KEY = "drawing/next_bond_order"


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeDrawingParametersSnapshot:
	"""Frozen user choices carried by one future Ferrum authoring gesture."""

	element: str
	order_name: str

	#============================================
	def bond_order(self) -> object:
		"""Convert the closed preference spelling at the private PyO3 boundary."""
		import ferrum_qt.ferrum.engine as engine
		if self.order_name == "single":
			return engine.DocumentBondOrderV1.single
		if self.order_name == "double":
			return engine.DocumentBondOrderV1.double
		if self.order_name == "triple":
			return engine.DocumentBondOrderV1.triple
		raise ValueError("Ferrum drawing parameters contain an unknown bond order")

	#============================================
	def bond_presentation(self) -> object:
		"""Convert frozen next-drawing choices at the private PyO3 boundary."""
		import ferrum_qt.ferrum.engine as engine
		presentations = engine.DocumentBondPresentationV1
		if self.order_name == "single":
			return presentations.normal_single
		if self.order_name == "double":
			return presentations.normal_double
		if self.order_name == "triple":
			return presentations.normal_triple
		raise ValueError("Ferrum drawing parameters contain an unknown presentation")

#============================================
def normalize_element(value: object) -> str | None:
	"""Return a valid plain atom name while retaining supported pseudo names."""
	if type(value) is not str:
		return None
	trimmed = value.strip()
	if not trimmed or not trimmed.isascii() or not trimmed.isalpha():
		return None
	return _periodic_spellings_by_casefold().get(trimmed.casefold(), trimmed)


#============================================
def _periodic_spellings_by_casefold() -> dict[str, str]:
	"""Read conventional picker spelling from the authoritative Rust display list."""
	import ferrum_qt.ferrum.engine as engine
	return {
		entry.symbol.casefold(): entry.symbol
		for entry in engine.periodic_display_entries_v1()
	}


#============================================
class FerrumNativeDrawingParameters(PySide6.QtCore.QObject):
	"""Store personal next-drawing choices outside Rust documents and CDML."""

	changed = PySide6.QtCore.Signal()
	_shared_application_owner: object | None = None
	_shared_application_model: "FerrumNativeDrawingParameters | None" = None

	#============================================
	def __init__(self, preferences: object | None = None,
			parent: PySide6.QtCore.QObject | None = None) -> None:
		"""Read QSettings-backed choices or provide the standalone-host defaults."""
		super().__init__(parent)
		self._preferences = preferences
		element = normalize_element(self._read(_ELEMENT_KEY, _DEFAULT_ELEMENT))
		self._element = _DEFAULT_ELEMENT if element is None else element
		order = self._read(_ORDER_KEY, _DEFAULT_ORDER)
		self._order_name = order if order in ("single", "double", "triple") else _DEFAULT_ORDER

	#============================================
	@classmethod
	def shared_application_model(cls, preferences: object) -> "FerrumNativeDrawingParameters":
		"""Return the one observable model for an ordinary application's Preferences."""
		if cls._shared_application_owner is not preferences:
			cls._shared_application_owner = preferences
			cls._shared_application_model = cls(preferences)
		if cls._shared_application_model is None:
			raise RuntimeError("Ferrum drawing parameters have no shared application model")
		return cls._shared_application_model

	#============================================
	def _read(self, key: str, default: object) -> object:
		"""Read one optional application preference without inventing document state."""
		if self._preferences is None:
			return default
		return self._preferences.value(key, default)

	#============================================
	def snapshot(self) -> FerrumNativeDrawingParametersSnapshot:
		"""Capture the effective values for one authoring intent."""
		return FerrumNativeDrawingParametersSnapshot(self._element, self._order_name)

	#============================================
	def set_element(self, value: object) -> bool:
		"""Accept one valid atom name and persist it as a user workflow preference."""
		normalized = normalize_element(value)
		if normalized is None:
			return False
		self._element = normalized
		self._persist(_ELEMENT_KEY, normalized)
		self.changed.emit()
		return True

	#============================================
	def set_order_name(self, value: object) -> bool:
		"""Accept one closed bond-order spelling and retain the valid current choice."""
		if value not in ("single", "double", "triple"):
			return False
		self._order_name = value
		self._persist(_ORDER_KEY, value)
		self.changed.emit()
		return True

	#============================================
	def _persist(self, key: str, value: str) -> None:
		"""Write only an ordinary valid user choice through the configured owner."""
		if self._preferences is not None:
			self._preferences.set_value(key, value)


#============================================
def common_elements() -> tuple[str, ...]:
	"""Return the authoritative periodic picker suggestions for Ferrum drawing."""
	import ferrum_qt.ferrum.engine as engine
	return tuple(entry.symbol for entry in engine.periodic_display_entries_v1())
