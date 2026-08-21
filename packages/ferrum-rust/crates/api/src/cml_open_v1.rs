//! Fixed-target CML open operation for one new Ferrum document.

use ferrum_chemistry::{BondOrder, CmlDecodedRecordV1, CmlRefusalReasonV1, decode_cml_bytes_v1};
use ferrum_document::{
    DocumentBondOrderV1, DocumentSession, DocumentSessionError, MoleculeInsertionAtomV1,
    MoleculeInsertionBondV1, MoleculeInsertionV1, PendingCreateMoleculeBatchV1, Point3V1,
};
use serde::Serialize;

use crate::interchange_import_v1::{
    CML_IMPORT_RESPONSE_BUDGET_BYTES_V1, CmlImportRefusalReasonV1, CmlImportRefusalV1,
};
use crate::protocol::{
    DocumentMoleculeInterchangeImportLossReportV1, DocumentMoleculeInterchangeImportSummaryV1,
};

pub(crate) const CML_OPEN_OPERATION_ID_V1: &str = "document.molecule.interchange.import.v1";
pub(crate) const CML_OPEN_RESPONSE_SCHEMA_V1: &str = "ferrum-cml-import-response-v1";

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum CmlOpenEnvelopeV1 {
    Success {
        schema: &'static str,
        summary: DocumentMoleculeInterchangeImportSummaryV1,
    },
    Refused {
        schema: &'static str,
        refusal: CmlImportRefusalV1,
    },
}

/// CDML remains private until a caller safely publishes it after admission.
#[derive(Clone, Debug)]
pub(crate) struct CompletedCmlOpenV1 {
    envelope: CmlOpenEnvelopeV1,
    document_cdml: Option<String>,
}

/// A private create-only CML admission held until its caller admits its final envelope.
pub(crate) struct PreparedCmlOpenV1 {
    session: DocumentSession,
    baseline_revision: u64,
    pending: PendingCreateMoleculeBatchV1,
    summary: DocumentMoleculeInterchangeImportSummaryV1,
}

impl PreparedCmlOpenV1 {
    #[must_use]
    pub(crate) fn summary(&self) -> &DocumentMoleculeInterchangeImportSummaryV1 {
        &self.summary
    }

    pub(crate) fn commit_and_take_cdml(mut self) -> Result<String, CmlImportRefusalV1> {
        self.session
            .commit_create_molecule_batch_v1(self.baseline_revision, &mut self.pending)
            .map_err(|error| CmlImportRefusalV1::for_reason(map_document_error(&error)))?;
        self.session
            .snapshot()
            .map(|snapshot| snapshot.cdml().to_owned())
            .map_err(|_| CmlImportRefusalV1::for_reason(CmlImportRefusalReasonV1::InternalFailure))
    }
}

impl CompletedCmlOpenV1 {
    #[must_use]
    pub(crate) const fn envelope(&self) -> &CmlOpenEnvelopeV1 {
        &self.envelope
    }

    pub(crate) fn take_document_cdml(&mut self) -> Option<String> {
        self.document_cdml.take()
    }
}

/// Decode CML and commit it to one temporary new document.
#[must_use]
pub(crate) fn open_cml_new_document_v1(cml_bytes: &[u8]) -> CompletedCmlOpenV1 {
    let prepared = match prepare_cml_new_document_v1(cml_bytes) {
        Ok(prepared) => prepared,
        Err(refusal) => return refused(refusal.reason()),
    };
    let envelope = CmlOpenEnvelopeV1::Success {
        schema: CML_OPEN_RESPONSE_SCHEMA_V1,
        summary: prepared.summary().clone(),
    };
    if !cml_open_envelope_is_admitted(&envelope) {
        return refused(CmlImportRefusalReasonV1::ResponseBytesLimit);
    }
    let document_cdml = match prepared.commit_and_take_cdml() {
        Ok(document) => document,
        Err(refusal) => return refused(refusal.reason()),
    };
    CompletedCmlOpenV1 {
        envelope,
        document_cdml: Some(document_cdml),
    }
}

/// Prepare one new document without exposing CDML or committing a history transition.
pub(crate) fn prepare_cml_new_document_v1(
    cml_bytes: &[u8],
) -> Result<PreparedCmlOpenV1, CmlImportRefusalV1> {
    let decoded = match decode_cml_bytes_v1(cml_bytes) {
        Ok(decoded) => decoded,
        Err(error) => {
            return Err(CmlImportRefusalV1::for_reason(map_decoder_reason(
                error.reason(),
            )));
        }
    };
    let source_record_count = decoded.records().len();
    let atom_count = decoded
        .records()
        .iter()
        .map(|record| record.atoms().len())
        .sum();
    let bond_count = decoded
        .records()
        .iter()
        .map(|record| record.bonds().len())
        .sum();
    let mut session = match DocumentSession::create_empty_document_v1() {
        Ok(session) => session,
        Err(_) => {
            return Err(CmlImportRefusalV1::for_reason(
                CmlImportRefusalReasonV1::InternalFailure,
            ));
        }
    };
    let baseline = match session.snapshot() {
        Ok(snapshot) => snapshot,
        Err(_) => {
            return Err(CmlImportRefusalV1::for_reason(
                CmlImportRefusalReasonV1::InternalFailure,
            ));
        }
    };
    let records = decoded
        .records()
        .iter()
        .map(convert_record)
        .collect::<Result<Vec<_>, _>>()?;
    let pending = match session.prepare_create_molecule_batch_v1(baseline.revision(), &records) {
        Ok(pending) => pending,
        Err(error) => return Err(CmlImportRefusalV1::for_reason(map_document_error(&error))),
    };
    let Some((document_revision, digest)) = pending.candidate_revision_and_digest_v1() else {
        return Err(CmlImportRefusalV1::for_reason(
            CmlImportRefusalReasonV1::InternalFailure,
        ));
    };
    Ok(PreparedCmlOpenV1 {
        session,
        baseline_revision: baseline.revision(),
        pending,
        summary: DocumentMoleculeInterchangeImportSummaryV1 {
            operation: CML_OPEN_OPERATION_ID_V1,
            target: "new_document",
            document_revision,
            document_digest_hex: digest.iter().map(|byte| format!("{byte:02x}")).collect(),
            source_record_count,
            inserted_record_count: source_record_count,
            atom_count,
            bond_count,
            profile_id: crate::interchange_import_v1::CML_SIMPLE_MOLECULE_IMPORT_PROFILE_V1,
            format_id: crate::interchange_import_v1::CML_SIMPLE_MOLECULE_IMPORT_FORMAT_V1,
            loss_report: DocumentMoleculeInterchangeImportLossReportV1 {
                source_ids_reallocated: true,
                lexical_xml_not_retained: true,
                semantic_loss: Vec::new(),
            },
        },
    })
}

fn convert_record(record: &CmlDecodedRecordV1) -> Result<MoleculeInsertionV1, CmlImportRefusalV1> {
    let atoms = record
        .atoms()
        .iter()
        .map(|atom| {
            let x = transform_coordinate(atom.x2())?;
            let y = transform_coordinate(-atom.y2())?;
            MoleculeInsertionAtomV1::new(
                atom.element().symbol(),
                Point3V1::new(x, y, 0.0).map_err(|_| {
                    CmlImportRefusalV1::for_reason(
                        CmlImportRefusalReasonV1::CandidateValidationFailed,
                    )
                })?,
                atom.formal_charge(),
                atom.isotope(),
                None,
            )
            .map_err(|_| {
                CmlImportRefusalV1::for_reason(CmlImportRefusalReasonV1::CandidateValidationFailed)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bonds = record
        .bonds()
        .iter()
        .map(|bond| {
            let order = match bond.order() {
                BondOrder::Single => DocumentBondOrderV1::Single,
                BondOrder::Double => DocumentBondOrderV1::Double,
                BondOrder::Triple => DocumentBondOrderV1::Triple,
                _ => {
                    return Err(CmlImportRefusalV1::for_reason(
                        CmlImportRefusalReasonV1::CandidateValidationFailed,
                    ));
                }
            };
            Ok(MoleculeInsertionBondV1::new(
                bond.start(),
                bond.end(),
                order,
            ))
        })
        .collect::<Result<Vec<_>, _>>()?;
    MoleculeInsertionV1::new(atoms, bonds).map_err(|_| {
        CmlImportRefusalV1::for_reason(CmlImportRefusalReasonV1::CandidateValidationFailed)
    })
}

fn transform_coordinate(value: f64) -> Result<f64, CmlImportRefusalV1> {
    const SCALE: f64 = 30.0;
    const LIMIT: f64 = 3_000_000.0;
    let value = SCALE * value;
    if !value.is_finite() || value.abs() > LIMIT {
        return Err(CmlImportRefusalV1::for_reason(
            CmlImportRefusalReasonV1::CandidateValidationFailed,
        ));
    }
    Ok(value)
}

fn map_document_error(error: &DocumentSessionError) -> CmlImportRefusalReasonV1 {
    match error {
        DocumentSessionError::Load(_) | DocumentSessionError::Serialize(_) => {
            CmlImportRefusalReasonV1::SerializationFailed
        }
        _ => CmlImportRefusalReasonV1::InternalFailure,
    }
}

fn map_decoder_reason(reason: CmlRefusalReasonV1) -> CmlImportRefusalReasonV1 {
    macro_rules! same_reason {
        ($($variant:ident),+ $(,)?) => {
            match reason {
                $(CmlRefusalReasonV1::$variant => CmlImportRefusalReasonV1::$variant,)+
            }
        };
    }
    same_reason!(
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
    )
}

pub(crate) fn canonical_cml_open_envelope_json_v1(
    envelope: &CmlOpenEnvelopeV1,
) -> Result<Vec<u8>, CmlImportRefusalV1> {
    let bytes = serde_json::to_vec(envelope).map_err(|_| {
        CmlImportRefusalV1::for_reason(CmlImportRefusalReasonV1::SerializationFailed)
    })?;
    if bytes.len() > CML_IMPORT_RESPONSE_BUDGET_BYTES_V1 {
        return Err(CmlImportRefusalV1::for_reason(
            CmlImportRefusalReasonV1::ResponseBytesLimit,
        ));
    }
    Ok(bytes)
}

fn refused(reason: CmlImportRefusalReasonV1) -> CompletedCmlOpenV1 {
    let envelope = CmlOpenEnvelopeV1::Refused {
        schema: CML_OPEN_RESPONSE_SCHEMA_V1,
        refusal: CmlImportRefusalV1::for_reason(reason),
    };
    // Refusals use only fixed schema and closed enum values, but still pass
    // through the same exact response measurement before they can leave this
    // CML-specific boundary.
    assert!(
        cml_open_envelope_is_admitted(&envelope),
        "closed CML refusal envelope must fit its frozen response budget"
    );
    CompletedCmlOpenV1 {
        envelope,
        document_cdml: None,
    }
}

fn cml_open_envelope_is_admitted(envelope: &CmlOpenEnvelopeV1) -> bool {
    canonical_cml_open_envelope_json_v1(envelope).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CML: &str = concat!(
        "<cml xmlns=\"http://www.xml-cml.org/schema/cml2/core\"><molecule><atomArray>",
        "<atom id=\"a1\" elementType=\"C\" x2=\"0\" y2=\"0\"/>",
        "</atomArray></molecule></cml>"
    );

    #[test]
    fn open_commits_and_measures_before_releasing_cdml() {
        let mut completed = open_cml_new_document_v1(CML.as_bytes());
        assert!(matches!(
            completed.envelope(),
            CmlOpenEnvelopeV1::Success { .. }
        ));
        assert!(canonical_cml_open_envelope_json_v1(completed.envelope()).is_ok());
        assert!(completed.take_document_cdml().is_some());
        assert!(completed.take_document_cdml().is_none());
    }

    #[test]
    fn refused_input_has_no_publishable_document() {
        let mut completed = open_cml_new_document_v1(b"not XML");
        assert!(matches!(
            completed.envelope(),
            CmlOpenEnvelopeV1::Refused { refusal, .. }
                if refusal.reason() == CmlImportRefusalReasonV1::InvalidXml
        ));
        assert!(completed.take_document_cdml().is_none());
    }

    #[test]
    fn unsupported_cml_keeps_its_closed_refusal_reason() {
        let completed = open_cml_new_document_v1(b"<cml xmlns=\"urn:unsupported\"/>");
        assert!(matches!(
            completed.envelope(),
            CmlOpenEnvelopeV1::Refused { refusal, .. }
                if refusal.reason() == CmlImportRefusalReasonV1::NamespaceUnsupported
        ));
    }

    #[test]
    fn exact_import_envelope_is_measured_before_cdml_handoff() {
        let mut completed = open_cml_new_document_v1(CML.as_bytes());
        let json = canonical_cml_open_envelope_json_v1(completed.envelope())
            .expect("admitted envelope fits the fixed response budget");
        assert!(
            String::from_utf8(json)
                .expect("JSON is UTF-8")
                .contains(CML_OPEN_OPERATION_ID_V1)
        );
        assert!(completed.take_document_cdml().is_some());
    }
}
