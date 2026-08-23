//! Typed payload and result facts for stateless presentation authoring.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One finite scene coordinate pair. The executor validates finite values
/// before a renderer gesture receives them.
#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationAuthorPointV1 {
    pub x: f64,
    pub y: f64,
}

/// One fenced, request-owned presentation authoring operation.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationAuthorRequestV1 {
    pub document: String,
    pub expected_revision: u64,
    pub expected_digest_hex: String,
    pub authoring: PresentationAuthoringRequestV1,
}

/// Closed presentation authoring vocabularies. Each variant owns only the
/// geometry and policy facts meaningful to its family.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PresentationAuthoringRequestV1 {
    Vector {
        vector_kind: ProtocolPresentationVectorKindV1,
        start: PresentationAuthorPointV1,
        end: PresentationAuthorPointV1,
        appearance_policy: ProtocolPresentationVectorAppearancePolicyV1,
    },
    CurvedTerminalArrow {
        terminal_kind: ProtocolCurvedTerminalArrowKindV1,
        start: PresentationAuthorPointV1,
        control: PresentationAuthorPointV1,
        end: PresentationAuthorPointV1,
    },
    CurvedEquilibriumArrow {
        start: PresentationAuthorPointV1,
        control: PresentationAuthorPointV1,
        end: PresentationAuthorPointV1,
    },
    Path {
        path_kind: ProtocolPresentationPathKindV1,
        points: Vec<PresentationAuthorPointV1>,
    },
    DirectBond {
        start: ProtocolDirectBondEndpointV1,
        end: ProtocolDirectBondEndpointV1,
        presentation: ProtocolDirectBondPresentationV1,
        new_atom_element: String,
        snap: ProtocolDirectBondSnapV1,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolPresentationVectorKindV1 {
    Line,
    Rectangle,
    Square,
    Oval,
    Circle,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolPresentationVectorAppearancePolicyV1 {
    EffectiveDrawingStandard,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolCurvedTerminalArrowKindV1 {
    Electron,
    Retro,
    Normal,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolPresentationPathKindV1 {
    Polyline,
    Polygon,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProtocolDirectBondEndpointV1 {
    ExistingAtom { atom_id: String },
    NewAtom { point: PresentationAuthorPointV1 },
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProtocolDirectBondPresentationV1 {
    Normal { order: ProtocolDirectBondOrderV1 },
    SolidWedge,
    HashedWedge,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolDirectBondOrderV1 {
    Single,
    Double,
    Triple,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolDirectBondSnapV1 {
    pub hex_grid: bool,
    pub angle_increment_degrees: Option<u16>,
    pub fixed_length_pt: Option<f64>,
}

/// The closed authoring variant which produced a successful operation.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationAuthoringKindV1 {
    Vector,
    CurvedTerminalArrow,
    CurvedEquilibriumArrow,
    Path,
    DirectBond,
}

/// Durable direct-bond result facts; identifiers come from the committed
/// document session rather than the client request.
#[derive(Clone, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationAuthorDirectBondOutcomeV1 {
    pub end_atom_identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub second_created_atom_identifier: Option<String>,
    pub created_new_atom: bool,
    pub created_new_molecule: bool,
}
