//! Resolution of durable V1 projection selectors against typed CDML records.

use super::projection_identity_v1::projection_document_object_id_from_record_v1;
use super::{DocumentObjectIdV1, PersistentId, ProjectionError, TypedDocument, TypedRecord};

impl TypedDocument {
    /// Resolve one validated V1 object selector against this exact retained document.
    ///
    /// The document-owned persisted `DocumentObjectIdV1` index resolves the
    /// selector, never a diagnostic XML path. A caller must use the returned
    /// record only while borrowing this immutable document.
    #[must_use]
    pub fn resolve_document_object_id(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<Option<&TypedRecord>, ProjectionError> {
        let Some(path) = self.indexed().resolve_document_object_id_path_v1(object_id) else {
            return Ok(None);
        };
        Ok(find_record_at_path(self.root(), path.components()))
    }

    /// Resolve an internal source record address to its persisted opaque selector.
    ///
    /// Source identifiers remain inside document mutation adapters; callers expose
    /// only the returned document-owned selector across the public boundary.
    #[must_use]
    pub(crate) fn document_object_id_for_source_id_v1(
        &self,
        source_id: &PersistentId,
    ) -> Result<Option<DocumentObjectIdV1>, ProjectionError> {
        let Some(record) = find_unique_record_by_source_id(self.root(), source_id.as_str()) else {
            return Ok(None);
        };
        projection_document_object_id_from_record_v1(record).map(Some)
    }

    /// Resolve one durable selector to its private persisted source identifier.
    ///
    /// This remains crate-private so source `id` values cannot become a document
    /// session API. It pairs with [`Self::document_object_id_for_source_id_v1`]
    /// for adapters that must translate retained IDREF semantics to durable facts.
    #[must_use]
    pub(crate) fn source_id_for_document_object_id_v1(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Result<Option<PersistentId>, ProjectionError> {
        let Some(record) = self.resolve_document_object_id(object_id)? else {
            return Ok(None);
        };
        let Some(source_id) = record.attribute("id") else {
            return Ok(None);
        };
        PersistentId::new(source_id.to_owned())
            .map(Some)
            .map_err(|error| ProjectionError::InvalidValue {
                context: record.path().to_string(),
                field: "source identity",
                value: format!("{source_id:?}: {error}"),
            })
    }
}

fn find_record_at_path<'a>(record: &'a TypedRecord, path: &[u32]) -> Option<&'a TypedRecord> {
    if record.path().components() == path {
        return Some(record);
    }
    record
        .typed_children()
        .iter()
        .find_map(|child| find_record_at_path(child.record(), path))
}

fn find_unique_record_by_source_id<'a>(
    record: &'a TypedRecord,
    source_id: &str,
) -> Option<&'a TypedRecord> {
    let mut matches = Vec::new();
    collect_records_by_source_id(record, source_id, &mut matches);
    if matches.len() == 1 {
        matches.pop()
    } else {
        None
    }
}

fn collect_records_by_source_id<'a>(
    record: &'a TypedRecord,
    source_id: &str,
    matches: &mut Vec<&'a TypedRecord>,
) {
    if record.attribute("id") == Some(source_id) {
        matches.push(record);
    }
    record
        .typed_children()
        .iter()
        .for_each(|child| collect_records_by_source_id(child.record(), source_id, matches));
}
