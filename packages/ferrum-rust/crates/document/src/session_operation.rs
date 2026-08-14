//! Narrow, typed operations accepted by the document transaction session.

use thiserror::Error;

use super::{
    ArrowPropertiesPatchV1, AtomMarkActionV1, AtomMarkKindV1, AtomPropertiesPatchV1,
    AtomRotationV1, BondPropertiesPatchV1, BracketPropertiesPatchV1, CleanGeometryUpdateV1,
    DocumentBondOrderV1, DrawingStandardPatchV1, GeometricPropertiesPatchV1, GeometryRepairV1,
    MoleculeCoordinateUpdateV1, PaperPropertiesPatchV1, PaperPropertyChangeV1, PersistentId,
    PlusPropertiesPatchV1, Point3V1, PreparedStraightenDepictionsV1, PresentationRootDeletionSetV1,
    PresentationRootDeletionV1, PresentationStackReorderV1, SessionDocumentObservationV1,
    TextPropertiesPatchV1, TopLevelTransformV1, TypedClass, TypedDocument, TypedDocumentError,
    WavyPropertiesPatchV1, XmlSerializationError, atom_properties_patch_v1::valid_atom_element,
};

/// Immutable result of one accepted session mutation or history transition.
///
/// The enclosed observation is created after the authoritative state transition.
/// Frontends must derive all follow-on projection and render facts from this one
/// revision- and digest-bound value rather than re-reading separate session views.
#[derive(Clone, Debug, PartialEq)]
pub struct SessionOperationResultV1 {
    observation: SessionDocumentObservationV1,
}

impl SessionOperationResultV1 {
    pub(super) fn new(observation: SessionDocumentObservationV1) -> Self {
        Self { observation }
    }

    /// Return the complete post-operation observation.
    #[must_use]
    pub fn observation(&self) -> &SessionDocumentObservationV1 {
        &self.observation
    }
}

/// Versioned session operation staging the initial supported document mutation.
#[derive(Clone, Debug, PartialEq)]
pub enum SessionOperation {
    /// The only currently supported protocol version.
    V1(SessionOperationV1),
}

/// First version of Rust-owned typed document operations.
#[derive(Clone, Debug, PartialEq)]
pub enum SessionOperationV1 {
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
        molecule_id: super::DocumentObjectIdV1,
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
    /// Transform complete durable direct-root objects.
    TransformTopLevelRoots {
        /// Complete exact-kind target set and closed transform.
        transform: TopLevelTransformV1,
    },
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
    /// Apply one validated appearance patch to a direct-root Wavy polyline.
    SetWavyProperties {
        /// Complete validated source-ID-targeted Wavy appearance intent.
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
#[derive(Debug, Error)]
pub enum SessionOperationError {
    /// Native linear-form conversion requires one or more exact selected atoms.
    #[error("linear-form conversion requires a nonempty exact atom selection")]
    EmptyLinearFormSelection,
    /// The native linear-form planner refused the authenticated graph facts.
    #[error("linear-form planning refused: {0}")]
    LinearFormPlan(#[source] ferrum_domain::linear_form::LinearFormPlanErrorV1),
    /// Session history could not reserve storage for a prepared transition.
    #[error("document history could not reserve storage for a prepared transition")]
    HistoryResourceExhausted,
    /// A requested element spelling is empty or has invalid XML-like content.
    #[error("atom element must be a nonblank plain element spelling")]
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
    #[error("typed Wavy presentation does not exist: {0}")]
    UnknownWavy(String),
    /// The requested durable bracket pair does not occur in the document.
    #[error("typed bracket pair does not exist: {0}")]
    UnknownBracketPair(String),
    /// A durable document-object selector does not occur in the retained document.
    #[error("document object does not exist: {0}")]
    UnknownDocumentObject(String),
    /// A durable selector names a typed record other than a molecule.
    #[error("document object is not a typed molecule: {0}")]
    InvalidCreateAtomTarget(String),
    /// A durable selector is not an atom usable by molecule-local bond creation.
    #[error("document object is not a bondable typed atom: {0}")]
    InvalidCreateBondTarget(String),
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
pub(super) enum Candidate {
    /// The requested semantic change leaves canonical content unchanged.
    NoChange,
    /// A fully validated retained tree ready for atomic acceptance.
    Changed(Box<TypedDocument>),
}

impl SessionOperation {
    pub(super) fn prepare(
        &self,
        current: &TypedDocument,
        current_revision: u64,
        current_digest: &[u8; 32],
    ) -> Result<Candidate, SessionOperationError> {
        match self {
            Self::V1(SessionOperationV1::SetAtomElement { atom_id, element }) => {
                if !valid_atom_element(element) {
                    return Err(SessionOperationError::InvalidAtomElement);
                }
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_element(&identifier, element)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetAtomProperties { patch }) => {
                let candidate = current.with_atom_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownAtom(patch.atom_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetAtomNumber {
                molecule_id,
                atom_id,
                number,
                show_number,
            }) => {
                let valid_pair = matches!((number, show_number), (Some(value), Some(_)) if *value > 0)
                    || matches!((number, show_number), (None, None));
                if !valid_pair {
                    return Err(SessionOperationError::InvalidAtomNumberPair);
                }
                let molecule = PersistentId::new(molecule_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let atom = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let assignment = number.zip(*show_number);
                let candidate = current.with_atom_number(&molecule, &atom, assignment)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetMoleculeName { molecule_id, name }) => {
                let name = name.as_deref().filter(|value| !value.is_empty());
                let candidate = current.with_molecule_name(molecule_id, name)?;
                let candidate = candidate.ok_or(SessionOperationError::UnknownMolecule)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::ApplyAtomMark {
                molecule_id,
                atom_id,
                action,
                kind,
                matching_mark_index,
            }) => {
                if *action == AtomMarkActionV1::Add && matching_mark_index.is_some() {
                    return Err(SessionOperationError::InvalidAtomMarkSelector);
                }
                let molecule = PersistentId::new(molecule_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let atom = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_mark(
                    &molecule,
                    &atom,
                    *action,
                    *kind,
                    *matching_mark_index,
                )?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetAtomPosition { atom_id, position }) => {
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_atom_position(&identifier, *position)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::RotateAtoms { rotation }) => {
                let candidate = current.with_atom_rotation(rotation)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::RepairGeometry { repair }) => {
                let candidate = current.with_geometry_repair(repair)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::DeleteAtom { atom_id }) => {
                let identifier = PersistentId::new(atom_id.clone())
                    .map_err(|_| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                let candidate = current.with_delete_atom(&identifier)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownAtom(atom_id.clone()))?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeleteBond { bond_id }) => {
                let identifier = PersistentId::new(bond_id.clone())
                    .map_err(|_| SessionOperationError::UnknownBond(bond_id.clone()))?;
                let candidate = current.with_delete_bond(&identifier)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownBond(bond_id.clone()))?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeletePresentationRoot { deletion }) => {
                let candidate = current.with_delete_presentation_root(deletion)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPresentationRoot(
                        deletion.presentation_id().as_str().to_owned(),
                    )
                })?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::DeletePresentationRoots { deletions }) => {
                let candidate = current.with_delete_presentation_roots(deletions)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPresentationRoot(
                        deletions.targets()[0].presentation_id().as_str().to_owned(),
                    )
                })?;
                Ok(Candidate::Changed(Box::new(candidate)))
            }
            Self::V1(SessionOperationV1::ReorderPresentationRoots { reorder }) => {
                let candidate = current.with_reorder_presentation_roots(reorder)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPresentationRoot(
                        reorder.targets()[0].presentation_id().as_str().to_owned(),
                    )
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::TransformTopLevelRoots { transform }) => {
                let candidate = current.with_top_level_transform(transform)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetBondOrder { bond_id, order }) => {
                let identifier = PersistentId::new(bond_id.clone())
                    .map_err(|_| SessionOperationError::UnknownBond(bond_id.clone()))?;
                let candidate = current.with_bond_order(&identifier, *order)?;
                let candidate =
                    candidate.ok_or_else(|| SessionOperationError::UnknownBond(bond_id.clone()))?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetBondProperties { patch }) => {
                let candidate = current.with_bond_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownBond(patch.bond_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetPlusProperties { patch }) => {
                let candidate = current.with_plus_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownPlus(patch.plus_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetTextProperties { patch }) => {
                let candidate = current.with_text_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownText(patch.text_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetPaperProperties { patch }) => {
                if patch.changes().is_empty() {
                    return Ok(Candidate::NoChange);
                }
                let effective_type = patch
                    .changes()
                    .iter()
                    .find_map(|change| match change {
                        PaperPropertyChangeV1::Type(value) => Some(value.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| current.paper_type_or_default_v1());
                if effective_type != "custom"
                    && patch
                        .changes()
                        .iter()
                        .any(|change| matches!(change, PaperPropertyChangeV1::Dimensions(_)))
                {
                    return Err(SessionOperationError::PaperDimensionsRequireCustom);
                }
                let candidate = current.with_paper_properties(patch)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetDrawingStandard { patch }) => {
                if patch.changes().is_empty() {
                    return Ok(Candidate::NoChange);
                }
                let candidate = current.with_drawing_standard(patch)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetArrowProperties { patch }) => {
                let candidate = current.with_arrow_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownArrow(patch.arrow_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetGeometricProperties { patch }) => {
                let candidate = current.with_geometric_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownGeometricPresentation(
                        patch.presentation_id().as_str().to_owned(),
                    )
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetWavyProperties { patch }) => {
                let candidate = current.with_wavy_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownWavy(patch.wavy_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetBracketProperties { patch }) => {
                let candidate = current.with_bracket_properties(patch)?;
                let candidate = candidate.ok_or_else(|| {
                    SessionOperationError::UnknownBracketPair(patch.pair_id().as_str().to_owned())
                })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetMoleculeAtomPositions { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let object_id = update.molecule_id().as_str().to_owned();
                let record = current
                    .resolve_document_object_id(update.molecule_id())
                    .ok_or_else(|| {
                        SessionOperationError::UnknownDocumentObject(object_id.clone())
                    })?;
                if record.class() != TypedClass::Molecule {
                    return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                        object_id,
                    ));
                }
                let source_id = record.attribute("id").ok_or_else(|| {
                    SessionOperationError::InvalidMoleculeCoordinateTarget(object_id.clone())
                })?;
                let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                    SessionOperationError::InvalidMoleculeCoordinateTarget(object_id)
                })?;
                let candidate = current
                    .with_molecule_atom_positions(&molecule_id, update.positions())?
                    .ok_or_else(|| {
                        SessionOperationError::UnknownDocumentObject(
                            update.molecule_id().as_str().to_owned(),
                        )
                    })?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::SetCleanGeometry { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let mut replacements = Vec::with_capacity(update.molecules().len());
                for molecule in update.molecules() {
                    let object_id = molecule.molecule_id().as_str().to_owned();
                    let record = current
                        .resolve_document_object_id(molecule.molecule_id())
                        .ok_or_else(|| {
                            SessionOperationError::UnknownDocumentObject(object_id.clone())
                        })?;
                    if record.class() != TypedClass::Molecule {
                        return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                            object_id,
                        ));
                    }
                    let source_id = record.attribute("id").ok_or_else(|| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id.clone())
                    })?;
                    let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id)
                    })?;
                    replacements.push((molecule_id, molecule.positions().to_vec()));
                }
                let candidate = current.with_clean_geometry_positions(&replacements)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
            Self::V1(SessionOperationV1::ApplyPreparedStraightenDepictions { update }) => {
                if update.source_revision() != current_revision {
                    return Err(SessionOperationError::MoleculeCoordinateRevisionMismatch {
                        prepared: update.source_revision(),
                        current: current_revision,
                    });
                }
                if update.source_digest() != current_digest {
                    return Err(SessionOperationError::MoleculeCoordinateDigestMismatch);
                }
                let mut replacements = Vec::with_capacity(update.molecules().len());
                for molecule in update.molecules() {
                    let object_id = molecule.molecule_id().as_str().to_owned();
                    let record = current
                        .resolve_document_object_id(molecule.molecule_id())
                        .ok_or_else(|| {
                            SessionOperationError::UnknownDocumentObject(object_id.clone())
                        })?;
                    if record.class() != TypedClass::Molecule {
                        return Err(SessionOperationError::InvalidMoleculeCoordinateTarget(
                            object_id,
                        ));
                    }
                    let source_id = record.attribute("id").ok_or_else(|| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id.clone())
                    })?;
                    let molecule_id = PersistentId::new(source_id.to_owned()).map_err(|_| {
                        SessionOperationError::InvalidMoleculeCoordinateTarget(object_id)
                    })?;
                    replacements.push((
                        molecule_id,
                        molecule.expected_positions().to_vec(),
                        molecule.positions().to_vec(),
                    ));
                }
                let candidate = current.with_prepared_straightening(&replacements)?;
                if candidate.to_xml()? == current.to_xml()? {
                    Ok(Candidate::NoChange)
                } else {
                    Ok(Candidate::Changed(Box::new(candidate)))
                }
            }
        }
    }
}
