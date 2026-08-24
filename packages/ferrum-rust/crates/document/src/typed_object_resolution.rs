//! Resolution of durable V1 projection selectors against typed CDML records.

use super::{DocumentObjectIdV1, TypedDocument, TypedRecord};

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
        find_record_by_object_id(self.root(), object_id)
    }
}

fn find_record_by_object_id<'a>(
    record: &'a TypedRecord,
    object_id: &DocumentObjectIdV1,
) -> Option<&'a TypedRecord> {
    if crate::document_object_id_from_record_v1(record).as_ref() == Some(object_id) {
        return Some(record);
    }
    record
        .typed_children()
        .iter()
        .find_map(|child| find_record_by_object_id(child.record(), object_id))
}
