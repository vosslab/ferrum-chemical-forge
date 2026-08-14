//! Revision-bound paper observation and atomic mutation behavior.

use xot::{Node, Xot};

use super::{
    DocumentSession, DocumentSessionError, SessionOperation, SessionOperationError,
    SessionOperationV1,
};
use crate::{
    PaperDimensionsMmV1, PaperOrientationV1, PaperPageIssueV1, PaperPropertiesPatchV1,
    PaperPropertiesPatchV1Error, PaperPropertyChangeV1, element_name,
};

const EXISTING: &str = concat!(
    "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\" ",
    "xmlns:v=\"urn:vendor\"><c:standard paper_type=\"Letter\" ",
    "paper_orientation=\"landscape\"/><c:paper type=\"legacy-preserve\" ",
    "orientation=\"portrait\" size_x=\"123\" size_y=\"456\" v:raw=\"keep\">",
    "<v:extension key=\"x\">payload</v:extension></c:paper>",
    "<v:note id=\"before\"/><c:paper type=\"A4\" orientation=\"portrait\" ",
    "v:second=\"untouched\"><v:later/></c:paper>",
    "<c:viewport id=\"view\" viewport=\"0 0 10 10\"/></c:cdml>"
);

fn operation(changes: Vec<PaperPropertyChangeV1>) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::SetPaperProperties {
        patch: PaperPropertiesPatchV1::new(changes).expect("valid paper patch"),
    })
}

#[test]
fn paper_projection_uses_first_core_records_and_valid_standard_defaults() {
    let source = concat!(
        "<cdml xmlns:v=\"urn:vendor\"><v:paper type=\"vendor\"/>",
        "<standard paper_type=\"Letter\" paper_orientation=\"landscape\"/>",
        "<viewport id=\"view\" viewport=\"1 2 3 4\"/></cdml>"
    );
    let session = DocumentSession::load(source).expect("source must load");
    let observation = session.observe(0).expect("paper projection must succeed");
    let paper = observation.projection().paper_layout();
    assert!(!paper.paper_present());
    assert_eq!(paper.default_type(), "Letter");
    assert_eq!(paper.default_orientation(), PaperOrientationV1::Landscape);
    assert_eq!(paper.paper_attributes().type_name(), None);
    assert_eq!(
        paper.effective_paper_attributes().type_name(),
        Some("Letter")
    );
    assert_eq!(
        paper.effective_paper_attributes().orientation(),
        Some("landscape")
    );
    assert_eq!(paper.viewport_attributes().id(), Some("view"));
    assert_eq!(paper.viewport_attributes().viewport(), Some("1 2 3 4"));
    assert_eq!(
        (paper.page().width_mm(), paper.page().height_mm()),
        (279.4, 215.9)
    );
    assert_eq!(paper.page().issue(), None);
    assert_eq!(paper.revision(), observation.snapshot().revision());
    assert_eq!(paper.digest(), observation.snapshot().digest());

    let fallback = DocumentSession::load(
        "<cdml><standard paper_type=\"custom\" paper_orientation=\"sideways\"/></cdml>",
    )
    .expect("fallback source must load");
    let paper = fallback
        .observe(0)
        .unwrap()
        .projection()
        .paper_layout()
        .clone();
    assert_eq!(paper.default_type(), "A4");
    assert_eq!(paper.default_orientation(), PaperOrientationV1::Portrait);
    assert_eq!(
        (paper.page().width_mm(), paper.page().height_mm()),
        (210.0, 297.0)
    );
    assert_eq!(paper.page().issue(), None);
}

#[test]
fn paper_patch_preserves_opaque_content_later_paper_order_and_history() {
    let mut session = DocumentSession::load(EXISTING).expect("source must load");
    let changed = session
        .submit(
            0,
            operation(vec![
                PaperPropertyChangeV1::Orientation(PaperOrientationV1::Landscape),
                PaperPropertyChangeV1::CropSvg(true),
                PaperPropertyChangeV1::CropMargin(0),
            ]),
        )
        .expect("paper patch must commit");
    let projected = changed
        .observation()
        .projection()
        .paper_layout()
        .paper_attributes();
    assert_eq!(projected.type_name(), Some("legacy-preserve"));
    assert_eq!(projected.orientation(), Some("landscape"));
    assert_eq!(projected.crop_svg(), Some("1"));
    assert_eq!(projected.crop_margin(), Some("0"));
    assert_eq!(projected.size_x(), Some("123"));
    assert_eq!(projected.size_y(), Some("456"));
    let page = changed.observation().projection().paper_layout().page();
    assert_eq!((page.width_mm(), page.height_mm()), (210.0, 297.0));
    assert_eq!(page.issue(), Some(PaperPageIssueV1::UnsupportedType));

    let xml = changed.observation().snapshot().cdml();
    assert!(xml.contains("v:raw=\"keep\""));
    assert!(xml.contains("<v:extension key=\"x\">payload</v:extension>"));
    assert!(xml.contains("v:second=\"untouched\""));
    let (tree, root) = parsed_root(xml);
    assert_eq!(
        direct_element_names(&tree, root),
        vec!["standard", "paper", "note", "paper", "viewport",]
    );

    let undone = session.undo(1).expect("paper patch must undo");
    assert_eq!(
        undone
            .observation()
            .projection()
            .paper_layout()
            .paper_attributes()
            .orientation(),
        Some("portrait")
    );
    let redone = session.redo(2).expect("paper patch must redo");
    assert_eq!(
        redone
            .observation()
            .projection()
            .paper_layout()
            .paper_attributes()
            .orientation(),
        Some("landscape")
    );
}

#[test]
fn paper_creation_custom_transition_and_invalid_intent_are_atomic() {
    let source = concat!(
        "<c:cdml xmlns:c=\"http://www.freesoftware.fsf.org/bkchem/cdml\" ",
        "xmlns:v=\"urn:vendor\"><c:standard paper_type=\"Letter\" ",
        "paper_orientation=\"landscape\"/><v:note/><c:viewport/></c:cdml>"
    );
    let mut session = DocumentSession::load(source).expect("source must load");
    let empty = session
        .submit(0, operation(vec![]))
        .expect("empty patch is accepted");
    assert_eq!(empty.observation().snapshot().revision(), 0);
    assert!(
        !empty
            .observation()
            .projection()
            .paper_layout()
            .paper_present()
    );

    let custom_size = PaperDimensionsMmV1::try_new(200.5, 300.25).unwrap();
    let custom = session
        .submit(
            0,
            operation(vec![
                PaperPropertyChangeV1::Type("custom".to_owned()),
                PaperPropertyChangeV1::Dimensions(custom_size),
                PaperPropertyChangeV1::ReplaceMinus(true),
            ]),
        )
        .expect("custom paper must commit");
    let paper = custom
        .observation()
        .projection()
        .paper_layout()
        .paper_attributes();
    assert_eq!(paper.type_name(), Some("custom"));
    assert_eq!(paper.orientation(), Some("landscape"));
    assert_eq!(paper.size_x(), Some("200.5"));
    assert_eq!(paper.size_y(), Some("300.25"));
    assert_eq!(paper.replace_minus(), Some("1"));
    let page = custom.observation().projection().paper_layout().page();
    assert_eq!((page.width_mm(), page.height_mm()), (300.25, 200.5));
    assert_eq!(page.issue(), None);
    let (tree, root) = parsed_root(custom.observation().snapshot().cdml());
    assert_eq!(
        direct_element_names(&tree, root),
        vec!["standard", "note", "paper", "viewport"]
    );

    let named = session
        .submit(
            1,
            operation(vec![PaperPropertyChangeV1::Type("C10".to_owned())]),
        )
        .expect("named paper must commit");
    let paper = named
        .observation()
        .projection()
        .paper_layout()
        .paper_attributes();
    assert_eq!(paper.type_name(), Some("C10"));
    assert_eq!(paper.size_x(), None);
    assert_eq!(paper.size_y(), None);

    assert_eq!(
        PaperPropertiesPatchV1::new(vec![
            PaperPropertyChangeV1::CropSvg(true),
            PaperPropertyChangeV1::CropSvg(false),
        ]),
        Err(PaperPropertiesPatchV1Error::DuplicateChange)
    );
    assert_eq!(
        PaperPropertiesPatchV1::new(vec![PaperPropertyChangeV1::Type("obsolete".to_owned()),]),
        Err(PaperPropertiesPatchV1Error::UnsupportedType)
    );
    let before = session.snapshot().expect("snapshot must work");
    assert!(matches!(
        session.submit(
            2,
            operation(vec![PaperPropertyChangeV1::Dimensions(custom_size),])
        ),
        Err(DocumentSessionError::Operation(
            SessionOperationError::PaperDimensionsRequireCustom
        ))
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
    assert!(matches!(
        session.submit(1, operation(vec![PaperPropertyChangeV1::CropMargin(12),])),
        Err(DocumentSessionError::RevisionConflict {
            expected: 1,
            actual: 2
        })
    ));
    assert_eq!(session.snapshot().expect("snapshot must work"), before);
}

fn parsed_root(source: &str) -> (Xot, Node) {
    let mut tree = Xot::new();
    let document = tree
        .parse(source)
        .expect("accepted candidate XML must parse");
    let root = tree.document_element(document).expect("CDML has one root");
    (tree, root)
}

fn direct_element_names(tree: &Xot, root: Node) -> Vec<String> {
    tree.children(root)
        .filter_map(|node| element_name(tree, node).map(|(local, _)| local))
        .collect()
}
