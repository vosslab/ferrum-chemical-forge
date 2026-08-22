"""Closed outcomes for one window-owned selected-molecule SMARTS capture."""

# Standard Library
import dataclasses


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSmartsSelectedRootCaptureTarget:
	"""One current Ferrum tab and viewport eligible for a pointer capture."""

	tab: object
	viewport: object


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSmartsSelectedRootCaptureUnavailable:
	"""A capture cannot begin or continue because its document is not current."""

	message: str


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSmartsSelectedRootCaptureRejected:
	"""Rust rejected a current capture with its documented SMARTS failure."""

	error: Exception


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumSmartsSelectedRootCaptureAccepted:
	"""Rust minted one opaque selected-query token for the authenticated tab."""

	tab: object
	token: object
