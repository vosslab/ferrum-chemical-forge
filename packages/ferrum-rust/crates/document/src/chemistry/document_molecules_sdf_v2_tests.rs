use crate::DocumentSession;
use ferrum_chemistry::MolblockVersion;

use super::{
    DocumentMoleculesSdfErrorV2, DocumentMoleculesSdfRequestErrorV2, DocumentMoleculesSdfRequestV2,
    prepare_document_molecules_sdf_from_source_ids_v2, prepare_document_molecules_sdf_v2,
};

fn observation() -> crate::SessionDocumentObservationV1 {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\">",
        "<molecule id=\"first\" name=\"first display\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule>",
        "<molecule id=\"second\" name=\"second display\"><atom id=\"b\" name=\"O\"><point x=\"4\" y=\"2\"/></atom></molecule>",
        "</cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    session.observe(0).expect("fixture projects")
}

fn direct_root_ids(
    observation: &crate::SessionDocumentObservationV1,
) -> Vec<crate::DocumentObjectIdV1> {
    observation
        .projection()
        .molecules()
        .iter()
        .map(|molecule| molecule.id().expect("fixture root is durable").clone())
        .collect()
}

#[test]
fn batch_request_rejects_a_duplicate_direct_root() {
    let observation = observation();
    let first = direct_root_ids(&observation).remove(0);

    assert_eq!(
        DocumentMoleculesSdfRequestV2::new(
            observation.snapshot().revision(),
            *observation.snapshot().digest(),
            vec![first.clone(), first],
            MolblockVersion::V2000,
        ),
        Err(DocumentMoleculesSdfRequestErrorV2::DuplicateMolecule)
    );
}

#[test]
fn batch_preparation_rejects_a_stale_observation() {
    let observation = observation();
    let request = DocumentMoleculesSdfRequestV2::new(
        observation.snapshot().revision() + 1,
        *observation.snapshot().digest(),
        direct_root_ids(&observation),
        MolblockVersion::V3000,
    )
    .expect("distinct roots form a request");

    assert!(matches!(
        prepare_document_molecules_sdf_v2(&observation, &request),
        Err(DocumentMoleculesSdfErrorV2::Observation(_))
    ));
}

#[test]
fn source_id_route_authenticates_before_source_lookup() {
    let observation = observation();

    assert!(matches!(
        prepare_document_molecules_sdf_from_source_ids_v2(
            &observation,
            observation.snapshot().revision() + 1,
            *observation.snapshot().digest(),
            &["missing".to_owned(), "first".to_owned()],
            MolblockVersion::V2000,
        ),
        Err(DocumentMoleculesSdfErrorV2::Observation(_))
    ));
}
