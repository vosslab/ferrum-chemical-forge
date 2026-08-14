use serde_json::Value;

use crate::render_observation_cli::{
    RenderObservationCliError, render_observation, render_observation_json,
};

const RENDERABLE_CDML: &str = "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

#[test]
fn canonical_wire_comes_from_the_newly_loaded_initial_session_observation() {
    let wire = render_observation(RENDERABLE_CDML).expect("render observation");
    let json = wire.to_canonical_json().expect("canonical JSON");
    let value: Value = serde_json::from_str(&json).expect("JSON object");

    assert_eq!(value["schema"], "ferrum-render-observation-v1");
    assert_eq!(value["document"]["revision"], 0);
    assert_eq!(value["molecule_plans"].as_array().map(Vec::len), Some(1));
    assert!(value["suppression"].is_null());
}

#[test]
fn canonical_json_is_one_object_without_a_stream_delimiter() {
    let json = render_observation_json(RENDERABLE_CDML).expect("canonical JSON");

    assert_eq!(json.matches('\n').count(), 0);
    assert_eq!(
        serde_json::from_str::<Value>(&json).expect("JSON object")["schema"],
        "ferrum-render-observation-v1"
    );
}

#[test]
fn invalid_presentation_suppression_is_a_cli_error() {
    let error = render_observation(
        "<cdml><standard line_color=\"blue\"/><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    )
    .expect_err("suppression must not produce a partial CLI report");

    assert!(matches!(error, RenderObservationCliError::Suppressed));
}

#[test]
fn unprojectable_cdml_is_an_observation_error() {
    let error = render_observation(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"/></molecule></cdml>",
    )
    .expect_err("missing atom point prevents an observation");

    assert!(matches!(error, RenderObservationCliError::Observation(_)));
}
