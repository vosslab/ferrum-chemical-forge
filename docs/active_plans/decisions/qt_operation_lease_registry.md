# Qt operation lease registry

## Status and scope

Approved implementation plan. This is a Qt-only foundation migration, not a
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

Patch 1 is the required first atomic migration. Add
`operation_leases.py` and `template_catalog_controller.py`; retain
`template_catalog_dialog.py`; move the existing native placement adapter to
`document_tab.py`; and delete both `template_catalog_window.py` and
`template_catalog_tab.py` with their ordered mixin bases. Preserve the public
`chemistry.template.catalog` QAction and its existing YAML menu/ribbon clients.

`TemplateCatalogController` is a window-parented `QObject`. It owns catalog
dialog publication, viewport event filtering, pointer-placement context, and
mouse/cursor restoration. `FerrumNativeDocumentTab` remains the sole native
mutation boundary through its explicit placement method. Patch 1 makes no Rust
or PyO3 change.

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
- `ferrum_qt/ferrum/close_decision.py`
- deletion of `ferrum_qt/ferrum/template_catalog_window.py` and
  `ferrum_qt/ferrum/template_catalog_tab.py`

Its exact test scope is `test_operation_leases.py` (new),
`test_template_catalog_controller.py` (new), the replacement public dialog/
window/lifecycle catalog tests, and `tests/e2e/e2e_template_catalog_authoring.py`.
It deletes legacy-mixin assertions rather than preserving private-field tests.
No resource YAML, Rust crate, PyO3 binding, or unrelated feature-family move
belongs in Patch 1.

Before Patch 2, obtain independent review, run the full Qt suite, the public
template-authoring E2E, the affected native/freshness suites, and the normal
aggregate gate. The Patch 1 acceptance boundary includes owner-capability
isolation, exact registered-tab identity, duplicate-acquire refusal,
idempotent cancellation, one-way terminal state, active-unregister refusal,
one-click clean close, ordinary dirty close, single refresh, no legacy
catalog intent/module/import, and one exact native placement.

### Patch 2: Local Document Open controller

Patch 2 starts only after Patch 1 acceptance. It converts Local Document Open
into an explicit controller and proves source-retaining asynchronous semantics.
It preserves the existing `file.open`, `file.open_current`, and
`file.open.cancel` QAction identities, shortcuts, and YAML/menu/ribbon clients.

Open uses `BLOCK_UNTIL_SETTLED`. A cancellation transitions to
`CANCELLATION_REQUESTED`, invalidates delivery and clears any declared queue,
but truthfully retains the exact source tab and immutable input facts until the
Qt-thread worker-finished relay settles it. Rust may finish reading; no late
delivery, document mutation, publication, focus theft, or destroyed-QObject
access is permitted. No thread termination claim is allowed.

Patch 2 production scope is limited to
`local_document_open_controller.py` (new), `main_window.py`,
`main_window_lifecycle.py`, `tab_operations.py`, `close_decision.py`, and
deletion of `local_document_open.py`. Its exact focused test scope is the new
Local Open controller/lease test plus the existing Open/background-job/action
identity tests and their public E2E. `local_document_open_types.py`, resource
YAML, Rust, and PyO3 remain unchanged unless a separately approved contract
proves that a change is required. Patch 2 does not introduce a Rust/PyO3
cancellation contract, a generic event bus, or a simultaneous migration of
another family. Its acceptance evidence includes `CANCELLATION_REQUESTED`,
source retention, worker-finished-only release, queued-operation disclosure,
cancellation delivery suppression, no stale cross-tab completion, and truthful
close behavior.

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
