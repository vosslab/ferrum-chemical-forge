//! Document-private adaptation from retained typed records to lower identity values.

use ferrum_document_projection::{DocumentObjectIdV1, ProjectionError, ProjectionLocalObjectKeyV1};

use crate::TypedRecord;

/// Read the required persisted durable identity for a retained projection record.
pub(crate) fn projection_document_object_id_from_record_v1(
    record: &TypedRecord,
) -> Result<DocumentObjectIdV1, ProjectionError> {
    record
        .document_object_id_metadata_v1()
        .map(|value| {
            DocumentObjectIdV1::parse(value.to_owned()).map_err(|error| {
                ProjectionError::InvalidValue {
                    context: record.path().to_string(),
                    field: "document object identity",
                    value: format!("{value:?}: {error}"),
                }
            })
        })
        .transpose()?
        .ok_or_else(|| ProjectionError::MissingDocumentObjectId {
            context: record.path().to_string(),
        })
}

/// Derive a projection-local lower identity from one retained typed record.
pub(crate) fn projection_local_object_key_from_record_v1(
    record: &TypedRecord,
) -> Result<ProjectionLocalObjectKeyV1, ProjectionError> {
    ProjectionLocalObjectKeyV1::from_path_components(record.path().components()).map_err(|error| {
        ProjectionError::InvalidValue {
            context: record.path().to_string(),
            field: "projection path",
            value: error.to_string(),
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::{TypedClass, TypedDocument};

    use super::projection_document_object_id_from_record_v1;

    #[test]
    fn typed_record_reads_a_persisted_durable_lower_identity() {
        let document =
            TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>")
                .expect("typed document must parse");
        let molecule = document
            .root()
            .children_of(TypedClass::Molecule)
            .next()
            .expect("typed document must retain its molecule");

        assert!(
            projection_document_object_id_from_record_v1(molecule)
                .expect("typed ingress must persist a durable identity")
                .as_str()
                .starts_with("ferrum-document-object-v1/")
        );
    }
}
