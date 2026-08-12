use std::collections::BTreeMap;

use ferrum_document::{TypedDocument, TypedRecord};
use xot::{Node, Value, Xot};

use crate::errors::CdmlError;
use crate::reports::{
    CdmlInspection, CdmlValidation, INSPECTION_SCHEMA, MoleculeInspection, REWRITE_CHECK_SCHEMA,
    RewriteCheck, VALIDATION_SCHEMA,
};

/// Inspect one CDML document through the typed Ferrum core model.
pub fn inspect_cdml(source: &str) -> Result<CdmlInspection, CdmlError> {
    let document = TypedDocument::parse(source)?;
    let projection = document.core_projection()?;
    let observation = DocumentObservation::from_document(&document);
    let molecules = projection
        .molecules()
        .iter()
        .map(|molecule| MoleculeInspection {
            source_id: molecule.source_id().map(|value| value.as_str().to_owned()),
            name: molecule.name().map(str::to_owned),
            atom_count: molecule.atoms().len(),
            group_count: molecule.groups().len(),
            text_count: molecule.texts().len(),
            query_count: molecule.queries().len(),
            bond_count: molecule.bonds().len(),
        })
        .collect();
    Ok(CdmlInspection {
        schema: INSPECTION_SCHEMA,
        document_version: projection.document_version().map(str::to_owned),
        persistent_id_count: observation.persistent_id_count,
        top_level_record_count: observation.top_level_records.len(),
        typed_record_counts: observation.typed_record_counts,
        diagnostic_count: observation.diagnostic_count,
        molecules,
    })
}

/// Validate CDML structurally, optionally requiring the current core projection.
pub fn validate_cdml(
    source: &str,
    require_core_projection: bool,
) -> Result<CdmlValidation, CdmlError> {
    let document = TypedDocument::parse(source)?;
    let observation = DocumentObservation::from_document(&document);
    let document_version = if require_core_projection {
        document
            .core_projection()?
            .document_version()
            .map(str::to_owned)
    } else {
        document.root().attribute("version").map(str::to_owned)
    };
    Ok(CdmlValidation {
        schema: VALIDATION_SCHEMA,
        valid: true,
        level: if require_core_projection {
            "core"
        } else {
            "structural"
        },
        document_version,
        persistent_id_count: observation.persistent_id_count,
        top_level_record_count: observation.top_level_records.len(),
        diagnostic_count: observation.diagnostic_count,
    })
}

/// Parse and structurally re-emit one CDML document.
pub fn rewrite_cdml(source: &str) -> Result<String, CdmlError> {
    Ok(prepare_rewrite(source)?.rewritten)
}

/// Verify the documented structural-preservation contract over a rewrite cycle.
pub fn verify_cdml_rewrite(source: &str) -> Result<RewriteCheck, CdmlError> {
    let transaction = prepare_rewrite(source)?;
    Ok(RewriteCheck {
        schema: REWRITE_CHECK_SCHEMA,
        valid: true,
        persistent_id_count: transaction.observation.persistent_id_count,
        top_level_record_count: transaction.observation.top_level_records.len(),
        typed_record_counts: transaction.observation.typed_record_counts,
        opaque_child_count: transaction.observation.opaque_child_count,
    })
}

/// Build the sole validated rewrite transaction used by checking and publication.
///
/// The comparison deliberately accepts serializer-normalized prefixes and attribute
/// order. It retains every parsed node kind, expanded element and attribute name,
/// attribute and text value, namespace URI context, and ordered child position.
/// The typed observation additionally checks persistent identity, typed classes,
/// diagnostics, and direct-record order.
fn prepare_rewrite(source: &str) -> Result<RewriteTransaction, CdmlError> {
    let original = TypedDocument::parse(source)?;
    let original_shape = XmlShape::parse(source)?;
    let observation = DocumentObservation::from_document(&original);
    let rewritten = original.to_xml()?;
    let reparsed = TypedDocument::parse(&rewritten)?;
    let reparsed_shape = XmlShape::parse(&rewritten)?;
    let reparsed_observation = DocumentObservation::from_document(&reparsed);
    if original_shape != reparsed_shape || observation != reparsed_observation {
        return Err(CdmlError::StructuralPreservation);
    }
    Ok(RewriteTransaction {
        rewritten,
        observation,
    })
}

struct RewriteTransaction {
    rewritten: String,
    observation: DocumentObservation,
}

/// A prefix-independent structural snapshot of XML retained by Ferrum.
#[derive(Debug, Eq, PartialEq)]
enum XmlShape {
    Document(Vec<Self>),
    Element {
        namespace: String,
        local_name: String,
        attributes: Vec<(String, String, String)>,
        namespace_uris: Vec<String>,
        children: Vec<Self>,
    },
    Text(String),
    Comment(String),
    ProcessingInstruction {
        namespace: String,
        local_name: String,
        data: Option<String>,
    },
}

impl XmlShape {
    fn parse(source: &str) -> Result<Self, CdmlError> {
        let mut tree = Xot::new();
        let document = tree.parse(source).map_err(CdmlError::StructuralSnapshot)?;
        Ok(Self::from_node(&tree, document))
    }

    fn from_node(tree: &Xot, node: Node) -> Self {
        match tree.value(node) {
            Value::Document => Self::Document(
                tree.children(node)
                    .map(|child| Self::from_node(tree, child))
                    .collect(),
            ),
            Value::Element(element) => {
                let (local_name, namespace) = tree.name_ns_str(element.name());
                let mut attributes = tree
                    .attributes(node)
                    .iter()
                    .map(|(name, value)| {
                        let (local_name, namespace) = tree.name_ns_str(name);
                        (namespace.to_owned(), local_name.to_owned(), value.clone())
                    })
                    .collect::<Vec<_>>();
                attributes.sort();
                let mut namespace_uris = tree
                    .namespaces_in_scope(node)
                    .map(|(_, namespace)| tree.namespace_str(namespace).to_owned())
                    .collect::<Vec<_>>();
                namespace_uris.sort();
                Self::Element {
                    namespace: namespace.to_owned(),
                    local_name: local_name.to_owned(),
                    attributes,
                    namespace_uris,
                    children: tree
                        .children(node)
                        .map(|child| Self::from_node(tree, child))
                        .collect(),
                }
            }
            Value::Text(text) => Self::Text(text.get().to_owned()),
            Value::Comment(comment) => Self::Comment(comment.get().to_owned()),
            Value::ProcessingInstruction(instruction) => {
                let (local_name, namespace) = tree.name_ns_str(instruction.target());
                Self::ProcessingInstruction {
                    namespace: namespace.to_owned(),
                    local_name: local_name.to_owned(),
                    data: instruction.data().map(str::to_owned),
                }
            }
            Value::Attribute(_) | Value::Namespace(_) => {
                panic!("attribute and namespace nodes are not normal children")
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct DocumentObservation {
    persistent_id_count: usize,
    top_level_records: Vec<(u32, Option<String>)>,
    typed_record_counts: BTreeMap<&'static str, usize>,
    diagnostic_count: usize,
    opaque_child_count: usize,
}

impl DocumentObservation {
    fn from_document(document: &TypedDocument) -> Self {
        let mut typed_record_counts = BTreeMap::new();
        let mut diagnostic_count = 0;
        let mut opaque_child_count = 0;
        count_typed_records(
            document.root(),
            &mut typed_record_counts,
            &mut diagnostic_count,
            &mut opaque_child_count,
        );
        let top_level_records = document
            .indexed()
            .records()
            .iter()
            .map(|record| {
                (
                    record.source_order().value(),
                    record
                        .identifier()
                        .map(|identifier| identifier.as_str().to_owned()),
                )
            })
            .collect();
        Self {
            persistent_id_count: document.indexed().persistent_id_count(),
            top_level_records,
            typed_record_counts,
            diagnostic_count,
            opaque_child_count,
        }
    }
}

fn count_typed_records(
    record: &TypedRecord,
    counts: &mut BTreeMap<&'static str, usize>,
    diagnostics: &mut usize,
    opaque_children: &mut usize,
) {
    *counts.entry(record.class().name()).or_default() += 1;
    *diagnostics += record.diagnostics().len();
    *opaque_children += record.unrecognized_children().len();
    for child in record.typed_children() {
        count_typed_records(child.record(), counts, diagnostics, opaque_children);
    }
}

#[cfg(test)]
mod tests {
    use super::{XmlShape, inspect_cdml, validate_cdml, verify_cdml_rewrite};

    const SIMPLE_CDML: &str = r#"<cdml version="0.16"><molecule id="m1"><atom id="a1" name="C"><point x="1" y="2"/></atom></molecule></cdml>"#;

    #[test]
    fn inspection_requires_core_projection_and_reports_owned_facts() {
        let inspection = inspect_cdml(SIMPLE_CDML).expect("valid CDML inspects");

        assert_eq!(inspection.schema, "ferrum-cdml-inspection-v1");
        assert_eq!(inspection.document_version.as_deref(), Some("0.16"));
        assert_eq!(inspection.persistent_id_count, 2);
        assert_eq!(inspection.typed_record_counts["molecule/atom"], 1);
        assert_eq!(inspection.molecules[0].atom_count, 1);
    }

    #[test]
    fn structural_validation_does_not_require_resolved_core_endpoints() {
        let source = r#"<cdml><molecule><atom id="a1"><point x="0" y="0"/></atom><bond start="a1" end="missing"/></molecule></cdml>"#;

        let structural = validate_cdml(source, false).expect("document structure validates");
        let error = validate_cdml(source, true).expect_err("core projection rejects endpoint");

        assert_eq!(structural.level, "structural");
        assert!(error.to_string().contains("unknown molecule-local vertex"));
    }

    #[test]
    fn rewrite_check_observes_retained_opaque_payload() {
        let source =
            r#"<cdml xmlns:q="urn:test"><q:payload id="foreign"><q:item/></q:payload></cdml>"#;
        let check = verify_cdml_rewrite(source).expect("opaque payload survives");

        assert!(check.valid);
        assert_eq!(check.opaque_child_count, 1);
        assert_eq!(check.persistent_id_count, 1);
    }

    #[test]
    fn structural_shape_rejects_same_count_different_retained_facts() {
        let original = XmlShape::parse(
            r#"<cdml xmlns:q="urn:vendor"><molecule id="m1"><atom id="a1" name="C"><point x="1" y="2"/></atom></molecule><q:payload state="kept">text</q:payload></cdml>"#,
        )
        .expect("inline CDML parses");
        let changed = XmlShape::parse(
            r#"<cdml xmlns:q="urn:vendor"><molecule id="m1"><atom id="a1" name="N"><point x="1" y="2"/></atom></molecule><q:payload state="changed">text</q:payload></cdml>"#,
        )
        .expect("inline CDML parses");

        assert_ne!(original, changed);
    }
}
