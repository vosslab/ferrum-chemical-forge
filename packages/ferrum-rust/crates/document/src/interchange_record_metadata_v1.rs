//! Strict recovery of Ferrum-owned interchange metadata from one direct molecule.

use thiserror::Error;
use xot::{Node, Xot};

use super::{CDML_NAMESPACE, INTERCHANGE_IMPORT_NAMESPACE_V1, TypedClass, TypedDocument};

/// One exact ordered SDF property recovered from persisted CDML.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterchangePropertyMetadataV1 {
    name: String,
    value: String,
}

impl InterchangePropertyMetadataV1 {
    /// Return the exact decoded property name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return the exact decoded property value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Consume this property into its exact owned parts.
    #[must_use]
    pub fn into_parts(self) -> (String, String) {
        (self.name, self.value)
    }
}

/// Exact imported interchange record metadata retained below one molecule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterchangeRecordMetadataV1 {
    title: String,
    properties: Vec<InterchangePropertyMetadataV1>,
}

impl InterchangeRecordMetadataV1 {
    /// Return the exact imported title, including deliberate empty text.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return properties in exact persisted order, including repeated names.
    #[must_use]
    pub fn properties(&self) -> &[InterchangePropertyMetadataV1] {
        &self.properties
    }

    /// Consume this record into its exact owned title and property sequence.
    #[must_use]
    pub fn into_parts(self) -> (String, Vec<InterchangePropertyMetadataV1>) {
        (self.title, self.properties)
    }
}

/// Recover one direct molecule's optional Ferrum interchange import metadata.
///
/// An absent metadata child returns `Ok(None)`. A child in Ferrum's namespace
/// is authoritative and must match the closed `utf8-hex-v1` grammar exactly.
///
/// # Errors
///
/// Returns [`InterchangeRecordMetadataErrorV1`] when the selector is not one typed
/// direct molecule, authoritative metadata is duplicated or malformed, text
/// cannot be decoded exactly, or owned storage cannot be reserved.
pub fn observe_interchange_record_metadata_v1(
    document: &TypedDocument,
    molecule_source_id: &str,
) -> Result<Option<InterchangeRecordMetadataV1>, InterchangeRecordMetadataErrorV1> {
    let typed_match_count = document
        .root()
        .children_of(TypedClass::Molecule)
        .filter(|record| record.attribute("id") == Some(molecule_source_id))
        .count();
    if typed_match_count != 1 {
        return Err(InterchangeRecordMetadataErrorV1::UnknownDirectMolecule);
    }

    let xml = document.indexed().xml();
    let tree = &xml.tree;
    let root = tree
        .document_element(xml.document)
        .map_err(|_| InterchangeRecordMetadataErrorV1::MalformedRecord)?;
    let molecule = tree.children(root).find(|node| {
        is_cdml_element(tree, *node, "molecule")
            && unqualified_attribute(tree, *node, "id") == Some(molecule_source_id)
    });
    let Some(molecule) = molecule else {
        return Err(InterchangeRecordMetadataErrorV1::UnknownDirectMolecule);
    };

    let mut metadata_node = None;
    for child in tree.children(molecule) {
        if !is_element(
            tree,
            child,
            "interchange-record",
            INTERCHANGE_IMPORT_NAMESPACE_V1,
        ) {
            continue;
        }
        if metadata_node.replace(child).is_some() {
            return Err(InterchangeRecordMetadataErrorV1::DuplicateMetadata);
        }
    }
    metadata_node
        .map(|node| decode_record(tree, node))
        .transpose()
}

fn decode_record(
    tree: &Xot,
    node: Node,
) -> Result<InterchangeRecordMetadataV1, InterchangeRecordMetadataErrorV1> {
    let mut encoding = None;
    let mut title = None;
    for (name, value) in tree.attributes(node).iter() {
        let (local, namespace) = tree.name_ns_str(name);
        match (namespace, local) {
            ("", "encoding") => encoding = Some(value.as_str()),
            ("", "title") => title = Some(value.as_str()),
            _ => return Err(InterchangeRecordMetadataErrorV1::MalformedRecord),
        }
    }
    if encoding != Some("utf8-hex-v1") {
        return Err(InterchangeRecordMetadataErrorV1::UnsupportedEncoding);
    }
    let title = decode_utf8_hex(title.ok_or(InterchangeRecordMetadataErrorV1::MalformedRecord)?)?;
    if title.contains(['\0', '\r', '\n']) {
        return Err(InterchangeRecordMetadataErrorV1::InvalidTitle);
    }

    let mut properties = Vec::new();
    for child in tree.children(node) {
        if !is_element(tree, child, "property", INTERCHANGE_IMPORT_NAMESPACE_V1) {
            return Err(InterchangeRecordMetadataErrorV1::MalformedRecord);
        }
        properties
            .try_reserve(1)
            .map_err(|_| InterchangeRecordMetadataErrorV1::ResourceAllocation)?;
        properties.push(decode_property(tree, child)?);
    }
    Ok(InterchangeRecordMetadataV1 { title, properties })
}

fn decode_property(
    tree: &Xot,
    node: Node,
) -> Result<InterchangePropertyMetadataV1, InterchangeRecordMetadataErrorV1> {
    if tree.children(node).next().is_some() {
        return Err(InterchangeRecordMetadataErrorV1::MalformedProperty);
    }
    let mut name = None;
    let mut value = None;
    for (attribute, text) in tree.attributes(node).iter() {
        let (local, namespace) = tree.name_ns_str(attribute);
        match (namespace, local) {
            ("", "name") => name = Some(text.as_str()),
            ("", "value") => value = Some(text.as_str()),
            _ => return Err(InterchangeRecordMetadataErrorV1::MalformedProperty),
        }
    }
    let name = decode_utf8_hex(name.ok_or(InterchangeRecordMetadataErrorV1::MalformedProperty)?)?;
    let value = decode_utf8_hex(value.ok_or(InterchangeRecordMetadataErrorV1::MalformedProperty)?)?;
    if name.is_empty() || name.contains(['\0', '\r', '\n', '<', '>']) {
        return Err(InterchangeRecordMetadataErrorV1::InvalidPropertyName);
    }
    if value.as_bytes().contains(&0)
        || value.contains('\r')
        || value.contains("\n\n")
        || value.ends_with('\n')
        || value.lines().any(|line| line == "$$$$")
    {
        return Err(InterchangeRecordMetadataErrorV1::InvalidPropertyValue);
    }
    Ok(InterchangePropertyMetadataV1 { name, value })
}

fn decode_utf8_hex(value: &str) -> Result<String, InterchangeRecordMetadataErrorV1> {
    if !value.len().is_multiple_of(2)
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        return Err(InterchangeRecordMetadataErrorV1::InvalidHex);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(value.len() / 2)
        .map_err(|_| InterchangeRecordMetadataErrorV1::ResourceAllocation)?;
    for &[high, low] in value.as_bytes().as_chunks::<2>().0 {
        bytes.push((hex_value(high) << 4) | hex_value(low));
    }
    String::from_utf8(bytes).map_err(|_| InterchangeRecordMetadataErrorV1::InvalidUtf8)
}

const fn hex_value(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => 0,
    }
}

fn is_cdml_element(tree: &Xot, node: Node, expected_local: &str) -> bool {
    let Some(element) = tree.element(node) else {
        return false;
    };
    let (local, namespace) = tree.name_ns_str(element.name());
    local == expected_local && (namespace == CDML_NAMESPACE)
}

fn is_element(tree: &Xot, node: Node, expected_local: &str, expected_namespace: &str) -> bool {
    let Some(element) = tree.element(node) else {
        return false;
    };
    let (local, namespace) = tree.name_ns_str(element.name());
    local == expected_local && namespace == expected_namespace
}

fn unqualified_attribute<'a>(tree: &'a Xot, node: Node, expected_local: &str) -> Option<&'a str> {
    tree.attributes(node).iter().find_map(|(name, value)| {
        let (local, namespace) = tree.name_ns_str(name);
        (local == expected_local && namespace.is_empty()).then_some(value.as_str())
    })
}

/// Rejection of a direct-root selector or authoritative persisted interchange metadata.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum InterchangeRecordMetadataErrorV1 {
    /// The source ID does not identify exactly one typed direct molecule.
    #[error("interchange metadata selector is not one typed direct molecule")]
    UnknownDirectMolecule,
    /// One molecule carries more than one authoritative metadata record.
    #[error("direct molecule carries duplicate Ferrum interchange metadata records")]
    DuplicateMetadata,
    /// The authoritative metadata record has unsupported attributes or children.
    #[error("Ferrum interchange metadata record structure is malformed")]
    MalformedRecord,
    /// The authoritative property has unsupported attributes or children.
    #[error("Ferrum interchange metadata property structure is malformed")]
    MalformedProperty,
    /// The retained encoding is absent or not the closed V1 spelling.
    #[error("Ferrum interchange metadata encoding is not utf8-hex-v1")]
    UnsupportedEncoding,
    /// One retained field is not lowercase even-length hexadecimal text.
    #[error("Ferrum interchange metadata contains invalid hexadecimal text")]
    InvalidHex,
    /// One decoded field is not UTF-8.
    #[error("Ferrum interchange metadata contains invalid UTF-8")]
    InvalidUtf8,
    /// The decoded title violates the persisted single-line grammar.
    #[error("Ferrum interchange metadata title is invalid")]
    InvalidTitle,
    /// A decoded property name violates the persisted field grammar.
    #[error("Ferrum interchange metadata property name is invalid")]
    InvalidPropertyName,
    /// A decoded property value violates the persisted field grammar.
    #[error("Ferrum interchange metadata property value is invalid")]
    InvalidPropertyValue,
    /// Exact decoded metadata storage could not be reserved.
    #[error("Ferrum interchange metadata storage could not be reserved")]
    ResourceAllocation,
}
