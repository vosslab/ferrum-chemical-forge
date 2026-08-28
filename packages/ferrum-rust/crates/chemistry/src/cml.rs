//! Closed, non-retaining CML1/CML2 molecule codec.
//!
//! The decoder owns only source chemistry facts.  In particular, CML's y-down
//! coordinates remain source coordinates here; document admission owns the
//! fixed Ferrum drawing transform and all persistent identity allocation.

use thiserror::Error;

use crate::{AtomicNumber, BondDirection, BondOrder, MolBond, MolBondDirectionError};

/// One source-space atom retained by the closed CML import profile.
#[derive(Clone, Debug, PartialEq)]
pub struct CmlSourceAtomV1 {
    source_id: String,
    element: AtomicNumber,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    x2: f64,
    y2: f64,
}

impl CmlSourceAtomV1 {
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub const fn element(&self) -> AtomicNumber {
        self.element
    }
    #[must_use]
    pub const fn formal_charge(&self) -> Option<i32> {
        self.formal_charge
    }
    #[must_use]
    pub const fn isotope(&self) -> Option<u16> {
        self.isotope
    }
    #[must_use]
    pub const fn x2(&self) -> f64 {
        self.x2
    }
    #[must_use]
    pub const fn y2(&self) -> f64 {
        self.y2
    }
}

/// One owned bond expressed by stable atom-order indexes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CmlSourceBondV1 {
    start: usize,
    end: usize,
    order: BondOrder,
    direction: Option<BondDirection>,
}

impl CmlSourceBondV1 {
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }
    #[must_use]
    pub const fn order(&self) -> BondOrder {
        self.order
    }
    #[must_use]
    pub const fn direction(&self) -> Option<BondDirection> {
        self.direction
    }
    /// Lower this closed CML source bond into the existing chemistry model.
    pub fn to_mol_bond(&self) -> std::result::Result<MolBond, MolBondDirectionError> {
        match self.direction {
            Some(direction) => {
                MolBond::directed(self.start, self.end, self.order, false, direction)
            }
            None => Ok(MolBond::new(self.start, self.end, self.order, false)),
        }
    }
}

/// One complete source molecule, before document-owned coordinate conversion.
#[derive(Clone, Debug, PartialEq)]
pub struct CmlDecodedRecordV1 {
    source_molecule_id: Option<String>,
    atoms: Vec<CmlSourceAtomV1>,
    bonds: Vec<CmlSourceBondV1>,
}

impl CmlDecodedRecordV1 {
    #[must_use]
    pub fn source_molecule_id(&self) -> Option<&str> {
        self.source_molecule_id.as_deref()
    }
    #[must_use]
    pub fn atoms(&self) -> &[CmlSourceAtomV1] {
        &self.atoms
    }
    #[must_use]
    pub fn bonds(&self) -> &[CmlSourceBondV1] {
        &self.bonds
    }
}

/// Bounded ordered CML records with no retained XML or toolkit state.
#[derive(Clone, Debug, PartialEq)]
pub struct CmlDecodedDocumentV1 {
    records: Vec<CmlDecodedRecordV1>,
}

/// Closed reasons why a chemistry record cannot be represented as canonical CML2.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CmlEncoderRefusalReasonV1 {
    EmptyDocument,
    TitleUnsupported,
    PropertiesUnsupported,
    CoordinatesRequired,
    CoordinateOutOfRange,
    AtomChemistryUnsupported,
    BondChemistryUnsupported,
    OutputBytesLimit,
    GeneratedDocumentRejected,
}

/// Redacted refusal from the closed canonical CML2 encoder.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("CML output refused: {reason:?}")]
pub struct CmlEncoderErrorV1 {
    reason: CmlEncoderRefusalReasonV1,
}

impl CmlEncoderErrorV1 {
    #[must_use]
    pub const fn reason(self) -> CmlEncoderRefusalReasonV1 {
        self.reason
    }
}

impl CmlDecodedDocumentV1 {
    #[must_use]
    pub fn records(&self) -> &[CmlDecodedRecordV1] {
        &self.records
    }
}

/// Closed decoder reasons, deliberately free of source excerpts or parser detail.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CmlRefusalReasonV1 {
    InvalidUtf8,
    InvalidXml,
    InvalidXmlDeclaration,
    UnexpectedXmlText,
    UnexpectedXmlNode,
    InvalidScalar,
    InvalidCoordinate,
    CoordinateNotFinite,
    CoordinateOutOfRange,
    DuplicateSourceId,
    DuplicateAtomId,
    DanglingBond,
    SelfBond,
    DuplicateBond,
    InvalidGraph,
    EmptyDocument,
    NamespaceUnsupported,
    RootUnsupported,
    ProfileMismatch,
    AttributeUnsupported,
    ArrayAttributeUnsupported,
    UnrepresentedSemanticFact,
    DtdForbidden,
    EntityForbidden,
    ExternalResourceForbidden,
    XincludeForbidden,
    StylesheetForbidden,
    InputBytesLimit,
    XmlTextBytesLimit,
    XmlDeclarationLimit,
    CommentBytesLimit,
    PiBytesLimit,
    XmlElementLimit,
    XmlDepthLimit,
    XmlAttributeLimit,
    AttributeValueLimit,
    RecordLimit,
    AtomsPerRecordLimit,
    AtomLimit,
    BondsPerRecordLimit,
    BondLimit,
    SourceIdMapLimit,
    IdentifierBytesLimit,
    InternalFailure,
}

/// Redacted rejection from the closed CML decoder.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("CML input refused: {reason:?}")]
pub struct CmlDecoderErrorV1 {
    reason: CmlRefusalReasonV1,
}

impl CmlDecoderErrorV1 {
    #[must_use]
    pub const fn reason(self) -> CmlRefusalReasonV1 {
        self.reason
    }
}

type Result<T> = std::result::Result<T, CmlDecoderErrorV1>;
fn refused<T>(reason: CmlRefusalReasonV1) -> Result<T> {
    Err(CmlDecoderErrorV1 { reason })
}

#[path = "cml_decoder.rs"]
mod cml_decoder;
pub use cml_decoder::decode_cml_bytes_v1;

#[path = "cml_encoder.rs"]
mod cml_encoder;
pub use cml_encoder::{encode_cml_decoded_document_v1, encode_cml_interchange_records_v1};

#[cfg(test)]
#[path = "cml_tests.rs"]
mod cml_tests;
