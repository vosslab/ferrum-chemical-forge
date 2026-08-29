//! Safe publication of one frozen selected-molecule export receipt.

use std::path::PathBuf;

use crate::DocumentMoleculeExport;
use crate::artifact_publication_v1::{
    ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
    publish_artifact_v1,
};
use thiserror::Error;

/// Publish the exact bytes of one authenticated selected-root export.
pub fn publish_document_molecule_export(
    receipt: &DocumentMoleculeExport,
    destination: PathBuf,
) -> Result<ArtifactPublicationOutcomeV1, DocumentMoleculeExportPublicationError> {
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(receipt.text().len()).is_err() {
        return Err(DocumentMoleculeExportPublicationError::ResourceAllocation { destination });
    }
    bytes.extend_from_slice(receipt.text().as_bytes());
    // ASVS 15.4.2: create the final entry atomically instead of observing then
    // replacing it.  A selected-root receipt is a new user-requested artifact,
    // never permission to overwrite an existing path.
    publish_artifact_v1(ArtifactPublicationRequestV1::new(destination, bytes).create_new())
        .map_err(Into::into)
}

/// Failure while materializing or publishing an export receipt.
#[derive(Debug, Error)]
pub enum DocumentMoleculeExportPublicationError {
    #[error(
        "document molecule export publication to {destination} could not reserve output storage"
    )]
    ResourceAllocation { destination: PathBuf },
    #[error(transparent)]
    Publication(#[from] ArtifactPublicationErrorV1),
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use ferrum_chemistry::{
        ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, NativeTextOutputLimit,
        SmilesMolecule,
    };

    use super::*;
    use crate::{
        DocumentMoleculeExportFormat, DocumentMoleculeExportRequest, DocumentSession,
        export_prepared_document_molecule, prepare_document_molecule_export,
    };

    static NEXT_TEST_DIRECTORY: AtomicU64 = AtomicU64::new(0);
    const SOURCE: &str = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\">",
        "<molecule id=\"root\"><atom id=\"a1\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    );

    struct SmilesEngine(RefCell<u8>);

    impl ChemEngine for SmilesEngine {
        fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles_to_molecule",
            })
        }

        fn generate_2d_coordinates(
            &self,
            _molecule: &MolGraph,
        ) -> Result<Coordinates, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "generate_2d_coordinates",
            })
        }

        fn molecule_to_smiles(
            &self,
            _molecule: &MolGraph,
            _limit: NativeTextOutputLimit,
        ) -> Result<String, ChemistryError> {
            *self.0.borrow_mut() += 1;
            Ok("C".to_owned())
        }

        fn kekulize(
            &self,
            molecule: &MolGraph,
            _options: KekulizeOptions,
        ) -> Result<MolGraph, ChemistryError> {
            Ok(molecule.clone())
        }
    }

    fn test_directory(label: &str) -> PathBuf {
        let serial = NEXT_TEST_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory = std::env::temp_dir()
            .canonicalize()
            .expect("temporary root must resolve")
            .join(format!(
                "ferrum-document-molecule-export-publication-{label}-{}-{serial}",
                std::process::id(),
            ));
        fs::create_dir(&directory).expect("test directory must create");
        directory
    }

    fn receipt() -> DocumentMoleculeExport {
        let session = DocumentSession::load(SOURCE).expect("fixture must load");
        let observation = session.observe(0).expect("fixture must project");
        let request = DocumentMoleculeExportRequest::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            observation.projection().molecules()[0]
                .document_object_id()
                .clone(),
            DocumentMoleculeExportFormat::CanonicalSmiles,
        );
        let prepared =
            prepare_document_molecule_export(&observation, &request).expect("fixture must prepare");
        let engine = SmilesEngine(RefCell::new(0));
        let receipt =
            export_prepared_document_molecule(&engine, prepared).expect("fixture must export");
        assert_eq!(*engine.0.borrow(), 1);
        receipt
    }

    #[test]
    fn selected_root_publication_creates_a_new_artifact_and_preserves_receipt_facts() {
        let directory = test_directory("fresh");
        let destination = directory.join("molecule.smi");
        let receipt = receipt();
        let expected_facts = (
            receipt.source_revision(),
            *receipt.source_digest(),
            receipt.molecule_id().clone(),
            receipt.format(),
            receipt.text().to_owned(),
        );

        let outcome = publish_document_molecule_export(&receipt, destination.clone())
            .expect("fresh publication must complete");

        assert!(matches!(
            outcome,
            ArtifactPublicationOutcomeV1::ConfirmedDurable(_)
                | ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(_)
        ));
        assert_eq!(
            fs::read_to_string(destination).expect("artifact must read"),
            "C"
        );
        assert_eq!(
            (
                receipt.source_revision(),
                *receipt.source_digest(),
                receipt.molecule_id().clone(),
                receipt.format(),
                receipt.text().to_owned(),
            ),
            expected_facts,
        );
        fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
    }

    #[test]
    fn selected_root_publication_refuses_existing_regular_file_without_overwriting_it() {
        let directory = test_directory("existing");
        let destination = directory.join("molecule.smi");
        fs::write(&destination, "existing molecule").expect("fixture must write");

        let error = publish_document_molecule_export(&receipt(), destination.clone())
            .expect_err("existing output must refuse publication");

        assert!(matches!(
            error,
            DocumentMoleculeExportPublicationError::Publication(
                ArtifactPublicationErrorV1::NotPublished { phase, source, .. }
            ) if phase == crate::artifact_publication_v1::ArtifactPrepublicationPhaseV1::ValidateBeforeTemporary
                && source.kind() == std::io::ErrorKind::AlreadyExists
        ));
        assert_eq!(
            fs::read_to_string(destination).expect("existing output must survive"),
            "existing molecule",
        );
        fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
    }

    #[cfg(unix)]
    #[test]
    fn selected_root_publication_refuses_symlink_and_hardlink_destinations_without_mutation() {
        let directory = test_directory("aliases");
        let protected = directory.join("protected.smi");
        fs::write(&protected, "protected molecule").expect("fixture must write");
        let symlink = directory.join("symlink.smi");
        std::os::unix::fs::symlink(&protected, &symlink).expect("symlink fixture must create");

        let symlink_error = publish_document_molecule_export(&receipt(), symlink)
            .expect_err("symlink output must refuse publication");
        assert!(matches!(
            symlink_error,
            DocumentMoleculeExportPublicationError::Publication(
                ArtifactPublicationErrorV1::RejectedDestination {
                    reason: crate::artifact_publication_v1::ArtifactDestinationRejectionV1::FinalIsSymlink,
                    ..
                }
            )
        ));

        let hardlink = directory.join("hardlink.smi");
        fs::hard_link(&protected, &hardlink).expect("hard-link fixture must create");
        let hardlink_error = publish_document_molecule_export(&receipt(), hardlink)
            .expect_err("hard-link output must refuse publication");
        assert!(matches!(
            hardlink_error,
            DocumentMoleculeExportPublicationError::Publication(
                ArtifactPublicationErrorV1::NotPublished { phase, source, .. }
            ) if phase == crate::artifact_publication_v1::ArtifactPrepublicationPhaseV1::ValidateBeforeTemporary
                && source.kind() == std::io::ErrorKind::AlreadyExists
        ));
        assert_eq!(
            fs::read_to_string(protected).expect("protected target must survive"),
            "protected molecule",
        );
        fs::remove_dir_all(directory).expect("test directory cleanup must succeed");
    }
}
