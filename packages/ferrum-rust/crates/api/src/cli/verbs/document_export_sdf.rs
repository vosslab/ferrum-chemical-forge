//! `ferrum document export-sdf` over Rust-owned multi-root SDF V2.

use std::io::{Read, Write};
use std::path::Path;

use ferrum_chemistry::MolblockVersion;
use ferrum_document::{
    DocumentSession, export_prepared_document_molecules_sdf_v2,
    prepare_document_molecules_sdf_from_source_ids_v2,
};

use crate::cli::{SdfVersion, engine_bundle};

use super::{VerbCliError, publish_or_write, read_document};

/// Export selected authored direct-molecule IDs through the document V2 contract.
pub(crate) fn run(
    input: &Path,
    molecule_ids: &[String],
    version: SdfVersion,
    output: &Path,
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
    let prepared = prepare_document_molecules_sdf_from_source_ids_v2(
        &observation,
        snapshot.revision(),
        *snapshot.digest(),
        molecule_ids,
        version.into(),
    )?;
    let engine =
        engine_bundle::active_native_engine().map_err(|_| VerbCliError::ChemistryUnavailable)?;
    let receipt = export_prepared_document_molecules_sdf_v2(&engine, prepared)?;
    publish_or_write(
        Some(output),
        receipt.sdf().as_bytes().to_vec(),
        source.retained_source,
        stdout,
        stderr,
    )
}

impl From<SdfVersion> for MolblockVersion {
    fn from(value: SdfVersion) -> Self {
        match value {
            SdfVersion::V2000 => Self::V2000,
            SdfVersion::V3000 => Self::V3000,
        }
    }
}
