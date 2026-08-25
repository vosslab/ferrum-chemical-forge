//! Bounded saved-template admission and authored-scale molecule placement.

use std::collections::HashSet;

use ferrum_geometry::Point2;
use thiserror::Error;

use super::{
    CDML_NAMESPACE, DocumentClipboardPasteErrorV1, DocumentClipboardPastePlanV1,
    DocumentObjectIdV1, PersistentId, ProjectionError, TopLevelRootKindV1, TopLevelRootSelectorV1,
    TopLevelTransformModeV1, TopLevelTransformV1, TopLevelTransformV1Error, TypedClass,
    TypedDocument, TypedDocumentError, TypedRecord, UnrecognizedNode, XmlInputBudgetV1,
    XmlSerializationError, element_name,
};

/// Stable schema identifier for one admitted native user template.
pub const DOCUMENT_USER_TEMPLATE_SCHEMA_V1: &str = "ferrum-document-user-template-v1";

/// Immutable, handle-free result of one bounded user-template inspection.
#[derive(Debug)]
pub struct DocumentUserTemplatePlanV1 {
    schema: &'static str,
    display_name: Option<String>,
    atom_centroid: Point2,
    insertion_plan: DocumentClipboardPastePlanV1,
}

impl DocumentUserTemplatePlanV1 {
    /// Return the closed user-template schema.
    #[must_use]
    pub const fn schema(&self) -> &'static str {
        self.schema
    }

    /// Return the trimmed authored molecule name when one is present.
    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    /// Return the finite source atom centroid in document points.
    #[must_use]
    pub const fn atom_centroid(&self) -> Point2 {
        self.atom_centroid
    }

    pub(super) fn declared_id_count(&self) -> usize {
        self.insertion_plan.declared_id_count()
    }
}

/// The durable molecule identity created by one accepted template insertion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentUserTemplateInsertedMoleculeV1 {
    object_id: DocumentObjectIdV1,
    source_id: PersistentId,
}

impl DocumentUserTemplateInsertedMoleculeV1 {
    /// Return the opaque durable selector for the inserted molecule.
    #[must_use]
    pub fn object_id(&self) -> &DocumentObjectIdV1 {
        &self.object_id
    }

    /// Return the fresh persistent XML identity installed on the molecule.
    #[must_use]
    pub fn source_id(&self) -> &PersistentId {
        &self.source_id
    }
}

/// Failure while inspecting or composing one saved user template.
#[derive(Debug, Error)]
pub enum DocumentUserTemplateErrorV1 {
    /// The caller's installed observation no longer matches the session state.
    #[error("user template insertion expected a different document digest")]
    DigestMismatch,
    /// XML admission, identity, or typed recognition failed.
    #[error("user template is invalid CDML: {0}")]
    Typed(#[from] TypedDocumentError),
    /// The document root contained comments, processing instructions, or opaque content.
    #[error("user template has unsupported root-level content")]
    UnsupportedRootContent,
    /// A direct persistent root was not a molecule or optional context envelope.
    #[error("user template has an unsupported direct persistent root")]
    UnsupportedRoot,
    /// The document contained more than one Standard or Paper context record.
    #[error("user template has a repeated {0} context envelope")]
    RepeatedContext(&'static str),
    /// Eligibility requires exactly one direct molecule.
    #[error("user template requires exactly one direct molecule")]
    MoleculeCardinality,
    /// Legacy attachment markers are not reusable molecule content.
    #[error("user template molecule contains a legacy template attachment marker")]
    LegacyTemplateMarker,
    /// Centroid placement requires at least one direct atom.
    #[error("user template molecule requires at least one direct atom")]
    MissingAtom,
    /// One direct atom did not carry exactly one direct point.
    #[error("user template atom requires exactly one direct point")]
    AtomPointCardinality,
    /// One atom point was not valid finite document geometry.
    #[error("user template atom point is invalid: {0}")]
    AtomGeometry(#[source] ProjectionError),
    /// The complete recognized molecule geometry is not safely translatable.
    #[error("user template has unsupported recognized coordinate geometry: {0}")]
    Geometry(#[source] TypedDocumentError),
    /// A recognized mark retained nested element content outside the placement grammar.
    #[error("user template mark has unsupported nested geometry")]
    MarkGeometry,
    /// A recognized molecule-local reference did not resolve inside that molecule.
    #[error("user template {field} reference must resolve inside its molecule")]
    ExternalReference { field: &'static str },
    /// A finite centroid could not be represented after bounded aggregation.
    #[error("user template atom centroid is not representable")]
    CentroidUnrepresentable,
    /// The retained tree refused a structural edit while isolating the molecule.
    #[error("user template molecule isolation failed: {0}")]
    Mutation(#[source] xot::Error),
    /// The isolated molecule could not be serialized.
    #[error("user template molecule isolation could not serialize CDML: {0}")]
    Serialize(#[from] XmlSerializationError),
    /// A supposedly validated root selector could not be represented.
    #[error("user template molecule selector is invalid: {0}")]
    TransformRequest(#[from] TopLevelTransformV1Error),
    /// Finite source and anchor values produced a non-finite displacement.
    #[error("user template placement displacement is not representable")]
    PlacementUnrepresentable,
    /// The shared retained-fragment composer rejected an already-admitted molecule.
    #[error("user template candidate composition failed")]
    CandidateComposition(#[source] DocumentClipboardPasteErrorV1),
    /// Candidate composition did not return exactly one durable molecule.
    #[error("user template inserted-molecule identity invariant failed")]
    InsertedMoleculeInvariant,
}

/// Inspect one complete external CDML template under an explicit caller-owned budget.
pub fn prepare_document_user_template_v1(
    source: &str,
    budget: XmlInputBudgetV1,
) -> Result<DocumentUserTemplatePlanV1, DocumentUserTemplateErrorV1> {
    let document = TypedDocument::parse_with_budget(source, budget)?;
    let molecule = eligible_molecule(&document)?;
    let display_name = molecule
        .attribute("name")
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_owned);
    reject_legacy_template_marker(molecule)?;
    reject_nested_mark_geometry(molecule)?;

    let molecule_document = isolate_molecule(document)?;
    let molecule = molecule_document
        .root()
        .children_of(TypedClass::Molecule)
        .next()
        .ok_or(DocumentUserTemplateErrorV1::MoleculeCardinality)?;
    validate_internal_references(molecule)?;
    let atom_centroid = atom_centroid(molecule)?;
    validate_complete_geometry(&molecule_document, molecule)?;

    let insertion_plan =
        super::clipboard_paste_v1::prepare_admitted_document_clipboard_paste_v1(molecule_document)
            .map_err(DocumentUserTemplateErrorV1::CandidateComposition)?;
    Ok(DocumentUserTemplatePlanV1 {
        schema: DOCUMENT_USER_TEMPLATE_SCHEMA_V1,
        display_name,
        atom_centroid,
        insertion_plan,
    })
}

fn eligible_molecule(
    document: &TypedDocument,
) -> Result<&TypedRecord, DocumentUserTemplateErrorV1> {
    for child in document.root().unrecognized_children() {
        match child.node() {
            UnrecognizedNode::Text(text) if text.trim().is_empty() => {}
            _ => return Err(DocumentUserTemplateErrorV1::UnsupportedRootContent),
        }
    }
    let mut standard_seen = false;
    let mut paper_seen = false;
    let mut molecules = Vec::new();
    for child in document.root().typed_children() {
        let record = child.record();
        match record.class() {
            TypedClass::Standard if standard_seen => {
                return Err(DocumentUserTemplateErrorV1::RepeatedContext("Standard"));
            }
            TypedClass::Standard => standard_seen = true,
            TypedClass::Paper if paper_seen => {
                return Err(DocumentUserTemplateErrorV1::RepeatedContext("Paper"));
            }
            TypedClass::Paper => paper_seen = true,
            TypedClass::Molecule => molecules.push(record),
            _ => return Err(DocumentUserTemplateErrorV1::UnsupportedRoot),
        }
    }
    let [molecule] = molecules.as_slice() else {
        return Err(DocumentUserTemplateErrorV1::MoleculeCardinality);
    };
    Ok(molecule)
}

fn reject_legacy_template_marker(
    molecule: &TypedRecord,
) -> Result<(), DocumentUserTemplateErrorV1> {
    if molecule
        .typed_children()
        .iter()
        .any(|child| child.record().class() == TypedClass::Template)
    {
        return Err(DocumentUserTemplateErrorV1::LegacyTemplateMarker);
    }
    Ok(())
}

fn reject_nested_mark_geometry(molecule: &TypedRecord) -> Result<(), DocumentUserTemplateErrorV1> {
    for vertex in molecule.typed_children().iter().map(|child| child.record()) {
        if !matches!(
            vertex.class(),
            TypedClass::Atom | TypedClass::Group | TypedClass::MoleculeText | TypedClass::Query
        ) {
            continue;
        }
        for mark in vertex.children_of(TypedClass::Mark) {
            if mark
                .unrecognized_children()
                .iter()
                .any(|child| matches!(child.node(), UnrecognizedNode::Element { .. }))
            {
                return Err(DocumentUserTemplateErrorV1::MarkGeometry);
            }
        }
    }
    Ok(())
}

fn isolate_molecule(
    mut document: TypedDocument,
) -> Result<TypedDocument, DocumentUserTemplateErrorV1> {
    let indexed = document.detached_indexed_mut();
    let root = indexed
        .xml
        .tree
        .document_element(indexed.xml.document)
        .map_err(DocumentUserTemplateErrorV1::Mutation)?;
    let element_children = indexed
        .xml
        .tree
        .children(root)
        .filter(|node| indexed.xml.tree.element(*node).is_some())
        .collect::<Vec<_>>();
    for child in element_children {
        let keep = element_name(&indexed.xml.tree, child)
            .is_some_and(|(name, namespace)| name == "molecule" && (namespace == CDML_NAMESPACE));
        if !keep {
            indexed
                .xml
                .tree
                .remove(child)
                .map_err(DocumentUserTemplateErrorV1::Mutation)?;
        }
    }
    TypedDocument::parse(&document.to_xml()?).map_err(Into::into)
}

fn validate_internal_references(molecule: &TypedRecord) -> Result<(), DocumentUserTemplateErrorV1> {
    let definitions = molecule
        .typed_children()
        .iter()
        .map(|child| child.record())
        .filter(|record| {
            matches!(
                record.class(),
                TypedClass::Atom
                    | TypedClass::Group
                    | TypedClass::MoleculeText
                    | TypedClass::Query
                    | TypedClass::Bond
                    | TypedClass::Fragment
            )
        })
        .filter_map(|record| record.attribute("id"))
        .collect::<HashSet<_>>();
    for child in molecule.typed_children() {
        let record = child.record();
        if record.class() == TypedClass::Bond {
            require_internal_reference(record, "start", &definitions)?;
            require_internal_reference(record, "end", &definitions)?;
        }
        if record.class() == TypedClass::Fragment {
            for member in record.typed_children() {
                if matches!(
                    member.record().class(),
                    TypedClass::FragmentBond | TypedClass::FragmentVertex
                ) {
                    require_internal_reference(member.record(), "id", &definitions)?;
                }
            }
        }
    }
    Ok(())
}

fn require_internal_reference(
    record: &TypedRecord,
    field: &'static str,
    definitions: &HashSet<&str>,
) -> Result<(), DocumentUserTemplateErrorV1> {
    if record
        .attribute(field)
        .is_none_or(|reference| !definitions.contains(reference))
    {
        return Err(DocumentUserTemplateErrorV1::ExternalReference { field });
    }
    Ok(())
}

fn atom_centroid(molecule: &TypedRecord) -> Result<Point2, DocumentUserTemplateErrorV1> {
    let atoms = molecule
        .typed_children()
        .iter()
        .map(|child| child.record())
        .filter(|record| record.class() == TypedClass::Atom)
        .collect::<Vec<_>>();
    if atoms.is_empty() {
        return Err(DocumentUserTemplateErrorV1::MissingAtom);
    }
    let points = atoms
        .into_iter()
        .map(|atom| {
            if atom
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.child_class() == TypedClass::Point)
            {
                return Err(DocumentUserTemplateErrorV1::AtomPointCardinality);
            }
            let points = atom.children_of(TypedClass::Point).collect::<Vec<_>>();
            let [point] = points.as_slice() else {
                return Err(DocumentUserTemplateErrorV1::AtomPointCardinality);
            };
            super::projection_v1::point(point).map_err(DocumentUserTemplateErrorV1::AtomGeometry)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let x = stable_mean(points.iter().map(|point| point.x()))?;
    let y = stable_mean(points.iter().map(|point| point.y()))?;
    Point2::new(x, y).map_err(|_| DocumentUserTemplateErrorV1::CentroidUnrepresentable)
}

fn stable_mean(
    values: impl ExactSizeIterator<Item = f64>,
) -> Result<f64, DocumentUserTemplateErrorV1> {
    let count = u32::try_from(values.len())
        .map_err(|_| DocumentUserTemplateErrorV1::CentroidUnrepresentable)?;
    let values = values.collect::<Vec<_>>();
    let scale = values
        .iter()
        .copied()
        .map(f64::abs)
        .reduce(f64::max)
        .ok_or(DocumentUserTemplateErrorV1::CentroidUnrepresentable)?;
    if scale == 0.0 {
        return Ok(0.0);
    }
    let mut sum = 0.0;
    let mut compensation = 0.0;
    for value in values {
        let adjusted = value / scale - compensation;
        let next = sum + adjusted;
        compensation = (next - sum) - adjusted;
        sum = next;
    }
    let normalized = (sum / f64::from(count)).clamp(-1.0, 1.0);
    let mean = normalized * scale;
    mean.is_finite()
        .then_some(mean)
        .ok_or(DocumentUserTemplateErrorV1::CentroidUnrepresentable)
}

fn validate_complete_geometry(
    document: &TypedDocument,
    molecule: &TypedRecord,
) -> Result<(), DocumentUserTemplateErrorV1> {
    let selector = TopLevelRootSelectorV1::new(
        crate::document_object_id_from_record_v1(molecule)
            .ok_or(DocumentUserTemplateErrorV1::MoleculeCardinality)?,
        TopLevelRootKindV1::Molecule,
    );
    let request = TopLevelTransformV1::new(
        vec![selector],
        TopLevelTransformModeV1::Translate { dx: 0.0, dy: 0.0 },
    )?;
    document
        .with_top_level_transform(&request)
        .map(|_| ())
        .map_err(DocumentUserTemplateErrorV1::Geometry)
}

pub(super) fn compose_document_user_template_candidate_v1(
    current: &TypedDocument,
    plan: &DocumentUserTemplatePlanV1,
    generated_ids: &[PersistentId],
    anchor: Point2,
) -> Result<(TypedDocument, DocumentUserTemplateInsertedMoleculeV1), DocumentUserTemplateErrorV1> {
    let dx = anchor.x() - plan.atom_centroid.x();
    let dy = anchor.y() - plan.atom_centroid.y();
    if !dx.is_finite() || !dy.is_finite() {
        return Err(DocumentUserTemplateErrorV1::PlacementUnrepresentable);
    }
    let (candidate, roots) = super::clipboard_paste_v1::compose_clipboard_paste_candidate_v1(
        current,
        &plan.insertion_plan,
        generated_ids,
        dx,
        dy,
    )
    .map_err(DocumentUserTemplateErrorV1::CandidateComposition)?;
    let [root] = roots.as_slice() else {
        return Err(DocumentUserTemplateErrorV1::InsertedMoleculeInvariant);
    };
    if root.kind() != TopLevelRootKindV1::Molecule {
        return Err(DocumentUserTemplateErrorV1::InsertedMoleculeInvariant);
    }
    Ok((
        candidate,
        DocumentUserTemplateInsertedMoleculeV1 {
            object_id: root.object_id().clone(),
            source_id: root.source_id().clone(),
        },
    ))
}
