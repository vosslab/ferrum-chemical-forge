use super::extract_cdsvg;

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";

#[test]
fn extracts_a_verified_cdml_payload() {
    let source = format!(
        concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg"><cdml xmlns="{CDML_NAMESPACE}" "#,
            r#"version="0.16"><paper/></cdml></svg>"#,
        ),
        CDML_NAMESPACE = CDML_NAMESPACE
    );

    let extracted = extract_cdsvg(&source).expect("canonical payload must extract");

    assert!(extracted.contains("<cdml"));
    assert!(extracted.contains("<paper"));
    ferrum_document::TypedDocument::parse(&extracted).expect("published text reparses");
}
