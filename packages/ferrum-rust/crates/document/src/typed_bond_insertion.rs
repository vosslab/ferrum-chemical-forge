//! Structured XML mutation for one validated molecule-local bond insertion.

use xot::Xot;

use super::{
    CDML_NAMESPACE, DocumentBondPresentationV1, PersistentId, Point3V1, TypedDocument,
    TypedDocumentError, element_name,
};

pub(super) struct BondedAtomInsertion<'a> {
    atom_id: &'a PersistentId,
    bond_id: &'a PersistentId,
    element: &'a str,
    position: Point3V1,
    presentation: DocumentBondPresentationV1,
}

impl<'a> BondedAtomInsertion<'a> {
    pub(super) const fn new(
        atom_id: &'a PersistentId,
        bond_id: &'a PersistentId,
        element: &'a str,
        position: Point3V1,
        presentation: DocumentBondPresentationV1,
    ) -> Self {
        Self {
            atom_id,
            bond_id,
            element,
            position,
            presentation,
        }
    }
}

impl TypedDocument {
    /// Build one candidate containing a new atom and its bond to an existing atom.
    ///
    /// The two XML insertions remain one typed semantic operation: callers receive
    /// only the complete candidate, and the session validates that complete
    /// projection before issuing an acceptance token.
    pub(crate) fn with_insert_bonded_atom(
        &self,
        molecule_id: &PersistentId,
        start_atom_id: &PersistentId,
        insertion: BondedAtomInsertion<'_>,
    ) -> Result<Self, TypedDocumentError> {
        let with_atom = self.with_insert_atom(
            molecule_id,
            insertion.atom_id,
            insertion.element,
            insertion.position,
        )?;
        with_atom.with_insert_bond(
            molecule_id,
            insertion.bond_id,
            start_atom_id,
            insertion.atom_id,
            insertion.presentation,
        )
    }

    /// Build a detached, fully indexed candidate containing one new typed bond.
    ///
    /// Both endpoints must be direct atoms of the named molecule. The check is
    /// repeated here even when the session already resolved durable selectors, so
    /// this XML mutation primitive cannot be used to manufacture a cross-molecule
    /// edge or a duplicate undirected edge.
    pub(crate) fn with_insert_bond(
        &self,
        molecule_id: &PersistentId,
        bond_id: &PersistentId,
        start_atom_id: &PersistentId,
        end_atom_id: &PersistentId,
        presentation: DocumentBondPresentationV1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(bond_id).is_some() {
            return Err(TypedDocumentError::DuplicateBondId(bond_id.clone()));
        }
        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let id_name = indexed.xml.tree.add_name("id");
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let (molecule, namespace) =
            find_molecule(&indexed.xml.tree, root, id_name, molecule_id.as_str())
                .ok_or_else(|| TypedDocumentError::UnknownMolecule(molecule_id.clone()))?;

        let start_name = indexed.xml.tree.add_name("start");
        let end_name = indexed.xml.tree.add_name("end");
        let children = indexed.xml.tree.children(molecule).collect::<Vec<_>>();
        for endpoint in [start_atom_id, end_atom_id] {
            if !children.iter().copied().any(|node| {
                is_cdml_element(&indexed.xml.tree, node, "atom")
                    && indexed.xml.tree.get_attribute(node, id_name) == Some(endpoint.as_str())
            }) {
                return Err(TypedDocumentError::InvalidBondEndpoint(endpoint.clone()));
            }
        }
        if children.iter().copied().any(|node| {
            is_cdml_element(&indexed.xml.tree, node, "bond")
                && endpoints_match(
                    &indexed.xml.tree,
                    node,
                    start_name,
                    end_name,
                    start_atom_id.as_str(),
                    end_atom_id.as_str(),
                )
        }) {
            return Err(TypedDocumentError::DuplicateBond {
                start: start_atom_id.clone(),
                end: end_atom_id.clone(),
            });
        }

        let bond_name = element_name_id(&mut indexed.xml.tree, "bond", &namespace);
        let type_name = indexed.xml.tree.add_name("type");
        let bond = indexed.xml.tree.new_element(bond_name);
        indexed
            .xml
            .tree
            .set_attribute(bond, id_name, bond_id.as_str());
        indexed
            .xml
            .tree
            .set_attribute(bond, type_name, presentation.cdml_token());
        indexed
            .xml
            .tree
            .set_attribute(bond, start_name, start_atom_id.as_str());
        indexed
            .xml
            .tree
            .set_attribute(bond, end_name, end_atom_id.as_str());
        indexed
            .xml
            .tree
            .append(molecule, bond)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}

fn find_molecule(
    tree: &Xot,
    root: xot::Node,
    id_name: xot::NameId,
    molecule_id: &str,
) -> Option<(xot::Node, String)> {
    tree.descendants(root).find_map(|node| {
        let (local_name, namespace) = element_name(tree, node)?;
        (local_name == "molecule"
            && valid_namespace(&namespace)
            && tree.get_attribute(node, id_name) == Some(molecule_id))
        .then_some((node, namespace))
    })
}

fn is_cdml_element(tree: &Xot, node: xot::Node, expected: &str) -> bool {
    element_name(tree, node).is_some_and(|(local_name, namespace)| {
        local_name == expected && valid_namespace(&namespace)
    })
}

fn valid_namespace(namespace: &str) -> bool {
    namespace == CDML_NAMESPACE
}

fn endpoints_match(
    tree: &Xot,
    node: xot::Node,
    start_name: xot::NameId,
    end_name: xot::NameId,
    requested_start: &str,
    requested_end: &str,
) -> bool {
    let start = tree.get_attribute(node, start_name);
    let end = tree.get_attribute(node, end_name);
    (start == Some(requested_start) && end == Some(requested_end))
        || (start == Some(requested_end) && end == Some(requested_start))
}

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}
