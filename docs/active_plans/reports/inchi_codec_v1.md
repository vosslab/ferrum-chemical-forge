# InChI Codec V1

Date: 2026-08-13

Status: implemented and verified as the completed InChI part of M5. The remaining
SMARTS export semantics are also green, and the reference SMARTS codec is export-only.

## Delivered Boundary

- ABI-4 now imports Standard and nonstandard Fixed-H InChI into the complete
  owned molecule graph, exports both modes, and derives the official 27-byte
  InChIKey.
- Rust validates input strings before dynamic loading, validates every native
  status/length/string result, copies the response into owned Rust values, and
  releases the adapter buffer before returning.
- PyO3 exposes frozen `InchiModeV1`, `parse_inchi`, `molecule_to_inchi`, and
  `inchi_to_inchi_key` values from the direct `ferrum_chem` extension.
- PyO3 also prepares one handle-free, placed InChI insertion for the standalone
  native editor. Qt runs that operation in a cancellable worker and submits the
  result only while the captured document revision and digest remain current.
- The Rust CLI adds `inchi inspect --adapter PATH INCHI` and
  `smiles to-inchi --adapter PATH [--fixed-hydrogen] SMILES`. The only behavior
  toggle selects a chemically meaningful output mode; native build policy is
  not exposed as CLI tuning.

## Security Review

The native adapter remains the only owner of RDKit objects. The boundary has a
single bounded text-response protocol, rejects embedded NUL and malformed
prefixes before native loading, validates the 27-character InChIKey grammar,
and maps native failures to typed Rust/Python errors. Input and output obey the
existing ABI allocation ceiling; this is a structural safety ceiling, not a
claimed product-size recommendation.

RDKit's InChI reader reports complete explicit-hydrogen counts and marks those
atoms as having no implicit hydrogens. The InChI insertion coordinator treats that
bit as parser-owned state only when the corresponding complete count is present;
positive counts are persisted and zero uses CDML's documented default. The shared
SMILES/Molfile graph converter remains strict. Chirality, radicals, atom maps,
bond stereo/direction, unresolved aromaticity, and unsupported bond orders still
fail before a document candidate exists.

The source builder pins and hash-verifies IUPAC InChI 1.07.3. Its configuration
disables host InChI discovery, directs installation into the private output
root, and passes the CMake provenance audit. The clean current-RDKit build also
identified RDKit's new public RingDecomposerLib dependency; the measured wheel
closure now declares that header and dylib explicitly instead of finding a host
copy.

## Evidence

- Source-built RDKit: `Release_2026_03_5`.
- Wheel: `ferrum_chem-26.8.0-cp312-cp312-macosx_11_0_arm64.whl`.
- Wheel SHA-256:
  `0f2de3ae9819545846af46efc45cae3eddbfbcabda5a0653f31d2a4ff6e79e6f`.
- Measured macOS arm64 native closure: 18 dylibs including Ferrum, RDKit InChI,
  and RingDecomposerLib; no RDKit Python, SWIG, or compiled Boost payload.
- Fresh installed-extension InChI tests: 4 passed under CPython 3.12 with
  isolated mode and bytecode disabled.
- The authoritative existing-build wheel E2E passed direct-extension document
  behavior, Standard/Fixed-H InChI, InChIKey, graph round trip, closure audit,
  and the same probes after replacing the adapter with distinct verified bytes.
- A disposable offline five-molecule corpus matched RDKit 2026.03.4 and
  2026.03.5 exactly for Standard InChI, Fixed-H InChI, InChIKey, and canonical
  round trip. Exact comparison is appropriate here because these are canonical
  identifiers; it is not a general byte-equivalence requirement.

The source build and cross-version corpus were one-time release evidence. The
permanent tests remain focused on validation, ownership, typed failures, wire
decoding, and installed public behavior; they do not use the network.

A disposable current-extension probe prepared methane, ethanol, benzene,
ammonium, sodium chloride, carbon dioxide, and isotope-labelled methane. A chiral
InChI remained a typed unsupported insertion. A separate public native-window
probe committed and rendered methane at revision 1 with four explicit hydrogens
and no OASA import. Those probes were implementation evidence, not permanent
private-worker, pixel, timing, or artifact-byte gates.
