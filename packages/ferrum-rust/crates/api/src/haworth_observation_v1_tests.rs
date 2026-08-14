use ferrum_document::{DocumentObjectIdV1, DocumentSession, PersistentId};
use ferrum_domain::haworth::RingForm;
use ferrum_render::{Paint, PositiveFinite, RenderOp, Rgb24};

use super::{
    DocumentHaworthObservationErrorV1, DocumentHaworthObservationRequestV1,
    observe_document_haworth_v1,
};

fn pyranose_source(extra: &str) -> String {
    format!(
        concat!(
            "<cdml><molecule id=\"before\"><atom id=\"before_atom\" name=\"C\"><point x=\"-1\" y=\"0\"/></atom></molecule>",
            "<molecule id=\"ring\">",
            "<atom id=\"a0\" name=\"O\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"a1\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
            "<atom id=\"a2\" name=\"C\"><point x=\"2\" y=\"0\"/></atom>",
            "<atom id=\"a3\" name=\"C\"><point x=\"3\" y=\"0\"/></atom>",
            "<atom id=\"a4\" name=\"C\"><point x=\"4\" y=\"0\"/></atom>",
            "<atom id=\"a5\" name=\"C\"><point x=\"5\" y=\"0\"/></atom>",
            "<atom id=\"a6\" name=\"C\"><point x=\"6\" y=\"0\"/></atom>",
            "<bond id=\"b0\" start=\"a0\" end=\"a1\" type=\"n1\"/>",
            "<bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"n1\"/>",
            "<bond id=\"b2\" start=\"a2\" end=\"a3\" type=\"n1\"/>",
            "<bond id=\"b3\" start=\"a3\" end=\"a4\" type=\"n1\"/>",
            "<bond id=\"b4\" start=\"a4\" end=\"a5\" type=\"n1\"/>",
            "<bond id=\"b5\" start=\"a5\" end=\"a0\" type=\"n1\"/>",
            "<molecule id=\"nested\"><atom id=\"nested_atom\" name=\"C\"><point x=\"7\" y=\"0\"/></atom></molecule>",
            "</molecule>{}</cdml>"
        ),
        extra
    )
}

fn request(molecule_id: DocumentObjectIdV1, cycle: &[&str]) -> DocumentHaworthObservationRequestV1 {
    DocumentHaworthObservationRequestV1::new(
        molecule_id,
        RingForm::Pyranose,
        cycle
            .iter()
            .map(|id| PersistentId::new(*id).expect("nonblank source ID"))
            .collect(),
        PersistentId::new("a1").expect("nonblank anomeric ID"),
        PositiveFinite::new(10.0).expect("positive scale"),
        PositiveFinite::new(1.0).expect("positive width"),
        Paint::rgb24(Rgb24::new("000000").expect("paint")),
    )
    .expect("request-local shape")
}

fn observation_and_id(source: &str) -> (DocumentSession, DocumentObjectIdV1) {
    let session = DocumentSession::load(source).expect("source must load");
    let observation = session.observe(0).expect("source must observe");
    let molecule_id = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("ring"))
        .expect("ring root must be projected")
        .id()
        .expect("direct source molecule has durable ID")
        .clone();
    (session, molecule_id)
}

#[test]
fn direct_pyranose_observation_retains_local_and_document_order_domains() {
    let (session, molecule_id) = observation_and_id(&pyranose_source(""));
    let observation = session.observe(0).expect("fresh observation");
    let before = observation.clone();
    let result = observe_document_haworth_v1(
        &observation,
        &request(molecule_id.clone(), &["a0", "a1", "a2", "a3", "a4", "a5"]),
    )
    .expect("isolated C/O cycle must plan");

    assert_eq!(
        result.provenance().revision().get(),
        observation.snapshot().revision()
    );
    assert_eq!(
        result.provenance().digest(),
        *observation.snapshot().digest()
    );
    let projected_root = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.id() == Some(&molecule_id))
        .expect("selected direct root must remain projected");
    assert_eq!(result.root().molecule_id(), &molecule_id);
    assert_eq!(
        result.root().projection_key(),
        projected_root.projection_key().as_str()
    );
    assert_eq!(
        result.root().source_id(),
        projected_root.source_id().expect("source ID")
    );
    assert_eq!(
        result.root().document_root_order(),
        projected_root.source_order()
    );
    assert_ne!(result.root().document_root_order(), 0);
    assert!(result.template_bounds().min_x().is_finite());
    assert!(result.template_bounds().min_y().is_finite());
    assert!(result.template_bounds().max_x() >= result.template_bounds().min_x());
    assert!(result.template_bounds().max_y() >= result.template_bounds().min_y());
    assert_eq!(
        result
            .plan()
            .batches()
            .iter()
            .map(|batch| batch.target().source_order())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );
    assert!(result.plan().batches().iter().any(|batch| {
        batch.operations().len() == 3
            && batch
                .operations()
                .iter()
                .all(|operation| matches!(operation, RenderOp::Line(_)))
    }));
    assert_eq!(observation, before);
}

#[test]
fn cycle_rotation_and_reversal_keep_canonical_selected_bond_order() {
    let (session, molecule_id) = observation_and_id(&pyranose_source(""));
    let observation = session.observe(0).expect("fresh observation");
    let rotated = observe_document_haworth_v1(
        &observation,
        &request(molecule_id.clone(), &["a2", "a3", "a4", "a5", "a0", "a1"]),
    )
    .expect("rotated cycle must plan");
    let reversed = observe_document_haworth_v1(
        &observation,
        &request(molecule_id, &["a1", "a0", "a5", "a4", "a3", "a2"]),
    )
    .expect("reversed cycle must plan");

    let orders = |value: &super::DocumentHaworthObservationV1| {
        value
            .plan()
            .batches()
            .iter()
            .map(|batch| batch.target().source_order())
            .collect::<Vec<_>>()
    };
    assert_eq!(rotated.provenance(), reversed.provenance());
    assert_eq!(orders(&rotated), orders(&reversed));
}

#[test]
fn stale_foreign_nested_and_invalid_topology_are_typed_and_observational() {
    let (session, molecule_id) = observation_and_id(&pyranose_source(
        "<molecule id=\"other\"><atom id=\"foreign\" name=\"C\"><point x=\"9\" y=\"0\"/></atom></molecule>",
    ));
    assert!(
        session.observe(1).is_err(),
        "revision guard precedes adapter"
    );
    let observation = session.observe(0).expect("fresh observation");
    let before_revision = observation.snapshot().revision();
    let before_digest = *observation.snapshot().digest();
    let foreign = DocumentHaworthObservationRequestV1::new(
        molecule_id.clone(),
        RingForm::Pyranose,
        ["a0", "a1", "a2", "a3", "a4", "foreign"]
            .into_iter()
            .map(|id| PersistentId::new(id).expect("ID"))
            .collect(),
        PersistentId::new("a1").expect("ID"),
        PositiveFinite::new(10.0).expect("scale"),
        PositiveFinite::new(1.0).expect("width"),
        Paint::rgb24(Rgb24::new("000000").expect("paint")),
    )
    .expect("request shape");
    assert!(matches!(
        observe_document_haworth_v1(&observation, &foreign),
        Err(DocumentHaworthObservationErrorV1::SelectedAtomNotInMolecule { .. })
    ));
    let nested =
        DocumentObjectIdV1::parse("ferrum-document-object-v1/6d6f6c6563756c65/source/6e6573746564")
            .expect("closed selector grammar");
    assert!(matches!(
        observe_document_haworth_v1(
            &observation,
            &request(nested, &["a0", "a1", "a2", "a3", "a4", "a5"])
        ),
        Err(DocumentHaworthObservationErrorV1::UnknownDirectMolecule { .. })
    ));
    assert!(matches!(
        observe_document_haworth_v1(
            &observation,
            &request(molecule_id.clone(), &["a0", "a1", "a2", "a3", "a4", "a6"])
        ),
        Err(DocumentHaworthObservationErrorV1::Topology(_))
    ));
    assert!(
        observe_document_haworth_v1(
            &observation,
            &request(molecule_id, &["a0", "a1", "a2", "a3", "a4", "a5"])
        )
        .is_ok()
    );
    assert_eq!(observation.snapshot().revision(), before_revision);
    assert_eq!(*observation.snapshot().digest(), before_digest);
}
