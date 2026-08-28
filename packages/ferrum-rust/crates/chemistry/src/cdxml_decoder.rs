//! Bounded Rust-owned CDXML simple-molecule import profile.

#[path = "cdxml_decoder/parser.rs"]
mod parser;
#[path = "cdxml_decoder/values.rs"]
mod values;

use thiserror::Error;
use xmlparser::Tokenizer;

use crate::{
    AtomicNumber, BondDirection, BondOrder, Coordinates, InterchangeRecordV1, MolAtom, MolBond,
    MolGraph, Point2,
};

/// Maximum accepted source size for the bounded CDXML simple-molecule profile.
pub const CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1: usize = 1_048_576;

/// A declared, intentionally omitted CDXML source category.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CdxmlLossCategoryV1 {
    LexicalSyntax,
    DocumentViewMetadata,
}

/// A fixed-single CDXML bond depiction retained until document admission.
///
/// This source-specific fact is deliberately separate from stereochemical
/// [`BondDirection`].  Its order is the source `MolGraph` bond order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CdxmlBondPresentationV1 {
    Wavy,
    Bold,
    Dashed,
}

/// One source fragment converted into an owned chemistry record.
#[derive(Clone, Debug, PartialEq)]
pub struct CdxmlDecodedRecordV1 {
    source_fragment_id: String,
    record: InterchangeRecordV1,
    bond_presentations: Vec<Option<CdxmlBondPresentationV1>>,
}
impl CdxmlDecodedRecordV1 {
    /// Construct one internally validated CDXML record and its source-order
    /// presentation carrier.
    ///
    /// `bond_presentations[index]` always describes `record.molecule().bonds()[index]`.
    pub(crate) fn new(
        source_fragment_id: String,
        record: InterchangeRecordV1,
        bond_presentations: Vec<Option<CdxmlBondPresentationV1>>,
    ) -> Result<Self> {
        let bonds = record.molecule().bonds();
        if bond_presentations.len() != bonds.len()
            || bond_presentations
                .iter()
                .zip(bonds)
                .any(|(presentation, bond)| {
                    presentation.is_some()
                        && (bond.order() != BondOrder::Single
                            || bond.direction() != BondDirection::None)
                })
        {
            return refused(CdxmlRefusalReasonV1::InternalFailure);
        }
        Ok(Self {
            source_fragment_id,
            record,
            bond_presentations,
        })
    }
    #[must_use]
    pub fn source_fragment_id(&self) -> &str {
        &self.source_fragment_id
    }
    #[must_use]
    pub fn record(&self) -> &InterchangeRecordV1 {
        &self.record
    }
    #[must_use]
    pub fn bond_presentations(&self) -> &[Option<CdxmlBondPresentationV1>] {
        &self.bond_presentations
    }
}

/// Ordered records and a canonical, deduplicated set of declared source losses.
#[derive(Clone, Debug, PartialEq)]
pub struct CdxmlDecodedDocumentV1 {
    pub(crate) records: Vec<CdxmlDecodedRecordV1>,
    pub(crate) declared_losses: Vec<CdxmlLossCategoryV1>,
}
impl CdxmlDecodedDocumentV1 {
    #[must_use]
    pub fn records(&self) -> &[CdxmlDecodedRecordV1] {
        &self.records
    }
    #[must_use]
    /// Return loss categories in canonical enum order, with each category at most once.
    pub fn declared_losses(&self) -> &[CdxmlLossCategoryV1] {
        &self.declared_losses
    }
}

/// Closed, redacted reasons a CDXML source cannot enter this profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CdxmlRefusalReasonV1 {
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
    AttributeUnsupported,
    UnrepresentedSemanticFact,
    DtdForbidden,
    EntityForbidden,
    InputBytesLimit,
    XmlElementLimit,
    AttributeValueLimit,
    RecordLimit,
    AtomsPerRecordLimit,
    BondsPerRecordLimit,
    IdentifierBytesLimit,
    InternalFailure,
}

/// Redacted rejection from the closed CDXML decoder.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("CDXML input refused: {reason:?}")]
pub struct CdxmlDecoderErrorV1 {
    pub(crate) reason: CdxmlRefusalReasonV1,
}
impl CdxmlDecoderErrorV1 {
    #[must_use]
    pub const fn reason(self) -> CdxmlRefusalReasonV1 {
        self.reason
    }
}
type Result<T> = std::result::Result<T, CdxmlDecoderErrorV1>;
fn refused<T>(reason: CdxmlRefusalReasonV1) -> Result<T> {
    Err(CdxmlDecoderErrorV1 { reason })
}

/// Decode bounded UTF-8 CDXML into direct page-fragment chemistry records.
pub fn decode_cdxml_bytes_v1(input: &[u8]) -> Result<CdxmlDecodedDocumentV1> {
    let input = input.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(input);
    if input.len() > CDXML_SIMPLE_MOLECULE_IMPORT_MAX_SOURCE_BYTES_V1 {
        return refused(CdxmlRefusalReasonV1::InputBytesLimit);
    }
    let source = std::str::from_utf8(input).map_err(|_| CdxmlDecoderErrorV1 {
        reason: CdxmlRefusalReasonV1::InvalidUtf8,
    })?;
    let mut parser = parser::Parser::new();
    for token in Tokenizer::from(source) {
        parser.token(token.map_err(|_| CdxmlDecoderErrorV1 {
            reason: CdxmlRefusalReasonV1::InvalidXml,
        })?)?;
    }
    parser.finish()
}

#[cfg(test)]
#[path = "cdxml_tests.rs"]
mod cdxml_tests;
