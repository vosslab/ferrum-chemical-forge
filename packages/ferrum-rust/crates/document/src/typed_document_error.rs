//! Concrete failures while projecting or mutating one retained typed document.

use thiserror::Error;

use super::{AtomMarkKindV1, IndexedDocumentError, PersistentId};

/// Parse or typed-projection failure.
#[derive(Debug, Error)]
pub enum TypedDocumentError {
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
    #[error("typed presentation is a bracket member and requires pair deletion: {0}")]
    PresentationRootIsBracketMember(PersistentId),
    /// A multi-root deletion selected only one member of a durable bracket pair.
    #[error("presentation deletion requires both members of bracket pair: {0}")]
    PartialBracketDeletion(String),
    /// A stack reorder selected only one member of a durable bracket pair.
    #[error("presentation stack reorder requires both members of bracket pair: {0}")]
    PartialBracketStackSelection(String),
    /// A rigid transform selected only one member of a durable bracket pair.
    #[error("top-level transform requires both members of bracket pair: {0}")]
    PartialBracketTransform(String),
    /// A durable direct-root transform target did not match its exact kind.
    #[error("typed top-level transform root does not exist: {0}")]
    UnknownTopLevelTransformRoot(PersistentId),
    /// A direct-root transform target has ambiguous or invalid persistent geometry.
    #[error("typed top-level transform root has unsupported geometry: {0}")]
    InvalidTopLevelTransformGeometry(PersistentId),
    /// A finite source geometry would become nonfinite under the requested transform.
    #[error("typed top-level transform produces nonfinite geometry: {0}")]
    NonFiniteTopLevelTransform(PersistentId),
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
    #[error("typed bracket has unsupported editable pair structure: {0}")]
    InvalidBracketPair(PersistentId),
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
    /// A closed bond style and order cannot be composed into an authored V1 type.
    #[error("typed bond has an unsupported V1 style/order combination: {0}")]
    UnsupportedBondStyleOrder(PersistentId),
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
    /// A structured XML mutation could not be applied to the retained tree.
    #[error("cannot mutate retained CDML: {0}")]
    Mutation(#[source] xot::Error),
}
