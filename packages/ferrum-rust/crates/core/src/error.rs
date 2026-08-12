use thiserror::Error;

use crate::RecordKind;

/// Core structural validation errors.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ModelError {
    #[error("legacy fingerprint has an invalid version, kind, length encoding, or field shape")]
    MalformedLegacyFingerprint,
    #[error("{axis} coordinate must be finite")]
    NonFiniteCoordinate { axis: &'static str },
    #[error("present atom element is blank")]
    BlankAtomElement,
    #[error("present atom multiplicity is zero")]
    ZeroMultiplicity,
    #[error("present bond type is blank")]
    BlankBondType,
    #[error("{kind:?} source record cannot carry a legacy occurrence")]
    SourceRecordHasLegacyOccurrence { kind: RecordKind },
    #[error("idless {kind:?} record needs an occurrence among equal fingerprints")]
    MissingLegacyOccurrence { kind: RecordKind },
    #[error("invalid non-atom vertex kind {kind:?}")]
    InvalidVertexKind { kind: RecordKind },
    #[error("{kind:?} identity does not match its kind, origin, or carried fields")]
    IdentityMismatch { kind: RecordKind },
    #[error("duplicate internal identity")]
    DuplicateIdentity,
    #[error("duplicate molecule-local source identifier")]
    DuplicateSourceId,
    #[error("bond has identical typed endpoints")]
    SelfBond,
    #[error("bond endpoint does not resolve to its declared typed vertex")]
    UnresolvedBondEndpoint,
    #[error("vertex reference variant does not match its record kind")]
    VertexKindMismatch,
}
