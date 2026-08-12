use crate::haworth::{HaworthError, HaworthTopologyBuilder, RingForm};

use super::fixtures::molecule;

#[test]
fn rejects_noncanonical_topology_facts() {
    let (ring_molecule, vertices) = molecule(RingForm::Pyranose, false, &[], &[]);
    let missing = HaworthTopologyBuilder::new(
        RingForm::Pyranose,
        vertices[2].atom.clone(),
        vertices.clone(),
    )
    .build(&ring_molecule)
    .expect_err("non-adjacent anomeric carbon must reject");
    assert_eq!(
        missing,
        HaworthError::InvalidSpec("anomeric atom must be adjacent to ring oxygen")
    );

    let (furanose, mut duplicate) = molecule(RingForm::Furanose, false, &[], &[]);
    duplicate[4] = duplicate[3].clone();
    let error =
        HaworthTopologyBuilder::new(RingForm::Furanose, duplicate[1].atom.clone(), duplicate)
            .build(&furanose)
            .expect_err("duplicate selected vertex must reject");
    assert_eq!(
        error,
        HaworthError::UnsupportedTopology("ring vertices must be distinct")
    );
}

#[test]
fn rejects_chord_fused_and_spiro_topology_reuse() {
    let (chord, vertices) = molecule(RingForm::Pyranose, false, &[], &[(1, 3)]);
    let error = HaworthTopologyBuilder::new(RingForm::Pyranose, vertices[1].atom.clone(), vertices)
        .build(&chord)
        .expect_err("chord must reject");
    assert_eq!(
        error,
        HaworthError::UnsupportedTopology(
            "initial profile requires an isolated chordless single cycle"
        )
    );

    let (fused, vertices) = molecule(RingForm::Pyranose, false, &["C"], &[(2, 6), (6, 3)]);
    let error = HaworthTopologyBuilder::new(RingForm::Pyranose, vertices[1].atom.clone(), vertices)
        .build(&fused)
        .expect_err("fused path must reject");
    assert_eq!(
        error,
        HaworthError::UnsupportedTopology(
            "initial profile requires an isolated chordless single cycle"
        )
    );

    let (spiro, vertices) = molecule(
        RingForm::Pyranose,
        false,
        &["C", "C"],
        &[(2, 6), (6, 7), (7, 2)],
    );
    let error = HaworthTopologyBuilder::new(RingForm::Pyranose, vertices[1].atom.clone(), vertices)
        .build(&spiro)
        .expect_err("spiro reuse must reject");
    assert_eq!(
        error,
        HaworthError::UnsupportedTopology("initial profile excludes fused and spiro ring reuse")
    );
}
