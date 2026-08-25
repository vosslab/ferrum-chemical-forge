//! Resolution of durable V1 projection selectors against typed CDML records.

use super::{DocumentObjectIdV1, PersistentId, TypedDocument, TypedRecord};

impl TypedDocument {
    /// Resolve one validated V1 object selector against this exact retained document.
    ///
    /// The selector is compared to the document's source-or-legacy typed-record
    /// identity, never to a diagnostic XML path. A caller must use the returned
    /// record only while borrowing this immutable document.
    #[must_use]
    pub fn resolve_document_object_id(
        &self,
        object_id: &DocumentObjectIdV1,
    ) -> Option<&TypedRecord> {
        self.indexed()
            .resolve_document_object_id_path_v1(object_id)
            .and_then(|path| find_record_at_path(self.root(), path.components()))
            .or_else(|| find_record_by_document_object_id(self.root(), object_id))
    }

    /// Resolve an internal source record address to its persisted opaque selector.
    ///
    /// Source identifiers remain inside document mutation adapters; callers expose
    /// only the returned document-owned selector across the public boundary.
    #[must_use]
    pub(crate) fn document_object_id_for_source_id_v1(
        &self,
        source_id: &PersistentId,
    ) -> Option<DocumentObjectIdV1> {
        find_unique_record_by_source_id(self.root(), source_id.as_str())
            .and_then(crate::document_object_id_from_record_v1)
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
    ) -> Option<PersistentId> {
        let source_id = self
            .resolve_document_object_id(object_id)?
            .attribute("id")?;
        PersistentId::new(source_id.to_owned()).ok()
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

fn find_record_by_document_object_id<'a>(
    record: &'a TypedRecord,
    object_id: &DocumentObjectIdV1,
) -> Option<&'a TypedRecord> {
    if crate::document_object_id_from_record_v1(record).as_ref() == Some(object_id) {
        return Some(record);
    }
    record
        .typed_children()
        .iter()
        .find_map(|child| find_record_by_document_object_id(child.record(), object_id))
}
