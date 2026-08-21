//! Descriptor-dispatched, atomic new-document interchange admission.

use std::{io::Read, path::Path};

use ferrum_chemistry::{
    BondOrder, ChemEngine, CmlDecodedRecordV1, CmlRefusalReasonV1, Coordinates,
    InterchangeRecordV1, MolAtom, MolBond, MolGraph, Point2 as ChemistryPoint2,
    decode_cml_bytes_v1, interchange_record_from_sdf_v1, validate_sdf_input,
};
use ferrum_document::artifact_publication_v1::RetainedSourceFileGuardV1;
use ferrum_document::{
    DocumentIngressErrorV1, DocumentSession, InterchangeRecordBuildErrorV1,
    PendingCreateInterchangeBatchV1, build_interchange_record_batch_insertion_v1,
    read_regular_file_with_origin_with_budget,
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
    baseline_revision: u64,
    pending: PendingCreateInterchangeBatchV1,
    summary: DocumentInterchangeImportSummaryV1,
}

impl PreparedInterchangeNewDocumentV1 {
    #[must_use]
    pub(crate) fn summary(&self) -> &DocumentInterchangeImportSummaryV1 {
        &self.summary
    }

    pub(crate) fn commit_and_take_session(
        mut self,
    ) -> Result<(DocumentSession, DocumentInterchangeImportSummaryV1), InterchangeImportRefusalV1>
    {
        self.session
            .commit_create_interchange_records_v1(self.baseline_revision, &mut self.pending)
            .map_err(|_| {
                InterchangeImportRefusalV1::for_reason(
                    InterchangeImportRefusalReasonV1::InternalFailure,
                )
            })?;
        Ok((self.session, self.summary))
    }
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
            let records = decode_cml_records(source.bytes())?;
            prepare_records(
                descriptor,
                records,
                &ferrum_chemistry::UnavailableChemEngine,
                provenance,
                vec![DocumentInterchangeLossCategoryV1::LexicalSyntax],
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
        InterchangeDecoderKeyV1::CmlSimpleMolecule => prepare_interchange_new_document_v1(
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

fn decode_cml_records(
    source: &[u8],
) -> Result<Vec<InterchangeRecordV1>, InterchangeImportRefusalV1> {
    decode_cml_bytes_v1(source)
        .map_err(|error| {
            InterchangeImportRefusalV1::for_reason(map_cml_decoder_reason(error.reason()))
        })?
        .records()
        .iter()
        .map(convert_cml_record)
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
    let placement = MoleculePlacementV1::new(
        IMPORT_BOND_LENGTH_PT_V1,
        Point2::new(0.0, 0.0).expect("finite origin"),
    )
    .map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    let batch = build_interchange_record_batch_insertion_v1(engine, &records, placement)
        .map_err(map_record_build_error)?;
    let mut session = DocumentSession::create_empty_document_v1().map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    let baseline = session.snapshot().map_err(|_| {
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InternalFailure)
    })?;
    let pending = session
        .prepare_create_interchange_records_v1(baseline.revision(), &batch)
        .map_err(|_| generic_source_refusal())?;
    let (document_revision, digest) =
        pending.candidate_revision_and_digest_v1().ok_or_else(|| {
            InterchangeImportRefusalV1::for_reason(
                InterchangeImportRefusalReasonV1::InternalFailure,
            )
        })?;
    Ok(PreparedInterchangeNewDocumentV1 {
        session,
        baseline_revision: baseline.revision(),
        pending,
        summary: summary(InterchangeImportSummaryFactsV1 {
            descriptor,
            provenance,
            imported_record_count: source_record_count,
            atom_count,
            bond_count,
            document_revision,
            digest,
            dropped_categories,
        }),
    })
}

/// Internal facts collected while preparing one interchange document.
///
/// Keeping this ownership-preserving aggregation private makes the summary
/// construction boundary self-describing without changing its protocol DTO.
struct InterchangeImportSummaryFactsV1 {
    descriptor: &'static InterchangeFormatDescriptorV1,
    provenance: DocumentInterchangeProvenanceV1,
    imported_record_count: usize,
    atom_count: usize,
    bond_count: usize,
    document_revision: u64,
    digest: [u8; 32],
    dropped_categories: Vec<DocumentInterchangeLossCategoryV1>,
}

fn summary(facts: InterchangeImportSummaryFactsV1) -> DocumentInterchangeImportSummaryV1 {
    DocumentInterchangeImportSummaryV1 {
        format_id: facts.descriptor.format_id().to_owned(),
        profile_id: facts.descriptor.profile_id().to_owned(),
        imported_record_count: facts.imported_record_count as u32,
        atom_count: facts.atom_count as u32,
        bond_count: facts.bond_count as u32,
        document_revision: facts.document_revision,
        document_digest_hex: facts
            .digest
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
        .map(|bond| match bond.order() {
            BondOrder::Single | BondOrder::Double | BondOrder::Triple => {
                Ok(MolBond::new(bond.start(), bond.end(), bond.order(), false))
            }
            _ => Err(generic_source_refusal()),
        })
        .collect::<Result<Vec<_>, _>>()?;
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
        CmlRefusalReasonV1::InputBytesLimit => InterchangeImportRefusalReasonV1::InputBytesLimit,
        CmlRefusalReasonV1::InternalFailure => InterchangeImportRefusalReasonV1::InternalFailure,
        _ => InterchangeImportRefusalReasonV1::CandidateValidationFailed,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Cursor};

    use super::*;

    fn temporary_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "ferrum-interchange-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("monotonic wall clock")
                .as_nanos(),
        ))
    }

    #[test]
    fn every_registered_descriptor_refuses_sources_above_its_limit() {
        enum SourceKind {
            RequestText,
            StandardInput,
            RegularFile,
        }

        for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
            let over_limit = vec![b'x'; descriptor.limits().max_source_bytes() + 1];
            let path = temporary_path(descriptor.format_id());
            fs::write(&path, &over_limit).expect("write bounded source");
            for source_kind in [
                SourceKind::RequestText,
                SourceKind::StandardInput,
                SourceKind::RegularFile,
            ] {
                let result = match source_kind {
                    SourceKind::RequestText => admit_interchange_source_v1(
                        descriptor,
                        InterchangeSourceInputV1::RequestText(&over_limit),
                    ),
                    SourceKind::StandardInput => {
                        let mut stdin = Cursor::new(&over_limit);
                        admit_interchange_source_v1(
                            descriptor,
                            InterchangeSourceInputV1::StandardInput(&mut stdin),
                        )
                    }
                    SourceKind::RegularFile => admit_interchange_source_v1(
                        descriptor,
                        InterchangeSourceInputV1::RegularFile(&path),
                    ),
                };
                assert!(matches!(
                    result,
                    Err(refusal) if refusal == InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InputBytesLimit)
                ));
            }
            fs::remove_file(path).expect("remove bounded source");
        }
    }

    #[test]
    fn every_registered_descriptor_preserves_admitted_bytes_and_provenance() {
        for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
            let bytes = format!("{} source bytes", descriptor.format_id()).into_bytes();
            assert!(bytes.len() <= descriptor.limits().max_source_bytes());
            assert!(!descriptor.profile_id().is_empty());

            let request = admit_interchange_source_v1(
                descriptor,
                InterchangeSourceInputV1::RequestText(&bytes),
            )
            .expect("request source should be admitted");
            assert_eq!(request.bytes(), bytes);
            assert_eq!(
                request.source_kind(),
                crate::protocol::DocumentInterchangeSourceKindV1::RequestText
            );

            let mut stdin = Cursor::new(bytes.clone());
            let standard_input = admit_interchange_source_v1(
                descriptor,
                InterchangeSourceInputV1::StandardInput(&mut stdin),
            )
            .expect("standard input should be admitted");
            assert_eq!(standard_input.bytes(), bytes);
            assert_eq!(
                standard_input.source_kind(),
                crate::protocol::DocumentInterchangeSourceKindV1::StandardInput
            );

            let path = temporary_path(descriptor.format_id());
            fs::write(&path, &bytes).expect("write regular source");
            let regular_file = admit_interchange_source_v1(
                descriptor,
                InterchangeSourceInputV1::RegularFile(&path),
            )
            .expect("regular file should be admitted");
            assert_eq!(regular_file.bytes(), bytes);
            assert_eq!(
                regular_file.source_kind(),
                crate::protocol::DocumentInterchangeSourceKindV1::RegularFile
            );
            assert!(regular_file.retained_source().is_some());
            drop(regular_file);
            fs::remove_file(path).expect("remove regular source");
        }
    }

    #[test]
    fn non_regular_file_is_redacted_at_shared_source_boundary() {
        let path = temporary_path("directory");
        fs::create_dir(&path).expect("make temporary directory");
        for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
            assert!(matches!(
                admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(&path)),
                Err(refusal) if refusal == generic_source_refusal()
            ));
        }
        fs::remove_dir(path).expect("remove temporary directory");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_is_refused_before_opening_at_shared_source_boundary() {
        use std::os::unix::fs::symlink;

        let target = temporary_path("target");
        let link = temporary_path("link");
        fs::write(&target, b"source").expect("write target");
        symlink(&target, &link).expect("create symlink");
        for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
            assert!(matches!(
                admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(&link)),
                Err(refusal) if refusal == generic_source_refusal()
            ));
        }
        fs::remove_file(link).expect("remove symlink");
        fs::remove_file(target).expect("remove target");
    }
}
