//! Explicit, caller-owned admission of untrusted CDML and CD-SVG source.
//!
//! This module owns raw-byte, reader, and ordinary local-file admission. The document crate owns
//! XML token/tree validation and typed CDML identity validation after this boundary admits UTF-8.
//! It intentionally provides neither a default budget nor format detection.

use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::artifact_publication_v1::{
    RetainedSourceFileGuardV1, RetainedSourceIdentityErrorV1, retain_regular_source_file_v1,
};
use crate::{
    CdsvgExtractionError, DocumentSession, DocumentSessionError, TypedDocument, TypedDocumentError,
    XmlInputBudgetV1, XmlInputError, extract_cdml_from_svg_with_budget,
};
use thiserror::Error;

/// Complete CDML resource policy selected by the ingress owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CdmlIngressBudgetV1 {
    /// XML limits applied to the original decoded CDML source.
    pub xml: XmlInputBudgetV1,
}

/// Complete CD-SVG resource policy selected by the ingress owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CdsvgIngressBudgetV1 {
    /// XML limits applied to the original decoded SVG wrapper source.
    pub wrapper: XmlInputBudgetV1,
    /// XML limits applied to the structurally serialized canonical CDML payload.
    pub payload: XmlInputBudgetV1,
}

/// The caller-selected document container and its complete resource policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentIngressFormatV1 {
    /// Canonical CDML text.
    Cdml(CdmlIngressBudgetV1),
    /// SVG wrapper containing exactly one canonical CDML payload.
    Cdsvg(CdsvgIngressBudgetV1),
}

/// Source context retained in typed ingress errors without including source contents.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentIngressOriginV1 {
    /// A caller-provided byte slice.
    Bytes,
    /// A caller-provided stream, commonly standard input.
    StandardInput,
    /// A local file path requested through the regular-file ingress route.
    File(PathBuf),
}

/// A local source did not satisfy the explicit ingress policy.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SourcePolicyErrorV1 {
    /// A reader cannot safely retain one sentinel byte beyond this configured limit.
    #[error("the {limit}-byte source limit cannot represent its required sentinel byte")]
    ByteLimitSentinelUnrepresentable {
        /// Caller-supplied outer raw-source byte limit.
        limit: usize,
    },
    /// The requested local path was a symlink before it was opened.
    #[error("local document ingress does not accept symlink paths")]
    Symlink,
    /// The opened local handle was not a regular file.
    #[error("local document ingress requires a regular file")]
    NonRegularFile,
}

/// Typed CDML failure after byte admission and UTF-8 decoding.
#[derive(Debug, Error)]
pub enum CdmlIngressErrorV1 {
    /// XML admission rejected the decoded source before or during tree retention.
    #[error(transparent)]
    XmlInput(#[from] XmlInputError),
    /// CDML identity, schema, or typed projection validation rejected the retained tree.
    #[error(transparent)]
    Typed(#[from] TypedDocumentError),
    /// An already admitted typed document could not initialize a revision-zero session.
    #[error(transparent)]
    Session(#[from] DocumentSessionError),
}

/// Failure while admitting a complete untrusted document source.
#[derive(Debug, Error)]
pub enum DocumentIngressErrorV1 {
    /// Reading the requested source failed.
    #[error("could not read document input from {origin:?}: {source}")]
    Read {
        /// Source context.
        origin: DocumentIngressOriginV1,
        /// Operating-system or reader error.
        #[source]
        source: io::Error,
    },
    /// The source route or its supplied limit violated ingress policy.
    #[error("document input from {origin:?} violates source policy: {reason}")]
    SourcePolicy {
        /// Source context.
        origin: DocumentIngressOriginV1,
        /// Stable policy fact.
        reason: SourcePolicyErrorV1,
    },
    /// Raw input exceeded the caller-selected outer source byte limit.
    #[error("document input from {origin:?} exceeds the {limit}-byte limit")]
    ByteLimitExceeded {
        /// Source context.
        origin: DocumentIngressOriginV1,
        /// Caller-supplied outer source limit.
        limit: usize,
        /// Exact length for slices, or the lower bound observed by a reader sentinel.
        observed_at_least: usize,
    },
    /// Admitted raw bytes were not valid UTF-8.
    #[error("document input from {origin:?} is not valid UTF-8")]
    Utf8 {
        /// Source context.
        origin: DocumentIngressOriginV1,
        /// Byte index immediately after the longest valid UTF-8 prefix.
        valid_up_to: Option<usize>,
    },
    /// CDML validation rejected an admitted UTF-8 source.
    #[error("CDML input from {origin:?} was rejected: {source}")]
    Cdml {
        /// Source context.
        origin: DocumentIngressOriginV1,
        /// XML or typed-CDML failure.
        #[source]
        source: CdmlIngressErrorV1,
    },
    /// CD-SVG wrapper or payload validation rejected an admitted UTF-8 source.
    #[error("CD-SVG input from {origin:?} was rejected: {source}")]
    Cdsvg {
        /// Source context.
        origin: DocumentIngressOriginV1,
        /// Wrapper, payload, or CD-SVG structure failure.
        #[source]
        source: CdsvgExtractionError,
    },
}

/// One admitted local document and the exact source descriptor kept for publication.
///
/// The retained source lets a later artifact publisher reject the original file
/// or an observed hard-link alias as an output destination. It is created from
/// the same descriptor that supplied the admitted bytes.
pub struct AdmittedDocumentFileV1 {
    session: DocumentSession,
    retained_source: RetainedSourceFileGuardV1,
}

impl AdmittedDocumentFileV1 {
    /// Borrow the initialized revision-zero session.
    #[must_use]
    pub const fn session(&self) -> &DocumentSession {
        &self.session
    }

    /// Consume the admission into its session and retained source descriptor.
    #[must_use]
    pub fn into_parts(self) -> (DocumentSession, RetainedSourceFileGuardV1) {
        (self.session, self.retained_source)
    }
}

/// Admit one caller-owned UTF-8 byte slice as CDML or CD-SVG under an explicit budget.
pub fn load_document_utf8_bytes_with_budget(
    source: &[u8],
    format: DocumentIngressFormatV1,
) -> Result<DocumentSession, DocumentIngressErrorV1> {
    load_document_bytes_from_origin(source, DocumentIngressOriginV1::Bytes, format)
}

/// Admit one reader as CDML or CD-SVG under an explicit budget.
///
/// The reader is consumed only through the caller-selected outer byte ceiling plus one sentinel
/// byte. A sentinel failure reports a lower bound, not a falsely exact total source size.
pub fn load_document_reader_with_budget(
    reader: &mut dyn Read,
    origin: DocumentIngressOriginV1,
    format: DocumentIngressFormatV1,
) -> Result<DocumentSession, DocumentIngressErrorV1> {
    let limit = outer_byte_limit(format);
    let sentinel = checked_sentinel(limit, &origin)?;
    let bytes = read_through_sentinel(reader, sentinel, &origin)?;
    if bytes.len() == sentinel {
        return Err(DocumentIngressErrorV1::ByteLimitExceeded {
            origin,
            limit,
            observed_at_least: sentinel,
        });
    }
    load_document_bytes_from_origin(&bytes, origin, format)
}

/// Admit one non-symlink regular local file as CDML or CD-SVG under an explicit budget.
///
/// This is a normal desktop-file policy, not a claim of a race-free privileged-file API: portable
/// `std` must inspect the path before open. The opened handle is independently checked regular.
pub fn load_document_file_with_budget(
    path: &Path,
    format: DocumentIngressFormatV1,
) -> Result<DocumentSession, DocumentIngressErrorV1> {
    let origin = DocumentIngressOriginV1::File(path.to_path_buf());
    let mut file = open_regular_file(path, &origin)?;
    load_document_reader_with_budget(&mut file, origin, format)
}

/// Admit one local document and retain the exact opened source for artifact publication.
///
/// This route exists for one-shot converters. Ordinary editors should use
/// [`load_document_file_with_budget`] and own their separate save baseline.
pub fn load_document_file_for_publication_with_budget(
    path: &Path,
    format: DocumentIngressFormatV1,
) -> Result<AdmittedDocumentFileV1, DocumentIngressErrorV1> {
    let origin = DocumentIngressOriginV1::File(path.to_path_buf());
    let mut file = open_regular_file(path, &origin)?;
    let session = load_document_reader_with_budget(&mut file, origin.clone(), format)?;
    let retained_source = retain_regular_source_file_v1(file)
        .map_err(|error| retained_source_error(origin, error))?;
    Ok(AdmittedDocumentFileV1 {
        session,
        retained_source,
    })
}

/// Read one bounded regular local source while retaining its opened descriptor.
///
/// This is the narrow file-policy primitive for a Rust-owned converter whose
/// decoder is not CDML. It applies the same non-symlink/regular-file policy as
/// ordinary document admission.
pub fn read_regular_file_with_origin_with_budget(
    path: &Path,
    max_bytes: usize,
) -> Result<(Vec<u8>, RetainedSourceFileGuardV1), DocumentIngressErrorV1> {
    let origin = DocumentIngressOriginV1::File(path.to_path_buf());
    let mut file = open_regular_file(path, &origin)?;
    let sentinel = checked_sentinel(max_bytes, &origin)?;
    let bytes = read_through_sentinel(&mut file, sentinel, &origin)?;
    if bytes.len() == sentinel {
        return Err(DocumentIngressErrorV1::ByteLimitExceeded {
            origin,
            limit: max_bytes,
            observed_at_least: sentinel,
        });
    }
    let source = retain_regular_source_file_v1(file)
        .map_err(|error| retained_source_error(origin, error))?;
    Ok((bytes, source))
}

fn open_regular_file(
    path: &Path,
    origin: &DocumentIngressOriginV1,
) -> Result<File, DocumentIngressErrorV1> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DocumentIngressErrorV1::Read {
        origin: origin.clone(),
        source,
    })?;
    if metadata.file_type().is_symlink() {
        return Err(DocumentIngressErrorV1::SourcePolicy {
            origin: origin.clone(),
            reason: SourcePolicyErrorV1::Symlink,
        });
    }
    let file = File::open(path).map_err(|source| DocumentIngressErrorV1::Read {
        origin: origin.clone(),
        source,
    })?;
    let opened = file
        .metadata()
        .map_err(|source| DocumentIngressErrorV1::Read {
            origin: origin.clone(),
            source,
        })?;
    if !opened.file_type().is_file() {
        return Err(DocumentIngressErrorV1::SourcePolicy {
            origin: origin.clone(),
            reason: SourcePolicyErrorV1::NonRegularFile,
        });
    }
    Ok(file)
}

fn retained_source_error(
    origin: DocumentIngressOriginV1,
    error: RetainedSourceIdentityErrorV1,
) -> DocumentIngressErrorV1 {
    match error {
        RetainedSourceIdentityErrorV1::Inspect { source } => {
            DocumentIngressErrorV1::Read { origin, source }
        }
        RetainedSourceIdentityErrorV1::NonRegular => DocumentIngressErrorV1::SourcePolicy {
            origin,
            reason: SourcePolicyErrorV1::NonRegularFile,
        },
    }
}

fn load_document_bytes_from_origin(
    source: &[u8],
    origin: DocumentIngressOriginV1,
    format: DocumentIngressFormatV1,
) -> Result<DocumentSession, DocumentIngressErrorV1> {
    let limit = outer_byte_limit(format);
    checked_sentinel(limit, &origin)?;
    if source.len() > limit {
        return Err(DocumentIngressErrorV1::ByteLimitExceeded {
            origin,
            limit,
            observed_at_least: source.len(),
        });
    }
    let text = std::str::from_utf8(source).map_err(|error| DocumentIngressErrorV1::Utf8 {
        origin: origin.clone(),
        valid_up_to: Some(error.valid_up_to()),
    })?;
    match format {
        DocumentIngressFormatV1::Cdml(budget) => {
            let document = TypedDocument::parse_with_budget(text, budget.xml)
                .map_err(|error| cdml_error(origin.clone(), error))?;
            DocumentSession::from_admitted_document(document).map_err(|source| {
                DocumentIngressErrorV1::Cdml {
                    origin,
                    source: CdmlIngressErrorV1::Session(source),
                }
            })
        }
        DocumentIngressFormatV1::Cdsvg(budget) => {
            let document = extract_cdml_from_svg_with_budget(text, budget.wrapper, budget.payload)
                .map_err(|source| DocumentIngressErrorV1::Cdsvg {
                    origin: origin.clone(),
                    source,
                })?;
            DocumentSession::from_admitted_document(document).map_err(|source| {
                DocumentIngressErrorV1::Cdml {
                    origin,
                    source: CdmlIngressErrorV1::Session(source),
                }
            })
        }
    }
}

fn cdml_error(
    origin: DocumentIngressOriginV1,
    error: TypedDocumentError,
) -> DocumentIngressErrorV1 {
    let source = match error {
        TypedDocumentError::XmlInput(source) => CdmlIngressErrorV1::XmlInput(source),
        source => CdmlIngressErrorV1::Typed(source),
    };
    DocumentIngressErrorV1::Cdml { origin, source }
}

fn outer_byte_limit(format: DocumentIngressFormatV1) -> usize {
    match format {
        DocumentIngressFormatV1::Cdml(budget) => budget.xml.max_utf8_bytes,
        DocumentIngressFormatV1::Cdsvg(budget) => budget.wrapper.max_utf8_bytes,
    }
}

fn checked_sentinel(
    limit: usize,
    origin: &DocumentIngressOriginV1,
) -> Result<usize, DocumentIngressErrorV1> {
    limit
        .checked_add(1)
        .ok_or_else(|| DocumentIngressErrorV1::SourcePolicy {
            origin: origin.clone(),
            reason: SourcePolicyErrorV1::ByteLimitSentinelUnrepresentable { limit },
        })
}

fn read_through_sentinel(
    reader: &mut dyn Read,
    sentinel: usize,
    origin: &DocumentIngressOriginV1,
) -> Result<Vec<u8>, DocumentIngressErrorV1> {
    let mut source = Vec::new();
    let mut chunk = [0_u8; 8192];
    while source.len() < sentinel {
        let remaining = sentinel - source.len();
        let chunk_length = chunk.len();
        let count = reader
            .read(&mut chunk[..remaining.min(chunk_length)])
            .map_err(|source| DocumentIngressErrorV1::Read {
                origin: origin.clone(),
                source,
            })?;
        if count == 0 {
            break;
        }
        source.extend_from_slice(&chunk[..count]);
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Cursor, Read},
        path::PathBuf,
    };

    use crate::{CdsvgExtractionError, XmlBudgetError, XmlInputBudgetV1, XmlInputError};

    use super::{
        CdmlIngressBudgetV1, CdsvgIngressBudgetV1, DocumentIngressErrorV1, DocumentIngressFormatV1,
        DocumentIngressOriginV1, SourcePolicyErrorV1, load_document_file_with_budget,
        load_document_reader_with_budget, load_document_utf8_bytes_with_budget,
    };

    const CDML: &str = "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"/>";
    const CDSVG: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"><cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"/></svg>";

    fn budget(bytes: usize) -> XmlInputBudgetV1 {
        XmlInputBudgetV1 {
            max_utf8_bytes: bytes,
            max_elements: 20,
            max_depth: 10,
            max_attributes: 20,
            max_text_bytes: 20,
        }
    }

    fn cdml_format(bytes: usize) -> DocumentIngressFormatV1 {
        DocumentIngressFormatV1::Cdml(CdmlIngressBudgetV1 { xml: budget(bytes) })
    }

    #[test]
    fn bytes_accept_exact_limit_and_reject_one_over_before_utf8() {
        let session =
            load_document_utf8_bytes_with_budget(CDML.as_bytes(), cdml_format(CDML.len()))
                .expect("exact byte budget must admit CDML");
        assert_eq!(
            session
                .snapshot()
                .expect("session must snapshot")
                .revision(),
            0
        );

        let source = [0xff_u8; 4];
        let error = load_document_utf8_bytes_with_budget(&source, cdml_format(3))
            .expect_err("over-budget bytes must reject before UTF-8 decoding");
        assert!(matches!(
            error,
            DocumentIngressErrorV1::ByteLimitExceeded {
                origin: DocumentIngressOriginV1::Bytes,
                limit: 3,
                observed_at_least: 4,
            }
        ));
    }

    #[test]
    fn invalid_utf8_and_dtd_have_distinct_typed_failures() {
        let error = load_document_utf8_bytes_with_budget(&[0xff], cdml_format(1))
            .expect_err("invalid UTF-8 must reject before XML parsing");
        assert!(matches!(
            error,
            DocumentIngressErrorV1::Utf8 {
                origin: DocumentIngressOriginV1::Bytes,
                valid_up_to: Some(0),
            }
        ));

        let dtd = b"<!DOCTYPE cdml><cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\"/>";
        let error = load_document_utf8_bytes_with_budget(dtd, cdml_format(dtd.len()))
            .expect_err("DTD must retain document-layer typed XML rejection");
        assert!(matches!(
            error,
            DocumentIngressErrorV1::Cdml {
                source: super::CdmlIngressErrorV1::XmlInput(XmlInputError::DtdForbidden),
                ..
            }
        ));

        let malformed = b"<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\">";
        let error = load_document_utf8_bytes_with_budget(malformed, cdml_format(malformed.len()))
            .expect_err("malformed XML must remain a document-layer XML failure");
        assert!(matches!(
            error,
            DocumentIngressErrorV1::Cdml {
                source: super::CdmlIngressErrorV1::XmlInput(
                    XmlInputError::Preflight(_) | XmlInputError::Xml(_)
                ),
                ..
            }
        ));
    }

    #[test]
    fn reader_reports_only_the_sentinel_lower_bound() {
        let mut reader = Cursor::new(vec![b'x'; 8]);
        let error = load_document_reader_with_budget(
            &mut reader,
            DocumentIngressOriginV1::StandardInput,
            cdml_format(3),
        )
        .expect_err("reader must stop at its one-byte sentinel");
        assert!(
            matches!(
                error,
                DocumentIngressErrorV1::ByteLimitExceeded {
                    origin: DocumentIngressOriginV1::StandardInput,
                    limit: 3,
                    observed_at_least: 4,
                }
            ),
            "unexpected CD-SVG admission error: {error:?}"
        );
    }

    #[test]
    fn failed_reader_and_unrepresentable_sentinel_are_typed() {
        struct FailingReader;
        impl Read for FailingReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("reader failed"))
            }
        }

        let mut reader = FailingReader;
        let error = load_document_reader_with_budget(
            &mut reader,
            DocumentIngressOriginV1::StandardInput,
            cdml_format(20),
        )
        .expect_err("reader failure must be typed");
        assert!(matches!(error, DocumentIngressErrorV1::Read { .. }));

        let error = load_document_utf8_bytes_with_budget(&[], cdml_format(usize::MAX))
            .expect_err("maximum usize cannot express a reader sentinel");
        assert!(matches!(
            error,
            DocumentIngressErrorV1::SourcePolicy {
                reason: SourcePolicyErrorV1::ByteLimitSentinelUnrepresentable { limit: usize::MAX },
                ..
            }
        ));
    }

    #[test]
    fn cdsvg_has_independent_wrapper_and_payload_budget_errors() {
        let format = DocumentIngressFormatV1::Cdsvg(CdsvgIngressBudgetV1 {
            wrapper: budget(CDSVG.len()),
            payload: budget(1),
        });
        let error = load_document_utf8_bytes_with_budget(CDSVG.as_bytes(), format)
            .expect_err("payload limit must remain independent of the wrapper limit");
        assert!(
            matches!(
                error,
                DocumentIngressErrorV1::Cdsvg {
                    source: CdsvgExtractionError::PayloadInput(XmlInputError::Budget(
                        XmlBudgetError::Utf8Bytes { limit: 1, .. }
                    )),
                    ..
                }
            ),
            "unexpected CD-SVG admission error: {error:?}"
        );
    }

    #[test]
    fn file_rejects_directory_and_symlink_before_session_construction() {
        let directory = std::env::temp_dir().join(format!(
            "ferrum-document-ingress-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&directory).expect("create fixture directory");
        let error = load_document_file_with_budget(&directory, cdml_format(100))
            .expect_err("directory is not an admissible document file");
        assert!(matches!(
            error,
            DocumentIngressErrorV1::SourcePolicy {
                reason: SourcePolicyErrorV1::NonRegularFile,
                ..
            }
        ));

        let target = directory.join("source.cdml");
        let link = directory.join("source-link.cdml");
        std::fs::write(&target, CDML).expect("write fixture CDML");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &link).expect("create fixture symlink");
            let error = load_document_file_with_budget(&link, cdml_format(100))
                .expect_err("symlink is not an admissible document path");
            assert!(matches!(
                error,
                DocumentIngressErrorV1::SourcePolicy {
                    reason: SourcePolicyErrorV1::Symlink,
                    ..
                }
            ));
            std::fs::remove_file(&link).expect("remove fixture symlink");
        }
        std::fs::remove_file(&target).expect("remove fixture CDML");
        std::fs::remove_dir(&directory).expect("remove fixture directory");
    }

    #[test]
    fn cdml_xml_limit_is_preserved_after_raw_byte_admission() {
        let error = load_document_utf8_bytes_with_budget(CDML.as_bytes(), cdml_format(CDML.len()))
            .expect("baseline must load")
            .snapshot()
            .expect("admitted session must snapshot");
        assert_eq!(error.revision(), 0);
    }

    #[test]
    fn file_origin_keeps_its_path_without_source_contents() {
        let origin = DocumentIngressOriginV1::File(PathBuf::from("document.cdml"));
        assert_eq!(
            origin,
            DocumentIngressOriginV1::File(PathBuf::from("document.cdml"))
        );
    }
}
