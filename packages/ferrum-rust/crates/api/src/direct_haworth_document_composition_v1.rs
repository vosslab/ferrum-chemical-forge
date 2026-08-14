//! Authentication boundary for composing one committed direct Haworth insertion.

use std::collections::HashSet;

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document::{
    CommittedDirectHaworthResultV1, DocumentDirectHaworthBondRoleV1,
    DocumentDirectHaworthBondTokenV1, DocumentHaworthPositionV1, PersistentId,
    ReobservedDirectHaworthBondFactV1,
};
use ferrum_domain::haworth::{
    AuthoredDirectGlycosidicHaworthBondRoleV1, DirectGlycosidicHaworthBondStyleV1,
    DirectGlycosidicHaworthPositionV1,
};
use ferrum_render::{
    AuthoredDirectGlycosidicHaworthRenderRequestV1, DocumentBondReplacementErrorV1,
    DocumentRenderCompositeV1, DocumentRenderContentV1, DocumentRenderIdentityV1,
    DocumentRenderOutcomeV1, RenderError, RenderTarget, compose_document_bond_replacement_v1,
    lower_authored_direct_glycosidic_haworth_v1,
};
use thiserror::Error;

use crate::{
    DepictionIssueV1, DocumentRenderPlanCompositionError, RenderObservationError,
    compose_document_render_plan_v1,
    depiction_profile_v1::resolve_direct_glycosidic_haworth_style_v1,
    render_observation_v1::derive_render_observation_from_accepted_operation_v1,
};

/// Failure while authenticating and composing one direct Haworth durable profile.
#[derive(Debug, Error)]
pub enum DirectHaworthDocumentCompositionErrorV1 {
    /// The inseparable operation and committed receipt name different revisions.
    #[error("committed direct Haworth receipt and operation provenance differ")]
    ReceiptOperationProvenanceMismatch,
    /// The selected molecule root cannot be authenticated in the accepted document.
    #[error("direct Haworth molecule root cannot be authenticated")]
    AuthenticatedRootMismatch,
    /// The durable profile facts and its retained authored depiction differ.
    #[error("direct Haworth durable facts and authored depiction differ")]
    DurableProfileDepictionMismatch,
    /// An authored direct bond does not resolve to one exact molecule target.
    #[error("direct Haworth bond targets do not match the render plan")]
    SelectedBondTargetMismatch,
    /// The accepted observation did not retain one internally consistent provenance.
    #[error("direct Haworth accepted observation provenance differs")]
    ObservationProvenanceMismatch,
    /// Closed observation lowering failed normally.
    #[error(transparent)]
    Observation(#[from] RenderObservationError),
    /// Ordinary whole-document plan composition failed normally.
    #[error(transparent)]
    Composition(#[from] DocumentRenderPlanCompositionError),
    /// Closed direct-Haworth style resolution rejected an accepted presentation fact.
    #[error("direct Haworth depiction style is invalid: {0:?}")]
    Depiction(DepictionIssueV1),
    /// Authored direct-plan lowering failed normally.
    #[error(transparent)]
    Render(#[from] RenderError),
    /// Renderer-owned replacement validation rejected the authenticated inputs.
    #[error(transparent)]
    Replacement(#[from] DocumentBondReplacementErrorV1),
}

pub(crate) trait DirectHaworthBondFactV1 {
    fn bond_identifier(&self) -> &PersistentId;
    fn endpoints(&self) -> &[PersistentId; 2];
    fn token(&self) -> DocumentDirectHaworthBondTokenV1;
    fn haworth_position(&self) -> Option<DocumentHaworthPositionV1>;
    fn role(&self) -> DocumentDirectHaworthBondRoleV1;
}

impl DirectHaworthBondFactV1 for ferrum_document::CommittedDirectHaworthBondFactV1 {
    fn bond_identifier(&self) -> &PersistentId {
        self.bond_identifier()
    }
    fn endpoints(&self) -> &[PersistentId; 2] {
        self.endpoints()
    }
    fn token(&self) -> DocumentDirectHaworthBondTokenV1 {
        self.token()
    }
    fn haworth_position(&self) -> Option<DocumentHaworthPositionV1> {
        self.haworth_position()
    }
    fn role(&self) -> DocumentDirectHaworthBondRoleV1 {
        self.role()
    }
}

impl DirectHaworthBondFactV1 for ReobservedDirectHaworthBondFactV1 {
    fn bond_identifier(&self) -> &PersistentId {
        self.bond_identifier()
    }
    fn endpoints(&self) -> &[PersistentId; 2] {
        self.endpoints()
    }
    fn token(&self) -> DocumentDirectHaworthBondTokenV1 {
        self.token()
    }
    fn haworth_position(&self) -> Option<DocumentHaworthPositionV1> {
        self.haworth_position()
    }
    fn role(&self) -> DocumentDirectHaworthBondRoleV1 {
        self.role()
    }
}

/// Compose an opaque selective replacement from one committed direct Haworth result.
///
/// The operation's immutable accepted observation and the committed receipt are the
/// whole authority for this route.  No session, document text, selector, or output
/// backend is accepted here.
pub fn compose_committed_direct_haworth_document_v1(
    committed: &CommittedDirectHaworthResultV1,
) -> Result<DocumentRenderCompositeV1, DirectHaworthDocumentCompositionErrorV1> {
    let operation = committed.operation().observation();
    let receipt = committed.receipt();
    if operation.snapshot().revision() != receipt.revision()
        || operation.snapshot().digest() != receipt.digest()
    {
        return Err(DirectHaworthDocumentCompositionErrorV1::ReceiptOperationProvenanceMismatch);
    }

    if receipt
        .bond_identifiers()
        .iter()
        .zip(receipt.bond_facts())
        .any(|(identifier, fact)| identifier != fact.bond_identifier())
    {
        return Err(DirectHaworthDocumentCompositionErrorV1::DurableProfileDepictionMismatch);
    }
    let (root, root_order) = authenticated_projection_root(operation, receipt)?;
    compose_authenticated_direct_haworth_document_v1(
        operation,
        &root,
        root_order,
        receipt.atom_identifiers(),
        receipt.bond_facts(),
        receipt.authored_depiction(),
    )
}

pub(crate) fn compose_authenticated_direct_haworth_document_v1<B: DirectHaworthBondFactV1>(
    operation: &ferrum_document::SessionDocumentObservationV1,
    root: &ferrum_document::DocumentObjectIdV1,
    root_order: u32,
    atom_identifiers: &[PersistentId],
    bond_facts: &[B],
    depiction: &ferrum_domain::haworth::AuthoredDirectGlycosidicHaworthDepictionV1,
) -> Result<DocumentRenderCompositeV1, DirectHaworthDocumentCompositionErrorV1> {
    let observation = derive_render_observation_from_accepted_operation_v1(operation)?;
    if observation.document().snapshot().revision() != operation.snapshot().revision()
        || observation.document().snapshot().digest() != operation.snapshot().digest()
    {
        return Err(DirectHaworthDocumentCompositionErrorV1::ObservationProvenanceMismatch);
    }
    let established = compose_document_render_plan_v1(&observation)?;
    if established.provenance().revision().get() != operation.snapshot().revision()
        || established.provenance().digest() != *operation.snapshot().digest()
    {
        return Err(DirectHaworthDocumentCompositionErrorV1::ObservationProvenanceMismatch);
    }

    let identity = DocumentRenderIdentityV1::durable(root.as_str())?;
    let molecule = authenticated_molecule_plan(&established, &identity, root_order)?;
    authenticate_receipt(depiction, atom_identifiers, bond_facts)?;
    let selected = resolve_selected_bonds(molecule, depiction)?;
    let style =
        resolve_direct_glycosidic_haworth_style_v1(operation.projection(), &observation.profile())
            .map_err(DirectHaworthDocumentCompositionErrorV1::Depiction)?;

    let request = AuthoredDirectGlycosidicHaworthRenderRequestV1::new(
        established.provenance(),
        depiction,
        style.paint(),
        style.line_width(),
        style.wedge_width(),
    );
    let direct = lower_authored_direct_glycosidic_haworth_v1(request)?;
    compose_document_bond_replacement_v1(established, identity, root_order, selected, direct)
        .map_err(Into::into)
}

fn authenticated_projection_root(
    operation: &ferrum_document::SessionDocumentObservationV1,
    receipt: &ferrum_document::CommittedDirectHaworthV1,
) -> Result<(ferrum_document::DocumentObjectIdV1, u32), DirectHaworthDocumentCompositionErrorV1> {
    let mut roots = operation
        .projection()
        .molecules()
        .iter()
        .filter(|molecule| molecule.source_id() == Some(receipt.molecule_identifier().as_str()));
    let Some(root) = roots.next() else {
        return Err(DirectHaworthDocumentCompositionErrorV1::AuthenticatedRootMismatch);
    };
    if roots.next().is_some() {
        return Err(DirectHaworthDocumentCompositionErrorV1::AuthenticatedRootMismatch);
    }
    let Some(identity) = root.id() else {
        return Err(DirectHaworthDocumentCompositionErrorV1::AuthenticatedRootMismatch);
    };
    Ok((identity.clone(), root.source_order()))
}

fn authenticated_molecule_plan<'a>(
    established: &'a ferrum_render::DocumentRenderPlanV1,
    identity: &ferrum_render::DocumentRenderIdentityV1,
    root_order: u32,
) -> Result<&'a ferrum_render::MoleculeRenderPlan, DirectHaworthDocumentCompositionErrorV1> {
    let mut roots = established
        .outcomes()
        .iter()
        .filter_map(|outcome| match outcome {
            DocumentRenderOutcomeV1::Root(root) if root.identity() == identity => Some(root),
            _ => None,
        });
    let Some(root) = roots.next() else {
        return Err(DirectHaworthDocumentCompositionErrorV1::AuthenticatedRootMismatch);
    };
    if roots.next().is_some() || root.source_order() != root_order {
        return Err(DirectHaworthDocumentCompositionErrorV1::AuthenticatedRootMismatch);
    }
    let DocumentRenderContentV1::Molecule(molecule) = root.content() else {
        return Err(DirectHaworthDocumentCompositionErrorV1::AuthenticatedRootMismatch);
    };
    Ok(molecule)
}

fn authenticate_receipt<B: DirectHaworthBondFactV1>(
    depiction: &ferrum_domain::haworth::AuthoredDirectGlycosidicHaworthDepictionV1,
    atom_identifiers: &[PersistentId],
    bond_facts: &[B],
) -> Result<(), DirectHaworthDocumentCompositionErrorV1> {
    if atom_identifiers.len() != depiction.canonical_atoms().len()
        || bond_facts.len() != depiction.canonical_bonds().len()
    {
        return Err(DirectHaworthDocumentCompositionErrorV1::DurableProfileDepictionMismatch);
    }
    for (identifier, atom) in atom_identifiers.iter().zip(depiction.canonical_atoms()) {
        if record(identifier, RecordKind::Atom) != Some(atom.atom().clone()) {
            return Err(DirectHaworthDocumentCompositionErrorV1::DurableProfileDepictionMismatch);
        }
    }
    for (fact, bond) in bond_facts.iter().zip(depiction.canonical_bonds()) {
        let endpoints = [
            record(&fact.endpoints()[0], RecordKind::Atom),
            record(&fact.endpoints()[1], RecordKind::Atom),
        ];
        if record(fact.bond_identifier(), RecordKind::Bond) != Some(bond.bond().clone())
            || endpoints
                != [
                    Some(bond.endpoints()[0].clone()),
                    Some(bond.endpoints()[1].clone()),
                ]
            || !same_role(fact.role(), bond.role())
            || !same_token(fact.token(), bond.token())
            || !same_position(fact.haworth_position(), bond.haworth_position())
        {
            return Err(DirectHaworthDocumentCompositionErrorV1::DurableProfileDepictionMismatch);
        }
    }
    Ok(())
}

fn resolve_selected_bonds(
    molecule: &ferrum_render::MoleculeRenderPlan,
    depiction: &ferrum_domain::haworth::AuthoredDirectGlycosidicHaworthDepictionV1,
) -> Result<Vec<RenderTarget>, DirectHaworthDocumentCompositionErrorV1> {
    let bonds = depiction.canonical_bonds();
    let mut selected = Vec::new();
    selected
        .try_reserve(bonds.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    let mut keys = HashSet::new();
    keys.try_reserve(bonds.len())
        .map_err(|_| RenderError::ResourceExhausted)?;
    for bond in bonds {
        let mut targets = molecule
            .batches()
            .iter()
            .map(|batch| batch.target())
            .chain(molecule.issues().iter().map(|issue| issue.target()))
            .filter(|target| target.record_id() == bond.bond());
        let Some(target) = targets.next() else {
            return Err(DirectHaworthDocumentCompositionErrorV1::SelectedBondTargetMismatch);
        };
        if targets.next().is_some()
            || target.record_id().kind() != RecordKind::Bond
            || target.source_order() != bond.authored_child_order()
            || !keys.insert((target.record_id().clone(), target.source_order()))
        {
            return Err(DirectHaworthDocumentCompositionErrorV1::SelectedBondTargetMismatch);
        }
        selected.push(target.clone());
    }
    Ok(selected)
}

fn record(identifier: &PersistentId, kind: RecordKind) -> Option<RecordId> {
    Identifier::new(identifier.as_str().to_owned())
        .ok()
        .map(|identifier| RecordId::from_source(kind, &identifier))
}

const fn same_role(
    document: DocumentDirectHaworthBondRoleV1,
    authored: AuthoredDirectGlycosidicHaworthBondRoleV1,
) -> bool {
    matches!(
        (document, authored),
        (
            DocumentDirectHaworthBondRoleV1::Ring,
            AuthoredDirectGlycosidicHaworthBondRoleV1::Ring
        ) | (
            DocumentDirectHaworthBondRoleV1::Bridge,
            AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge
        )
    )
}

const fn same_token(
    document: DocumentDirectHaworthBondTokenV1,
    authored: DirectGlycosidicHaworthBondStyleV1,
) -> bool {
    matches!(
        (document, authored),
        (
            DocumentDirectHaworthBondTokenV1::Q1,
            DirectGlycosidicHaworthBondStyleV1::Q1
        ) | (
            DocumentDirectHaworthBondTokenV1::W1,
            DirectGlycosidicHaworthBondStyleV1::W1
        ) | (
            DocumentDirectHaworthBondTokenV1::N1,
            DirectGlycosidicHaworthBondStyleV1::N1
        )
    )
}

const fn same_position(
    document: Option<DocumentHaworthPositionV1>,
    authored: Option<DirectGlycosidicHaworthPositionV1>,
) -> bool {
    matches!(
        (document, authored),
        (None, None)
            | (
                Some(DocumentHaworthPositionV1::Front),
                Some(DirectGlycosidicHaworthPositionV1::Front)
            )
            | (
                Some(DocumentHaworthPositionV1::Back),
                Some(DirectGlycosidicHaworthPositionV1::Back)
            )
    )
}
