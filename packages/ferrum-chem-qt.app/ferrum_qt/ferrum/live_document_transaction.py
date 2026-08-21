"""Transient live-document query retirement owned by one Ferrum tab."""

# Standard Library
import collections.abc
import dataclasses


#============================================
@dataclasses.dataclass(frozen=True)
class _LiveSmartsSelectedAvailabilityV1:
	"""Copied selected-query availability with no root or selector facts."""

	available: bool
	recovery: str

# local repo modules
import ferrum_qt.ferrum.document_tab_errors


#============================================
class _RetiringDocumentSessionV1:
	"""Forward one Rust session while fencing mutation before it begins.

	This is deliberately a Qt ownership seam, not a second document model.  It
	knows only which binding method starts a document-changing operation and asks
	the owning tab to retire its temporary renderer-issued paint first.
	"""

	_MUTATING_PREFIXES = ("apply_", "commit_", "create_")
	_MUTATING_NAMES = frozenset((
		"submit",
		"undo",
		"redo",
		"save_atomic",
		"set_document_molecule_name_v1",
		"convert_linear_form_v1",
	))
	_REPROJECTION_NAMES = frozenset(("observe_render",))

	#============================================
	def __init__(self, tab: object, session: object) -> None:
		"""Keep the only underlying Rust session without exporting it to Qt tools."""
		self._tab = tab
		self._session = session

	#============================================
	def __getattr__(self, name: str) -> object:
		"""Wrap only closed document-transition calls; all reads pass through."""
		if name in self._REPROJECTION_NAMES:
			return self._reprojection_call()
		method = getattr(self._session, name)
		if not isinstance(name, str) or not callable(method):
			return method
		if name in self._MUTATING_NAMES or name.startswith(self._MUTATING_PREFIXES):
			return self._mutation_call(method)
		return method

	#============================================
	def _mutation_call(self, method: collections.abc.Callable[..., object]) -> object:
		"""Return one call which retires before the Rust document can change."""
		def call(*args: object, **kwargs: object) -> object:
			return self._tab._retire_then_mutate_document_v1(method, *args, **kwargs)
		return call

	#============================================
	def _reprojection_call(self) -> object:
		"""Route legacy observation callers through the live-plan transaction."""
		def call(*args: object, **kwargs: object) -> object:
			return self._tab._publish_live_render_plan_v1(*args, **kwargs)
		return call


#============================================
class FerrumLiveDocumentTransactionMixin:
	"""Own one disposable live-query overlay and all document transition fences."""

	_FULL_RETIREMENT_REASONS = frozenset((
		"construction_failure",
		"document_mutation",
		"document_reprojection",
		"tab_deactivated",
		"tab_disposed",
	))
	_RECEIPT_RETIREMENT_REASONS = frozenset((
		"dock_rerun",
		"stale_delivery",
	))

	#============================================
	def _initialize_live_document_transaction_v1(self, session: object) -> None:
		"""Install the sole tab-local owner around one already-created Rust session."""
		self._live_document_session_v1 = session
		self._live_smarts_overlay_item_v1: object | None = None
		self._live_smarts_receipt_v1: object | None = None
		self._live_smarts_run_token_v1 = 0
		self._live_smarts_active_run_token_v1: int | None = None
		self._live_smarts_retirement_error_v1: Exception | None = None
		self._live_smarts_retirement_available_v1 = True
		self._live_smarts_invalidation_callback_v1: collections.abc.Callable[[], None] | None = None
		self._session = _RetiringDocumentSessionV1(self, session)

	#============================================
	def _bind_live_smarts_invalidation_callback_v1(self,
			callback: collections.abc.Callable[[], None] | None) -> None:
		"""Bind the one active dock to closed document-transition notification."""
		if callback is not None and not callable(callback):
			raise TypeError("Ferrum live SMARTS invalidation callback must be callable")
		self._live_smarts_invalidation_callback_v1 = callback

	#============================================
	def _notify_live_smarts_invalidation_v1(self) -> None:
		"""Clear copied dock state after the tab has retired native query state."""
		callback = self._live_smarts_invalidation_callback_v1
		if callback is not None:
			callback()

	#============================================
	def _begin_live_smarts_query_run_v1(self) -> None:
		"""Retire an older installed run before the dock dispatches a new query.

		This is the only rerun transition.  It belongs before the private bridge
		issues a new receipt, never in the first-paint installer: native receipts
		carry independently redeemable result rows, so retiring during that commit
		would invalidate the rows the dock has not shown yet.
		"""
		self._require_live()
		if (
			self._live_smarts_overlay_item_v1 is None
			and self._live_smarts_receipt_v1 is None
			and self._live_smarts_active_run_token_v1 is None
		):
			return
		self._require_live_smarts_receipt_retirement_v1("dock_rerun")

	#============================================
	def _live_smarts_selected_query_availability_v1(self,
			selection_provider: collections.abc.Callable[[], object | None],
			) -> _LiveSmartsSelectedAvailabilityV1:
		"""Return a copied, non-identifying selected-query readiness state.

		The structural-selection owner supplies its existing transient selection only
		for this synchronous check.  The dock receives neither that object nor any
		durable/root facts, and native admission still repeats the validation.
		"""
		if self._disposed or self.requires_refresh:
			return _LiveSmartsSelectedAvailabilityV1(False, "document_not_ready")
		try:
			selection = selection_provider()
			targets = () if selection is None else tuple(selection.targets)
			if len(targets) == 0:
				return _LiveSmartsSelectedAvailabilityV1(False, "select_one_molecule")
			if len(targets) != 1:
				return _LiveSmartsSelectedAvailabilityV1(False, "select_one_molecule")
			if targets[0].kind != "molecule":
				return _LiveSmartsSelectedAvailabilityV1(False, "select_one_molecule")
			return _LiveSmartsSelectedAvailabilityV1(True, "available")
		except Exception:
			return _LiveSmartsSelectedAvailabilityV1(False, "unavailable")
		finally:
			selection = None

	#============================================
	def _run_live_smarts_selected_query_v1(self,
			selection_provider: collections.abc.Callable[[], object | None],
			per_molecule_limit: int, total_limit: int) -> object:
		"""Capture and consume the selected-query token in one tab-private call."""
		self._require_live()
		selection = None
		selected_query = None
		try:
			selection = selection_provider()
			capture = getattr(
				self._live_document_session_v1,
				"_capture_live_document_smarts_selected_query_v1",
			)
			run = getattr(self._live_document_session_v1,
				"_run_live_document_smarts_query_v1")
			if not callable(capture) or not callable(run):
				raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
					"Ferrum live SMARTS selected query is unavailable; refresh before editing.",
				)
			selected_query = capture(selection)
			return run(selected_query, per_molecule_limit, total_limit)
		finally:
			# Neither the generic selection nor the opaque token may survive this
			# boundary in a dock field, queued callback, or tab state.
			selected_query = None
			selection = None

	#============================================
	def _install_live_smarts_query_overlay_v1(self, item: object, receipt: object) -> int:
		"""Commit the first paint for a freshly issued run without native retirement."""
		if item is None or receipt is None:
			raise TypeError("Ferrum live SMARTS paint requires an item and opaque receipt")
		self._require_live()
		if not self._live_smarts_retirement_available_v1:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum live SMARTS retirement is unavailable; refresh before editing.",
			)
		if (
			self._live_smarts_overlay_item_v1 is not None
			or self._live_smarts_receipt_v1 is not None
			or self._live_smarts_active_run_token_v1 is not None
		):
			# A caller skipped the sole pre-dispatch rerun fence.  The candidate has
			# already been issued natively, so preserve neither it nor the prior run.
			self._remove_rejected_live_smarts_overlay_item_v1(item)
			self._require_live_smarts_receipt_retirement_v1("stale_delivery")
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum cannot install a SMARTS run over an active run. Run the query again.",
			)
		self._live_smarts_run_token_v1 += 1
		self._live_smarts_overlay_item_v1 = item
		self._live_smarts_receipt_v1 = receipt
		self._live_smarts_active_run_token_v1 = self._live_smarts_run_token_v1
		return self._live_smarts_run_token_v1

	#============================================
	def _replace_live_smarts_query_overlay_v1(self, item: object) -> None:
		"""Swap transient renderer paint without redeeming or retiring the live run.

		The caller has already redeemed one unconsumed result row through the
		private native bridge and built this unattached renderer-issued item. This
		Qt-only transaction deliberately receives neither the receipt nor row
		identity: successful replacement keeps both the current opaque receipt and
		run token so a different unconsumed row can later be activated.
		"""
		self._require_live()
		if self._live_smarts_receipt_v1 is None:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum has no live SMARTS query to show.",
			)
		if self._live_smarts_active_run_token_v1 is None:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum has no live SMARTS query to show.",
			)
		if not self._live_smarts_retirement_available_v1:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum live SMARTS retirement is unavailable; refresh before editing.",
			)
		if item is None:
			raise TypeError("Ferrum live SMARTS replacement requires a renderer item")

		old_item = self._live_smarts_overlay_item_v1
		old_scene = None if old_item is None else old_item.scene()
		scene = old_scene
		if scene is None:
			scene = self._view.scene()
		if scene is None:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum cannot show a SMARTS match without a current canvas.",
			)
		if item.scene() is not None:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum cannot show a SMARTS match from another canvas.",
			)

		removed_old_item = False
		try:
			if old_item is not None and old_scene is not None:
				old_scene.removeItem(old_item)
				old_item.setParentItem(None)
				removed_old_item = True
			self._attach_live_smarts_overlay_item_v1(scene, item)
		except Exception as exc:
			self._remove_rejected_live_smarts_overlay_item_v1(item)
			restored = True
			if removed_old_item:
				restored = self._restore_replaced_live_smarts_overlay_item_v1(
					old_scene, old_item,
				)
			if not restored:
				# The previous paint no longer has an owner. Its receipt must not
				# outlive that visual, so retire the complete native transaction.
				# A native retirement failure retains its normal fail-closed state.
				self._require_live_smarts_receipt_retirement_v1("stale_delivery")
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum could not show that SMARTS match. Run the query again.",
			) from exc
		self._live_smarts_overlay_item_v1 = item

	#============================================
	def _clear_live_smarts_query_overlay_v1(self) -> bool:
		"""Remove only transient paint while preserving the live native receipt."""
		item = self._live_smarts_overlay_item_v1
		if item is None:
			return False
		try:
			scene = item.scene()
			if scene is not None:
				scene.removeItem(item)
			item.setParentItem(None)
		except RuntimeError as exc:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum could not clear the SMARTS match highlight. Run the query again.",
			) from exc
		self._live_smarts_overlay_item_v1 = None
		return True

	#============================================
	def _attach_live_smarts_overlay_item_v1(self, scene: object, item: object) -> None:
		"""Attach a renderer-issued item without interpreting its paint instruction."""
		scene.addItem(item)

	#============================================
	def _remove_rejected_live_smarts_overlay_item_v1(self, item: object) -> None:
		"""Best-effort detach only a failed replacement candidate, never the live run."""
		try:
			scene = item.scene()
			if scene is not None:
				scene.removeItem(item)
			item.setParentItem(None)
		except RuntimeError:
			pass

	#============================================
	def _restore_replaced_live_smarts_overlay_item_v1(self,
			scene: object, item: object) -> bool:
		"""Restore prior paint, reporting whether the run still has a visual owner."""
		try:
			if item.scene() is None:
				scene.addItem(item)
			return True
		except RuntimeError:
			return False

	#============================================
	def _retire_live_smarts_query_v1(self, reason: str) -> bool:
		"""Synchronously remove current transient paint before a closed transition."""
		if reason not in self._FULL_RETIREMENT_REASONS:
			raise ValueError("Ferrum live SMARTS retirement reason is not closed")
		return self._retire_live_smarts_state_v1(
			reason, "_retire_live_document_smarts_query_v1",
		)

	#============================================
	def _retire_live_smarts_receipts_v1(self, reason: str) -> bool:
		"""Revoke query receipts and paint while retaining the published render plan."""
		if reason not in self._RECEIPT_RETIREMENT_REASONS:
			raise ValueError("Ferrum live SMARTS receipt retirement reason is not closed")
		return self._retire_live_smarts_state_v1(
			reason, "_retire_live_document_smarts_receipts_v1",
		)

	#============================================
	def _retire_live_smarts_state_v1(self, reason: str, entry_point: str) -> bool:
		"""Clear tab-owned transient state after one explicit native retirement call."""
		if not self._live_smarts_retirement_available_v1:
			return False
		item = self._live_smarts_overlay_item_v1
		if item is not None:
			try:
				scene = item.scene()
				if scene is not None:
					scene.removeItem(item)
				item.setParentItem(None)
			except RuntimeError:
				# Qt may have already destroyed an item with a retiring scene.  Its
				# absence still satisfies this idempotent visual-retirement contract.
				pass
		try:
			retire = getattr(self._live_document_session_v1,
				entry_point)
		except AttributeError as exc:
			self._live_smarts_retirement_error_v1 = exc
			self._live_smarts_retirement_available_v1 = False
			return False
		if not callable(retire):
			self._live_smarts_retirement_error_v1 = TypeError(
				"Ferrum live SMARTS retirement entry point is not callable",
			)
			self._live_smarts_retirement_available_v1 = False
			return False
		try:
			retire()
		except Exception as exc:
			self._live_smarts_retirement_error_v1 = exc
			self._live_smarts_retirement_available_v1 = False
			return False
		self._live_smarts_overlay_item_v1 = None
		self._live_smarts_receipt_v1 = None
		self._live_smarts_active_run_token_v1 = None
		self._live_smarts_retirement_error_v1 = None
		self._live_smarts_retirement_available_v1 = True
		return True

	#============================================
	def retire_live_smarts_query(self, reason: str) -> bool:
		"""Private Rust callback hook; it receives only a closed retirement reason."""
		return self._retire_live_smarts_query_v1(reason)

	#============================================
	def _retire_then_mutate_document_v1(self,
			operation: collections.abc.Callable[..., object],
			*args: object, **kwargs: object) -> object:
		"""Fence a Rust mutation before invoking its bound session method."""
		self._require_live_smarts_retirement_v1("document_mutation")
		self._notify_live_smarts_invalidation_v1()
		return operation(*args, **kwargs)

	#============================================
	def _retire_then_reproject_document_v1(self,
			operation: collections.abc.Callable[..., object],
			*args: object, **kwargs: object) -> object:
		"""Fence one Rust observation or Qt replacement before it can be displayed."""
		self._require_live_smarts_retirement_v1("document_reprojection")
		self._notify_live_smarts_invalidation_v1()
		return operation(*args, **kwargs)

	#============================================
	def _install_published_render_plan_v1(self,
			operation: collections.abc.Callable[..., object],
			*args: object, **kwargs: object) -> object:
		"""Install an already-published plan without retiring its new Rust state.

		The caller must obtain the observation from
		`_publish_live_render_plan_v1()`.  That transaction has already retired the
		obsolete Qt overlay and Rust receipt before publication.  A second native
		retirement here would clear the newly committed plan before a live query can
		use it.
		"""
		return operation(*args, **kwargs)

	#============================================
	def _publish_live_render_plan_v1(self, expected_revision: int) -> object:
		"""Publish one API-owned live plan before Qt installs its observation."""
		try:
			publish = getattr(self._session,
				"_publish_live_render_plan_v1")
		except AttributeError as exc:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum live render-plan publication is unavailable; refresh before editing.",
			) from exc
		if not callable(publish):
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum live render-plan publication is unavailable; refresh before editing.",
			)
		try:
			return self._retire_then_reproject_document_v1(publish, expected_revision)
		except ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError:
			raise
		except Exception as exc:
			raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
				"Ferrum live render-plan publication failed; refresh before editing.",
			) from exc

	#============================================
	def _require_live_smarts_retirement_v1(self, reason: str) -> None:
		"""Fail closed when native retirement cannot prove its receipt is unusable."""
		if self._retire_live_smarts_query_v1(reason):
			return
		raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
			"Ferrum live SMARTS retirement is unavailable; refresh before editing.",
		) from self._live_smarts_retirement_error_v1

	#============================================
	def _require_live_smarts_receipt_retirement_v1(self, reason: str) -> None:
		"""Fail closed when query cleanup cannot revoke every derived receipt."""
		if self._retire_live_smarts_receipts_v1(reason):
			return
		raise ferrum_qt.ferrum.document_tab_errors.FerrumNativeDocumentTabError(
			"Ferrum live SMARTS receipt retirement is unavailable; refresh before editing.",
		) from self._live_smarts_retirement_error_v1

	#============================================
	def _retire_if_current_live_run_v1(self, token: int, reason: str) -> bool:
		"""Retire only a current asynchronous delivery; stale work is a strict no-op."""
		if type(token) is not int:
			raise TypeError("Ferrum live SMARTS run token must be an int")
		if token != self._live_smarts_active_run_token_v1:
			return False
		self._require_live_smarts_receipt_retirement_v1(reason)
		return True
