//! Typed-CDML to immutable lower-projection adaptation.

use std::collections::{BTreeSet, HashMap};

use ferrum_core::{BondOrder, BondStyle};
use ferrum_document_projection::{
    AtomProjectionV1, BondEndpointKindV1, BondEndpointV1, BondProjectionV1,
    DocumentDirectRootKindV1, DocumentDirectRootV1, DocumentHaworthPositionV1, DocumentObjectIdV1,
    DocumentProjectionProvenanceV1, DocumentProjectionV1, DocumentProjectionV1Error,
    DrawingStandardV1, FontFactsV1, MoleculeProjectionChildrenV1, MoleculeProjectionV1,
    NonAtomVertexKindV1, NonAtomVertexProjectionV1, NonZeroFiniteV1, Point3V1, PositiveFiniteV1,
    PresentationLengthV1, PresentationProjectionIssueV1, PresentationRootProjectionV1,
    ProjectionError, ProjectionIssueCodeV1, ProjectionIssueV1, Rgb24V1, RichTextV1,
    TransparentOrRgb24V1, VisibilityV1,
};

use crate::atom_mark_projection::atom_marks;
use crate::compact_group_projection_v1::compact_group;
use crate::{DocumentSnapshot, TypedChild, TypedClass, TypedDocument, TypedRecord};

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

/// Build one lower immutable projection from its sole authoritative snapshot.
pub(crate) fn document_projection_from_snapshot_v1(
    snapshot: &DocumentSnapshot,
) -> Result<DocumentProjectionV1, ProjectionError> {
    let document =
        TypedDocument::parse(snapshot.cdml()).map_err(|error| ProjectionError::InvalidValue {
            context: "document snapshot".to_owned(),
            field: "cdml",
            value: error.to_string(),
        })?;
    let mut issues = Vec::new();
    let mut persisted_standard = None;
    let mut molecules = Vec::new();
    let mut direct_roots = Vec::new();
    let mut presentation_roots = Vec::new();
    let mut presentation_issues = Vec::new();
    let presentation_context =
        crate::presentation_stack_projection_v1::PresentationProjectionContextV1::new(&document)?;
    let version = document.root().attribute("version");
    for child in document.root().typed_children() {
        match child.record().class() {
            TypedClass::Standard if persisted_standard.is_none() => {
                persisted_standard = Some(drawing_standard(child.record(), &mut issues)?);
            }
            TypedClass::Molecule => {
                let id = direct_root_id(child.record())?;
                molecules.push(molecule(child, version, &mut issues)?);
                direct_roots.push(DocumentDirectRootV1::new(
                    id,
                    child.position(),
                    DocumentDirectRootKindV1::Molecule,
                ));
            }
            class if crate::presentation_stack_projection_v1::is_presentation_class_v1(class) => {
                let target =
                    crate::presentation_stack_projection_v1::presentation_target_from_child_v1(
                        child,
                    )?;
                let issue_start = presentation_issues.len();
                match presentation_context.project_root(child, &mut presentation_issues)? {
                    Some(root) => {
                        direct_roots.push(DocumentDirectRootV1::new(
                            target.document_object_id().clone(),
                            child.position(),
                            DocumentDirectRootKindV1::Presentation(target.record_kind()),
                        ));
                        presentation_roots.push(root);
                    }
                    None => {
                        let issue = presentation_issues[issue_start..]
                            .iter()
                            .rev()
                            .find(|issue| issue.target() == &target)
                            .ok_or_else(|| ProjectionError::InvalidValue {
                                context: child.record().path().to_string(),
                                field: "presentation projection",
                                value: "rejected presentation has no target issue".to_owned(),
                            })?;
                        direct_roots.push(DocumentDirectRootV1::new(
                            target.document_object_id().clone(),
                            child.position(),
                            DocumentDirectRootKindV1::RejectedPresentation(issue.code()),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    let provenance = DocumentProjectionProvenanceV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        snapshot.is_dirty(),
    );
    let paper_layout = crate::paper_properties_v1::paper_layout_from_snapshot(&document, snapshot);
    validate_direct_root_payloads(
        &direct_roots,
        &molecules,
        &presentation_roots,
        &presentation_issues,
    )?;
    let presentation_stack =
        presentation_context.into_stack(snapshot, presentation_roots, presentation_issues)?;
    DocumentProjectionV1::try_new(
        provenance,
        persisted_standard,
        paper_layout,
        molecules,
        direct_roots,
        presentation_stack,
        issues,
    )
    .map_err(projection_aggregate_error)
}

fn direct_root_id(record: &TypedRecord) -> Result<DocumentObjectIdV1, ProjectionError> {
    crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)
}

fn validate_direct_root_payloads(
    direct_roots: &[DocumentDirectRootV1],
    molecules: &[MoleculeProjectionV1],
    presentation_roots: &[PresentationRootProjectionV1],
    presentation_issues: &[PresentationProjectionIssueV1],
) -> Result<(), ProjectionError> {
    let direct_ids = direct_roots
        .iter()
        .map(|root| root.document_object_id().as_str())
        .collect::<BTreeSet<_>>();
    for molecule in molecules {
        require_direct_root(
            &direct_ids,
            molecule.document_object_id(),
            "molecule payload",
        )?;
    }
    for root in presentation_roots {
        require_direct_root(
            &direct_ids,
            root.target().document_object_id(),
            "presentation payload",
        )?;
    }
    for issue in presentation_issues {
        require_direct_root(
            &direct_ids,
            issue.target().document_object_id(),
            "presentation issue",
        )?;
    }
    for root in direct_roots {
        let id = root.document_object_id();
        let has_payload = match root.kind() {
            DocumentDirectRootKindV1::Molecule => molecules
                .iter()
                .any(|molecule| molecule.document_object_id() == id),
            DocumentDirectRootKindV1::Presentation(kind) => presentation_roots.iter().any(|root| {
                root.target().document_object_id() == id && root.target().record_kind() == kind
            }),
            DocumentDirectRootKindV1::RejectedPresentation(code) => {
                presentation_issues
                    .iter()
                    .any(|issue| issue.target().document_object_id() == id && issue.code() == code)
                    && !presentation_roots
                        .iter()
                        .any(|root| root.target().document_object_id() == id)
            }
        };
        if !has_payload {
            return Err(ProjectionError::InvalidValue {
                context: id.as_str().to_owned(),
                field: "document direct root",
                value: "direct root has no corresponding payload".to_owned(),
            });
        }
    }
    Ok(())
}

fn require_direct_root(
    direct_ids: &BTreeSet<&str>,
    id: &DocumentObjectIdV1,
    payload: &'static str,
) -> Result<(), ProjectionError> {
    if direct_ids.contains(id.as_str()) {
        return Ok(());
    }
    Err(ProjectionError::InvalidValue {
        context: id.as_str().to_owned(),
        field: "document direct root",
        value: format!("{payload} has no direct root"),
    })
}

fn projection_aggregate_error(source: DocumentProjectionV1Error) -> ProjectionError {
    ProjectionError::InvalidValue {
        context: "document snapshot".to_owned(),
        field: "projection provenance",
        value: source.to_string(),
    }
}

fn projection_issue_from_record_v1(
    code: ProjectionIssueCodeV1,
    record: &TypedRecord,
    detail: impl Into<String>,
) -> ProjectionIssueV1 {
    ProjectionIssueV1::try_new(code, record.path().to_string(), detail.into())
        .expect("typed record paths are nonempty structural locations")
}

fn molecule(
    child: &TypedChild,
    version: Option<&str>,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<MoleculeProjectionV1, ProjectionError> {
    let record = child.record();
    let endpoints = endpoint_index(record)?;
    let mut atoms = Vec::new();
    let mut compact_groups = Vec::new();
    let mut non_atom_vertices = Vec::new();
    let mut bonds = Vec::new();
    for nested in record.typed_children() {
        match nested.record().class() {
            TypedClass::Atom => {
                let atom = atom(nested, issues)?;
                atoms.push(atom);
            }
            TypedClass::CompactGroup => {
                compact_groups.push(compact_group(nested)?);
                non_atom_vertices.push(non_atom_vertex(nested, NonAtomVertexKindV1::CompactGroup)?);
            }
            TypedClass::MoleculeText => {
                non_atom_vertices.push(non_atom_vertex(nested, NonAtomVertexKindV1::MoleculeText)?)
            }
            TypedClass::Query => {
                non_atom_vertices.push(non_atom_vertex(nested, NonAtomVertexKindV1::Query)?)
            }
            TypedClass::Bond => {
                bonds.push(bond(nested, version, &endpoints, issues)?);
            }
            _ => {}
        }
    }
    MoleculeProjectionV1::try_new(
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?,
        crate::projection_local_object_key_from_record_v1(record)?,
        record.attribute("id").map(str::to_owned),
        record.attribute("name").map(str::to_owned),
        MoleculeProjectionChildrenV1 {
            atoms,
            compact_groups,
            non_atom_vertices,
            bonds,
        },
    )
    .map_err(|source| ProjectionError::InvalidValue {
        context: context(record),
        field: "molecule children",
        value: source.to_string(),
    })
}

fn non_atom_vertex(
    child: &TypedChild,
    kind: NonAtomVertexKindV1,
) -> Result<NonAtomVertexProjectionV1, ProjectionError> {
    let record = child.record();
    Ok(NonAtomVertexProjectionV1::new(
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?,
        crate::projection_local_object_key_from_record_v1(record)?,
        record.attribute("id").map(str::to_owned),
        child.position(),
        kind,
    ))
}

#[derive(Clone)]
struct EndpointTarget {
    id: DocumentObjectIdV1,
    kind: BondEndpointKindV1,
}

fn endpoint_index(
    record: &TypedRecord,
) -> Result<HashMap<String, EndpointTarget>, ProjectionError> {
    let mut endpoints = HashMap::new();
    for child in record.typed_children() {
        let target_record = child.record();
        let target = match target_record.class() {
            TypedClass::Atom => Some(EndpointTarget {
                id: crate::projection_identity_v1::projection_document_object_id_from_record_v1(
                    target_record,
                )?,
                kind: BondEndpointKindV1::Atom,
            }),
            TypedClass::CompactGroup => {
                let group = compact_group(child)?;
                target_record.attribute("id").map(|_| EndpointTarget {
                    id: group.id().clone(),
                    kind: BondEndpointKindV1::Group,
                })
            }
            TypedClass::MoleculeText => Some(EndpointTarget {
                id: crate::projection_identity_v1::projection_document_object_id_from_record_v1(
                    target_record,
                )?,
                kind: BondEndpointKindV1::MoleculeText,
            }),
            TypedClass::Query => Some(EndpointTarget {
                id: crate::projection_identity_v1::projection_document_object_id_from_record_v1(
                    target_record,
                )?,
                kind: BondEndpointKindV1::Query,
            }),
            _ => None,
        };
        if let (Some(source_id), Some(target)) = (target_record.attribute("id"), target) {
            endpoints.insert(source_id.to_owned(), target);
        }
    }
    Ok(endpoints)
}

fn atom(
    child: &TypedChild,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<AtomProjectionV1, ProjectionError> {
    let record = child.record();
    let point_record = record
        .children_of(TypedClass::Point)
        .next()
        .ok_or_else(|| ProjectionError::MissingPoint {
            context: context(record),
        })?;
    let position = point(point_record)?;
    let label_font = record
        .children_of(TypedClass::Font)
        .next()
        .map(|font| font_facts(font, issues))
        .transpose()?;
    let label_text = record
        .children_of(TypedClass::FormattedText)
        .next()
        .map(|text| RichTextV1::new(text.text_content()));
    let marks = atom_marks(record, position, issues);
    Ok(AtomProjectionV1::new(
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?,
        crate::projection_local_object_key_from_record_v1(record)?,
        record.attribute("id").map(str::to_owned),
        child.position(),
        record.attribute("name").map(str::to_owned),
        position,
        optional_scalar(record, "charge", issues),
        optional_scalar(record, "isotope", issues),
        optional_scalar(record, "explicit_hydrogens", issues),
        optional_scalar(record, "valency", issues),
        optional_scalar(record, "multiplicity", issues),
        optional_scalar(record, "free_sites", issues),
        optional_positive_integer(record, "number", issues),
        optional_visibility(record, "show_number", issues),
        marks,
        label_font,
        label_text,
        optional_visibility(record, "show", issues),
        optional_visibility(record, "hydrogens", issues),
        optional_transparent_color(record, "background-color", issues)?,
    ))
}

fn bond(
    child: &TypedChild,
    version: Option<&str>,
    endpoints: &HashMap<String, EndpointTarget>,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<BondProjectionV1, ProjectionError> {
    let record = child.record();
    let start = resolve_endpoint(record, "start", endpoints, issues)?;
    let end = resolve_endpoint(record, "end", endpoints, issues)?;
    let source_type = record.attribute("type").map(str::to_owned);
    let (order, style) = source_type
        .as_deref()
        .map(|token| bond_semantics(version, token))
        .unwrap_or((None, None));
    if source_type.is_some() && (order.is_none() || style.is_none()) {
        issues.push(projection_issue_from_record_v1(
            ProjectionIssueCodeV1::UnsupportedBondType,
            record,
            "bond type is retained but has no V1 normalized depiction",
        ));
    }
    let line_width = optional_positive(record, "line_width", issues)?;
    let bond_width = optional_signed_spacing(record, "bond_width", issues);
    let wedge_width = optional_positive(record, "wedge_width", issues)?;
    let center = optional_bool(record, "center", issues);
    let color = optional_color(record, "color", issues)?;
    let haworth_position = optional_haworth_position(record, issues);
    Ok(BondProjectionV1::new(
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?,
        crate::projection_local_object_key_from_record_v1(record)?,
        record.attribute("id").map(str::to_owned),
        child.position(),
        start,
        end,
        source_type,
        order,
        style,
        haworth_position,
        line_width,
        bond_width,
        wedge_width,
        center,
        color,
    ))
}

fn resolve_endpoint(
    record: &TypedRecord,
    field: &'static str,
    endpoints: &HashMap<String, EndpointTarget>,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<BondEndpointV1, ProjectionError> {
    let Some(value) = record.attribute(field) else {
        issues.push(projection_issue_from_record_v1(
            ProjectionIssueCodeV1::MissingBondEndpoint,
            record,
            format!("required {field} endpoint is absent"),
        ));
        return Ok(BondEndpointV1::missing());
    };
    let Some(target) = endpoints.get(value) else {
        issues.push(projection_issue_from_record_v1(
            ProjectionIssueCodeV1::UnknownBondEndpoint,
            record,
            format!("{field} points at unknown ID {value:?}"),
        ));
        return Ok(BondEndpointV1::unknown(value.to_owned()));
    };
    if !matches!(
        target.kind,
        BondEndpointKindV1::Atom | BondEndpointKindV1::Group
    ) {
        issues.push(projection_issue_from_record_v1(
            ProjectionIssueCodeV1::UnsupportedBondEndpoint,
            record,
            format!("{field} points at {}", endpoint_kind_name(target.kind)),
        ));
    }
    BondEndpointV1::resolved(value.to_owned(), target.id.clone(), target.kind).ok_or_else(|| {
        ProjectionError::InvalidValue {
            context: context(record),
            field,
            value: "resolved endpoint has an unsupported target kind".to_owned(),
        }
    })
}

fn endpoint_kind_name(kind: BondEndpointKindV1) -> &'static str {
    match kind {
        BondEndpointKindV1::Atom => "atom",
        BondEndpointKindV1::Group => "group",
        BondEndpointKindV1::MoleculeText => "molecule text",
        BondEndpointKindV1::Query => "query",
        BondEndpointKindV1::Unknown => "unknown endpoint",
        BondEndpointKindV1::Missing => "missing endpoint",
    }
}

pub(crate) fn point(record: &TypedRecord) -> Result<Point3V1, ProjectionError> {
    Point3V1::new(
        coordinate(record, "x")?,
        coordinate(record, "y")?,
        record
            .attribute("z")
            .map(|_| coordinate(record, "z"))
            .transpose()?
            .unwrap_or(0.0),
    )
}
fn coordinate(record: &TypedRecord, field: &'static str) -> Result<f64, ProjectionError> {
    let value = record
        .attribute(field)
        .ok_or_else(|| ProjectionError::InvalidValue {
            context: context(record),
            field,
            value: "<absent>".to_owned(),
        })?;
    let (raw, scale) = value
        .strip_suffix("cm")
        .map(|raw| (raw, POINTS_PER_CENTIMETRE))
        .unwrap_or((value, 1.0));
    raw.parse::<f64>()
        .map(|value| value * scale)
        .map_err(|_| ProjectionError::InvalidValue {
            context: context(record),
            field,
            value: value.to_owned(),
        })
}
fn optional_scalar<T: std::str::FromStr>(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Option<T> {
    let value = record.attribute(field)?;
    match value.parse() {
        Ok(value) => Some(value),
        Err(_) => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                format!("{field} value {value:?} is invalid"),
            ));
            None
        }
    }
}
fn font_facts(
    record: &TypedRecord,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<FontFactsV1, ProjectionError> {
    Ok(FontFactsV1::new(
        record.attribute("family").map(str::to_owned),
        optional_positive(record, "size", issues)?,
        optional_color(record, "color", issues)?,
    ))
}
fn drawing_standard(
    record: &TypedRecord,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<DrawingStandardV1, ProjectionError> {
    let atom_standard = record.children_of(TypedClass::StandardAtom).next();
    let bond_standard = record.children_of(TypedClass::StandardBond).next();
    Ok(DrawingStandardV1::new(
        optional_positive(record, "line_width", issues)?,
        bond_standard
            .map(|bond| optional_positive(bond, "width", issues))
            .transpose()?
            .flatten(),
        bond_standard
            .map(|bond| optional_positive(bond, "wedge-width", issues))
            .transpose()?
            .flatten(),
        bond_standard
            .map(|bond| optional_ratio(bond, "double-ratio", issues))
            .transpose()?
            .flatten(),
        optional_positive(record, "font_size", issues)?,
        record.attribute("font_family").map(str::to_owned),
        optional_color(record, "line_color", issues)?,
        atom_standard.and_then(|atom| optional_visibility(atom, "show_hydrogens", issues)),
        optional_transparent_color(record, "area_color", issues)?,
    ))
}

fn optional_ratio(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<Option<PositiveFiniteV1>, ProjectionError> {
    let value = optional_positive(record, field, issues)?;
    if value.is_some_and(|value| value.value() > 1.0) {
        issues.push(projection_issue_from_record_v1(
            ProjectionIssueCodeV1::InvalidPresentationFact,
            record,
            format!("{field} must be at most 1"),
        ));
        return Ok(None);
    }
    Ok(value)
}

fn optional_signed_spacing(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Option<NonZeroFiniteV1> {
    let value = record.attribute(field)?;
    let number = value
        .strip_suffix("px")
        .unwrap_or(value)
        .parse::<f64>()
        .ok();
    let spacing = number.and_then(NonZeroFiniteV1::new);
    if spacing.is_none() {
        issues.push(projection_issue_from_record_v1(
            ProjectionIssueCodeV1::InvalidPresentationFact,
            record,
            format!("{field} must be finite and non-zero"),
        ));
    }
    spacing
}
fn optional_positive(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<Option<PositiveFiniteV1>, ProjectionError> {
    let Some(value) = record.attribute(field) else {
        return Ok(None);
    };
    match PresentationLengthV1::parse(value) {
        Some(value) => Ok(Some(value.value())),
        None => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                format!("{field} must be positive and finite"),
            ));
            Ok(None)
        }
    }
}

fn optional_color(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<Option<Rgb24V1>, ProjectionError> {
    let Some(value) = record.attribute(field) else {
        return Ok(None);
    };
    match Rgb24V1::new(value) {
        Some(value) => Ok(Some(value)),
        None => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                format!("{field} must be #rgb or #rrggbb"),
            ));
            Ok(None)
        }
    }
}

fn optional_transparent_color(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<Option<TransparentOrRgb24V1>, ProjectionError> {
    let Some(value) = record.attribute(field) else {
        return Ok(None);
    };
    match TransparentOrRgb24V1::new(value) {
        Some(value) => Ok(Some(value)),
        None => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                format!("{field} must be empty, none, #rgb, or #rrggbb"),
            ));
            Ok(None)
        }
    }
}

fn optional_visibility(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Option<VisibilityV1> {
    let value = record.attribute(field)?;
    match VisibilityV1::parse(value) {
        Some(value) => Some(value),
        None => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                format!("{field} must be one of 0,false,no,off,1,true,yes,on"),
            ));
            None
        }
    }
}

fn optional_positive_integer(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Option<u64> {
    let value = record.attribute(field)?;
    match value.parse::<u64>().ok().filter(|value| *value > 0) {
        Some(number) if number.to_string() == value => Some(number),
        _ => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                format!("{field} must be a canonical positive decimal integer"),
            ));
            None
        }
    }
}

fn optional_bool(
    record: &TypedRecord,
    field: &'static str,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Option<bool> {
    match optional_visibility(record, field, issues) {
        Some(VisibilityV1::Enabled) => Some(true),
        Some(VisibilityV1::Disabled) => Some(false),
        None => None,
    }
}

fn optional_haworth_position(
    record: &TypedRecord,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Option<DocumentHaworthPositionV1> {
    match record.attribute("haworth_position") {
        Some("front") => Some(DocumentHaworthPositionV1::Front),
        Some("back") => Some(DocumentHaworthPositionV1::Back),
        Some(_) => {
            issues.push(projection_issue_from_record_v1(
                ProjectionIssueCodeV1::InvalidPresentationFact,
                record,
                "haworth_position must be front or back",
            ));
            None
        }
        None => None,
    }
}
fn bond_semantics(version: Option<&str>, token: &str) -> (Option<BondOrder>, Option<BondStyle>) {
    if version == Some("0.8") {
        match token {
            "s" => return (Some(BondOrder::Single), Some(BondStyle::Normal)),
            "d" => return (Some(BondOrder::Double), Some(BondStyle::Normal)),
            _ => {}
        }
    }
    crate::project_source_bond_semantics(token)
}
fn context(record: &TypedRecord) -> String {
    format!("{} at {}", record.class().name(), record.path())
}

#[cfg(test)]
mod bond_semantics_tests {
    use ferrum_core::{BondOrder, BondStyle};

    use super::bond_semantics;

    #[test]
    fn projection_preserves_current_source_semantics_beyond_authoring_vocabulary() {
        assert_eq!(
            bond_semantics(Some("26.08"), "b1"),
            (Some(BondOrder::Single), Some(BondStyle::Bold))
        );
        assert_eq!(
            bond_semantics(Some("26.08"), "d1"),
            (Some(BondOrder::Single), Some(BondStyle::Dashed))
        );
        assert_eq!(
            bond_semantics(Some("26.08"), "b2"),
            (Some(BondOrder::Double), Some(BondStyle::Bold))
        );
        assert_eq!(
            bond_semantics(Some("26.08"), "n0"),
            (Some(BondOrder::Other(0)), Some(BondStyle::Normal))
        );
    }
}
