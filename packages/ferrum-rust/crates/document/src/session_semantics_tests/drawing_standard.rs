//! Revision-bound drawing-standard projection and atomic mutation behavior.

use super::{DocumentSession, SessionOperation, SessionOperationV1};
use crate::{
    DrawingStandardPatchV1, DrawingStandardPatchV1Error, DrawingStandardPropertyChangeV1, Rgb24V1,
    TransparentOrRgb24V1, VisibilityV1,
};

const EXISTING: &str = concat!(
    "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\" ",
    "xmlns:v=\"urn:vendor\"><c:info/><v:before/><c:metadata/>",
    "<c:standard line_width=\"1\" font_size=\"12\" font_family=\"Telex\" ",
    "line_color=\"#000000\" area_color=\"\" paper_type=\"Letter\" v:keep=\"yes\">",
    "<c:bond width=\"6\" wedge-width=\"5\" double-ratio=\"0.75\" ",
    "v:bond=\"keep\"><v:child/></c:bond><c:atom show_hydrogens=\"0\"/>",
    "</c:standard><c:molecule id=\"m\"/><c:standard line_width=\"99\"/>",
    "</c:cdml>"
);

fn operation(changes: Vec<DrawingStandardPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetDrawingStandard {
        patch: DrawingStandardPatchV1::new(changes).expect("valid drawing-standard patch"),
    })
}

#[test]
fn drawing_standard_patch_preserves_opaque_source_and_history() {
    let mut session = DocumentSession::load(EXISTING).expect("source must load");
    let result = session
        .submit(
            0,
            operation(vec![
                DrawingStandardPropertyChangeV1::LineWidth(2.5),
                DrawingStandardPropertyChangeV1::FontSize(18),
                DrawingStandardPropertyChangeV1::FontFamily("  Fira Sans  ".to_owned()),
                DrawingStandardPropertyChangeV1::LineColor(rgb("#123456")),
                DrawingStandardPropertyChangeV1::AreaColor(Some(rgb("#abcdef"))),
                DrawingStandardPropertyChangeV1::BondWidth(7.5),
                DrawingStandardPropertyChangeV1::WedgeWidth(8.5),
                DrawingStandardPropertyChangeV1::DoubleRatio(0.6),
                DrawingStandardPropertyChangeV1::ShowHydrogens(true),
            ]),
        )
        .expect("drawing-standard patch must commit");
    let standard = result
        .observation()
        .projection()
        .drawing_standard()
        .expect("first standard must project");
    assert_eq!(standard.line_width().unwrap().value(), 2.5);
    assert_eq!(standard.font_size().unwrap().value(), 18.0);
    assert_eq!(standard.font_family(), Some("Fira Sans"));
    assert_eq!(standard.line_color().unwrap().as_str(), "#123456");
    assert_eq!(
        standard.area_color(),
        Some(&TransparentOrRgb24V1::Rgb24(rgb("#abcdef")))
    );
    assert_eq!(standard.bond_width().unwrap().value(), 7.5);
    assert_eq!(standard.wedge_width().unwrap().value(), 8.5);
    assert_eq!(standard.double_ratio().unwrap().value(), 0.6);
    assert_eq!(standard.show_hydrogens(), Some(VisibilityV1::Enabled));
    let xml = result.observation().snapshot().cdml();
    assert!(xml.contains("paper_type=\"Letter\""));
    assert!(xml.contains("v:keep=\"yes\""));
    assert!(xml.contains("v:bond=\"keep\""));
    assert!(xml.contains("<v:child/>"));
    assert!(xml.contains("<c:standard line_width=\"99\"/>"));

    let undone = session.undo(1).expect("standard patch must undo");
    assert_eq!(
        undone
            .observation()
            .projection()
            .drawing_standard()
            .unwrap()
            .line_width()
            .unwrap()
            .value(),
        1.0
    );
    let redone = session.redo(2).expect("standard patch must redo");
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml())
        .expect("saved snapshot must reopen");
    let reopened = reopened.observe(0).expect("reopened standard must project");
    assert_eq!(
        reopened
            .projection()
            .drawing_standard()
            .unwrap()
            .double_ratio()
            .unwrap()
            .value(),
        0.6
    );
}

#[test]
fn drawing_standard_creation_is_ordered_and_empty_patch_is_a_noop() {
    let source = concat!(
        "<cdml xmlns:v=\"urn:vendor\"><info/><v:between/><metadata/>",
        "<molecule id=\"m\"/></cdml>"
    );
    let mut session = DocumentSession::load(source).expect("source must load");
    let empty = session
        .submit(0, operation(vec![]))
        .expect("empty patch must be accepted");
    assert_eq!(empty.observation().snapshot().revision(), 0);
    assert!(empty
        .observation()
        .projection()
        .drawing_standard()
        .is_none());

    let result = session
        .submit(
            0,
            operation(vec![
                DrawingStandardPropertyChangeV1::AreaColor(None),
                DrawingStandardPropertyChangeV1::ShowHydrogens(false),
                DrawingStandardPropertyChangeV1::DoubleRatio(1.0),
            ]),
        )
        .expect("first standard must be created");
    let xml = result.observation().snapshot().cdml();
    assert!(xml.find("<metadata").unwrap() < xml.find("<standard").unwrap());
    assert!(xml.find("<standard").unwrap() < xml.find("<molecule").unwrap());
    assert!(xml.contains("area_color=\"\""));
    assert!(xml.contains("show_hydrogens=\"0\""));
    assert_eq!(
        result
            .observation()
            .projection()
            .drawing_standard()
            .unwrap()
            .area_color(),
        Some(&TransparentOrRgb24V1::Transparent)
    );
}

#[test]
fn drawing_standard_patch_rejects_duplicate_and_unrepresentable_values() {
    assert_eq!(
        DrawingStandardPatchV1::new(vec![
            DrawingStandardPropertyChangeV1::LineWidth(1.0),
            DrawingStandardPropertyChangeV1::LineWidth(2.0),
        ]),
        Err(DrawingStandardPatchV1Error::DuplicateChange)
    );
    for value in [0.0, -1.0, 1000.1, f64::NAN, f64::INFINITY] {
        assert_eq!(
            DrawingStandardPatchV1::new(vec![DrawingStandardPropertyChangeV1::WedgeWidth(value),]),
            Err(DrawingStandardPatchV1Error::WidthOutOfRange)
        );
    }
    assert_eq!(
        DrawingStandardPatchV1::new(vec![DrawingStandardPropertyChangeV1::FontSize(3)]),
        Err(DrawingStandardPatchV1Error::FontSizeOutOfRange)
    );
    assert_eq!(
        DrawingStandardPatchV1::new(vec![DrawingStandardPropertyChangeV1::FontFamily(
            "   ".to_owned(),
        )]),
        Err(DrawingStandardPatchV1Error::InvalidFontFamily)
    );
    assert_eq!(
        DrawingStandardPatchV1::new(vec![DrawingStandardPropertyChangeV1::DoubleRatio(1.1)]),
        Err(DrawingStandardPatchV1Error::DoubleRatioOutOfRange)
    );
}

fn rgb(value: &str) -> Rgb24V1 {
    Rgb24V1::new(value).expect("test color must be valid")
}
