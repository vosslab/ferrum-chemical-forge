use ferrum_document::DocumentSession;
use serde_json::Value;

use crate::{RenderObservationWireV1, observe_render_v1};

#[test]
fn initial_revision_observation_has_one_matching_document_and_plan_provenance() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");

    assert_eq!(observation.document().snapshot().revision(), 0);
    assert_eq!(
        observation.document().snapshot().digest(),
        observation.document().projection().digest()
    );
    assert!(
        observation
            .molecule_plans()
            .iter()
            .all(|plan| plan.revision().get() == observation.document().snapshot().revision())
    );
    assert_eq!(
        observation.molecule_plans()[0].molecule().source_id(),
        Some("m")
    );
    let wire = serde_json::to_value(observation.wire()).expect("wire");
    let glyphs = &wire["molecule_plans"][0]["plan"]["batches"][0]["operations"][0]["operation"]["runs"]
        [0]["glyphs"];
    assert_eq!(glyphs.as_array().expect("glyph placements").len(), 1);
    assert!(glyphs[0]["glyph_index"].as_u64().is_some());
    assert!(glyphs[0]["origin"].is_object());
}

#[test]
fn molecule_plan_roots_retain_document_order_across_presentation_content() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><polyline id=\"line\"><point x=\"0\" y=\"0\"/><point x=\"1\" y=\"1\"/></polyline><molecule id=\"last\"><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let [first, last] = observation.molecule_plans() else {
        panic!("expected one render entry for each molecule");
    };

    assert_eq!(
        (
            first.molecule().source_order(),
            last.molecule().source_order()
        ),
        (0, 2)
    );
    assert_eq!(
        (
            first.batches()[0].target().source_order(),
            last.batches()[0].target().source_order(),
        ),
        (0, 0)
    );
}

#[test]
fn plus_layout_is_verified_centered_and_closed_against_font_substitution() {
    let session = DocumentSession::load(
        "<cdml><plus id=\"p\" font_size=\"18\" color=\"#123456\" background-color=\"#abcdef\"><point x=\"10\" y=\"20\"/></plus><plus id=\"family\"><point x=\"30\" y=\"40\"/><font family=\"Arial\"/></plus></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let [plus] = observation.plus_renders() else {
        panic!("only the verified-face plus is renderable");
    };
    assert_eq!(plus.target().source_id(), Some("p"));
    assert_eq!((plus.anchor().x(), plus.anchor().y()), (10.0, 20.0));
    let [run] = plus.operation().runs() else {
        panic!("fixed plus content has one run");
    };
    assert_eq!(run.text(), "+");
    assert_eq!(run.glyphs().len(), 1);
    assert_eq!(plus.operation().paint().color().as_str(), "123456");
    assert_eq!(plus.background().unwrap().color().as_str(), "abcdef");
    let bounds = plus.bounds();
    assert!(bounds.left() < 0.0 && bounds.right() > 0.0);
    assert!(bounds.top() < 0.0 && bounds.bottom() > 0.0);
    assert_eq!(observation.issues().len(), 1);
    assert!(observation.issues()[0].detail().contains("Arial"));

    let wire = observation.wire();
    let encoded = wire.to_canonical_json().expect("wire JSON");
    assert_eq!(
        RenderObservationWireV1::from_json(&encoded).expect("wire"),
        wire
    );
    let mut forged: Value = serde_json::from_str(&encoded).expect("JSON");
    let glyph = &mut forged["plus_renders"][0]["operation"]["runs"][0]["glyphs"][0]["glyph_index"];
    *glyph = Value::from(glyph.as_u64().expect("glyph") + 1);
    assert!(RenderObservationWireV1::from_json(&forged.to_string()).is_err());
}

#[test]
fn text_layout_preserves_lines_scripts_spaces_and_exact_verified_glyphs() {
    let session = DocumentSession::load(
        "<cdml><text id=\"label\" background-color=\"#abcdef\"><point x=\"10\" y=\"20\"/><font size=\"18\" color=\"#123456\"/><ftext>Line one\nH&lt;sub&gt;2&lt;/sub&gt;O</ftext></text></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let [text] = observation.text_renders() else {
        panic!("expected one verified Text render");
    };
    assert_eq!(text.target().source_id(), Some("label"));
    assert_eq!((text.anchor().x(), text.anchor().y()), (10.0, 20.0));
    assert_eq!(text.operation().size().get(), 18.0);
    assert_eq!(text.operation().paint().color().as_str(), "123456");
    assert_eq!(text.background().unwrap().color().as_str(), "abcdef");
    assert_eq!(
        text.source_runs()
            .iter()
            .map(|run| (run.text(), run.script()))
            .collect::<Vec<_>>(),
        vec![
            ("Line one\nH", ferrum_render::TextScript::Baseline),
            ("2", ferrum_render::TextScript::Subscript),
            ("O", ferrum_render::TextScript::Baseline),
        ]
    );
    assert_eq!(text.operation().runs()[0].text(), "Line one");
    assert_eq!(text.operation().runs()[1].text(), "H");
    assert!(text.operation().runs()[1].origin().y() > text.operation().runs()[0].origin().y());
    assert!(text.bounds().left() <= 0.0 && text.bounds().top() <= 0.0);
    assert!(text.bounds().right() > 0.0 && text.bounds().bottom() > 0.0);

    let wire = observation.wire();
    let encoded = wire.to_canonical_json().expect("wire JSON");
    assert_eq!(
        RenderObservationWireV1::from_json(&encoded).expect("wire"),
        wire
    );
    let mut forged: Value = serde_json::from_str(&encoded).expect("JSON");
    let glyph = &mut forged["text_renders"][0]["operation"]["runs"][0]["glyphs"][0]["glyph_index"];
    *glyph = Value::from(3);
    assert!(RenderObservationWireV1::from_json(&forged.to_string()).is_err());
}

#[test]
fn unsupported_text_faces_styles_and_glyphs_are_targeted_issues_without_substitution() {
    let session = DocumentSession::load(
        "<cdml><text id=\"family\"><point x=\"0\" y=\"0\"/><font family=\"Arial\"/><ftext>family</ftext></text><text id=\"bold\"><point x=\"0\" y=\"20\"/><ftext>&lt;b&gt;bold&lt;/b&gt;</ftext></text><text id=\"glyph\"><point x=\"0\" y=\"40\"/><ftext>&#x1F600;</ftext></text></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    assert!(observation.text_renders().is_empty());
    assert_eq!(
        observation
            .issues()
            .iter()
            .map(|issue| (issue.code(), issue.target()))
            .collect::<Vec<_>>(),
        vec![
            (
                crate::DepictionIssueCodeV1::UnsupportedAuthoredFontFamily,
                "ferrum-projection-local-v1/0",
            ),
            (
                crate::DepictionIssueCodeV1::UnsupportedTextStyle,
                "ferrum-projection-local-v1/1",
            ),
            (
                crate::DepictionIssueCodeV1::UnsupportedFeature,
                "ferrum-projection-local-v1/2",
            ),
        ]
    );
}

#[test]
fn stale_revision_rejects_before_a_render_observation_is_created() {
    let session = DocumentSession::load("<cdml/>").expect("session");
    let error = observe_render_v1(&session, 1).expect_err("stale revision");

    assert!(
        error
            .to_string()
            .contains("expected 1, current revision is 0")
    );
}

#[test]
fn invalid_presentation_is_a_closed_suppression_with_no_default_render_plan() {
    let session = DocumentSession::load(
        "<cdml><standard line_color=\"blue\"/><molecule id=\"m\"><atom id=\"a\" name=\"N\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");

    assert!(observation.molecule_plans().is_empty());
    assert!(observation.suppression().is_some());
    assert!(!observation.issues().is_empty());
}

#[test]
fn unsupported_bond_is_an_exact_plan_exclusion_not_a_single_line_fallback() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"w2\"/></molecule></cdml>",
    )
    .expect("session");
    let observation = observe_render_v1(&session, 0).expect("observation");
    let plan = &observation.molecule_plans()[0];

    assert!(
        plan.batches()
            .iter()
            .all(|batch| batch.target().record_id().kind() != ferrum_core::RecordKind::Bond)
    );
    assert_eq!(plan.issues().len(), 1);
    assert!(matches!(
        plan.issues()[0].kind(),
        ferrum_render::RenderIssueKind::UnsupportedFeature { feature }
            if feature.contains("Some(\"w2\")")
    ));
}

#[test]
fn wire_is_strict_and_closes_revision_provenance_mismatches() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let wire = observe_render_v1(&session, 0).expect("observation").wire();
    let encoded = wire.to_canonical_json().expect("wire JSON");
    assert_eq!(
        RenderObservationWireV1::from_json(&encoded).expect("wire"),
        wire
    );

    let mut unknown: Value = serde_json::from_str(&encoded).expect("JSON");
    unknown["unexpected"] = Value::Bool(true);
    assert!(RenderObservationWireV1::from_json(&unknown.to_string()).is_err());

    let mut mismatch: Value = serde_json::from_str(&encoded).expect("JSON");
    mismatch["document"]["revision"] = Value::from(1);
    assert!(RenderObservationWireV1::from_json(&mismatch.to_string()).is_err());

    let mut forged_digest: Value = serde_json::from_str(&encoded).expect("JSON");
    forged_digest["molecule_plans"][0]["plan"]["provenance"]["digest"][0] = Value::from(255);
    assert!(RenderObservationWireV1::from_json(&forged_digest.to_string()).is_err());

    let mut forged_root: Value = serde_json::from_str(&encoded).expect("JSON");
    forged_root["molecule_plans"][0]["molecule"]["source_id"] = Value::from("other");
    assert!(RenderObservationWireV1::from_json(&forged_root.to_string()).is_err());
}
