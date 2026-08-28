//! Immutable molecule and bond projection values without document traversal authority.

use std::collections::HashSet;

use ferrum_core::{BondOrder, BondStyle};
use serde::Serialize;
use thiserror::Error;

use super::{
    AtomProjectionV1, DirectMoleculeGraphAtomFact, DirectMoleculeGraphAtomInput,
    DirectMoleculeGraphBondFact, DirectMoleculeGraphEndpoint, DirectMoleculeGraphFacts,
    DirectMoleculeGraphFactsError, DoubleBondCarrierMarkProjectionV1, NonAtomVertexFact,
    NonAtomVertexKindV1,
};
use crate::{
    CompactGroupProjectionV1, DocumentObjectIdV1, NonZeroFiniteV1, PositiveFiniteV1,
    ProjectionLocalObjectKeyV1, Rgb24V1,
};

/// The typed target category carried by one retained bond endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BondEndpointKindV1 {
    Atom,
    Group,
    MoleculeText,
    Query,
    Unknown,
    Missing,
}

/// Authored Haworth depth carried by a retained bond presentation fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentHaworthPositionV1 {
    Front,
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
    /// Construct the absence of one required endpoint attribute.
    #[must_use]
    pub const fn missing() -> Self {
        Self {
            source_id: None,
            object_id: None,
            kind: BondEndpointKindV1::Missing,
        }
    }
    /// Construct an endpoint whose authored token resolves to no retained object.
    #[must_use]
    pub fn unknown(source_id: String) -> Self {
        Self {
            source_id: Some(source_id),
            object_id: None,
            kind: BondEndpointKindV1::Unknown,
        }
    }
    /// Construct a retained endpoint only for one resolved target category.
    #[must_use]
    pub fn resolved(
        source_id: String,
        object_id: DocumentObjectIdV1,
        kind: BondEndpointKindV1,
    ) -> Option<Self> {
        matches!(
            kind,
            BondEndpointKindV1::Atom
                | BondEndpointKindV1::Group
                | BondEndpointKindV1::MoleculeText
                | BondEndpointKindV1::Query
        )
        .then_some(Self {
            source_id: Some(source_id),
            object_id: Some(object_id),
            kind,
        })
    }
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
    id: DocumentObjectIdV1,
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
    /// Construct a bond projection from already-normalized source facts.
    #[expect(
        clippy::too_many_arguments,
        reason = "each immutable bond fact remains explicit at the projection boundary"
    )]
    #[must_use]
    pub fn new(
        id: DocumentObjectIdV1,
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
    ) -> Self {
        Self {
            id,
            projection_key,
            source_id,
            source_order,
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
        }
    }
    /// Return the exact durable document object ID for this retained bond.
    #[must_use]
    pub fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.id
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
    id: DocumentObjectIdV1,
    projection_key: ProjectionLocalObjectKeyV1,
    source_id: Option<String>,
    name: Option<String>,
    atoms: Vec<AtomProjectionV1>,
    compact_groups: Vec<CompactGroupProjectionV1>,
    non_atom_vertices: Vec<NonAtomVertexProjectionV1>,
    bonds: Vec<BondProjectionV1>,
    stereo_depictions: Vec<DoubleBondCarrierMarkProjectionV1>,
}

/// Complete immutable child inventory required to construct a molecule projection.
pub struct MoleculeProjectionChildrenV1 {
    pub atoms: Vec<AtomProjectionV1>,
    pub compact_groups: Vec<CompactGroupProjectionV1>,
    pub non_atom_vertices: Vec<NonAtomVertexProjectionV1>,
    pub bonds: Vec<BondProjectionV1>,
}

/// Closed retained inventory for each non-atom molecule child.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NonAtomVertexProjectionV1 {
    id: DocumentObjectIdV1,
    projection_key: ProjectionLocalObjectKeyV1,
    source_id: Option<String>,
    source_order: u32,
    kind: NonAtomVertexKindV1,
}
impl NonAtomVertexProjectionV1 {
    #[must_use]
    pub const fn new(
        id: DocumentObjectIdV1,
        projection_key: ProjectionLocalObjectKeyV1,
        source_id: Option<String>,
        source_order: u32,
        kind: NonAtomVertexKindV1,
    ) -> Self {
        Self {
            id,
            projection_key,
            source_id,
            source_order,
            kind,
        }
    }
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.id
    }
    #[must_use]
    pub const fn projection_key(&self) -> &ProjectionLocalObjectKeyV1 {
        &self.projection_key
    }
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.source_order
    }
    #[must_use]
    pub const fn kind(&self) -> NonAtomVertexKindV1 {
        self.kind
    }
}

/// Closed refusal taxonomy for invalid molecule child aggregates.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum MoleculeProjectionV1Error {
    /// One child category did not preserve its source encounter order.
    #[error(
        "{child_kind} child source order must strictly increase: {previous_source_order} before {source_order}"
    )]
    UnorderedChildSourceOrder {
        child_kind: &'static str,
        previous_source_order: u32,
        source_order: u32,
    },
    /// Two retained children claim the same absolute source position.
    #[error("molecule child source order {source_order} is duplicated")]
    DuplicateChildSourceOrder { source_order: u32 },
    /// Two retained children claim one projection-local identity.
    #[error("molecule child projection key is duplicated: {projection_key}")]
    DuplicateChildProjectionKey { projection_key: String },
    /// Two retained children claim one authored source identity.
    #[error("molecule child source ID is duplicated: {source_id}")]
    DuplicateChildSourceId { source_id: String },
    /// Two retained children claim one durable document identity.
    #[error("molecule child document object ID is duplicated: {object_id}")]
    DuplicateChildDocumentObjectId { object_id: String },
    /// The retained compact-group view and closed non-atom inventory differ.
    #[error("compact-group inventory is not an exact one-to-one correspondence")]
    CompactGroupInventoryMismatch,
}

impl MoleculeProjectionV1 {
    /// Construct a validated molecule projection from immutable source-ordered children.
    pub fn try_new(
        id: DocumentObjectIdV1,
        projection_key: ProjectionLocalObjectKeyV1,
        source_id: Option<String>,
        name: Option<String>,
        children: MoleculeProjectionChildrenV1,
    ) -> Result<Self, MoleculeProjectionV1Error> {
        let MoleculeProjectionChildrenV1 {
            atoms,
            compact_groups,
            non_atom_vertices,
            bonds,
        } = children;
        validate_strict_source_order("atom", &atoms, AtomProjectionV1::source_order)?;
        validate_strict_source_order(
            "compact-group",
            &compact_groups,
            CompactGroupProjectionV1::source_order,
        )?;
        validate_strict_source_order("bond", &bonds, BondProjectionV1::source_order)?;
        validate_strict_source_order(
            "non-atom",
            &non_atom_vertices,
            NonAtomVertexProjectionV1::source_order,
        )?;
        validate_compact_group_inventory(&compact_groups, &non_atom_vertices)?;
        validate_child_identities(&atoms, &non_atom_vertices, &bonds)?;
        Ok(Self {
            id,
            projection_key,
            source_id,
            name,
            atoms,
            compact_groups,
            non_atom_vertices,
            bonds,
            stereo_depictions: Vec::new(),
        })
    }
    /// Return the exact durable document object ID for this retained molecule.
    #[must_use]
    pub fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.id
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
    /// Return first-class compact groups in nested source order.
    #[must_use]
    pub fn compact_groups(&self) -> &[CompactGroupProjectionV1] {
        &self.compact_groups
    }

    /// Return the complete retained non-atom topology inventory.
    #[must_use]
    pub fn non_atom_vertices(&self) -> &[NonAtomVertexProjectionV1] {
        &self.non_atom_vertices
    }

    /// Convert this complete projection to pure graph-lowering facts.
    pub fn direct_molecule_graph_facts(
        &self,
        include_coordinates: bool,
    ) -> Result<DirectMoleculeGraphFacts, DirectMoleculeGraphFactsError> {
        self.direct_molecule_graph_facts_with_atom_metadata(include_coordinates, |_| ())
            .map(|(facts, _)| facts)
    }

    /// Convert this projection to facts while emitting caller-owned atom metadata.
    ///
    /// The returned metadata has the exact same order as the emitted atom facts.
    /// This transfers no graph or document authority to the projection layer.
    pub fn direct_molecule_graph_facts_with_atom_metadata<T>(
        &self,
        include_coordinates: bool,
        mut atom_metadata: impl FnMut(&AtomProjectionV1) -> T,
    ) -> Result<(DirectMoleculeGraphFacts, Vec<T>), DirectMoleculeGraphFactsError> {
        let (atoms, metadata): (Vec<_>, Vec<T>) = self
            .atoms
            .iter()
            .map(|atom| {
                (
                    DirectMoleculeGraphAtomFact::new(DirectMoleculeGraphAtomInput {
                        element: atom.element().map(str::to_owned),
                        position: atom.position(),
                        formal_charge: atom.formal_charge(),
                        isotope: atom.isotope(),
                        explicit_hydrogens: atom.explicit_hydrogens(),
                        valence: atom.valence(),
                        multiplicity: atom.multiplicity(),
                        free_sites: atom.free_sites(),
                    }),
                    atom_metadata(atom),
                )
            })
            .unzip();
        let mut atom_positions = std::collections::HashMap::new();
        for (position, atom) in self.atoms.iter().enumerate() {
            atom_positions.insert(atom.document_object_id(), position);
        }
        let endpoint = |endpoint: &BondEndpointV1| match endpoint.kind() {
            BondEndpointKindV1::Atom => endpoint
                .object_id()
                .and_then(|id| atom_positions.get(id).copied())
                .map(DirectMoleculeGraphEndpoint::Atom)
                .ok_or(DirectMoleculeGraphFactsError::UnresolvedEndpoint),
            BondEndpointKindV1::Group
            | BondEndpointKindV1::MoleculeText
            | BondEndpointKindV1::Query => Ok(DirectMoleculeGraphEndpoint::NonAtom),
            BondEndpointKindV1::Unknown => Ok(DirectMoleculeGraphEndpoint::Unknown),
            BondEndpointKindV1::Missing => Ok(DirectMoleculeGraphEndpoint::Missing),
        };
        let mut bonds = Vec::with_capacity(self.bonds.len());
        for bond in &self.bonds {
            bonds.push(DirectMoleculeGraphBondFact::new(
                endpoint(bond.start())?,
                endpoint(bond.end())?,
                bond.order(),
                bond.style().cloned(),
            ));
        }
        let non_atoms = self
            .non_atom_vertices
            .iter()
            .map(|item| NonAtomVertexFact::new(item.kind(), item.source_order()))
            .collect();
        Ok((
            DirectMoleculeGraphFacts::new(atoms, bonds, non_atoms, include_coordinates),
            metadata,
        ))
    }
    /// Return bonds in nested source order.
    #[must_use]
    pub fn bonds(&self) -> &[BondProjectionV1] {
        &self.bonds
    }

    /// Attach resolved E/Z carrier facts from the same document observation.
    #[must_use]
    pub fn with_double_bond_carrier_marks(
        mut self,
        stereo_depictions: Vec<DoubleBondCarrierMarkProjectionV1>,
    ) -> Self {
        self.stereo_depictions = stereo_depictions;
        self
    }

    /// Return explicit E/Z carrier marks without deriving geometry or chemistry.
    #[must_use]
    pub fn double_bond_carrier_marks(&self) -> &[DoubleBondCarrierMarkProjectionV1] {
        &self.stereo_depictions
    }
}

fn validate_strict_source_order<T>(
    child_kind: &'static str,
    children: &[T],
    source_order: impl Fn(&T) -> u32,
) -> Result<(), MoleculeProjectionV1Error> {
    let mut previous_source_order = None;
    for child in children {
        let current_source_order = source_order(child);
        if let Some(previous_source_order) = previous_source_order
            && current_source_order <= previous_source_order
        {
            return Err(MoleculeProjectionV1Error::UnorderedChildSourceOrder {
                child_kind,
                previous_source_order,
                source_order: current_source_order,
            });
        }
        previous_source_order = Some(current_source_order);
    }
    Ok(())
}

fn validate_compact_group_inventory(
    compact_groups: &[CompactGroupProjectionV1],
    non_atom_vertices: &[NonAtomVertexProjectionV1],
) -> Result<(), MoleculeProjectionV1Error> {
    let compact_inventory = non_atom_vertices
        .iter()
        .filter(|value| value.kind() == NonAtomVertexKindV1::CompactGroup)
        .collect::<Vec<_>>();
    if compact_inventory.len() != compact_groups.len()
        || compact_groups.iter().any(|group| {
            !compact_inventory.iter().any(|item| {
                item.document_object_id() == group.document_object_id()
                    && item.source_order() == group.source_order()
            })
        })
    {
        return Err(MoleculeProjectionV1Error::CompactGroupInventoryMismatch);
    }
    Ok(())
}

fn validate_child_identities(
    atoms: &[AtomProjectionV1],
    non_atom_vertices: &[NonAtomVertexProjectionV1],
    bonds: &[BondProjectionV1],
) -> Result<(), MoleculeProjectionV1Error> {
    let mut source_orders = HashSet::new();
    let mut projection_keys = HashSet::new();
    let mut source_ids = HashSet::new();
    let mut object_ids = HashSet::new();
    for child in atoms
        .iter()
        .map(MoleculeChildIdentityV1::from_atom)
        .chain(
            non_atom_vertices
                .iter()
                .map(MoleculeChildIdentityV1::from_non_atom),
        )
        .chain(bonds.iter().map(MoleculeChildIdentityV1::from_bond))
    {
        if !source_orders.insert(child.source_order) {
            return Err(MoleculeProjectionV1Error::DuplicateChildSourceOrder {
                source_order: child.source_order,
            });
        }
        if !projection_keys.insert(child.projection_key) {
            return Err(MoleculeProjectionV1Error::DuplicateChildProjectionKey {
                projection_key: child.projection_key.to_owned(),
            });
        }
        if let Some(source_id) = child.source_id
            && !source_ids.insert(source_id)
        {
            return Err(MoleculeProjectionV1Error::DuplicateChildSourceId {
                source_id: source_id.to_owned(),
            });
        }
        if !object_ids.insert(child.object_id) {
            return Err(MoleculeProjectionV1Error::DuplicateChildDocumentObjectId {
                object_id: child.object_id.as_str().to_owned(),
            });
        }
    }
    Ok(())
}

struct MoleculeChildIdentityV1<'a> {
    source_order: u32,
    projection_key: &'a str,
    source_id: Option<&'a str>,
    object_id: &'a DocumentObjectIdV1,
}

impl<'a> MoleculeChildIdentityV1<'a> {
    fn from_atom(atom: &'a AtomProjectionV1) -> Self {
        Self {
            source_order: atom.source_order(),
            projection_key: atom.projection_key().as_str(),
            source_id: atom.source_id(),
            object_id: atom.document_object_id(),
        }
    }

    fn from_non_atom(non_atom: &'a NonAtomVertexProjectionV1) -> Self {
        Self {
            source_order: non_atom.source_order(),
            projection_key: non_atom.projection_key().as_str(),
            source_id: non_atom.source_id(),
            object_id: non_atom.document_object_id(),
        }
    }

    fn from_bond(bond: &'a BondProjectionV1) -> Self {
        Self {
            source_order: bond.source_order(),
            projection_key: bond.projection_key().as_str(),
            source_id: bond.source_id(),
            object_id: bond.document_object_id(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MoleculeProjectionChildrenV1, MoleculeProjectionV1, MoleculeProjectionV1Error};
    use crate::{
        AtomProjectionV1, CompactGroupAttachmentV1, CompactGroupCatalogKeyV1,
        CompactGroupProjectionV1, CompactGroupV1, DocumentObjectIdV1, NonAtomVertexKindV1,
        NonAtomVertexProjectionV1, Point3V1, ProjectionLocalObjectKeyV1,
    };

    fn atom(source_order: u32, path: &[u32], source_id: &str) -> AtomProjectionV1 {
        atom_with_id(
            DocumentObjectIdV1::from_entropy_bytes([source_order as u8; 16]),
            source_order,
            path,
            source_id,
        )
    }

    fn atom_with_id(
        id: DocumentObjectIdV1,
        source_order: u32,
        path: &[u32],
        source_id: &str,
    ) -> AtomProjectionV1 {
        AtomProjectionV1::new(
            id,
            ProjectionLocalObjectKeyV1::from_path_components(path)
                .expect("nonempty test path is a projection key"),
            Some(source_id.to_owned()),
            source_order,
            Some("C".to_owned()),
            Point3V1::new(0.0, 0.0, 0.0).expect("finite test coordinates"),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Vec::new(),
            None,
            None,
            None,
            None,
            None,
        )
    }

    fn compact_group(id: DocumentObjectIdV1, source_order: u32) -> CompactGroupProjectionV1 {
        let catalog_key = CompactGroupCatalogKeyV1::Methyl;
        let attachment = CompactGroupAttachmentV1::new(catalog_key, 0, 0.0)
            .expect("methyl accepts its first attachment site");
        let group = CompactGroupV1::new(
            id,
            catalog_key,
            Point3V1::new(1.0, 0.0, 0.0).expect("finite test coordinates"),
            attachment,
        );
        CompactGroupProjectionV1::from_group(&group, source_order)
    }

    fn non_atom(
        id: DocumentObjectIdV1,
        source_order: u32,
        kind: NonAtomVertexKindV1,
    ) -> NonAtomVertexProjectionV1 {
        NonAtomVertexProjectionV1::new(
            id,
            ProjectionLocalObjectKeyV1::from_path_components(&[9, source_order]).expect("test key"),
            Some(format!("non-atom-{source_order}")),
            source_order,
            kind,
        )
    }

    fn molecule(
        atoms: Vec<AtomProjectionV1>,
    ) -> Result<MoleculeProjectionV1, MoleculeProjectionV1Error> {
        MoleculeProjectionV1::try_new(
            DocumentObjectIdV1::from_entropy_bytes([0; 16]),
            ProjectionLocalObjectKeyV1::from_path_components(&[0])
                .expect("nonempty test path is a projection key"),
            Some("molecule-1".to_owned()),
            None,
            MoleculeProjectionChildrenV1 {
                atoms,
                compact_groups: Vec::new(),
                non_atom_vertices: Vec::new(),
                bonds: Vec::new(),
            },
        )
    }

    #[test]
    fn molecule_factory_accepts_mixed_children_and_refuses_cross_kind_identity_collision() {
        let group_id = DocumentObjectIdV1::from_entropy_bytes([2; 16]);
        assert!(
            MoleculeProjectionV1::try_new(
                DocumentObjectIdV1::from_entropy_bytes([3; 16]),
                ProjectionLocalObjectKeyV1::from_path_components(&[0])
                    .expect("nonempty test path is a projection key"),
                Some("molecule-1".to_owned()),
                None,
                MoleculeProjectionChildrenV1 {
                    atoms: vec![atom(1, &[0, 0], "atom-1")],
                    compact_groups: vec![compact_group(group_id.clone(), 2)],
                    non_atom_vertices: vec![NonAtomVertexProjectionV1::new(
                        group_id.clone(),
                        ProjectionLocalObjectKeyV1::from_path_components(&[0, 1])
                            .expect("test key"),
                        Some("group-1".to_owned()),
                        2,
                        NonAtomVertexKindV1::CompactGroup,
                    )],
                    bonds: Vec::new(),
                },
            )
            .is_ok()
        );

        assert!(matches!(
            MoleculeProjectionV1::try_new(
                DocumentObjectIdV1::from_entropy_bytes([4; 16]),
                ProjectionLocalObjectKeyV1::from_path_components(&[0])
                    .expect("nonempty test path is a projection key"),
                Some("molecule-1".to_owned()),
                None,
                MoleculeProjectionChildrenV1 {
                    atoms: vec![atom_with_id(group_id.clone(), 1, &[0, 0], "atom-1")],
                    compact_groups: vec![compact_group(group_id.clone(), 2)],
                    non_atom_vertices: vec![NonAtomVertexProjectionV1::new(
                    group_id.clone(),
                    ProjectionLocalObjectKeyV1::from_path_components(&[0, 1])
                        .expect("test key"),
                    Some("group-1".to_owned()),
                    2,
                    NonAtomVertexKindV1::CompactGroup,
                    )],
                    bonds: Vec::new(),
                },
            ),
            Err(MoleculeProjectionV1Error::DuplicateChildDocumentObjectId { object_id })
                if object_id == group_id.as_str()
        ));
    }

    #[test]
    fn molecule_factory_refuses_invalid_child_order_and_identity() {
        let first = atom(1, &[0, 0], "atom-1");
        let second = atom(2, &[0, 1], "atom-2");
        assert!(molecule(vec![first.clone(), second.clone()]).is_ok());
        assert!(matches!(
            molecule(vec![second.clone(), first.clone()]),
            Err(MoleculeProjectionV1Error::UnorderedChildSourceOrder {
                child_kind: "atom",
                previous_source_order: 2,
                source_order: 1,
            })
        ));
        assert!(matches!(
            molecule(vec![first, atom(2, &[0, 0], "atom-1")]),
            Err(MoleculeProjectionV1Error::DuplicateChildProjectionKey { .. })
        ));
    }

    #[test]
    fn molecule_factory_requires_exact_compact_group_inventory() {
        let group_id = DocumentObjectIdV1::from_entropy_bytes([7; 16]);
        let group = compact_group(group_id.clone(), 2);
        let make = |inventory| {
            MoleculeProjectionV1::try_new(
                DocumentObjectIdV1::from_entropy_bytes([8; 16]),
                ProjectionLocalObjectKeyV1::from_path_components(&[0]).expect("test key"),
                None,
                None,
                MoleculeProjectionChildrenV1 {
                    atoms: vec![atom(1, &[0, 0], "atom-1")],
                    compact_groups: vec![group.clone()],
                    non_atom_vertices: inventory,
                    bonds: Vec::new(),
                },
            )
        };
        assert!(matches!(
            make(Vec::new()),
            Err(MoleculeProjectionV1Error::CompactGroupInventoryMismatch)
        ));
        assert!(matches!(
            make(vec![non_atom(
                DocumentObjectIdV1::from_entropy_bytes([6; 16]),
                2,
                NonAtomVertexKindV1::CompactGroup,
            )]),
            Err(MoleculeProjectionV1Error::CompactGroupInventoryMismatch)
        ));
        assert!(matches!(
            make(vec![non_atom(
                group_id.clone(),
                3,
                NonAtomVertexKindV1::CompactGroup
            )]),
            Err(MoleculeProjectionV1Error::CompactGroupInventoryMismatch)
        ));
        assert!(
            make(vec![non_atom(
                group_id,
                2,
                NonAtomVertexKindV1::CompactGroup
            )])
            .is_ok()
        );
    }

    #[test]
    fn molecule_factory_refuses_surplus_duplicate_and_cross_kind_non_atom_collisions() {
        let group_id = DocumentObjectIdV1::from_entropy_bytes([7; 16]);
        let base = |inventory| {
            MoleculeProjectionV1::try_new(
                DocumentObjectIdV1::from_entropy_bytes([8; 16]),
                ProjectionLocalObjectKeyV1::from_path_components(&[0]).expect("test key"),
                None,
                None,
                MoleculeProjectionChildrenV1 {
                    atoms: vec![atom(1, &[0, 0], "atom-1")],
                    compact_groups: vec![compact_group(group_id.clone(), 2)],
                    non_atom_vertices: inventory,
                    bonds: Vec::new(),
                },
            )
        };
        assert!(
            base(vec![
                non_atom(group_id.clone(), 2, NonAtomVertexKindV1::CompactGroup),
                non_atom(
                    DocumentObjectIdV1::from_entropy_bytes([9; 16]),
                    3,
                    NonAtomVertexKindV1::Query
                ),
            ])
            .is_ok(),
            "a complete inventory may contain a valid query child"
        );
        assert!(matches!(
            base(vec![
                non_atom(group_id.clone(), 2, NonAtomVertexKindV1::CompactGroup),
                non_atom(group_id.clone(), 3, NonAtomVertexKindV1::MoleculeText),
            ]),
            Err(MoleculeProjectionV1Error::DuplicateChildDocumentObjectId { .. })
        ));
        assert!(matches!(
            base(vec![
                non_atom(group_id.clone(), 2, NonAtomVertexKindV1::CompactGroup),
                non_atom(
                    DocumentObjectIdV1::from_entropy_bytes([9; 16]),
                    1,
                    NonAtomVertexKindV1::Query
                ),
            ]),
            Err(MoleculeProjectionV1Error::UnorderedChildSourceOrder {
                child_kind: "non-atom",
                ..
            })
        ));
    }
}
