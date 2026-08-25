//! Outer immutable document-projection aggregate and provenance.

use serde::Serialize;
use thiserror::Error;

use crate::{
    DrawingStandardV1, MoleculeProjectionV1, PaperLayoutProjectionV1,
    PresentationStackProjectionV1, ProjectionIssueV1,
};

/// Stable schema identifier for [`DocumentProjectionV1`].
pub const DOCUMENT_PROJECTION_SCHEMA_V1: &str = "ferrum-document-projection-v1";

/// Snapshot provenance for one immutable document projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DocumentProjectionProvenanceV1 {
    revision: u64,
    digest: [u8; 32],
    is_dirty: bool,
}

impl DocumentProjectionProvenanceV1 {
    #[must_use]
    pub const fn new(revision: u64, digest: [u8; 32], is_dirty: bool) -> Self {
        Self {
            revision,
            digest,
            is_dirty,
        }
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
    pub const fn is_dirty(&self) -> bool {
        self.is_dirty
    }
}

/// Failure while composing immutable values from one snapshot.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DocumentProjectionV1Error {
    #[error("presentation stack revision differs from document projection provenance")]
    PresentationRevisionMismatch,
    #[error("presentation stack digest differs from document projection provenance")]
    PresentationDigestMismatch,
    #[error("stereo depiction names a molecule outside this projection: {molecule_id}")]
    StereoDepictionMoleculeMissing { molecule_id: String },
}

/// Immutable V1 projection from one authoritative document snapshot.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentProjectionV1 {
    schema: &'static str,
    provenance: DocumentProjectionProvenanceV1,
    drawing_standard: Option<DrawingStandardV1>,
    paper_layout: PaperLayoutProjectionV1,
    molecules: Vec<MoleculeProjectionV1>,
    presentation_stack: PresentationStackProjectionV1,
    issues: Vec<ProjectionIssueV1>,
}

impl DocumentProjectionV1 {
    /// Construct one complete immutable projection with a single snapshot provenance.
    pub fn try_new(
        provenance: DocumentProjectionProvenanceV1,
        drawing_standard: Option<DrawingStandardV1>,
        paper_layout: PaperLayoutProjectionV1,
        molecules: Vec<MoleculeProjectionV1>,
        presentation_stack: PresentationStackProjectionV1,
        issues: Vec<ProjectionIssueV1>,
    ) -> Result<Self, DocumentProjectionV1Error> {
        if presentation_stack.revision() != provenance.revision() {
            return Err(DocumentProjectionV1Error::PresentationRevisionMismatch);
        }
        if presentation_stack.digest() != provenance.digest() {
            return Err(DocumentProjectionV1Error::PresentationDigestMismatch);
        }
        Ok(Self {
            schema: DOCUMENT_PROJECTION_SCHEMA_V1,
            provenance,
            drawing_standard,
            paper_layout,
            molecules,
            presentation_stack,
            issues,
        })
    }

    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.provenance.revision()
    }

    #[must_use]
    pub const fn digest(&self) -> &[u8; 32] {
        self.provenance.digest()
    }

    #[must_use]
    pub const fn is_dirty(&self) -> bool {
        self.provenance.is_dirty()
    }

    #[must_use]
    pub fn drawing_standard(&self) -> Option<&DrawingStandardV1> {
        self.drawing_standard.as_ref()
    }

    #[must_use]
    pub const fn paper_layout(&self) -> &PaperLayoutProjectionV1 {
        &self.paper_layout
    }

    #[must_use]
    pub fn molecules(&self) -> &[MoleculeProjectionV1] {
        &self.molecules
    }

    #[must_use]
    pub const fn presentation_stack(&self) -> &PresentationStackProjectionV1 {
        &self.presentation_stack
    }

    #[must_use]
    pub fn issues(&self) -> &[ProjectionIssueV1] {
        &self.issues
    }

    /// Attach resolved E/Z drawing facts to one existing molecule projection.
    pub fn with_molecule_double_bond_carrier_marks(
        mut self,
        molecule_id: &crate::DocumentObjectIdV1,
        marks: Vec<crate::DoubleBondCarrierMarkProjectionV1>,
    ) -> Result<Self, DocumentProjectionV1Error> {
        let Some(molecule) = self
            .molecules
            .iter_mut()
            .find(|molecule| molecule.id() == Some(molecule_id))
        else {
            return Err(DocumentProjectionV1Error::StereoDepictionMoleculeMissing {
                molecule_id: molecule_id.as_str().to_owned(),
            });
        };
        *molecule = molecule.clone().with_double_bond_carrier_marks(marks);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DOCUMENT_PROJECTION_SCHEMA_V1, DocumentProjectionProvenanceV1, DocumentProjectionV1,
        DocumentProjectionV1Error,
    };
    use crate::{
        PaperAttributesV1, PaperLayoutProjectionV1, PaperOrientationV1, PaperPageV1,
        PositiveFiniteV1, PresentationStackProjectionV1, ViewportAttributesV1,
    };

    fn stack(revision: u64, digest: [u8; 32]) -> PresentationStackProjectionV1 {
        PresentationStackProjectionV1::new(revision, digest, Vec::new(), Vec::new(), Vec::new())
            .expect("an empty presentation stack is valid")
    }

    fn aggregate(
        provenance: DocumentProjectionProvenanceV1,
        presentation_stack: PresentationStackProjectionV1,
    ) -> Result<DocumentProjectionV1, DocumentProjectionV1Error> {
        DocumentProjectionV1::try_new(
            provenance,
            None,
            paper_layout(),
            Vec::new(),
            presentation_stack,
            Vec::new(),
        )
    }

    fn paper_layout() -> PaperLayoutProjectionV1 {
        PaperLayoutProjectionV1::new(
            0,
            [0; 32],
            false,
            PaperAttributesV1::default(),
            PaperAttributesV1::default(),
            ViewportAttributesV1::default(),
            "A4".to_owned(),
            PaperOrientationV1::Portrait,
            PaperPageV1::from_resolved_dimensions(
                PositiveFiniteV1::new(210.0).expect("A4 width is positive"),
                PositiveFiniteV1::new(297.0).expect("A4 height is positive"),
                PaperOrientationV1::Portrait,
                None,
            )
            .expect("A4 dimensions have finite scene bounds"),
        )
    }

    #[test]
    fn aggregate_keeps_schema_and_authoritative_provenance() {
        let projection = aggregate(
            DocumentProjectionProvenanceV1::new(7, [3; 32], true),
            stack(7, [3; 32]),
        )
        .expect("matching snapshot provenance is valid");

        assert_eq!(projection.schema(), DOCUMENT_PROJECTION_SCHEMA_V1);
        assert_eq!(projection.revision(), 7);
        assert_eq!(projection.digest(), &[3; 32]);
        assert!(projection.is_dirty());
    }

    #[test]
    fn aggregate_refuses_stack_revision_from_another_snapshot() {
        assert_eq!(
            aggregate(
                DocumentProjectionProvenanceV1::new(7, [3; 32], false),
                stack(8, [3; 32]),
            ),
            Err(DocumentProjectionV1Error::PresentationRevisionMismatch)
        );
    }

    #[test]
    fn aggregate_refuses_stack_digest_from_another_snapshot() {
        assert_eq!(
            aggregate(
                DocumentProjectionProvenanceV1::new(7, [3; 32], false),
                stack(7, [4; 32]),
            ),
            Err(DocumentProjectionV1Error::PresentationDigestMismatch)
        );
    }
}
