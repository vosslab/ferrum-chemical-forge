use ferrum_core::{Atom, Bond, BondOrder, Identifier, Molecule, Position, VertexRef};
use ferrum_document::{DocumentSession, Point3V1};
use ferrum_domain::haworth::{
    DirectGlycosidicHaworthTopologyV1, HaworthTopologyBuilder, HaworthVertex, RingForm,
    direct_glycosidic_haworth_authoring_receipt_v1,
};

use super::{
    DepictionProfileV1, compose_committed_direct_haworth_document_v1,
    compose_reobserved_direct_haworth_document_v1,
    depiction_profile_v1::resolve_direct_glycosidic_haworth_style_v1,
};

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
    .expect("atom")
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
    .expect("bond")
}

fn authoring_receipt() -> ferrum_domain::haworth::DirectGlycosidicHaworthAuthoringReceiptV1 {
    const FIRST: usize = 5;
    const SECOND: usize = 6;
    let bridge = FIRST + SECOND;
    let mut atoms: Vec<_> = (0..bridge)
        .map(|index| {
            atom(
                index,
                if index == 0 || index == FIRST {
                    "O"
                } else {
                    "C"
                },
            )
        })
        .collect();
    atoms.push(atom(bridge, "O"));
    let mut bonds: Vec<_> = (0..FIRST)
        .map(|index| bond(index, &atoms[index], &atoms[(index + 1) % FIRST]))
        .collect();
    bonds.extend((0..SECOND).map(|index| {
        let start = FIRST + index;
        bond(
            FIRST + index,
            &atoms[start],
            &atoms[FIRST + (index + 1) % SECOND],
        )
    }));
    bonds.push(bond(FIRST + SECOND, &atoms[1], &atoms[bridge]));
    bonds.push(bond(FIRST + SECOND + 1, &atoms[FIRST + 1], &atoms[bridge]));
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
    .expect("source molecule");
    let ring = |offset: usize, form: RingForm, size: usize| {
        let vertices = atoms[offset..offset + size]
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
            ring(0, RingForm::Furanose, FIRST),
            ring(FIRST, RingForm::Pyranose, SECOND),
        ],
        atoms[bridge].identity().clone(),
        [
            bonds[FIRST + SECOND].identity().clone(),
            bonds[FIRST + SECOND + 1].identity().clone(),
        ],
    )
    .expect("direct topology");
    direct_glycosidic_haworth_authoring_receipt_v1(&molecule, topology, 6.0)
        .expect("authoring receipt")
}

#[test]
fn committed_direct_haworth_composes_from_its_accepted_observation() {
    let mut session = DocumentSession::load(concat!(
        "<cdml><standard line_color=\"#123456\" ",
        "line_width=\"2px\"><bond wedge-width=\"7px\"/></standard></cdml>",
    ))
    .expect("document standard");
    let receipt = authoring_receipt();
    let mut pending = session
        .prepare_create_direct_haworth_v1(
            0,
            &receipt,
            Point3V1::new(11.0, -7.0, 0.0).expect("finite anchor"),
        )
        .expect("prepare direct Haworth");
    let committed = session
        .commit_create_direct_haworth_v1(0, &mut pending)
        .expect("commit direct Haworth");
    let style = resolve_direct_glycosidic_haworth_style_v1(
        committed.operation().observation().projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("accepted document standard resolves direct Haworth style");
    assert_eq!(style.paint().color().as_str(), "123456");
    assert_eq!(style.line_width().get(), 2.0);
    assert_eq!(style.wedge_width().get(), 7.0);

    let composite = compose_committed_direct_haworth_document_v1(&committed)
        .expect("committed receipt authenticates its complete bond targets");
    assert_eq!(
        composite.provenance().revision().get(),
        committed.operation().observation().snapshot().revision()
    );
    assert_eq!(
        composite.provenance().digest(),
        *committed.operation().observation().snapshot().digest()
    );
    assert!(composite.page().width() > 0.0);

    let reopened = DocumentSession::load(committed.operation().observation().snapshot().cdml())
        .expect("committed snapshot reopens");
    let selector = reopened
        .observe(0)
        .expect("reopened observation")
        .projection()
        .molecules()[0]
        .id()
        .expect("reopened durable molecule")
        .clone();
    let reobserved = reopened
        .observe_direct_glycosidic_haworth_v1(0, &selector)
        .expect("closed direct profile re-observes");
    let reopened_style = resolve_direct_glycosidic_haworth_style_v1(
        reobserved.observation().projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("reopened accepted standard resolves direct Haworth style");
    assert_eq!(reopened_style.paint().color().as_str(), "123456");
    assert_eq!(reopened_style.line_width().get(), 2.0);
    assert_eq!(reopened_style.wedge_width().get(), 7.0);
    let reobserved_composite = compose_reobserved_direct_haworth_document_v1(&reobserved)
        .expect("re-observed receipt authenticates custom standard and selected bonds");
    assert_eq!(
        reobserved_composite.provenance().revision().get(),
        reobserved.observation().snapshot().revision()
    );
    assert_eq!(
        reobserved_composite.provenance().digest(),
        *reobserved.observation().snapshot().digest()
    );
}
