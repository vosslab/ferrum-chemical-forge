use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document::DocumentSession;
use ferrum_render::{RenderDisplayLayerV1, RenderOp, VectorStrokeLineCapV1};
use serde_json::Value;

use crate::depiction_profile_v1::DepictionResolutionV1;
use crate::{
    DEPICTION_PROFILE_SCHEMA_V1, DepictionIssueCodeV1, DepictionProfileV1,
    render_document_projection_v1,
};

#[test]
fn resolution_wire_rejects_unknown_fields_and_profile_aliases() {
    let resolution = DepictionResolutionV1::new(0, [0; 32], vec![], vec![]);
    let mut wire = serde_json::to_value(resolution).expect("wire");
    wire["unexpected"] = Value::Bool(true);
    assert!(serde_json::from_value::<DepictionResolutionV1>(wire).is_err());
    let mut wire =
        serde_json::to_value(DepictionResolutionV1::new(0, [0; 32], vec![], vec![])).expect("wire");
    wire["profile"] = Value::String("system-font-fallback".to_owned());
    assert!(serde_json::from_value::<DepictionResolutionV1>(wire).is_err());
}

#[test]
fn profile_emits_explicit_ferrum_defaults_from_initial_session_provenance() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let projection = observation.projection();
    let resolution =
        render_document_projection_v1(projection, &DepictionProfileV1::ferrum_default())
            .expect("resolution");

    assert_eq!(
        serde_json::to_value(&resolution).expect("wire")["profile"],
        DEPICTION_PROFILE_SCHEMA_V1
    );
    assert_eq!(resolution.projection_revision(), 0);
    assert!(resolution.issues().is_empty());
    let wire = serde_json::to_value(&resolution).expect("wire");
    let first_text = &wire["plans"][0]["plan"]["batches"][0]["operations"][0]["operation"];
    assert_eq!(first_text["face"], "ferrum-telex-regular-v1");
    assert_eq!(first_text["size"], 12.0);
    assert_eq!(first_text["paint"], "000000");
    assert_eq!(first_text["z"], 30);
    let bond = wire["plans"][0]["plan"]["batches"]
        .as_array()
        .expect("batches")
        .iter()
        .find(|batch| batch["operations"][0]["kind"] == "line")
        .expect("bond batch");
    assert_eq!(bond["operations"][0]["operation"]["width"], 1.0);
    assert_eq!(bond["operations"][0]["operation"]["paint"], "000000");
    assert_eq!(bond["operations"][0]["operation"]["z"], 10);
}

#[test]
fn declared_haworth_front_forms_lower_through_the_ordinary_v2_observation() {
    let session = DocumentSession::load(
		"<cdml><standard bond_width=\"6px\"/><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom><atom id=\"c\" name=\"C\"><point x=\"40\" y=\"0\"/></atom><atom id=\"d\" name=\"C\"><point x=\"60\" y=\"0\"/></atom><bond id=\"w\" start=\"a\" end=\"b\" type=\"w1\" haworth_position=\"front\"/><bond id=\"q\" start=\"b\" end=\"c\" type=\"q1\" haworth_position=\"front\"/><bond id=\"n\" start=\"c\" end=\"d\" type=\"n1\" haworth_position=\"back\"/></molecule></cdml>",
	)
	.expect("document");
    let resolution = render_document_projection_v1(
        session.observe(0).expect("observation").projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    let batches = resolution.plans()[0].batches();
    let q = batches
        .iter()
        .find(|batch| batch.display_layer() == RenderDisplayLayerV1::HaworthFrontStroke)
        .expect("q1/front batch");
    let w = batches
        .iter()
        .find(|batch| batch.display_layer() == RenderDisplayLayerV1::HaworthFrontWedge)
        .expect("w1/front batch");

    assert!(matches!(q.operations(), [RenderOp::Path(path)]
		if matches!(path.stroke(), Some(stroke)
			if stroke.line_cap() == VectorStrokeLineCapV1::Round && stroke.width().get() > 1.0)));
    assert!(
        matches!(w.operations(), [RenderOp::Path(path)]
		if path.fill().is_some() && path.commands().iter().any(|command| matches!(command, ferrum_render::ScenePathCommandV2::CubicTo { .. })))
            && batches
                .iter()
                .any(|batch| matches!(batch.operations(), [RenderOp::Line(_)]))
            && resolution.issues().is_empty()
    );
}

#[test]
fn malformed_haworth_front_fact_isolated_to_its_bond_target() {
    let session = DocumentSession::load(
		"<cdml><molecule id=\"m\"><atom id=\"a\" name=\"N\" background-color=\"#ffffff\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom><atom id=\"c\" name=\"O\"><point x=\"40\" y=\"0\"/></atom><bond id=\"bad\" start=\"a\" end=\"b\" type=\"q1\"/><bond id=\"normal\" start=\"b\" end=\"c\" type=\"n1\"/></molecule></cdml>",
	)
	.expect("document");
    let resolution = render_document_projection_v1(
        session.observe(0).expect("observation").projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    let mut operations = resolution.plans()[0]
        .batches()
        .iter()
        .flat_map(|batch| batch.operations());

    assert!(
        resolution.plans()[0]
            .plan()
            .issues()
            .iter()
            .any(|issue| matches!(
                issue.kind(),
                ferrum_render::RenderIssueKind::UnsupportedFeature { .. }
            ))
    );
    assert!(operations.any(|operation| matches!(
        operation,
        RenderOp::Mask(_) | RenderOp::Text(_) | RenderOp::Line(_)
    )));
}

#[test]
fn authored_visible_atom_number_lowers_to_an_explicit_annotation() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\" number=\"27\" show_number=\"yes\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let resolution = render_document_projection_v1(
        observation.projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");

    assert!(resolution.issues().is_empty());
    let wire = serde_json::to_value(&resolution).expect("wire");
    let operations = wire["plans"][0]["plan"]["batches"][0]["operations"]
        .as_array()
        .expect("atom operations");
    assert_eq!(operations.len(), 2);
    let number = &operations[1]["operation"];
    assert_eq!(number["origin"]["x"], 8.0);
    assert_eq!(number["origin"]["y"], -12.0);
    assert_eq!(number["size"], 9.0);
    assert_eq!(number["paint"], "0000c8");
    assert_eq!(number["z"], 40);
    assert_eq!(number["runs"][0]["text"], "27");
}

#[test]
fn hidden_or_invalid_atom_numbers_never_receive_a_frontend_fallback() {
    let hidden = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\" number=\"27\" show_number=\"no\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let hidden = render_document_projection_v1(
        hidden.observe(0).expect("observation").projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    let hidden_wire = serde_json::to_value(&hidden).expect("wire");
    assert_eq!(
        hidden_wire["plans"][0]["plan"]["batches"][0]["operations"]
            .as_array()
            .expect("atom operations")
            .len(),
        1
    );

    let invalid = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\" number=\"not-a-number\" show_number=\"yes\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let invalid = render_document_projection_v1(
        invalid.observe(0).expect("observation").projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    assert!(invalid.plans().is_empty());
    assert_eq!(
        invalid.suppression(),
        Some(crate::DepictionSuppressionV1::InvalidPresentationFacts)
    );
}

#[test]
fn authored_font_family_is_an_issue_without_a_toolkit_substitution() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"N\"><point x=\"0\" y=\"0\"/><font family=\"Arial\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let projection = observation.projection();
    let resolution =
        render_document_projection_v1(projection, &DepictionProfileV1::ferrum_default())
            .expect("resolution");

    assert!(resolution.plans()[0].batches().is_empty());
    assert!(
        resolution
            .issues()
            .iter()
            .any(|issue| { issue.code() == DepictionIssueCodeV1::UnsupportedAuthoredFontFamily })
    );
}

#[test]
fn malformed_presentation_fact_is_closed_before_default_resolution() {
    let session = DocumentSession::load(
        "<cdml><standard line_color=\"blue\"/><molecule id=\"m\"><atom id=\"a\" name=\"N\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let projection = observation.projection();
    let resolution =
        render_document_projection_v1(projection, &DepictionProfileV1::ferrum_default())
            .expect("resolution");

    assert!(resolution.plans().is_empty());
    assert_eq!(
        resolution.suppression(),
        Some(crate::DepictionSuppressionV1::InvalidPresentationFacts)
    );
    assert!(
        resolution
            .issues()
            .iter()
            .any(|issue| issue.code() == DepictionIssueCodeV1::InvalidPresentationFact)
    );
    let Value::String(schema) = serde_json::to_value(&resolution).expect("wire")["schema"].clone()
    else {
        panic!("closed schema string");
    };
    assert_eq!(schema, "ferrum-depiction-resolution-v1");
}

#[test]
fn local_standard_and_profile_facts_resolve_without_renderer_defaults() {
    let session = DocumentSession::load(
        "<cdml><standard line_width=\"2px\" font_size=\"13\" line_color=\"#123\" area_color=\"#abc\"><bond width=\"6px\"/><atom show_hydrogens=\"yes\"/></standard><molecule id=\"m\"><atom id=\"a\" name=\"N\" explicit_hydrogens=\"2\" hydrogens=\"off\" background-color=\"#def\"><point x=\"0\" y=\"0\"/><font size=\"14\" color=\"#456\"/></atom><atom id=\"b\" name=\"O\" explicit_hydrogens=\"2\"><point x=\"20\" y=\"0\"/></atom><atom id=\"c\" name=\"C\" background-color=\"\"><point x=\"40\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n2\" line_width=\"3px\" bond_width=\"8px\" color=\"#789\"/></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let resolution = render_document_projection_v1(
        observation.projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    let batches = serde_json::to_value(&resolution).expect("wire")["plans"][0]["plan"]["batches"]
        .as_array()
        .expect("batches")
        .clone();
    let atom_a = &batches[0]["operations"];
    assert_eq!(atom_a[0]["kind"], "mask");
    assert_eq!(atom_a[0]["operation"]["paint"], "ddeeff");
    assert_eq!(atom_a[0]["operation"]["z"], 20);
    assert_eq!(atom_a[1]["operation"]["size"], 14.0);
    assert_eq!(atom_a[1]["operation"]["paint"], "445566");
    assert!(
        atom_a[1]["operation"]["runs"]
            .as_array()
            .expect("runs")
            .iter()
            .all(|run| run["text"] != "H")
    );
    let atom_b = &batches[1]["operations"];
    assert_eq!(atom_b[0]["operation"]["paint"], "aabbcc");
    assert_eq!(atom_b[1]["operation"]["size"], 13.0);
    assert_eq!(atom_b[1]["operation"]["paint"], "112233");
    assert!(
        atom_b[1]["operation"]["runs"]
            .as_array()
            .expect("runs")
            .iter()
            .any(|run| run["text"] == "H")
    );
    let atom_c = &batches[2]["operations"];
    assert_eq!(atom_c.as_array().expect("operations").len(), 1);
    assert_eq!(atom_c[0]["kind"], "text");
    let bond_operations = batches[3]["operations"].as_array().expect("bond lines");
    let first_bond = &bond_operations[0]["operation"];
    let second_bond = &bond_operations[1]["operation"];
    assert_eq!(first_bond["width"], 3.0);
    assert_eq!(first_bond["paint"], "778899");
    assert_eq!(
        second_bond["start"]["y"].as_f64().expect("second y")
            - first_bond["start"]["y"].as_f64().expect("first y"),
        8.0
    );
}

#[test]
fn normal_and_directed_cdml_bonds_lower_to_supported_render_facts() {
    for source_type in ["n2", "n3"] {
        let source = format!(
            "<cdml><standard><bond width=\"10\"/></standard><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"{source_type}\"/></molecule></cdml>",
        );
        let session = DocumentSession::load(&source).expect("session");
        let observation = session.observe(0).expect("observation");
        let resolution = render_document_projection_v1(
            observation.projection(),
            &DepictionProfileV1::ferrum_default(),
        )
        .expect("resolution");
        let plan = &resolution.plans()[0];
        assert!(plan.issues().is_empty(), "{source_type}");
        let bond = plan
            .batches()
            .iter()
            .find(|batch| batch.target().record_id().kind() == RecordKind::Bond)
            .expect("normal bond has one complete render batch");
        assert!(
            bond.operations()
                .iter()
                .any(|operation| matches!(operation, ferrum_render::RenderOp::Line(_))),
            "{source_type}",
        );
    }

    let session = DocumentSession::load(
        "<cdml><standard><bond width=\"10\" wedge_width=\"12\"/></standard><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"solid\" start=\"a\" end=\"b\" type=\"w1\"/><bond id=\"hashed\" start=\"b\" end=\"a\" type=\"h1\"/></molecule></cdml>",
    ).expect("session");
    let resolution = render_document_projection_v1(
        session.observe(0).expect("observation").projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    assert!(resolution.issues().is_empty());
    let batches = &resolution.plans()[0].plan().batches();
    let solid = batches
        .iter()
        .find(|batch| {
            batch
                .operations()
                .iter()
                .any(|operation| matches!(operation, ferrum_render::RenderOp::Path(_)))
        })
        .expect("solid wedge batch");
    let hashed = batches
        .iter()
        .find(|batch| {
            batch.target().record_id().kind() == RecordKind::Bond
                && batch
                    .operations()
                    .iter()
                    .all(|operation| matches!(operation, ferrum_render::RenderOp::Line(_)))
        })
        .expect("hashed wedge batch");
    assert!(solid.operations().iter().any(|operation| matches!(operation, ferrum_render::RenderOp::Path(path) if path.fill().is_some())));
    assert!(hashed.operations().iter().all(|operation| matches!(operation, ferrum_render::RenderOp::Line(line) if line.start().x().is_finite() && line.end().x().is_finite())));
}

#[test]
fn negative_signed_bond_width_is_a_durable_plan_issue_without_coercion() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n2\" bond_width=\"-4\"/></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let projection = observation.projection();
    let bond = &projection.molecules()[0].bonds()[0];
    assert_eq!(bond.bond_width().expect("signed source fact").value(), -4.0);

    let resolution =
        render_document_projection_v1(projection, &DepictionProfileV1::ferrum_default())
            .expect("resolution");
    let plan = &resolution.plans()[0];
    assert!(
        plan.batches()
            .iter()
            .all(|batch| batch.target().record_id().kind() != RecordKind::Bond)
    );
    assert_eq!(plan.issues().len(), 1);
    let issue = &plan.issues()[0];
    assert_eq!(
        issue.target().record_id(),
        &RecordId::from_source(RecordKind::Bond, &Identifier::new("ab").expect("source ID"))
    );
    assert_eq!(issue.target().source_order(), bond.source_order());
    assert!(matches!(
        issue.kind(),
        ferrum_render::RenderIssueKind::UnsupportedFeature { feature }
            if feature.contains("unsupported signed bond lane placement")
                && feature.contains("bond_width=-4")
    ));
    assert_eq!(plan.revision().get(), projection.revision());
    assert_eq!(plan.provenance().digest(), *projection.digest());
    assert!(resolution.issues().is_empty());
}

#[test]
fn positive_signed_bond_width_remains_a_complete_bond_batch() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n2\" bond_width=\"4\"/></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let projection = observation.projection();
    let resolution =
        render_document_projection_v1(projection, &DepictionProfileV1::ferrum_default())
            .expect("resolution");
    let plan = &resolution.plans()[0];
    assert!(plan.issues().is_empty());
    assert!(
        plan.batches()
            .iter()
            .any(|batch| batch.target().record_id().kind() == RecordKind::Bond)
    );
}

#[test]
fn every_non_single_or_unrecognized_cdml_bond_preserves_its_exact_source_detail() {
    let session = DocumentSession::load(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"w2\" start=\"a\" end=\"b\" type=\"w2\"/><bond id=\"n5\" start=\"a\" end=\"b\" type=\"n5\"/><bond id=\"a1\" start=\"a\" end=\"b\" type=\"a1\"/><bond id=\"z1\" start=\"a\" end=\"b\" type=\"z1\"/><bond id=\"d2\" start=\"a\" end=\"b\" type=\"d2\"/><bond id=\"q1\" start=\"a\" end=\"b\" type=\"q1\"/></molecule></cdml>",
    )
    .expect("session");
    let observation = session.observe(0).expect("observation");
    let resolution = render_document_projection_v1(
        observation.projection(),
        &DepictionProfileV1::ferrum_default(),
    )
    .expect("resolution");
    let plan = &resolution.plans()[0];
    assert!(
        plan.batches()
            .iter()
            .all(|batch| batch.target().source_order() < 2)
    );
    let expected = ["w2", "n5", "a1", "z1", "d2", "q1"];
    assert_eq!(plan.issues().len(), expected.len());
    for (issue, source_type) in plan.issues().iter().zip(expected) {
        assert!(issue.target().source_order() >= 2);
        assert!(matches!(
            issue.kind(),
            ferrum_render::RenderIssueKind::UnsupportedFeature { feature }
                if feature.contains(&format!("Some(\"{source_type}\")"))
        ));
    }
}
