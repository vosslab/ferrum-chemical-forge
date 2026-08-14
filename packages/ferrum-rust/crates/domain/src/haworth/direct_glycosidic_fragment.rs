//! Checked, bridge-aware geometry for one direct-glycosidic Haworth profile.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::RecordId;

use crate::haworth::{
    BondDepiction, DirectGlycosidicHaworthLayoutRequestV1, DirectGlycosidicHaworthTopologyV1,
    HaworthError, HaworthPoint, layout_direct_glycosidic_haworth_v1,
};

/// Owned request to assemble one checked direct-glycosidic fragment.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthFragmentRequestV1 {
    /// Previously graph-validated canonical two-ring topology.
    pub topology: DirectGlycosidicHaworthTopologyV1,
    /// Positive finite local drawing-unit edge length.
    pub scale: f64,
}

/// Immutable lowering receipt for a direct exterior-oxygen glycosidic profile.
///
/// Selected atoms are exactly both canonical ring cycles and the exterior bridge
/// oxygen. Selected bonds are exactly both cycle bond sets and the two bridge
/// bonds. Ring substituents and their bonds are deliberately absent.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectGlycosidicHaworthFragmentV1 {
    topology: DirectGlycosidicHaworthTopologyV1,
    coordinates: BTreeMap<RecordId, HaworthPoint>,
    ring_edges: BTreeMap<RecordId, [RecordId; 2]>,
    ring_depictions: BTreeMap<RecordId, BondDepiction>,
    ring_geometry: BTreeMap<RecordId, [HaworthPoint; 2]>,
    bridge_edges: BTreeMap<RecordId, [RecordId; 2]>,
    bridge_geometry: BTreeMap<RecordId, [HaworthPoint; 2]>,
    atom_source_orders: BTreeMap<RecordId, usize>,
    bond_source_orders: BTreeMap<RecordId, usize>,
    bounds: [HaworthPoint; 2],
}

impl DirectGlycosidicHaworthFragmentV1 {
    /// Return the owned validated topology receipt.
    #[must_use]
    pub const fn topology(&self) -> &DirectGlycosidicHaworthTopologyV1 {
        &self.topology
    }

    /// Return coordinates for exactly the selected atoms.
    #[must_use]
    pub fn coordinates(&self) -> &BTreeMap<RecordId, HaworthPoint> {
        &self.coordinates
    }

    /// Return canonical cyclic endpoint identities keyed by selected ring bond.
    #[must_use]
    pub fn ring_edges(&self) -> &BTreeMap<RecordId, [RecordId; 2]> {
        &self.ring_edges
    }

    /// Return copied Haworth depiction semantics keyed by selected ring bond.
    #[must_use]
    pub fn ring_depictions(&self) -> &BTreeMap<RecordId, BondDepiction> {
        &self.ring_depictions
    }

    /// Return endpoint geometry for selected ring bonds.
    #[must_use]
    pub fn ring_geometry(&self) -> &BTreeMap<RecordId, [HaworthPoint; 2]> {
        &self.ring_geometry
    }

    /// Return `[ring carbon, exterior oxygen]` identities for bridge bonds.
    #[must_use]
    pub fn bridge_edges(&self) -> &BTreeMap<RecordId, [RecordId; 2]> {
        &self.bridge_edges
    }

    /// Return endpoint geometry for bridge bonds in the matching identity order.
    #[must_use]
    pub fn bridge_geometry(&self) -> &BTreeMap<RecordId, [HaworthPoint; 2]> {
        &self.bridge_geometry
    }

    /// Return graph-local source positions for exactly the selected atoms.
    ///
    /// `BTreeMap` key iteration is identity sorting, not source, drawing, or
    /// stereochemical role order.
    #[must_use]
    pub fn atom_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.atom_source_orders
    }

    /// Return graph-local source positions for exactly the selected bonds.
    ///
    /// `BTreeMap` key iteration is identity sorting, not source, drawing, or
    /// stereochemical role order.
    #[must_use]
    pub fn bond_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.bond_source_orders
    }

    /// Return finite bounds over all selected coordinates, including bridge oxygen.
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }
}

/// Assemble a complete checked fragment from validated topology and local layout.
pub fn assemble_direct_glycosidic_haworth_fragment_v1(
    request: &DirectGlycosidicHaworthFragmentRequestV1,
) -> Result<DirectGlycosidicHaworthFragmentV1, HaworthError> {
    let layout = layout_direct_glycosidic_haworth_v1(&DirectGlycosidicHaworthLayoutRequestV1 {
        topology: request.topology.clone(),
        scale: request.scale,
    })?;
    let mut coordinates = BTreeMap::new();
    let mut ring_edges = BTreeMap::new();
    let mut ring_depictions = BTreeMap::new();
    let mut ring_geometry = BTreeMap::new();

    for (ring, depiction) in request.topology.rings().iter().zip(layout.depictions()) {
        for (atom, point) in depiction.coordinates() {
            if coordinates.insert(atom.clone(), *point).is_some() {
                return Err(HaworthError::InvalidSpec(
                    "direct glycosidic ring coordinates must be disjoint",
                ));
            }
        }
        let vertices = ring.topology().vertices();
        for (index, bond) in ring.topology().bond_ids().iter().enumerate() {
            let endpoints = [
                vertices[index].atom.clone(),
                vertices[(index + 1) % vertices.len()].atom.clone(),
            ];
            let geometry = endpoint_geometry(&coordinates, &endpoints)?;
            let depiction_semantics =
                depiction
                    .bonds()
                    .get(bond)
                    .copied()
                    .ok_or(HaworthError::InvalidSpec(
                        "ring depiction is missing selected cycle bond",
                    ))?;
            if ring_edges.insert(bond.clone(), endpoints).is_some()
                || ring_depictions
                    .insert(bond.clone(), depiction_semantics)
                    .is_some()
                || ring_geometry.insert(bond.clone(), geometry).is_some()
            {
                return Err(HaworthError::InvalidSpec(
                    "direct glycosidic ring bonds must be disjoint",
                ));
            }
        }
    }
    if coordinates
        .insert(layout.bridge_atom().clone(), layout.bridge_point())
        .is_some()
    {
        return Err(HaworthError::InvalidSpec(
            "bridge oxygen must be exterior to selected rings",
        ));
    }

    let mut bridge_edges = BTreeMap::new();
    let mut bridge_geometry = BTreeMap::new();
    for (bond, endpoints) in layout.bridge_endpoints() {
        let geometry = endpoint_geometry(&coordinates, endpoints)?;
        if bridge_edges
            .insert(bond.clone(), endpoints.clone())
            .is_some()
            || bridge_geometry.insert(bond.clone(), geometry).is_some()
        {
            return Err(HaworthError::InvalidSpec(
                "direct glycosidic bridge bonds must be distinct",
            ));
        }
    }
    let bounds = coordinate_bounds(coordinates.values().copied())?;
    validate_fragment_facts(
        &request.topology,
        &coordinates,
        &ring_edges,
        &ring_depictions,
        &ring_geometry,
        &bridge_edges,
        &bridge_geometry,
        &layout.atom_source_orders().clone(),
        &layout.bond_source_orders().clone(),
        bounds,
    )?;
    Ok(DirectGlycosidicHaworthFragmentV1 {
        topology: request.topology.clone(),
        coordinates,
        ring_edges,
        ring_depictions,
        ring_geometry,
        bridge_edges,
        bridge_geometry,
        atom_source_orders: layout.atom_source_orders().clone(),
        bond_source_orders: layout.bond_source_orders().clone(),
        bounds,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_fragment_facts(
    topology: &DirectGlycosidicHaworthTopologyV1,
    coordinates: &BTreeMap<RecordId, HaworthPoint>,
    ring_edges: &BTreeMap<RecordId, [RecordId; 2]>,
    ring_depictions: &BTreeMap<RecordId, BondDepiction>,
    ring_geometry: &BTreeMap<RecordId, [HaworthPoint; 2]>,
    bridge_edges: &BTreeMap<RecordId, [RecordId; 2]>,
    bridge_geometry: &BTreeMap<RecordId, [HaworthPoint; 2]>,
    atom_source_orders: &BTreeMap<RecordId, usize>,
    bond_source_orders: &BTreeMap<RecordId, usize>,
    bounds: [HaworthPoint; 2],
) -> Result<(), HaworthError> {
    let selected_atoms: BTreeSet<_> = topology
        .rings()
        .iter()
        .flat_map(|ring| {
            ring.topology()
                .vertices()
                .iter()
                .map(|vertex| vertex.atom.clone())
        })
        .chain(std::iter::once(topology.bridge().atom().clone()))
        .collect();
    let selected_ring_bonds: BTreeSet<_> = topology
        .rings()
        .iter()
        .flat_map(|ring| ring.topology().bond_ids().iter().cloned())
        .collect();
    let selected_bridge_bonds: BTreeSet<_> = topology.bridge().bonds().iter().cloned().collect();
    let selected_bonds: BTreeSet<_> = selected_ring_bonds
        .union(&selected_bridge_bonds)
        .cloned()
        .collect();
    if coordinates.keys().cloned().collect::<BTreeSet<_>>() != selected_atoms
        || atom_source_orders.keys().cloned().collect::<BTreeSet<_>>() != selected_atoms
        || bond_source_orders.keys().cloned().collect::<BTreeSet<_>>() != selected_bonds
    {
        return Err(HaworthError::InvalidSpec(
            "fragment source-order and coordinate identities must cover selected graph facts",
        ));
    }
    if ring_edges.keys().cloned().collect::<BTreeSet<_>>() != selected_ring_bonds
        || ring_depictions.keys().cloned().collect::<BTreeSet<_>>() != selected_ring_bonds
        || ring_geometry.keys().cloned().collect::<BTreeSet<_>>() != selected_ring_bonds
        || bridge_edges.keys().cloned().collect::<BTreeSet<_>>() != selected_bridge_bonds
        || bridge_geometry.keys().cloned().collect::<BTreeSet<_>>() != selected_bridge_bonds
        || !ring_edges
            .keys()
            .all(|bond| !bridge_edges.contains_key(bond))
    {
        return Err(HaworthError::InvalidSpec(
            "fragment bond facts must partition exactly the selected bonds",
        ));
    }
    for ring in topology.rings() {
        let vertices = ring.topology().vertices();
        for (index, bond) in ring.topology().bond_ids().iter().enumerate() {
            let expected = [
                vertices[index].atom.clone(),
                vertices[(index + 1) % vertices.len()].atom.clone(),
            ];
            if ring_edges.get(bond) != Some(&expected)
                || ring_geometry.get(bond) != Some(&endpoint_geometry(coordinates, &expected)?)
            {
                return Err(HaworthError::InvalidSpec(
                    "ring geometry must match canonical selected cycle endpoints",
                ));
            }
        }
    }
    for ring in topology.rings() {
        let bond = ring.attachment_bond();
        let expected = [
            ring.attachment_atom().clone(),
            topology.bridge().atom().clone(),
        ];
        if bridge_edges.get(bond) != Some(&expected)
            || bridge_geometry.get(bond) != Some(&endpoint_geometry(coordinates, &expected)?)
        {
            return Err(HaworthError::InvalidSpec(
                "bridge geometry must match selected carbon and exterior oxygen",
            ));
        }
    }
    if !coordinates.values().copied().all(finite)
        || !bounds.into_iter().all(finite)
        || bounds != coordinate_bounds(coordinates.values().copied())?
    {
        return Err(HaworthError::InvalidSpec(
            "fragment coordinates and bounds must be finite and exact",
        ));
    }
    Ok(())
}

fn endpoint_geometry(
    coordinates: &BTreeMap<RecordId, HaworthPoint>,
    endpoints: &[RecordId; 2],
) -> Result<[HaworthPoint; 2], HaworthError> {
    let start = coordinates
        .get(&endpoints[0])
        .copied()
        .ok_or(HaworthError::InvalidSpec(
            "selected edge start has no coordinate",
        ))?;
    let end = coordinates
        .get(&endpoints[1])
        .copied()
        .ok_or(HaworthError::InvalidSpec(
            "selected edge end has no coordinate",
        ))?;
    Ok([start, end])
}

fn coordinate_bounds(
    points: impl Iterator<Item = HaworthPoint>,
) -> Result<[HaworthPoint; 2], HaworthError> {
    let mut points = points;
    let first = points.next().ok_or(HaworthError::InvalidSpec(
        "fragment has no selected coordinates",
    ))?;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in points {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    Ok([
        HaworthPoint { x: min_x, y: min_y },
        HaworthPoint { x: max_x, y: max_y },
    ])
}

fn finite(point: HaworthPoint) -> bool {
    point.x.is_finite() && point.y.is_finite()
}
