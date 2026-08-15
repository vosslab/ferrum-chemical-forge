//! API-owned composition of one document observation and verified render plans.

use std::collections::HashSet;

use ferrum_document::{
    DocumentSession, DocumentSessionError, MoleculeProjectionV1, PresentationRootProjectionV1,
    SessionDocumentObservationV1,
};
use ferrum_render::{
    MoleculeRenderPlan, RenderBatch, RenderError, RenderIssue, RenderProvenance, RenderRevision,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    DEPICTION_PROFILE_SCHEMA_V1, DepictionError, DepictionIssueV1, DepictionProfileV1,
    DepictionSuppressionV1, DocumentPlusRenderV1, DocumentTextRenderV1,
    render_document_projection_v1,
};

/// Closed schema identifier for the final API-owned render observation.
pub const RENDER_OBSERVATION_SCHEMA_V1: &str = "ferrum-render-observation-v1";

/// Document-root identity and order for one molecule render plan.
///
/// The fields are copied from the same immutable projection that produced the
/// plan. They let a frontend retain root-level ordering without interpreting
/// CDML or treating molecule-local atom and bond order as document-root order.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoleculeRenderRootV1 {
    id: Option<String>,
    projection_key: String,
    source_id: Option<String>,
    source_order: u32,
}

impl MoleculeRenderRootV1 {
    fn from_projection(value: &MoleculeProjectionV1) -> Self {
        Self {
            id: value.id().map(|id| id.as_str().to_owned()),
            projection_key: value.projection_key().as_str().to_owned(),
            source_id: value.source_id().map(str::to_owned),
            source_order: value.source_order(),
        }
    }

    fn new(
        id: Option<String>,
        projection_key: String,
        source_id: Option<String>,
        source_order: u32,
    ) -> Result<Self, String> {
        if !valid_projection_key(&projection_key) {
            return Err("invalid molecule render projection key".to_owned());
        }
        match (&id, &source_id) {
            (None, None) => {}
            (Some(id), Some(source_id))
                if !source_id.is_empty() && *id == molecule_object_id(source_id) => {}
            _ => return Err("molecule render identity does not match its source ID".to_owned()),
        }
        Ok(Self {
            id,
            projection_key,
            source_id,
            source_order,
        })
    }

    /// Return the durable document object key, when the molecule authored an ID.
    #[must_use]
    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    /// Return the non-operation key unique within this immutable projection.
    #[must_use]
    pub fn projection_key(&self) -> &str {
        &self.projection_key
    }

    /// Return the literal authored CDML ID, when present.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }

    /// Return the molecule's direct document-root child position.
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
}

impl<'de> Deserialize<'de> for MoleculeRenderRootV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            id: Option<String>,
            projection_key: String,
            source_id: Option<String>,
            source_order: u32,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(
            wire.id,
            wire.projection_key,
            wire.source_id,
            wire.source_order,
        )
        .map_err(serde::de::Error::custom)
    }
}

/// One document-root molecule and its existing complete render plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeRenderPlanV2 {
    molecule: MoleculeRenderRootV1,
    plan: MoleculeRenderPlan,
}

impl DocumentMoleculeRenderPlanV2 {
    pub(crate) fn from_projection(
        molecule: &MoleculeProjectionV1,
        plan: MoleculeRenderPlan,
    ) -> Self {
        Self {
            molecule: MoleculeRenderRootV1::from_projection(molecule),
            plan,
        }
    }

    /// Return the document-root molecule facts that own this plan.
    #[must_use]
    pub const fn molecule(&self) -> &MoleculeRenderRootV1 {
        &self.molecule
    }

    /// Return the complete molecule-local renderer plan.
    #[must_use]
    pub const fn plan(&self) -> &MoleculeRenderPlan {
        &self.plan
    }

    /// Return the exact source document revision.
    #[must_use]
    pub const fn revision(&self) -> RenderRevision {
        self.plan.revision()
    }

    /// Return the exact source document revision and digest.
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.plan.provenance()
    }

    /// Return immutable molecule-local target batches in source order.
    #[must_use]
    pub fn batches(&self) -> &[RenderBatch] {
        self.plan.batches()
    }

    /// Return molecule-local targets that were deliberately excluded.
    #[must_use]
    pub fn issues(&self) -> &[RenderIssue] {
        self.plan.issues()
    }
}

/// A revision-checked immutable document observation with its complete render result.
///
/// This type is constructed only by [`observe_render_v1`]. That one call obtains the
/// document snapshot and projection, invokes the closed verified-Telex depiction entry,
/// and lowers the projection. It therefore has no API for combining separately-read
/// snapshots, projections, resolutions, or plans.
#[derive(Debug)]
pub struct RenderObservationV1 {
    document: SessionDocumentObservationV1,
    profile: DepictionProfileV1,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    issues: Vec<DepictionIssueV1>,
    suppression: Option<DepictionSuppressionV1>,
}

impl RenderObservationV1 {
    fn from_document(
        document: SessionDocumentObservationV1,
        profile: DepictionProfileV1,
    ) -> Result<Self, RenderObservationError> {
        let resolution = render_document_projection_v1(document.projection(), &profile)?;
        let revision = document.snapshot().revision();
        let digest = document.snapshot().digest();
        let render_revision = RenderRevision::new(revision)
            .map_err(|_| RenderObservationError::ProvenanceMismatch)?;
        if document.projection().revision() != revision || document.projection().digest() != digest
        {
            return Err(RenderObservationError::ProvenanceMismatch);
        }
        if resolution.projection_revision() != revision || resolution.projection_digest() != digest
        {
            return Err(RenderObservationError::ProvenanceMismatch);
        }
        if resolution.plans().iter().any(|entry| {
            entry.plan().revision() != render_revision
                || entry.plan().provenance().digest() != *digest
        }) {
            return Err(RenderObservationError::ProvenanceMismatch);
        }
        validate_projection_plan_roots(document.projection().molecules(), resolution.plans())?;
        validate_projection_plus_roots(
            document.projection().presentation_stack().roots(),
            resolution.plus_renders(),
        )?;
        validate_projection_text_roots(
            document.projection().presentation_stack().roots(),
            resolution.text_renders(),
        )?;
        Ok(Self {
            document,
            profile,
            molecule_plans: resolution.plans().to_vec(),
            plus_renders: resolution.plus_renders().to_vec(),
            text_renders: resolution.text_renders().to_vec(),
            issues: resolution.issues().to_vec(),
            suppression: resolution.suppression(),
        })
    }

    /// Return the one authoritative document observation that produced every plan.
    #[must_use]
    pub fn document(&self) -> &SessionDocumentObservationV1 {
        &self.document
    }

    /// Return the closed profile used to resolve document presentation facts.
    #[must_use]
    pub const fn profile(&self) -> DepictionProfileV1 {
        self.profile
    }

    /// Return complete molecule plans in document root order.
    #[must_use]
    pub fn molecule_plans(&self) -> &[DocumentMoleculeRenderPlanV2] {
        &self.molecule_plans
    }

    /// Return verified-Telex plus layouts in document root order.
    #[must_use]
    pub fn plus_renders(&self) -> &[DocumentPlusRenderV1] {
        &self.plus_renders
    }

    /// Return verified-Telex direct-root Text layouts in document root order.
    #[must_use]
    pub fn text_renders(&self) -> &[DocumentTextRenderV1] {
        &self.text_renders
    }

    /// Return explicit profile exclusions without a fallback batch.
    #[must_use]
    pub fn issues(&self) -> &[DepictionIssueV1] {
        &self.issues
    }

    /// Return the typed whole-projection suppression, when presentation is invalid.
    #[must_use]
    pub const fn suppression(&self) -> Option<DepictionSuppressionV1> {
        self.suppression
    }

    /// Return the frozen, validated render-facing wire DTO.
    #[must_use]
    pub fn wire(&self) -> RenderObservationWireV1 {
        RenderObservationWireV1 {
            schema: RENDER_OBSERVATION_SCHEMA_V1.to_owned(),
            document: RenderDocumentProvenanceV1 {
                revision: self.document.snapshot().revision(),
                digest: *self.document.snapshot().digest(),
            },
            profile: self.profile.schema().to_owned(),
            molecule_plans: self.molecule_plans.clone(),
            plus_renders: self.plus_renders.clone(),
            text_renders: self.text_renders.clone(),
            issues: self.issues.clone(),
            suppression: self.suppression,
        }
    }
}

/// Lower the immutable post-operation observation with Ferrum's closed profile.
///
/// This is deliberately narrower than [`observe_render_v1`]: a committed
/// operation has already supplied the one authoritative observation, so this
/// projection must not re-observe a mutable [`DocumentSession`].
pub(crate) fn derive_render_observation_from_accepted_operation_v1(
    observation: &SessionDocumentObservationV1,
) -> Result<RenderObservationV1, RenderObservationError> {
    RenderObservationV1::from_document(observation.clone(), DepictionProfileV1::ferrum_default())
}

/// Obtain and lower exactly one revision-guarded session observation.
///
/// `expected_revision` is required, including for an initial session where the valid
/// expected value is zero. The closed depiction entry loads and verifies the only Telex
/// asset itself; this API accepts no font path, environment, or system-font selector.
pub fn observe_render_v1(
    session: &DocumentSession,
    expected_revision: u64,
) -> Result<RenderObservationV1, RenderObservationError> {
    let document = session.observe(expected_revision)?;
    RenderObservationV1::from_document(document, DepictionProfileV1::ferrum_default())
}

/// Failure while producing one final render observation.
#[derive(Debug, Error)]
pub enum RenderObservationError {
    /// The request did not name the current authoritative document revision.
    #[error(transparent)]
    Document(#[from] DocumentSessionError),
    /// Closed depiction resolution rejected lower-level rendering.
    #[error(transparent)]
    Depiction(#[from] DepictionError),
    /// Internal sources did not share the single required revision and digest.
    #[error("render observation provenance did not match its authoritative document")]
    ProvenanceMismatch,
    /// A renderer plan was not owned by the molecule root that produced it.
    #[error("render molecule roots did not match the authoritative document projection")]
    MoleculeRootMismatch,
    /// A plus layout was not owned by the presentation root that produced it.
    #[error("render plus roots did not match the authoritative document projection")]
    PlusRootMismatch,
    /// A Text layout was not owned by the presentation root that produced it.
    #[error("render Text roots did not match the authoritative document projection")]
    TextRootMismatch,
}

/// Immutable document identity carried by the render-facing wire DTO.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderDocumentProvenanceV1 {
    revision: u64,
    digest: [u8; 32],
}

impl RenderDocumentProvenanceV1 {
    /// Return the exact document revision, including valid initial revision zero.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Return the exact structural CDML digest.
    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// Strict, versioned wire representation of a final render observation.
///
/// The DTO contains render-facing immutable provenance rather than a deserializable
/// document authority. Decoding it can validate what a frontend received, but cannot
/// forge a `SessionDocumentObservationV1` or submit a session operation.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderObservationWireV1 {
    schema: String,
    document: RenderDocumentProvenanceV1,
    profile: String,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    issues: Vec<DepictionIssueV1>,
    suppression: Option<DepictionSuppressionV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedRenderObservationWireV1 {
    schema: String,
    document: RenderDocumentProvenanceV1,
    profile: String,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    issues: Vec<DepictionIssueV1>,
    suppression: Option<DepictionSuppressionV1>,
}

impl RenderObservationWireV1 {
    fn from_unchecked(wire: UncheckedRenderObservationWireV1) -> Result<Self, String> {
        let UncheckedRenderObservationWireV1 {
            schema,
            document,
            profile,
            molecule_plans,
            plus_renders,
            text_renders,
            issues,
            suppression,
        } = wire;
        if schema != RENDER_OBSERVATION_SCHEMA_V1 || profile != DEPICTION_PROFILE_SCHEMA_V1 {
            return Err("unknown render-observation schema or depiction profile".to_owned());
        }
        if molecule_plans.iter().any(|entry| {
            entry.plan().revision().get() != document.revision
                || entry.plan().provenance().digest() != document.digest
        }) {
            return Err("render-plan revision does not match document provenance".to_owned());
        }
        validate_wire_plan_roots(&molecule_plans)?;
        validate_wire_plus_roots(&plus_renders)?;
        validate_wire_text_roots(&text_renders)?;
        if suppression.is_some()
            && (!molecule_plans.is_empty() || !plus_renders.is_empty() || !text_renders.is_empty())
        {
            return Err("suppressed render observation cannot contain render payloads".to_owned());
        }
        Ok(Self {
            schema,
            document,
            profile,
            molecule_plans,
            plus_renders,
            text_renders,
            issues,
            suppression,
        })
    }

    /// Return exact document provenance shared by every plan.
    #[must_use]
    pub const fn document(&self) -> &RenderDocumentProvenanceV1 {
        &self.document
    }

    /// Return plans that are complete batches or exact target exclusions.
    #[must_use]
    pub fn molecule_plans(&self) -> &[DocumentMoleculeRenderPlanV2] {
        &self.molecule_plans
    }

    /// Return exact fixed-content presentation layouts.
    #[must_use]
    pub fn plus_renders(&self) -> &[DocumentPlusRenderV1] {
        &self.plus_renders
    }

    /// Return exact direct-root Text layouts.
    #[must_use]
    pub fn text_renders(&self) -> &[DocumentTextRenderV1] {
        &self.text_renders
    }

    /// Serialize canonical JSON for the closed wire grammar.
    pub fn to_canonical_json(&self) -> Result<String, RenderError> {
        serde_json::to_string(self).map_err(|error| RenderError::Serialization(error.to_string()))
    }

    /// Decode only the exact V1 grammar and validate all provenance links.
    pub fn from_json(input: &str) -> Result<Self, RenderError> {
        serde_json::from_str(input).map_err(|error| RenderError::InvalidJson(error.to_string()))
    }
}

impl<'de> Deserialize<'de> for RenderObservationWireV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::from_unchecked(UncheckedRenderObservationWireV1::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

fn validate_projection_plus_roots(
    roots: &[PresentationRootProjectionV1],
    renders: &[DocumentPlusRenderV1],
) -> Result<(), RenderObservationError> {
    let pluses = roots.iter().filter_map(|root| match root {
        PresentationRootProjectionV1::Plus { plus } => Some(plus),
        _ => None,
    });
    let mut pluses = pluses.peekable();
    for render in renders {
        while pluses
            .peek()
            .is_some_and(|plus| plus.target().source_order() < render.target().source_order())
        {
            pluses.next();
        }
        let Some(plus) = pluses.next() else {
            return Err(RenderObservationError::PlusRootMismatch);
        };
        if plus.target() != render.target() {
            return Err(RenderObservationError::PlusRootMismatch);
        }
    }
    Ok(())
}

fn validate_projection_text_roots(
    roots: &[PresentationRootProjectionV1],
    renders: &[DocumentTextRenderV1],
) -> Result<(), RenderObservationError> {
    let texts = roots.iter().filter_map(|root| match root {
        PresentationRootProjectionV1::Text { text } => Some(text),
        _ => None,
    });
    let mut texts = texts.peekable();
    for render in renders {
        while texts
            .peek()
            .is_some_and(|text| text.target().source_order() < render.target().source_order())
        {
            texts.next();
        }
        let Some(text) = texts.next() else {
            return Err(RenderObservationError::TextRootMismatch);
        };
        if text.target() != render.target() {
            return Err(RenderObservationError::TextRootMismatch);
        }
    }
    Ok(())
}

fn validate_projection_plan_roots(
    molecules: &[MoleculeProjectionV1],
    plans: &[DocumentMoleculeRenderPlanV2],
) -> Result<(), RenderObservationError> {
    let mut molecule_index = 0;
    for entry in plans {
        while molecule_index < molecules.len()
            && molecules[molecule_index].source_order() < entry.molecule().source_order()
        {
            molecule_index += 1;
        }
        let Some(molecule) = molecules.get(molecule_index) else {
            return Err(RenderObservationError::MoleculeRootMismatch);
        };
        if MoleculeRenderRootV1::from_projection(molecule) != *entry.molecule() {
            return Err(RenderObservationError::MoleculeRootMismatch);
        }
        molecule_index += 1;
    }
    Ok(())
}

fn validate_wire_plan_roots(plans: &[DocumentMoleculeRenderPlanV2]) -> Result<(), String> {
    let mut previous_order = None;
    let mut projection_keys = HashSet::new();
    let mut durable_ids = HashSet::new();
    for entry in plans {
        let root = entry.molecule();
        if previous_order.is_some_and(|order| root.source_order() <= order) {
            return Err("molecule render plans are not in document root order".to_owned());
        }
        if !projection_keys.insert(root.projection_key()) {
            return Err("molecule render plans contain a duplicate projection key".to_owned());
        }
        if root.id().is_some_and(|id| !durable_ids.insert(id)) {
            return Err("molecule render plans contain a duplicate durable ID".to_owned());
        }
        previous_order = Some(root.source_order());
    }
    Ok(())
}

fn validate_wire_plus_roots(renders: &[DocumentPlusRenderV1]) -> Result<(), String> {
    let mut previous_order = None;
    let mut projection_keys = HashSet::new();
    let mut durable_ids = HashSet::new();
    for render in renders {
        let target = render.target();
        if previous_order.is_some_and(|order| target.source_order() <= order) {
            return Err("plus renders are not in document root order".to_owned());
        }
        if !projection_keys.insert(target.projection_key().as_str()) {
            return Err("plus renders contain a duplicate projection key".to_owned());
        }
        if target
            .id()
            .is_some_and(|id| !durable_ids.insert(id.as_str()))
        {
            return Err("plus renders contain a duplicate durable ID".to_owned());
        }
        previous_order = Some(target.source_order());
    }
    Ok(())
}

fn validate_wire_text_roots(renders: &[DocumentTextRenderV1]) -> Result<(), String> {
    let mut previous_order = None;
    let mut projection_keys = HashSet::new();
    let mut durable_ids = HashSet::new();
    for render in renders {
        let target = render.target();
        if previous_order.is_some_and(|order| target.source_order() <= order) {
            return Err("Text renders are not in document root order".to_owned());
        }
        if !projection_keys.insert(target.projection_key().as_str()) {
            return Err("Text renders contain a duplicate projection key".to_owned());
        }
        if target
            .id()
            .is_some_and(|id| !durable_ids.insert(id.as_str()))
        {
            return Err("Text renders contain a duplicate durable ID".to_owned());
        }
        previous_order = Some(target.source_order());
    }
    Ok(())
}

fn valid_projection_key(value: &str) -> bool {
    let Some(path) = value.strip_prefix("ferrum-projection-local-v1/") else {
        return false;
    };
    !path.is_empty()
        && path
            .split('.')
            .all(|component| component.parse::<u32>().is_ok())
}

fn molecule_object_id(source_id: &str) -> String {
    format!(
        "ferrum-document-object-v1/{}/source/{}",
        hex(b"cdml/molecule"),
        hex(source_id.as_bytes())
    )
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
