"""Behavioral contract for the Qt-local operation lease registry."""

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.ferrum.operation_leases


#============================================
class _LiveTab:
	"""Minimal registered-tab stand-in with explicit disposal state."""

	def __init__(self) -> None:
		"""Start as a live tab."""
		self.is_disposed = False


#============================================
def _catalog_owner(
		) -> tuple[
			ferrum_qt.ferrum.operation_leases.OperationLeaseRegistry,
			ferrum_qt.ferrum.operation_leases.LeaseOwnerCapability,
			]:
	"""Return an isolated registry and its catalog controller capability."""
	registry = ferrum_qt.ferrum.operation_leases.OperationLeaseRegistry()
	capability = registry.register_family(
		ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG,
	)
	return registry, capability


#============================================
def test_registered_tabs_are_exact_operation_owners() -> None:
	"""A lease on one registered tab does not become active for another tab."""
	registry, capability = _catalog_owner()
	owner_tab = _LiveTab()
	other_tab = _LiveTab()
	registry.bind_tab(owner_tab)
	registry.bind_tab(other_tab)
	lease = registry.acquire(
		capability,
		tab=owner_tab,
		close_policy=(
			ferrum_qt.ferrum.operation_leases.ClosePolicy.
			CANCEL_AND_BLOCK_TAB_CLOSE
		),
	)
	assert registry.active_for_tab(owner_tab) == (lease,)
	assert registry.active_for_tab(other_tab) == ()


#============================================
def test_capability_cannot_settle_another_family_lease() -> None:
	"""A controller capability cannot mutate another family's operation."""
	registry, catalog_capability = _catalog_owner()
	open_capability = registry.register_family(
		ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN,
	)
	tab = _LiveTab()
	registry.bind_tab(tab)
	lease = registry.acquire(
		open_capability,
		tab=tab,
		close_policy=(
			ferrum_qt.ferrum.operation_leases.ClosePolicy.BLOCK_UNTIL_SETTLED
		),
	)
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.settle(
			catalog_capability, lease,
			ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLED,
		)
	assert registry.has_active(
		ferrum_qt.ferrum.operation_leases.OperationFamily.LOCAL_DOCUMENT_OPEN,
		tab,
	)


#============================================
def test_duplicate_unregistered_or_disposed_acquisition_is_refused() -> None:
	"""A family cannot admit a duplicate, unbound, or disposed tab lease."""
	registry, capability = _catalog_owner()
	bound_tab = _LiveTab()
	unbound_tab = _LiveTab()
	disposed_tab = _LiveTab()
	registry.bind_tab(bound_tab)
	registry.bind_tab(disposed_tab)
	registry.acquire(
		capability,
		tab=bound_tab,
		close_policy=(
			ferrum_qt.ferrum.operation_leases.ClosePolicy.
			CANCEL_AND_BLOCK_TAB_CLOSE
		),
	)
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.acquire(
			capability,
			tab=bound_tab,
			close_policy=(
				ferrum_qt.ferrum.operation_leases.ClosePolicy.
				CANCEL_AND_BLOCK_TAB_CLOSE
			),
		)
	disposed_tab.is_disposed = True
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.acquire(
			capability,
			tab=disposed_tab,
			close_policy=(
				ferrum_qt.ferrum.operation_leases.ClosePolicy.
				CANCEL_AND_BLOCK_TAB_CLOSE
			),
		)
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.acquire(
			capability,
			tab=unbound_tab,
			close_policy=(
				ferrum_qt.ferrum.operation_leases.ClosePolicy.
				CANCEL_AND_BLOCK_TAB_CLOSE
			),
		)


#============================================
def test_cancellation_request_is_idempotent_until_terminal_settlement() -> None:
	"""Repeated cancellation remains one cancellation-requested operation."""
	registry, capability = _catalog_owner()
	tab = _LiveTab()
	registry.bind_tab(tab)
	lease = registry.acquire(
		capability,
		tab=tab,
		close_policy=(
			ferrum_qt.ferrum.operation_leases.ClosePolicy.
			CANCEL_AND_BLOCK_TAB_CLOSE
		),
	)
	first = registry.request_cancellation(capability, lease, "user-cancel")
	second = registry.request_cancellation(capability, lease, "user-cancel")
	assert first == second
	assert registry.has_active(
		ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG,
		tab,
	)


#============================================
def test_terminal_settlement_retires_operation_and_is_one_way() -> None:
	"""A settled operation frees its tab and cannot be settled again."""
	registry, capability = _catalog_owner()
	tab = _LiveTab()
	registry.bind_tab(tab)
	lease = registry.acquire(
		capability,
		tab=tab,
		close_policy=(
			ferrum_qt.ferrum.operation_leases.ClosePolicy.
			CANCEL_AND_BLOCK_TAB_CLOSE
		),
	)
	settled = registry.settle(
		capability, lease,
		ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED,
	)
	assert settled.state is ferrum_qt.ferrum.operation_leases.LeaseState.COMPLETED
	assert not registry.has_active(
		ferrum_qt.ferrum.operation_leases.OperationFamily.TEMPLATE_CATALOG,
		tab,
	)
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.settle(
			capability, lease,
			ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLED,
		)


#============================================
def test_active_tab_cannot_unregister_until_cancelled_operation_settles() -> None:
	"""Tab teardown waits for its cancelling lease to reach a terminal state."""
	registry, capability = _catalog_owner()
	tab = _LiveTab()
	registry.bind_tab(tab)
	lease = registry.acquire(
		capability,
		tab=tab,
		close_policy=(
			ferrum_qt.ferrum.operation_leases.ClosePolicy.
			CANCEL_AND_BLOCK_TAB_CLOSE
		),
	)
	registry.request_cancellation(capability, lease, "tab-close")
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.unregister_tab(tab)
	registry.settle(
		capability, lease,
		ferrum_qt.ferrum.operation_leases.LeaseState.CANCELLED,
	)
	registry.unregister_tab(tab)
	with pytest.raises(ferrum_qt.ferrum.operation_leases.OperationLeaseError):
		registry.acquire(
			capability,
			tab=tab,
			close_policy=(
				ferrum_qt.ferrum.operation_leases.ClosePolicy.
				CANCEL_AND_BLOCK_TAB_CLOSE
			),
		)
