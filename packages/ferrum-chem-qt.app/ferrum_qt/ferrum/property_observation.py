"""Current projection receipt for Ferrum read-only property clients."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.ferrum.document_tab_errors as native_document_tab_errors


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativePropertyObservation:
	"""One current Rust projection paired with its disposable Qt selection."""

	document: object
	selection: tuple[ferrum_qt.canvas.ferrum_render_projection.RenderTargetKey, ...]


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumLivePropertyObservationAvailable:
	"""Identify one current property observation safe for passive clients."""

	observation: FerrumNativePropertyObservation


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumLivePropertyObservationUnavailable:
	"""State why passive clients must show no document facts."""

	reason: str
	requires_refresh: bool


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumLivePropertyObservationStale:
	"""Request one authoritative refresh before passive observation resumes."""

	reason: str


#============================================
class FerrumNativePropertyObservationMixin:
	"""Expose immutable installed facts without granting an inspector authority."""

	#============================================
	def observe_properties(self) -> FerrumNativePropertyObservation:
		"""Return one receipt only when document and scene provenance still match."""
		self._require_live()
		self._require_current_projection()
		if self._document_observation is None:
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum tab has no installed document projection",
			)
		document = self._document_observation.projection
		projection = self._require_projection()
		if (
			document.revision != projection.revision
			or document.digest != projection.digest
		):
			raise native_document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum property observation provenance is inconsistent",
			)
		return FerrumNativePropertyObservation(document, projection.selected_targets())

	#============================================
	def resolve_live_property_observation(self) -> (
			FerrumLivePropertyObservationAvailable
			| FerrumLivePropertyObservationUnavailable
			| FerrumLivePropertyObservationStale
			):
		"""Classify passive property access without exposing stale Qt projection facts."""
		if self._disposed:
			return FerrumLivePropertyObservationUnavailable("tab_disposed", False)
		if self.requires_refresh:
			return FerrumLivePropertyObservationStale("authoritative_refresh_required")
		try:
			return FerrumLivePropertyObservationAvailable(self.observe_properties())
		except native_document_tab_errors.FerrumNativeDocumentTabError:
			return FerrumLivePropertyObservationStale("projection_observation_inconsistent")
