//! Document-private adaptation from retained typed records to lower identity values.

use ferrum_document_projection::{DocumentObjectIdV1, ProjectionError, ProjectionLocalObjectKeyV1};

use crate::TypedRecord;

/// Derive a durable lower identity from one retained typed record.
pub(crate) fn document_object_id_from_record_v1(
    record: &TypedRecord,
) -> Option<DocumentObjectIdV1> {
    projection_document_object_id_from_record_v1(record)
        .ok()
        .flatten()
}

/// Derive a durable identity for a projection without erasing malformed source facts.
pub(crate) fn projection_document_object_id_from_record_v1(
    record: &TypedRecord,
) -> Result<Option<DocumentObjectIdV1>, ProjectionError> {
    record
        .attribute("id")
        .map(|source_id| {
            DocumentObjectIdV1::from_class_source(record.class().name(), source_id).map_err(
                |error| ProjectionError::InvalidValue {
                    context: record.path().to_string(),
                    field: "id",
                    value: format!("{source_id:?}: {error}"),
                },
            )
        })
        .transpose()
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

    use super::document_object_id_from_record_v1;

    #[test]
    fn typed_record_derives_a_durable_lower_identity() {
        let document =
            TypedDocument::parse("<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>")
                .expect("typed document must parse");
        let molecule = document
            .root()
            .children_of(TypedClass::Molecule)
            .next()
            .expect("typed document must retain its molecule");

        assert_eq!(
            document_object_id_from_record_v1(molecule)
                .expect("authored ID must derive a durable identity")
                .as_str(),
            "ferrum-document-object-v1/63646d6c2f6d6f6c6563756c65/source/6d",
        );
    }
}
