//! CDML name tables and non-stereo molecule XML serialization.

use xot::{Node, Xot};

use super::super::{
    CDML_NAMESPACE, INTERCHANGE_IMPORT_NAMESPACE_V1, InterchangeRecordInsertionV1,
    MoleculeInsertionAtomV1, PersistentId, TypedDocumentError,
};

pub(crate) fn valid_cdml_namespace(namespace: String) -> String {
    if namespace == CDML_NAMESPACE {
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
    pub(crate) stereo_semantics: xot::NameId,
    pub(crate) stereo_depictions: xot::NameId,
    pub(crate) tetrahedral: xot::NameId,
    pub(crate) double_bond_stereo: xot::NameId,
    pub(crate) tetrahedral_directed_bond: xot::NameId,
    pub(crate) double_bond_carrier_mark: xot::NameId,
    pub(crate) center: xot::NameId,
    pub(crate) ligands: xot::NameId,
    pub(crate) parity: xot::NameId,
    pub(crate) bond_index: xot::NameId,
    pub(crate) start_ligand: xot::NameId,
    pub(crate) end_ligand: xot::NameId,
    pub(crate) configuration: xot::NameId,
    pub(crate) double_bond_index: xot::NameId,
    pub(crate) carrier_bond_index: xot::NameId,
    pub(crate) presentation: xot::NameId,
    pub(crate) mark: xot::NameId,
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
            stereo_semantics: element_name_id(tree, "stereoSemantics", &namespace),
            stereo_depictions: element_name_id(tree, "stereoDepictions", &namespace),
            tetrahedral: element_name_id(tree, "tetrahedral", &namespace),
            double_bond_stereo: element_name_id(tree, "doubleBond", &namespace),
            tetrahedral_directed_bond: element_name_id(tree, "tetrahedralDirectedBond", &namespace),
            double_bond_carrier_mark: element_name_id(tree, "doubleBondCarrierMark", &namespace),
            center: tree.add_name("center"),
            ligands: tree.add_name("ligands"),
            parity: tree.add_name("parity"),
            bond_index: tree.add_name("bondIndex"),
            start_ligand: tree.add_name("startLigand"),
            end_ligand: tree.add_name("endLigand"),
            configuration: tree.add_name("configuration"),
            double_bond_index: tree.add_name("doubleBondIndex"),
            carrier_bond_index: tree.add_name("carrierBondIndex"),
            presentation: tree.add_name("presentation"),
            mark: tree.add_name("mark"),
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
    atom: &MoleculeInsertionAtomV1,
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

pub(super) fn append_interchange_metadata(
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

pub(super) fn xml_attribute_safe(value: &str) -> bool {
    value.chars().all(|character| {
        matches!(character, '\u{9}' | '\u{A}' | '\u{D}')
            || ('\u{20}'..='\u{D7FF}').contains(&character)
            || ('\u{E000}'..='\u{FFFD}').contains(&character)
            || ('\u{10000}'..='\u{10FFFF}').contains(&character)
    })
}
