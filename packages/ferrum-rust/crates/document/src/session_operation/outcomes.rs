use super::*;
use crate::{
    DocumentCompactGroupMaterializationResultV1, DocumentMoleculeHydrogenMaterializationResultV1,
    DocumentObjectIdV1,
};

/// Immutable result of one accepted session mutation or history transition.
///
/// The enclosed observation is created after the authoritative state transition.
/// Frontends must derive all follow-on projection and render facts from this one
/// revision- and digest-bound value rather than re-reading separate session views.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionOperationResultV1 {
    observation: SessionDocumentObservationV1,
    outcome: SessionOperationOutcomeV1,
}

impl SessionOperationResultV1 {
    pub(crate) fn new(observation: SessionDocumentObservationV1) -> Self {
        Self {
            observation,
            outcome: SessionOperationOutcomeV1::Standard,
        }
    }

    pub(crate) fn with_outcome(mut self, outcome: SessionOperationOutcomeV1) -> Self {
        self.outcome = outcome;
        self
    }

    /// Return the complete post-operation observation.
    #[must_use]
    pub fn observation(&self) -> &SessionDocumentObservationV1 {
        &self.observation
    }

    #[must_use]
    pub const fn outcome(&self) -> &SessionOperationOutcomeV1 {
        &self.outcome
    }
}

/// Semantic facts that become authoritative only after a session operation commits.
#[derive(Clone, Debug, PartialEq)]
pub enum SessionOperationOutcomeV1 {
    /// Existing operations have no operation-specific post-commit facts.
    Standard,
    /// Direct-bond facts produced by a successful direct-bond operation.
    DirectBondV1(DirectBondOperationOutcomeV1),
    /// Identity allocated by a committed generic atom creation.
    AtomCreatedV1(AtomCreatedOutcomeV1),
    /// Identity allocated by a committed generic bond creation.
    BondCreatedV1(BondCreatedOutcomeV1),
    /// Explicit-hydrogen facts produced by a successful materialization operation.
    MoleculeHydrogensMaterializedV1(DocumentMoleculeHydrogenMaterializationResultV1),
    /// Typed compact-group facts produced by a successful materialization operation.
    CompactGroupMaterializedV1(DocumentCompactGroupMaterializationResultV1),
    /// Identity facts produced by a successful complete molecule insertion.
    MoleculeInsertedV1(MoleculeInsertedOutcomeV1),
    /// Source-ordered identity facts produced by a successful interchange batch insertion.
    InterchangeRecordBatchInsertedV1(InterchangeRecordBatchInsertedOutcomeV1),
    /// Catalog placement facts produced by a successful catalog operation.
    CatalogMoleculePlacementV1(CatalogMoleculePlacementOutcomeV1),
    /// Presentation-root facts produced by a successful semantic authoring operation.
    CreatedPresentationRootV1(CreatedPresentationRootOutcomeV1),
    /// Reaction identity allocated by a successful generic reaction creation.
    ReactionCreatedV1(ReactionCreatedOutcomeV1),
    /// Reaction identity retained by a successful generic membership replacement.
    ReactionMembershipReplacedV1(ReactionMembershipReplacedOutcomeV1),
    /// Reaction identity removed by a successful generic definition deletion.
    ReactionDefinitionDeletedV1(ReactionDefinitionDeletedOutcomeV1),
}

/// Complete semantic request for one atom authoring operation.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateAtomV1 {
    molecule: DocumentObjectIdV1,
    element: String,
    position: Point3V1,
}

impl CreateAtomV1 {
    #[must_use]
    pub const fn new(molecule: DocumentObjectIdV1, element: String, position: Point3V1) -> Self {
        Self {
            molecule,
            element,
            position,
        }
    }
    #[must_use]
    pub const fn molecule(&self) -> &DocumentObjectIdV1 {
        &self.molecule
    }
    #[must_use]
    pub fn element(&self) -> &str {
        &self.element
    }
    #[must_use]
    pub const fn position(&self) -> Point3V1 {
        self.position
    }
}

/// Complete semantic request for one bond authoring operation.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateBondV1 {
    start_atom: DocumentObjectIdV1,
    end_atom: DocumentObjectIdV1,
    presentation: DocumentBondPresentationV1,
}

impl CreateBondV1 {
    #[must_use]
    pub const fn new(
        start_atom: DocumentObjectIdV1,
        end_atom: DocumentObjectIdV1,
        presentation: DocumentBondPresentationV1,
    ) -> Self {
        Self {
            start_atom,
            end_atom,
            presentation,
        }
    }
    #[must_use]
    pub const fn start_atom(&self) -> &DocumentObjectIdV1 {
        &self.start_atom
    }
    #[must_use]
    pub const fn end_atom(&self) -> &DocumentObjectIdV1 {
        &self.end_atom
    }
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.presentation
    }
}

/// Durable atom identity that becomes authoritative only after generic commit.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomCreatedOutcomeV1 {
    atom_identifier: PersistentId,
}
impl AtomCreatedOutcomeV1 {
    pub(crate) const fn new(atom_identifier: PersistentId) -> Self {
        Self { atom_identifier }
    }
    #[must_use]
    pub const fn atom_identifier(&self) -> &PersistentId {
        &self.atom_identifier
    }
}

/// Durable bond identity that becomes authoritative only after generic commit.
#[derive(Clone, Debug, PartialEq)]
pub struct BondCreatedOutcomeV1 {
    bond_identifier: PersistentId,
}
impl BondCreatedOutcomeV1 {
    pub(crate) const fn new(bond_identifier: PersistentId) -> Self {
        Self { bond_identifier }
    }
    #[must_use]
    pub const fn bond_identifier(&self) -> &PersistentId {
        &self.bond_identifier
    }
}

/// Durable identifiers created by one committed complete molecule insertion.
#[derive(Clone, Debug, PartialEq)]
pub struct MoleculeInsertedOutcomeV1 {
    molecule_identifier: PersistentId,
    atom_identifiers: Vec<PersistentId>,
    bond_identifiers: Vec<PersistentId>,
}

impl MoleculeInsertedOutcomeV1 {
    pub(crate) fn new(
        molecule_identifier: PersistentId,
        atom_identifiers: Vec<PersistentId>,
        bond_identifiers: Vec<PersistentId>,
    ) -> Self {
        Self {
            molecule_identifier,
            atom_identifiers,
            bond_identifiers,
        }
    }

    #[must_use]
    pub const fn molecule_identifier(&self) -> &PersistentId {
        &self.molecule_identifier
    }

    #[must_use]
    pub fn atom_identifiers(&self) -> &[PersistentId] {
        &self.atom_identifiers
    }

    #[must_use]
    pub fn bond_identifiers(&self) -> &[PersistentId] {
        &self.bond_identifiers
    }
}

/// Source-ordered committed outcomes for one atomic interchange batch.
#[derive(Clone, Debug, PartialEq)]
pub struct InterchangeRecordBatchInsertedOutcomeV1 {
    records: Vec<MoleculeInsertedOutcomeV1>,
}

impl InterchangeRecordBatchInsertedOutcomeV1 {
    pub(crate) fn new(records: Vec<MoleculeInsertedOutcomeV1>) -> Self {
        Self { records }
    }

    #[must_use]
    pub fn records(&self) -> &[MoleculeInsertedOutcomeV1] {
        &self.records
    }
}

/// Durable reaction identity that becomes authoritative only after generic commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionCreatedOutcomeV1 {
    reaction_id: String,
}

impl ReactionCreatedOutcomeV1 {
    pub(crate) const fn new(reaction_id: String) -> Self {
        Self { reaction_id }
    }

    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
}

/// Durable reaction identity whose members were replaced by generic commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionMembershipReplacedOutcomeV1 {
    reaction_id: String,
}

impl ReactionMembershipReplacedOutcomeV1 {
    pub(crate) const fn new(reaction_id: String) -> Self {
        Self { reaction_id }
    }

    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
}

/// Durable reaction identity whose definition was removed by generic commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReactionDefinitionDeletedOutcomeV1 {
    reaction_id: String,
}

impl ReactionDefinitionDeletedOutcomeV1 {
    pub(crate) const fn new(reaction_id: String) -> Self {
        Self { reaction_id }
    }

    #[must_use]
    pub fn reaction_id(&self) -> &str {
        &self.reaction_id
    }
}

/// Private reaction facts supplied to generic transition staging.
///
/// The session owns the complete staging envelope because it also carries
/// session-private deferred effects. This narrow operation-owned value carries
/// only the reaction fact that becomes public after generic redemption.
#[derive(Debug)]
pub(crate) enum ReactionOperationOutcomeStagingV1 {
    ReactionCreatedV1(String),
    ReactionMembershipReplacedV1(String),
    ReactionDefinitionDeletedV1(String),
}

/// Closed semantic class for a newly committed presentation root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreatedPresentationRootKindV1 {
    StraightNormalArrow,
    StraightEquilibriumArrow,
    Plus,
    CurvedTerminalArrow,
    CurvedEquilibriumArrow,
    Path,
    Vector,
}

/// Durable presentation selector that becomes authoritative only after commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatedPresentationRootOutcomeV1 {
    root: crate::PresentationRootSelectorV1,
    kind: CreatedPresentationRootKindV1,
}

impl CreatedPresentationRootOutcomeV1 {
    pub(crate) const fn new(
        root: crate::PresentationRootSelectorV1,
        kind: CreatedPresentationRootKindV1,
    ) -> Self {
        Self { root, kind }
    }

    #[must_use]
    pub const fn root(&self) -> &crate::PresentationRootSelectorV1 {
        &self.root
    }

    #[must_use]
    pub const fn kind(&self) -> CreatedPresentationRootKindV1 {
        self.kind
    }
}

/// Stable catalog key preserved as semantic user intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogPlacementKeyV1(String);

impl CatalogPlacementKeyV1 {
    pub fn new(value: String) -> Result<Self, SessionOperationError> {
        if value.trim().is_empty() {
            return Err(SessionOperationError::InvalidCatalogPlacement(
                "catalog key must be nonempty".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Closed catalog content lowered by the document session.
#[derive(Clone, Debug, PartialEq)]
pub enum CatalogMoleculePlacementContentV1 {
    Molecule(MoleculeInsertionV1),
    StandaloneHaworth(StandaloneDGlucoseHaworthRecipeV1),
}

/// Semantic intent to place one closed catalog entry at one finite anchor.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogMoleculePlacementV1 {
    catalog_key: CatalogPlacementKeyV1,
    anchor: super::super::PresentationGesturePoint2V1,
    content: CatalogMoleculePlacementContentV1,
}

impl CatalogMoleculePlacementV1 {
    #[must_use]
    pub const fn new(
        catalog_key: CatalogPlacementKeyV1,
        anchor: super::super::PresentationGesturePoint2V1,
        content: CatalogMoleculePlacementContentV1,
    ) -> Self {
        Self {
            catalog_key,
            anchor,
            content,
        }
    }

    #[must_use]
    pub const fn catalog_key(&self) -> &CatalogPlacementKeyV1 {
        &self.catalog_key
    }
    #[must_use]
    pub const fn anchor(&self) -> super::super::PresentationGesturePoint2V1 {
        self.anchor
    }
    #[must_use]
    pub const fn content(&self) -> &CatalogMoleculePlacementContentV1 {
        &self.content
    }
}

/// Authoritative catalog facts returned only after generic commit.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogMoleculePlacementOutcomeV1 {
    catalog_key: CatalogPlacementKeyV1,
    anchor: super::super::PresentationGesturePoint2V1,
    root_identifier: PersistentId,
}

impl CatalogMoleculePlacementOutcomeV1 {
    pub(crate) fn new(
        catalog_key: CatalogPlacementKeyV1,
        anchor: super::super::PresentationGesturePoint2V1,
        root_identifier: PersistentId,
    ) -> Self {
        Self {
            catalog_key,
            anchor,
            root_identifier,
        }
    }
    #[must_use]
    pub const fn catalog_key(&self) -> &CatalogPlacementKeyV1 {
        &self.catalog_key
    }
    #[must_use]
    pub const fn anchor(&self) -> super::super::PresentationGesturePoint2V1 {
        self.anchor
    }
    #[must_use]
    pub const fn root_identifier(&self) -> &PersistentId {
        &self.root_identifier
    }
}

/// Authoritative facts from one committed direct-bond operation.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectBondOperationOutcomeV1 {
    bond: PersistentId,
    end_atom: PersistentId,
    second_created_atom: Option<PersistentId>,
    created_new_atom: bool,
    created_new_molecule: bool,
}

impl DirectBondOperationOutcomeV1 {
    pub(crate) fn new(
        bond: PersistentId,
        end_atom: PersistentId,
        second_created_atom: Option<PersistentId>,
        created_new_atom: bool,
        created_new_molecule: bool,
    ) -> Self {
        Self {
            bond,
            end_atom,
            second_created_atom,
            created_new_atom,
            created_new_molecule,
        }
    }

    #[must_use]
    pub fn bond(&self) -> &PersistentId {
        &self.bond
    }
    #[must_use]
    pub fn end_atom(&self) -> &PersistentId {
        &self.end_atom
    }
    #[must_use]
    pub fn second_created_atom(&self) -> Option<&PersistentId> {
        self.second_created_atom.as_ref()
    }
    #[must_use]
    pub const fn created_new_atom(&self) -> bool {
        self.created_new_atom
    }
    #[must_use]
    pub const fn created_new_molecule(&self) -> bool {
        self.created_new_molecule
    }
}

/// Semantic direct-bond request accepted by the generic session operation protocol.
#[derive(Clone, Debug, PartialEq)]
pub struct CreateDirectBondV1 {
    fence: DocumentFenceV1,
    start: DirectBondEndpointIntent,
    end: DirectBondEndpointIntent,
    presentation: DocumentBondPresentationV1,
    new_atom_element: String,
    snap: DirectBondSnapPolicyV1,
}

impl CreateDirectBondV1 {
    pub fn new(
        fence: DocumentFenceV1,
        start: DirectBondEndpointIntent,
        end: DirectBondEndpointIntent,
        presentation: DocumentBondPresentationV1,
        new_atom_element: String,
        snap: DirectBondSnapPolicyV1,
    ) -> Result<Self, DirectBondAdmissionRefusalV1> {
        if new_atom_element.trim().is_empty() {
            return Err(DirectBondAdmissionRefusalV1::InvalidEndpointInput);
        }
        Ok(Self {
            fence,
            start,
            end,
            presentation,
            new_atom_element,
            snap,
        })
    }

    #[must_use]
    pub const fn fence(&self) -> DocumentFenceV1 {
        self.fence
    }
    #[must_use]
    pub const fn start(&self) -> &DirectBondEndpointIntent {
        &self.start
    }
    #[must_use]
    pub const fn end(&self) -> &DirectBondEndpointIntent {
        &self.end
    }
    #[must_use]
    pub const fn presentation(&self) -> DocumentBondPresentationV1 {
        self.presentation
    }
    #[must_use]
    pub fn new_atom_element(&self) -> &str {
        &self.new_atom_element
    }
    #[must_use]
    pub const fn snap(&self) -> DirectBondSnapPolicyV1 {
        self.snap
    }
}
