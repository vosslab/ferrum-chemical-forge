use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
use ferrum_domain::haworth::{
    DirectGlycosidicHaworthTopologyV1, HaworthTopologyBuilder, HaworthVertex, RingForm,
    direct_glycosidic_haworth_authoring_receipt_v1,
};

use super::super::{
    DocumentDirectHaworthBondRoleV1, DocumentDirectHaworthBondTokenV1, DocumentSession,
    DocumentSessionError, Point3V1,
};

const LEGACY_ISSUE_SOURCE: &str = concat!(
    "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"legacy\"><atom id=\"la\" name=\"C\">",
    "<point x=\"0\" y=\"0\"/></atom><atom id=\"lb\" name=\"O\">",
    "<point x=\"1\" y=\"0\"/></atom>",
    "<bond id=\"legacy-bond\" type=\"q1\" start=\"la\" end=\"lb\" ",
    "haworth_position=\"sideways\"/></molecule></cdml>",
);

fn atom(index: usize, element: &str) -> Atom {
    Atom::new(
        Some(Identifier::new(format!("source-atom-{index}")).expect("identifier")),
        Some(element.to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("position"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("source atom")
}

fn bond(index: usize, start: &Atom, end: &Atom) -> Bond {
    Bond::new(
        Some(Identifier::new(format!("source-bond-{index}")).expect("identifier")),
        VertexRef::Atom(start.identity().clone()),
        VertexRef::Atom(end.identity().clone()),
        None,
        Some(BondOrder::Single),
        None,
        Some(false),
        None,
    )
    .expect("source bond")
}

fn receipt() -> ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringReceiptV1 {
    receipt_for_forms(5, RingForm::Furanose, 6, RingForm::Pyranose)
}

fn receipt_for_forms(
    first_count: usize,
    first_form: RingForm,
    second_count: usize,
    second_form: RingForm,
) -> ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringReceiptV1 {
    let bridge_index = first_count + second_count;
    let mut atoms: Vec<_> = (0..bridge_index)
        .map(|index| {
            atom(
                index,
                if index == 0 || index == first_count {
                    "O"
                } else {
                    "C"
                },
            )
        })
        .collect();
    atoms.push(atom(bridge_index, "O"));
    let mut bonds: Vec<_> = (0..first_count)
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % first_count]))
        .collect();
    bonds.extend((0..second_count).map(|index| {
        let start = first_count + index;
        bond(
            first_count + index,
            &atoms[start],
            &atoms[first_count + (index + 1) % second_count],
        )
    }));
    bonds.push(bond(
        first_count + second_count,
        &atoms[1],
        &atoms[bridge_index],
    ));
    bonds.push(bond(
        first_count + second_count + 1,
        &atoms[first_count + 1],
        &atoms[bridge_index],
    ));
    let molecule = Molecule::new(
        Some(Identifier::new("closed-authoring-source").expect("identifier")),
        None,
        atoms.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds.clone(),
        None,
    )
    .expect("closed source molecule");
    let ring = |offset: usize, form: RingForm, vertex_count: usize| {
        let vertices = atoms[offset..offset + vertex_count]
            .iter()
            .map(|atom| HaworthVertex {
                atom: atom.identity().clone(),
            })
            .collect::<Vec<_>>();
        HaworthTopologyBuilder::new(form, vertices[1].atom.clone(), vertices)
            .build(&molecule)
            .expect("ring topology")
    };
    let topology = DirectGlycosidicHaworthTopologyV1::classify(
        &molecule,
        [
            ring(0, first_form, first_count),
            ring(first_count, second_form, second_count),
        ],
        atoms[bridge_index].identity().clone(),
        [
            bonds[first_count + second_count].identity().clone(),
            bonds[first_count + second_count + 1].identity().clone(),
        ],
    )
    .expect("closed topology");
    direct_glycosidic_haworth_authoring_receipt_v1(&molecule, topology, 6.0)
        .expect("authoring receipt")
}

#[test]
fn direct_haworth_commit_owns_one_complete_ordered_molecule_and_exact_provenance() {
    let receipt = receipt();
    let anchor = Point3V1::new(11.0, -7.0, 2.0).expect("anchor");
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty source loads");

    let mut pending = session
        .prepare_create_direct_haworth_v1(0, &receipt, anchor)
        .expect("renderable direct candidate prepares");
    let result = session
        .commit_create_direct_haworth_v1(0, &mut pending)
        .expect("prepared candidate commits");
    let operation = result.operation().observation();
    let committed = result.receipt();
    assert_eq!(committed.revision(), operation.snapshot().revision());
    assert_eq!(committed.digest(), operation.snapshot().digest());

    let projection = operation.projection();
    assert_eq!(projection.molecules().len(), 1);
    let molecule = projection
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some(committed.molecule_identifier().as_str()))
        .expect("exact inserted molecule root");
    assert_eq!(molecule.atoms().len(), committed.atom_identifiers().len());
    assert_eq!(molecule.bonds().len(), committed.bond_identifiers().len());
    for (index, (atom, fact)) in molecule
        .atoms()
        .iter()
        .zip(receipt.atoms_in_canonical_order())
        .enumerate()
    {
        assert_eq!(
            atom.source_id(),
            Some(committed.atom_identifiers()[index].as_str())
        );
        assert_eq!(atom.source_order(), index as u32);
        assert_eq!(
            atom.element(),
            Some(match fact.element() {
                ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon =>
                    "C",
                ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen =>
                    "O",
            })
        );
        assert_eq!(
            atom.position(),
            Point3V1::new(
                fact.local().x + anchor.x(),
                fact.local().y + anchor.y(),
                anchor.z()
            )
            .expect("translated position")
        );
        assert_eq!(
            (
                atom.formal_charge(),
                atom.isotope(),
                atom.explicit_hydrogens()
            ),
            (None, None, None)
        );
    }
    for (index, (bond, fact)) in molecule
        .bonds()
        .iter()
        .zip(committed.bond_facts())
        .enumerate()
    {
        assert_eq!(bond.source_id(), Some(fact.bond_identifier().as_str()));
        assert_eq!(bond.source_order(), (molecule.atoms().len() + index) as u32);
        assert_eq!(bond.start().source_id(), Some(fact.endpoints()[0].as_str()));
        assert_eq!(bond.end().source_id(), Some(fact.endpoints()[1].as_str()));
        assert_eq!(
            bond.source_type(),
            Some(match fact.token() {
                DocumentDirectHaworthBondTokenV1::Q1 => "q1",
                DocumentDirectHaworthBondTokenV1::W1 => "w1",
                DocumentDirectHaworthBondTokenV1::N1 => "n1",
            })
        );
        assert_eq!(bond.haworth_position(), fact.haworth_position());
        assert_eq!(
            fact.role(),
            if index >= committed.bond_facts().len() - 2 {
                DocumentDirectHaworthBondRoleV1::Bridge
            } else {
                DocumentDirectHaworthBondRoleV1::Ring
            }
        );
    }
    let depiction = committed.authored_depiction();
    assert_eq!(depiction.rings().len(), 2);
    assert_eq!(
        depiction.ring_bonds().len() + depiction.bridge_bonds().len(),
        committed.bond_facts().len()
    );
    assert!(
        depiction
            .ring_bonds()
            .values()
            .all(|bond| bond.authored_child_order()
                < molecule.bonds().len() as u32 + molecule.atoms().len() as u32)
    );
    assert!(
        depiction
            .bridge_bonds()
            .values()
            .all(|bond| bond.authored_child_order() >= molecule.atoms().len() as u32)
    );
}

#[test]
fn direct_haworth_rejects_stale_preparation_without_consuming_generated_identity_or_state() {
    let receipt = receipt();
    let mut rejected =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("source loads");
    let before = rejected.snapshot().expect("baseline");
    assert!(
        rejected
            .prepare_create_direct_haworth_v1(
                1,
                &receipt,
                Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
            )
            .is_err()
    );
    assert_eq!(rejected.snapshot().expect("unchanged snapshot"), before);
    let mut after_invalid = rejected
        .prepare_create_direct_haworth_v1(
            0,
            &receipt,
            Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
        )
        .expect("valid preparation follows rejection");
    let mut clean =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("clean source loads");
    let clean_pending = clean
        .prepare_create_direct_haworth_v1(
            0,
            &receipt,
            Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
        )
        .expect("clean preparation");
    assert_eq!(
        after_invalid.molecule_identifier(),
        clean_pending.molecule_identifier()
    );
    assert_eq!(
        after_invalid.atom_identifiers(),
        clean_pending.atom_identifiers()
    );
    assert_eq!(
        after_invalid.bond_identifiers(),
        clean_pending.bond_identifiers()
    );
    rejected
        .commit_create_direct_haworth_v1(0, &mut after_invalid)
        .expect("still committable");
}

#[test]
fn direct_haworth_pending_is_retryable_before_success_and_reopens_with_same_facts() {
    let receipt = receipt();
    let anchor = Point3V1::new(3.0, 5.0, 0.0).expect("anchor");
    let mut owner =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("owner source");
    let mut foreign =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("foreign source");
    let mut pending = owner
        .prepare_create_direct_haworth_v1(0, &receipt, anchor)
        .expect("prepare");
    assert!(
        foreign
            .commit_create_direct_haworth_v1(0, &mut pending)
            .is_err()
    );
    assert!(matches!(
        owner.commit_create_direct_haworth_v1(1, &mut pending),
        Err(DocumentSessionError::RevisionConflict { .. })
    ));
    let result = owner
        .commit_create_direct_haworth_v1(0, &mut pending)
        .expect("retry succeeds");
    assert!(
        owner
            .commit_create_direct_haworth_v1(1, &mut pending)
            .is_err()
    );
    let reopened = DocumentSession::load(result.operation().observation().snapshot().cdml())
        .expect("saved CDML reopens");
    let reopened_observation = reopened.observe(0).expect("reopened projection");
    let molecule = reopened_observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule.source_id() == Some(result.receipt().molecule_identifier().as_str())
        })
        .expect("durable molecule survives reopen");
    assert_eq!(
        molecule
            .atoms()
            .iter()
            .map(|atom| atom.source_id())
            .collect::<Vec<_>>(),
        result
            .receipt()
            .atom_identifiers()
            .iter()
            .map(|id| Some(id.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        molecule
            .bonds()
            .iter()
            .map(|bond| bond.source_id())
            .collect::<Vec<_>>(),
        result
            .receipt()
            .bond_identifiers()
            .iter()
            .map(|id| Some(id.as_str()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn direct_haworth_reobservation_recovers_a_saved_closed_profile() {
    let receipt = receipt();
    let anchor = Point3V1::new(3.0, 5.0, 2.0).expect("anchor");
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("empty source loads");
    let mut pending = session
        .prepare_create_direct_haworth_v1(0, &receipt, anchor)
        .expect("prepare");
    let committed = session
        .commit_create_direct_haworth_v1(0, &mut pending)
        .expect("commit");
    let saved = committed
        .operation()
        .observation()
        .snapshot()
        .cdml()
        .to_owned();
    let reopened = DocumentSession::load(&saved).expect("saved source reopens");
    let selected = reopened
        .observe(0)
        .expect("reopened observation")
        .projection()
        .molecules()
        .iter()
        .find(|molecule| {
            molecule.source_id() == Some(committed.receipt().molecule_identifier().as_str())
        })
        .and_then(|molecule| molecule.id())
        .expect("reopened durable molecule selector")
        .clone();
    let reobserved = reopened
        .observe_direct_glycosidic_haworth_v1(0, &selected)
        .expect("closed saved profile re-observes");
    assert_eq!(reobserved.molecule(), &selected);
    assert_eq!(reobserved.root_order(), 0);
    assert_eq!(
        reobserved.atom_identifiers(),
        committed.receipt().atom_identifiers()
    );
    assert_eq!(
        reobserved.bond_facts().len(),
        committed.receipt().bond_facts().len()
    );
    for (reobserved_bond, committed_bond) in reobserved
        .bond_facts()
        .iter()
        .zip(committed.receipt().bond_facts())
    {
        assert_eq!(
            reobserved_bond.bond_identifier(),
            committed_bond.bond_identifier()
        );
        assert_eq!(reobserved_bond.endpoints(), committed_bond.endpoints());
        assert_eq!(reobserved_bond.token(), committed_bond.token());
        assert_eq!(
            reobserved_bond.haworth_position(),
            committed_bond.haworth_position()
        );
        assert_eq!(reobserved_bond.role(), committed_bond.role());
    }
    let depiction = reobserved.authored_depiction();
    assert_eq!(
        depiction.coordinates(),
        committed.receipt().authored_depiction().coordinates()
    );
    assert!(
        depiction
            .bounds()
            .into_iter()
            .all(|point| point.x.is_finite() && point.y.is_finite())
    );
    assert_eq!(
        depiction
            .canonical_atoms()
            .iter()
            .map(|atom| atom.authored_child_order())
            .collect::<Vec<_>>(),
        (0..reobserved.atom_identifiers().len() as u32).collect::<Vec<_>>()
    );
    assert_eq!(
        depiction
            .canonical_bonds()
            .iter()
            .map(|bond| bond.authored_child_order())
            .collect::<Vec<_>>(),
        (reobserved.atom_identifiers().len() as u32
            ..reobserved.atom_identifiers().len() as u32 + reobserved.bond_facts().len() as u32)
            .collect::<Vec<_>>()
    );
}

#[test]
fn direct_haworth_refuses_an_unrenderable_complete_candidate_without_mutation() {
    let receipt = receipt();
    let anchor = Point3V1::new(3.0, 5.0, 2.0).expect("anchor");
    let mut session = DocumentSession::load(LEGACY_ISSUE_SOURCE).expect("source loads");
    let baseline = session.snapshot().expect("baseline snapshot");

    let error = session
        .prepare_create_direct_haworth_v1(0, &receipt, anchor)
        .expect_err("renderer admission rejects a complete candidate with excluded roots");

    assert_eq!(
        error.to_string(),
        "invalid direct Haworth insertion: candidate was refused by renderer admission"
    );
    assert_eq!(session.snapshot().expect("refusal is inert"), baseline);
}

#[test]
fn direct_haworth_reobservation_rejects_selected_profile_mutations() {
    let receipt = receipt();
    let mut session =
        DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("source loads");
    let mut pending = session
        .prepare_create_direct_haworth_v1(
            0,
            &receipt,
            Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
        )
        .expect("prepare");
    let committed = session
        .commit_create_direct_haworth_v1(0, &mut pending)
        .expect("commit");
    let source = committed
        .operation()
        .observation()
        .snapshot()
        .cdml()
        .to_owned();
    let molecule = committed.receipt().molecule_identifier().as_str();
    let w = committed
        .receipt()
        .bond_facts()
        .iter()
        .find(|bond| bond.token() == DocumentDirectHaworthBondTokenV1::W1)
        .expect("closed receipt has a shoulder");
    let bridge = committed
        .receipt()
        .bond_facts()
        .iter()
        .find(|bond| bond.role() == DocumentDirectHaworthBondRoleV1::Bridge)
        .expect("closed receipt has bridge");
    let shoulder = format!(
        "start=\"{}\" end=\"{}\"",
        w.endpoints()[0],
        w.endpoints()[1]
    );
    let reversed_shoulder = format!(
        "start=\"{}\" end=\"{}\"",
        w.endpoints()[1],
        w.endpoints()[0]
    );
    let bridge_direction = format!(
        "start=\"{}\" end=\"{}\"",
        bridge.endpoints()[0],
        bridge.endpoints()[1]
    );
    let reversed_bridge = format!(
        "start=\"{}\" end=\"{}\"",
        bridge.endpoints()[1],
        bridge.endpoints()[0]
    );
    let mutations = [
        (
            "shoulder direction",
            source.replacen(&shoulder, &reversed_shoulder, 1),
        ),
        (
            "role and depth",
            source.replacen("type=\"q1\"", "type=\"n1\"", 1),
        ),
        (
            "extra molecule attribute",
            source.replacen(
                &format!("<molecule id=\"{molecule}\""),
                &format!("<molecule id=\"{molecule}\" name=\"extra\""),
                1,
            ),
        ),
        (
            "bridge direction",
            source.replacen(&bridge_direction, &reversed_bridge, 1),
        ),
    ];
    for (label, source) in mutations {
        let reopened =
            DocumentSession::load(&source).expect("mutated source remains retained CDML");
        let selected = reopened
            .observe(0)
            .expect("mutated source remains projectable")
            .projection()
            .molecules()
            .iter()
            .find(|candidate| candidate.source_id() == Some(molecule))
            .and_then(|candidate| candidate.id())
            .expect("selected durable molecule")
            .clone();
        assert!(
            reopened
                .observe_direct_glycosidic_haworth_v1(0, &selected)
                .is_err(),
            "{label} must reject the selected closed profile"
        );
    }
}

#[test]
fn direct_haworth_reobservation_accepts_each_closed_ring_form_pair() {
    for (first_count, first_form, second_count, second_form) in [
        (5, RingForm::Furanose, 5, RingForm::Furanose),
        (5, RingForm::Furanose, 6, RingForm::Pyranose),
        (6, RingForm::Pyranose, 5, RingForm::Furanose),
        (6, RingForm::Pyranose, 6, RingForm::Pyranose),
    ] {
        let receipt = receipt_for_forms(first_count, first_form, second_count, second_form);
        let mut session =
            DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"/>").expect("source loads");
        let mut pending = session
            .prepare_create_direct_haworth_v1(
                0,
                &receipt,
                Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
            )
            .expect("prepare");
        let committed = session
            .commit_create_direct_haworth_v1(0, &mut pending)
            .expect("commit");
        let reopened = DocumentSession::load(committed.operation().observation().snapshot().cdml())
            .expect("saved source reopens");
        let selected = reopened
            .observe(0)
            .expect("reopened observation")
            .projection()
            .molecules()[0]
            .id()
            .expect("durable selector")
            .clone();
        let reobserved = reopened
            .observe_direct_glycosidic_haworth_v1(0, &selected)
            .expect("closed source order remains a re-observable profile");
        assert_eq!(
            reobserved
                .authored_depiction()
                .rings()
                .each_ref()
                .map(|ring| ring.ring_form()),
            [first_form, second_form]
        );
    }
}

#[test]
fn direct_haworth_reobservation_stale_foreign_and_non_molecule_selectors_do_not_mutate() {
    let session = DocumentSession::load("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>").expect("source loads");
    let before = session.snapshot().expect("before snapshot");
    let atom = session
        .observe(0)
        .expect("baseline observation")
        .projection()
        .molecules()[0]
        .atoms()[0]
        .id()
        .expect("atom selector")
        .clone();
    let foreign = super::super::DocumentObjectIdV1::from_class_source("molecule", "foreign")
        .expect("nonempty primitives produce a durable identity");
    for (revision, selector) in [(1, &atom), (0, &atom), (0, &foreign)] {
        assert!(
            session
                .observe_direct_glycosidic_haworth_v1(revision, selector)
                .is_err()
        );
        assert_eq!(session.snapshot().expect("unchanged snapshot"), before);
    }
}
