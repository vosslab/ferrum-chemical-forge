use super::*;
use crate::{
    DocumentCompactGroupMaterializationRefusalV1, DocumentCompactGroupMaterializationRequestV1,
    DocumentMoleculeHydrogenMaterializationRefusalV1,
    DocumentMoleculeHydrogenMaterializationRequestV1, InterchangeRecordBatchInsertionV1,
    ReverseDirectedBondEndpointsV1,
};

/// Versioned session operation staging the initial supported document mutation.
#[derive(Clone, Debug, PartialEq)]
pub enum SessionOperation {
    /// The only currently supported protocol version.
    V1(SessionOperationV1),
}

/// First version of Rust-owned typed document operations.
#[derive(Clone, Debug, PartialEq)]
pub enum SessionOperationV1 {
    /// Materialize one attached typed compact group into ordinary editable chemistry.
    MaterializeCompactGroupV1(DocumentCompactGroupMaterializationRequestV1),
    /// Materialize the ordinary implicit hydrogens of one selected direct molecule.
    MaterializeMoleculeHydrogensV1(DocumentMoleculeHydrogenMaterializationRequestV1),
    /// Insert one complete frozen molecule through document-owned identity allocation.
    InsertMoleculeV1(crate::MoleculeInsertionRequestV1),

    /// Create one Haworth molecule through document-owned semantic lowering.
    CreateHaworthMoleculeV1(CreateHaworthMoleculeV1),
    /// Insert one nonempty source-ordered interchange batch atomically.
    InsertInterchangeRecordBatchV1(InterchangeRecordBatchInsertionV1),
    /// Create one closed curved terminal arrow through document-owned lowering.
    CreateCurvedTerminalArrowV1(CreateCurvedTerminalArrowV1),
    /// Create one closed curved equilibrium arrow through document-owned lowering.
    CreateCurvedEquilibriumArrowV1(CreateCurvedEquilibriumArrowV1),
    /// Create one closed presentation path through document-owned lowering.
    CreatePresentationPathV1(CreatePresentationPathV1),
    /// Create one closed presentation vector through document-owned lowering.
    CreatePresentationVectorV1(CreatePresentationVectorV1),
    /// Create one straight arrow or standard Plus through document-owned lowering.
    CreatePresentationRootV1(CreatePresentationRootV1),
    /// Place one closed catalog molecule through document-owned semantic lowering.
    PlaceCatalogMoleculeV1(CatalogMoleculePlacementV1),
    /// Create one direct bond through document-owned semantic lowering.
    CreateDirectBondV1(CreateDirectBondV1),
    /// Create one renderer-admitted atom in an existing molecule.
    CreateAtomV1(CreateAtomV1),
    /// Create one renderer-admitted bond between existing atoms.
    CreateBondV1(CreateBondV1),
    /// Create one complete strict direct reaction with a document-minted ID.
    CreateReactionV1(CreateReactionV1),
    /// Replace every recognized member of one strict direct reaction.
    ReplaceReactionMembersV1(ReplaceReactionMembersV1),
    /// Delete one strict direct reaction definition while retaining its members.
    DeleteReactionV1(DeleteReactionV1),
    /// Replace the element spelling of an existing typed atom.
    SetAtomElement { atom_id: String, element: String },
    /// Apply one validated unique-field atom-properties patch atomically.
    SetAtomProperties {
        /// Complete validated source-ID-targeted property intent.
        patch: AtomPropertiesPatchV1,
    },
    /// Assign or clear one direct atom's persistent number and visibility pair.
    SetAtomNumber {
        /// Durable authored direct-root molecule ID.
        molecule_id: String,
        /// Durable authored direct atom ID.
        atom_id: String,
        /// Positive decimal number, or `None` only for a clear.
        number: Option<u64>,
        /// Explicit visibility, or `None` only for a clear.
        show_number: Option<bool>,
    },
    /// Replace or remove one direct-root molecule's exact authored name.
    SetMoleculeName {
        /// Opaque durable direct-root molecule selector.
        molecule_id: super::super::DocumentObjectIdV1,
        /// Exact name; `None` or an empty string removes the attribute.
        name: Option<String>,
    },
    /// Add or remove one supported direct mark from one durable direct atom.
    ApplyAtomMark {
        /// Durable authored direct-root molecule ID.
        molecule_id: String,
        /// Durable authored direct atom ID.
        atom_id: String,
        /// Exact add or removal intent.
        action: AtomMarkActionV1,
        /// Closed authored mark type.
        kind: AtomMarkKindV1,
        /// Optional zero-based ordinal among direct marks of this exact type.
        matching_mark_index: Option<u32>,
    },
    /// Replace one existing typed atom's finite Cartesian point.
    SetAtomPosition {
        /// Durable authored atom ID.
        atom_id: String,
        /// Complete finite replacement point.
        position: Point3V1,
    },
    /// Rotate selected durable direct atoms around one finite scene-space center.
    RotateAtoms {
        /// Complete molecule-owned targets, center, and radian angle.
        rotation: AtomRotationV1,
    },
    /// Repair selected direct-root molecule geometry through a closed Rust planner.
    RepairGeometry {
        /// Complete selected molecule set, kind, and target spacing.
        repair: GeometryRepairV1,
    },
    /// Delete one durable atom together with every typed incident bond.
    DeleteAtom {
        /// Durable authored atom ID.
        atom_id: String,
    },
    /// Delete one durable typed bond without changing either endpoint atom.
    DeleteBond {
        /// Durable authored bond ID.
        bond_id: String,
    },
    /// Delete one exact set of direct atoms/bonds from one molecule atomically.
    DeleteStructure {
        /// Durable authored direct-root molecule ID.
        molecule_id: String,
        /// Durable authored direct atom IDs.
        atom_ids: Vec<String>,
        /// Durable authored direct bond IDs.
        bond_ids: Vec<String>,
    },
    /// Delete one supported durable direct-root presentation record.
    DeletePresentationRoot {
        /// Exact durable source ID and closed record kind.
        deletion: PresentationRootDeletionV1,
    },
    /// Delete one complete nonempty set of durable direct-root presentation records.
    DeletePresentationRoots {
        /// Exact-kind durable targets validated as one atomic set.
        deletions: PresentationRootDeletionSetV1,
    },
    /// Reorder durable direct-root presentation records without moving non-element slots.
    ReorderPresentationRoots {
        /// Complete exact-kind target set and closed ordering transformation.
        reorder: PresentationStackReorderV1,
    },
    /// Apply one direct capability-free layout transform to complete durable roots.
    ApplyTopLevelRootLayoutTransformV1(TopLevelRootLayoutTransformV1),
    /// Translate complete durable roots from an interaction-derived displacement.
    TranslateTopLevelRootsV1(TopLevelRootTranslationV1),
    /// Replace one durable typed bond's normal covalent order.
    SetBondOrder {
        /// Durable authored bond ID.
        bond_id: String,
        /// Complete replacement order.
        order: DocumentBondOrderV1,
    },
    /// Apply one validated unique-field bond-properties patch atomically.
    SetBondProperties {
        /// Complete validated source-ID-targeted property intent.
        patch: BondPropertiesPatchV1,
    },
    /// Reverse the retained endpoint direction of one directed wedge bond.
    ReverseDirectedBondEndpointsV1(ReverseDirectedBondEndpointsV1),
    /// Apply one validated unique-field direct-root Plus properties patch atomically.
    SetPlusProperties {
        /// Complete validated source-ID-targeted property intent.
        patch: PlusPropertiesPatchV1,
    },
    /// Apply one validated unique-field direct-root Text properties patch atomically.
    SetTextProperties {
        /// Complete validated source-ID-targeted property intent.
        patch: TextPropertiesPatchV1,
    },
    /// Apply one validated unique-field paper-properties patch atomically.
    SetPaperProperties {
        /// Complete document-global paper property intent.
        patch: PaperPropertiesPatchV1,
    },
    /// Apply one validated unique-field document drawing-standard patch atomically.
    SetDrawingStandard {
        /// Complete document-global drawing-default intent.
        patch: DrawingStandardPatchV1,
    },
    /// Apply one validated unique-field direct-root Arrow properties patch atomically.
    SetArrowProperties {
        /// Complete validated source-ID-targeted property intent.
        patch: ArrowPropertiesPatchV1,
    },
    /// Apply one validated appearance patch to a direct-root geometric presentation.
    SetGeometricProperties {
        /// Complete validated source-ID-targeted appearance intent.
        patch: GeometricPropertiesPatchV1,
    },
    /// Apply one validated appearance patch to a durable Wavy polyline.
    SetWavyProperties {
        /// Complete validated durable-ID-targeted Wavy appearance intent.
        patch: WavyPropertiesPatchV1,
    },
    /// Apply one validated common appearance patch to a durable bracket pair.
    SetBracketProperties {
        /// Complete validated pair-ID-targeted appearance intent.
        patch: BracketPropertiesPatchV1,
    },
    /// Replace every direct atom point in one durable molecule from one source snapshot.
    SetMoleculeAtomPositions {
        /// Complete revision- and digest-bound replacement positions.
        update: MoleculeCoordinateUpdateV1,
    },
    /// Replace every direct atom Point3 in several molecules as one transition.
    SetMoleculeAtomPositionsBatch {
        /// Complete, unique, revision- and digest-bound replacement positions.
        update: MoleculeCoordinateBatchUpdateV1,
    },
    /// Replace direct atom x/y coordinates for a prepared molecule set atomically.
    SetCleanGeometry {
        /// Complete revision-, digest-, target-, and atom-order-bound layouts.
        update: CleanGeometryUpdateV1,
    },
    /// Apply complete revision-bound whole-depiction straightening results atomically.
    ApplyPreparedStraightenDepictions {
        /// Factory-created layout results from the exact current document state.
        update: PreparedStraightenDepictionsV1,
    },
}

/// Typed operation failure before an accepted state transition.
impl SessionOperationV1 {
    /// Create one ordinary topology-only molecule insertion operation.
    #[must_use]
    pub fn insert_molecule_v1(molecule: MoleculeInsertionV1) -> Self {
        Self::InsertMoleculeV1(molecule.into())
    }
}

/// Typed operation failure before an accepted state transition.
#[derive(Debug, Error)]
pub enum SessionOperationError {
    /// Typed compact-group materialization was refused before a candidate transition.
    #[error(transparent)]
    CompactGroupMaterialization(#[from] DocumentCompactGroupMaterializationRefusalV1),
    /// Typed compact-group materialization must be prepared by the transition core.
    #[error("compact-group materialization must be prepared by the session transition core")]
    CompactGroupMaterializationRequiresTransitionCore,
    /// Explicit-hydrogen materialization was refused before a candidate transition.
    #[error(transparent)]
    HydrogenMaterialization(#[from] DocumentMoleculeHydrogenMaterializationRefusalV1),
    /// Explicit-hydrogen materialization must be prepared by the transition core.
    #[error("hydrogen materialization must be prepared by the session transition core")]
    HydrogenMaterializationRequiresTransitionCore,
    /// Complete molecule insertion must be prepared by the session transition core.
    #[error("molecule insertion must be prepared by the session transition core")]
    MoleculeInsertionRequiresTransitionCore,
    /// Interchange batch insertion must be prepared by the session transition core.
    #[error("interchange batch insertion must be prepared by the session transition core")]
    InterchangeRecordBatchInsertionRequiresTransitionCore,
    /// A closed presentation request must be prepared by the session transition core.
    #[error("presentation creation must be prepared by the session transition core")]
    PresentationCreateRequiresTransitionCore,
    /// A closed catalog request could not be lowered into an admitted document state.
    #[error("invalid catalog placement: {0}")]
    InvalidCatalogPlacement(String),
    /// A semantic direct-bond operation must be prepared by the session transition core.
    #[error(transparent)]
    DirectBond(#[from] DirectBondAdmissionRefusalV1),
    /// A semantic direct-reaction operation was refused before candidate lowering.
    #[error(transparent)]
    Reaction(#[from] ReactionOperationRefusalV1),
    /// A closed explicit-fragment request could not prove its molecule-local facts.
    #[error(transparent)]
    ExplicitFragment(#[from] DocumentExplicitFragmentErrorV1),
    /// Native linear-form conversion requires one or more exact selected atoms.
    #[error("linear-form conversion requires a nonempty exact atom selection")]
    EmptyLinearFormSelection,
    /// The native linear-form planner refused the authenticated graph facts.
    #[error("linear-form planning refused: {0}")]
    LinearFormPlan(#[source] ferrum_domain::linear_form::LinearFormPlanErrorV1),
    /// Session history could not reserve storage for a prepared transition.
    #[error("document history could not reserve storage for a prepared transition")]
    HistoryResourceExhausted,
    /// A requested element spelling is not a canonical periodic-table symbol.
    #[error("atom element must be a canonical periodic-table symbol")]
    InvalidAtomElement,
    /// Atom number assignment requires a positive value and explicit visibility;
    /// clearing requires both values absent.
    #[error(
        "atom number requires a positive integer and boolean visibility, or an empty clear pair"
    )]
    InvalidAtomNumberPair,
    /// Add intent cannot carry a same-type removal selector.
    #[error("atom mark add does not accept a matching mark index")]
    InvalidAtomMarkSelector,
    /// A Wavy gesture could not produce bounded finite persistent geometry.
    #[error("invalid Wavy insertion: {0}")]
    InvalidWavyInsertion(String),
    /// A bracket gesture could not produce finite persistent pair geometry.
    #[error("invalid bracket insertion: {0}")]
    InvalidBracketInsertion(String),
    /// A detached direct-Haworth receipt cannot enter the closed insertion flow.
    #[error("invalid direct Haworth insertion: {0}")]
    InvalidDirectHaworthInsertion(String),
    /// A closed detached regular-ring request could not form ordinary CDML facts.
    #[error("invalid regular ring insertion: {0}")]
    InvalidRegularRingInsertion(String),
    /// A closed standalone D-glucose Haworth recipe could not be authored.
    #[error("invalid standalone Haworth insertion: {0}")]
    InvalidStandaloneHaworthInsertion(String),
    /// Custom dimensions were requested for an effective named paper type.
    #[error("paper dimensions apply only to an effective custom paper type")]
    PaperDimensionsRequireCustom,
    /// The requested typed atom does not occur in the retained document.
    #[error("typed atom does not exist: {0}")]
    UnknownAtom(String),
    /// The requested direct-root typed molecule does not occur in the retained document.
    #[error("typed direct-root molecule does not exist")]
    UnknownMolecule,
    /// The requested typed bond does not occur in the retained document.
    #[error("typed bond does not exist: {0}")]
    UnknownBond(String),
    /// The requested direct-root typed Plus does not occur in the retained document.
    #[error("typed Plus does not exist: {0}")]
    UnknownPlus(String),
    /// The requested direct-root typed Text does not occur in the retained document.
    #[error("typed Text does not exist: {0}")]
    UnknownText(String),
    /// The requested durable presentation root did not match its exact record kind.
    #[error("typed presentation root does not exist: {0}")]
    UnknownPresentationRoot(String),
    /// The requested direct-root typed Arrow does not occur in the retained document.
    #[error("typed Arrow does not exist: {0}")]
    UnknownArrow(String),
    /// The requested direct-root geometric presentation does not occur in the document.
    #[error("typed geometric presentation does not exist: {0}")]
    UnknownGeometricPresentation(String),
    /// The requested direct-root Wavy polyline does not occur in the document.
    #[error("typed Wavy presentation does not exist")]
    UnknownWavy(ferrum_document_projection::DocumentObjectIdV1),
    /// The requested durable bracket members do not occur in the document.
    #[error("typed bracket pair does not exist: {0:?}")]
    UnknownBracketPair([ferrum_document_projection::DocumentObjectIdV1; 2]),
    /// A durable document-object selector does not occur in the retained document.
    #[error("document object does not exist: {0}")]
    UnknownDocumentObject(String),
    /// A durable selector names a typed record other than a molecule.
    #[error("document object is not a typed molecule: {0}")]
    InvalidCreateAtomTarget(String),
    /// A durable selector is not an atom usable by molecule-local bond creation.
    #[error("document object is not a bondable typed atom: {0}")]
    InvalidCreateBondTarget(String),
    /// A durable selector cannot satisfy a named live chemistry operation.
    #[error("document object is not a valid live chemistry target: {0}")]
    InvalidLiveChemicalTarget(String),
    /// A durable selector is not a molecule usable by coordinate regeneration.
    #[error("document object is not a coordinate-editable typed molecule: {0}")]
    InvalidMoleculeCoordinateTarget(String),
    /// Prepared coordinate facts came from a different session revision.
    #[error(
        "molecule coordinates were prepared at revision {prepared}, current revision is {current}"
    )]
    MoleculeCoordinateRevisionMismatch { prepared: u64, current: u64 },
    /// Prepared coordinate facts came from different document content.
    #[error("molecule coordinates were prepared from a different document digest")]
    MoleculeCoordinateDigestMismatch,
    /// A factory-created whole-depiction result was structurally invalid.
    #[error("invalid prepared whole-depiction straightening result: {0}")]
    InvalidStraightenDepiction(String),
    /// A bond cannot connect one atom to itself.
    #[error("a bond cannot connect atom {0} to itself")]
    CreateBondSelfLoop(String),
    /// Both bond endpoints must be direct atoms of one durable molecule.
    #[error("bond endpoints belong to different molecules")]
    CreateBondAcrossMolecules,
    /// The requested undirected atom pair is already connected.
    #[error("a bond already connects {start} and {end}")]
    CreateBondDuplicate { start: String, end: String },
    /// The session cannot issue another generated persistent atom identity.
    #[error("generated atom identifier space is exhausted")]
    AtomIdentifierExhausted,
    /// The session cannot issue another generated persistent compact-group identity.
    #[error("generated compact-group identifier space is exhausted")]
    GroupIdentifierExhausted,
    /// The session cannot issue another generated persistent molecule identity.
    #[error("generated molecule identifier space is exhausted")]
    MoleculeIdentifierExhausted,
    /// The session cannot issue another generated persistent bond identity.
    #[error("generated bond identifier space is exhausted")]
    BondIdentifierExhausted,
    /// The session cannot issue another generated presentation identity.
    #[error("generated presentation identifier space is exhausted")]
    PresentationIdentifierExhausted,
    /// The session cannot issue another generated fragment identity.
    #[error("generated fragment identifier space is exhausted")]
    FragmentIdentifierExhausted,
    /// The session cannot issue another generated imported-fragment identity.
    #[error("generated imported-fragment identifier space is exhausted")]
    FragmentImportIdentifierExhausted,
    /// Storage needed to allocate generated persistent identities was unavailable.
    #[error("generated identifier allocation failed")]
    GeneratedIdentifierAllocationFailed,
    /// Candidate construction or retained-document validation failed.
    #[error("cannot prepare document candidate: {0}")]
    Candidate(#[from] TypedDocumentError),
    /// Candidate comparison could not serialize retained CDML.
    #[error("cannot serialize document candidate: {0}")]
    Serialize(#[from] XmlSerializationError),
}

/// One detached candidate outcome.
pub(crate) enum Candidate {
    /// The requested semantic change leaves canonical content unchanged.
    NoChange,
    /// A fully validated retained tree ready for atomic acceptance.
    Changed(Box<TypedDocument>),
}

pub(crate) fn validate_reaction_members(
    members: &[(DirectReactionRoleV1, String)],
) -> Result<(), ReactionOperationRefusalV1> {
    let reactants = members
        .iter()
        .filter(|(role, _)| *role == DirectReactionRoleV1::Reactant)
        .count();
    let products = members
        .iter()
        .filter(|(role, _)| *role == DirectReactionRoleV1::Product)
        .count();
    let arrows = members
        .iter()
        .filter(|(role, _)| *role == DirectReactionRoleV1::Arrow)
        .count();
    if reactants == 0 || products == 0 || arrows != 1 {
        return Err(ReactionOperationRefusalV1::MissingRequiredMembers);
    }
    if members
        .iter()
        .any(|(_, identifier)| identifier.trim().is_empty())
    {
        return Err(ReactionOperationRefusalV1::EmptyMemberIdentifier);
    }
    let identifiers = members
        .iter()
        .map(|(_, identifier)| identifier)
        .collect::<std::collections::HashSet<_>>();
    if identifiers.len() != members.len() {
        return Err(ReactionOperationRefusalV1::DuplicateMember);
    }
    Ok(())
}

fn expected_reaction_member_kind(role: DirectReactionRoleV1) -> DirectCdmlRootKindV1 {
    match role {
        DirectReactionRoleV1::Reactant | DirectReactionRoleV1::Product => {
            DirectCdmlRootKindV1::Molecule
        }
        DirectReactionRoleV1::Arrow => DirectCdmlRootKindV1::Arrow,
        DirectReactionRoleV1::Condition => DirectCdmlRootKindV1::Text,
        DirectReactionRoleV1::Plus => DirectCdmlRootKindV1::Plus,
    }
}

pub(crate) fn validate_reaction_members_against_document(
    document: &TypedDocument,
    members: &[(DirectReactionRoleV1, String)],
    excluded_reaction: Option<&str>,
) -> Result<(), ReactionOperationRefusalV1> {
    let index = DirectCdmlSemanticIndexV1::from_document(document);
    for (role, identifier) in members {
        let root = index
            .roots()
            .iter()
            .find(|root| root.identifier() == Some(identifier.as_str()))
            .ok_or(ReactionOperationRefusalV1::MissingMember)?;
        if root.kind() != expected_reaction_member_kind(*role) {
            return Err(ReactionOperationRefusalV1::WrongMemberKind);
        }
        if index.roots().iter().any(|root| {
            root.kind() == DirectCdmlRootKindV1::Reaction
                && root.identifier() != excluded_reaction
                && root
                    .reaction_members()
                    .iter()
                    .any(|member| member == identifier)
        }) {
            return Err(ReactionOperationRefusalV1::CrossReactionReuse);
        }
    }
    Ok(())
}

pub(super) fn strict_reaction_exists(document: &TypedDocument, reaction_id: &str) -> bool {
    let Ok(source) = document.to_xml() else {
        return false;
    };
    super::super::inspect_direct_reactions_v1(&source).is_ok_and(|definitions| {
        definitions.iter().any(|definition| {
            definition.identifier() == Some(reaction_id) && definition.is_strict()
        })
    })
}

fn reaction_document_object_id(
    document: &TypedDocument,
    reaction_id: &str,
) -> Result<crate::DocumentObjectIdV1, SessionOperationError> {
    let source_id = PersistentId::new(reaction_id.to_owned())
        .map_err(|_| ReactionOperationRefusalV1::InvalidDefinition)?;
    document
        .document_object_id_for_source_id_v1(&source_id)
        .ok_or(ReactionOperationRefusalV1::InvalidDefinition.into())
}

pub(super) fn prepare_create_reaction(
    current: &TypedDocument,
    request: &CreateReactionV1,
) -> Result<(TypedDocument, crate::DocumentObjectIdV1), SessionOperationError> {
    validate_reaction_members_against_document(current, request.members(), None)?;
    let index = DirectCdmlSemanticIndexV1::from_document(current);
    let reaction_id = (1_u64..)
        .map(|number| format!("rxn-{number}"))
        .find(|identifier| !index.reserves_identifier(identifier))
        .ok_or(ReactionOperationRefusalV1::InvalidDefinition)?;
    let source = current.to_xml()?;
    let candidate = append_direct_cdml_reaction_v1(&source, &reaction_id, request.members())
        .map_err(|_| ReactionOperationRefusalV1::InvalidDefinition)?;
    let candidate = TypedDocument::parse(&candidate).map_err(SessionOperationError::Candidate)?;
    let reaction_document_object_id = reaction_document_object_id(&candidate, &reaction_id)?;
    Ok((candidate, reaction_document_object_id))
}

pub(super) fn prepare_replace_reaction_members(
    current: &TypedDocument,
    request: &ReplaceReactionMembersV1,
) -> Result<(TypedDocument, crate::DocumentObjectIdV1), SessionOperationError> {
    if !strict_reaction_exists(current, request.reaction_id()) {
        return Err(ReactionOperationRefusalV1::InvalidDefinition.into());
    }
    validate_reaction_members_against_document(
        current,
        request.members(),
        Some(request.reaction_id()),
    )?;
    let reaction_document_object_id = reaction_document_object_id(current, request.reaction_id())?;
    let source = current.to_xml()?;
    let candidate =
        replace_direct_cdml_reaction_members_v1(&source, request.reaction_id(), request.members())
            .map_err(|_| ReactionOperationRefusalV1::InvalidDefinition)?;
    let candidate = TypedDocument::parse(&candidate).map_err(SessionOperationError::Candidate)?;
    Ok((candidate, reaction_document_object_id))
}

pub(super) fn prepare_delete_reaction(
    current: &TypedDocument,
    request: &DeleteReactionV1,
) -> Result<(TypedDocument, crate::DocumentObjectIdV1), SessionOperationError> {
    if !strict_reaction_exists(current, request.reaction_id()) {
        return Err(ReactionOperationRefusalV1::InvalidDefinition.into());
    }
    let reaction_document_object_id = reaction_document_object_id(current, request.reaction_id())?;
    let source = current.to_xml()?;
    let candidate = delete_direct_cdml_reaction_definition_v1(&source, request.reaction_id())
        .map_err(|_| ReactionOperationRefusalV1::InvalidDefinition)?;
    let candidate = TypedDocument::parse(&candidate).map_err(SessionOperationError::Candidate)?;
    Ok((candidate, reaction_document_object_id))
}
