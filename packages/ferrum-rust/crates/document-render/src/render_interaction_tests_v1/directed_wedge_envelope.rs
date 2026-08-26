use super::*;

const DIRECTED_WEDGE_CASES: [(&str, &str); 4] =
    [("w1", "12"), ("w1", "3"), ("h1", "12"), ("h1", "3")];

#[test]
fn structural_directed_wedges_use_one_renderer_derived_envelope_across_admitted_widths() {
    for (bond_type, wedge_width) in DIRECTED_WEDGE_CASES {
        assert_directed_wedge_envelope(bond_type, wedge_width);
    }
}

fn assert_directed_wedge_envelope(bond_type: &str, wedge_width: &str) {
    let source = format!(
        concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
            "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"b\" name=\"O\"><point x=\"30\" y=\"0\"/></atom>",
            "<bond id=\"ab\" type=\"{}\" wedge_width=\"{}\" start=\"a\" end=\"b\"/>",
            "</molecule></cdml>",
        ),
        bond_type, wedge_width
    );
    let session = RenderInteractionSessionV1::new(DocumentSession::load(&source).expect("load"));
    let observation = session
        .observe_structure_interaction_v1(fence(&session))
        .expect("observe");
    let bond = observation
        .targets()
        .iter()
        .find(|target| target.kind() == StructureTargetKindV1::Bond)
        .expect("directed wedge has a semantic bond target");
    assert_eq!(
        observation
            .targets()
            .iter()
            .filter(|target| target.object_id() == bond.object_id())
            .count(),
        1,
        "the durable wedge identity has no legacy structural child sibling"
    );
    assert_eq!(bond.kind(), StructureTargetKindV1::Bond);

    let bounds = bond.bounds();
    let shoulder_x = bounds.left() + (bounds.right() - bounds.left()) * 0.625;
    let shoulder_y = bounds.top() + (bounds.bottom() - bounds.top()) * 0.8125;
    let selection = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: shoulder_x,
                y: shoulder_y,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("renderer-derived visible shoulder selects the durable bond");
    assert_eq!(selection.targets(), std::slice::from_ref(bond));

    let marquee = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Marquee {
                left: bounds.left(),
                top: bounds.top(),
                right: bounds.right(),
                bottom: bounds.bottom(),
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("raw renderer-derived envelope is marquee-contained");
    assert_eq!(marquee.targets(), std::slice::from_ref(bond));
}
