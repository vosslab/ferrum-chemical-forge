//! Direct CLI presentation of the selected-root export operation.

use std::io::{Read, Write};
use std::path::Path;

use ferrum_document::{
    DocumentSession, export_prepared_document_molecule, prepare_document_molecule_export,
};

use crate::cli::{DocumentExportFormat, engine_bundle};

use super::{VerbCliError, publish_or_write, read_document};

pub(crate) fn run(
    input: &Path,
    molecule_id: &str,
    format: DocumentExportFormat,
    output: Option<&Path>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let source = read_document(input, stdin)?;
    let session =
        DocumentSession::load(&source.document).map_err(|_| VerbCliError::UnexpectedOutcome)?;
    let observation = session
        .observe(0)
        .map_err(|_| VerbCliError::UnexpectedOutcome)?;
    let snapshot = observation.snapshot();
    let molecule_id = ferrum_document::DocumentObjectIdV1::parse(molecule_id.to_owned())
        .map_err(|_| VerbCliError::UnexpectedOutcome)?;
    let prepared = prepare_document_molecule_export(
        &observation,
        &ferrum_document::DocumentMoleculeExportRequest::new(
            snapshot.revision(),
            *snapshot.digest(),
            molecule_id,
            format.into(),
        ),
    )?;
    let engine =
        engine_bundle::active_native_engine().map_err(|_| VerbCliError::ChemistryUnavailable)?;
    let receipt = export_prepared_document_molecule(&engine, prepared)?;
    publish_or_write(
        output,
        receipt.text().as_bytes().to_vec(),
        source.retained_source,
        stdout,
        stderr,
    )
}

impl From<DocumentExportFormat> for ferrum_document::DocumentMoleculeExportFormat {
    fn from(value: DocumentExportFormat) -> Self {
        match value {
            DocumentExportFormat::MolfileV2000 => Self::MolfileV2000,
            DocumentExportFormat::MolfileV3000 => Self::MolfileV3000,
            DocumentExportFormat::SdfV2000 => Self::SdfV2000,
            DocumentExportFormat::SdfV3000 => Self::SdfV3000,
            DocumentExportFormat::CanonicalSmiles => Self::CanonicalSmiles,
            DocumentExportFormat::InchiStandard => Self::InchiStandard,
            DocumentExportFormat::InchiFixedHydrogen => Self::InchiFixedHydrogen,
        }
    }
}
