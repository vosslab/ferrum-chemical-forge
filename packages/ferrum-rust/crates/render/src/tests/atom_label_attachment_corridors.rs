//! Cross-style and cross-orientation atom-label attachment behavior.

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document_projection::DocumentObjectIdV1;

use crate::atom_bond::build_atom_bond_plan;
use crate::render_target::RenderPlanEntryContextV1;
use crate::*;

fn size(value: f64) -> PositiveFinite {
    PositiveFinite::new(value).expect("test size is positive")
}

fn paint() -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new("112233").expect("test paint"))
}

fn record_id(kind: RecordKind, id: &str) -> RecordId {
    RecordId::new(kind, Identifier::new(id).expect("test identifier")).expect("test record")
}

fn context(entropy: u8, kind: RecordKind, id: &str, order: u32) -> RenderPlanEntryContextV1 {
    RenderPlanEntryContextV1::new(
        RenderTarget::document_object(DocumentObjectIdV1::from_entropy_bytes([entropy; 16])),
        record_id(kind, id),
        order,
        Some(DocumentObjectIdV1::from_entropy_bytes([0xb6; 16])),
    )
}

fn atom(
    entropy: u8,
    id: &str,
    order: u32,
    position: (f64, f64),
    label: AtomLabelFacts,
) -> AtomRenderTarget {
    AtomRenderTarget::new(
        context(entropy, RecordKind::Atom, id, order),
        RenderPoint::new(position.0, position.1).expect("test position"),
        label,
        TargetVisibility::Visible,
    )
    .expect("test atom")
}

#[test]
fn decorated_endpoint_admits_every_supported_style_under_eight_rotations() {
    let environment =
        FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
    let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
        .expect("verified molecule-label metrics");
    let styles = [
        BondStyle::NormalSingle,
        BondStyle::Double,
        BondStyle::Triple,
        BondStyle::SolidWedge,
        BondStyle::HashedWedge,
        BondStyle::HaworthFrontStroke,
        BondStyle::HaworthFrontWedge,
        BondStyle::Bold,
        BondStyle::Dashed,
        BondStyle::Wavy,
    ];
    let directions = [
        (40.0, 0.0),
        (30.0, 30.0),
        (0.0, 40.0),
        (-30.0, 30.0),
        (-40.0, 0.0),
        (-30.0, -30.0),
        (0.0, -40.0),
        (30.0, -30.0),
    ];
    for style in styles {
        for direction in directions {
            let first = atom(
                0x11,
                "decorated",
                1,
                (0.0, 0.0),
                AtomLabelFacts::new("P", Some(31), 1, 4).expect("decorated label"),
            );
            let second = atom(
                0x12,
                "plain",
                3,
                direction,
                AtomLabelFacts::new("I", None, 0, 0).expect("plain label"),
            );
            let bond = BondRenderTarget::new(
                context(0x13, RecordKind::Bond, "bond", 2),
                record_id(RecordKind::Atom, "decorated"),
                record_id(RecordKind::Atom, "plain"),
                style.clone(),
                TargetVisibility::Visible,
            )
            .expect("test bond");
            let request = AtomBondRenderRequest::new(
                RenderProvenance::new(RenderRevision::new(1).expect("revision"), [7; 32]),
                vec![first, second],
                vec![bond],
                AtomLabelFontProfile::new(FontFace::molecule_label(), size(10.0), paint()),
                size(1.0),
                size(6.0),
                BondInkClearance::new(size(1.25)),
                paint(),
            )
            .expect("test request");
            let plan = build_atom_bond_plan(&request, &metrics).expect("test plan");
            assert!(
                plan.issues().is_empty(),
                "{style:?} at {direction:?}: {:?}",
                plan.issues()
            );
            assert_eq!(plan.batches().len(), 3, "{style:?} at {direction:?}");
        }
    }
}
