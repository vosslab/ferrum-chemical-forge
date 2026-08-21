//! Immutable renderer transfer records.
//!
//! These values are deliberately detached from CDML parsing, Xot nodes, document
//! sessions, history, and UI state. A document implementation converts one
//! already-fenced observation into this closed, serializable value.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const RENDER_DOCUMENT_MODEL_SCHEMA_V1: &str = "ferrum-render-document-model-v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderIdentityModelV1 {
    durable_object_id: Option<String>,
    projection_key: String,
    source_id: Option<String>,
    source_order: u32,
}

impl RenderIdentityModelV1 {
    #[must_use]
    pub fn new(
        durable_object_id: Option<String>,
        projection_key: String,
        source_id: Option<String>,
        source_order: u32,
    ) -> Self {
        Self {
            durable_object_id,
            projection_key,
            source_id,
            source_order,
        }
    }

    #[must_use]
    pub fn durable_object_id(&self) -> Option<&str> {
        self.durable_object_id.as_deref()
    }
    #[must_use]
    pub fn projection_key(&self) -> &str {
        &self.projection_key
    }
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderPoint3ModelV1 {
    x: f64,
    y: f64,
    z: f64,
}

impl RenderPoint3ModelV1 {
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, RenderDocumentModelErrorV1> {
        if [x, y, z].into_iter().any(|value| !value.is_finite()) {
            return Err(RenderDocumentModelErrorV1::NonFinitePoint);
        }
        Ok(Self { x, y, z })
    }
    #[must_use]
    pub const fn x(self) -> f64 {
        self.x
    }
    #[must_use]
    pub const fn y(self) -> f64 {
        self.y
    }
    #[must_use]
    pub const fn z(self) -> f64 {
        self.z
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderAtomModelV1 {
    identity: RenderIdentityModelV1,
    element: Option<String>,
    position: RenderPoint3ModelV1,
    facts: Value,
}

impl RenderAtomModelV1 {
    #[must_use]
    pub fn new(
        identity: RenderIdentityModelV1,
        element: Option<String>,
        position: RenderPoint3ModelV1,
        facts: Value,
    ) -> Self {
        Self {
            identity,
            element,
            position,
            facts,
        }
    }
    #[must_use]
    pub const fn identity(&self) -> &RenderIdentityModelV1 {
        &self.identity
    }
    #[must_use]
    pub fn element(&self) -> Option<&str> {
        self.element.as_deref()
    }
    #[must_use]
    pub const fn position(&self) -> RenderPoint3ModelV1 {
        self.position
    }
    #[must_use]
    pub const fn facts(&self) -> &Value {
        &self.facts
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderBondEndpointKindModelV1 {
    Atom,
    Group,
    MoleculeText,
    Query,
    Unknown,
    Missing,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderBondEndpointModelV1 {
    source_id: Option<String>,
    durable_object_id: Option<String>,
    kind: RenderBondEndpointKindModelV1,
}

impl RenderBondEndpointModelV1 {
    #[must_use]
    pub fn new(
        source_id: Option<String>,
        durable_object_id: Option<String>,
        kind: RenderBondEndpointKindModelV1,
    ) -> Self {
        Self {
            source_id,
            durable_object_id,
            kind,
        }
    }
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    #[must_use]
    pub fn durable_object_id(&self) -> Option<&str> {
        self.durable_object_id.as_deref()
    }
    #[must_use]
    pub const fn kind(&self) -> RenderBondEndpointKindModelV1 {
        self.kind
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderBondModelV1 {
    identity: RenderIdentityModelV1,
    start: RenderBondEndpointModelV1,
    end: RenderBondEndpointModelV1,
    facts: Value,
}

impl RenderBondModelV1 {
    #[must_use]
    pub fn new(
        identity: RenderIdentityModelV1,
        start: RenderBondEndpointModelV1,
        end: RenderBondEndpointModelV1,
        facts: Value,
    ) -> Self {
        Self {
            identity,
            start,
            end,
            facts,
        }
    }
    #[must_use]
    pub const fn identity(&self) -> &RenderIdentityModelV1 {
        &self.identity
    }
    #[must_use]
    pub const fn start(&self) -> &RenderBondEndpointModelV1 {
        &self.start
    }
    #[must_use]
    pub const fn end(&self) -> &RenderBondEndpointModelV1 {
        &self.end
    }
    #[must_use]
    pub const fn facts(&self) -> &Value {
        &self.facts
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderMoleculeModelV1 {
    identity: RenderIdentityModelV1,
    name: Option<String>,
    atoms: Vec<RenderAtomModelV1>,
    bonds: Vec<RenderBondModelV1>,
    facts: Value,
}

impl RenderMoleculeModelV1 {
    #[must_use]
    pub fn new(
        identity: RenderIdentityModelV1,
        name: Option<String>,
        atoms: Vec<RenderAtomModelV1>,
        bonds: Vec<RenderBondModelV1>,
        facts: Value,
    ) -> Self {
        Self {
            identity,
            name,
            atoms,
            bonds,
            facts,
        }
    }
    #[must_use]
    pub const fn identity(&self) -> &RenderIdentityModelV1 {
        &self.identity
    }
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    #[must_use]
    pub fn atoms(&self) -> &[RenderAtomModelV1] {
        &self.atoms
    }
    #[must_use]
    pub fn bonds(&self) -> &[RenderBondModelV1] {
        &self.bonds
    }
    #[must_use]
    pub const fn facts(&self) -> &Value {
        &self.facts
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderPresentationKindModelV1 {
    Arrow,
    Plus,
    Text,
    Polyline,
    Wavy,
    RoundBracket,
    Rectangle,
    Square,
    Oval,
    Circle,
    Polygon,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderPresentationRootModelV1 {
    identity: RenderIdentityModelV1,
    kind: RenderPresentationKindModelV1,
    facts: Value,
}

impl RenderPresentationRootModelV1 {
    #[must_use]
    pub fn new(
        identity: RenderIdentityModelV1,
        kind: RenderPresentationKindModelV1,
        facts: Value,
    ) -> Self {
        Self {
            identity,
            kind,
            facts,
        }
    }
    #[must_use]
    pub const fn identity(&self) -> &RenderIdentityModelV1 {
        &self.identity
    }
    #[must_use]
    pub const fn kind(&self) -> RenderPresentationKindModelV1 {
        self.kind
    }
    #[must_use]
    pub const fn facts(&self) -> &Value {
        &self.facts
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderDiagnosticModelV1 {
    category: String,
    path: String,
    detail: String,
}

impl RenderDiagnosticModelV1 {
    #[must_use]
    pub fn new(category: String, path: String, detail: String) -> Self {
        Self {
            category,
            path,
            detail,
        }
    }
    #[must_use]
    pub fn category(&self) -> &str {
        &self.category
    }
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderTelexFactsModelV1 {
    family: String,
    profile: String,
}

impl RenderTelexFactsModelV1 {
    #[must_use]
    pub fn ferrum_v1() -> Self {
        Self {
            family: "Telex".to_owned(),
            profile: "ferrum-v1".to_owned(),
        }
    }
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }
    #[must_use]
    pub fn profile(&self) -> &str {
        &self.profile
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderDocumentModelV1 {
    schema: String,
    revision: u64,
    digest: [u8; 32],
    paper: Value,
    drawing_standard: Option<Value>,
    telex: RenderTelexFactsModelV1,
    molecules: Vec<RenderMoleculeModelV1>,
    presentation_roots: Vec<RenderPresentationRootModelV1>,
    diagnostics: Vec<RenderDiagnosticModelV1>,
}

impl RenderDocumentModelV1 {
    #[must_use]
    pub fn new(
        revision: u64,
        digest: [u8; 32],
        paper: Value,
        drawing_standard: Option<Value>,
        molecules: Vec<RenderMoleculeModelV1>,
        presentation_roots: Vec<RenderPresentationRootModelV1>,
        diagnostics: Vec<RenderDiagnosticModelV1>,
    ) -> Self {
        Self {
            schema: RENDER_DOCUMENT_MODEL_SCHEMA_V1.to_owned(),
            revision,
            digest,
            paper,
            drawing_standard,
            telex: RenderTelexFactsModelV1::ferrum_v1(),
            molecules,
            presentation_roots,
            diagnostics,
        }
    }
    #[must_use]
    pub fn schema(&self) -> &str {
        &self.schema
    }
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    #[must_use]
    pub const fn paper(&self) -> &Value {
        &self.paper
    }
    #[must_use]
    pub const fn drawing_standard(&self) -> Option<&Value> {
        self.drawing_standard.as_ref()
    }
    #[must_use]
    pub const fn telex(&self) -> &RenderTelexFactsModelV1 {
        &self.telex
    }
    #[must_use]
    pub fn molecules(&self) -> &[RenderMoleculeModelV1] {
        &self.molecules
    }
    #[must_use]
    pub fn presentation_roots(&self) -> &[RenderPresentationRootModelV1] {
        &self.presentation_roots
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[RenderDiagnosticModelV1] {
        &self.diagnostics
    }
}

#[derive(Clone, Debug, thiserror::Error, Eq, PartialEq)]
#[non_exhaustive]
pub enum RenderDocumentModelErrorV1 {
    #[error("render transfer point is not finite")]
    NonFinitePoint,
}
