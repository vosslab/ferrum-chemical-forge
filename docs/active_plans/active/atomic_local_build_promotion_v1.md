# Atomic Local Build Promotion V1

## Status

Completed design plan. This plan is a narrowly scoped hardening follow-on for the
local Ferrum program promotion boundary. It does not change chemistry,
rendering, document admission, Qt workflows, or public packaging.

## Evidence and current stability

The delivered `ferrum-local-runtime-receipt-v4` seals the selected local
program's extension, native adapter, lease wrappers, and the CLI and Qt
payloads that the wrappers execute. One immutable `build/programs/program-*`
root contains the complete local program. Atomic replacement of `build/current`
publishes that root, while stable `build/bin` and `build/runtime` links preserve
the local command paths without independently promoting launcher and runtime
trees. The receipt therefore proves artifacts from one selected root and
rejects mixed-root combinations.

The P1 repair that validates the candidate under build before publication is a
separate, earlier hardening item. This plan starts only after that repair is
present and proven. It neither replaces nor weakens candidate validation.

`build.sh` owns local-program staging, the checkout-size guard, launcher
generation, build locking, and cleanup. Each launcher holds the immutable
program root's runtime lease through execution, so cleanup retains active roots
and reclaims only inactive, non-current owned roots. `all_test.sh` consumes the
sealed selected runtime through the stable paths.

## Scope

Create one atomic promotion boundary for the complete local Ferrum program:

- the CLI launcher;
- the Qt launcher;
- the sealed Python runtime and its receipt;
- the `engine-v1` native-engine bundle;
- the existing local runtime validation relationship between these artifacts.
- the sourced `source_me.sh` development bootstrap selector, which must select
  the staged native runtime and fail closed when that runtime is absent or
  cannot be selected, including when invoked outside the checkout directory.

Preserve these user-facing local commands and inputs:

```bash
./build.sh
build/bin/ferrum
build/bin/ferrum-qt
build/runtime/python
./all_test.sh
```

Preserve the 20 GiB checkout guard, the build lock, successful-build cleanup,
and failed-build survival of the prior runnable local program. The result
remains a disposable repository-local build for testing.

## Non-goals

- Publishing, installation, wheel distribution, registries, network access,
  CI workflows, and versioned release semantics.
- Artifact-retention policy beyond cleanup necessary to leave one current local
  program and remove failed or superseded temporary build material.
- General Python package import policy, publishing, installation, or global
  environment behavior. Within this local-build contract, `source_me.sh` owns
  the GUI import provenance order: repository Qt source first, sealed staged
  runtime second, then retained caller entries. It is not a package-policy,
  installer, or distribution mechanism.
- Changes to chemistry behavior, Qt UI behavior, document admission, or the
  receipt schema's security semantics unless their path references must change
  to describe the selected topology.
- Reintroducing legacy output roots, compatibility aliases, or independent
  launcher/runtime promotion paths.

## Selected durable topology

Use one immutable, versioned program root and one atomic current-pointer
replacement:

```text
build/
  current -> programs/<program-id>
  bin -> current/bin
  runtime -> current/runtime
  programs/
	<program-id>/
		bin/
			ferrum
			ferrum.program
			ferrum-qt
			ferrum-qt.program
      runtime/
        python/
          ferrum-local-runtime-receipt.json
          ferrum_chem<abi-suffix>
          .dylibs -> ../engine-v1
        engine-v1/
```

`build/bin` and `build/runtime` are stable relative references to `current`,
not independently promoted directories. Existing commands therefore retain
their spelling while every path resolution reaches the same immutable program
root. Promotion creates an owner-unique temporary current-pointer symlink to
the fully validated program root and replaces `build/current` with
`os.replace`/the platform-equivalent atomic same-directory rename. Startup
removes stale owned pointer staging while holding the build lock. No read can
resolve a new
launcher with an old runtime after the pointer swap.

The program identifier is an opaque build-local staging identifier, not a
package or publication version. It is used only to name a private immutable
program directory while promotion is in progress.

## Candidate topology considered

### Retain independent `build/bin` and `build/runtime` moves

Rejected. Each move can be recoverable, yet commands crossing both paths can
observe mixed generations.

### Publish a single `build/program` directory and change all callers

Rejected. It would break established local commands and sealed test inputs for
no correctness gain over stable links through a shared pointer.

### Use one versioned root plus `build/current`

Selected. The full program becomes immutable before publication, one pointer
is the sole public mutation, and existing local paths remain stable.

## Implementation sequence

1. **Measure current path contracts.** Record the current launcher, receipt,
   runtime, engine-bundle, size-guard, cleanup, and `all_test.sh` references.
   Confirm the current candidate-under-build validation has completed before
   this work starts.
2. **Introduce a program-root path model.** Give build and receipt helpers one
   authoritative local-program root with named `bin`, `runtime`, and
   `engine-v1` children. Derive stable paths through `build/current`; remove
   independent final-destination assumptions.
3. **Stage the complete program privately.** Allocate a fresh directory below
   `build/programs/`, build the native closure, stage the extension and both
   launchers beneath that one root, and write the receipt only after all
   artifact bytes exist.
4. **Validate before promotion.** Run the existing sealed receipt/import and
   launcher capability checks against the candidate program root. Candidate
   failure removes only the candidate and leaves `build/current` unchanged.
5. **Promote one pointer.** Create and atomically replace `build/current` only
   after candidate validation succeeds. Stable `build/bin` and `build/runtime`
   links are created once as path infrastructure, or repaired before any build
   starts while no current program is changed.
6. **Clean only lease-inactive local build material.** Before publication,
   create one stable lease inode inside every immutable program root. Generated
   CLI and GUI launchers retain a shared advisory lock on that inode through
   exec and process exit. After successful pointer promotion, cleanup attempts
   a nonblocking exclusive lock on each non-current `program-*` root's lease: remove when
   it succeeds, preserve shared-held roots, and fail safe on indeterminate lock
   errors. A missing, non-regular, or unreadable lease marks a non-current root
   as malformed pre-production local build output and it is reclaimed. On every
   locked startup, before the checkout guard or new compiler work, repeat that
   lease check for non-current `program-*` roots. This recovers an immutable
	root stranded after its rename but before current-pointer replacement. The
   20 GiB checkout guard bounds retained local-build storage; cleanup must not
   scan or remove unrelated repository content, native source inputs, or the
   active program. After publication, cleanup never mutates a program root
   through stable `build/current`, `build/bin`, or `build/runtime` paths;
   candidate-native temporary work is removed only before staging promotion.
   The wrapper and its executed payload are named closed artifact identities in
   the receipt; payload resolution derives only from the selected program root
   and its fixed repository-relative topology (ASVS 5.3.2).
7. **Update consumers and documentation.** Make `all_test.sh`, receipt
   validation, CLI/Qt launchers, local E2E, and the sourced `source_me.sh`
   bootstrap selector use the authoritative current program root while retaining
   their documented stable paths. The selector must fail closed rather than
   import a globally installed extension. Update the relevant local-build
   documentation in the implementation task.

## Pre-production compatibility decision

The pre-production state permits removal of the old independent final-output
layout. The public local path contract remains `build/bin/*` and
`build/runtime/python`; callers must not receive a transition fallback or an
alternate legacy root. A clean build establishes the stable link structure.
This is a deliberate breaking internal-layout change with preserved developer
commands.

Under the stable build lock, obsolete direct `build/bin` and `build/runtime`
directories are retired before candidate staging. They are disposable local
build artifacts, not a launchable program or lease-bearing compatibility root.

## Verification design

### Permanent tests

Add only behavior-level tests that prove stable contracts:

- path-model tests show CLI, GUI, runtime, receipt, and engine bundle resolve
  under one current program root;
- receipt validation rejects a launcher/runtime/engine combination outside its
  selected root;
- `all_test.sh` obtains its sealed runtime input through the current program
  boundary and retains its existing user-facing path;
- build helper tests show an invalid candidate cannot replace the current
  pointer and that a successful candidate does;
- cleanup tests use a temporary build root and prove it retains the selected
  program and a valid lease-held superseded root while removing inactive owned
  or malformed `program-*` directories, without touching unrelated content;
- existing local CLI smoke, Python binding, Qt, and repository-hygiene gates
  remain part of `./all_test.sh`.

These are permanent because they protect stable build contracts and resource
safety. They must use temporary directories and synthetic small trees, with no
network, native rebuild, timing threshold, real user home, or screenshot
assertion.

A deterministic lifecycle test covers missing, non-regular, and unreadable
lease states. It does not force a genuine non-contention lease-probe error:
the supported platform offers no deterministic fixture for that error without
fragile syscall monkeypatching, so fail-safe retention for such an error remains
a documented residual behavior.

### One-time failure-injection evidence

Use a disposable temporary build root or temporary pointer namespace to prove:

1. start with a valid current program A;
2. construct candidate B;
3. inject failure before receipt/import/launcher validation completes;
4. prove every stable path still resolves to A and A launches;
5. construct and validate B;
6. perform the current-pointer replacement;
7. prove all stable paths resolve to B and no stable path resolves to a mixed
   A/B program.

Also inject a cleanup failure after pointer replacement and prove the selected
current root remains usable; cleanup may leave disposable inactive material for
later cleanup rather than jeopardize the runnable program. This is
implementation evidence, not a permanent test if it requires a real native
build or filesystem fault injection unavailable in normal test environments.

## Acceptance criteria

- A complete local program is staged and validated under one immutable root
  before it is observable through stable local paths.
- Exactly one atomic current-pointer replacement publishes the local program.
- `build/bin/ferrum`, `build/bin/ferrum-qt`, and `build/runtime/python` always
  resolve through the same selected root.
- A failed candidate build preserves the prior runnable local program.
- A locked startup removes an inactive promoted-root orphan after a crash before
  pointer replacement, while preserving the prior current program.
- The receipt proves artifacts from the same selected root and rejects mixed
	roots.
- The receipt seals both lease wrappers and the executable CLI and Qt payloads
	that they execute.
- `all_test.sh` retains sealed local runtime input and its normal gates.
- From the checkout or an outside working directory, `source_me.sh` selects
  only the staged selected native runtime and fails closed when it cannot do so.
- The 20 GiB checkout guard and native-build cleanup remain active.
- No install, publish, archive-retention, network, or legacy-output workflow
  is added.
- Targeted tests pass, the one-time failure-injection record is captured, then
  `./all_test.sh` passes from a freshly promoted local program.

## Risks and decisions required during implementation

The implementation must confirm platform semantics for replacing a symlink in
one directory and the behavior of active local processes whose program root is
later unlinked. If those measurements show that immediate superseded-root
cleanup can invalidate an active local process, cleanup must be delayed to the
next successful build startup rather than broaden artifact retention. That is
a cleanup timing decision, not a reason to weaken atomic promotion.
