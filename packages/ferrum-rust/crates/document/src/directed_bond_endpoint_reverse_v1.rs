//! Reversal of the authored direction of one retained wedge bond.

use thiserror::Error;
use xot::{Node, Xot};

use super::{CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError, element_name};

/// A closed request to reverse one directed wedge bond's retained endpoints.
#[derive(Clone, Debug, PartialEq)]
pub struct ReverseDirectedBondEndpointsV1 {
    source_bond_id: PersistentId,
}

impl ReverseDirectedBondEndpointsV1 {
    /// Validate the durable source identity targeted by this operation.
    pub fn new(
        source_bond_id: impl Into<String>,
    ) -> Result<Self, ReverseDirectedBondEndpointsV1Error> {
        let source_bond_id = PersistentId::new(source_bond_id.into())
            .map_err(|_| ReverseDirectedBondEndpointsV1Error::InvalidSourceBondId)?;
        Ok(Self { source_bond_id })
    }

    /// Return the exact retained CDML source-bond identity.
    #[must_use]
    pub fn source_bond_id(&self) -> &PersistentId {
        &self.source_bond_id
    }
}

/// Invalid reversal intent rejected before document lookup.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ReverseDirectedBondEndpointsV1Error {
    /// The durable CDML source-bond identifier is empty or otherwise invalid.
    #[error("directed bond reversal requires a valid persistent source-bond ID")]
    InvalidSourceBondId,
}

impl TypedDocument {
    /// Return a detached candidate with exactly one directed bond's endpoints exchanged.
    pub(crate) fn with_reversed_directed_bond_endpoints(
        &self,
        reverse: &ReverseDirectedBondEndpointsV1,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        {
            let indexed = candidate.detached_indexed_mut();
            let tree = &mut indexed.xml.tree;
            let root = tree
                .document_element(indexed.xml.document)
                .expect("a parsed CDML document has a document element");
            let id_name = tree.add_name("id");
            let mut target = None;
            for molecule in tree.children(root).collect::<Vec<_>>() {
                if !is_cdml_element(tree, molecule, "molecule") {
                    continue;
                }
                for bond in tree.children(molecule).collect::<Vec<_>>() {
                    if !is_cdml_element(tree, bond, "bond")
                        || tree.get_attribute(bond, id_name)
                            != Some(reverse.source_bond_id().as_str())
                    {
                        continue;
                    }
                    if target.replace((molecule, bond)).is_some() {
                        return Ok(None);
                    }
                }
            }
            let Some((molecule, bond)) = target else {
                return Ok(None);
            };

            let type_name = tree.add_name("type");
            if !matches!(tree.get_attribute(bond, type_name), Some("w1" | "h1")) {
                return Err(TypedDocumentError::UnsupportedDirectedBondEndpointReversal(
                    reverse.source_bond_id().clone(),
                ));
            }
            let start_name = tree.add_name("start");
            let end_name = tree.add_name("end");
            let start = tree
                .get_attribute(bond, start_name)
                .map(str::to_owned)
                .ok_or_else(|| {
                    TypedDocumentError::InvalidBondEndpoint(reverse.source_bond_id().clone())
                })?;
            let end = tree
                .get_attribute(bond, end_name)
                .map(str::to_owned)
                .ok_or_else(|| {
                    TypedDocumentError::InvalidBondEndpoint(reverse.source_bond_id().clone())
                })?;
            if !direct_atom_exists(tree, molecule, id_name, &start)
                || !direct_atom_exists(tree, molecule, id_name, &end)
            {
                return Err(TypedDocumentError::InvalidBondEndpoint(
                    reverse.source_bond_id().clone(),
                ));
            }
            if start == end {
                return Err(TypedDocumentError::InvalidBondEndpoint(
                    reverse.source_bond_id().clone(),
                ));
            }
            tree.set_attribute(bond, start_name, &end);
            tree.set_attribute(bond, end_name, &start);
        }
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}

fn direct_atom_exists(tree: &Xot, molecule: Node, id_name: xot::NameId, identifier: &str) -> bool {
    tree.children(molecule).any(|node| {
        is_cdml_element(tree, node, "atom") && tree.get_attribute(node, id_name) == Some(identifier)
    })
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && namespace == CDML_NAMESPACE
    })
}
