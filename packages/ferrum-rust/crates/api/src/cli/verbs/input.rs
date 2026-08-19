//! Bounded CDML input shared by the human-oriented verbs.

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use ferrum_document::artifact_publication_v1::{
    RetainedSourceFileGuardV1, retain_regular_source_file_v1,
};

use ferrum_document::{
    DocumentIngressOriginV1, load_document_file_for_publication_with_budget,
    load_document_reader_with_budget, local_cdml_ingress_format_v1,
};

use super::VerbCliError;

pub(crate) struct VerbDocumentInput {
    pub(crate) document: String,
    pub(crate) retained_source: Option<RetainedSourceFileGuardV1>,
}

/// A bounded UTF-8 interchange source and its retained file identity.
pub(crate) struct VerbTextInput {
    pub(crate) text: String,
    pub(crate) retained_source: Option<RetainedSourceFileGuardV1>,
}

pub(crate) fn read_document(
    input: &Path,
    stdin: &mut dyn Read,
) -> Result<VerbDocumentInput, VerbCliError> {
    let (session, retained_source) = if crate::transport::streams::is_standard_stream(input) {
        let session = load_document_reader_with_budget(
            stdin,
            DocumentIngressOriginV1::StandardInput,
            local_cdml_ingress_format_v1(),
        )?;
        (session, None)
    } else {
        let admitted =
            load_document_file_for_publication_with_budget(input, local_cdml_ingress_format_v1())?;
        let (session, retained_source) = admitted.into_parts();
        (session, Some(retained_source))
    };
    let snapshot = session.snapshot()?;
    Ok(VerbDocumentInput {
        document: snapshot.cdml().to_owned(),
        retained_source,
    })
}

/// Read one bounded UTF-8 source without interpreting it outside the protocol.
///
/// The protocol's own format-aware codec remains the sole semantic validator.
/// Retaining a named source preserves the safe no-follow replacement guarantee
/// when a successful verb writes back to the same path.
pub(crate) fn read_text(
    input: &Path,
    stdin: &mut dyn Read,
    limit: usize,
) -> Result<VerbTextInput, VerbCliError> {
    if crate::transport::streams::is_standard_stream(input) {
        return Ok(VerbTextInput {
            text: read_utf8_bounded(stdin, "standard input", limit)?,
            retained_source: None,
        });
    }

    let label = input.display().to_string();
    let mut file = File::open(input).map_err(|source| VerbCliError::Input {
        input: label.clone(),
        source,
    })?;
    let retained_source =
        retain_regular_source_file_v1(file.try_clone().map_err(|source| VerbCliError::Input {
            input: label.clone(),
            source,
        })?)
        .map_err(|source| VerbCliError::Input {
            input: label.clone(),
            source: io::Error::other(source),
        })?;
    Ok(VerbTextInput {
        text: read_utf8_bounded(&mut file, &label, limit)?,
        retained_source: Some(retained_source),
    })
}

fn read_utf8_bounded(
    reader: &mut dyn Read,
    input: &str,
    limit: usize,
) -> Result<String, VerbCliError> {
    let mut bytes = Vec::new();
    reader
        .take(
            u64::try_from(limit)
                .expect("Ferrum interchange input limit fits u64")
                .saturating_add(1),
        )
        .read_to_end(&mut bytes)
        .map_err(|source| VerbCliError::Input {
            input: input.to_owned(),
            source,
        })?;
    if bytes.len() > limit {
        return Err(VerbCliError::InputTooLarge {
            input: input.to_owned(),
            limit,
        });
    }
    String::from_utf8(bytes).map_err(|source| VerbCliError::InvalidUtf8 {
        input: input.to_owned(),
        source,
    })
}
