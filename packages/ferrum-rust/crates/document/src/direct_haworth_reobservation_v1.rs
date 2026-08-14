//! Strict durable re-observation of one direct-glycosidic Haworth molecule.

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_domain::haworth::{
    AuthoredDirectGlycosidicHaworthBondRoleV1, AuthoredDirectGlycosidicHaworthDepictionV1,
    DirectGlycosidicHaworthAuthoringAtomElementV1, DirectGlycosidicHaworthBondStyleV1,
    DirectGlycosidicHaworthPositionV1, DurableDirectGlycosidicHaworthAtomFactV1,
    DurableDirectGlycosidicHaworthBondFactV1, DurableDirectGlycosidicHaworthProfileV1,
    DurableDirectGlycosidicHaworthRingFactV1, HaworthError, HaworthPoint, RingForm,
    authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1,
};
use thiserror::Error;

use crate::{
    DocumentDirectHaworthBondRoleV1, DocumentDirectHaworthBondTokenV1, DocumentHaworthPositionV1,
    DocumentObjectIdV1, PersistentId, Point3V1, SessionDocumentObservationV1, TypedClass,
    TypedDocument, TypedRecord, typed_coordinate::parse_coordinate,
};

/// Read-only re-observation errors for the closed durable Haworth profile.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum DirectHaworthReobservationErrorV1 {
    /// The selector did not name a current molecule record with a durable ID.
    #[error("direct Haworth selector does not name a current durable molecule")]
    Selector,
    /// The selected molecule has facts outside the exact durable V1 profile.
    #[error("selected molecule does not match the closed direct Haworth profile")]
    SelectedProfile,
    /// Projection reported an issue at or below the selected molecule.
    #[error("selected molecule has a projection issue")]
    SelectedProjectionIssue,
    /// A small selected-vector allocation could not be reserved.
    #[error("direct Haworth re-observation could not reserve selected facts")]
    Allocation,
}

/// Owned current-state receipt for one re-authenticated direct Haworth profile.
///
/// The embedded observation retains its ordinary snapshot CDML transitively; this
/// type adds neither a CDML getter nor a session reference.
#[derive(Clone, Debug, PartialEq)]
pub struct ReobservedDirectHaworthV1 {
    observation: SessionDocumentObservationV1,
    molecule: DocumentObjectIdV1,
    root_order: u32,
    atom_identifiers: Vec<PersistentId>,
    bond_facts: Vec<ReobservedDirectHaworthBondFactV1>,
    authored_depiction: AuthoredDirectGlycosidicHaworthDepictionV1,
}

impl ReobservedDirectHaworthV1 {
    #[must_use]
    pub fn observation(&self) -> &SessionDocumentObservationV1 {
        &self.observation
    }
    #[must_use]
    pub fn molecule(&self) -> &DocumentObjectIdV1 {
        &self.molecule
    }
    #[must_use]
    pub const fn root_order(&self) -> u32 {
        self.root_order
    }
    #[must_use]
    pub fn atom_identifiers(&self) -> &[PersistentId] {
        &self.atom_identifiers
    }
    #[must_use]
    pub fn bond_facts(&self) -> &[ReobservedDirectHaworthBondFactV1] {
        &self.bond_facts
    }
    #[must_use]
    pub fn authored_depiction(&self) -> &AuthoredDirectGlycosidicHaworthDepictionV1 {
        &self.authored_depiction
    }
}

/// One canonical durable bond fact recovered from the selected current source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReobservedDirectHaworthBondFactV1 {
    bond_identifier: PersistentId,
    endpoints: [PersistentId; 2],
    token: DocumentDirectHaworthBondTokenV1,
    haworth_position: Option<DocumentHaworthPositionV1>,
    role: DocumentDirectHaworthBondRoleV1,
}

impl ReobservedDirectHaworthBondFactV1 {
    #[must_use]
    pub fn bond_identifier(&self) -> &PersistentId {
        &self.bond_identifier
    }
    #[must_use]
    pub fn endpoints(&self) -> &[PersistentId; 2] {
        &self.endpoints
    }
    #[must_use]
    pub const fn token(&self) -> DocumentDirectHaworthBondTokenV1 {
        self.token
    }
    #[must_use]
    pub const fn haworth_position(&self) -> Option<DocumentHaworthPositionV1> {
        self.haworth_position
    }
    #[must_use]
    pub const fn role(&self) -> DocumentDirectHaworthBondRoleV1 {
        self.role
    }
}

pub(crate) struct ExtractedDirectHaworthV1 {
    molecule: DocumentObjectIdV1,
    molecule_source: PersistentId,
    molecule_path: String,
    atom_identifiers: Vec<PersistentId>,
    bond_facts: Vec<ReobservedDirectHaworthBondFactV1>,
    authored_depiction: AuthoredDirectGlycosidicHaworthDepictionV1,
}

pub(crate) fn extract(
    document: &TypedDocument,
    selector: &DocumentObjectIdV1,
) -> Result<ExtractedDirectHaworthV1, DirectHaworthReobservationErrorV1> {
    let molecule = document
        .resolve_document_object_id(selector)
        .filter(|record| record.class() == TypedClass::Molecule)
        .ok_or(DirectHaworthReobservationErrorV1::Selector)?;
    exact_attributes(molecule, &["id"])?;
    exact_content(molecule)?;
    let molecule_source = persistent_id(molecule)?;
    let children = molecule.typed_children();
    let first_bond = children
        .iter()
        .position(|child| child.record().class() == TypedClass::Bond)
        .unwrap_or(children.len());
    if children[..first_bond]
        .iter()
        .any(|child| child.record().class() != TypedClass::Atom)
        || children[first_bond..]
            .iter()
            .any(|child| child.record().class() != TypedClass::Bond)
        || children
            .iter()
            .enumerate()
            .any(|(index, child)| child.position() != index as u32)
    {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    let mut atoms = Vec::new();
    atoms
        .try_reserve_exact(first_bond)
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    for (index, child) in children[..first_bond].iter().enumerate() {
        atoms.push(atom(child.record(), index as u32)?);
    }
    let mut bonds = Vec::new();
    bonds
        .try_reserve_exact(children.len() - first_bond)
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    for (index, child) in children[first_bond..].iter().enumerate() {
        bonds.push(bond(child.record(), (first_bond + index) as u32)?);
    }
    let profile = profile_for_counts(&atoms, &bonds)?;
    let authored_depiction =
        authored_direct_glycosidic_haworth_depiction_from_durable_profile_v1(profile)
            .map_err(map_domain_error)?;
    let mut atom_identifiers = Vec::new();
    atom_identifiers
        .try_reserve_exact(atoms.len())
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    for atom in &atoms {
        atom_identifiers.push(atom.identifier.clone());
    }
    let mut bond_facts = Vec::new();
    bond_facts
        .try_reserve_exact(bonds.len())
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    for bond in bonds {
        bond_facts.push(bond.receipt());
    }
    Ok(ExtractedDirectHaworthV1 {
        molecule: selector.clone(),
        molecule_source,
        molecule_path: molecule.path().to_string(),
        atom_identifiers,
        bond_facts,
        authored_depiction,
    })
}

const fn map_domain_error(error: HaworthError) -> DirectHaworthReobservationErrorV1 {
    match error {
        HaworthError::ResourceExhausted => DirectHaworthReobservationErrorV1::Allocation,
        HaworthError::InvalidSpec(_)
        | HaworthError::UnsupportedTopology(_)
        | HaworthError::StaleTopology(_)
        | HaworthError::Unplaceable(_) => DirectHaworthReobservationErrorV1::SelectedProfile,
    }
}

pub(crate) fn finish(
    extracted: ExtractedDirectHaworthV1,
    observation: SessionDocumentObservationV1,
) -> Result<ReobservedDirectHaworthV1, DirectHaworthReobservationErrorV1> {
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|candidate| candidate.id() == Some(&extracted.molecule))
        .ok_or(DirectHaworthReobservationErrorV1::Selector)?;
    if molecule.source_id() != Some(extracted.molecule_source.as_str())
        || molecule.atoms().len() != extracted.atom_identifiers.len()
        || molecule.bonds().len() != extracted.bond_facts.len()
    {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    if observation.projection().issues().iter().any(|issue| {
        issue.path() == extracted.molecule_path
            || issue
                .path()
                .strip_prefix(&extracted.molecule_path)
                .is_some_and(|tail| tail.starts_with('/'))
    }) {
        return Err(DirectHaworthReobservationErrorV1::SelectedProjectionIssue);
    }
    for (index, (projected, expected)) in molecule
        .atoms()
        .iter()
        .zip(&extracted.atom_identifiers)
        .enumerate()
    {
        if projected.source_order() != index as u32
            || projected.source_id() != Some(expected.as_str())
            || projected.id().is_none()
        {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
    }
    for (index, (projected, expected)) in molecule
        .bonds()
        .iter()
        .zip(&extracted.bond_facts)
        .enumerate()
    {
        if projected.source_order() != (extracted.atom_identifiers.len() + index) as u32
            || projected.source_id() != Some(expected.bond_identifier().as_str())
            || projected.start().source_id() != Some(expected.endpoints()[0].as_str())
            || projected.end().source_id() != Some(expected.endpoints()[1].as_str())
        {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
    }
    let root_order = molecule.source_order();
    Ok(ReobservedDirectHaworthV1 {
        observation,
        molecule: extracted.molecule,
        root_order,
        atom_identifiers: extracted.atom_identifiers,
        bond_facts: extracted.bond_facts,
        authored_depiction: extracted.authored_depiction,
    })
}

#[derive(Clone)]
struct AtomFact {
    identifier: PersistentId,
    record: RecordId,
    element: DirectGlycosidicHaworthAuthoringAtomElementV1,
    point: Point3V1,
    child_order: u32,
}

struct BondFact {
    identifier: PersistentId,
    record: RecordId,
    endpoints: [PersistentId; 2],
    token: DocumentDirectHaworthBondTokenV1,
    position: Option<DocumentHaworthPositionV1>,
    child_order: u32,
}

impl BondFact {
    fn receipt(self) -> ReobservedDirectHaworthBondFactV1 {
        let role = if self.position.is_some() {
            DocumentDirectHaworthBondRoleV1::Ring
        } else {
            DocumentDirectHaworthBondRoleV1::Bridge
        };
        ReobservedDirectHaworthBondFactV1 {
            bond_identifier: self.identifier,
            endpoints: self.endpoints,
            token: self.token,
            haworth_position: self.position,
            role,
        }
    }
}

fn exact_attributes(
    record: &TypedRecord,
    names: &[&str],
) -> Result<(), DirectHaworthReobservationErrorV1> {
    if !record.unknown_attributes().is_empty()
        || record.typed_attributes().len() != names.len()
        || names.iter().any(|name| record.attribute(name).is_none())
    {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    Ok(())
}

fn exact_content(record: &TypedRecord) -> Result<(), DirectHaworthReobservationErrorV1> {
    if !record.typed_text().is_empty()
        || !record.unrecognized_children().is_empty()
        || !record.diagnostics().is_empty()
    {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    Ok(())
}

fn persistent_id(record: &TypedRecord) -> Result<PersistentId, DirectHaworthReobservationErrorV1> {
    PersistentId::new(
        record
            .attribute("id")
            .ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)?,
    )
    .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)
}

fn record_id(
    record: &TypedRecord,
    kind: RecordKind,
) -> Result<RecordId, DirectHaworthReobservationErrorV1> {
    let source = Identifier::new(persistent_id(record)?.as_str().to_owned())
        .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)?;
    Ok(RecordId::from_source(kind, &source))
}

fn atom(
    record: &TypedRecord,
    child_order: u32,
) -> Result<AtomFact, DirectHaworthReobservationErrorV1> {
    exact_attributes(record, &["id", "name"])?;
    exact_content(record)?;
    let [point] = record.typed_children() else {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    };
    if point.position() != 0 || point.record().class() != TypedClass::Point {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    let point_record = point.record();
    exact_attributes(point_record, &["x", "y", "z"])?;
    exact_content(point_record)?;
    if !point_record.typed_children().is_empty() {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    let point = Point3V1::new(
        parse_coordinate(point_record.attribute("x").unwrap_or_default())
            .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)?,
        parse_coordinate(point_record.attribute("y").unwrap_or_default())
            .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)?,
        parse_coordinate(point_record.attribute("z").unwrap_or_default())
            .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)?,
    )
    .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)?;
    let element = match record.attribute("name") {
        Some("C") => DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon,
        Some("O") => DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen,
        _ => return Err(DirectHaworthReobservationErrorV1::SelectedProfile),
    };
    Ok(AtomFact {
        identifier: persistent_id(record)?,
        record: record_id(record, RecordKind::Atom)?,
        element,
        point,
        child_order,
    })
}

fn bond(
    record: &TypedRecord,
    child_order: u32,
) -> Result<BondFact, DirectHaworthReobservationErrorV1> {
    exact_content(record)?;
    if !record.typed_children().is_empty() {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    let has_position = record.attribute("haworth_position").is_some();
    let required = if has_position {
        &["id", "type", "start", "end", "haworth_position"][..]
    } else {
        &["id", "type", "start", "end"][..]
    };
    exact_attributes(record, required)?;
    let token = match record.attribute("type") {
        Some("q1") => DocumentDirectHaworthBondTokenV1::Q1,
        Some("w1") => DocumentDirectHaworthBondTokenV1::W1,
        Some("n1") => DocumentDirectHaworthBondTokenV1::N1,
        _ => return Err(DirectHaworthReobservationErrorV1::SelectedProfile),
    };
    let position = match record.attribute("haworth_position") {
        Some("front") => Some(DocumentHaworthPositionV1::Front),
        Some("back") => Some(DocumentHaworthPositionV1::Back),
        None => None,
        Some(_) => return Err(DirectHaworthReobservationErrorV1::SelectedProfile),
    };
    let endpoints = ["start", "end"].map(|name| {
        PersistentId::new(
            record
                .attribute(name)
                .ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)?,
        )
        .map_err(|_| DirectHaworthReobservationErrorV1::SelectedProfile)
    });
    Ok(BondFact {
        identifier: persistent_id(record)?,
        record: record_id(record, RecordKind::Bond)?,
        endpoints: [endpoints[0].clone()?, endpoints[1].clone()?],
        token,
        position,
        child_order,
    })
}

#[derive(Clone, Copy)]
struct Shape {
    first: usize,
    second: usize,
}

fn profile_for_counts(
    atoms: &[AtomFact],
    bonds: &[BondFact],
) -> Result<DurableDirectGlycosidicHaworthProfileV1, DirectHaworthReobservationErrorV1> {
    let mut accepted = None;
    for (first, second) in [(5, 5), (5, 6), (6, 5), (6, 6)] {
        let shape = Shape { first, second };
        if atoms.len() != first + second + 1 || bonds.len() != first + second + 2 {
            continue;
        }
        if let Ok(profile) = profile_for_shape(atoms, bonds, shape)
            && accepted.replace(profile).is_some()
        {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
    }
    accepted.ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)
}

fn profile_for_shape(
    atoms: &[AtomFact],
    bonds: &[BondFact],
    shape: Shape,
) -> Result<DurableDirectGlycosidicHaworthProfileV1, DirectHaworthReobservationErrorV1> {
    let bridge = atoms
        .last()
        .ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)?;
    if bridge.element != DirectGlycosidicHaworthAuthoringAtomElementV1::Oxygen {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    let mut atom_facts = Vec::new();
    atom_facts
        .try_reserve_exact(atoms.len())
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    for atom in atoms {
        atom_facts.push(DurableDirectGlycosidicHaworthAtomFactV1::new(
            atom.record.clone(),
            atom.element,
            HaworthPoint {
                x: atom.point.x(),
                y: atom.point.y(),
            },
            atom.child_order,
        ));
    }
    let first_atoms = &atoms[..shape.first];
    let second_atoms = &atoms[shape.first..shape.first + shape.second];
    let first_bonds = &bonds[..shape.first];
    let second_bonds = &bonds[shape.first..shape.first + shape.second];
    positional_cycle(first_atoms, first_bonds)?;
    positional_cycle(second_atoms, second_bonds)?;
    bridge_bonds(
        &bonds[shape.first + shape.second..],
        first_atoms,
        second_atoms,
        bridge,
    )?;
    let mut bond_facts = Vec::new();
    bond_facts
        .try_reserve_exact(bonds.len())
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    for bond in bonds {
        let role = if bond.position.is_some() {
            AuthoredDirectGlycosidicHaworthBondRoleV1::Ring
        } else {
            AuthoredDirectGlycosidicHaworthBondRoleV1::Bridge
        };
        let token = match bond.token {
            DocumentDirectHaworthBondTokenV1::Q1 => DirectGlycosidicHaworthBondStyleV1::Q1,
            DocumentDirectHaworthBondTokenV1::W1 => DirectGlycosidicHaworthBondStyleV1::W1,
            DocumentDirectHaworthBondTokenV1::N1 => DirectGlycosidicHaworthBondStyleV1::N1,
        };
        let position = match bond.position {
            Some(DocumentHaworthPositionV1::Front) => {
                Some(DirectGlycosidicHaworthPositionV1::Front)
            }
            Some(DocumentHaworthPositionV1::Back) => Some(DirectGlycosidicHaworthPositionV1::Back),
            None => None,
        };
        let endpoints = bond
            .endpoints
            .each_ref()
            .map(|id| atom_by_id(atoms, id).map(|atom| atom.record.clone()));
        bond_facts.push(DurableDirectGlycosidicHaworthBondFactV1::new(
            bond.record.clone(),
            [endpoints[0].clone()?, endpoints[1].clone()?],
            role,
            token,
            position,
            bond.child_order,
        ));
    }
    let rings = [
        ring(first_atoms, first_bonds, shape.first)?,
        ring(second_atoms, second_bonds, shape.second)?,
    ];
    Ok(DurableDirectGlycosidicHaworthProfileV1::new(
        atom_facts, bond_facts, rings,
    ))
}

fn atom_by_id<'a>(
    atoms: &'a [AtomFact],
    id: &PersistentId,
) -> Result<&'a AtomFact, DirectHaworthReobservationErrorV1> {
    atoms
        .iter()
        .find(|atom| &atom.identifier == id)
        .ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)
}

fn positional_cycle(
    atoms: &[AtomFact],
    bonds: &[BondFact],
) -> Result<(), DirectHaworthReobservationErrorV1> {
    let q = bonds
        .iter()
        .position(|bond| bond.token == DocumentDirectHaworthBondTokenV1::Q1)
        .ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)?;
    if bonds
        .iter()
        .filter(|bond| bond.token == DocumentDirectHaworthBondTokenV1::Q1)
        .count()
        != 1
    {
        return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
    }
    for (index, bond) in bonds.iter().enumerate() {
        if bond.position.is_none()
            || !connects(
                bond,
                &atoms[index].identifier,
                &atoms[(index + 1) % atoms.len()].identifier,
            )
        {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
        let previous = (q + atoms.len() - 1) % atoms.len();
        let next = (q + 1) % atoms.len();
        if index == q {
            if bond.position != Some(DocumentHaworthPositionV1::Front) {
                return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
            }
        } else if index == previous {
            if bond.token != DocumentDirectHaworthBondTokenV1::W1
                || bond.position != Some(DocumentHaworthPositionV1::Front)
                || bond.endpoints
                    != [
                        atoms[previous].identifier.clone(),
                        atoms[q].identifier.clone(),
                    ]
            {
                return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
            }
        } else if index == next {
            let outer = (next + 1) % atoms.len();
            if bond.token != DocumentDirectHaworthBondTokenV1::W1
                || bond.position != Some(DocumentHaworthPositionV1::Front)
                || bond.endpoints
                    != [
                        atoms[outer].identifier.clone(),
                        atoms[next].identifier.clone(),
                    ]
            {
                return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
            }
        } else if bond.token != DocumentDirectHaworthBondTokenV1::N1
            || bond.position != Some(DocumentHaworthPositionV1::Back)
        {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
    }
    Ok(())
}

fn connects(bond: &BondFact, left: &PersistentId, right: &PersistentId) -> bool {
    (bond.endpoints[0] == *left && bond.endpoints[1] == *right)
        || (bond.endpoints[0] == *right && bond.endpoints[1] == *left)
}

fn bridge_bonds(
    bonds: &[BondFact],
    first: &[AtomFact],
    second: &[AtomFact],
    bridge: &AtomFact,
) -> Result<(), DirectHaworthReobservationErrorV1> {
    let mut seen = [false; 2];
    for bond in bonds {
        if bond.token != DocumentDirectHaworthBondTokenV1::N1
            || bond.position.is_some()
            || bond.endpoints[1] != bridge.identifier
        {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
        let cycle = if has_carbon(first, &bond.endpoints[0]) {
            0
        } else if has_carbon(second, &bond.endpoints[0]) {
            1
        } else {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        };
        if std::mem::replace(&mut seen[cycle], true) {
            return Err(DirectHaworthReobservationErrorV1::SelectedProfile);
        }
    }
    seen.into_iter()
        .all(|value| value)
        .then_some(())
        .ok_or(DirectHaworthReobservationErrorV1::SelectedProfile)
}

fn has_carbon(atoms: &[AtomFact], identifier: &PersistentId) -> bool {
    atoms.iter().any(|atom| {
        atom.identifier == *identifier
            && atom.element == DirectGlycosidicHaworthAuthoringAtomElementV1::Carbon
    })
}

fn ring(
    atoms: &[AtomFact],
    bonds: &[BondFact],
    count: usize,
) -> Result<DurableDirectGlycosidicHaworthRingFactV1, DirectHaworthReobservationErrorV1> {
    let form = match count {
        5 => RingForm::Furanose,
        6 => RingForm::Pyranose,
        _ => return Err(DirectHaworthReobservationErrorV1::SelectedProfile),
    };
    let mut ring_atoms = Vec::new();
    let mut ring_bonds = Vec::new();
    ring_atoms
        .try_reserve_exact(atoms.len())
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    ring_bonds
        .try_reserve_exact(bonds.len())
        .map_err(|_| DirectHaworthReobservationErrorV1::Allocation)?;
    ring_atoms.extend(atoms.iter().map(|atom| atom.record.clone()));
    ring_bonds.extend(bonds.iter().map(|bond| bond.record.clone()));
    Ok(DurableDirectGlycosidicHaworthRingFactV1::new(
        form, ring_atoms, ring_bonds,
    ))
}
