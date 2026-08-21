//! Structured XML mutation for complete, validated molecule insertions.

use std::collections::BTreeSet;

use xot::{Node, Xot};

use super::{
    CDML_NAMESPACE, INTERCHANGE_IMPORT_NAMESPACE_V1, InterchangeRecordInsertionV1,
    MoleculeInsertionV1, PersistentId, TypedDocument, TypedDocumentError, element_name,
};

impl TypedDocument {
    pub(crate) fn with_insert_molecule(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        molecule: &MoleculeInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(molecule_id, atom_ids, bond_ids, molecule, None)
    }

    pub(crate) fn with_insert_interchange_record(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        record: &InterchangeRecordInsertionV1,
    ) -> Result<Self, TypedDocumentError> {
        self.with_insert_molecule_record(
            molecule_id,
            atom_ids,
            bond_ids,
            record.molecule(),
            Some(record),
        )
    }

    fn with_insert_molecule_record(
        &self,
        molecule_id: &PersistentId,
        atom_ids: &[PersistentId],
        bond_ids: &[PersistentId],
        molecule: &MoleculeInsertionV1,
        interchange_record: Option<&InterchangeRecordInsertionV1>,
    ) -> Result<Self, TypedDocumentError> {
        if atom_ids.len() != molecule.atoms().len() || bond_ids.len() != molecule.bonds().len() {
            return Err(TypedDocumentError::InsertionIdentityCountMismatch);
        }
        let mut supplied_ids = BTreeSet::new();
        for identifier in std::iter::once(molecule_id)
            .chain(atom_ids.iter())
            .chain(bond_ids.iter())
        {
            if self.indexed().resolve_id(identifier).is_some() || !supplied_ids.insert(identifier) {
                return Err(TypedDocumentError::DuplicateInsertionId(identifier.clone()));
            }
        }

        let mut candidate = self.detached_candidate()?;
        let indexed = candidate.detached_indexed_mut();
        let root = indexed
            .xml
            .tree
            .document_element(indexed.xml.document)
            .expect("a parsed CDML document has a document element");
        let (_, namespace) = element_name(&indexed.xml.tree, root)
            .expect("a parsed CDML document has an XML root element");
        let namespace = valid_cdml_namespace(namespace);
        let names = InsertionNames::new(&mut indexed.xml.tree, namespace);
        let molecule_node = indexed.xml.tree.new_element(names.molecule);
        indexed
            .xml
            .tree
            .set_attribute(molecule_node, names.id, molecule_id.as_str());
        if let Some(record) = interchange_record
            && !record.title().is_empty()
            && xml_attribute_safe(record.title())
        {
            indexed
                .xml
                .tree
                .set_attribute(molecule_node, names.name, record.title());
        }

        for (atom_id, atom) in atom_ids.iter().zip(molecule.atoms()) {
            append_atom(&mut indexed.xml.tree, molecule_node, &names, atom_id, atom)?;
        }
        for (bond_id, bond) in bond_ids.iter().zip(molecule.bonds()) {
            let bond_node = indexed.xml.tree.new_element(names.bond);
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.id, bond_id.as_str());
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.bond_type, bond.order().cdml_token());
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.start, atom_ids[bond.start()].as_str());
            indexed
                .xml
                .tree
                .set_attribute(bond_node, names.end, atom_ids[bond.end()].as_str());
            indexed
                .xml
                .tree
                .append(molecule_node, bond_node)
                .map_err(TypedDocumentError::Mutation)?;
        }
        if let Some(record) = interchange_record {
            append_interchange_metadata(&mut indexed.xml.tree, molecule_node, record)?;
        }
        indexed
            .xml
            .tree
            .append(root, molecule_node)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = candidate.to_xml()?;
        Self::parse(&serialized)
    }
}

pub(crate) fn valid_cdml_namespace(namespace: String) -> String {
    if namespace.is_empty() || namespace == CDML_NAMESPACE {
        namespace
    } else {
        unreachable!("TypedDocument accepts only no-namespace or CDML roots")
    }
}

#[derive(Clone, Copy)]
pub(crate) struct InsertionNames {
    pub(crate) molecule: xot::NameId,
    pub(crate) atom: xot::NameId,
    pub(crate) point: xot::NameId,
    pub(crate) bond: xot::NameId,
    pub(crate) id: xot::NameId,
    pub(crate) name: xot::NameId,
    pub(crate) charge: xot::NameId,
    pub(crate) isotope: xot::NameId,
    pub(crate) explicit_hydrogens: xot::NameId,
    pub(crate) x: xot::NameId,
    pub(crate) y: xot::NameId,
    pub(crate) z: xot::NameId,
    pub(crate) bond_type: xot::NameId,
    pub(crate) start: xot::NameId,
    pub(crate) end: xot::NameId,
    pub(crate) haworth_position: xot::NameId,
}

impl InsertionNames {
    pub(crate) fn new(tree: &mut Xot, namespace: String) -> Self {
        Self {
            molecule: element_name_id(tree, "molecule", &namespace),
            atom: element_name_id(tree, "atom", &namespace),
            point: element_name_id(tree, "point", &namespace),
            bond: element_name_id(tree, "bond", &namespace),
            id: tree.add_name("id"),
            name: tree.add_name("name"),
            charge: tree.add_name("charge"),
            isotope: tree.add_name("isotope"),
            explicit_hydrogens: tree.add_name("explicit_hydrogens"),
            x: tree.add_name("x"),
            y: tree.add_name("y"),
            z: tree.add_name("z"),
            bond_type: tree.add_name("type"),
            start: tree.add_name("start"),
            end: tree.add_name("end"),
            haworth_position: tree.add_name("haworth_position"),
        }
    }
}

fn element_name_id(tree: &mut Xot, local_name: &str, namespace: &str) -> xot::NameId {
    if namespace.is_empty() {
        tree.add_name(local_name)
    } else {
        let namespace = tree.add_namespace(namespace);
        tree.add_name_ns(local_name, namespace)
    }
}

pub(crate) fn append_atom(
    tree: &mut Xot,
    molecule_node: Node,
    names: &InsertionNames,
    atom_id: &PersistentId,
    atom: &super::MoleculeInsertionAtomV1,
) -> Result<(), TypedDocumentError> {
    let atom_node = tree.new_element(names.atom);
    tree.set_attribute(atom_node, names.id, atom_id.as_str());
    tree.set_attribute(atom_node, names.name, atom.element());
    set_optional_attribute(tree, atom_node, names.charge, atom.formal_charge());
    set_optional_attribute(tree, atom_node, names.isotope, atom.isotope());
    set_optional_attribute(
        tree,
        atom_node,
        names.explicit_hydrogens,
        atom.explicit_hydrogens(),
    );
    let point_node = tree.new_element(names.point);
    let position = atom.position();
    tree.set_attribute(point_node, names.x, position.x().to_string());
    tree.set_attribute(point_node, names.y, position.y().to_string());
    tree.set_attribute(point_node, names.z, position.z().to_string());
    tree.append(atom_node, point_node)
        .map_err(TypedDocumentError::Mutation)?;
    tree.append(molecule_node, atom_node)
        .map_err(TypedDocumentError::Mutation)
}

fn set_optional_attribute<T: ToString>(
    tree: &mut Xot,
    node: Node,
    name: xot::NameId,
    value: Option<T>,
) {
    if let Some(value) = value {
        tree.set_attribute(node, name, value.to_string());
    }
}

fn append_interchange_metadata(
    tree: &mut Xot,
    molecule_node: Node,
    record: &InterchangeRecordInsertionV1,
) -> Result<(), TypedDocumentError> {
    let namespace = tree.add_namespace(INTERCHANGE_IMPORT_NAMESPACE_V1);
    let record_name = tree.add_name_ns("interchange-record", namespace);
    let property_name = tree.add_name_ns("property", namespace);
    let encoding_name = tree.add_name("encoding");
    let title_name = tree.add_name("title");
    let name_name = tree.add_name("name");
    let value_name = tree.add_name("value");

    let record_node = tree.new_element(record_name);
    let prefix = tree.add_prefix("ferrum-interchange");
    tree.set_namespace(record_node, prefix, namespace);
    tree.set_attribute(record_node, encoding_name, "utf8-hex-v1");
    tree.set_attribute(record_node, title_name, utf8_hex(record.title()));
    for property in record.properties() {
        let property_node = tree.new_element(property_name);
        tree.set_attribute(property_node, name_name, utf8_hex(property.name()));
        tree.set_attribute(property_node, value_name, utf8_hex(property.value()));
        tree.append(record_node, property_node)
            .map_err(TypedDocumentError::Mutation)?;
    }
    tree.append(molecule_node, record_node)
        .map_err(TypedDocumentError::Mutation)
}

fn utf8_hex(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(value.len() * 2);
    for byte in value.bytes() {
        encoded.push(HEX[usize::from(byte >> 4)] as char);
        encoded.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    encoded
}

fn xml_attribute_safe(value: &str) -> bool {
    value.chars().all(|character| {
        matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
            || ('\u{20}'..='\u{D7FF}').contains(&character)
            || ('\u{E000}'..='\u{FFFD}').contains(&character)
            || ('\u{10000}'..='\u{10FFFF}').contains(&character)
    })
}
