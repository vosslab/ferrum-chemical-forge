//! Canonical typed-CDML writing for first-class compact known groups.

use xot::Node;

use super::{
    CDML_NAMESPACE, CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, PersistentId, Point3V1,
    TypedDocument, TypedDocumentError, element_name,
    typed_coordinate::canonical_authored_coordinate,
    typed_molecule_insertion::{InsertionNames, valid_cdml_namespace},
};

impl TypedDocument {
    /// Return a detached candidate with a compact group in a new direct root.
    pub(crate) fn with_insert_free_compact_group(
        &self,
        molecule_id: &PersistentId,
        group_id: &PersistentId,
        catalog_key: CompactGroupCatalogKeyV1,
        anchor: Point3V1,
        attachment: CompactGroupAttachmentV1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(molecule_id).is_some()
            || self.indexed().resolve_id(group_id).is_some()
        {
            return Err(TypedDocumentError::DuplicateInsertionId(group_id.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has an XML root");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let namespace = valid_cdml_namespace(namespace);
        let molecule_name = element_name_id(&mut indexed.xml.tree, "molecule", &namespace);
        let id_name = indexed.xml.tree.add_name("id");
        let molecule = indexed.xml.tree.new_element(molecule_name);
        indexed
            .xml
            .tree
            .set_attribute(molecule, id_name, molecule_id.as_str());
        indexed
            .xml
            .tree
            .append(root, molecule)
            .map_err(TypedDocumentError::Mutation)?;
        let names = InsertionNames::new(&mut indexed.xml.tree, namespace.clone());
        append_compact_group(
            &mut indexed.xml.tree,
            molecule,
            &namespace,
            &names,
            group_id,
            catalog_key,
            anchor,
            attachment,
        )?;
        Self::parse(&candidate.to_xml()?)
    }

    /// Return a detached candidate with one canonical V1 compact-group record.
    ///
    /// Session operations retain identity allocation, fencing, history, and commit
    /// authority. This typed layer only owns canonical XML construction.
    pub(crate) fn with_insert_compact_group(
        &self,
        molecule_id: &PersistentId,
        group_id: &PersistentId,
        catalog_key: CompactGroupCatalogKeyV1,
        anchor: Point3V1,
        attachment: CompactGroupAttachmentV1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(group_id).is_some() {
            return Err(TypedDocumentError::DuplicateInsertionId(group_id.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has an XML root");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let namespace = valid_cdml_namespace(namespace);
        let names = InsertionNames::new(&mut indexed.xml.tree, namespace.clone());
        let molecule = direct_molecule(&indexed.xml.tree, root, names.id, molecule_id)?;

        append_compact_group(
            &mut indexed.xml.tree,
            molecule,
            &namespace,
            &names,
            group_id,
            catalog_key,
            anchor,
            attachment,
        )?;
        Self::parse(&candidate.to_xml()?)
    }

    /// Return a detached candidate with one canonical normal atom-to-group bond.
    pub(crate) fn with_insert_compact_group_bond(
        &self,
        molecule_id: &PersistentId,
        bond_id: &PersistentId,
        atom_id: &PersistentId,
        group_id: &PersistentId,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(bond_id).is_some() {
            return Err(TypedDocumentError::DuplicateBondId(bond_id.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has an XML root");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let namespace = valid_cdml_namespace(namespace);
        let names = InsertionNames::new(&mut indexed.xml.tree, namespace.clone());
        let molecule = direct_molecule(&indexed.xml.tree, root, names.id, molecule_id)?;
        let atom_exists = indexed.xml.tree.children(molecule).any(|child| {
            element_name(&indexed.xml.tree, child)
                .is_some_and(|(name, ns)| name == "atom" && ns == CDML_NAMESPACE)
                && indexed.xml.tree.get_attribute(child, names.id) == Some(atom_id.as_str())
        });
        let group_exists = indexed.xml.tree.children(molecule).any(|child| {
            element_name(&indexed.xml.tree, child)
                .is_some_and(|(name, ns)| name == "compact-group" && ns == CDML_NAMESPACE)
                && indexed.xml.tree.get_attribute(child, names.id) == Some(group_id.as_str())
        });
        if !atom_exists || !group_exists {
            return Err(TypedDocumentError::InvalidBondEndpoint(atom_id.clone()));
        }
        let bond_name = element_name_id(&mut indexed.xml.tree, "bond", &namespace);
        let type_name = indexed.xml.tree.add_name("type");
        let start_name = indexed.xml.tree.add_name("start");
        let end_name = indexed.xml.tree.add_name("end");
        let bond = indexed.xml.tree.new_element(bond_name);
        indexed
            .xml
            .tree
            .set_attribute(bond, names.id, bond_id.as_str());
        indexed.xml.tree.set_attribute(bond, type_name, "n1");
        indexed
            .xml
            .tree
            .set_attribute(bond, start_name, atom_id.as_str());
        indexed
            .xml
            .tree
            .set_attribute(bond, end_name, group_id.as_str());
        indexed
            .xml
            .tree
            .append(molecule, bond)
            .map_err(TypedDocumentError::Mutation)?;
        Self::parse(&candidate.to_xml()?)
    }
}

fn append_compact_group(
    tree: &mut xot::Xot,
    molecule: Node,
    namespace: &str,
    names: &InsertionNames,
    group_id: &PersistentId,
    catalog_key: CompactGroupCatalogKeyV1,
    anchor: Point3V1,
    attachment: CompactGroupAttachmentV1,
) -> Result<(), TypedDocumentError> {
    let compact_group = element_name_id(tree, "compact-group", namespace);
    let version = tree.add_name("version");
    let catalog_key_name = tree.add_name("catalog-key");
    let attachment_index_name = tree.add_name("attachment-index");
    let orientation_degrees = tree.add_name("orientation-degrees");
    let group = tree.new_element(compact_group);
    tree.set_attribute(group, names.id, group_id.as_str());
    tree.set_attribute(group, version, "1");
    tree.set_attribute(group, catalog_key_name, catalog_key.as_str());
    tree.set_attribute(
        group,
        attachment_index_name,
        attachment.attachment_index().to_string(),
    );
    tree.set_attribute(
        group,
        orientation_degrees,
        attachment.orientation_degrees().to_string(),
    );
    let point = tree.new_element(names.point);
    tree.set_attribute(point, names.x, canonical_authored_coordinate(anchor.x()));
    tree.set_attribute(point, names.y, canonical_authored_coordinate(anchor.y()));
    tree.set_attribute(point, names.z, canonical_authored_coordinate(anchor.z()));
    tree.append(group, point).map_err(TypedDocumentError::Mutation)?;
    tree.append(molecule, group)
        .map_err(TypedDocumentError::Mutation)
}

fn element_name_id(tree: &mut xot::Xot, local_name: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}

fn direct_molecule(
    tree: &xot::Xot,
    root: Node,
    id_name: xot::NameId,
    molecule_id: &PersistentId,
) -> Result<Node, TypedDocumentError> {
    tree.children(root)
        .find(|node| {
            let Some((local_name, namespace)) = element_name(tree, *node) else {
                return false;
            };
            local_name == "molecule"
                && namespace == CDML_NAMESPACE
                && tree.get_attribute(*node, id_name) == Some(molecule_id.as_str())
        })
        .ok_or_else(|| TypedDocumentError::UnknownMolecule(molecule_id.clone()))
}
