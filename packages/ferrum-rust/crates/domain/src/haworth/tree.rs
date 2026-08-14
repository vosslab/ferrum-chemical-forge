//! Bounded, deterministic tree assembly for explicit Haworth rings.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrum_core::{BondOrder, Molecule, RecordId, RecordKind, VertexRef};
use serde::{Deserialize, Serialize};

use crate::haworth::{
    BondDepiction, HaworthDepiction, HaworthError, HaworthLayoutRequest, HaworthPoint,
    HaworthTopology, layout_single_ring,
};

/// Largest deliberately advertised Haworth tree accepted by this profile.
///
/// The representation and traversal are N-ring rather than cardinality-specific.
/// This ceiling bounds validation, collision checks, and serialized response size.
pub const MAX_TREE_RINGS: usize = 32;
const CLEARANCE_FACTOR: f64 = 0.32;

/// One ring node in an explicitly selected glycosidic tree.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthRingNode {
    /// Caller-owned stable node identity, unique within one request.
    pub node_id: u32,
    /// Graph-validated canonical isolated ring topology.
    pub topology: HaworthTopology,
}

/// A selected atom on one selected ring.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaworthAttachment {
    /// The ring node containing `atom`.
    pub node_id: u32,
    /// The explicit ring atom used by the glycosidic link.
    pub atom: RecordId,
}

/// A directed, already-known glycosidic connection between two ring atoms.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlycosidicLink {
    /// Durable bond identity for this link.
    pub bond: RecordId,
    /// Attachment on the rooted parent ring.
    pub parent: HaworthAttachment,
    /// Attachment on the child ring.
    pub child: HaworthAttachment,
}

/// A bounded request for a rooted, acyclic N-ring Haworth fragment.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthTreeRequest {
    /// Immutable graph snapshot that proves every selected ring and inter-ring bond.
    pub molecule: Molecule,
    /// Explicit ring nodes in arbitrary storage order.
    pub rings: Vec<HaworthRingNode>,
    /// Directed parent-to-child links in arbitrary storage order.
    pub links: Vec<GlycosidicLink>,
    /// Root ring node identity.
    pub root: u32,
    /// Positive finite ring-edge length in Ferrum drawing units.
    pub scale: f64,
}

/// Finite geometry of one accepted glycosidic connection.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaworthLinkGeometry {
    /// Parent endpoint in Ferrum drawing units.
    pub parent: HaworthPoint,
    /// Child endpoint in Ferrum drawing units.
    pub child: HaworthPoint,
    /// Durable midpoint label/connector anchor.
    pub label_anchor: HaworthPoint,
}

/// Durable parent-to-child atom attachment proof for one glycosidic bond.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaworthLinkTopology {
    /// Rooted parent ring identity.
    pub parent_ring: u32,
    /// Rooted child ring identity.
    pub child_ring: u32,
    /// Parent-ring attachment atom in the canonical rooted direction.
    pub parent_atom: RecordId,
    /// Child-ring attachment atom in the canonical rooted direction.
    pub child_atom: RecordId,
}

/// Canonical ordered atom and bond cycle for one durable ring node.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HaworthRingTopology {
    /// Stable ring-node identity within this fragment.
    pub node_id: u32,
    /// Ring form fixes the permitted canonical member count.
    pub ring_form: crate::haworth::RingForm,
    /// Canonical oxygen-first anchor atom.
    pub oxygen_atom: RecordId,
    /// Canonical final anomeric-carbon anchor atom.
    pub anomeric_atom: RecordId,
    /// Canonical cyclic atom order.
    pub atoms: Vec<RecordId>,
    /// Matching cyclic edge order: `bonds[i]` joins `atoms[i]` to next atom.
    pub bonds: Vec<RecordId>,
}

/// Durable, pure Haworth fragment facts ready for a renderer or operation layer.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthFragment {
    coordinates: BTreeMap<RecordId, HaworthPoint>,
    ring_bonds: BTreeMap<RecordId, BondDepiction>,
    ring_cycles: Vec<Vec<RecordId>>,
    ring_topology: Vec<HaworthRingTopology>,
    ring_edges: BTreeMap<RecordId, [RecordId; 2]>,
    bond_geometry: BTreeMap<RecordId, [HaworthPoint; 2]>,
    links: BTreeMap<RecordId, HaworthLinkGeometry>,
    link_topology: BTreeMap<RecordId, HaworthLinkTopology>,
    label_anchors: BTreeMap<RecordId, HaworthPoint>,
    source_orders: BTreeMap<RecordId, u32>,
    bounds: [HaworthPoint; 2],
    graph_fingerprint: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HaworthFragmentWire {
    coordinates: Vec<(RecordId, HaworthPoint)>,
    ring_bonds: Vec<(RecordId, BondDepiction)>,
    ring_cycles: Vec<Vec<RecordId>>,
    ring_topology: Vec<HaworthRingTopology>,
    ring_edges: Vec<(RecordId, [RecordId; 2])>,
    bond_geometry: Vec<(RecordId, [HaworthPoint; 2])>,
    links: Vec<(RecordId, HaworthLinkGeometry)>,
    link_topology: Vec<(RecordId, HaworthLinkTopology)>,
    label_anchors: Vec<(RecordId, HaworthPoint)>,
    source_orders: Vec<(RecordId, u32)>,
    bounds: [HaworthPoint; 2],
    graph_fingerprint: u64,
}

impl Serialize for HaworthFragment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        HaworthFragmentWire {
            coordinates: self
                .coordinates
                .iter()
                .map(|(id, point)| (id.clone(), *point))
                .collect(),
            ring_bonds: self
                .ring_bonds
                .iter()
                .map(|(id, value)| (id.clone(), *value))
                .collect(),
            ring_cycles: self.ring_cycles.clone(),
            ring_topology: self.ring_topology.clone(),
            ring_edges: self
                .ring_edges
                .iter()
                .map(|(id, atoms)| (id.clone(), atoms.clone()))
                .collect(),
            bond_geometry: self
                .bond_geometry
                .iter()
                .map(|(id, value)| (id.clone(), *value))
                .collect(),
            links: self
                .links
                .iter()
                .map(|(id, value)| (id.clone(), *value))
                .collect(),
            link_topology: self
                .link_topology
                .iter()
                .map(|(id, topology)| (id.clone(), topology.clone()))
                .collect(),
            label_anchors: self
                .label_anchors
                .iter()
                .map(|(id, value)| (id.clone(), *value))
                .collect(),
            source_orders: self
                .source_orders
                .iter()
                .map(|(id, order)| (id.clone(), *order))
                .collect(),
            bounds: self.bounds,
            graph_fingerprint: self.graph_fingerprint,
        }
        .serialize(serializer)
    }
}

impl HaworthFragment {
    fn deserialize_wire<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = HaworthFragmentWire::deserialize(deserializer)?;
        let coordinate_count = wire.coordinates.len();
        let bond_count = wire.ring_bonds.len();
        let geometry_count = wire.bond_geometry.len();
        let link_count = wire.links.len();
        let link_topology_count = wire.link_topology.len();
        let label_count = wire.label_anchors.len();
        let source_order_count = wire.source_orders.len();
        let coordinates: BTreeMap<_, _> = wire.coordinates.into_iter().collect();
        let ring_bonds: BTreeMap<_, _> = wire.ring_bonds.into_iter().collect();
        let edge_count = wire.ring_edges.len();
        let ring_edges: BTreeMap<_, _> = wire.ring_edges.into_iter().collect();
        let bond_geometry: BTreeMap<_, _> = wire.bond_geometry.into_iter().collect();
        let links: BTreeMap<_, _> = wire.links.into_iter().collect();
        let link_topology: BTreeMap<_, _> = wire.link_topology.into_iter().collect();
        let label_anchors: BTreeMap<_, _> = wire.label_anchors.into_iter().collect();
        let source_orders: BTreeMap<_, _> = wire.source_orders.into_iter().collect();
        if coordinates.len() != coordinate_count
            || ring_bonds.len() != bond_count
            || ring_edges.len() != edge_count
            || bond_geometry.len() != geometry_count
            || links.len() != link_count
            || link_topology.len() != link_topology_count
            || label_anchors.len() != label_count
            || source_orders.len() != source_order_count
        {
            return Err(serde::de::Error::custom(
                "fragment wire identities must not repeat",
            ));
        }
        if coordinates.is_empty()
            || coordinates
                .iter()
                .any(|(id, point)| id.kind() != RecordKind::Atom || !finite(*point))
            || ring_bonds
                .iter()
                .any(|(id, _)| id.kind() != RecordKind::Bond)
            || bond_geometry.iter().any(|(id, endpoints)| {
                id.kind() != RecordKind::Bond || endpoints.iter().any(|point| !finite(*point))
            })
            || links.iter().any(|(id, link)| {
                id.kind() != RecordKind::Bond
                    || !finite(link.parent)
                    || !finite(link.child)
                    || !finite(link.label_anchor)
            })
            || links.keys().any(|id| ring_bonds.contains_key(id))
            || link_topology.len() != links.len()
            || links.keys().any(|id| !link_topology.contains_key(id))
            || label_anchors.len() != links.len()
            || links
                .keys()
                .any(|id| label_anchors.get(id) != Some(&links[id].label_anchor))
        {
            return Err(serde::de::Error::custom("fragment wire facts are invalid"));
        }
        let selected: BTreeSet<_> = ring_bonds.keys().chain(links.keys()).cloned().collect();
        if source_orders.len() != selected.len()
            || source_orders.keys().any(|bond| !selected.contains(bond))
            || source_orders.values().collect::<BTreeSet<_>>().len() != source_orders.len()
        {
            return Err(serde::de::Error::custom(
                "fragment source orders must be a total unique selected-bond partition",
            ));
        }
        if ring_bonds.len() != bond_geometry.len()
            || ring_bonds.keys().any(|id| !bond_geometry.contains_key(id))
        {
            return Err(serde::de::Error::custom(
                "fragment ring bond geometry must match ring semantics",
            ));
        }
        let expected_bounds =
            fragment_bounds(&coordinates, &links).map_err(serde::de::Error::custom)?;
        if wire.bounds != expected_bounds || !wire.bounds.iter().all(|point| finite(*point)) {
            return Err(serde::de::Error::custom(
                "fragment bounds must be finite and derived",
            ));
        }
        if !crate::haworth::wire_validation::validate_fragment_topology(
            crate::haworth::wire_validation::WireTopologyFacts {
                coordinates: &coordinates,
                bonds: &ring_bonds,
                geometry: &bond_geometry,
                cycles: &wire.ring_cycles,
                edges: &ring_edges,
                rings: &wire.ring_topology,
                links: &links,
                link_topology: &link_topology,
            },
        ) {
            return Err(serde::de::Error::custom(
                "fragment Haworth face and edge facts are invalid",
            ));
        }
        let expected_fingerprint = fingerprint(FingerprintFacts {
            points: &coordinates,
            bonds: &ring_bonds,
            bond_geometry: &bond_geometry,
            ring_cycles: &wire.ring_cycles,
            ring_topology: &wire.ring_topology,
            ring_edges: &ring_edges,
            links: &links,
            link_topology: &link_topology,
            label_anchors: &label_anchors,
            source_orders: &source_orders,
        });
        if wire.graph_fingerprint != expected_fingerprint {
            return Err(serde::de::Error::custom(
                "fragment fingerprint does not match durable facts",
            ));
        }
        Ok(Self {
            coordinates,
            ring_bonds,
            ring_cycles: wire.ring_cycles,
            ring_topology: wire.ring_topology,
            ring_edges,
            bond_geometry,
            links,
            link_topology,
            label_anchors,
            source_orders,
            bounds: wire.bounds,
            graph_fingerprint: wire.graph_fingerprint,
        })
    }

    /// Restore a serialized fragment only by rebuilding it from its authoritative request.
    ///
    /// The request supplies the validated molecular graph and the declared C-1 selection;
    /// a fragment wire alone cannot establish either fact. The wire must exactly match the
    /// deterministic fragment reconstructed from that authority.
    pub fn restore<'de, D>(request: &HaworthTreeRequest, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let restored = Self::deserialize_wire(deserializer)?;
        let authoritative = layout_tree(request).map_err(serde::de::Error::custom)?;
        if restored != authoritative {
            return Err(serde::de::Error::custom(
                "fragment wire does not match authoritative Haworth request",
            ));
        }
        Ok(restored)
    }
}

impl HaworthFragment {
    /// Return all finite atom coordinates keyed by durable identity.
    #[must_use]
    pub fn coordinates(&self) -> &BTreeMap<RecordId, HaworthPoint> {
        &self.coordinates
    }
    /// Return durable semantic face treatment for every ring bond.
    #[must_use]
    pub fn ring_bonds(&self) -> &BTreeMap<RecordId, BondDepiction> {
        &self.ring_bonds
    }
    /// Return finite endpoints for every ring bond.
    #[must_use]
    pub fn bond_geometry(&self) -> &BTreeMap<RecordId, [HaworthPoint; 2]> {
        &self.bond_geometry
    }
    /// Return finite geometry for every glycosidic link.
    #[must_use]
    pub fn links(&self) -> &BTreeMap<RecordId, HaworthLinkGeometry> {
        &self.links
    }
    /// Return graph-bound parent and child atom attachments for every link.
    #[must_use]
    pub fn link_topology(&self) -> &BTreeMap<RecordId, HaworthLinkTopology> {
        &self.link_topology
    }
    /// Return deliberate, durable link label anchors.
    #[must_use]
    pub fn label_anchors(&self) -> &BTreeMap<RecordId, HaworthPoint> {
        &self.label_anchors
    }
    /// Return graph-bound stable source order for every rendered bond.
    #[must_use]
    pub fn source_orders(&self) -> &BTreeMap<RecordId, u32> {
        &self.source_orders
    }
    /// Return bounds derived exactly from all atom and link coordinates.
    #[must_use]
    pub const fn bounds(&self) -> [HaworthPoint; 2] {
        self.bounds
    }
    /// Return a deterministic fingerprint of the accepted topology and layout facts.
    #[must_use]
    pub const fn graph_fingerprint(&self) -> u64 {
        self.graph_fingerprint
    }
}

/// Assemble explicit rings as a deterministic, rooted, collision-free tree.
pub fn layout_tree(request: &HaworthTreeRequest) -> Result<HaworthFragment, HaworthError> {
    if !request.scale.is_finite() || request.scale <= 0.0 {
        return Err(HaworthError::InvalidSpec(
            "scale must be finite and positive",
        ));
    }
    if request.rings.is_empty() || request.rings.len() > MAX_TREE_RINGS {
        return Err(HaworthError::UnsupportedTopology(
            "tree ring count is outside this profile",
        ));
    }
    let ring_map = index_rings(&request.rings)?;
    if !ring_map.contains_key(&request.root) {
        return Err(HaworthError::InvalidSpec("tree root must name a ring node"));
    }
    let children = validate_tree(request, &ring_map)?;
    let mut placements = BTreeMap::new();
    placements.insert(request.root, (0.0, 0.0));
    let mut queue = VecDeque::from([request.root]);
    while let Some(parent) = queue.pop_front() {
        let parent_depiction = crate::haworth::placement::placed_depiction(
            ring_map[&parent],
            request.scale,
            placements[&parent],
        )?;
        let Some(outgoing) = children.get(&parent) else {
            continue;
        };
        for (rank, link) in outgoing.iter().enumerate() {
            let parent_anchor = point_for(&parent_depiction, &link.parent.atom)?;
            let child_base = layout_single_ring(&HaworthLayoutRequest {
                topology: ring_map[&link.child.node_id].topology.clone(),
                scale: request.scale,
            })?;
            let child_anchor = point_for(&child_base, &link.child.atom)?;
            let translation = crate::haworth::placement::find_translation(
                request,
                &ring_map,
                &mut placements,
                link.child.node_id,
                parent_anchor,
                child_anchor,
                rank,
            )?;
            placements.insert(link.child.node_id, translation);
            queue.push_back(link.child.node_id);
        }
    }
    if placements.len() != ring_map.len() {
        return Err(HaworthError::UnsupportedTopology(
            "tree must be connected to its root",
        ));
    }
    assemble_fragment(request, &ring_map, &placements)
}

fn index_rings(rings: &[HaworthRingNode]) -> Result<BTreeMap<u32, &HaworthRingNode>, HaworthError> {
    let mut result = BTreeMap::new();
    for ring in rings {
        if result.insert(ring.node_id, ring).is_some() {
            return Err(HaworthError::InvalidSpec(
                "tree ring node ids must be unique",
            ));
        }
    }
    Ok(result)
}

fn validate_tree<'a>(
    request: &'a HaworthTreeRequest,
    rings: &BTreeMap<u32, &'a HaworthRingNode>,
) -> Result<BTreeMap<u32, Vec<&'a GlycosidicLink>>, HaworthError> {
    if request.links.len() + 1 != rings.len() {
        return Err(HaworthError::UnsupportedTopology(
            "tree must have exactly one fewer link than rings",
        ));
    }
    let mut children = BTreeMap::<u32, Vec<&GlycosidicLink>>::new();
    let mut incoming = BTreeSet::new();
    let mut used_attachment = BTreeSet::new();
    let mut bonds = BTreeSet::new();
    let ring_bonds: BTreeSet<_> = rings
        .values()
        .flat_map(|ring| ring.topology.bond_ids().iter().cloned())
        .collect();
    for link in &request.links {
        if link.bond.kind() != RecordKind::Bond
            || ring_bonds.contains(&link.bond)
            || !bonds.insert(link.bond.clone())
        {
            return Err(HaworthError::InvalidSpec(
                "tree links require unique non-ring bond identities",
            ));
        }
        if link.parent.node_id == link.child.node_id {
            return Err(HaworthError::UnsupportedTopology(
                "tree links cannot join a ring to itself",
            ));
        }
        for attachment in [&link.parent, &link.child] {
            let Some(ring) = rings.get(&attachment.node_id) else {
                return Err(HaworthError::StaleTopology(
                    "link names an absent ring node",
                ));
            };
            if attachment.atom.kind() != RecordKind::Atom
                || !ring
                    .topology
                    .vertices()
                    .iter()
                    .any(|vertex| vertex.atom == attachment.atom)
            {
                return Err(HaworthError::StaleTopology(
                    "link attachment is not a selected ring atom",
                ));
            }
            if !used_attachment.insert((attachment.node_id, attachment.atom.clone())) {
                return Err(HaworthError::UnsupportedTopology(
                    "attachment vertices cannot be reused",
                ));
            }
        }
        if !incoming.insert(link.child.node_id) {
            return Err(HaworthError::UnsupportedTopology(
                "each child ring requires one parent",
            ));
        }
        children.entry(link.parent.node_id).or_default().push(link);
    }
    validate_selected_graph(request, rings, &bonds)?;
    if incoming.contains(&request.root) || incoming.len() + 1 != rings.len() {
        return Err(HaworthError::UnsupportedTopology(
            "tree must have one explicit root and one parent per child",
        ));
    }
    for links in children.values_mut() {
        links.sort_by_key(|link| (link.child.node_id, link.bond.clone()));
    }
    Ok(children)
}

/// Prove that declarations describe all and only the inter-ring bonds in the
/// immutable graph snapshot. The proof is deliberately rebuilt per request so
/// no caller-controlled cache can become stale.
fn validate_selected_graph(
    request: &HaworthTreeRequest,
    rings: &BTreeMap<u32, &HaworthRingNode>,
    declared_links: &BTreeSet<RecordId>,
) -> Result<(), HaworthError> {
    let by_identity: BTreeMap<_, _> = request
        .molecule
        .bonds()
        .iter()
        .map(|bond| (bond.identity().clone(), bond))
        .collect();
    let mut atom_nodes = BTreeMap::new();
    let mut declared_ring_bonds = BTreeSet::new();
    for (node, ring) in rings {
        for vertex in ring.topology.vertices() {
            if atom_nodes.insert(vertex.atom.clone(), *node).is_some() {
                return Err(HaworthError::UnsupportedTopology(
                    "rings cannot share atom identities",
                ));
            }
        }
        for (index, bond_id) in ring.topology.bond_ids().iter().enumerate() {
            let Some(bond) = by_identity.get(bond_id) else {
                return Err(HaworthError::StaleTopology(
                    "selected ring bond is absent from molecule snapshot",
                ));
            };
            let start = &ring.topology.vertices()[index].atom;
            let end = &ring.topology.vertices()[(index + 1) % ring.topology.vertices().len()].atom;
            if !matches!((bond.start(), bond.end()), (VertexRef::Atom(actual_start), VertexRef::Atom(actual_end)) if (actual_start == start && actual_end == end) || (actual_start == end && actual_end == start))
                || bond.order() != Some(BondOrder::Single)
                || bond.aromatic() != Some(false)
                || !declared_ring_bonds.insert(bond_id.clone())
            {
                return Err(HaworthError::StaleTopology(
                    "selected ring edge does not match molecule snapshot",
                ));
            }
        }
    }
    let mut actual_links = BTreeSet::new();
    for bond in request.molecule.bonds() {
        let (VertexRef::Atom(start), VertexRef::Atom(end)) = (bond.start(), bond.end()) else {
            continue;
        };
        let (Some(start_node), Some(end_node)) = (atom_nodes.get(start), atom_nodes.get(end))
        else {
            continue;
        };
        if start_node == end_node {
            if !declared_ring_bonds.contains(bond.identity()) {
                return Err(HaworthError::StaleTopology(
                    "molecule has an undeclared selected-ring bond",
                ));
            }
            continue;
        }
        if bond.order() != Some(BondOrder::Single) || bond.aromatic() != Some(false) {
            return Err(HaworthError::UnsupportedTopology(
                "inter-ring links must be explicit non-aromatic single bonds",
            ));
        }
        actual_links.insert(bond.identity().clone());
        let declared = request
            .links
            .iter()
            .find(|link| link.bond == *bond.identity())
            .ok_or(HaworthError::StaleTopology(
                "molecule has an undeclared inter-ring bond",
            ))?;
        let endpoints_match = matches!(
            (bond.start(), bond.end()),
            (VertexRef::Atom(a), VertexRef::Atom(b))
                if (a == &declared.parent.atom && b == &declared.child.atom)
                    || (a == &declared.child.atom && b == &declared.parent.atom)
        );
        if !endpoints_match {
            return Err(HaworthError::StaleTopology(
                "declared link endpoints do not match graph bond",
            ));
        }
    }
    if &actual_links != declared_links {
        return Err(HaworthError::StaleTopology(
            "declared links do not match complete selected graph topology",
        ));
    }
    Ok(())
}

fn point_for(depiction: &HaworthDepiction, atom: &RecordId) -> Result<HaworthPoint, HaworthError> {
    depiction
        .coordinates()
        .get(atom)
        .copied()
        .ok_or(HaworthError::StaleTopology(
            "attachment coordinate is absent",
        ))
}

fn assemble_fragment(
    request: &HaworthTreeRequest,
    rings: &BTreeMap<u32, &HaworthRingNode>,
    placements: &BTreeMap<u32, (f64, f64)>,
) -> Result<HaworthFragment, HaworthError> {
    let mut coordinates = BTreeMap::new();
    let mut ring_bonds = BTreeMap::new();
    let mut bond_geometry = BTreeMap::new();
    let mut ring_cycles = Vec::new();
    let mut ring_topology = Vec::new();
    let mut ring_edges = BTreeMap::new();
    for (id, ring) in rings {
        let depiction =
            crate::haworth::placement::placed_depiction(ring, request.scale, placements[id])?;
        for (atom, point) in depiction.coordinates() {
            if coordinates.insert(atom.clone(), *point).is_some() {
                return Err(HaworthError::UnsupportedTopology(
                    "rings cannot share atom identities",
                ));
            }
        }
        for (index, bond) in ring.topology.bond_ids().iter().enumerate() {
            let start = depiction.coordinates()[&ring.topology.vertices()[index].atom];
            let end = depiction.coordinates()
                [&ring.topology.vertices()[(index + 1) % ring.topology.vertices().len()].atom];
            ring_bonds.insert(bond.clone(), depiction.bonds()[bond]);
            bond_geometry.insert(bond.clone(), [start, end]);
            ring_edges.insert(
                bond.clone(),
                [
                    ring.topology.vertices()[index].atom.clone(),
                    ring.topology.vertices()[(index + 1) % ring.topology.vertices().len()]
                        .atom
                        .clone(),
                ],
            );
        }
        ring_cycles.push(ring.topology.bond_ids().to_vec());
        ring_topology.push(HaworthRingTopology {
            node_id: *id,
            ring_form: ring.topology.ring_form(),
            oxygen_atom: ring.topology.vertices()[0].atom.clone(),
            anomeric_atom: ring.topology.vertices()[ring.topology.vertices().len() - 1]
                .atom
                .clone(),
            atoms: ring
                .topology
                .vertices()
                .iter()
                .map(|vertex| vertex.atom.clone())
                .collect(),
            bonds: ring.topology.bond_ids().to_vec(),
        });
    }
    if !clearance_ok(&coordinates, request.scale) {
        return Err(HaworthError::Unplaceable(
            "nonbonded ring atoms violate the tree clearance",
        ));
    }
    let mut links = BTreeMap::new();
    let mut link_topology = BTreeMap::new();
    let mut label_anchors = BTreeMap::new();
    let selected_bonds: BTreeSet<_> = ring_bonds
        .keys()
        .chain(request.links.iter().map(|link| &link.bond))
        .cloned()
        .collect();
    let source_orders = request
        .molecule
        .bonds()
        .iter()
        .enumerate()
        .filter_map(|(index, bond)| {
            selected_bonds
                .contains(bond.identity())
                .then_some((bond.identity().clone(), index as u32))
        })
        .collect::<BTreeMap<_, _>>();
    if source_orders.len() != selected_bonds.len() {
        return Err(HaworthError::StaleTopology(
            "selected bond is absent from molecule snapshot",
        ));
    }
    for link in &request.links {
        let parent = coordinates[&link.parent.atom];
        let child = coordinates[&link.child.atom];
        let anchor = HaworthPoint {
            x: (parent.x + child.x) / 2.0,
            y: (parent.y + child.y) / 2.0,
        };
        if !finite(parent) || !finite(child) || !finite(anchor) {
            return Err(HaworthError::Unplaceable(
                "tree link geometry is not finite",
            ));
        }
        links.insert(
            link.bond.clone(),
            HaworthLinkGeometry {
                parent,
                child,
                label_anchor: anchor,
            },
        );
        link_topology.insert(
            link.bond.clone(),
            HaworthLinkTopology {
                parent_ring: link.parent.node_id,
                child_ring: link.child.node_id,
                parent_atom: link.parent.atom.clone(),
                child_atom: link.child.atom.clone(),
            },
        );
        label_anchors.insert(link.bond.clone(), anchor);
    }
    let bounds = fragment_bounds(&coordinates, &links)?;
    if !crate::haworth::wire_validation::validate_fragment_topology(
        crate::haworth::wire_validation::WireTopologyFacts {
            coordinates: &coordinates,
            bonds: &ring_bonds,
            geometry: &bond_geometry,
            cycles: &ring_cycles,
            edges: &ring_edges,
            rings: &ring_topology,
            links: &links,
            link_topology: &link_topology,
        },
    ) {
        return Err(HaworthError::InvalidSpec(
            "assembled Haworth face or geometry semantics are invalid",
        ));
    }
    Ok(HaworthFragment {
        graph_fingerprint: fingerprint(FingerprintFacts {
            points: &coordinates,
            bonds: &ring_bonds,
            bond_geometry: &bond_geometry,
            ring_cycles: &ring_cycles,
            ring_topology: &ring_topology,
            ring_edges: &ring_edges,
            links: &links,
            link_topology: &link_topology,
            label_anchors: &label_anchors,
            source_orders: &source_orders,
        }),
        coordinates,
        ring_bonds,
        ring_cycles,
        ring_topology,
        ring_edges,
        bond_geometry,
        links,
        link_topology,
        label_anchors,
        source_orders,
        bounds,
    })
}

fn clearance_ok(points: &BTreeMap<RecordId, HaworthPoint>, scale: f64) -> bool {
    let clearance_sq = (scale * CLEARANCE_FACTOR).powi(2);
    let values: Vec<_> = points.values().copied().collect();
    values.iter().enumerate().all(|(index, point)| {
        values[index + 1..].iter().all(|other| {
            let dx = point.x - other.x;
            let dy = point.y - other.y;
            dx.mul_add(dx, dy * dy) >= clearance_sq
        })
    })
}

fn fragment_bounds(
    points: &BTreeMap<RecordId, HaworthPoint>,
    links: &BTreeMap<RecordId, HaworthLinkGeometry>,
) -> Result<[HaworthPoint; 2], HaworthError> {
    let mut values: Vec<_> = points.values().copied().collect();
    values.extend(
        links
            .values()
            .flat_map(|link| [link.parent, link.child, link.label_anchor]),
    );
    let Some(first) = values.first().copied() else {
        return Err(HaworthError::Unplaceable("fragment has no points"));
    };
    if values.iter().any(|point| !finite(*point)) {
        return Err(HaworthError::Unplaceable(
            "fragment contains non-finite points",
        ));
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (first.x, first.y, first.x, first.y);
    for point in values {
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

struct FingerprintFacts<'a> {
    points: &'a BTreeMap<RecordId, HaworthPoint>,
    bonds: &'a BTreeMap<RecordId, BondDepiction>,
    bond_geometry: &'a BTreeMap<RecordId, [HaworthPoint; 2]>,
    ring_cycles: &'a [Vec<RecordId>],
    ring_topology: &'a [HaworthRingTopology],
    ring_edges: &'a BTreeMap<RecordId, [RecordId; 2]>,
    links: &'a BTreeMap<RecordId, HaworthLinkGeometry>,
    link_topology: &'a BTreeMap<RecordId, HaworthLinkTopology>,
    label_anchors: &'a BTreeMap<RecordId, HaworthPoint>,
    source_orders: &'a BTreeMap<RecordId, u32>,
}

fn fingerprint(facts: FingerprintFacts<'_>) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for value in format!(
        "{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}",
        facts.points,
        facts.bonds,
        facts.bond_geometry,
        facts.ring_cycles,
        facts.ring_topology,
        facts.ring_edges,
        facts.links,
        facts.link_topology,
        facts.label_anchors,
        facts.source_orders
    )
    .bytes()
    {
        hash ^= u64::from(value);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

#[cfg(test)]
pub(crate) fn refresh_wire_fingerprint(value: &mut serde_json::Value) {
    let wire: HaworthFragmentWire = serde_json::from_value(value.clone()).expect("test wire");
    let coordinates = wire.coordinates.into_iter().collect();
    let bonds = wire.ring_bonds.into_iter().collect();
    let edges = wire.ring_edges.into_iter().collect();
    let geometry = wire.bond_geometry.into_iter().collect();
    let links = wire.links.into_iter().collect();
    let link_topology = wire.link_topology.into_iter().collect();
    let anchors = wire.label_anchors.into_iter().collect();
    let orders = wire.source_orders.into_iter().collect();
    let fingerprint = fingerprint(FingerprintFacts {
        points: &coordinates,
        bonds: &bonds,
        bond_geometry: &geometry,
        ring_cycles: &wire.ring_cycles,
        ring_topology: &wire.ring_topology,
        ring_edges: &edges,
        links: &links,
        link_topology: &link_topology,
        label_anchors: &anchors,
        source_orders: &orders,
    });
    value["graph_fingerprint"] = serde_json::json!(fingerprint);
}
