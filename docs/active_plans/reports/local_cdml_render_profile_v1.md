# Local CDML admission and render profile V1

Date: 2026-08-14

## Decision

Ferrum uses one extensible command namespace for document artifacts:

```text
ferrum cdml render svg INPUT --output OUTPUT
ferrum cdml render pdf INPUT --output OUTPUT
ferrum cdml render png INPUT --output OUTPUT --width WIDTH --height HEIGHT
```

All three formats call their native `DrawSinkV1` backend after one shared
complete-document admission step. PDF is generated directly by the vector PDF
sink; it is not SVG text placed in a PDF container. PNG rasterizes the same
admitted plan at exact caller-selected pixel dimensions.

The shared input profile supports only uncompressed CDML. The first artifact
commands do not infer a format from an extension. Ordinary Qt Open uses the
same admission policy for `.cdml` documents. CD-SVG and compressed input remain
closed. The CLI and private runtime Python seam remain pre-M18 until their
public contracts are frozen.

## Input resource policy

`ferrum-local-cdml-ingress-v1` is a versioned operational resource envelope:

| Resource | V1 limit |
| --- | ---: |
| UTF-8 source bytes | 16,777,216 |
| Element starts | 262,144 |
| Maximum nesting depth | 64 |
| Attributes, including namespace declarations | 1,048,576 |
| Lexical text and CDATA bytes | 8,388,608 |

The source-byte ceiling is the dominant ordinary desktop and CLI allocation
guard. The other dimensions independently reject compact XML that trades a
small source for excessive retained-tree fan-out, attributes, text, or depth.
These values are not XML validity rules and do not promise that every historical
document fits. A changed supported population or resource model requires a new
named profile rather than silently changing V1.

The previous measurement evidence remains useful context. The tracked CDML
fixtures reached 3,994 bytes, 68 elements, depth 4, 191 attributes, and 181 text
bytes; historical application templates reached 20,015 bytes, 316 elements,
depth 4, 974 attributes, and 2,245 text bytes. The V1 policy is deliberately an
engineering safety envelope, not a multiplier presented as corpus compatibility.
The project owner delegated the deployment choice on 2026-08-14 and asked that
the adaptable long-term boundary take priority over a provisional command.

## Rendering and publication

Each backend has a separate named V1 operational policy:

| Profile | Resource | Default |
| --- | --- | ---: |
| SVG | Completed UTF-8 artifact bytes | 67,108,864 |
| `ferrum-local-pdf-render-v1` | Completed PDF bytes | 67,108,864 |
| `ferrum-local-pdf-render-v1` | Traversal items before PDF allocation | 1,048,576 |
| `ferrum-local-pdf-render-v1` | Lowered path commands before PDF allocation | 8,388,608 |
| `ferrum-local-png-render-v1` | Raw RGBA bytes before pixmap allocation | 268,435,456 |
| `ferrum-local-png-render-v1` | Completed PNG bytes | 67,108,864 |

SVG checks the exact attempted serialized length while appending. PDF performs
its structural preflight before writer allocation and checks the completed
artifact afterward. PNG checks `width * height * 4` before pixmap allocation
and checks the encoded artifact. PNG additionally requires exact nonzero pixel
dimensions and an explicit transparent or RGB background; the CLI default is
opaque white. XML validation and renderer geometry have their own allocations,
so these are not described as whole-process memory bounds. CLI flags may select
different per-invocation caps without silently changing the named defaults.

The API observes one exact revision-zero admitted session, composes one
authenticated page plan, and refuses any plan with excluded roots. The CLI
therefore cannot silently publish a partial drawing. A future opt-in partial
mode must own an explicit exclusion report rather than weakening this default.

For file input, the same opened regular descriptor supplies admitted bytes and
remains live through publication. The descriptor-relative publisher refuses the
source path or an observed hard-link alias, uses its private same-directory
temporary, and preserves an existing destination on prepublication failure.
Confirmed publication stays silent; directory-entry-unconfirmed publication
returns success with an explicit standard-error warning, while possibly
published and prepublication failures remain errors.
Standard input and output use `-`; the same input and format-specific resource
policies still apply.

## Ordinary desktop Open

The ordinary OASA-free `MainWindow` calls the named Rust profile directly for
interactive, programmatic, and launch-file Open. Python supplies only an exact
path; it never reads document bytes or reconstructs the five numeric limits.
One private worker prepares the Rust-owned revision-zero session and its exact
render observation off the Qt thread. A one-use receipt transfers both together,
and the tab constructor verifies their revision and digest before it builds a
scene. A failed source, parse, render, provenance, or installation step publishes
no partial tab and leaves the current document unchanged.

Multiple startup paths queue through this same boundary. Cancellation and window
close invalidate future delivery but do not claim to preempt a native read already
in progress. The explicit compatibility host consumes the same complete receipt
synchronously; it no longer reads files in Python or discards and recomputes the
prepared observation. Same-tab replacement and recent-file routing are separate
product contracts, not reasons to weaken admission.

## Extension rule

- PDF calls `render_document_plan_to_pdf_v1` with explicit completed-output and
  plan-complexity budgets. It never wraps or rasterizes SVG.
- PNG calls `render_document_plan_to_png_v1` with explicit pixel dimensions,
  background, raw-RGBA, and encoded-output budgets.
- CD-SVG and compressed sources need separate versioned ingress profiles.
- Desktop and CLI callers share the named input profile; numeric limits remain
  Rust-owned rather than copied into frontend configuration.
- M17/M18 may freeze names, exit behavior, and reporting, but should preserve
  the one render namespace and native-backend ownership.

Permanent tests cover bounded stdin, structural SVG/PDF/PNG output, exact PNG
dimensions, source preservation, source/destination alias refusal, symlink
refusal, format-specific limit containment, shared complete-plan admission,
session nonmutation, and exact backend budget behavior. Corpus measurements and
visual artifact inspection remain one-time evidence rather than test fixtures.
