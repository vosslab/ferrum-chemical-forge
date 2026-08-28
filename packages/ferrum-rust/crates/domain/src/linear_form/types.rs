//! Owned graph facts and results for native linear-form planning.

use ferrum_core::RecordId;
use ferrum_geometry::Point2;
use thiserror::Error;

/// Native point-space spacing for a Ferrum-generated linear-form bond.
///
/// This is private planner state represented exactly as the integral CDML token
/// emitted by the document adapter; it is not a serialized protocol itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LinearFormBondLength(u16);

impl LinearFormBondLength {
    /// Ferrum's canonical native linear-form spacing in PostScript points.
    pub const NATIVE: Self = Self(40);

    /// Return the point-space spacing used to construct coordinates.
    #[must_use]
    pub const fn points(self) -> f64 {
        self.0 as f64
    }

    /// Return the exact integral token written to CDML metadata.
    #[must_use]
    pub const fn cdml_integer(self) -> u16 {
        self.0
    }
}

/// One direct atom in durable document source order.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearFormAtomV1 {
    atom_id: RecordId,
    point: Point2,
}

impl LinearFormAtomV1 {
    /// Create one atom fact with a finite point.
    #[must_use]
    pub const fn new(atom_id: RecordId, point: Point2) -> Self {
        Self { atom_id, point }
    }

    /// Return the durable atom identity.
    #[must_use]
    pub const fn atom_id(&self) -> &RecordId {
        &self.atom_id
    }

    /// Return the atom's finite point-space coordinate.
    #[must_use]
    pub const fn point(&self) -> Point2 {
        self.point
    }
}

/// One direct bond between two direct atom identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearFormBondV1 {
    bond_id: RecordId,
    start: RecordId,
    end: RecordId,
}

impl LinearFormBondV1 {
    /// Create one bond fact. Endpoint membership is checked by the planner.
    #[must_use]
    pub const fn new(bond_id: RecordId, start: RecordId, end: RecordId) -> Self {
        Self {
            bond_id,
            start,
            end,
        }
    }

    /// Return the durable bond identity.
    #[must_use]
    pub const fn bond_id(&self) -> &RecordId {
        &self.bond_id
    }

    /// Return the first atom endpoint.
    #[must_use]
    pub const fn start(&self) -> &RecordId {
        &self.start
    }

    /// Return the second atom endpoint.
    #[must_use]
    pub const fn end(&self) -> &RecordId {
        &self.end
    }
}

/// Direct-root graph facts, preserving atom and bond source order.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearFormGraphV1 {
    atoms: Vec<LinearFormAtomV1>,
    bonds: Vec<LinearFormBondV1>,
}

impl LinearFormGraphV1 {
    /// Combine direct atom and bond facts in their durable source order.
    #[must_use]
    pub const fn new(atoms: Vec<LinearFormAtomV1>, bonds: Vec<LinearFormBondV1>) -> Self {
        Self { atoms, bonds }
    }

    /// Return direct atoms in durable source order.
    #[must_use]
    pub fn atoms(&self) -> &[LinearFormAtomV1] {
        &self.atoms
    }

    /// Return direct bonds in durable source order.
    #[must_use]
    pub fn bonds(&self) -> &[LinearFormBondV1] {
        &self.bonds
    }
}

/// A selected nonempty atom set and its direct-root graph facts.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearFormRequestV1 {
    selected_atoms: Vec<RecordId>,
    graph: LinearFormGraphV1,
    bond_length: LinearFormBondLength,
}

impl LinearFormRequestV1 {
    /// Combine exact selected atom identities with one direct-root graph.
    #[must_use]
    pub const fn new(
        selected_atoms: Vec<RecordId>,
        graph: LinearFormGraphV1,
        bond_length: LinearFormBondLength,
    ) -> Self {
        Self {
            selected_atoms,
            graph,
            bond_length,
        }
    }

    /// Return selected atom identities in caller-captured source order.
    #[must_use]
    pub fn selected_atoms(&self) -> &[RecordId] {
        &self.selected_atoms
    }

    /// Return the selected root's direct graph facts.
    #[must_use]
    pub const fn graph(&self) -> &LinearFormGraphV1 {
        &self.graph
    }

    /// Return the native point-space spacing for the requested plan.
    #[must_use]
    pub const fn bond_length(&self) -> LinearFormBondLength {
        self.bond_length
    }
}

/// One atom coordinate replacement in point space.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearFormPointReplacementV1 {
    atom_id: RecordId,
    point: Point2,
}

impl LinearFormPointReplacementV1 {
    pub(crate) const fn new(atom_id: RecordId, point: Point2) -> Self {
        Self { atom_id, point }
    }

    /// Return the replaced durable atom identity.
    #[must_use]
    pub const fn atom_id(&self) -> &RecordId {
        &self.atom_id
    }

    /// Return the replacement finite point.
    #[must_use]
    pub const fn point(&self) -> Point2 {
        self.point
    }
}

/// Exact generated-fragment member order, without a generated fragment identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearFormMetadataShapeV1 {
    atom_members: Vec<RecordId>,
    bond_members: Vec<RecordId>,
}

impl LinearFormMetadataShapeV1 {
    pub(crate) const fn new(atom_members: Vec<RecordId>, bond_members: Vec<RecordId>) -> Self {
        Self {
            atom_members,
            bond_members,
        }
    }

    /// Return vertex members in `linear-form-direction-v1` path order.
    #[must_use]
    pub fn atom_members(&self) -> &[RecordId] {
        &self.atom_members
    }

    /// Return bond members in matching path-edge order.
    #[must_use]
    pub fn bond_members(&self) -> &[RecordId] {
        &self.bond_members
    }
}

/// The complete pure mutation plan, without XML or generated identifiers.
#[derive(Clone, Debug, PartialEq)]
pub struct LinearFormPlanV1 {
    ordered_atoms: Vec<RecordId>,
    ordered_bonds: Vec<RecordId>,
    selected_replacements: Vec<LinearFormPointReplacementV1>,
    exterior_replacements: Vec<LinearFormPointReplacementV1>,
    hydrogen_visible_atoms: Vec<RecordId>,
    metadata: LinearFormMetadataShapeV1,
    bond_length: LinearFormBondLength,
}

impl LinearFormPlanV1 {
    pub(crate) const fn new(
        ordered_atoms: Vec<RecordId>,
        ordered_bonds: Vec<RecordId>,
        selected_replacements: Vec<LinearFormPointReplacementV1>,
        exterior_replacements: Vec<LinearFormPointReplacementV1>,
        hydrogen_visible_atoms: Vec<RecordId>,
        metadata: LinearFormMetadataShapeV1,
        bond_length: LinearFormBondLength,
    ) -> Self {
        Self {
            ordered_atoms,
            ordered_bonds,
            selected_replacements,
            exterior_replacements,
            hydrogen_visible_atoms,
            metadata,
            bond_length,
        }
    }

    /// Return the source-order-directed selected path.
    #[must_use]
    pub fn ordered_atoms(&self) -> &[RecordId] {
        &self.ordered_atoms
    }

    /// Return induced bonds in path order.
    #[must_use]
    pub fn ordered_bonds(&self) -> &[RecordId] {
        &self.ordered_bonds
    }

    /// Return native-spacing replacements for selected atoms.
    #[must_use]
    pub fn selected_replacements(&self) -> &[LinearFormPointReplacementV1] {
        &self.selected_replacements
    }

    /// Return translations for uniquely anchored exterior components.
    #[must_use]
    pub fn exterior_replacements(&self) -> &[LinearFormPointReplacementV1] {
        &self.exterior_replacements
    }

    /// Return selected atoms whose hydrogen visibility becomes explicit.
    #[must_use]
    pub fn hydrogen_visible_atoms(&self) -> &[RecordId] {
        &self.hydrogen_visible_atoms
    }

    /// Return the generated-metadata shape without a fragment identity.
    #[must_use]
    pub const fn metadata(&self) -> &LinearFormMetadataShapeV1 {
        &self.metadata
    }

    /// Return the exact native spacing that produced this plan.
    #[must_use]
    pub const fn bond_length(&self) -> LinearFormBondLength {
        self.bond_length
    }
}

/// Typed refusal or resource failure from pure linear-form planning.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LinearFormPlanErrorV1 {
    /// No atoms were selected.
    #[error("linear form requires at least one selected atom")]
    EmptySelection,
    /// One selected durable atom identity was repeated.
    #[error("linear form selected one atom more than once")]
    DuplicateAtomId,
    /// One direct durable bond identity was repeated.
    #[error("linear form graph repeats one durable bond identity")]
    DuplicateBondId,
    /// A selected or direct bond endpoint is not one direct atom of this graph.
    #[error("linear form refers to an unknown or foreign atom")]
    UnknownOrForeignAtom,
    /// The induced selected graph is not one simple path.
    #[error("linear form selection is not a single simple path")]
    NotSinglePath,
    /// An input or derived coordinate is not finite.
    #[error("linear form requires finite point coordinates")]
    NonFinitePoint,
    /// An exterior component touches more than one selected path atom.
    #[error("linear form exterior component has multiple selected anchors")]
    ExteriorComponentHasMultipleAnchors,
    /// Required owned planner storage could not be reserved.
    #[error("linear form planner resource allocation failed")]
    ResourceExhausted,
}
