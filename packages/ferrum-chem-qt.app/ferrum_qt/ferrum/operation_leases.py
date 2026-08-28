"""Qt-local ownership for tab-bound operation lifecycles.

This registry deliberately owns only application lifecycle identity and state.
Feature controllers retain their own payloads and invoke Rust-backed document
operations through their explicit tab ports.
"""

# Standard Library
import dataclasses
import enum


#============================================
class OperationFamily(enum.Enum):
	"""Name the closed set of controller-owned operation families."""

	TEMPLATE_CATALOG = "template_catalog"
	LOCAL_DOCUMENT_OPEN = "local_document_open"


#============================================
class ClosePolicy(enum.Enum):
	"""State how a tab-close coordinator treats one active operation."""

	CANCEL_AND_BLOCK_TAB_CLOSE = "cancel_and_block_tab_close"
	BLOCK_UNTIL_SETTLED = "block_until_settled"


#============================================
class LeaseState(enum.Enum):
	"""Describe the closed lifecycle state of one admitted operation."""

	ACTIVE = "active"
	CANCELLATION_REQUESTED = "cancellation_requested"
	COMPLETED = "completed"
	REFUSED = "refused"
	FAILED = "failed"
	CANCELLED = "cancelled"


_TERMINAL_STATES = frozenset({
	LeaseState.COMPLETED,
	LeaseState.REFUSED,
	LeaseState.FAILED,
	LeaseState.CANCELLED,
})


#============================================
class OperationLeaseError(RuntimeError):
	"""Report a typed violation of the Qt operation-lifecycle contract."""


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class OperationLeaseId:
	"""Identify one registry-local operation admission."""

	family: OperationFamily
	sequence: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class TabLeaseIdentity:
	"""Identify one exact tab object during its registry registration lifetime."""

	sequence: int


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LeaseOwnerCapability:
	"""Carry the opaque authority for one registered operation family."""

	family: OperationFamily
	_registry_token: object = dataclasses.field(repr=False, compare=False)


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class OperationLease:
	"""Expose immutable lifecycle facts for one exact tab-bound operation."""

	lease_id: OperationLeaseId
	tab_identity: TabLeaseIdentity
	close_policy: ClosePolicy
	state: LeaseState
	_tab: object = dataclasses.field(repr=False, compare=False)

	#============================================
	@property
	def family(self) -> OperationFamily:
		"""Return the controller family that admitted this lease."""
		family = self.lease_id.family
		return family

	#============================================
	def tab(self) -> object:
		"""Return the exact registered tab object retained by this lease."""
		tab = self._tab
		return tab


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class PreparedTerminalReplacement:
	"""Carry one registry-validated terminal replacement to its closed commit."""

	lease_id: OperationLeaseId
	tab_identity: TabLeaseIdentity
	_token: object = dataclasses.field(repr=False, compare=False)


#============================================
@dataclasses.dataclass(slots=True)
class _BoundTab:
	"""Keep the exact tab object paired with its opaque registration identity."""

	identity: TabLeaseIdentity
	tab: object


#============================================
@dataclasses.dataclass(slots=True)
class _LeaseRecord:
	"""Keep the registry-owned mutable lifecycle state outside lease snapshots."""

	lease_id: OperationLeaseId
	tab_identity: TabLeaseIdentity
	close_policy: ClosePolicy
	state: LeaseState
	tab: object


#============================================
class OperationLeaseRegistry:
	"""Own lifecycle state for one main window's exact registered tabs."""

	#============================================
	def __init__(self) -> None:
		"""Create an empty registry with no tab or controller registrations."""
		self._bound_tabs: list[_BoundTab] = []
		self._capabilities: dict[OperationFamily, LeaseOwnerCapability] = {}
		self._leases: dict[OperationLeaseId, _LeaseRecord] = {}
		self._active_by_family_tab: dict[
				 tuple[OperationFamily, TabLeaseIdentity], OperationLeaseId,
		] = {}
		self._prepared_terminal_replacements: dict[object, _LeaseRecord] = {}
		self._next_tab_sequence = 1
		self._next_lease_sequence = 1

	#============================================
	def register_family(self, family: OperationFamily) -> LeaseOwnerCapability:
		"""Issue the one opaque lifecycle authority for ``family``."""
		self._require_family(family)
		if family in self._capabilities:
			raise OperationLeaseError(
				f"Ferrum operation family {family.value} is already registered",
			)
		capability = LeaseOwnerCapability(family, object())
		self._capabilities[family] = capability
		return capability

	#============================================
	def bind_tab(self, tab: object) -> TabLeaseIdentity:
		"""Bind one exact tab object for lifecycle ownership."""
		if tab is None:
			raise OperationLeaseError("Ferrum cannot bind a missing document tab")
		if self._tab_is_disposed(tab):
			raise OperationLeaseError("Ferrum cannot bind a disposed document tab")
		bound = self._bound_tab_for(tab)
		if bound is not None:
			return bound.identity
		identity = TabLeaseIdentity(self._next_tab_sequence)
		self._next_tab_sequence += 1
		self._bound_tabs.append(_BoundTab(identity, tab))
		return identity

	#============================================
	def acquire(
			self, capability: LeaseOwnerCapability, *, tab: object,
			close_policy: ClosePolicy,
			) -> OperationLease:
		"""Admit the capability's one active lease for an exact registered tab."""
		family = self._family_for_capability(capability)
		self._require_close_policy(close_policy)
		bound = self._bound_tab_for(tab)
		if bound is None:
			raise OperationLeaseError("Ferrum cannot acquire a lease for an unregistered tab")
		if self._tab_is_disposed(bound.tab):
			raise OperationLeaseError("Ferrum cannot acquire a lease for a disposed tab")
		active_key = (family, bound.identity)
		if active_key in self._active_by_family_tab:
			raise OperationLeaseError(
				"Ferrum already has an active lease for this family and exact tab",
			)
		lease_id = OperationLeaseId(family, self._next_lease_sequence)
		self._next_lease_sequence += 1
		record = _LeaseRecord(
			lease_id, bound.identity, close_policy, LeaseState.ACTIVE, bound.tab,
		)
		self._leases[lease_id] = record
		self._active_by_family_tab[active_key] = lease_id
		lease = self._lease_snapshot(record)
		return lease

	#============================================
	def active_for_tab(self, tab: object) -> tuple[OperationLease, ...]:
		"""Return active lease snapshots owned by this exact registered tab."""
		bound = self._bound_tab_for(tab)
		if bound is None:
			return ()
		leases = tuple(
			self._lease_snapshot(self._leases[lease_id])
			for (family, identity), lease_id in self._active_by_family_tab.items()
			if identity == bound.identity
		)
		return leases

	#============================================
	def has_active(
			self, family: OperationFamily, tab: object | None = None,
			) -> bool:
		"""Return whether a family has active lifecycle ownership in this window."""
		self._require_family(family)
		if tab is None:
			is_active = any(
				active_family is family
				for active_family, identity in self._active_by_family_tab
			)
			return is_active
		bound = self._bound_tab_for(tab)
		if bound is None:
			return False
		is_active = (family, bound.identity) in self._active_by_family_tab
		return is_active

	#============================================
	def request_cancellation(
			self, capability: LeaseOwnerCapability, lease: OperationLease,
			reason: object,
			) -> OperationLease:
		"""Record one idempotent cancellation request without performing cleanup."""
		del reason
		record = self._record_for_capability(capability, lease)
		if record.state is LeaseState.ACTIVE:
			record.state = LeaseState.CANCELLATION_REQUESTED
		elif record.state is not LeaseState.CANCELLATION_REQUESTED:
			raise OperationLeaseError("Ferrum cannot cancel a terminal operation lease")
		updated_lease = self._lease_snapshot(record)
		return updated_lease

	#============================================
	def settle(
			self, capability: LeaseOwnerCapability, lease: OperationLease,
			terminal_state: LeaseState,
			) -> OperationLease:
		"""Terminally settle one active lease and retire its active index entry."""
		self._require_terminal_state(terminal_state)
		record = self._record_for_capability(capability, lease)
		if record.state not in (LeaseState.ACTIVE, LeaseState.CANCELLATION_REQUESTED):
			raise OperationLeaseError("Ferrum cannot settle an already terminal operation lease")
		record.state = terminal_state
		active_key = (record.lease_id.family, record.tab_identity)
		del self._active_by_family_tab[active_key]
		settled_lease = self._lease_snapshot(record)
		# The returned snapshot is the controller's one callback outcome.  The
		# registry must not retain terminal tab ownership after that outcome.
		del self._leases[record.lease_id]
		return settled_lease

	#============================================
	def unregister_tab(self, tab: object) -> None:
		"""Retire a tab registration only after all of its leases are terminal."""
		bound = self._bound_tab_for(tab)
		if bound is None:
			raise OperationLeaseError("Ferrum cannot unregister a tab that is not bound")
		if any(identity == bound.identity for family, identity in self._active_by_family_tab):
			raise OperationLeaseError("Ferrum cannot unregister a tab with an active lease")
		self._bound_tabs.remove(bound)

	#============================================
	def prepare_terminal_replacement(
			self, capability: LeaseOwnerCapability, lease: OperationLease,
			source_tab: object,
			) -> PreparedTerminalReplacement:
		"""Validate and detach one source before irreversible tab disposal."""
		record = self._record_for_capability(capability, lease)
		if record.tab is not source_tab or record.state is not LeaseState.ACTIVE:
			raise OperationLeaseError("Ferrum replacement requires its active exact source lease")
		bound = self._bound_tab_for(source_tab)
		if bound is None or bound.identity != record.tab_identity:
			raise OperationLeaseError("Ferrum replacement source is not bound to its lease")
		self._bound_tabs.remove(bound)
		token = object()
		self._prepared_terminal_replacements[token] = record
		prepared = PreparedTerminalReplacement(record.lease_id, bound.identity, token)
		return prepared

	#============================================
	def restore_prepared_terminal_replacement(
			self, prepared: PreparedTerminalReplacement, source_tab: object,
			) -> None:
		"""Restore one refused terminal replacement before source disposal."""
		record = self._prepared_record(prepared)
		if record.tab is not source_tab:
			raise OperationLeaseError("Ferrum replacement cannot restore another source lease")
		if self._bound_tab_for(source_tab) is not None:
			raise OperationLeaseError("Ferrum replacement source is already bound")
		self._bound_tabs.append(_BoundTab(record.tab_identity, source_tab))
		del self._prepared_terminal_replacements[prepared._token]

	#============================================
	def complete_prepared_terminal_replacement(
			self, prepared: PreparedTerminalReplacement,
			) -> OperationLease:
		"""Close one prepared replacement through the registry's private mutation."""
		record = self._prepared_terminal_replacements.pop(prepared._token)
		record.state = LeaseState.COMPLETED
		active_key = (record.lease_id.family, record.tab_identity)
		del self._active_by_family_tab[active_key]
		settled = self._lease_snapshot(record)
		del self._leases[record.lease_id]
		return settled

	#============================================
	def restore_detached_source_for_terminal_replacement(
			self, capability: LeaseOwnerCapability, lease: OperationLease,
			old_tab: object, identity: TabLeaseIdentity,
			) -> None:
		"""Restore a failed replacement source with its original tab identity."""
		record = self._record_for_capability(capability, lease)
		if record.tab is not old_tab or record.tab_identity != identity:
			raise OperationLeaseError("Ferrum replacement cannot restore another source lease")
		if self._bound_tab_for(old_tab) is not None:
			raise OperationLeaseError("Ferrum replacement source is already bound")
		self._bound_tabs.append(_BoundTab(identity, old_tab))

	#============================================
	def _bound_tab_for(self, tab: object) -> _BoundTab | None:
		"""Find a registration by exact object identity, never equality or tab title."""
		for bound in self._bound_tabs:
			if bound.tab is tab:
				return bound
		return None

	#============================================
	def _prepared_record(self, prepared: PreparedTerminalReplacement) -> _LeaseRecord:
		"""Validate one opaque prepared replacement before recoverable rollback."""
		if type(prepared) is not PreparedTerminalReplacement:
			raise OperationLeaseError("Ferrum replacement requires a prepared terminal token")
		record = self._prepared_terminal_replacements.get(prepared._token)
		if record is None:
			raise OperationLeaseError("Ferrum replacement terminal token is no longer active")
		if record.lease_id != prepared.lease_id or record.tab_identity != prepared.tab_identity:
			raise OperationLeaseError("Ferrum replacement terminal token does not match its lease")
		return record

	#============================================
	def _tab_is_disposed(self, tab: object) -> bool:
		"""Read the tab's existing disposal authority without retaining its state."""
		if not hasattr(tab, "is_disposed"):
			return False
		is_disposed = tab.is_disposed
		return is_disposed

	#============================================
	def _family_for_capability(
			self, capability: LeaseOwnerCapability,
			) -> OperationFamily:
		"""Verify that an opaque capability was issued by this registry."""
		if type(capability) is not LeaseOwnerCapability:
			raise OperationLeaseError("Ferrum operation ownership requires a lease capability")
		registered = self._capabilities.get(capability.family)
		if registered is None or registered._registry_token is not capability._registry_token:
			raise OperationLeaseError("Ferrum operation capability does not belong to this registry")
		family = capability.family
		return family

	#============================================
	def _record_for_capability(
			self, capability: LeaseOwnerCapability, lease: OperationLease,
			) -> _LeaseRecord:
		"""Verify exact owner authority and locate the one registry-owned record."""
		family = self._family_for_capability(capability)
		if type(lease) is not OperationLease:
			raise OperationLeaseError("Ferrum operation lifecycle requires an operation lease")
		if lease.family is not family:
			raise OperationLeaseError("Ferrum operation capability cannot control another family")
		record = self._leases.get(lease.lease_id)
		if record is None:
			raise OperationLeaseError("Ferrum operation lease does not belong to this registry")
		if record.tab is not lease.tab() or record.tab_identity != lease.tab_identity:
			raise OperationLeaseError("Ferrum operation lease does not retain its registered tab")
		return record

	#============================================
	def _lease_snapshot(self, record: _LeaseRecord) -> OperationLease:
		"""Create immutable read-only lifecycle facts from registry-owned state."""
		lease = OperationLease(
			record.lease_id, record.tab_identity, record.close_policy,
			record.state, record.tab,
		)
		return lease

	#============================================
	def _require_family(self, family: OperationFamily) -> None:
		"""Refuse values outside the closed operation-family model."""
		if type(family) is not OperationFamily:
			raise OperationLeaseError("Ferrum operation family must be an OperationFamily")

	#============================================
	def _require_close_policy(self, close_policy: ClosePolicy) -> None:
		"""Refuse values outside the closed tab-close policy model."""
		if type(close_policy) is not ClosePolicy:
			raise OperationLeaseError("Ferrum operation close policy must be a ClosePolicy")

	#============================================
	def _require_terminal_state(self, terminal_state: LeaseState) -> None:
		"""Refuse nonterminal lifecycle states at the terminal settlement boundary."""
		if type(terminal_state) is not LeaseState or terminal_state not in _TERMINAL_STATES:
			raise OperationLeaseError("Ferrum operation lease settlement requires a terminal state")
