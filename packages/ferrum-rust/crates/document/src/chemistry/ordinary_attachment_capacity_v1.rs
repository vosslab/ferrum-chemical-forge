//! Candidate-aware document adapter for ordinary single-bond attachment capacity.

use std::collections::{HashMap, HashSet};

use ferrum_chemistry::{
    OrdinaryAttachmentAnchorV1, OrdinaryAttachmentBondOrderV1, OrdinaryAttachmentCapacityOutcomeV1,
    OrdinaryAttachmentCapacityReasonV1, OrdinaryAttachmentCapacityRecoveryV1,
    OrdinaryAttachmentProfileV1, admit_ordinary_attachment_capacity_v1,
};
use ferrum_core::{BondOrder, Molecule, RecordId, VertexRef};
use thiserror::Error;

use super::document_chemistry_limits_v1::{
    DOCUMENT_CHEMISTRY_MAX_BONDS_V1, DOCUMENT_CHEMISTRY_MAX_COMPONENTS_V1,
    DOCUMENT_CHEMISTRY_MAX_VERTICES_V1,
};

/// Exact generated exterior bond expected in one detached typed candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrdinaryAttachmentCandidateWitnessV1 {
    anchor_atom: RecordId,
    added_group: RecordId,
    added_bond: RecordId,
    profile: OrdinaryAttachmentProfileV1,
}

impl OrdinaryAttachmentCandidateWitnessV1 {
    #[must_use]
    pub(crate) const fn new(
        anchor_atom: RecordId,
        added_group: RecordId,
        added_bond: RecordId,
        profile: OrdinaryAttachmentProfileV1,
    ) -> Self {
        Self {
            anchor_atom,
            added_group,
            added_bond,
            profile,
        }
    }
}

/// Stable candidate admission result suitable for later renderer preflight composition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentOrdinaryAttachmentAvailabilityV1 {
    Available,
    Unavailable {
        reason: DocumentOrdinaryAttachmentReasonV1,
        recovery: DocumentOrdinaryAttachmentRecoveryV1,
    },
}

/// Closed document-side unavailable reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentOrdinaryAttachmentReasonV1 {
    ResourceLimit,
    SourceFactsUnsupported,
    CapacityUnavailable,
}

/// Closed recovery vocabulary for the candidate admission boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DocumentOrdinaryAttachmentRecoveryV1 {
    ReduceRoot,
    EditStructure,
    ChooseAnotherAtom,
}

/// Candidate construction faults are distinct from chemistry availability.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum OrdinaryAttachmentCandidateErrorV1 {
    #[error("ordinary attachment candidate does not contain its exact witnessed records")]
    WitnessMismatch,
    #[error("ordinary attachment candidate graph is structurally invalid")]
    GraphInvalid,
}

/// Validate a detached candidate and calculate its selected atom's incoming-bond slot.
pub(crate) fn admit_candidate_ordinary_attachment_capacity_v1(
    molecule: &Molecule,
    witness: &OrdinaryAttachmentCandidateWitnessV1,
) -> Result<DocumentOrdinaryAttachmentAvailabilityV1, OrdinaryAttachmentCandidateErrorV1> {
    if !within_resource_limits(molecule)? {
        return Ok(DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
            reason: DocumentOrdinaryAttachmentReasonV1::ResourceLimit,
            recovery: DocumentOrdinaryAttachmentRecoveryV1::ReduceRoot,
        });
    }
    let anchor = molecule
        .atoms()
        .iter()
        .find(|atom| atom.identity() == &witness.anchor_atom)
        .ok_or(OrdinaryAttachmentCandidateErrorV1::WitnessMismatch)?;
    if !molecule
        .groups()
        .iter()
        .any(|group| group.identity() == &witness.added_group)
    {
        return Err(OrdinaryAttachmentCandidateErrorV1::WitnessMismatch);
    }
    let anchor_vertex = VertexRef::Atom(witness.anchor_atom.clone());
    let group_vertex = VertexRef::Group(witness.added_group.clone());
    let witnessed = molecule
        .bonds()
        .iter()
        .find(|bond| bond.identity() == &witness.added_bond)
        .ok_or(OrdinaryAttachmentCandidateErrorV1::WitnessMismatch)?;
    let joins_witness = (witnessed.start() == &anchor_vertex && witnessed.end() == &group_vertex)
        || (witnessed.start() == &group_vertex && witnessed.end() == &anchor_vertex);
    if !joins_witness
        || map_bond_order(witnessed.order(), witnessed.aromatic())
            != OrdinaryAttachmentBondOrderV1::Single
    {
        return Err(OrdinaryAttachmentCandidateErrorV1::WitnessMismatch);
    }
    let mut incident_orders = Vec::new();
    incident_orders
        .try_reserve_exact(molecule.bonds().len())
        .map_err(|_| OrdinaryAttachmentCandidateErrorV1::GraphInvalid)?;
    for bond in molecule.bonds() {
        if bond.identity() == &witness.added_bond {
            continue;
        }
        if bond.start() == &anchor_vertex || bond.end() == &anchor_vertex {
            incident_orders.push(map_bond_order(bond.order(), bond.aromatic()));
        }
    }
    let Some(element) = anchor.element() else {
        return Ok(source_unavailable());
    };
    let outcome = admit_ordinary_attachment_capacity_v1(
        witness.profile,
        OrdinaryAttachmentAnchorV1 {
            element,
            formal_charge: anchor.formal_charge(),
            explicit_hydrogens: anchor.explicit_hydrogens(),
            authored_valence: anchor.valence(),
            multiplicity: anchor.multiplicity(),
            free_sites: anchor.free_sites(),
            incident_bond_orders: &incident_orders,
        },
    );
    Ok(map_outcome(outcome))
}

fn map_outcome(
    outcome: OrdinaryAttachmentCapacityOutcomeV1,
) -> DocumentOrdinaryAttachmentAvailabilityV1 {
    match outcome {
        OrdinaryAttachmentCapacityOutcomeV1::Admitted(_) => {
            DocumentOrdinaryAttachmentAvailabilityV1::Available
        }
        OrdinaryAttachmentCapacityOutcomeV1::Unavailable { reason, recovery } => {
            let document_reason = match reason {
                OrdinaryAttachmentCapacityReasonV1::ElementOutsideProfile
                | OrdinaryAttachmentCapacityReasonV1::ChargeOutsideProfile
                | OrdinaryAttachmentCapacityReasonV1::AuthoredCapacityOverride
                | OrdinaryAttachmentCapacityReasonV1::RadicalOrMultiplicity
                | OrdinaryAttachmentCapacityReasonV1::AromaticBond
                | OrdinaryAttachmentCapacityReasonV1::UnsupportedBondOrder
                | OrdinaryAttachmentCapacityReasonV1::DemandOverflow => {
                    DocumentOrdinaryAttachmentReasonV1::SourceFactsUnsupported
                }
                OrdinaryAttachmentCapacityReasonV1::ExceedsCapacity => {
                    DocumentOrdinaryAttachmentReasonV1::CapacityUnavailable
                }
            };
            let document_recovery = match recovery {
                OrdinaryAttachmentCapacityRecoveryV1::ChooseAnotherAtom => {
                    DocumentOrdinaryAttachmentRecoveryV1::ChooseAnotherAtom
                }
                OrdinaryAttachmentCapacityRecoveryV1::UseSupportedOrdinaryStructure
                | OrdinaryAttachmentCapacityRecoveryV1::RemoveOrChangeAuthoredCapacityFact => {
                    DocumentOrdinaryAttachmentRecoveryV1::EditStructure
                }
            };
            DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
                reason: document_reason,
                recovery: document_recovery,
            }
        }
    }
}

const fn source_unavailable() -> DocumentOrdinaryAttachmentAvailabilityV1 {
    DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
        reason: DocumentOrdinaryAttachmentReasonV1::SourceFactsUnsupported,
        recovery: DocumentOrdinaryAttachmentRecoveryV1::EditStructure,
    }
}

fn map_bond_order(
    order: Option<BondOrder>,
    aromatic: Option<bool>,
) -> OrdinaryAttachmentBondOrderV1 {
    if aromatic == Some(true) {
        return OrdinaryAttachmentBondOrderV1::Aromatic;
    }
    match order {
        Some(BondOrder::Single) => OrdinaryAttachmentBondOrderV1::Single,
        Some(BondOrder::Double) => OrdinaryAttachmentBondOrderV1::Double,
        Some(BondOrder::Triple) => OrdinaryAttachmentBondOrderV1::Triple,
        _ => OrdinaryAttachmentBondOrderV1::Unsupported,
    }
}

fn within_resource_limits(molecule: &Molecule) -> Result<bool, OrdinaryAttachmentCandidateErrorV1> {
    let vertices = molecule.atoms().len() + molecule.groups().len();
    if vertices > DOCUMENT_CHEMISTRY_MAX_VERTICES_V1
        || molecule.bonds().len() > DOCUMENT_CHEMISTRY_MAX_BONDS_V1
    {
        return Ok(false);
    }
    let mut indices = HashMap::new();
    indices
        .try_reserve(vertices)
        .map_err(|_| OrdinaryAttachmentCandidateErrorV1::GraphInvalid)?;
    for atom in molecule.atoms() {
        indices.insert(VertexRef::Atom(atom.identity().clone()), indices.len());
    }
    for group in molecule.groups() {
        indices.insert(VertexRef::Group(group.identity().clone()), indices.len());
    }
    let mut connected = HashSet::new();
    connected
        .try_reserve(vertices)
        .map_err(|_| OrdinaryAttachmentCandidateErrorV1::GraphInvalid)?;
    let mut components = 0usize;
    for vertex in indices.keys() {
        if connected.contains(vertex) {
            continue;
        }
        components += 1;
        if components > DOCUMENT_CHEMISTRY_MAX_COMPONENTS_V1 {
            return Ok(false);
        }
        let mut frontier = vec![vertex.clone()];
        while let Some(current) = frontier.pop() {
            if !connected.insert(current.clone()) {
                continue;
            }
            for bond in molecule.bonds() {
                let other = if bond.start() == &current {
                    Some(bond.end())
                } else if bond.end() == &current {
                    Some(bond.start())
                } else {
                    None
                };
                if let Some(other) = other {
                    if !indices.contains_key(other) {
                        return Err(OrdinaryAttachmentCandidateErrorV1::GraphInvalid);
                    }
                    if !connected.contains(other) {
                        frontier.push(other.clone());
                    }
                }
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::{DocumentSession, TypedDocument};

    use super::*;

    fn candidate(source: &str) -> (Molecule, OrdinaryAttachmentCandidateWitnessV1) {
        let observation = DocumentSession::load(source)
            .expect("candidate source loads")
            .observe(0)
            .expect("candidate source projects");
        let molecule_id = observation.projection().molecules()[0]
            .id()
            .expect("candidate root is durable");
        let document = TypedDocument::parse(source).expect("candidate source types");
        let molecule = document
            .core_molecule(molecule_id)
            .expect("typed molecule resolves")
            .expect("typed root is a molecule");
        let witness = OrdinaryAttachmentCandidateWitnessV1::new(
            molecule.atoms()[0].identity().clone(),
            molecule.groups()[0].identity().clone(),
            molecule.bonds()[0].identity().clone(),
            OrdinaryAttachmentProfileV1::NormalSingle,
        );
        (molecule, witness)
    }

    #[test]
    fn existing_group_bond_demand_is_counted_with_the_new_witnessed_attachment() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
            "<atom id=\"anchor\" name=\"C\" explicit_hydrogens=\"2\"><point x=\"0\" y=\"0\"/></atom>",
            "<compact-group id=\"existing\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"10\" y=\"0\"/></compact-group>",
            "<compact-group id=\"added\" version=\"1\" catalog-key=\"nitro\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"-10\" y=\"0\"/></compact-group>",
            "<bond id=\"new_attachment\" start=\"anchor\" end=\"added\" type=\"n1\"/>",
            "<bond id=\"existing_attachment\" start=\"anchor\" end=\"existing\" type=\"n1\"/>",
            "</molecule></cdml>",
        );
        let (molecule, mut witness) = candidate(source);
        witness.added_group = molecule.groups()[1].identity().clone();
        witness.added_bond = molecule.bonds()[0].identity().clone();

        let admission = admit_candidate_ordinary_attachment_capacity_v1(&molecule, &witness)
            .expect("well-formed candidate evaluates");

        assert_eq!(
            admission,
            DocumentOrdinaryAttachmentAvailabilityV1::Available
        );
    }

    #[test]
    fn existing_group_bond_closes_the_saturated_carbon_attachment_boundary() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
            "<atom id=\"anchor\" name=\"C\" explicit_hydrogens=\"3\"><point x=\"0\" y=\"0\"/></atom>",
            "<compact-group id=\"existing\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"10\" y=\"0\"/></compact-group>",
            "<compact-group id=\"added\" version=\"1\" catalog-key=\"nitro\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"-10\" y=\"0\"/></compact-group>",
            "<bond id=\"new_attachment\" start=\"anchor\" end=\"added\" type=\"n1\"/>",
            "<bond id=\"existing_attachment\" start=\"anchor\" end=\"existing\" type=\"n1\"/>",
            "</molecule></cdml>",
        );
        let (molecule, mut witness) = candidate(source);
        witness.added_group = molecule.groups()[1].identity().clone();
        witness.added_bond = molecule.bonds()[0].identity().clone();

        let admission = admit_candidate_ordinary_attachment_capacity_v1(&molecule, &witness)
            .expect("well-formed candidate evaluates");

        assert_eq!(
            admission,
            DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
                reason: DocumentOrdinaryAttachmentReasonV1::CapacityUnavailable,
                recovery: DocumentOrdinaryAttachmentRecoveryV1::ChooseAnotherAtom,
            }
        );
    }

    #[test]
    fn unsupported_atom_profile_is_a_closed_source_unavailability() {
        let source = concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">",
            "<atom id=\"anchor\" name=\"C\" multiplicity=\"2\"><point x=\"0\" y=\"0\"/></atom>",
            "<compact-group id=\"added\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"10\" y=\"0\"/></compact-group>",
            "<bond id=\"new_attachment\" start=\"anchor\" end=\"added\" type=\"n1\"/>",
            "</molecule></cdml>",
        );
        let (molecule, witness) = candidate(source);

        let admission = admit_candidate_ordinary_attachment_capacity_v1(&molecule, &witness)
            .expect("well-formed candidate evaluates");

        assert_eq!(
            admission,
            DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
                reason: DocumentOrdinaryAttachmentReasonV1::SourceFactsUnsupported,
                recovery: DocumentOrdinaryAttachmentRecoveryV1::EditStructure,
            }
        );
    }

    #[test]
    fn oversized_candidate_root_returns_resource_limit_before_witness_resolution() {
        let mut atoms = String::new();
        for index in 0..=DOCUMENT_CHEMISTRY_MAX_VERTICES_V1 {
            atoms.push_str(&format!(
                "<atom id=\"atom-{index}\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>"
            ));
        }
        let source = format!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"root\">{atoms}</molecule></cdml>"
        );
        let observation = DocumentSession::load(&source)
            .expect("oversized source remains structurally valid")
            .observe(0)
            .expect("oversized source projects");
        let molecule_id = observation.projection().molecules()[0]
            .id()
            .expect("oversized root is durable");
        let document = TypedDocument::parse(&source).expect("oversized source types");
        let molecule = document
            .core_molecule(molecule_id)
            .expect("typed oversized molecule resolves")
            .expect("typed oversized root is a molecule");
        let anchor = molecule.atoms()[0].identity().clone();
        let witness = OrdinaryAttachmentCandidateWitnessV1::new(
            anchor.clone(),
            anchor.clone(),
            anchor,
            OrdinaryAttachmentProfileV1::NormalSingle,
        );

        let admission = admit_candidate_ordinary_attachment_capacity_v1(&molecule, &witness)
            .expect("bound is an availability outcome");

        assert!(matches!(
            admission,
            DocumentOrdinaryAttachmentAvailabilityV1::Unavailable {
                reason: DocumentOrdinaryAttachmentReasonV1::ResourceLimit,
                recovery: DocumentOrdinaryAttachmentRecoveryV1::ReduceRoot,
            }
        ));
    }
}
