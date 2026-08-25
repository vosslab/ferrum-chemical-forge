//! Bounded, fenced oxidation-state observation for one selected document atom.

use ferrum_chemistry::{
    OxidationStateErrorV1, OxidationStateObservationV1, OxidationStateResourceV1,
    OxidationStateUnavailableReasonV1, admit_oxidation_state_root_v1,
    observe_admitted_oxidation_state_v1,
};
use ferrum_core::{Identifier, RecordId, RecordKind};
use thiserror::Error;

use crate::{DocumentObjectIdV1, DocumentSnapshot, TypedClass, TypedDocument};

use super::document_molecule_graph_v1::{DocumentMoleculeGraphError, document_molecule_graph_v1};

/// Immutable current-session fence and selected atom address for oxidation V1.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentAtomOxidationObservationRequestV1 {
    expected_revision: u64,
    expected_digest: [u8; 32],
    molecule_id: DocumentObjectIdV1,
    atom_id: DocumentObjectIdV1,
}

impl DocumentAtomOxidationObservationRequestV1 {
    /// Construct one exact-session request for an atom in one direct root.
    #[must_use]
    pub const fn new(
        expected_revision: u64,
        expected_digest: [u8; 32],
        molecule_id: DocumentObjectIdV1,
        atom_id: DocumentObjectIdV1,
    ) -> Self {
        Self {
            expected_revision,
            expected_digest,
            molecule_id,
            atom_id,
        }
    }
    #[must_use]
    pub const fn expected_revision(&self) -> u64 {
        self.expected_revision
    }
    #[must_use]
    pub const fn expected_digest(&self) -> &[u8; 32] {
        &self.expected_digest
    }
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }
    #[must_use]
    pub const fn atom_id(&self) -> &DocumentObjectIdV1 {
        &self.atom_id
    }
}

/// Closed unavailability vocabulary for a complete selected root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentAtomOxidationUnavailableReasonV1 {
    ElementOutsideProfile,
    FormalChargeUnavailable,
    HydrogenTopologyUnsupported,
    AromaticityUnsupported,
    RadicalUnsupported,
    BondOrderUnavailable,
    BondOrderUnsupported,
    NonAtomVertexUnsupported,
    CoordinationOrDelocalizationUnsupported,
    ComponentInvariantFailed,
    ArithmeticOverflow,
}

/// One read-only selected-atom oxidation observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentAtomOxidationObservationV1 {
    Accepted {
        oxidation_number: i16,
    },
    Unavailable {
        reason: DocumentAtomOxidationUnavailableReasonV1,
    },
}

/// One chemistry-owned resource bound reported at the document boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentAtomOxidationResourceV1 {
    Atoms,
    Bonds,
    Components,
}

/// Typed refusal before the selected root can be assessed.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DocumentAtomOxidationRefusalV1 {
    #[error("document revision is stale")]
    StaleObservation,
    #[error("document digest is stale")]
    DigestMismatch,
    #[error("selected molecule is not one durable direct root")]
    UnknownDirectMolecule,
    #[error("selected atom is not a durable authored atom")]
    UnknownAtom,
    #[error("selected atom does not belong to the selected direct root")]
    AtomNotInSelectedRoot,
    #[error("direct-root provenance cannot be corroborated")]
    DirectRootMismatch,
    #[error("document facts cannot be lowered into the closed oxidation-state profile")]
    UnsupportedDocument,
    #[error("oxidation-state computation rejected the authenticated graph")]
    InvalidAuthenticatedGraph,
}

/// The public result of a selected-root oxidation assessment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentAtomOxidationResultV1 {
    Observation(DocumentAtomOxidationObservationV1),
    ResourceLimit {
        resource: DocumentAtomOxidationResourceV1,
        maximum: usize,
        actual: usize,
    },
}

/// Execute one read-only observation against the current retained state.
pub(crate) fn observe_current_document_atom_oxidation_v1(
    document: &TypedDocument,
    snapshot: &DocumentSnapshot,
    request: &DocumentAtomOxidationObservationRequestV1,
) -> Result<DocumentAtomOxidationResultV1, DocumentAtomOxidationRefusalV1> {
    if snapshot.revision() != request.expected_revision {
        return Err(DocumentAtomOxidationRefusalV1::StaleObservation);
    }
    if snapshot.digest() != &request.expected_digest {
        return Err(DocumentAtomOxidationRefusalV1::DigestMismatch);
    }
    let root = document
        .resolve_document_object_id(&request.molecule_id)
        .filter(|record| {
            record.class() == TypedClass::Molecule && record.path().components().len() == 1
        })
        .ok_or(DocumentAtomOxidationRefusalV1::UnknownDirectMolecule)?;
    let selected = document
        .resolve_document_object_id(&request.atom_id)
        .filter(|record| record.class() == TypedClass::Atom)
        .ok_or(DocumentAtomOxidationRefusalV1::UnknownAtom)?;
    if selected.path().components().len() != 2
        || selected.path().components().first() != root.path().components().first()
    {
        return Err(DocumentAtomOxidationRefusalV1::AtomNotInSelectedRoot);
    }
    let atom_count = root.children_of(TypedClass::Atom).count();
    let bond_count = root.children_of(TypedClass::Bond).count();
    let admission = match admit_oxidation_state_root_v1(atom_count, bond_count) {
        Ok(admission) => admission,
        Err(OxidationStateErrorV1::ResourceLimit {
            resource,
            maximum,
            actual,
        }) => {
            return Ok(DocumentAtomOxidationResultV1::ResourceLimit {
                resource: map_resource(resource),
                maximum,
                actual,
            });
        }
        Err(_) => return Err(DocumentAtomOxidationRefusalV1::InvalidAuthenticatedGraph),
    };
    let root_source = root
        .attribute("id")
        .ok_or(DocumentAtomOxidationRefusalV1::DirectRootMismatch)?;
    let selected_atom_record_id = RecordId::new(
        RecordKind::Atom,
        Identifier::new(
            selected
                .attribute("id")
                .ok_or(DocumentAtomOxidationRefusalV1::DirectRootMismatch)?
                .to_owned(),
        )
        .map_err(|_| DocumentAtomOxidationRefusalV1::DirectRootMismatch)?,
    )
    .map_err(|_| DocumentAtomOxidationRefusalV1::DirectRootMismatch)?;
    if let Some(reason) = selected_root_profile_unavailability_v1(root) {
        return Ok(unavailable(reason));
    }
    let molecule = document
        .core_molecule(&request.molecule_id)
        .map_err(|_| DocumentAtomOxidationRefusalV1::UnsupportedDocument)?
        .ok_or(DocumentAtomOxidationRefusalV1::DirectRootMismatch)?;
    if molecule.source_id().as_str() != root_source {
        return Err(DocumentAtomOxidationRefusalV1::DirectRootMismatch);
    }
    let lowered = match document_molecule_graph_v1(&molecule) {
        Ok(value) => value,
        Err(error) => return Ok(unavailable(map_lowering(error)?)),
    };
    let (graph, _, records) = lowered.into_parts_with_atom_records();
    let selected_position = records
        .iter()
        .position(|record| record == &selected_atom_record_id)
        .ok_or(DocumentAtomOxidationRefusalV1::DirectRootMismatch)?;
    match observe_admitted_oxidation_state_v1(&admission, &graph, selected_position) {
        Ok(value) => Ok(DocumentAtomOxidationResultV1::Observation(map_observation(
            value,
        ))),
        Err(OxidationStateErrorV1::ResourceLimit {
            resource,
            maximum,
            actual,
        }) => Ok(DocumentAtomOxidationResultV1::ResourceLimit {
            resource: map_resource(resource),
            maximum,
            actual,
        }),
        Err(
            OxidationStateErrorV1::SelectedAtomOutOfRange { .. }
            | OxidationStateErrorV1::InvalidGraphStructure,
        ) => Err(DocumentAtomOxidationRefusalV1::InvalidAuthenticatedGraph),
    }
}

fn unavailable(reason: DocumentAtomOxidationUnavailableReasonV1) -> DocumentAtomOxidationResultV1 {
    DocumentAtomOxidationResultV1::Observation(DocumentAtomOxidationObservationV1::Unavailable {
        reason,
    })
}

/// Classify ordinary, complete-root source facts before allocating an owned core molecule.
///
/// This intentionally covers the chemistry profile rather than structural provenance. Missing
/// bond endpoints, invalid points, and durable identity mismatches remain typed refusals from
/// the authenticated loader path below.
fn selected_root_profile_unavailability_v1(
    root: &crate::TypedRecord,
) -> Option<DocumentAtomOxidationUnavailableReasonV1> {
    for child in root.typed_children() {
        let record = child.record();
        match record.class() {
            TypedClass::Atom => {
                let element = record.attribute("name");
                if !matches!(element, Some("H" | "C" | "N" | "O")) {
                    return Some(DocumentAtomOxidationUnavailableReasonV1::ElementOutsideProfile);
                }
                let charge = record
                    .attribute("charge")
                    .and_then(|value| value.parse::<i32>().ok());
                if !matches!(charge, Some(-4..=4)) {
                    return Some(DocumentAtomOxidationUnavailableReasonV1::FormalChargeUnavailable);
                }
                if record.attribute("explicit_hydrogens") != Some("0") {
                    return Some(
                        DocumentAtomOxidationUnavailableReasonV1::HydrogenTopologyUnsupported,
                    );
                }
                if record
                    .attribute("multiplicity")
                    .is_some_and(|value| value != "1")
                {
                    return Some(DocumentAtomOxidationUnavailableReasonV1::RadicalUnsupported);
                }
                if record.attribute("isotope").is_some()
                    || record.attribute("valency").is_some()
                    || record.attribute("multiplicity").is_some()
                    || record.attribute("free_sites").is_some()
                {
                    return Some(
                        DocumentAtomOxidationUnavailableReasonV1::CoordinationOrDelocalizationUnsupported,
                    );
                }
            }
            TypedClass::Bond => {
                let Some(source_type) = record.attribute("type") else {
                    return Some(DocumentAtomOxidationUnavailableReasonV1::BondOrderUnavailable);
                };
                if source_type == "n4" {
                    return Some(DocumentAtomOxidationUnavailableReasonV1::AromaticityUnsupported);
                }
                if !matches!(source_type, "n1" | "n2" | "n3") {
                    return Some(DocumentAtomOxidationUnavailableReasonV1::BondOrderUnsupported);
                }
            }
            TypedClass::Group | TypedClass::MoleculeText | TypedClass::Query => {
                return Some(DocumentAtomOxidationUnavailableReasonV1::NonAtomVertexUnsupported);
            }
            _ => {}
        }
    }
    None
}

fn map_lowering(
    error: DocumentMoleculeGraphError,
) -> Result<DocumentAtomOxidationUnavailableReasonV1, DocumentAtomOxidationRefusalV1> {
    match error {
        DocumentMoleculeGraphError::UnsupportedVertex { .. }
        | DocumentMoleculeGraphError::UnsupportedBondEndpoint { .. } => {
            Ok(DocumentAtomOxidationUnavailableReasonV1::NonAtomVertexUnsupported)
        }
        DocumentMoleculeGraphError::MissingElement { .. }
        | DocumentMoleculeGraphError::InvalidElement { .. } => {
            Ok(DocumentAtomOxidationUnavailableReasonV1::ElementOutsideProfile)
        }
        DocumentMoleculeGraphError::UnsupportedAtomFact { .. }
        | DocumentMoleculeGraphError::UnsupportedBondStyle { .. } => {
            Ok(DocumentAtomOxidationUnavailableReasonV1::CoordinationOrDelocalizationUnsupported)
        }
        DocumentMoleculeGraphError::UnsupportedBondOrder { .. } => {
            Ok(DocumentAtomOxidationUnavailableReasonV1::BondOrderUnavailable)
        }
        DocumentMoleculeGraphError::EmptyMolecule | DocumentMoleculeGraphError::Graph(_) => {
            Err(DocumentAtomOxidationRefusalV1::InvalidAuthenticatedGraph)
        }
        DocumentMoleculeGraphError::DuplicateAtomIdentity { .. }
        | DocumentMoleculeGraphError::ResourceAllocation => {
            Err(DocumentAtomOxidationRefusalV1::UnsupportedDocument)
        }
    }
}

fn map_observation(value: OxidationStateObservationV1) -> DocumentAtomOxidationObservationV1 {
    match value {
        OxidationStateObservationV1::Accepted { oxidation_number } => {
            DocumentAtomOxidationObservationV1::Accepted { oxidation_number }
        }
        OxidationStateObservationV1::Unavailable { reason } => {
            DocumentAtomOxidationObservationV1::Unavailable {
                reason: map_unavailable(reason),
            }
        }
    }
}

fn map_unavailable(
    reason: OxidationStateUnavailableReasonV1,
) -> DocumentAtomOxidationUnavailableReasonV1 {
    match reason {
        OxidationStateUnavailableReasonV1::ElementOutsideProfile => {
            DocumentAtomOxidationUnavailableReasonV1::ElementOutsideProfile
        }
        OxidationStateUnavailableReasonV1::FormalChargeUnavailable => {
            DocumentAtomOxidationUnavailableReasonV1::FormalChargeUnavailable
        }
        OxidationStateUnavailableReasonV1::HydrogenTopologyUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::HydrogenTopologyUnsupported
        }
        OxidationStateUnavailableReasonV1::AromaticityUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::AromaticityUnsupported
        }
        OxidationStateUnavailableReasonV1::RadicalUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::RadicalUnsupported
        }
        OxidationStateUnavailableReasonV1::BondOrderUnavailable => {
            DocumentAtomOxidationUnavailableReasonV1::BondOrderUnavailable
        }
        OxidationStateUnavailableReasonV1::BondOrderUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::BondOrderUnsupported
        }
        OxidationStateUnavailableReasonV1::NonAtomVertexUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::NonAtomVertexUnsupported
        }
        OxidationStateUnavailableReasonV1::CoordinationOrDelocalizationUnsupported => {
            DocumentAtomOxidationUnavailableReasonV1::CoordinationOrDelocalizationUnsupported
        }
        OxidationStateUnavailableReasonV1::ComponentInvariantFailed => {
            DocumentAtomOxidationUnavailableReasonV1::ComponentInvariantFailed
        }
        OxidationStateUnavailableReasonV1::ArithmeticOverflow => {
            DocumentAtomOxidationUnavailableReasonV1::ArithmeticOverflow
        }
    }
}

fn map_resource(resource: OxidationStateResourceV1) -> DocumentAtomOxidationResourceV1 {
    match resource {
        OxidationStateResourceV1::Atoms => DocumentAtomOxidationResourceV1::Atoms,
        OxidationStateResourceV1::Bonds => DocumentAtomOxidationResourceV1::Bonds,
        OxidationStateResourceV1::Components => DocumentAtomOxidationResourceV1::Components,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentSession;

    fn selected_request(
        session: &DocumentSession,
        root: usize,
        atom: usize,
    ) -> DocumentAtomOxidationObservationRequestV1 {
        let observation = session.observe(0).expect("current observation");
        let molecule = &observation.projection().molecules()[root];
        DocumentAtomOxidationObservationRequestV1::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            molecule.id().expect("durable root").clone(),
            molecule.atoms()[atom].id().expect("durable atom").clone(),
        )
    }

    fn water() -> &'static str {
        concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"><molecule id=\"water\">",
            "<atom id=\"oxygen\" name=\"O\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"0\" y=\"0\"/></atom>",
            "<atom id=\"hydrogen-a\" name=\"H\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"1\" y=\"0\"/></atom>",
            "<atom id=\"hydrogen-b\" name=\"H\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"-1\" y=\"0\"/></atom>",
            "<bond id=\"bond-a\" start=\"oxygen\" end=\"hydrogen-a\" type=\"n1\"/>",
            "<bond id=\"bond-b\" start=\"oxygen\" end=\"hydrogen-b\" type=\"n1\"/>",
            "</molecule></cdml>"
        )
    }

    #[test]
    fn session_operation_observes_selected_atom_without_mutation() {
        let session = DocumentSession::load(water()).expect("water");
        let before = session.snapshot().expect("before");
        assert_eq!(
            session.observe_atom_oxidation_v1(&selected_request(&session, 0, 0)),
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Accepted {
                    oxidation_number: -2
                }
            ))
        );
        assert_eq!(session.snapshot().expect("after"), before);
    }

    #[test]
    fn remote_unsupported_fact_makes_the_selected_atom_unavailable() {
        let source = water().replace(
            "</molecule>",
            "<query id=\"remote\" name=\"R\"><point x=\"3\" y=\"0\"/></query></molecule>",
        );
        let session = DocumentSession::load(&source).expect("root with group");
        assert_eq!(
            session.observe_atom_oxidation_v1(&selected_request(&session, 0, 0)),
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Unavailable {
                    reason: DocumentAtomOxidationUnavailableReasonV1::NonAtomVertexUnsupported
                }
            ))
        );
    }

    #[test]
    fn selected_root_exceeding_the_atom_bound_is_refused_before_lowering() {
        let atoms = (0..257)
            .map(|index| {
                format!(
                    "<atom id=\"h{index}\" name=\"H\" charge=\"0\"><point x=\"0\" y=\"0\"/></atom>"
                )
            })
            .collect::<String>();
        let source = format!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"large\">{atoms}</molecule></cdml>"
        );
        let session = DocumentSession::load(&source).expect("large valid root");
        assert_eq!(
            session.observe_atom_oxidation_v1(&selected_request(&session, 0, 0)),
            Ok(DocumentAtomOxidationResultV1::ResourceLimit {
                resource: DocumentAtomOxidationResourceV1::Atoms,
                maximum: 256,
                actual: 257,
            })
        );
    }

    #[test]
    fn stale_request_is_refused_by_the_session_fence() {
        let session = DocumentSession::load(water()).expect("water");
        let mut request = selected_request(&session, 0, 0);
        request.expected_revision = 1;
        assert_eq!(
            session.observe_atom_oxidation_v1(&request),
            Err(DocumentAtomOxidationRefusalV1::StaleObservation)
        );
    }

    #[test]
    fn nonfirst_selected_atom_returns_its_own_oxidation_number() {
        let session = DocumentSession::load(water()).expect("water");
        assert_eq!(
            session.observe_atom_oxidation_v1(&selected_request(&session, 0, 1)),
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Accepted {
                    oxidation_number: 1
                }
            ))
        );
    }

    #[test]
    fn digest_mismatch_is_refused_without_mutation() {
        let session = DocumentSession::load(water()).expect("water");
        let before = session.snapshot().expect("before");
        let mut request = selected_request(&session, 0, 0);
        request.expected_digest[0] ^= 1;
        assert_eq!(
            session.observe_atom_oxidation_v1(&request),
            Err(DocumentAtomOxidationRefusalV1::DigestMismatch)
        );
        assert_eq!(session.snapshot().expect("after"), before);
    }

    #[test]
    fn atom_from_another_direct_root_is_refused() {
        let source = format!(
            "{}<molecule id=\"second\"><atom id=\"carbon\" name=\"C\" charge=\"0\" explicit_hydrogens=\"0\"><point x=\"3\" y=\"0\"/></atom></molecule>",
            water().replace("</cdml>", "")
        );
        let source = format!("{source}</cdml>");
        let session = DocumentSession::load(&source).expect("two roots");
        let observation = session.observe(0).expect("current observation");
        let first = &observation.projection().molecules()[0];
        let second = &observation.projection().molecules()[1];
        let request = DocumentAtomOxidationObservationRequestV1::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            first.id().expect("first durable root").clone(),
            second.atoms()[0].id().expect("second durable atom").clone(),
        );
        assert_eq!(
            session.observe_atom_oxidation_v1(&request),
            Err(DocumentAtomOxidationRefusalV1::AtomNotInSelectedRoot)
        );
    }

    #[test]
    fn remote_radical_profile_fact_is_closed_unavailable() {
        let source = water().replace(
            "<atom id=\"hydrogen-b\"",
            "<atom id=\"hydrogen-b\" multiplicity=\"2\"",
        );
        let session = DocumentSession::load(&source).expect("root with remote radical");
        assert_eq!(
            session.observe_atom_oxidation_v1(&selected_request(&session, 0, 0)),
            Ok(DocumentAtomOxidationResultV1::Observation(
                DocumentAtomOxidationObservationV1::Unavailable {
                    reason: DocumentAtomOxidationUnavailableReasonV1::RadicalUnsupported
                }
            ))
        );
    }
}
