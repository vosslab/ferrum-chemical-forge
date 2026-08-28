//! Descriptor-dispatched, atomic new-document interchange admission.

use std::{io::Read, path::Path};

use ferrum_chemistry::{
    CdxmlDecodedDocumentV1, CdxmlDecodedRecordV1, CdxmlLossCategoryV1, CdxmlRefusalReasonV1,
    ChemEngine, CmlDecodedRecordV1, CmlRefusalReasonV1, Coordinates, InterchangeRecordV1, MolAtom,
    MolGraph, Point2 as ChemistryPoint2, decode_cdxml_bytes_v1, decode_cml_bytes_v1,
    interchange_record_from_sdf_v1, validate_sdf_input,
};
use ferrum_document::artifact_publication_v1::RetainedSourceFileGuardV1;
use ferrum_document::{
    DocumentIngressErrorV1, DocumentSession, InterchangeRecordBuildErrorV1,
    PreparedSessionTransitionV1, SessionOperation, SessionOperationTransitionRequestV1,
    SessionOperationV1, TransitionAuthorizationV1, build_cdxml_record_batch_insertion,
    build_interchange_record_batch_insertion_v1, read_regular_file_with_origin_with_budget,
};
use ferrum_geometry::{MoleculePlacementV1, Point2};

use crate::interchange_import_v1::{
    InterchangeDecoderKeyV1, InterchangeFormatDescriptorV1, InterchangeImportRefusalReasonV1,
    InterchangeImportRefusalV1,
};
use crate::protocol::runtime::ChemistryRuntimeV1;
use crate::protocol::{
    DocumentInterchangeImportLossReportV1, DocumentInterchangeImportSummaryV1,
    DocumentInterchangeLossCategoryV1, DocumentInterchangeProvenanceV1,
};

const IMPORT_BOND_LENGTH_PT_V1: f64 = 40.0;

/// Private capability resolver for descriptor-selected installed-wheel work.
///
/// The caller cannot choose a decoder or request a chemistry runtime.  The
/// generic interchange core calls this only for descriptors that require one.
#[cfg(feature = "python-binding")]
pub(crate) trait LocalInterchangeRuntimeResolverV1 {
    fn chemistry_runtime(
        &self,
    ) -> Result<
        crate::protocol::runtime::TrustedLibraryChemistryRuntimeV1,
        InterchangeImportRefusalV1,
    >;
}

pub(crate) struct AdmittedInterchangeSourceV1 {
    bytes: Vec<u8>,
    source_kind: crate::protocol::DocumentInterchangeSourceKindV1,
    retained_source: Option<RetainedSourceFileGuardV1>,
}

impl AdmittedInterchangeSourceV1 {
    #[must_use]
    pub(crate) fn source_kind(&self) -> crate::protocol::DocumentInterchangeSourceKindV1 {
        self.source_kind
    }

    #[must_use]
    pub(crate) fn retained_source(&self) -> Option<&RetainedSourceFileGuardV1> {
        self.retained_source.as_ref()
    }

    fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

pub(crate) enum InterchangeSourceInputV1<'a> {
    RequestText(&'a [u8]),
    StandardInput(&'a mut dyn Read),
    RegularFile(&'a Path),
}

pub(crate) fn admit_interchange_source_v1(
    descriptor: &'static InterchangeFormatDescriptorV1,
    input: InterchangeSourceInputV1<'_>,
) -> Result<AdmittedInterchangeSourceV1, InterchangeImportRefusalV1> {
    let limit = descriptor.limits().max_source_bytes();
    match input {
        InterchangeSourceInputV1::RequestText(bytes) => admit_bytes(
            bytes,
            limit,
            crate::protocol::DocumentInterchangeSourceKindV1::RequestText,
        ),
        InterchangeSourceInputV1::StandardInput(reader) => {
            let mut bytes = Vec::new();
            reader
                .take(limit.saturating_add(1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| generic_source_refusal())?;
            admit_bytes(
                &bytes,
                limit,
                crate::protocol::DocumentInterchangeSourceKindV1::StandardInput,
            )
        }
        InterchangeSourceInputV1::RegularFile(path) => {
            let (bytes, retained_source) =
                read_regular_file_with_origin_with_budget(path, limit).map_err(map_source_error)?;
            Ok(AdmittedInterchangeSourceV1 {
                bytes,
                source_kind: crate::protocol::DocumentInterchangeSourceKindV1::RegularFile,
                retained_source: Some(retained_source),
            })
        }
    }
}

/// Read one descriptor-authorized regular local interchange source as UTF-8.
///
/// This is intentionally only a source-admission boundary.  It does not select
/// a decoder, acquire a chemistry runtime, create a document, or mutate one;
/// callers that need those operations use their distinct typed boundaries.
#[cfg(feature = "python-binding")]
pub(crate) fn read_local_interchange_utf8_source_v1(
    descriptor: &'static InterchangeFormatDescriptorV1,
    path: &Path,
) -> Result<String, InterchangeImportRefusalV1> {
    let source =
        admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(path))?;
    std::str::from_utf8(source.bytes())
        .map(str::to_owned)
        .map_err(|_| {
            InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InvalidUtf8)
        })
}

fn admit_bytes(
    bytes: &[u8],
    limit: usize,
    source_kind: crate::protocol::DocumentInterchangeSourceKindV1,
) -> Result<AdmittedInterchangeSourceV1, InterchangeImportRefusalV1> {
    if bytes.len() > limit {
        return Err(InterchangeImportRefusalV1::for_reason(
            InterchangeImportRefusalReasonV1::InputBytesLimit,
        ));
    }
    Ok(AdmittedInterchangeSourceV1 {
        bytes: bytes.to_vec(),
        source_kind,
        retained_source: None,
    })
}

fn map_source_error(error: DocumentIngressErrorV1) -> InterchangeImportRefusalV1 {
    match error {
        DocumentIngressErrorV1::ByteLimitExceeded { .. } => InterchangeImportRefusalV1::for_reason(
            InterchangeImportRefusalReasonV1::InputBytesLimit,
        ),
        _ => generic_source_refusal(),
    }
}

const fn generic_source_refusal() -> InterchangeImportRefusalV1 {
    InterchangeImportRefusalV1::for_reason(
        InterchangeImportRefusalReasonV1::CandidateValidationFailed,
    )
}

pub(crate) struct PreparedInterchangeNewDocumentV1 {
    session: DocumentSession,
    transition: PreparedSessionTransitionV1,
    facts: InterchangeImportSummaryFacts,
}

impl PreparedInterchangeNewDocumentV1 {
    pub(crate) fn commit_and_take_session(
        mut self,
    ) -> Result<(DocumentSession, DocumentInterchangeImportSummaryV1), InterchangeImportRefusalV1>
    {
        self.session
            .commit_session_operation_transition_v1(&mut self.transition)
            .map_err(|_| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::InternalFailure,
                )
            })?;
        let snapshot = self.session.snapshot().map_err(|_| {
            InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::InternalFailure,
            )
        })?;
        require_clean_document_render(&self.session, snapshot.revision())?;
        Ok((self.session, summary(self.facts, &snapshot)))
    }
}

/// Verify a newly committed private candidate is publishable through the one
/// authoritative document-rendering path.
///
/// Interchange callers only receive a session after this gate succeeds.  A
/// failed candidate remains owned by this preparation object and is dropped;
/// no CLI response or desktop prepared tab can observe its committed state.
fn require_clean_document_render(
    session: &DocumentSession,
    revision: u64,
) -> Result<(), InterchangeImportRefusalV1> {
    let observation = session
        .observe_render_v2(revision)
        .map_err(|_| generic_source_refusal())?;
    if observation.resolved().suppression().is_some()
        || observation
            .resolved()
            .molecule_plans()
            .iter()
            .any(|plan| !plan.issues().is_empty() || !plan.member_issues().is_empty())
    {
        return Err(generic_source_refusal());
    }
    Ok(())
}

/// Decode and prepare a new document through the descriptor-selected adapter.
/// The only decoder/runtime branch lives here; every successful decoder feeds
/// the same typed record batch and one create/commit transaction.
pub(crate) fn prepare_interchange_new_document_v1<R: ChemistryRuntimeV1>(
    descriptor: &'static InterchangeFormatDescriptorV1,
    source: &AdmittedInterchangeSourceV1,
    runtime: &R,
    provenance: DocumentInterchangeProvenanceV1,
) -> Result<PreparedInterchangeNewDocumentV1, InterchangeImportRefusalV1> {
    match descriptor.decoder() {
        InterchangeDecoderKeyV1::CmlSimpleMolecule => {
            let records = decode_cml_simple_molecule_records_v1(source.bytes())?;
            prepare_records(
                descriptor,
                records,
                &ferrum_chemistry::UnavailableChemEngine,
                provenance,
                vec![DocumentInterchangeLossCategoryV1::LexicalSyntax],
            )
        }
        InterchangeDecoderKeyV1::CdxmlSimpleMolecule => {
            let decoded = decode_cdxml_simple_molecule_document_v1(source.bytes())?;
            prepare_cdxml_records(
                descriptor,
                decoded.records(),
                &ferrum_chemistry::UnavailableChemEngine,
                provenance,
                map_cdxml_losses(decoded.declared_losses()),
            )
        }
        InterchangeDecoderKeyV1::Sdf => runtime
            .with_engine(|engine| {
                Ok(
                    decode_sdf_records(engine, source.bytes()).and_then(|records| {
                        prepare_records(descriptor, records, engine, provenance, Vec::new())
                    }),
                )
            })
            .map_err(|_| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::ChemistryRuntimeUnavailable,
                )
            })?,
    }
}

/// Convert CML/CML2 through the same native document-admission transaction as local ingress.
///
/// This keeps CML decoding, record lowering, durable identity allocation, and canonical CDML
/// serialization in their established owners without acquiring a chemistry runtime.
pub(crate) fn convert_cml_simple_molecule_to_cdml_v1(
    source: &[u8],
) -> Result<(String, usize), InterchangeImportRefusalV1> {
    let descriptor = crate::InterchangeFormatRegistryV1::lookup_input_alias("cml")?;
    let source =
        admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RequestText(source))?;
    let provenance = DocumentInterchangeProvenanceV1 {
        format_id: descriptor.format_id().to_owned(),
        profile_id: descriptor.profile_id().to_owned(),
        source_kind: source.source_kind(),
    };
    let prepared = prepare_interchange_new_document_v1(
        descriptor,
        &source,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        provenance,
    )?;
    let (session, summary) = prepared.commit_and_take_session()?;
    let snapshot = session.snapshot().map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    Ok((
        snapshot.cdml().to_owned(),
        summary.imported_record_count as usize,
    ))
}

/// Admit and prepare one regular local interchange file through a
/// descriptor-owned decoder and capability policy.
///
/// This is the only installed-wheel local-file preparation boundary.  Frontend
/// adapters supply an authenticated descriptor identity and never branch on
/// decoder keys or acquire a runtime themselves.
#[cfg(feature = "python-binding")]
pub(crate) fn prepare_local_interchange_new_document_v1<R: LocalInterchangeRuntimeResolverV1>(
    descriptor: &'static InterchangeFormatDescriptorV1,
    path: &Path,
    runtime_resolver: &R,
) -> Result<
    (
        AdmittedInterchangeSourceV1,
        PreparedInterchangeNewDocumentV1,
    ),
    InterchangeImportRefusalV1,
> {
    let source =
        admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(path))?;
    let provenance = crate::protocol::DocumentInterchangeProvenanceV1 {
        format_id: descriptor.format_id().to_owned(),
        profile_id: descriptor.profile_id().to_owned(),
        source_kind: source.source_kind(),
    };
    let prepared = match descriptor.decoder() {
        InterchangeDecoderKeyV1::CmlSimpleMolecule
        | InterchangeDecoderKeyV1::CdxmlSimpleMolecule => prepare_interchange_new_document_v1(
            descriptor,
            &source,
            &crate::protocol::runtime::NoChemistryRuntimeV1,
            provenance,
        ),
        InterchangeDecoderKeyV1::Sdf => {
            let runtime = runtime_resolver.chemistry_runtime()?;
            prepare_interchange_new_document_v1(descriptor, &source, &runtime, provenance)
        }
    }?;
    Ok((source, prepared))
}

/// Decode the registry-owned CML simple-molecule profile into reusable records.
///
/// Document import and chemistry conversion share this bridge so atom, bond,
/// and coordinate lowering cannot diverge between their two public surfaces.
pub(crate) fn decode_cml_simple_molecule_document_v1(
    source: &[u8],
) -> Result<ferrum_chemistry::CmlDecodedDocumentV1, InterchangeImportRefusalV1> {
    decode_cml_bytes_v1(source).map_err(|error| {
        InterchangeImportRefusalV1::for_reason(map_cml_decoder_reason(error.reason()))
    })
}

pub(crate) fn decode_cml_simple_molecule_records_v1(
    source: &[u8],
) -> Result<Vec<InterchangeRecordV1>, InterchangeImportRefusalV1> {
    decode_cml_simple_molecule_document_v1(source)?
        .records()
        .iter()
        .map(convert_cml_record)
        .collect()
}

/// Decode the registry-owned CDXML simple-molecule profile into owned records.
///
/// The chemistry decoder owns the closed XML profile. This adapter retains
/// only its typed records and loss categories, then shares the one document
/// preparation transaction used by the other interchange profiles.
pub(crate) fn decode_cdxml_simple_molecule_document_v1(
    source: &[u8],
) -> Result<CdxmlDecodedDocumentV1, InterchangeImportRefusalV1> {
    decode_cdxml_bytes_v1(source).map_err(|error| {
        InterchangeImportRefusalV1::for_reason(map_cdxml_decoder_reason(error.reason()))
    })
}

fn map_cdxml_losses(losses: &[CdxmlLossCategoryV1]) -> Vec<DocumentInterchangeLossCategoryV1> {
    losses
        .iter()
        .map(|loss| match loss {
            CdxmlLossCategoryV1::LexicalSyntax => DocumentInterchangeLossCategoryV1::LexicalSyntax,
            CdxmlLossCategoryV1::DocumentViewMetadata => {
                DocumentInterchangeLossCategoryV1::DocumentViewMetadata
            }
        })
        .collect()
}

fn decode_sdf_records(
    engine: &dyn ChemEngine,
    source: &[u8],
) -> Result<Vec<InterchangeRecordV1>, InterchangeImportRefusalV1> {
    let text = std::str::from_utf8(source).map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InvalidUtf8)
    })?;
    validate_sdf_input(text).map_err(|_| generic_source_refusal())?;
    engine
        .sdf_to_records(text)
        .map(|records| {
            records
                .into_iter()
                .map(interchange_record_from_sdf_v1)
                .collect()
        })
        .map_err(|_| generic_source_refusal())
}

fn prepare_records(
    descriptor: &'static InterchangeFormatDescriptorV1,
    records: Vec<InterchangeRecordV1>,
    engine: &dyn ChemEngine,
    provenance: DocumentInterchangeProvenanceV1,
    dropped_categories: Vec<DocumentInterchangeLossCategoryV1>,
) -> Result<PreparedInterchangeNewDocumentV1, InterchangeImportRefusalV1> {
    let source_record_count = records.len();
    let atom_count = records
        .iter()
        .map(|record| record.molecule().atoms().len())
        .sum();
    let bond_count = records
        .iter()
        .map(|record| record.molecule().bonds().len())
        .sum();
    prepare_batch(
        descriptor,
        source_record_count,
        atom_count,
        bond_count,
        provenance,
        dropped_categories,
        |placement| {
            build_interchange_record_batch_insertion_v1(engine, &records, placement)
                .map_err(map_record_build_error)
        },
    )
}

/// Prepare decoded CDXML records through the document-owned presentation adapter.
///
/// The adapter is the only conversion from the source-specific carrier to
/// durable document presentation.  This API layer deliberately retains the
/// exact shared session, placement, count, and publication transaction.
fn prepare_cdxml_records(
    descriptor: &'static InterchangeFormatDescriptorV1,
    records: &[CdxmlDecodedRecordV1],
    engine: &dyn ChemEngine,
    provenance: DocumentInterchangeProvenanceV1,
    dropped_categories: Vec<DocumentInterchangeLossCategoryV1>,
) -> Result<PreparedInterchangeNewDocumentV1, InterchangeImportRefusalV1> {
    let source_record_count = records.len();
    let atom_count = records
        .iter()
        .map(|record| record.record().molecule().atoms().len())
        .sum();
    let bond_count = records
        .iter()
        .map(|record| record.record().molecule().bonds().len())
        .sum();
    prepare_batch(
        descriptor,
        source_record_count,
        atom_count,
        bond_count,
        provenance,
        dropped_categories,
        |placement| {
            build_cdxml_record_batch_insertion(engine, records, placement)
                .map_err(|_| generic_source_refusal())
        },
    )
}

fn prepare_batch<F>(
    descriptor: &'static InterchangeFormatDescriptorV1,
    source_record_count: usize,
    atom_count: usize,
    bond_count: usize,
    provenance: DocumentInterchangeProvenanceV1,
    dropped_categories: Vec<DocumentInterchangeLossCategoryV1>,
    build_batch: F,
) -> Result<PreparedInterchangeNewDocumentV1, InterchangeImportRefusalV1>
where
    F: FnOnce(
        MoleculePlacementV1,
    ) -> Result<
        ferrum_document::InterchangeRecordBatchInsertionV1,
        InterchangeImportRefusalV1,
    >,
{
    let mut session = DocumentSession::create_empty_document_v1().map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    let baseline = session.snapshot().map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    let page = session
        .observe(baseline.revision())
        .map_err(|_| {
            InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::InternalFailure,
            )
        })?
        .projection()
        .paper_layout()
        .page();
    let placement = MoleculePlacementV1::new(
        IMPORT_BOND_LENGTH_PT_V1,
        Point2::new(
            (page.scene_left() + page.scene_right()) / 2.0,
            (page.scene_top() + page.scene_bottom()) / 2.0,
        )
        .expect("the document projection publishes finite page bounds"),
    )
    .map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    let batch = build_batch(placement)?;
    let transition = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            baseline.revision(),
            SessionOperation::V1(SessionOperationV1::InsertInterchangeRecordBatchV1(batch)),
            TransitionAuthorizationV1::none(),
        ))
        .map_err(|_| generic_source_refusal())?;
    Ok(PreparedInterchangeNewDocumentV1 {
        session,
        transition,
        facts: InterchangeImportSummaryFacts {
            descriptor,
            provenance,
            imported_record_count: source_record_count,
            atom_count,
            bond_count,
            dropped_categories,
        },
    })
}

/// Internal facts collected while preparing one interchange document.
///
/// Keeping this ownership-preserving aggregation private makes the summary
/// construction boundary self-describing without changing its protocol DTO.
struct InterchangeImportSummaryFacts {
    descriptor: &'static InterchangeFormatDescriptorV1,
    provenance: DocumentInterchangeProvenanceV1,
    imported_record_count: usize,
    atom_count: usize,
    bond_count: usize,
    dropped_categories: Vec<DocumentInterchangeLossCategoryV1>,
}

fn summary(
    facts: InterchangeImportSummaryFacts,
    snapshot: &ferrum_document::DocumentSnapshot,
) -> DocumentInterchangeImportSummaryV1 {
    DocumentInterchangeImportSummaryV1 {
        format_id: facts.descriptor.format_id().to_owned(),
        profile_id: facts.descriptor.profile_id().to_owned(),
        imported_record_count: facts.imported_record_count as u32,
        atom_count: facts.atom_count as u32,
        bond_count: facts.bond_count as u32,
        document_revision: snapshot.revision(),
        document_digest_hex: snapshot
            .digest()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
        provenance: facts.provenance,
        loss_report: DocumentInterchangeImportLossReportV1 {
            source_identifiers_reallocated: true,
            dropped_categories: facts.dropped_categories,
        },
    }
}

fn convert_cml_record(
    record: &CmlDecodedRecordV1,
) -> Result<InterchangeRecordV1, InterchangeImportRefusalV1> {
    let atoms = record
        .atoms()
        .iter()
        .map(|atom| {
            MolAtom::new(
                atom.element(),
                atom.formal_charge(),
                atom.isotope(),
                None,
                false,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| generic_source_refusal())?;
    let bonds = record
        .bonds()
        .iter()
        .map(|bond| bond.to_mol_bond())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| generic_source_refusal())?;
    let coordinates = record
        .atoms()
        .iter()
        .map(|atom| {
            ChemistryPoint2::new(
                transform_coordinate(atom.x2())?,
                transform_coordinate(-atom.y2())?,
            )
            .map_err(|_| generic_source_refusal())
        })
        .collect::<Result<Vec<_>, _>>()?;
    let graph = MolGraph::new(atoms, bonds, Some(Coordinates::new(coordinates)))
        .map_err(|_| generic_source_refusal())?;
    Ok(InterchangeRecordV1::new(
        graph,
        record.source_molecule_id().map(str::to_owned),
        Vec::new(),
    ))
}

fn transform_coordinate(value: f64) -> Result<f64, InterchangeImportRefusalV1> {
    let value = 30.0 * value;
    if !value.is_finite() || value.abs() > 3_000_000.0 {
        return Err(generic_source_refusal());
    }
    Ok(value)
}

fn map_record_build_error(_error: InterchangeRecordBuildErrorV1) -> InterchangeImportRefusalV1 {
    generic_source_refusal()
}

fn map_cml_decoder_reason(reason: CmlRefusalReasonV1) -> InterchangeImportRefusalReasonV1 {
    match reason {
        CmlRefusalReasonV1::InvalidUtf8 => InterchangeImportRefusalReasonV1::InvalidUtf8,
        CmlRefusalReasonV1::InvalidXml => InterchangeImportRefusalReasonV1::InvalidXml,
        CmlRefusalReasonV1::InvalidXmlDeclaration => {
            InterchangeImportRefusalReasonV1::InvalidXmlDeclaration
        }
        CmlRefusalReasonV1::UnexpectedXmlText => {
            InterchangeImportRefusalReasonV1::UnexpectedXmlText
        }
        CmlRefusalReasonV1::UnexpectedXmlNode => {
            InterchangeImportRefusalReasonV1::UnexpectedXmlNode
        }
        CmlRefusalReasonV1::InvalidScalar => InterchangeImportRefusalReasonV1::InvalidScalar,
        CmlRefusalReasonV1::InvalidCoordinate => {
            InterchangeImportRefusalReasonV1::InvalidCoordinate
        }
        CmlRefusalReasonV1::CoordinateNotFinite => {
            InterchangeImportRefusalReasonV1::CoordinateNotFinite
        }
        CmlRefusalReasonV1::CoordinateOutOfRange => {
            InterchangeImportRefusalReasonV1::CoordinateOutOfRange
        }
        CmlRefusalReasonV1::DuplicateSourceId => {
            InterchangeImportRefusalReasonV1::DuplicateSourceId
        }
        CmlRefusalReasonV1::DuplicateAtomId => InterchangeImportRefusalReasonV1::DuplicateAtomId,
        CmlRefusalReasonV1::DanglingBond => InterchangeImportRefusalReasonV1::DanglingBond,
        CmlRefusalReasonV1::SelfBond => InterchangeImportRefusalReasonV1::SelfBond,
        CmlRefusalReasonV1::DuplicateBond => InterchangeImportRefusalReasonV1::DuplicateBond,
        CmlRefusalReasonV1::InvalidGraph => InterchangeImportRefusalReasonV1::InvalidGraph,
        CmlRefusalReasonV1::EmptyDocument => InterchangeImportRefusalReasonV1::EmptyDocument,
        CmlRefusalReasonV1::NamespaceUnsupported => {
            InterchangeImportRefusalReasonV1::NamespaceUnsupported
        }
        CmlRefusalReasonV1::RootUnsupported => InterchangeImportRefusalReasonV1::RootUnsupported,
        CmlRefusalReasonV1::ProfileMismatch => InterchangeImportRefusalReasonV1::ProfileMismatch,
        CmlRefusalReasonV1::AttributeUnsupported => {
            InterchangeImportRefusalReasonV1::AttributeUnsupported
        }
        CmlRefusalReasonV1::ArrayAttributeUnsupported => {
            InterchangeImportRefusalReasonV1::ArrayAttributeUnsupported
        }
        CmlRefusalReasonV1::UnrepresentedSemanticFact => {
            InterchangeImportRefusalReasonV1::UnrepresentedSemanticFact
        }
        CmlRefusalReasonV1::DtdForbidden => InterchangeImportRefusalReasonV1::DtdForbidden,
        CmlRefusalReasonV1::EntityForbidden => InterchangeImportRefusalReasonV1::EntityForbidden,
        CmlRefusalReasonV1::ExternalResourceForbidden => {
            InterchangeImportRefusalReasonV1::ExternalResourceForbidden
        }
        CmlRefusalReasonV1::XincludeForbidden => {
            InterchangeImportRefusalReasonV1::XincludeForbidden
        }
        CmlRefusalReasonV1::StylesheetForbidden => {
            InterchangeImportRefusalReasonV1::StylesheetForbidden
        }
        CmlRefusalReasonV1::InputBytesLimit => InterchangeImportRefusalReasonV1::InputBytesLimit,
        CmlRefusalReasonV1::XmlTextBytesLimit => {
            InterchangeImportRefusalReasonV1::XmlTextBytesLimit
        }
        CmlRefusalReasonV1::XmlDeclarationLimit => {
            InterchangeImportRefusalReasonV1::XmlDeclarationLimit
        }
        CmlRefusalReasonV1::CommentBytesLimit => {
            InterchangeImportRefusalReasonV1::CommentBytesLimit
        }
        CmlRefusalReasonV1::PiBytesLimit => InterchangeImportRefusalReasonV1::PiBytesLimit,
        CmlRefusalReasonV1::XmlElementLimit => InterchangeImportRefusalReasonV1::XmlElementLimit,
        CmlRefusalReasonV1::XmlDepthLimit => InterchangeImportRefusalReasonV1::XmlDepthLimit,
        CmlRefusalReasonV1::XmlAttributeLimit => {
            InterchangeImportRefusalReasonV1::XmlAttributeLimit
        }
        CmlRefusalReasonV1::AttributeValueLimit => {
            InterchangeImportRefusalReasonV1::AttributeValueLimit
        }
        CmlRefusalReasonV1::RecordLimit => InterchangeImportRefusalReasonV1::RecordLimit,
        CmlRefusalReasonV1::AtomsPerRecordLimit => {
            InterchangeImportRefusalReasonV1::AtomsPerRecordLimit
        }
        CmlRefusalReasonV1::AtomLimit => InterchangeImportRefusalReasonV1::AtomLimit,
        CmlRefusalReasonV1::BondsPerRecordLimit => {
            InterchangeImportRefusalReasonV1::BondsPerRecordLimit
        }
        CmlRefusalReasonV1::BondLimit => InterchangeImportRefusalReasonV1::BondLimit,
        CmlRefusalReasonV1::SourceIdMapLimit => InterchangeImportRefusalReasonV1::SourceIdMapLimit,
        CmlRefusalReasonV1::IdentifierBytesLimit => {
            InterchangeImportRefusalReasonV1::IdentifierBytesLimit
        }
        CmlRefusalReasonV1::InternalFailure => InterchangeImportRefusalReasonV1::InternalFailure,
    }
}

fn map_cdxml_decoder_reason(reason: CdxmlRefusalReasonV1) -> InterchangeImportRefusalReasonV1 {
    match reason {
        CdxmlRefusalReasonV1::InvalidUtf8 => InterchangeImportRefusalReasonV1::InvalidUtf8,
        CdxmlRefusalReasonV1::InvalidXml => InterchangeImportRefusalReasonV1::InvalidXml,
        CdxmlRefusalReasonV1::InvalidXmlDeclaration => {
            InterchangeImportRefusalReasonV1::InvalidXmlDeclaration
        }
        CdxmlRefusalReasonV1::UnexpectedXmlText => {
            InterchangeImportRefusalReasonV1::UnexpectedXmlText
        }
        CdxmlRefusalReasonV1::UnexpectedXmlNode => {
            InterchangeImportRefusalReasonV1::UnexpectedXmlNode
        }
        CdxmlRefusalReasonV1::InvalidScalar => InterchangeImportRefusalReasonV1::InvalidScalar,
        CdxmlRefusalReasonV1::InvalidCoordinate => {
            InterchangeImportRefusalReasonV1::InvalidCoordinate
        }
        CdxmlRefusalReasonV1::CoordinateNotFinite => {
            InterchangeImportRefusalReasonV1::CoordinateNotFinite
        }
        CdxmlRefusalReasonV1::CoordinateOutOfRange => {
            InterchangeImportRefusalReasonV1::CoordinateOutOfRange
        }
        CdxmlRefusalReasonV1::DuplicateSourceId => {
            InterchangeImportRefusalReasonV1::DuplicateSourceId
        }
        CdxmlRefusalReasonV1::DuplicateAtomId => InterchangeImportRefusalReasonV1::DuplicateAtomId,
        CdxmlRefusalReasonV1::DanglingBond => InterchangeImportRefusalReasonV1::DanglingBond,
        CdxmlRefusalReasonV1::SelfBond => InterchangeImportRefusalReasonV1::SelfBond,
        CdxmlRefusalReasonV1::DuplicateBond => InterchangeImportRefusalReasonV1::DuplicateBond,
        CdxmlRefusalReasonV1::InvalidGraph => InterchangeImportRefusalReasonV1::InvalidGraph,
        CdxmlRefusalReasonV1::EmptyDocument => InterchangeImportRefusalReasonV1::EmptyDocument,
        CdxmlRefusalReasonV1::NamespaceUnsupported => {
            InterchangeImportRefusalReasonV1::NamespaceUnsupported
        }
        CdxmlRefusalReasonV1::RootUnsupported => InterchangeImportRefusalReasonV1::RootUnsupported,
        CdxmlRefusalReasonV1::AttributeUnsupported => {
            InterchangeImportRefusalReasonV1::AttributeUnsupported
        }
        CdxmlRefusalReasonV1::UnrepresentedSemanticFact => {
            InterchangeImportRefusalReasonV1::UnrepresentedSemanticFact
        }
        CdxmlRefusalReasonV1::DtdForbidden => InterchangeImportRefusalReasonV1::DtdForbidden,
        CdxmlRefusalReasonV1::EntityForbidden => InterchangeImportRefusalReasonV1::EntityForbidden,
        CdxmlRefusalReasonV1::InputBytesLimit => InterchangeImportRefusalReasonV1::InputBytesLimit,
        CdxmlRefusalReasonV1::XmlElementLimit => InterchangeImportRefusalReasonV1::XmlElementLimit,
        CdxmlRefusalReasonV1::AttributeValueLimit => {
            InterchangeImportRefusalReasonV1::AttributeValueLimit
        }
        CdxmlRefusalReasonV1::RecordLimit => InterchangeImportRefusalReasonV1::RecordLimit,
        CdxmlRefusalReasonV1::AtomsPerRecordLimit => {
            InterchangeImportRefusalReasonV1::AtomsPerRecordLimit
        }
        CdxmlRefusalReasonV1::BondsPerRecordLimit => {
            InterchangeImportRefusalReasonV1::BondsPerRecordLimit
        }
        CdxmlRefusalReasonV1::IdentifierBytesLimit => {
            InterchangeImportRefusalReasonV1::IdentifierBytesLimit
        }
        CdxmlRefusalReasonV1::InternalFailure => InterchangeImportRefusalReasonV1::InternalFailure,
    }
}

#[cfg(test)]
#[path = "document_interchange_import_tests.rs"]
mod tests;
