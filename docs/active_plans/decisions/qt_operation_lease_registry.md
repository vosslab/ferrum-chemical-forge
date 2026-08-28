# Qt operation lease registry

## Status and scope

Patch 1 is implemented and accepted by independent architecture, Qt/HCI, and
tests/docs review with no P1-P3. This is a
Qt-only foundation migration, not a
Rust, PyO3, chemistry, document-schema, or directory-reorganization change.
It applies only after the current automated baseline is green and preserves
M5.A and full parity as open until their separate human acceptance gates pass.

The current ordered `MainWindow` and `DocumentTab` mixins duplicate operation
lifetime, busy, close, and action-refresh state. This plan replaces that
duplication one feature family at a time. Existing private mixin interfaces are
implementation evidence, not compatibility contracts.

## Decision

Use one `OperationLeaseRegistry` per `FerrumNativeMainWindow` and explicit
feature controllers. The registry owns only Qt lifecycle identity and state;
Rust remains sole authority for admission, document validity, snapshots,
receipts, payloads, mutation, and typed native outcomes.

All new Qt-local names are unversioned: `OperationFamily`, `ClosePolicy`,
`LeaseState`, `OperationLeaseId`, `TabLeaseIdentity`, `OperationLease`,
`LeaseOwnerCapability`, `OperationLeaseError`, and
`OperationLeaseRegistry`. They are private in-process mechanisms rather than
durable cross-language contracts.

The registry binds exact registered tab objects, issues an opaque capability
per registered family, and is the sole owner of lifecycle state and active
indexes. A controller retains only its feature payload and gets its tab from
the lease. It must not duplicate a busy flag, retain a second tab-liveness
model, inspect a tab session, or retrieve another controller by name.

The closed state machine is:

```text
ACTIVE -> CANCELLATION_REQUESTED -> COMPLETED | REFUSED | FAILED | CANCELLED
ACTIVE -> COMPLETED | REFUSED | FAILED | CANCELLED
```

Terminal transitions are once-only and retire the active index. Cancellation is
an idempotent request, not a claim that Rust stopped. Direct typed operations
are `bind_tab`, `acquire`, `active_for_tab`, `has_active`,
`request_cancellation`, `settle`, and `unregister_tab`. Capability misuse,
unregistered/disposed acquisition, duplicate family/tab acquisition, invalid
transition, and active-tab unregister raise `OperationLeaseError`.

No aliases, old intent fields, generic event bus, service locator, or Qt/Rust
lease bridge are approved. The registry has no QAction, widget, signal, CDML,
path, payload, prepared receipt, or document-mutation authority. Controllers
receive narrow dependencies and invoke one injected non-reentrant refresh seam.

## Patch order

### Patch 1: Template Catalog controller

Patch 1 is the first completed atomic migration. It added
`operation_leases.py` and `template_catalog_controller.py`, retained
`template_catalog_dialog.py`, moved the native placement adapter to
`document_tab.py`, and deleted both `template_catalog_window.py` and
`template_catalog_tab.py` with their ordered mixin bases. It preserves the public
`chemistry.template.catalog` QAction and its existing YAML menu/ribbon clients.

Patch 1 also completed a narrow lifecycle-registration repair for pristine
Local Open replacement. Phase 1 fully integrates the provisional new tab:
shared product hooks run and the registry binds it before publication. Phase 2
can exactly restore the old registration after typed old-unregister/disposal
refusal; if provisional integration fails, the new tab is completely retired,
unbound, disposed, and stripped of its product hooks. Phase 3 commits after old
disposal is irreversible and provides no fictional rollback. Shutdown settles
catalog placement and retires clean tabs through the registry-aware ordinary
close lifecycle. A dirty user close remains an intentional presented refusal;
deterministic tests explicitly discard the tabs they own. This is bounded Patch
1 repair only, not a broader atomicity claim or Patch 2 migration.

`TemplateCatalogController` is a window-parented `QObject`. It owns catalog
dialog publication, viewport event filtering, pointer-placement context, and
mouse/cursor restoration. `FerrumNativeDocumentTab` remains the sole native
mutation boundary through its explicit placement method. Patch 1 makes no Rust
or PyO3 change.

`FerrumNativeMainWindowLifecycleMixin` owns one explicit close adapter shared by
both window hosts. The adapter uses the ordinary registry-aware close lifecycle:
clean tabs settle and close, dirty user close remains a presented decision, and
test-owned tabs close only through explicit `DISCARD`. Focused tests use that
contract directly; `tests/e2e/ferrum_qt_e2e.py` owns the shared
`close_e2e_main_window` teardown helper for every registered E2E scenario.
Queued operation presentation uses a `QTimer` callback bound to the owning Qt
window context, so queued delivery cannot outlive its window.

Catalog placement uses `CANCEL_AND_BLOCK_TAB_CLOSE` only while pointer cleanup
is active. Escape, focus loss, right click, tool replacement, tab close, and
window shutdown synchronously remove the event filter, restore pointer state,
settle `CANCELLED`, and perform the existing dialog behavior. A left click first
retires pointer state, then settles the native result as `COMPLETED`, `REFUSED`,
or `FAILED`; a stale fence is `REFUSED` and never retries implicitly.

Close has the stronger pre-production contract: an armed catalog on a clean tab
is cancelled and the same close attempt continues to `CLOSED`; a dirty tab
continues to its ordinary dirty decision. Delete
`CloseResult.CATALOG_PLACEMENT_BLOCKED` and its guard. Reserve the new typed
`CloseResult.OPERATION_CANCELLATION_FAILED` only for an unexpected exact
`OperationLeaseError` during synchronous cleanup; preserve the tab in that
case. Do not catch broad Qt exceptions or synthesize a retry-only result.

Patch 1 production scope is limited to:

- `ferrum_qt/ferrum/operation_leases.py` (new)
- `ferrum_qt/ferrum/template_catalog_controller.py` (new)
- `ferrum_qt/ferrum/main_window.py`
- `ferrum_qt/ferrum/main_window_lifecycle.py`
- `ferrum_qt/ferrum/document_tab.py`
- `ferrum_qt/ferrum/tab_operations.py`
- `ferrum_qt/ferrum/free_compact_group_placement.py`
- `ferrum_qt/ferrum/user_templates.py`
- `ferrum_qt/ferrum/structure_selection.py`
- `ferrum_qt/ferrum/close_decision.py`
- deletion of `ferrum_qt/ferrum/template_catalog_window.py` and
  `ferrum_qt/ferrum/template_catalog_tab.py`

Its exact test scope is `test_operation_leases.py` (new),
`test_template_catalog_controller.py` (new), the replacement public dialog/
window/lifecycle catalog tests, and `tests/e2e/e2e_template_catalog_authoring.py`.
It deletes legacy-mixin assertions rather than preserving private-field tests.
No resource YAML, Rust crate, PyO3 binding, or unrelated feature-family move
belongs in Patch 1.

Before Patch 2, preserve the accepted Patch 1 contract and its evidence. Fresh
`./build.sh`, `tests/e2e/run_all.sh`, and `./all_test.sh` passed, with 8,119
hygiene checks, 294 PyO3 tests, and 370 Qt tests; an independent full Qt run
passed 369 before its added regression. The Patch 1 acceptance boundary includes owner-capability
isolation, exact registered-tab identity, duplicate-acquire refusal,
idempotent cancellation, one-way terminal state, active-unregister refusal,
one-click clean close, ordinary dirty close, single refresh, no legacy
catalog intent/module/import, and one exact native placement.
Native visual, VoiceOver, contrast, and focus acceptance remain human gates;
remote CI and release remain separate. Patch 1 does not accept Patch 2, M5.A,
or full OASA/BKChem parity.

### Patch 2: Local Document Open controller

Patch 2 is implemented and accepted at the code/automated-gate boundary. It
replaces the stateful `local_document_open.py` mixin with four explicit modules:
`local_document_open_contract.py`, `local_document_open_composition.py`,
`local_document_open_controller.py`, and `local_document_open_delivery.py`.
The frozen contract owns immutable request, fence, intent, and closed-outcome
facts plus a callback-only window port. Composition is the one adapter from
that port to the public `MainWindow` host. The controller owns declaration,
queue, actions, workers, exact source-tab leases, and terminal settlement. The
delivery module owns one intent's staged worker facts and Qt-thread admission,
replacement, dialog presentation, and outcome publication. It preserves
`file.open`, `file.open_current`, and `file.open.cancel` QAction identities,
shortcuts, and YAML/menu/ribbon clients.

Open uses an exact source-tab `LOCAL_DOCUMENT_OPEN` lease with
`BLOCK_UNTIL_SETTLED`. A cancellation transitions to
`CANCELLATION_REQUESTED`, invalidates delivery and clears any declared queue,
but truthfully retains the exact source tab and immutable input facts until the
Qt-thread worker-finished relay settles it. The window-owned named
`_LocalDocumentOpenWorkerRelay` receives prepared and failed facts by queued
Qt connection, stages them in the intent delivery, and permits delivery only
from the worker-finished path. Rust may finish reading; no late delivery,
document mutation, publication, focus theft, or destroyed-QObject access is
permitted. No thread termination claim is allowed.

Queued requests remain frozen at declaration. A stale or closed source is
accounted as `REFUSED`; it never reanchors to another tab. Completed, failed,
refused, and cancelled delivery settle only at worker finish. Successful
background Open activates a tab only when its captured source/focus facts still
hold. The delivery asks the host for a QWidget-owned dirty-replacement dialog
and rechecks cancellation and fences after that dialog and before commit.
Startup is transactional: incomplete provisional installation retires its
worker/relay/product state before terminal failure. The lifecycle replacement
transaction retains the same source lease through exact-ID/identity detach,
typed rollback, and terminal settlement; the lease never transfers to the
incoming tab. Once durable replacement is irreversible, `COMPLETED` remains
the truthful outcome even if typed post-commit presentation fails; pre-commit
invalid worker protocol or invariant errors complete safe cleanup and failed
lease settlement, then propagate rather than being broadly masked.

Patch 2 production scope is the four new Local Open modules,
`main_window.py`, `main_window_lifecycle.py`, `operation_leases.py`,
`tab_operations.py`, and `close_decision.py`; it deletes
`local_document_open.py` and unused `native_app.py`. `MainWindow` is the sole
product startup composition. The patch leaves `local_document_open_types.py`,
resource YAML, Rust, and PyO3 unchanged. It adds no Rust/PyO3 cancellation
contract, generic event bus, service locator, or simultaneous feature migration.

Focused Local Open/lease/CDML evidence now passes 43 checks. `./build.sh` and
`bash tests/e2e/run_all.sh` exit 0; the registered E2E lane includes
`ferrum-local-document-open-lifecycle-e2e-v1`. `./all_test.sh` exits 0 with
8,097 hygiene checks, registered E2Es, 294 PyO3 binding tests, and 395 Qt
tests. The focused suite keeps unit-contract behavior in pytest and moves the
public open/save/reopen, nested dirty-dialog, and post-commit recovery workflows
to that E2E lane. Final independent architecture audit ACCEPT found no P1-P3.
Native visual, VoiceOver, contrast, focus-ring review, remote CI, release,
M5.A, and full parity remain separate and open.

## Interaction and accessibility boundary

The author-facing vocabulary is Working, Cancelling (waiting for safe finish),
and Finished. Presentation identifies the operation, owner tab, phase,
cancellability, and recovery without exposing worker, revision, receipt, or
path internals. Action eligibility derives from declared compatibility queries,
not a global busy boolean. Preserve safe exceptions rather than disabling every
action while any operation exists.

Each cancellable operation remains reachable through its labelled existing YAML
action. The operation surface has an explicit accessible name, description,
state/value, readable status alternative, non-color-only busy/cancelling state,
and keyboard-reachable Cancel control. Background completion never steals focus
from another tab; terminal focus restoration is allowed only when the source
tab remains current and valid. Escape remains scoped to the active picker,
dialog, or canvas tool and does not silently cancel background Rust work.

Automated tests can prove action identity, enablement, named controls,
keyboard activation, focus policy, and typed recovery. They cannot prove native
screen-reader announcements, actual focus-ring visibility, or light/dark/high-
contrast rendering. VoiceOver/native accessibility-inspector review, contrast
measurement, keyboard walkthrough, and fresh screenshots remain human gates.

## Verification commands

Run the repository-prescribed build and aggregate gates plus the affected Qt,
native, and E2E commands recorded by the implementation change. At minimum:

```bash
./build.sh
source source_me.sh && python3 -m pytest packages/ferrum-chem-qt.app/tests -q
source source_me.sh && python3 tests/e2e/e2e_template_catalog_authoring.py
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
./all_test.sh
git diff --check
git diff --cached --check
```

Run the focused registry/controller/lifecycle suite before the broad Qt suite.
For Patch 2, add the focused Local Open cancellation/fence suite before the
same broad gates. Evidence must be recorded against the exact source revision;
passing Patch 1 does not approve Patch 2 or full parity.

## References

- [FULL_PARITY_RUST_FIRST.md](../active/FULL_PARITY_RUST_FIRST.md)
- [m5_template_catalog_v1.md](m5_template_catalog_v1.md)
- [CODE_ARCHITECTURE.md](../../CODE_ARCHITECTURE.md)
- [FILE_STRUCTURE.md](../../FILE_STRUCTURE.md)
