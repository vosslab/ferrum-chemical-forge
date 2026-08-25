//! API-owned composition of one document observation and verified render plans.

use std::collections::HashSet;

use crate::{
    CompactGroupRenderPrimitiveV1, MoleculeRenderPlan, RenderBatch, RenderError, RenderIssue,
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
pub const RESOLVED_DOCUMENT_RENDER_SCHEMA_V1: &str = "ferrum-resolved-document-render-v1";

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
pub struct DocumentMoleculeRenderPlanV2 {
    molecule: MoleculeRenderRootV1,
    plan: MoleculeRenderPlan,
    member_ids: Vec<DocumentObjectIdV1>,
    member_issues: Vec<MoleculeMemberDepictionIssueV1>,
    #[serde(skip, default)]
    compact_group_primitives: Vec<CompactGroupRenderPrimitiveV1>,
}

impl DocumentMoleculeRenderPlanV2 {
    pub(crate) fn from_document_object_id(
        document_object_id: DocumentObjectIdV1,
        plan: MoleculeRenderPlan,
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
/// This type is constructed only by [`observe_render_v1`]. That one call obtains the
/// document snapshot and projection, invokes the closed verified-Telex depiction entry,
/// and lowers the projection. It therefore has no API for combining separately-read
/// snapshots, projections, resolutions, or plans.
#[derive(Debug)]
pub struct ResolvedDocumentRenderV1 {
    projection: DocumentProjectionV1,
    profile: DepictionProfileV1,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

impl ResolvedDocumentRenderV1 {
    fn from_projection(
        projection: DocumentProjectionV1,
        profile: DepictionProfileV1,
    ) -> Result<Self, ResolvedDocumentRenderErrorV1> {
        let resolution = render_document_projection_v1(&projection, &profile)?;
        let revision = projection.revision();
        let digest = projection.digest();
        let render_revision = RenderRevision::new(revision)
            .map_err(|_| ResolvedDocumentRenderErrorV1::ProvenanceMismatch)?;
        if resolution.projection_revision() != revision || resolution.projection_digest() != digest
        {
            return Err(ResolvedDocumentRenderErrorV1::ProvenanceMismatch);
        }
        if resolution.plans().iter().any(|entry| {
            entry.plan().revision() != render_revision
                || entry.plan().provenance().digest() != *digest
        }) {
            return Err(ResolvedDocumentRenderErrorV1::ProvenanceMismatch);
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

    /// Return the typed whole-projection suppression, when presentation is invalid.
    #[must_use]
    pub const fn suppression(&self) -> Option<DepictionSuppressionV1> {
        self.suppression
    }

    /// Return the frozen, validated render-facing wire DTO.
    #[must_use]
    pub fn wire(&self) -> ResolvedDocumentRenderWireV1 {
        ResolvedDocumentRenderWireV1 {
            schema: RESOLVED_DOCUMENT_RENDER_SCHEMA_V1.to_owned(),
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
pub fn resolve_document_render_v1(
    projection: DocumentProjectionV1,
    profile: DepictionProfileV1,
) -> Result<ResolvedDocumentRenderV1, ResolvedDocumentRenderErrorV1> {
    ResolvedDocumentRenderV1::from_projection(projection, profile)
}

/// Failure while producing one final render observation.
#[derive(Debug, Error)]
pub enum ResolvedDocumentRenderErrorV1 {
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
pub struct ResolvedDocumentRenderWireV1 {
    schema: String,
    document: RenderDocumentProvenanceV1,
    profile: String,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UncheckedResolvedDocumentRenderWireV1 {
    schema: String,
    document: RenderDocumentProvenanceV1,
    profile: String,
    molecule_plans: Vec<DocumentMoleculeRenderPlanV2>,
    plus_renders: Vec<DocumentPlusRenderV1>,
    text_renders: Vec<DocumentTextRenderV1>,
    suppression: Option<DepictionSuppressionV1>,
}

impl ResolvedDocumentRenderWireV1 {
    fn from_unchecked(wire: UncheckedResolvedDocumentRenderWireV1) -> Result<Self, String> {
        let UncheckedResolvedDocumentRenderWireV1 {
            schema,
            document,
            profile,
            molecule_plans,
            plus_renders,
            text_renders,
            suppression,
        } = wire;
        if schema != RESOLVED_DOCUMENT_RENDER_SCHEMA_V1 || profile != DEPICTION_PROFILE_SCHEMA_V1 {
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

impl<'de> Deserialize<'de> for ResolvedDocumentRenderWireV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::from_unchecked(UncheckedResolvedDocumentRenderWireV1::deserialize(
            deserializer,
        )?)
        .map_err(serde::de::Error::custom)
    }
}

fn validate_projection_plus_roots(
    entries: &[PresentationStackEntryV1],
    renders: &[DocumentPlusRenderV1],
) -> Result<(), ResolvedDocumentRenderErrorV1> {
    let pluses = entries.iter().filter_map(|entry| match entry.root() {
        PresentationRootProjectionV1::Plus { plus } => Some(plus),
        _ => None,
    });
    for render in renders {
        let Some(plus) = pluses.clone().find(|plus| plus.target() == render.target()) else {
            return Err(ResolvedDocumentRenderErrorV1::PlusRootMismatch);
        };
        if plus.target() != render.target() {
            return Err(ResolvedDocumentRenderErrorV1::PlusRootMismatch);
        }
    }
    Ok(())
}

fn validate_projection_text_roots(
    entries: &[PresentationStackEntryV1],
    renders: &[DocumentTextRenderV1],
) -> Result<(), ResolvedDocumentRenderErrorV1> {
    let texts = entries.iter().filter_map(|entry| match entry.root() {
        PresentationRootProjectionV1::Text { text } => Some(text),
        _ => None,
    });
    for render in renders {
        let Some(text) = texts.clone().find(|text| text.target() == render.target()) else {
            return Err(ResolvedDocumentRenderErrorV1::TextRootMismatch);
        };
        if text.target() != render.target() {
            return Err(ResolvedDocumentRenderErrorV1::TextRootMismatch);
        }
    }
    Ok(())
}

fn validate_projection_plan_roots(
    molecules: &[MoleculeProjectionV1],
    plans: &[DocumentMoleculeRenderPlanV2],
) -> Result<(), ResolvedDocumentRenderErrorV1> {
    if molecules.len() != plans.len() {
        return Err(ResolvedDocumentRenderErrorV1::MoleculeRootMismatch);
    }
    for (molecule, entry) in molecules.iter().zip(plans) {
        if molecule.id() != Some(entry.molecule().document_object_id()) {
            return Err(ResolvedDocumentRenderErrorV1::MoleculeRootMismatch);
        }
    }
    Ok(())
}

fn validate_wire_plan_roots(plans: &[DocumentMoleculeRenderPlanV2]) -> Result<(), String> {
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

    fn plan() -> MoleculeRenderPlan {
        MoleculeRenderPlan::new(
            RenderProvenance::new(RenderRevision::new(1).expect("test revision"), [1; 32]),
            Vec::new(),
            Vec::new(),
        )
        .expect("empty molecule plan")
    }

    #[test]
    fn molecule_member_issue_refuses_a_foreign_durable_target() {
        let owner = object_id(1);
        let member = object_id(2);
        let foreign = object_id(3);
        let result = DocumentMoleculeRenderPlanV2::from_document_object_id(
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
        let entry = DocumentMoleculeRenderPlanV2::from_document_object_id(
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
}
