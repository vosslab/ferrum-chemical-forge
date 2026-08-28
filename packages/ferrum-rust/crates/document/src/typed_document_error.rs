//! Concrete failures while projecting or mutating one retained typed document.

use ferrum_document_projection::{DocumentLocationV1, DocumentObjectIdV1, ProjectionError};
use thiserror::Error;

use super::{AtomMarkKindV1, IndexedDocumentError, PersistentId};

/// Parse or typed-projection failure.
#[derive(Debug, Error)]
pub enum TypedDocumentError {
    /// Typed projection rejected malformed persisted document facts.
    #[error(transparent)]
    Projection(#[from] ProjectionError),
    /// An addressable typed record did not carry its required source identifier.
    #[error("addressable typed record has no source identifier at {location:?}")]
    MissingStructuralSourceId { location: DocumentLocationV1 },
    /// An addressable typed record carried a blank source identifier.
    #[error("addressable typed record has an invalid source identifier at {location:?}")]
    InvalidStructuralSourceId { location: DocumentLocationV1 },
    /// Two addressable typed records claimed one source identifier.
    #[error("duplicate source identifier at {first:?} and {duplicate:?}")]
    DuplicateStructuralSourceId {
        first: DocumentLocationV1,
        duplicate: DocumentLocationV1,
    },
    /// Persisted opaque metadata did not use the closed document-object grammar.
    #[error(
        "addressable typed record has invalid persisted document-object identity at {location:?}"
    )]
    InvalidPersistedDocumentObjectId { location: DocumentLocationV1 },
    /// Two addressable records claimed one persisted opaque document-object identity.
    #[error("duplicate persisted document-object identity at {first:?} and {duplicate:?}")]
    DuplicatePersistedDocumentObjectId {
        first: DocumentLocationV1,
        duplicate: DocumentLocationV1,
    },
    /// The OS entropy source could not allocate a document-object identity.
    #[error("could not allocate a document-object identity")]
    DocumentObjectIdEntropy(#[source] getrandom::Error),
    /// Bounded allocation retries collided with already persisted identities.
    #[error("document-object identity allocation exhausted at {location:?}")]
    DocumentObjectIdAllocationExhausted { location: DocumentLocationV1 },
    /// A direct Text or Plus requested a font outside the bundled closed face set.
    #[error("unsupported_text_face for {root_id}: {family:?}; use Telex Regular (bundled)")]
    UnsupportedTextFace {
        /// Authored direct-root identifier, or a stable fallback when absent.
        root_id: String,
        /// Exact rejected authored family spelling.
        family: String,
    },
    /// A typed atom's authored multiplicity is not a positive 16-bit integer.
    #[error(
        "typed atom {atom_id} has invalid multiplicity {value:?}; expected a positive 16-bit integer"
    )]
    InvalidAtomMultiplicity {
        /// Authored atom ID, or a stable descriptive fallback when absent.
        atom_id: String,
        /// Exact retained lexical value.
        value: String,
    },
    /// A direct molecule root has no typed vertex representable by the molecular model.
    #[error(
        "typed direct molecule {molecule_id} has no supported molecular vertex; expected atom, compact-group, text, or query"
    )]
    EmptyDirectMolecule {
        /// Authored molecule ID, or a stable descriptive fallback when absent.
        molecule_id: String,
    },
    /// Legacy molecule-local group records have no current typed CDML meaning.
    #[error("legacy molecule group records are unsupported")]
    UnsupportedLegacyGroup,
    /// A compact-group deletion request did not name one eligible direct-root molecule.
    #[error("compact-group deletion molecule is not one eligible direct-root molecule: {0}")]
    InvalidCompactGroupDeletionMolecule(PersistentId),
    /// A compact-group deletion request did not name one direct compact group.
    #[error("compact-group deletion target is not one direct compact group: {0}")]
    InvalidCompactGroupDeletionTarget(PersistentId),
    /// A compact-group deletion target did not have one direct exterior atom bond.
    #[error("compact-group deletion target has invalid direct exterior topology: {0}")]
    InvalidCompactGroupDeletionTopology(PersistentId),
    /// A structural deletion request did not name one eligible direct-root molecule.
    #[error("structural deletion molecule is not one eligible direct-root molecule: {0}")]
    InvalidStructureDeletionMolecule(PersistentId),
    /// An eligible structural-deletion molecule has content outside the direct core profile.
    #[error("structural deletion molecule has unsupported direct content: {0}")]
    UnsupportedStructureDeletionMolecule(PersistentId),
    /// An eligible structural-deletion molecule has malformed direct graph topology.
    #[error("structural deletion molecule has malformed direct topology: {0}")]
    InvalidStructureDeletionTopology(PersistentId),
    /// A structural deletion selection did not resolve to one direct durable child.
    #[error("structural deletion target is not one direct durable child: {0}")]
    InvalidStructureDeletionTarget(PersistentId),
    /// A reaction role would be left invalid by removing or splitting its molecule.
    #[error("structural deletion cannot remove or split reaction-referenced molecule: {0}")]
    ReactionReferencedStructureDeletion(PersistentId),
    /// A presentation deletion would leave a direct reaction role dangling.
    #[error("presentation deletion cannot remove reaction-referenced root")]
    ReactionReferencedPresentationDeletion(DocumentObjectIdV1),
    /// Session-only structural deletion must receive its allocated split identities.
    #[error("structural deletion requires session-owned split identities")]
    StructuralDeletionRequiresSession,
    /// A complete-root translation observation received an invalid root request.
    #[error(transparent)]
    TopLevelTransform(#[from] super::TopLevelTransformV1Error),
    /// A linear-form adapter allocation could not be reserved from admitted source size.
    #[error("typed linear-form conversion exhausted available resources")]
    LinearFormResourceExhausted,
    /// A linear-form request did not name one direct typed molecule.
    #[error("typed linear-form molecule selector is not one direct molecule")]
    InvalidLinearFormMolecule,
    /// A linear-form request named an atom outside its selected direct molecule.
    #[error("typed linear-form atom selector is not one direct atom: {0}")]
    InvalidLinearFormAtom(PersistentId),
    /// A direct molecule contains an unsupported linear-form atom, bond, or mark fact.
    #[error("typed linear-form source has unsupported content: {0}")]
    InvalidLinearFormSource(PersistentId),
    /// More than one exact generated linear form owns the requested members.
    #[error("typed linear-form ownership is ambiguous")]
    AmbiguousLinearFormOwnership,
    /// The supplied generated fragment identity is already reserved.
    #[error("typed linear-form fragment identifier is already reserved: {0}")]
    DuplicateLinearFormFragment(PersistentId),
    /// XML admission failed before the retained tree was built.
    #[error(transparent)]
    XmlInput(#[from] super::XmlInputError),
    /// XML parsing or document identity validation failed.
    #[error(transparent)]
    Indexed(#[from] IndexedDocumentError),
    /// An opaque subtree could not be snapshotted structurally.
    #[error("cannot retain an opaque CDML subtree: {0}")]
    OpaqueSnapshot(#[source] xot::Error),
    /// A namespaced unknown attribute had no usable in-scope prefix.
    #[error("cannot retain an unknown CDML attribute name: {0}")]
    AttributeName(#[source] xot::Error),
    /// A first-class compact-group V1 record carried an undeclared attribute.
    #[error("compact-group V1 has an undeclared attribute: {attribute}")]
    UndeclaredCompactGroupAttribute {
        /// Attribute spelling as authored in the retained XML.
        attribute: String,
    },
    /// A first-class compact-group V1 record carried content outside its one anchor point.
    #[error("compact-group V1 has undeclared content")]
    UndeclaredCompactGroupContent,
    /// A retained tree could not be structurally serialized for a typed mutation.
    #[error("cannot serialize retained CDML: {0}")]
    Serialize(#[from] super::XmlSerializationError),
    /// A typed atom element spelling is blank or contains non-letter characters.
    #[error("atom element must be a nonblank plain element spelling")]
    InvalidAtomElement,
    /// The requested molecule does not occur in the retained document.
    #[error("typed molecule does not exist: {0}")]
    UnknownMolecule(PersistentId),
    /// The requested atom ID is already reserved by retained document content.
    #[error("persistent atom ID already exists: {0}")]
    DuplicateAtomId(PersistentId),
    /// The requested compact-group ID is already reserved by retained document content.
    #[error("persistent compact-group ID already exists: {0}")]
    DuplicateGroupId(PersistentId),
    /// The requested bond ID is already reserved by retained document content.
    #[error("persistent bond ID already exists: {0}")]
    DuplicateBondId(PersistentId),
    /// A requested bond endpoint is not a direct typed atom of the target molecule.
    #[error("bond endpoint is not an atom in the target molecule: {0}")]
    InvalidBondEndpoint(PersistentId),
    /// A typed atom targeted for movement has no direct point child.
    #[error("typed atom has no movable point: {0}")]
    MissingAtomPosition(PersistentId),
    /// An atom-properties font edit found more than one direct typed font.
    #[error("typed atom has multiple direct label fonts: {0}")]
    AmbiguousAtomFonts(PersistentId),
    /// A direct-root Plus edit did not find exactly one core point.
    #[error("typed Plus has unsupported direct-child geometry: {0}")]
    InvalidPlusStructure(PersistentId),
    /// A direct-root Plus font edit found more than one direct typed font.
    #[error("typed Plus has multiple direct fonts: {0}")]
    AmbiguousPlusFonts(PersistentId),
    /// A direct-root Text edit found malformed or unsupported core children.
    #[error("typed Text has unsupported editable structure: {0}")]
    InvalidTextStructure(PersistentId),
    /// A direct-root Text font edit found more than one direct typed font.
    #[error("typed Text has multiple direct fonts: {0}")]
    AmbiguousTextFonts(PersistentId),
    /// An individual presentation deletion targeted one member of a durable bracket pair.
    #[error("typed presentation is a bracket member and requires pair deletion")]
    PresentationRootIsBracketMember(DocumentObjectIdV1),
    /// A multi-root deletion selected only one member of a durable bracket pair.
    #[error("presentation deletion requires both members of bracket pair")]
    PartialBracketDeletion([DocumentObjectIdV1; 2]),
    /// A stack reorder selected only one member of a durable bracket pair.
    #[error("presentation stack reorder requires both members of bracket pair")]
    PartialBracketStackSelection([DocumentObjectIdV1; 2]),
    /// A rigid transform selected only one member of a durable bracket pair.
    #[error("top-level transform requires both members of bracket pair")]
    PartialBracketTransform([DocumentObjectIdV1; 2]),
    /// A durable direct-root transform target did not match its exact kind.
    #[error("typed top-level transform root does not exist")]
    UnknownTopLevelTransformRoot(DocumentObjectIdV1),
    /// A direct-root transform target has ambiguous or invalid persistent geometry.
    #[error("typed top-level transform root has unsupported geometry")]
    InvalidTopLevelTransformGeometry(DocumentObjectIdV1),
    /// A finite source geometry would become nonfinite under the requested transform.
    #[error("typed top-level transform produces nonfinite geometry")]
    NonFiniteTopLevelTransform(DocumentObjectIdV1),
    /// A rotation target did not resolve to one direct atom in its declared molecule.
    #[error("typed atom rotation target does not exist: molecule {molecule_id}, atom {atom_id}")]
    UnknownAtomRotationTarget {
        molecule_id: PersistentId,
        atom_id: PersistentId,
    },
    /// A rotation target does not have one finite direct persistent point.
    #[error("typed atom rotation target has unsupported geometry: {0}")]
    InvalidAtomRotationGeometry(PersistentId),
    /// A finite source point would become nonfinite under the requested rotation.
    #[error("typed atom rotation produces nonfinite geometry: {0}")]
    NonFiniteAtomRotation(PersistentId),
    /// A selected direct-root molecule does not exist.
    #[error("typed geometry-repair molecule does not exist: {0}")]
    UnknownGeometryRepairMolecule(PersistentId),
    /// A selected molecule contains core semantics outside the implemented repair subset.
    #[error("typed geometry-repair molecule has unsupported content: {0}")]
    UnsupportedGeometryRepairMolecule(PersistentId),
    /// A selected molecule atom does not have one finite direct point.
    #[error("typed geometry-repair atom has unsupported geometry: {0}")]
    InvalidGeometryRepairAtom(PersistentId),
    /// A selected molecule bond lacks a valid durable endpoint fact.
    #[error("typed geometry-repair bond has invalid field: {0}")]
    InvalidGeometryRepairBond(String),
    /// A source identity cannot enter the internal repair graph.
    #[error("typed geometry-repair identity is invalid: {0}")]
    InvalidGeometryRepairIdentity(PersistentId),
    /// The pure repair planner rejected the selected molecule.
    #[error("typed geometry-repair planning failed for {molecule_id}: {detail}")]
    GeometryRepairPlanning {
        molecule_id: PersistentId,
        detail: String,
    },
    /// A planned atom coordinate no longer matches its source snapshot.
    #[error("typed geometry-repair coordinate precondition failed: {0}")]
    GeometryRepairPrecondition(PersistentId),
    /// A direct-root Arrow edit found malformed or unsupported core children.
    #[error("typed Arrow has unsupported editable structure: {0}")]
    InvalidArrowStructure(PersistentId),
    /// A requested Arrow field has an invalid retained lexical value.
    #[error("typed Arrow has an invalid requested property: {0}")]
    InvalidArrowProperty(PersistentId),
    /// A direct-root geometric edit found malformed or unsupported core content.
    #[error("typed geometric presentation has unsupported editable structure: {0}")]
    InvalidGeometricStructure(PersistentId),
    /// A requested geometric field has an invalid retained lexical value.
    #[error("typed geometric presentation has an invalid requested property: {0}")]
    InvalidGeometricProperty(PersistentId),
    /// Fill intent was supplied for an open polyline.
    #[error("typed geometric presentation does not support the requested fill: {0}")]
    InapplicableGeometricProperty(PersistentId),
    /// A specialized Wavy polyline was sent to the ordinary geometry operation.
    #[error("typed geometric presentation uses a dedicated specialized operation: {0}")]
    SpecializedGeometricTarget(PersistentId),
    /// A Wavy edit found malformed or unsupported core point content.
    #[error("typed Wavy presentation has unsupported editable structure: {0}")]
    InvalidWavyStructure(PersistentId),
    /// A requested Wavy field has an invalid retained lexical value.
    #[error("typed Wavy presentation has an invalid requested property: {0}")]
    InvalidWavyProperty(PersistentId),
    /// The effective drawing standard cannot supply a trustworthy new bracket stroke.
    #[error("drawing standard has invalid bracket stroke facts")]
    InvalidBracketStandard,
    /// A marked bracket pair does not have exactly two editable retained sides.
    #[error("typed bracket has unsupported editable pair structure")]
    InvalidBracketPair([DocumentObjectIdV1; 2]),
    /// An atom-number edit targeted an atom carrying the incompatible legacy mark.
    #[error("typed atom has a direct legacy atom-number mark: {0}")]
    LegacyAtomNumberMark(PersistentId),
    /// An atom-mark edit target did not carry exactly one usable direct point.
    #[error("typed atom has unsupported mark geometry: {0}")]
    InvalidAtomMarkPoint(PersistentId),
    /// A chemical atom-mark delta addressed a malformed integer scalar.
    #[error("typed atom {atom} has an invalid {field} value for a mark operation")]
    InvalidAtomMarkScalar {
        atom: PersistentId,
        /// Addressed atom scalar.
        field: &'static str,
    },
    /// A chemical atom-mark delta would leave the supported scalar range.
    #[error("typed atom {atom} mark operation would set {field} to {value}")]
    AtomMarkScalarOutOfRange {
        atom: PersistentId,
        /// Addressed atom scalar.
        field: &'static str,
        /// Rejected result.
        value: i32,
    },
    /// A selected same-type mark ordinal did not exist on the target atom.
    #[error("typed atom {atom} has no {kind:?} mark at same-type ordinal {index}")]
    AtomMarkIndexOutOfRange {
        /// Durable target atom ID.
        atom: PersistentId,
        /// Exact supported mark kind.
        kind: AtomMarkKindV1,
        /// Rejected zero-based same-type ordinal.
        index: u32,
    },
    /// A bond type cannot safely compose one closed V1 order/style edit.
    #[error("typed bond has an unsupported V1 type: {0}")]
    UnsupportedBondType(PersistentId),
    /// A direct bond cannot reverse endpoint direction unless it is exactly `w1` or `h1`.
    #[error("typed bond does not support directed endpoint reversal: {0}")]
    UnsupportedDirectedBondEndpointReversal(PersistentId),
    /// A non-normal presentation cannot be changed through the normal-order operation.
    #[error("typed bond presentation does not support normal-order replacement: {0}")]
    UnsupportedBondPresentationOrder(PersistentId),
    /// An authored scalar property is not meaningful for the final bond presentation.
    #[error("typed bond presentation does not support {property}: {bond_id}")]
    IncompatibleBondPresentationProperty {
        /// Durable target bond ID.
        bond_id: PersistentId,
        /// Exact closed property name.
        property: &'static str,
    },
    /// A complete molecule-coordinate update supplied the wrong atom count.
    #[error(
        "molecule {molecule} has {expected} typed atoms but the coordinate update supplied {actual}"
    )]
    MoleculePositionCountMismatch {
        /// Durable target molecule ID.
        molecule: PersistentId,
        /// Direct typed atom count in the retained molecule.
        expected: usize,
        /// Supplied coordinate count.
        actual: usize,
    },
    /// A coordinate update resolved a molecule record outside the direct-root boundary.
    #[error("molecule {0} is not a direct typed coordinate target")]
    InvalidMoleculeCoordinateTarget(PersistentId),
    /// One direct atom in a molecule-coordinate update had no movable point.
    #[error("typed atom {atom_index} in molecule {molecule} has no movable point")]
    MissingMoleculeAtomPosition {
        /// Durable target molecule ID.
        molecule: PersistentId,
        /// Zero-based typed atom source order.
        atom_index: usize,
    },
    /// A retained bond already connects the requested endpoints.
    #[error("a bond already connects {start} and {end}")]
    DuplicateBond {
        /// First requested atom ID.
        start: PersistentId,
        /// Second requested atom ID.
        end: PersistentId,
    },
    /// A complete insertion supplied duplicate or already-reserved persistent identity.
    #[error("persistent molecule insertion ID already exists: {0}")]
    DuplicateInsertionId(PersistentId),
    /// Session-owned atom or bond identities did not match the validated insertion graph.
    #[error("molecule insertion identity counts do not match its graph")]
    InsertionIdentityCountMismatch,
    /// A requested durable stereo report cannot be represented by this molecule.
    #[error("molecule insertion has invalid durable stereo semantics")]
    InvalidStereoSemantics,
    /// A retained CDML stereo-semantics child has malformed scalar content.
    #[error("CDML stereo semantics has malformed {field}")]
    MalformedStereoSemantics {
        /// The canonical scalar or attribute that could not be decoded.
        field: &'static str,
    },
    /// A retained CDML stereo-semantics child uses an unsupported entry or value.
    #[error("CDML stereo semantics has unsupported {field}")]
    UnsupportedStereoSemantics {
        /// The canonical child, attribute, or enum spelling that is unsupported.
        field: &'static str,
    },
    /// A structured XML mutation could not be applied to the retained tree.
    #[error("cannot mutate retained CDML: {0}")]
    Mutation(#[source] xot::Error),
}
