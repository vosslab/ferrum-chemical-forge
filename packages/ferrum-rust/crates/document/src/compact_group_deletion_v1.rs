//! Typed planning and detached mutation for one attached compact-group deletion.

use xot::{Node, Xot};

use super::{CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError, element_name};

/// Authoritative direct CDML facts removed by one compact-group deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompactGroupDeletionReceiptV1 {
    molecule_id: PersistentId,
    compact_group_id: PersistentId,
    exterior_bond_id: PersistentId,
}

impl CompactGroupDeletionReceiptV1 {
    /// Return the direct molecule that owned the deleted compact group.
    #[must_use]
    pub fn molecule_id(&self) -> &PersistentId {
        &self.molecule_id
    }

    /// Return the deleted direct compact-group durable ID.
    #[must_use]
    pub fn compact_group_id(&self) -> &PersistentId {
        &self.compact_group_id
    }

    /// Return the deleted unique direct bond from the compact group to its atom.
    #[must_use]
    pub fn exterior_bond_id(&self) -> &PersistentId {
        &self.exterior_bond_id
    }
}

/// Fully validated compact-group deletion intent awaiting session admission.
#[derive(Clone, Debug)]
pub(crate) struct CompactGroupDeletionPlanV1 {
    receipt: CompactGroupDeletionReceiptV1,
}

impl TypedDocument {
    /// Plan removal of one direct compact group and its exact exterior atom bond.
    pub(crate) fn prepare_delete_compact_group_v1(
        &self,
        molecule_id: PersistentId,
        compact_group_id: PersistentId,
    ) -> Result<CompactGroupDeletionPlanV1, TypedDocumentError> {
        let indexed = self.indexed();
        let tree = &indexed.xml().tree;
        let root = tree
            .document_element(indexed.xml().document)
            .expect("a parsed CDML document has a document element");
        let molecule = tree
            .children(root)
            .find(|node| is_cdml_element(tree, *node, "molecule") && id(tree, *node) == Some(molecule_id.as_str()))
            .ok_or_else(|| TypedDocumentError::InvalidCompactGroupDeletionMolecule(molecule_id.clone()))?;
        let groups = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "compact-group") && id(tree, *node) == Some(compact_group_id.as_str()))
            .collect::<Vec<_>>();
        if groups.len() != 1 {
            return Err(TypedDocumentError::InvalidCompactGroupDeletionTarget(
                molecule_id,
            ));
        }
        let exterior_bonds = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "bond"))
            .filter_map(|bond| {
                let start = attribute(tree, bond, "start")?;
                let end = attribute(tree, bond, "end")?;
                ((start == compact_group_id.as_str()) || (end == compact_group_id.as_str()))
                    .then(|| (bond, start, end))
            })
            .collect::<Vec<_>>();
        if exterior_bonds.len() != 1 {
            return Err(TypedDocumentError::InvalidCompactGroupDeletionTopology(
                molecule_id,
            ));
        }
        let (bond, start, end) = exterior_bonds[0];
        let exterior_atom_id = if start == compact_group_id.as_str() {
            end
        } else {
            start
        };
        let matching_atoms = tree
            .children(molecule)
            .filter(|node| is_cdml_element(tree, *node, "atom") && id(tree, *node) == Some(exterior_atom_id))
            .count();
        let exterior_bond_id = PersistentId::new(id(tree, bond).unwrap_or_default().to_owned())
            .map_err(|_| TypedDocumentError::InvalidCompactGroupDeletionTopology(molecule_id.clone()))?;
        if matching_atoms != 1 {
            return Err(TypedDocumentError::InvalidCompactGroupDeletionTopology(
                molecule_id,
            ));
        }
        Ok(CompactGroupDeletionPlanV1 {
            receipt: CompactGroupDeletionReceiptV1 {
                molecule_id,
                compact_group_id,
                exterior_bond_id,
            },
        })
    }

    /// Apply one validated compact-group deletion to a detached typed candidate.
    pub(crate) fn commit_delete_compact_group_v1(
        &self,
        plan: &CompactGroupDeletionPlanV1,
    ) -> Result<(Self, CompactGroupDeletionReceiptV1), TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let molecule = tree
            .children(root)
            .find(|node| {
                is_cdml_element(tree, *node, "molecule")
                    && id(tree, *node) == Some(plan.receipt.molecule_id.as_str())
            })
            .ok_or_else(|| {
                TypedDocumentError::InvalidCompactGroupDeletionMolecule(
                    plan.receipt.molecule_id.clone(),
                )
            })?;
        let group = tree.children(molecule).find(|node| {
            is_cdml_element(tree, *node, "compact-group")
                && id(tree, *node) == Some(plan.receipt.compact_group_id.as_str())
        });
        let bond = tree.children(molecule).find(|node| {
            is_cdml_element(tree, *node, "bond")
                && id(tree, *node) == Some(plan.receipt.exterior_bond_id.as_str())
        });
        let (Some(group), Some(bond)) = (group, bond) else {
            return Err(TypedDocumentError::InvalidCompactGroupDeletionTarget(
                plan.receipt.molecule_id.clone(),
            ));
        };
        tree.remove(bond).map_err(TypedDocumentError::Mutation)?;
        tree.remove(group).map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Ok((Self::parse(&serialized)?, plan.receipt.clone()))
    }
}

fn attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (namespace.is_empty() && local == expected).then_some(value.as_str())
    })
}

fn id<'a>(tree: &'a Xot, node: Node) -> Option<&'a str> {
    attribute(tree, node, "id")
}

fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(local, namespace)| local == expected && namespace == CDML_NAMESPACE)
}
