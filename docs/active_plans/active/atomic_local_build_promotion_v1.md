# Atomic Local Build Promotion V1

## Status

Active design plan. This plan is a narrowly scoped hardening follow-on for the
local Ferrum program promotion boundary. It does not change chemistry,
rendering, document admission, Qt workflows, or public packaging.

## Evidence and current stability

The receipt-sealing review at
`/private/tmp/ferrum-review-local-runtime-receipt-sealing.md` found the current
receipt V2 sound for the local launcher source binding, artifact hashing,
sealed `all_test.sh` runtime input, and cleanup direction. Its remaining P2
issue is structural: the public local program is published through distinct
`build/bin` and `build/runtime` trees. Recoverable moves can prevent a torn
write, but they cannot make a reader observe both trees as one transaction.

The P1 repair that validates the candidate under build before publication is a
separate, earlier hardening item. This plan starts only after that repair is
present and proven. It neither replaces nor weakens candidate validation.

Graphify identifies `build.sh` as the owner of `build_local_program`, runtime
cleanup, the checkout-size guard, launcher generation, and the build lock.
The receipt implementation identifies the current local program as launchers,
Python runtime, and `engine-v1`; `all_test.sh` consumes the sealed runtime from
`build/runtime/python` and the two launchers from `build/bin`.

## Scope

Create one atomic promotion boundary for the complete local Ferrum program:

- the CLI launcher;
- the Qt launcher;
- the sealed Python runtime and its receipt;
- the `engine-v1` native-engine bundle;
- the existing local runtime validation relationship between these artifacts.

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
- Changes to Python package import policy, chemistry behavior, Qt UI behavior,
  document admission, or the receipt schema's security semantics unless their
  path references must change to describe the selected topology.
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
        ferrum-qt
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
root. Promotion creates a temporary `build/current.next` symlink to the fully
validated program root and replaces `build/current` with `os.replace`/the
platform-equivalent atomic same-directory rename. No read can resolve a new
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
6. **Clean only unreachable local build material.** After successful pointer
   promotion, remove the superseded inactive program and failed staging
   directories. Keep the selected current root. The cleanup must not scan or
   remove unrelated repository content, native source inputs, or the active
   program.
7. **Update consumers and documentation.** Make `all_test.sh`, receipt
   validation, CLI/Qt launchers, and local E2E use the authoritative current
   program root while retaining their documented stable paths. Update the
   relevant local-build documentation in the implementation task.

## Pre-production compatibility decision

The pre-production state permits removal of the old independent final-output
layout. The public local path contract remains `build/bin/*` and
`build/runtime/python`; callers must not receive a transition fallback or an
alternate legacy root. A clean build establishes the stable link structure.
This is a deliberate breaking internal-layout change with preserved developer
commands.

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
  program while removing only its inactive staged/superseded local program
  directories;
- existing local CLI smoke, Python binding, Qt, and repository-hygiene gates
  remain part of `./all_test.sh`.

These are permanent because they protect stable build contracts and resource
safety. They must use temporary directories and synthetic small trees, with no
network, native rebuild, timing threshold, real user home, or screenshot
assertion.

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
- The receipt proves artifacts from the same selected root and rejects mixed
  roots.
- `all_test.sh` retains sealed local runtime input and its normal gates.
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
