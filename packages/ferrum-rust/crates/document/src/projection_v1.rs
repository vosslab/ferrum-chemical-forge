//! Versioned, immutable document projection DTOs built from typed CDML facts.

use std::collections::HashMap;

use ferrum_core::{BondOrder, BondStyle};
use serde::Serialize;
use thiserror::Error;

use crate::atom_mark_projection::atom_marks;
use crate::atom_projection_v1::AtomProjectionV1;
use crate::projection_identity_v1::{DocumentObjectIdV1, ProjectionLocalObjectKeyV1};

use super::{
    DocumentSnapshot, DrawingStandardV1, FontFactsV1, NonZeroFiniteV1, PaperLayoutProjectionV1,
    PositiveFiniteV1, PresentationLengthV1, PresentationStackProjectionV1, Rgb24V1, RichTextV1,
    TransparentOrRgb24V1, TypedChild, TypedClass, TypedDocument, TypedRecord, VisibilityV1,
};

const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

/// Stable schema identifier for [`DocumentProjectionV1`].
pub const DOCUMENT_PROJECTION_SCHEMA_V1: &str = "ferrum-document-projection-v1";

/// Validated finite Cartesian coordinates carried by an atom projection.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Point3V1 {
    x: f64,
    y: f64,
    z: f64,
}

impl Point3V1 {
    /// Construct a finite coordinate.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, ProjectionError> {
        for (axis, value) in [("x", x), ("y", y), ("z", z)] {
            if !value.is_finite() {
                return Err(ProjectionError::NonFiniteCoordinate { axis });
            }
        }
        Ok(Self { x, y, z })
    }
    /// Return the x coordinate.
    #[must_use]
    pub fn x(self) -> f64 {
        self.x
    }
    /// Return the y coordinate.
    #[must_use]
    pub fn y(self) -> f64 {
        self.y
    }
    /// Return the z coordinate.
    #[must_use]
    pub fn z(self) -> f64 {
        self.z
    }
}

/// One recognized but non-renderable typed fact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectionIssueV1 {
    code: ProjectionIssueCodeV1,
    path: String,
    detail: String,
}

impl ProjectionIssueV1 {
    pub(crate) fn new(
        code: ProjectionIssueCodeV1,
        record: &TypedRecord,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            code,
            path: record.path().to_string(),
            detail: detail.into(),
        }
    }
    /// Return the stable issue category.
    #[must_use]
    pub fn code(&self) -> ProjectionIssueCodeV1 {
        self.code
    }
    /// Return the typed record path that supplied the issue.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    /// Return actionable source detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Stable categories for facts not represented by the V1 molecule projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionIssueCodeV1 {
    MissingBondEndpoint,
    UnsupportedBondEndpoint,
    UnknownBondEndpoint,
    UnsupportedBondType,
    InvalidPresentationFact,
}

/// The typed target category carried by one retained bond endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BondEndpointKindV1 {
    /// A resolved atom can participate in V1 molecule rendering.
    Atom,
    /// A resolved group is retained but not yet renderable as an atom endpoint.
    Group,
    /// A resolved molecule text object is retained but not a renderable endpoint.
    MoleculeText,
    /// A resolved query object is retained but not a renderable endpoint.
    Query,
    /// The source endpoint names no known retained object.
    Unknown,
    /// The source omitted this required endpoint attribute.
    Missing,
}

/// Authored Haworth depth carried by a retained bond presentation fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentHaworthPositionV1 {
    /// The bond is authored in front of the Haworth plane.
    Front,
    /// The bond is authored behind the Haworth plane.
    Back,
}

/// One literal bond endpoint reference and its resolved durable object key, if any.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct BondEndpointV1 {
    source_id: Option<String>,
    object_id: Option<DocumentObjectIdV1>,
    kind: BondEndpointKindV1,
}

impl BondEndpointV1 {
    /// Return the literal authored endpoint token, when supplied.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }

    /// Return the resolved durable object key, when the target was retained.
    #[must_use]
    pub fn object_id(&self) -> Option<&DocumentObjectIdV1> {
        self.object_id.as_ref()
    }

    /// Return the endpoint target category without synthesizing a fallback.
    #[must_use]
    pub fn kind(&self) -> BondEndpointKindV1 {
        self.kind
    }
}

/// Immutable bond facts in source order.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BondProjectionV1 {
    id: Option<DocumentObjectIdV1>,
    projection_key: ProjectionLocalObjectKeyV1,
    source_id: Option<String>,
    source_order: u32,
    start: BondEndpointV1,
    end: BondEndpointV1,
    source_type: Option<String>,
    order: Option<BondOrder>,
    style: Option<BondStyle>,
    haworth_position: Option<DocumentHaworthPositionV1>,
    line_width: Option<PositiveFiniteV1>,
    bond_width: Option<NonZeroFiniteV1>,
    wedge_width: Option<PositiveFiniteV1>,
    center: Option<bool>,
    color: Option<Rgb24V1>,
}

impl BondProjectionV1 {
    /// Return the stable object key.
    #[must_use]
    pub fn id(&self) -> Option<&DocumentObjectIdV1> {
        self.id.as_ref()
    }
    /// Return the non-operation key unique within this projection.
    #[must_use]
    pub fn projection_key(&self) -> &ProjectionLocalObjectKeyV1 {
        &self.projection_key
    }
    /// Return the literal CDML ID when authored.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    /// Return the child position in its molecule.
    #[must_use]
    pub fn source_order(&self) -> u32 {
        self.source_order
    }
    /// Return the first retained endpoint fact.
    #[must_use]
    pub fn start(&self) -> &BondEndpointV1 {
        &self.start
    }
    /// Return the second retained endpoint fact.
    #[must_use]
    pub fn end(&self) -> &BondEndpointV1 {
        &self.end
    }
    /// Return the authored bond type token.
    #[must_use]
    pub fn source_type(&self) -> Option<&str> {
        self.source_type.as_deref()
    }
    /// Return the normalized order when the token is understood.
    #[must_use]
    pub fn order(&self) -> Option<BondOrder> {
        self.order
    }
    /// Return the normalized drawing style when the token is understood.
    #[must_use]
    pub fn style(&self) -> Option<&BondStyle> {
        self.style.as_ref()
    }
    /// Return authored Haworth depth without inferring a bond style or depiction.
    #[must_use]
    pub fn haworth_position(&self) -> Option<DocumentHaworthPositionV1> {
        self.haworth_position
    }
    /// Return authored positive line width.
    #[must_use]
    pub fn line_width(&self) -> Option<PositiveFiniteV1> {
        self.line_width
    }
    /// Return the authored signed parallel-lane spacing.
    #[must_use]
    pub fn bond_width(&self) -> Option<NonZeroFiniteV1> {
        self.bond_width
    }
    /// Return authored positive wedge width.
    #[must_use]
    pub fn wedge_width(&self) -> Option<PositiveFiniteV1> {
        self.wedge_width
    }
    /// Return authored centered-double-bond intent without choosing a default.
    #[must_use]
    pub fn center(&self) -> Option<bool> {
        self.center
    }
    /// Return authored line colour.
    #[must_use]
    pub fn color(&self) -> Option<&Rgb24V1> {
        self.color.as_ref()
    }
}

/// One molecule and its source-ordered renderable children.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MoleculeProjectionV1 {
    id: Option<DocumentObjectIdV1>,
    projection_key: ProjectionLocalObjectKeyV1,
    source_id: Option<String>,
    source_order: u32,
    name: Option<String>,
    atoms: Vec<AtomProjectionV1>,
    bonds: Vec<BondProjectionV1>,
}

impl MoleculeProjectionV1 {
    /// Return the stable object key.
    #[must_use]
    pub fn id(&self) -> Option<&DocumentObjectIdV1> {
        self.id.as_ref()
    }
    /// Return the non-operation key unique within this projection.
    #[must_use]
    pub fn projection_key(&self) -> &ProjectionLocalObjectKeyV1 {
        &self.projection_key
    }
    /// Return literal CDML ID when authored.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    /// Return the root-child position.
    #[must_use]
    pub fn source_order(&self) -> u32 {
        self.source_order
    }
    /// Return the authored name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    /// Return atoms in nested source order.
    #[must_use]
    pub fn atoms(&self) -> &[AtomProjectionV1] {
        &self.atoms
    }
    /// Return bonds in nested source order.
    #[must_use]
    pub fn bonds(&self) -> &[BondProjectionV1] {
        &self.bonds
    }
}

/// Immutable V1 projection from one authoritative typed CDML document.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DocumentProjectionV1 {
    schema: &'static str,
    revision: u64,
    digest: [u8; 32],
    is_dirty: bool,
    drawing_standard: Option<DrawingStandardV1>,
    paper_layout: PaperLayoutProjectionV1,
    molecules: Vec<MoleculeProjectionV1>,
    presentation_stack: PresentationStackProjectionV1,
    issues: Vec<ProjectionIssueV1>,
}

impl DocumentProjectionV1 {
    /// Build a projection from the one immutable snapshot that owns its provenance.
    pub(crate) fn from_snapshot(
        document: &TypedDocument,
        snapshot: &DocumentSnapshot,
    ) -> Result<Self, ProjectionError> {
        let mut issues = Vec::new();
        let mut persisted_standard = None;
        let mut molecules = Vec::new();
        let version = document.root().attribute("version");
        for child in document.root().typed_children() {
            match child.record().class() {
                TypedClass::Standard if persisted_standard.is_none() => {
                    persisted_standard = Some(drawing_standard(child.record(), &mut issues)?);
                }
                TypedClass::Molecule => molecules.push(molecule(child, version, &mut issues)?),
                _ => {}
            }
        }
        Ok(Self {
            schema: DOCUMENT_PROJECTION_SCHEMA_V1,
            revision: snapshot.revision(),
            digest: *snapshot.digest(),
            is_dirty: snapshot.is_dirty(),
            drawing_standard: persisted_standard,
            paper_layout: PaperLayoutProjectionV1::from_snapshot(document, snapshot),
            molecules,
            presentation_stack: PresentationStackProjectionV1::from_snapshot(document, snapshot),
            issues,
        })
    }
    /// Return the closed wire schema identifier.
    #[must_use]
    pub fn schema(&self) -> &'static str {
        self.schema
    }
    /// Return the snapshot revision.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Return the snapshot digest.
    #[must_use]
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
    /// Return whether the source snapshot differed from its saved baseline.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }
    /// Return persisted drawing-standard facts, when authored.
    #[must_use]
    pub fn drawing_standard(&self) -> Option<&DrawingStandardV1> {
        self.drawing_standard.as_ref()
    }
    /// Return paper and viewport facts from the same authoritative snapshot.
    #[must_use]
    pub fn paper_layout(&self) -> &PaperLayoutProjectionV1 {
        &self.paper_layout
    }
    /// Return molecules in root source order.
    #[must_use]
    pub fn molecules(&self) -> &[MoleculeProjectionV1] {
        &self.molecules
    }
    /// Return supported direct-root presentation targets from the same snapshot.
    #[must_use]
    pub fn presentation_stack(&self) -> &PresentationStackProjectionV1 {
        &self.presentation_stack
    }
    /// Return issues in stable encounter order.
    #[must_use]
    pub fn issues(&self) -> &[ProjectionIssueV1] {
        &self.issues
    }
}

/// Projection construction rejected a required or invalid typed fact.
#[derive(Debug, Error)]
pub enum ProjectionError {
    /// A renderable atom lacked its required point child.
    #[error("{context}: required point is absent")]
    MissingPoint { context: String },
    /// A source scalar could not be parsed for the named projection fact.
    #[error("{context}: {field} value {value:?} is invalid")]
    InvalidValue {
        context: String,
        field: &'static str,
        value: String,
    },
    /// A coordinate was not finite after CDML unit conversion.
    #[error("coordinate {axis} is not finite")]
    NonFiniteCoordinate { axis: &'static str },
}

fn molecule(
    child: &TypedChild,
    version: Option<&str>,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<MoleculeProjectionV1, ProjectionError> {
    let record = child.record();
    let endpoints = endpoint_index(record);
    let mut atoms = Vec::new();
    let mut bonds = Vec::new();
    for nested in record.typed_children() {
        match nested.record().class() {
            TypedClass::Atom => {
                let atom = atom(nested, issues)?;
                atoms.push(atom);
            }
            TypedClass::Bond => {
                bonds.push(bond(nested, version, &endpoints, issues)?);
            }
            _ => {}
        }
    }
    Ok(MoleculeProjectionV1 {
        id: DocumentObjectIdV1::from_record(record),
        projection_key: ProjectionLocalObjectKeyV1::from_record(record),
        source_id: record.attribute("id").map(str::to_owned),
        source_order: child.position(),
        name: record.attribute("name").map(str::to_owned),
        atoms,
        bonds,
    })
}

#[derive(Clone)]
struct EndpointTarget {
    id: DocumentObjectIdV1,
    kind: BondEndpointKindV1,
}

fn endpoint_index(record: &TypedRecord) -> HashMap<String, EndpointTarget> {
    record
        .typed_children()
        .iter()
        .filter_map(|child| {
            let record = child.record();
            let kind = match record.class() {
                TypedClass::Atom => BondEndpointKindV1::Atom,
                TypedClass::Group => BondEndpointKindV1::Group,
                TypedClass::MoleculeText => BondEndpointKindV1::MoleculeText,
                TypedClass::Query => BondEndpointKindV1::Query,
                _ => return None,
            };
            record.attribute("id").map(|source_id| {
                (
                    source_id.to_owned(),
                    EndpointTarget {
                        id: DocumentObjectIdV1::from_record(record).expect(
                            "records indexed by persistent source ID have durable selectors",
                        ),
                        kind,
                    },
                )
            })
        })
        .collect()
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
    Ok(AtomProjectionV1 {
        id: DocumentObjectIdV1::from_record(record),
        projection_key: ProjectionLocalObjectKeyV1::from_record(record),
        source_id: record.attribute("id").map(str::to_owned),
        source_order: child.position(),
        element: record.attribute("name").map(str::to_owned),
        position,
        formal_charge: optional_scalar(record, "charge", issues),
        isotope: optional_scalar(record, "isotope", issues),
        explicit_hydrogens: optional_scalar(record, "explicit_hydrogens", issues),
        valence: optional_scalar(record, "valency", issues),
        multiplicity: optional_scalar(record, "multiplicity", issues),
        free_sites: optional_scalar(record, "free_sites", issues),
        number: optional_positive_integer(record, "number", issues),
        show_number: optional_visibility(record, "show_number", issues),
        marks,
        label_font,
        label_text,
        show: optional_visibility(record, "show", issues),
        hydrogens: optional_visibility(record, "hydrogens", issues),
        background_color: optional_transparent_color(record, "background-color", issues)?,
    })
}

fn bond(
    child: &TypedChild,
    version: Option<&str>,
    endpoints: &HashMap<String, EndpointTarget>,
    issues: &mut Vec<ProjectionIssueV1>,
) -> Result<BondProjectionV1, ProjectionError> {
    let record = child.record();
    let start = resolve_endpoint(record, "start", endpoints, issues);
    let end = resolve_endpoint(record, "end", endpoints, issues);
    let source_type = record.attribute("type").map(str::to_owned);
    let (order, style) = source_type
        .as_deref()
        .map(|token| bond_semantics(version, token))
        .unwrap_or((None, None));
    if source_type.is_some() && (order.is_none() || style.is_none()) {
        issues.push(ProjectionIssueV1::new(
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
    Ok(BondProjectionV1 {
        id: DocumentObjectIdV1::from_record(record),
        projection_key: ProjectionLocalObjectKeyV1::from_record(record),
        source_id: record.attribute("id").map(str::to_owned),
        source_order: child.position(),
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
    })
}

fn resolve_endpoint(
    record: &TypedRecord,
    field: &'static str,
    endpoints: &HashMap<String, EndpointTarget>,
    issues: &mut Vec<ProjectionIssueV1>,
) -> BondEndpointV1 {
    let Some(value) = record.attribute(field) else {
        issues.push(ProjectionIssueV1::new(
            ProjectionIssueCodeV1::MissingBondEndpoint,
            record,
            format!("required {field} endpoint is absent"),
        ));
        return BondEndpointV1 {
            source_id: None,
            object_id: None,
            kind: BondEndpointKindV1::Missing,
        };
    };
    let Some(target) = endpoints.get(value) else {
        issues.push(ProjectionIssueV1::new(
            ProjectionIssueCodeV1::UnknownBondEndpoint,
            record,
            format!("{field} points at unknown ID {value:?}"),
        ));
        return BondEndpointV1 {
            source_id: Some(value.to_owned()),
            object_id: None,
            kind: BondEndpointKindV1::Unknown,
        };
    };
    if target.kind != BondEndpointKindV1::Atom {
        issues.push(ProjectionIssueV1::new(
            ProjectionIssueCodeV1::UnsupportedBondEndpoint,
            record,
            format!("{field} points at {}", endpoint_kind_name(target.kind)),
        ));
    }
    BondEndpointV1 {
        source_id: Some(value.to_owned()),
        object_id: Some(target.id.clone()),
        kind: target.kind,
    }
}

fn endpoint_kind_name(kind: BondEndpointKindV1) -> &'static str {
    match kind {
        BondEndpointKindV1::Atom => "atom",
        BondEndpointKindV1::Group => "group",
        BondEndpointKindV1::MoleculeText => "molecule text",
        BondEndpointKindV1::Query => "query",
        BondEndpointKindV1::Unknown | BondEndpointKindV1::Missing => unreachable!(),
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
            issues.push(ProjectionIssueV1::new(
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
        issues.push(ProjectionIssueV1::new(
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
        issues.push(ProjectionIssueV1::new(
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
            issues.push(ProjectionIssueV1::new(
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
            issues.push(ProjectionIssueV1::new(
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
            issues.push(ProjectionIssueV1::new(
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
            issues.push(ProjectionIssueV1::new(
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
            issues.push(ProjectionIssueV1::new(
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
            issues.push(ProjectionIssueV1::new(
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
    let Some(digits) = token.get(1..) else {
        return (None, None);
    };
    let order = if digits.is_empty() {
        None
    } else {
        let Ok(number) = digits.parse() else {
            return (None, None);
        };
        Some(match number {
            1 => BondOrder::Single,
            2 => BondOrder::Double,
            3 => BondOrder::Triple,
            4 => BondOrder::Aromatic,
            other => BondOrder::Other(other),
        })
    };
    let style = token.chars().next().map(|character| match character {
        'n' => BondStyle::Normal,
        'w' => BondStyle::Wedge,
        'h' | 'l' | 'r' => BondStyle::Hashed,
        'a' => BondStyle::Adder,
        'b' => BondStyle::Bold,
        'd' => BondStyle::Dashed,
        'o' => BondStyle::Dotted,
        's' => BondStyle::Wavy,
        'q' => BondStyle::HaworthFront,
        other => BondStyle::Other(other.to_string()),
    });
    (order, style)
}
fn context(record: &TypedRecord) -> String {
    format!("{} at {}", record.class().name(), record.path())
}
