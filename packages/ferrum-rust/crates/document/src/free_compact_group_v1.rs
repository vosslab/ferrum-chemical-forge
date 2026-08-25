//! Typed free compact-group placement facts and CDML mutation.

use crate::{
    CDML_NAMESPACE, CompactGroupCatalogKeyV1, PersistentId, Point3V1, TypedDocument,
    TypedDocumentError, element_name,
};
use thiserror::Error;

/// One closed free compact-group placement request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaceFreeCompactGroupV1 {
    catalog_key: CompactGroupCatalogKeyV1,
    anchor: Point3V1,
}

impl PlaceFreeCompactGroupV1 {
    /// Build one finite scene placement request.
    #[must_use]
    pub const fn new(catalog_key: CompactGroupCatalogKeyV1, anchor: Point3V1) -> Self {
        Self {
            catalog_key,
            anchor,
        }
    }

    /// Return the closed requested catalog key.
    #[must_use]
    pub const fn catalog_key(self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    /// Return the finite snapped scene anchor.
    #[must_use]
    pub const fn anchor(self) -> Point3V1 {
        self.anchor
    }
}

/// Closed refusals before durable identity allocation or session mutation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum FreeCompactGroupErrorV1 {
    #[error("free compact-group placement supports the Methyl catalog key only")]
    UnsupportedCatalogKey,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FreeCompactGroupCandidateV1 {
    anchor: Point3V1,
    catalog_key: CompactGroupCatalogKeyV1,
    orientation_degrees: i16,
}

impl FreeCompactGroupCandidateV1 {
    pub(crate) const fn anchor(self) -> Point3V1 {
        self.anchor
    }

    pub(crate) const fn catalog_key(self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    pub(crate) const fn orientation_degrees(self) -> i16 {
        self.orientation_degrees
    }
}

/// Derive the canonical positive-x orientation for the initially supported free group.
pub(crate) fn free_compact_group_candidate_v1(
    request: PlaceFreeCompactGroupV1,
) -> Result<FreeCompactGroupCandidateV1, FreeCompactGroupErrorV1> {
    if request.catalog_key() != CompactGroupCatalogKeyV1::Methyl {
        return Err(FreeCompactGroupErrorV1::UnsupportedCatalogKey);
    }
    Ok(FreeCompactGroupCandidateV1 {
        anchor: request.anchor(),
        catalog_key: request.catalog_key(),
        orientation_degrees: 0,
    })
}

impl TypedDocument {
    /// Build one direct-root molecule containing only one free compact group.
    pub(crate) fn with_place_free_compact_group_v1(
        &self,
        molecule_id: &PersistentId,
        group_id: &PersistentId,
        candidate: FreeCompactGroupCandidateV1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(molecule_id).is_some() {
            return Err(TypedDocumentError::DuplicateInsertionId(
                molecule_id.clone(),
            ));
        }
        if self.indexed().resolve_id(group_id).is_some() {
            return Err(TypedDocumentError::DuplicateGroupId(group_id.clone()));
        }
        if molecule_id == group_id {
            return Err(TypedDocumentError::DuplicateInsertionId(
                molecule_id.clone(),
            ));
        }

        let mut detached = self.detached_candidate()?;
        let indexed = detached.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("parsed CDML has a root");
        let (_, namespace) = element_name(tree, root).expect("parsed CDML has a root element");
        let namespace = tree.add_namespace(if namespace == CDML_NAMESPACE {
            &namespace
        } else {
            CDML_NAMESPACE
        });
        let molecule_name = tree.add_name_ns("molecule", namespace);
        let group_name = tree.add_name_ns("compact-group", namespace);
        let point_name = tree.add_name_ns("point", namespace);
        let id_name = tree.add_name("id");
        let molecule = tree.new_element(molecule_name);
        tree.set_attribute(molecule, id_name, molecule_id.as_str());
        let group = tree.new_element(group_name);
        for (name, value) in [
            ("id", group_id.as_str()),
            ("version", "1"),
            ("catalog-key", candidate.catalog_key().as_str()),
            ("attachment-index", "0"),
        ] {
            let attribute = tree.add_name(name);
            tree.set_attribute(group, attribute, value);
        }
        let orientation_name = tree.add_name("orientation-degrees");
        tree.set_attribute(
            group,
            orientation_name,
            candidate.orientation_degrees().to_string(),
        );
        let point = tree.new_element(point_name);
        let x_name = tree.add_name("x");
        let y_name = tree.add_name("y");
        tree.set_attribute(point, x_name, candidate.anchor().x().to_string());
        tree.set_attribute(point, y_name, candidate.anchor().y().to_string());
        tree.append(group, point)
            .map_err(TypedDocumentError::Mutation)?;
        tree.append(molecule, group)
            .map_err(TypedDocumentError::Mutation)?;
        tree.append(root, molecule)
            .map_err(TypedDocumentError::Mutation)?;
        Self::parse(&detached.to_xml()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn methyl_candidate_uses_the_requested_anchor_and_canonical_orientation() {
        let anchor = Point3V1::new(12.0, -4.0, 0.0).expect("anchor");
        let candidate = free_compact_group_candidate_v1(PlaceFreeCompactGroupV1::new(
            CompactGroupCatalogKeyV1::Methyl,
            anchor,
        ))
        .expect("candidate");
        assert_eq!(candidate.anchor(), anchor);
        assert_eq!(candidate.orientation_degrees(), 0);
    }

    #[test]
    fn non_methyl_closed_keys_are_refused_before_mutation() {
        let request = PlaceFreeCompactGroupV1::new(
            CompactGroupCatalogKeyV1::Nitro,
            Point3V1::new(0.0, 0.0, 0.0).expect("anchor"),
        );
        assert_eq!(
            free_compact_group_candidate_v1(request),
            Err(FreeCompactGroupErrorV1::UnsupportedCatalogKey),
        );
    }
}
