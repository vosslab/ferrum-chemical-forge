//! API-owned composition of one document observation and verified render plans.

use std::collections::HashSet;

use crate::{
    CompactGroupRenderPrimitiveV1, MoleculeRenderPlanV4, RenderBatchV4, RenderError, RenderIssue,
    RenderProvenance, RenderRevision,
};
use ferrum_document_projection::{
    DocumentObjectIdV1, DocumentProjectionV1, MoleculeProjectionV1, PresentationRootProjectionV1,
    PresentationStackEntryV1,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    DEPICTION_PROFILE_SCHEMA_V1, DepictionError, DepictionProfileV1, DepictionSuppressionV1,
    DocumentPlusRenderV1, DocumentTextRenderV1, MoleculeMemberDepictionIssueV1,
    render_document_projection_v1,
};

/// Closed schema identifier for the final API-owned render observation.
pub const RESOLVED_DOCUMENT_RENDER_SCHEMA_V2: &str = "ferrum-resolved-document-render-v2";

/// The durable document-root identity for one molecule render plan.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoleculeRenderRootV1 {
    document_object_id: DocumentObjectIdV1,
}

impl MoleculeRenderRootV1 {
    pub(crate) const fn new(document_object_id: DocumentObjectIdV1) -> Self {
        Self { document_object_id }
    }

    /// Return the required durable document object identity.
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }
}

/// One document-root molecule and its existing complete render plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentMoleculeRenderPlanV4 {
    molecule: MoleculeRenderRootV1,
    plan: MoleculeRenderPlanV4,
    member_ids: Vec<DocumentObjectIdV1>,
    member_issues: Vec<MoleculeMemberDepictionIssueV1>,
    #[serde(skip, default)]
    compact_group_primitives: Vec<CompactGroupRenderPrimitiveV1>,
}

impl DocumentMoleculeRenderPlanV4 {
    pub(crate) fn from_document_object_id(
        document_object_id: DocumentObjectIdV1,
        plan: MoleculeRenderPlanV4,
        compact_group_primitives: Vec<CompactGroupRenderPrimitiveV1>,
        member_ids: Vec<DocumentObjectIdV1>,
        member_issues: Vec<MoleculeMemberDepictionIssueV1>,
    ) -> Result<Self, RenderError> {
        let entry = Self {
            molecule: MoleculeRenderRootV1::new(document_object_id),
            plan,
            member_ids,
            member_issues,
            compact_group_primitives,
        };
        entry.validate_member_issues()?;
        Ok(entry)
    }

    /// Return the document-root molecule facts that own this plan.
    #[must_use]
    pub const fn molecule(&self) -> &MoleculeRenderRootV1 {
        &self.molecule
    }

    /// Return the complete molecule-local renderer plan.
    #[must_use]
    pub const fn plan(&self) -> &MoleculeRenderPlanV4 {
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
    pub fn batches(&self) -> &[RenderBatchV4] {
        self.plan.batches()
    }

    /// Return first-class compact-group render primitives in molecule source order.
    #[must_use]
    pub fn compact_group_primitives(&self) -> &[CompactGroupRenderPrimitiveV1] {
        &self.compact_group_primitives
    }

    /// Return molecule-local targets that were deliberately excluded.
    #[must_use]
    pub fn issues(&self) -> &[RenderIssue] {
        self.plan.issues()
    }

    /// Return durable projected atom, bond, and compact-group member identities.
    #[must_use]
    pub fn member_ids(&self) -> &[DocumentObjectIdV1] {
        &self.member_ids
    }

    /// Return diagnostics retained by this molecule's durable owner.
    #[must_use]
    pub fn member_issues(&self) -> &[MoleculeMemberDepictionIssueV1] {
        &self.member_issues
    }

    fn validate_member_issues(&self) -> Result<(), RenderError> {
        let members = self.member_ids.iter().collect::<HashSet<_>>();
        if members.len() != self.member_ids.len()
            || self
                .member_issues
                .iter()
                .any(|issue| !members.contains(issue.target()))
        {
            return Err(RenderError::InvalidRequest(
                "molecule depiction issue target is not a projected molecule member".to_owned(),
            ));
        }
        Ok(())
    }
}

/// A revision-checked immutable document observation with its complete render result.
///
/// This type is constructed only by [`resolve_document_render_v2`]. That call accepts
/// one immutable document projection, invokes the closed verified-Atkinson Hyperlegible Next depiction entry,
/// and lowers the projection. It therefore has no API for combining separately-read
/// projections, resolutions, or plans.
#[derive(Debug)]
pub struct ResolvedDocumentRenderV2 {
    projection: DocumentProjectionV1,
    profile: DepictionProfileV1,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV4>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

impl ResolvedDocumentRenderV2 {
    fn from_projection(
        projection: DocumentProjectionV1,
        profile: DepictionProfileV1,
    ) -> Result<Self, ResolvedDocumentRenderErrorV2> {
        let resolution = render_document_projection_v1(&projection, &profile)?;
        let revision = projection.revision();
        let digest = projection.digest();
        let render_revision = RenderRevision::new(revision)
            .map_err(|_| ResolvedDocumentRenderErrorV2::ProvenanceMismatch)?;
        if resolution.projection_revision() != revision || resolution.projection_digest() != digest
        {
            return Err(ResolvedDocumentRenderErrorV2::ProvenanceMismatch);
        }
        if resolution.plans().iter().any(|entry| {
            entry.plan().revision() != render_revision
                || entry.plan().provenance().digest() != *digest
        }) {
            return Err(ResolvedDocumentRenderErrorV2::ProvenanceMismatch);
        }
        validate_projection_plan_roots(projection.molecules(), resolution.plans())?;
        validate_projection_plus_roots(
            projection.presentation_stack().entries(),
            resolution.plus_renders(),
        )?;
        validate_projection_text_roots(
            projection.presentation_stack().entries(),
            resolution.text_renders(),
        )?;
        Ok(Self {
            projection,
            profile,
            molecule_plans: resolution.plans().to_vec(),
            plus_renders: resolution.plus_renders().to_vec(),
            text_renders: resolution.text_renders().to_vec(),
            suppression: resolution.suppression(),
        })
    }

    /// Return the one authoritative document observation that produced every plan.
    #[must_use]
    pub const fn projection(&self) -> &DocumentProjectionV1 {
        &self.projection
    }

    /// Return the closed profile used to resolve document presentation facts.
    #[must_use]
    pub const fn profile(&self) -> DepictionProfileV1 {
        self.profile
    }

    /// Return complete molecule plans in document root order.
    #[must_use]
    pub fn molecule_plans(&self) -> &[DocumentMoleculeRenderPlanV4] {
        &self.molecule_plans
    }

    /// Return verified-Atkinson Hyperlegible Next plus layouts in document root order.
    #[must_use]
    pub fn plus_renders(&self) -> &[DocumentPlusRenderV1] {
        &self.plus_renders
    }

    /// Return verified-Atkinson Hyperlegible Next direct-root Text layouts in document root order.
    #[must_use]
    pub fn text_renders(&self) -> &[DocumentTextRenderV1] {
        &self.text_renders
    }

    /// Return the typed whole-projection suppression, when presentation is invalid.
    #[must_use]
    pub const fn suppression(&self) -> Option<DepictionSuppressionV1> {
        self.suppression
    }

    /// Return the frozen, validated render-facing wire DTO.
    #[must_use]
    pub fn wire(&self) -> ResolvedDocumentRenderWireV2 {
        ResolvedDocumentRenderWireV2 {
            schema: RESOLVED_DOCUMENT_RENDER_SCHEMA_V2.to_owned(),
            document: RenderDocumentProvenanceV1 {
                revision: self.projection.revision(),
                digest: *self.projection.digest(),
            },
            profile: self.profile.schema().to_owned(),
            molecule_plans: self.molecule_plans.clone(),
            plus_renders: self.plus_renders.clone(),
            text_renders: self.text_renders.clone(),
            suppression: self.suppression,
        }
    }
}

/// Resolve one lower immutable document projection without session authority.
pub fn resolve_document_render_v2(
    projection: DocumentProjectionV1,
    profile: DepictionProfileV1,
) -> Result<ResolvedDocumentRenderV2, ResolvedDocumentRenderErrorV2> {
    ResolvedDocumentRenderV2::from_projection(projection, profile)
}

/// Failure while producing one final render observation.
#[derive(Debug, Error)]
pub enum ResolvedDocumentRenderErrorV2 {
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
/// forge a `DocumentProjectionV1` or submit a session operation.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedDocumentRenderWireV2 {
    schema: String,
    document: RenderDocumentProvenanceV1,
    profile: String,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV4>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedResolvedDocumentRenderWireV2 {
    schema: String,
    document: RenderDocumentProvenanceV1,
    profile: String,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV4>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

impl ResolvedDocumentRenderWireV2 {
    fn from_unchecked(wire: UncheckedResolvedDocumentRenderWireV2) -> Result<Self, String> {
        let UncheckedResolvedDocumentRenderWireV2 {
            schema,
            document,
            profile,
            molecule_plans,
            plus_renders,
            text_renders,
            suppression,
        } = wire;
        if schema != RESOLVED_DOCUMENT_RENDER_SCHEMA_V2 || profile != DEPICTION_PROFILE_SCHEMA_V1 {
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
    pub fn molecule_plans(&self) -> &[DocumentMoleculeRenderPlanV4] {
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

    /// Decode only the exact V2 grammar and validate all provenance links.
    pub fn from_json(input: &str) -> Result<Self, RenderError> {
        serde_json::from_str(input).map_err(|error| RenderError::InvalidJson(error.to_string()))
    }
}

impl<'de> Deserialize<'de> for ResolvedDocumentRenderWireV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::from_unchecked(UncheckedResolvedDocumentRenderWireV2::deserialize(
            deserializer,
        )?)
        .map_err(serde::de::Error::custom)
    }
}

fn validate_projection_plus_roots(
    entries: &[PresentationStackEntryV1],
    renders: &[DocumentPlusRenderV1],
) -> Result<(), ResolvedDocumentRenderErrorV2> {
    let pluses = entries.iter().filter_map(|entry| match entry.root() {
        PresentationRootProjectionV1::Plus { plus } => Some(plus),
        _ => None,
    });
    for render in renders {
        let Some(plus) = pluses.clone().find(|plus| plus.target() == render.target()) else {
            return Err(ResolvedDocumentRenderErrorV2::PlusRootMismatch);
        };
        if plus.target() != render.target() {
            return Err(ResolvedDocumentRenderErrorV2::PlusRootMismatch);
        }
    }
    Ok(())
}

fn validate_projection_text_roots(
    entries: &[PresentationStackEntryV1],
    renders: &[DocumentTextRenderV1],
) -> Result<(), ResolvedDocumentRenderErrorV2> {
    let texts = entries.iter().filter_map(|entry| match entry.root() {
        PresentationRootProjectionV1::Text { text } => Some(text),
        _ => None,
    });
    for render in renders {
        let Some(text) = texts.clone().find(|text| text.target() == render.target()) else {
            return Err(ResolvedDocumentRenderErrorV2::TextRootMismatch);
        };
        if text.target() != render.target() {
            return Err(ResolvedDocumentRenderErrorV2::TextRootMismatch);
        }
    }
    Ok(())
}

fn validate_projection_plan_roots(
    molecules: &[MoleculeProjectionV1],
    plans: &[DocumentMoleculeRenderPlanV4],
) -> Result<(), ResolvedDocumentRenderErrorV2> {
    if molecules.len() != plans.len() {
        return Err(ResolvedDocumentRenderErrorV2::MoleculeRootMismatch);
    }
    for (molecule, entry) in molecules.iter().zip(plans) {
        if molecule.document_object_id() != entry.molecule().document_object_id() {
            return Err(ResolvedDocumentRenderErrorV2::MoleculeRootMismatch);
        }
    }
    Ok(())
}

fn validate_wire_plan_roots(plans: &[DocumentMoleculeRenderPlanV4]) -> Result<(), String> {
    let mut durable_ids = HashSet::new();
    for entry in plans {
        let root = entry.molecule();
        if !durable_ids.insert(root.document_object_id()) {
            return Err("molecule render plans contain a duplicate durable ID".to_owned());
        }
    }
    Ok(())
}

fn validate_wire_plus_roots(renders: &[DocumentPlusRenderV1]) -> Result<(), String> {
    let mut durable_ids = HashSet::new();
    for render in renders {
        let target = render.target();
        if !durable_ids.insert(target.document_object_id()) {
            return Err("plus renders contain a duplicate durable ID".to_owned());
        }
    }
    Ok(())
}

fn validate_wire_text_roots(renders: &[DocumentTextRenderV1]) -> Result<(), String> {
    let mut durable_ids = HashSet::new();
    for render in renders {
        let target = render.target();
        if !durable_ids.insert(target.document_object_id()) {
            return Err("Text renders contain a duplicate durable ID".to_owned());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DepictionIssueCodeV1, MoleculeMemberDepictionIssueV1};

    fn object_id(seed: u8) -> DocumentObjectIdV1 {
        DocumentObjectIdV1::from_entropy_bytes([seed; 16])
    }

    fn plan() -> MoleculeRenderPlanV4 {
        MoleculeRenderPlanV4::new(
            RenderProvenance::new(RenderRevision::new(1).expect("test revision"), [1; 32]),
            Vec::new(),
            Vec::new(),
        )
        .expect("empty molecule plan")
    }

    fn wire() -> ResolvedDocumentRenderWireV2 {
        ResolvedDocumentRenderWireV2 {
            schema: RESOLVED_DOCUMENT_RENDER_SCHEMA_V2.to_owned(),
            document: RenderDocumentProvenanceV1 {
                revision: 1,
                digest: [1; 32],
            },
            profile: DEPICTION_PROFILE_SCHEMA_V1.to_owned(),
            molecule_plans: Vec::new(),
            plus_renders: Vec::new(),
            text_renders: Vec::new(),
            suppression: None,
        }
    }

    #[test]
    fn molecule_member_issue_refuses_a_foreign_durable_target() {
        let owner = object_id(1);
        let member = object_id(2);
        let foreign = object_id(3);
        let result = DocumentMoleculeRenderPlanV4::from_document_object_id(
            owner,
            plan(),
            Vec::new(),
            vec![member],
            vec![MoleculeMemberDepictionIssueV1::new(
                foreign,
                DepictionIssueCodeV1::UnsupportedRichLabel,
                "test foreign member",
            )],
        );
        assert!(result.is_err());
    }

    #[test]
    fn molecule_member_issue_is_retained_by_its_molecule_plan() {
        let owner = object_id(4);
        let atom = object_id(5);
        let entry = DocumentMoleculeRenderPlanV4::from_document_object_id(
            owner,
            plan(),
            Vec::new(),
            vec![atom.clone(), object_id(6), object_id(7)],
            vec![MoleculeMemberDepictionIssueV1::new(
                atom.clone(),
                DepictionIssueCodeV1::UnsupportedRichLabel,
                "structured label unavailable",
            )],
        )
        .expect("owner-bound member issue");
        assert_eq!(entry.member_issues()[0].target(), &atom);
        assert_eq!(
            entry.member_issues()[0].code(),
            DepictionIssueCodeV1::UnsupportedRichLabel
        );
    }

    #[test]
    fn resolved_wire_refuses_the_retired_v1_schema() {
        let json = wire()
            .to_canonical_json()
            .expect("canonical V2 wire")
            .replace(
                RESOLVED_DOCUMENT_RENDER_SCHEMA_V2,
                "ferrum-resolved-document-render-v1",
            );
        assert!(ResolvedDocumentRenderWireV2::from_json(&json).is_err());
    }

    #[test]
    fn resolved_wire_refuses_unknown_fields() {
        let mut value = serde_json::to_value(wire()).expect("serialize V2 wire");
        value
            .as_object_mut()
            .expect("wire JSON object")
            .insert("compatibility".to_owned(), serde_json::Value::Null);
        let json = serde_json::to_string(&value).expect("wire JSON text");
        assert!(ResolvedDocumentRenderWireV2::from_json(&json).is_err());
    }

    #[test]
    fn resolved_wire_refuses_a_retired_nested_v3_plan() {
        let mut wire = wire();
        wire.molecule_plans.push(
            DocumentMoleculeRenderPlanV4::from_document_object_id(
                object_id(8),
                plan(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
            .expect("valid V4 document plan"),
        );
        let json = wire
            .to_canonical_json()
            .expect("canonical V2 wire")
            .replace("ferrum-render-plan-v4", "ferrum-render-plan-v3");
        assert!(ResolvedDocumentRenderWireV2::from_json(&json).is_err());
    }
}
