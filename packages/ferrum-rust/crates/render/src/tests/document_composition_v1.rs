use ferrum_document_projection::{
    DocumentDirectRootKindV1, DocumentDirectRootV1, DocumentObjectIdV1,
    DocumentProjectionProvenanceV1, DocumentProjectionV1, MoleculeProjectionChildrenV1,
    MoleculeProjectionV1, PaperAttributesV1, PaperLayoutFactsV1, PaperLayoutProjectionV1,
    PaperOrientationV1, PaperPageV1, PositiveFiniteV1, PresentationFactProvenanceV1,
    PresentationProjectionIssueCodeV1, PresentationProjectionIssueV1, PresentationRecordKindV1,
    PresentationRootProjectionV1, PresentationStackProjectionV1, PresentationTargetV1,
    ProjectionLocalObjectKeyV1, ViewportAttributesV1,
};

use crate::{
    DepictionProfileV1, DocumentRenderOutcomeV1, RenderError, compose_document_render_plan_v1,
    resolve_document_render_v2,
};

const REVISION: u64 = 19;
const DIGEST: [u8; 32] = [9; 32];

#[test]
fn composition_uses_interleaved_direct_root_order_including_sparse_rejection() {
    let molecule_id = object_id(1);
    let plus_id = object_id(2);
    let rejected_id = object_id(3);
    let projection = projection(
        vec![molecule(molecule_id.clone())],
        vec![PresentationRootProjectionV1::Plus {
            plus: standard_plus(plus_id.clone()),
        }],
        vec![PresentationProjectionIssueV1::new(
            presentation_target(rejected_id.clone(), PresentationRecordKindV1::Arrow),
            PresentationProjectionIssueCodeV1::UnsupportedArrowType,
            "unsupported arrow",
        )],
        vec![
            DocumentDirectRootV1::new(molecule_id, 2, DocumentDirectRootKindV1::Molecule),
            DocumentDirectRootV1::new(
                plus_id,
                8,
                DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Plus),
            ),
            DocumentDirectRootV1::new(
                rejected_id,
                21,
                DocumentDirectRootKindV1::RejectedPresentation(
                    PresentationProjectionIssueCodeV1::UnsupportedArrowType,
                ),
            ),
        ],
    );

    let observation = resolve_document_render_v2(projection, DepictionProfileV1::ferrum_default())
        .expect("complete direct-root observation");
    let plan = compose_document_render_plan_v1(&observation).expect("direct roots compose");

    assert_eq!(
        plan.outcomes()
            .iter()
            .map(DocumentRenderOutcomeV1::paint_order)
            .collect::<Vec<_>>(),
        vec![2, 8, 21]
    );
    assert!(matches!(
        &plan.outcomes()[2],
        DocumentRenderOutcomeV1::Exclusion(exclusion)
            if exclusion.feature() == "rejected_projection:UnsupportedArrowType"
    ));
}

#[test]
fn composition_refuses_presentation_payload_with_a_mismatched_direct_root_kind() {
    let plus_id = object_id(4);
    let projection = projection(
        Vec::new(),
        vec![PresentationRootProjectionV1::Plus {
            plus: standard_plus(plus_id.clone()),
        }],
        Vec::new(),
        vec![DocumentDirectRootV1::new(
            plus_id,
            5,
            DocumentDirectRootKindV1::Presentation(PresentationRecordKindV1::Text),
        )],
    );

    let observation = resolve_document_render_v2(projection, DepictionProfileV1::ferrum_default())
        .expect("observation preserves payload identity");

    assert!(matches!(
        compose_document_render_plan_v1(&observation),
        Err(crate::DocumentRenderPlanCompositionError::Render(
            RenderError::InvalidRequest(_)
        ))
    ));
}

fn projection(
    molecules: Vec<MoleculeProjectionV1>,
    roots: Vec<PresentationRootProjectionV1>,
    issues: Vec<PresentationProjectionIssueV1>,
    direct_roots: Vec<DocumentDirectRootV1>,
) -> DocumentProjectionV1 {
    DocumentProjectionV1::try_new(
        DocumentProjectionProvenanceV1::new(REVISION, DIGEST, false),
        None,
        paper_layout(),
        molecules,
        direct_roots,
        PresentationStackProjectionV1::new(REVISION, DIGEST, roots, Vec::new(), issues)
            .expect("closed presentation stack"),
        Vec::new(),
    )
    .expect("closed document projection")
}

fn molecule(id: DocumentObjectIdV1) -> MoleculeProjectionV1 {
    MoleculeProjectionV1::try_new(
        id,
        ProjectionLocalObjectKeyV1::from_path_components(&[0])
            .expect("nonempty molecule projection path"),
        None,
        None,
        MoleculeProjectionChildrenV1 {
            atoms: Vec::new(),
            compact_groups: Vec::new(),
            non_atom_vertices: Vec::new(),
            bonds: Vec::new(),
        },
    )
    .expect("empty direct molecule projection")
}

fn standard_plus(id: DocumentObjectIdV1) -> ferrum_document_projection::PlusProjectionV1 {
    let target = presentation_target(id, PresentationRecordKindV1::Plus);
    let font = ferrum_document_projection::PresentationFontV1::try_new(
        ferrum_document_projection::PresentationFontFaceV1::TelexRegularV1,
        PresentationFactProvenanceV1::Builtin,
        PositiveFiniteV1::new(14.0).expect("built-in font size"),
        PresentationFactProvenanceV1::Builtin,
        ferrum_document_projection::Rgb24V1::new("#000000").expect("built-in font colour"),
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("built-in Plus font");
    let background = ferrum_document_projection::PresentationFillV1::try_new(
        None,
        PresentationFactProvenanceV1::Builtin,
    )
    .expect("built-in Plus background");
    ferrum_document_projection::PlusProjectionV1::try_new(
        target,
        ferrum_document_projection::Point3V1::new(20.0, 30.0, 0.0).expect("finite Plus anchor"),
        font,
        background,
    )
    .expect("valid Plus projection")
}

fn presentation_target(
    id: DocumentObjectIdV1,
    kind: PresentationRecordKindV1,
) -> PresentationTargetV1 {
    PresentationTargetV1::new(id, kind)
}

fn paper_layout() -> PaperLayoutProjectionV1 {
    PaperLayoutProjectionV1::new(
        REVISION,
        DIGEST,
        PaperLayoutFactsV1 {
            paper_present: false,
            paper_attributes: PaperAttributesV1::default(),
            effective_paper_attributes: PaperAttributesV1::default(),
            viewport_attributes: ViewportAttributesV1::default(),
            default_type: "A4".to_owned(),
            default_orientation: PaperOrientationV1::Portrait,
            page: PaperPageV1::from_resolved_dimensions(
                PositiveFiniteV1::new(210.0).expect("A4 width"),
                PositiveFiniteV1::new(297.0).expect("A4 height"),
                PaperOrientationV1::Portrait,
                None,
            )
            .expect("finite A4 page"),
        },
    )
}

fn object_id(value: u8) -> DocumentObjectIdV1 {
    DocumentObjectIdV1::from_entropy_bytes([value; 16])
}
