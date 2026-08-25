//! Canonical planning and detached mutation for direct-molecule structural deletion.

use std::collections::{BTreeSet, HashMap, HashSet};

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, PersistentId, TypedDocument, TypedDocumentError,
    document_object_identity_v1::{DOCUMENT_OBJECT_NAMESPACE_V1, is_document_object_attribute_v1},
    element_name,
    reaction_reference_graph_v1::direct_reaction_reference_graph,
};

/// One retained connected component reported by structural deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructureDeletionComponentV1 {
    molecule_id: PersistentId,
    atom_ids: Vec<PersistentId>,
    bond_ids: Vec<PersistentId>,
}

impl StructureDeletionComponentV1 {
    #[must_use]
    pub fn molecule_id(&self) -> &PersistentId {
        &self.molecule_id
    }
    #[must_use]
    pub fn atom_ids(&self) -> &[PersistentId] {
        &self.atom_ids
    }
    #[must_use]
    pub fn bond_ids(&self) -> &[PersistentId] {
        &self.bond_ids
    }
}

/// Authoritative source-order outcome of one accepted structural deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StructureDeletionReceiptV1 {
    removed_atom_ids: Vec<PersistentId>,
    removed_bond_ids: Vec<PersistentId>,
    components: Vec<StructureDeletionComponentV1>,
}

impl StructureDeletionReceiptV1 {
    #[must_use]
    pub fn removed_atom_ids(&self) -> &[PersistentId] {
        &self.removed_atom_ids
    }
    #[must_use]
    pub fn removed_bond_ids(&self) -> &[PersistentId] {
        &self.removed_bond_ids
    }
    #[must_use]
    pub fn components(&self) -> &[StructureDeletionComponentV1] {
        &self.components
    }
}

/// Fully validated immutable structural-deletion plan awaiting session identities.
#[derive(Clone, Debug)]
pub(crate) struct StructureDeletionPlanV1 {
    molecule_id: PersistentId,
    retained_components: Vec<(Vec<PersistentId>, Vec<PersistentId>)>,
    receipt: StructureDeletionReceiptV1,
}

impl StructureDeletionPlanV1 {
    pub(crate) fn additional_molecule_count(&self) -> usize {
        self.retained_components.len().saturating_sub(1)
    }
}

impl TypedDocument {
    /// Plan one exact direct-root structural deletion without mutating this document.
    pub(crate) fn prepare_delete_structure(
        &self,
        molecule_id: PersistentId,
        atom_ids: Vec<PersistentId>,
        bond_ids: Vec<PersistentId>,
    ) -> Result<StructureDeletionPlanV1, TypedDocumentError> {
        if (atom_ids.is_empty() && bond_ids.is_empty()) || !unique(&atom_ids) || !unique(&bond_ids)
        {
            return Err(TypedDocumentError::InvalidStructureDeletionTarget(
                molecule_id,
            ));
        }
        let indexed = self.indexed();
        let tree = &indexed.xml().tree;
        let root = tree
            .document_element(indexed.xml().document)
            .expect("a parsed CDML document has a document element");
        let molecule = tree
            .children(root)
            .find(|node| {
                is_cdml_element(tree, *node, "molecule")
                    && attribute(tree, *node, "id") == Some(molecule_id.as_str())
            })
            .ok_or_else(|| {
                TypedDocumentError::InvalidStructureDeletionMolecule(molecule_id.clone())
            })?;
        validate_molecule_profile(tree, molecule, &molecule_id)?;
        let graph = direct_graph(tree, molecule, &molecule_id)?;
        let atoms = graph.atoms;
        let bonds = graph.bonds;
        let requested_atoms = atom_ids.iter().collect::<HashSet<_>>();
        let requested_bonds = bond_ids.iter().collect::<HashSet<_>>();
        if !requested_atoms
            .iter()
            .all(|id| atoms.iter().any(|(atom, _)| *atom == **id))
            || !requested_bonds
                .iter()
                .all(|id| bonds.iter().any(|(bond, _)| *bond == **id))
        {
            return Err(TypedDocumentError::InvalidStructureDeletionTarget(
                molecule_id,
            ));
        }
        let removed_atom_ids = atoms
            .iter()
            .filter(|(id, _)| requested_atoms.contains(id))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let removed_bond_ids = bonds
            .iter()
            .filter_map(|(id, bond)| {
                (requested_bonds.contains(id)
                    || requested_atoms.contains(&bond.start)
                    || requested_atoms.contains(&bond.end))
                .then_some(id.clone())
            })
            .collect::<Vec<_>>();
        let removed_bond_set = removed_bond_ids.iter().collect::<HashSet<_>>();
        let surviving_atoms = atoms
            .iter()
            .filter(|(id, _)| !requested_atoms.contains(id))
            .map(|(id, _)| id.clone())
            .collect::<Vec<_>>();
        let surviving_bonds = bonds
            .iter()
            .filter(|(id, _)| !removed_bond_set.contains(id))
            .map(|(id, bond)| (id.clone(), bond.clone()))
            .collect::<Vec<_>>();
        let retained_components = components(&surviving_atoms, &surviving_bonds);
        if direct_reaction_reference_graph(self).contains(molecule_id.as_str())
            && retained_components.len() != 1
        {
            return Err(TypedDocumentError::ReactionReferencedStructureDeletion(
                molecule_id,
            ));
        }
        let components = retained_components
            .iter()
            .enumerate()
            .map(|(index, (atoms, bonds))| StructureDeletionComponentV1 {
                molecule_id: if index == 0 {
                    molecule_id.clone()
                } else {
                    PersistentId::new(format!("pending-{index}")).expect("constant persistent ID")
                },
                atom_ids: atoms.clone(),
                bond_ids: bonds.clone(),
            })
            .collect();
        Ok(StructureDeletionPlanV1 {
            molecule_id,
            retained_components,
            receipt: StructureDeletionReceiptV1 {
                removed_atom_ids,
                removed_bond_ids,
                components,
            },
        })
    }

    /// Apply an already validated plan using only IDs allocated by its session.
    pub(crate) fn commit_delete_structure(
        &self,
        plan: &StructureDeletionPlanV1,
        later_molecule_ids: &[PersistentId],
    ) -> Result<(Self, StructureDeletionReceiptV1), TypedDocumentError> {
        if later_molecule_ids.len() != plan.additional_molecule_count() {
            return Err(TypedDocumentError::StructuralDeletionRequiresSession);
        }
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
                    && attribute(tree, *node, "id") == Some(plan.molecule_id.as_str())
            })
            .ok_or_else(|| {
                TypedDocumentError::InvalidStructureDeletionMolecule(plan.molecule_id.clone())
            })?;
        if plan.retained_components.is_empty() {
            tree.remove(molecule)
                .map_err(TypedDocumentError::Mutation)?;
        } else {
            let later_components = plan
                .retained_components
                .iter()
                .skip(1)
                .map(|component| {
                    let clone = tree.clone_with_prefixes(molecule);
                    let object_namespace = tree.add_namespace(DOCUMENT_OBJECT_NAMESPACE_V1);
                    let object_id = tree.add_name_ns("id", object_namespace);
                    tree.remove_attribute(clone, object_id);
                    let retained = component
                        .0
                        .iter()
                        .chain(&component.1)
                        .map(PersistentId::as_str)
                        .collect::<HashSet<_>>();
                    for child in tree
                        .children(clone)
                        .filter(|child| tree.is_element(*child))
                        .collect::<Vec<_>>()
                    {
                        tree.remove(child)
                            .expect("a detached component clone can remove direct children");
                    }
                    for child in tree
                        .children(molecule)
                        .filter(|child| tree.is_element(*child))
                        .collect::<Vec<_>>()
                    {
                        if attribute(tree, child, "id").is_some_and(|id| retained.contains(id)) {
                            tree.append(clone, child)
                                .expect("a detached component root accepts moved direct children");
                        }
                    }
                    (clone, component)
                })
                .collect::<Vec<_>>();
            retain_component(tree, molecule, &plan.retained_components[0])?;
            let mut insertion = molecule;
            for ((component, _), molecule_id) in
                later_components.into_iter().zip(later_molecule_ids)
            {
                let id_name = tree.add_name("id");
                let name_name = tree.add_name("name");
                tree.set_attribute(component, id_name, molecule_id.as_str());
                tree.remove_attribute(component, name_name);
                tree.insert_after(insertion, component)
                    .map_err(TypedDocumentError::Mutation)?;
                insertion = component;
            }
        }
        let serialized = candidate.to_xml()?;
        let document = Self::parse(&serialized)?;
        let mut receipt = plan.receipt.clone();
        for (component, identifier) in receipt
            .components
            .iter_mut()
            .skip(1)
            .zip(later_molecule_ids)
        {
            component.molecule_id = identifier.clone();
        }
        Ok((document, receipt))
    }
}

#[derive(Clone)]
struct Bond {
    start: PersistentId,
    end: PersistentId,
}

/// Validated direct atom/bond records of one structural-deletion molecule.
///
/// Keeping these collections named prevents topology validation from leaking
/// a positional tuple across the planning boundary.
struct DirectStructureGraphV1 {
    atoms: Vec<(PersistentId, Node)>,
    bonds: Vec<(PersistentId, Bond)>,
}

fn direct_graph(
    tree: &Xot,
    molecule: Node,
    molecule_id: &PersistentId,
) -> Result<DirectStructureGraphV1, TypedDocumentError> {
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    for child in tree.children(molecule) {
        if tree.is_text(child) {
            if tree.string_value(child).trim().is_empty() {
                continue;
            }
            return Err(TypedDocumentError::UnsupportedStructureDeletionMolecule(
                molecule_id.clone(),
            ));
        }
        if !tree.is_element(child) {
            return Err(TypedDocumentError::UnsupportedStructureDeletionMolecule(
                molecule_id.clone(),
            ));
        }
        let is_atom = is_cdml_element(tree, child, "atom");
        let is_bond = is_cdml_element(tree, child, "bond");
        if !is_atom && !is_bond {
            return Err(TypedDocumentError::UnsupportedStructureDeletionMolecule(
                molecule_id.clone(),
            ));
        }
        let identifier =
            PersistentId::new(attribute(tree, child, "id").unwrap_or_default().to_owned())
                .map_err(|_| {
                    TypedDocumentError::InvalidStructureDeletionTopology(molecule_id.clone())
                })?;
        if is_atom {
            if atoms.iter().any(|(id, _)| *id == identifier) {
                return Err(TypedDocumentError::InvalidStructureDeletionTopology(
                    molecule_id.clone(),
                ));
            }
            atoms.push((identifier, child));
        } else if is_bond {
            let start = PersistentId::new(
                attribute(tree, child, "start")
                    .unwrap_or_default()
                    .to_owned(),
            )
            .map_err(|_| {
                TypedDocumentError::InvalidStructureDeletionTopology(molecule_id.clone())
            })?;
            let end =
                PersistentId::new(attribute(tree, child, "end").unwrap_or_default().to_owned())
                    .map_err(|_| {
                        TypedDocumentError::InvalidStructureDeletionTopology(molecule_id.clone())
                    })?;
            if bonds.iter().any(|(id, _)| *id == identifier) {
                return Err(TypedDocumentError::InvalidStructureDeletionTopology(
                    molecule_id.clone(),
                ));
            }
            bonds.push((identifier, Bond { start, end }));
        }
    }
    if bonds.iter().any(|(_, bond)| {
        bond.start == bond.end
            || !atoms.iter().any(|(id, _)| *id == bond.start)
            || !atoms.iter().any(|(id, _)| *id == bond.end)
    }) {
        return Err(TypedDocumentError::InvalidStructureDeletionTopology(
            molecule_id.clone(),
        ));
    }
    Ok(DirectStructureGraphV1 { atoms, bonds })
}

fn components(
    atoms: &[PersistentId],
    bonds: &[(PersistentId, Bond)],
) -> Vec<(Vec<PersistentId>, Vec<PersistentId>)> {
    let mut adjacency = HashMap::<PersistentId, Vec<PersistentId>>::new();
    for atom in atoms {
        adjacency.insert(atom.clone(), Vec::new());
    }
    for (_, bond) in bonds {
        adjacency
            .get_mut(&bond.start)
            .expect("validated")
            .push(bond.end.clone());
        adjacency
            .get_mut(&bond.end)
            .expect("validated")
            .push(bond.start.clone());
    }
    let mut seen = HashSet::new();
    let mut output = Vec::new();
    for atom in atoms {
        if seen.contains(atom) {
            continue;
        }
        let mut stack = vec![atom.clone()];
        let mut members = BTreeSet::new();
        while let Some(current) = stack.pop() {
            if !members.insert(current.clone()) {
                continue;
            }
            for next in &adjacency[&current] {
                if !members.contains(next) {
                    stack.push(next.clone());
                }
            }
        }
        seen.extend(members.iter().cloned());
        let component_atoms = atoms
            .iter()
            .filter(|id| members.contains(*id))
            .cloned()
            .collect::<Vec<_>>();
        let component_bonds = bonds
            .iter()
            .filter(|(_, bond)| members.contains(&bond.start))
            .map(|(id, _)| id.clone())
            .collect();
        output.push((component_atoms, component_bonds));
    }
    output
}

fn retain_component(
    tree: &mut Xot,
    molecule: Node,
    component: &(Vec<PersistentId>, Vec<PersistentId>),
) -> Result<(), TypedDocumentError> {
    let retained = component
        .0
        .iter()
        .chain(&component.1)
        .map(PersistentId::as_str)
        .collect::<HashSet<_>>();
    for child in tree
        .children(molecule)
        .filter(|child| tree.is_element(*child))
        .collect::<Vec<_>>()
    {
        if !attribute(tree, child, "id").is_some_and(|id| retained.contains(id)) {
            tree.remove(child).map_err(TypedDocumentError::Mutation)?;
        }
    }
    Ok(())
}

fn validate_molecule_profile(
    tree: &Xot,
    molecule: Node,
    identifier: &PersistentId,
) -> Result<(), TypedDocumentError> {
    for (name, _) in tree.attributes(molecule).iter() {
        let (local, namespace) = tree.name_ns_str(name);
        if !is_document_object_attribute_v1(namespace, local)
            && (!namespace.is_empty() || (local != "id" && local != "name"))
        {
            return Err(TypedDocumentError::UnsupportedStructureDeletionMolecule(
                identifier.clone(),
            ));
        }
    }
    Ok(())
}

fn unique(ids: &[PersistentId]) -> bool {
    ids.iter().collect::<HashSet<_>>().len() == ids.len()
}
fn attribute<'a>(tree: &'a Xot, node: Node, expected: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (namespace.is_empty() && local == expected).then_some(value.as_str())
    })
}
fn is_cdml_element(tree: &Xot, node: Node, expected: &str) -> bool {
    element_name(tree, node)
        .is_some_and(|(local, namespace)| local == expected && (namespace == CDML_NAMESPACE))
}

impl TypedDocument {
    /// Return a detached candidate without one durable atom or its incident bonds.
    pub(crate) fn with_delete_atom(
        &self,
        identifier: &PersistentId,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let id = tree.add_name("id");
        let start = tree.add_name("start");
        let end = tree.add_name("end");
        let root = tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let atom = tree.descendants(root).find(|node| {
            is_cdml_element(tree, *node, "atom")
                && tree.get_attribute(*node, id) == Some(identifier.as_str())
        });
        let Some(atom) = atom else {
            return Ok(None);
        };
        let bonds = tree
            .descendants(root)
            .filter(|node| {
                is_cdml_element(tree, *node, "bond")
                    && (tree.get_attribute(*node, start) == Some(identifier.as_str())
                        || tree.get_attribute(*node, end) == Some(identifier.as_str()))
            })
            .collect::<Vec<_>>();
        for bond in bonds {
            tree.remove(bond).map_err(TypedDocumentError::Mutation)?;
        }
        tree.remove(atom).map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }

    /// Return a detached candidate without one durable typed bond.
    pub(crate) fn with_delete_bond(
        &self,
        identifier: &PersistentId,
    ) -> Result<Option<Self>, TypedDocumentError> {
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let id = tree.add_name("id");
        let root = tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let bond = tree.descendants(root).find(|node| {
            is_cdml_element(tree, *node, "bond")
                && tree.get_attribute(*node, id) == Some(identifier.as_str())
        });
        let Some(bond) = bond else {
            return Ok(None);
        };
        tree.remove(bond).map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized).map(Some)
    }
}
