//! Bounded, deterministic tree assembly for explicit Haworth rings.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrum_core::{RecordId, RecordKind};
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
const LINK_DISTANCE_FACTOR: f64 = 4.0;

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

/// Durable, pure Haworth fragment facts ready for a renderer or operation layer.
#[derive(Clone, Debug, PartialEq)]
pub struct HaworthFragment {
    coordinates: BTreeMap<RecordId, HaworthPoint>,
    ring_bonds: BTreeMap<RecordId, BondDepiction>,
    bond_geometry: BTreeMap<RecordId, [HaworthPoint; 2]>,
    links: BTreeMap<RecordId, HaworthLinkGeometry>,
    label_anchors: BTreeMap<RecordId, HaworthPoint>,
    bounds: [HaworthPoint; 2],
    graph_fingerprint: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct HaworthFragmentWire {
    coordinates: Vec<(RecordId, HaworthPoint)>,
    ring_bonds: Vec<(RecordId, BondDepiction)>,
    bond_geometry: Vec<(RecordId, [HaworthPoint; 2])>,
    links: Vec<(RecordId, HaworthLinkGeometry)>,
    label_anchors: Vec<(RecordId, HaworthPoint)>,
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
            label_anchors: self
                .label_anchors
                .iter()
                .map(|(id, value)| (id.clone(), *value))
                .collect(),
            bounds: self.bounds,
            graph_fingerprint: self.graph_fingerprint,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for HaworthFragment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = HaworthFragmentWire::deserialize(deserializer)?;
        let coordinate_count = wire.coordinates.len();
        let bond_count = wire.ring_bonds.len();
        let geometry_count = wire.bond_geometry.len();
        let link_count = wire.links.len();
        let label_count = wire.label_anchors.len();
        let coordinates: BTreeMap<_, _> = wire.coordinates.into_iter().collect();
        let ring_bonds: BTreeMap<_, _> = wire.ring_bonds.into_iter().collect();
        let bond_geometry: BTreeMap<_, _> = wire.bond_geometry.into_iter().collect();
        let links: BTreeMap<_, _> = wire.links.into_iter().collect();
        let label_anchors: BTreeMap<_, _> = wire.label_anchors.into_iter().collect();
        if coordinates.len() != coordinate_count
            || ring_bonds.len() != bond_count
            || bond_geometry.len() != geometry_count
            || links.len() != link_count
            || label_anchors.len() != label_count
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
            || label_anchors.len() != links.len()
            || links
                .keys()
                .any(|id| label_anchors.get(id) != Some(&links[id].label_anchor))
        {
            return Err(serde::de::Error::custom("fragment wire facts are invalid"));
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
        let expected_fingerprint = fingerprint(&coordinates, &ring_bonds, &links);
        if wire.graph_fingerprint != expected_fingerprint {
            return Err(serde::de::Error::custom(
                "fragment fingerprint does not match durable facts",
            ));
        }
        Ok(Self {
            coordinates,
            ring_bonds,
            bond_geometry,
            links,
            label_anchors,
            bounds: wire.bounds,
            graph_fingerprint: wire.graph_fingerprint,
        })
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
    /// Return deliberate, durable link label anchors.
    #[must_use]
    pub fn label_anchors(&self) -> &BTreeMap<RecordId, HaworthPoint> {
        &self.label_anchors
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
        let parent_depiction =
            placed_depiction(ring_map[&parent], request.scale, placements[&parent])?;
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
            let angle = branch_angle(rank);
            let distance = request.scale * LINK_DISTANCE_FACTOR;
            let desired = HaworthPoint {
                x: parent_anchor.x + distance * angle.cos(),
                y: parent_anchor.y + distance * angle.sin(),
            };
            let translation = (desired.x - child_anchor.x, desired.y - child_anchor.y);
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

fn branch_angle(rank: usize) -> f64 {
    // Golden-angle enumeration is deterministic and does not privilege a fixed ring count.
    let golden_angle = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    (rank as f64) * golden_angle
}

fn placed_depiction(
    ring: &HaworthRingNode,
    scale: f64,
    translation: (f64, f64),
) -> Result<HaworthDepiction, HaworthError> {
    let base = layout_single_ring(&HaworthLayoutRequest {
        topology: ring.topology.clone(),
        scale,
    })?;
    let mut coordinates = base.coordinates().clone();
    for point in coordinates.values_mut() {
        point.x += translation.0;
        point.y += translation.1;
    }
    let bounds = shifted_bounds(base.bounds(), translation);
    HaworthDepiction::new(base.ring_form(), coordinates, base.bonds().clone(), bounds)
}

fn shifted_bounds(bounds: [HaworthPoint; 2], translation: (f64, f64)) -> [HaworthPoint; 2] {
    [
        HaworthPoint {
            x: bounds[0].x + translation.0,
            y: bounds[0].y + translation.1,
        },
        HaworthPoint {
            x: bounds[1].x + translation.0,
            y: bounds[1].y + translation.1,
        },
    ]
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
    for (id, ring) in rings {
        let depiction = placed_depiction(ring, request.scale, placements[id])?;
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
        }
    }
    if !clearance_ok(&coordinates, request.scale) {
        return Err(HaworthError::Unplaceable(
            "nonbonded ring atoms violate the tree clearance",
        ));
    }
    let mut links = BTreeMap::new();
    let mut label_anchors = BTreeMap::new();
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
        label_anchors.insert(link.bond.clone(), anchor);
    }
    let bounds = fragment_bounds(&coordinates, &links)?;
    Ok(HaworthFragment {
        graph_fingerprint: fingerprint(&coordinates, &ring_bonds, &links),
        coordinates,
        ring_bonds,
        bond_geometry,
        links,
        label_anchors,
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

fn fingerprint(
    points: &BTreeMap<RecordId, HaworthPoint>,
    bonds: &BTreeMap<RecordId, BondDepiction>,
    links: &BTreeMap<RecordId, HaworthLinkGeometry>,
) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for value in format!("{points:?}{bonds:?}{links:?}").bytes() {
        hash ^= u64::from(value);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}
