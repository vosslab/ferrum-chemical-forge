# Ferrum-Chem third-party notices

The `ferrum-chem` wheel carries the following notice texts in its
`*.dist-info/licenses/` directory. The release owner reviews this inventory with
the M20 native-wheel receipt before publication.

| Component | Selected source | Wheel purpose | Included notice |
| --- | --- | --- | --- |
| Ferrum-Chem | committed `LICENSE.LGPL-3.0.md` | Ferrum native extension and `libferrum_chem.dylib` | `FERRUM-CHEM-LGPL-3.0.txt` |
| RDKit | `Release_2026_03_5`, `license.txt` | declared RDKit native dylib closure | `RDKIT-BSD-3-CLAUSE.txt` |
| InChI | `v1.07.3`, `INCHI-1-SRC/INCHI_API/libinchi/src/inchi_dll.c` leading MIT comment | InChI portions of `libRDKitInchi.1.dylib` and `libRDKitRDInchiLib.1.dylib` | `INCHI-MIT.txt` |
| Telex Regular | committed `crates/render/assets/licenses/Telex-OFL-1.1.txt` | embedded native render font bytes | `TELEX-OFL-1.1.txt` |

The M20 receipt records the pinned source archives and the observed native closure
for the wheel under review. If those sources or that closure changes, update this
index and review the copied notice texts before release.
