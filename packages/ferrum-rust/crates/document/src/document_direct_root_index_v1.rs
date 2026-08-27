//! Crate-private durable direct-root lookup facts.

use std::collections::HashMap;

use ferrum_document_projection::{
    DocumentDirectRootKindV1, DocumentObjectIdV1, DocumentProjectionV1,
};
use thiserror::Error;

/// Failure while indexing durable molecule roots by document-wide paint position.
#[derive(Debug, Error)]
pub enum DocumentDirectRootIndexErrorV1 {
    #[error("document direct-root index could not reserve owned storage")]
    ResourceAllocation,
    #[error("document projection has inconsistent durable molecule direct roots")]
    ProjectionMismatch,
}

/// Map every durable molecule root to its exact document-wide paint position.
pub fn document_direct_root_paint_orders_v1(
    projection: &DocumentProjectionV1,
) -> Result<HashMap<&DocumentObjectIdV1, u32>, DocumentDirectRootIndexErrorV1> {
    let mut molecule_ids = HashMap::new();
    molecule_ids
        .try_reserve(projection.molecules().len())
        .map_err(|_| DocumentDirectRootIndexErrorV1::ResourceAllocation)?;
    for molecule in projection.molecules() {
        let molecule_id = molecule.document_object_id();
        if molecule_ids.insert(molecule_id, ()).is_some() {
            return Err(DocumentDirectRootIndexErrorV1::ProjectionMismatch);
        }
    }

    let mut paint_orders = HashMap::new();
    paint_orders
        .try_reserve(molecule_ids.len())
        .map_err(|_| DocumentDirectRootIndexErrorV1::ResourceAllocation)?;
    for direct_root in projection.direct_roots() {
        if direct_root.kind() != DocumentDirectRootKindV1::Molecule {
            continue;
        }
        let molecule_id = direct_root.document_object_id();
        if !molecule_ids.contains_key(molecule_id)
            || paint_orders
                .insert(molecule_id, direct_root.paint_order())
                .is_some()
        {
            return Err(DocumentDirectRootIndexErrorV1::ProjectionMismatch);
        }
    }
    if paint_orders.len() != molecule_ids.len() {
        return Err(DocumentDirectRootIndexErrorV1::ProjectionMismatch);
    }
    Ok(paint_orders)
}
