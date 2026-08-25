# M16 session-adoption evidence

This report preserves the detailed M16 adoption record after the active plan was
reduced to an actionable tracker on 2026-08-19. M16 is complete as a bounded
session-adoption milestone; it is not a product-release claim. M19, M20, and M22
remain open for integration, packaging, and release closure.

## Accepted native session boundary

The ordinary Ferrum route owns a bounded Rust document session. It opens local,
uncompressed CDML through a versioned budgeted-file profile; renders an authoritative
observation; saves/reopens without frontend reconstruction; and preserves opaque
content, direct-child identities, source order, and revision/digest provenance.
All mutations are prepared from immutable detached facts, then committed on the GUI
thread only if their captured revision/digest still match. Sessions are thread
confined and updates are serialized per session.

The accepted native operations include selecting/editing atom elements, adding a
free-standing atom, connecting selected atoms or extending an atom with a single
bond, moving/deleting atoms, deleting/editing bonds, changing normal bond order and
supported presentation facts, coordinate regeneration, save/reopen, undo/redo, and
detached SVG/PDF/bounded PNG export. Each operation has a Rust identity/provenance
receipt and a typed no-op, stale, malformed, unsupported, or resource-limit outcome.
The focused records are [native bond creation](native_bond_creation_v1.md),
[native atom movement](native_atom_move_v1.md),
[native atom deletion](native_atom_deletion_v1.md),
[native bond deletion](native_bond_deletion_v1.md),
[native bond order](native_bond_order_v1.md),
[native bond properties](native_bond_properties_v1.md),
[native coordinate regeneration](native_coordinate_regeneration_v1.md), and
[native molfile import](native_molfile_import_v1.md).

## Explicitly bounded import and domain routes

The standalone evidence route imported bounded UTF-8 SDF batches, keeping ordered
record properties in a preserved Ferrum extension child, and imported InChI through
the packaged adapter off the UI thread. Those are evidence for bounded supported
profiles, not an unconstrained text-import subsystem. Unrepresentable chirality,
radicals, maps, stereo facts, compression, CD-SVG, and other unsupported sources
produce typed refusals before partial session mutation.

Native Molfile import, peptide sequence insertion through
`prepare_ferrum_peptide_insertion_v1`, and linear-form conversion use their own
complete-document/session contracts. The historical compatibility host,
live OASA workers, plugin registrar, and product OASA dependency are retired. The
application never redirects an unsupported route into a hidden OASA editor.

## M16 completion boundary

M16 closed when one ordinary Rust-native window owned every route classified
supported at that time and every remaining historical route was explicitly refused
or deliberately dropped. It did not certify all rows in the evolving capability
matrix, guarantee package installation, establish a second platform, promise a
plugin system, or establish release-artifact provenance. Those are M19, M20, and
M22 concerns. The active plan records the still-open integration decisions and
validation lanes.
