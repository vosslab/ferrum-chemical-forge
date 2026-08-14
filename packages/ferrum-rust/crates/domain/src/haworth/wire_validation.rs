//! Self-validating durable Haworth fragment topology reconstruction.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrum_core::RecordId;

use crate::haworth::{
    BondDepiction, Face, HaworthLinkGeometry, HaworthLinkTopology, HaworthPoint,
    HaworthRingTopology, RingForm, WedgeEdgeRole,
};

pub(crate) struct WireTopologyFacts<'a> {
    pub(crate) coordinates: &'a BTreeMap<RecordId, HaworthPoint>,
    pub(crate) bonds: &'a BTreeMap<RecordId, BondDepiction>,
    pub(crate) geometry: &'a BTreeMap<RecordId, [HaworthPoint; 2]>,
    pub(crate) cycles: &'a [Vec<RecordId>],
    pub(crate) edges: &'a BTreeMap<RecordId, [RecordId; 2]>,
    pub(crate) rings: &'a [HaworthRingTopology],
    pub(crate) links: &'a BTreeMap<RecordId, HaworthLinkGeometry>,
    pub(crate) link_topology: &'a BTreeMap<RecordId, HaworthLinkTopology>,
}

pub(crate) fn validate_fragment_topology(facts: WireTopologyFacts<'_>) -> bool {
    let WireTopologyFacts {
        coordinates,
        bonds,
        geometry,
        cycles,
        edges,
        rings,
        links,
        link_topology,
    } = facts;
    if rings.is_empty() || rings.len() != cycles.len() || links.len() + 1 != rings.len() {
        return false;
    }
    let mut nodes = BTreeSet::new();
    let mut atoms = BTreeMap::new();
    let mut ring_bonds = BTreeSet::new();
    for (ring, cycle) in rings.iter().zip(cycles) {
        if !(ring.atoms.len() == 5 || ring.atoms.len() == 6)
            || ring.atoms.len() != ring.bonds.len()
            || &ring.bonds != cycle
            || !nodes.insert(ring.node_id)
            || ring.atoms.first() != Some(&ring.oxygen_atom)
            || ring.atoms.last() != Some(&ring.anomeric_atom)
            || ring.atoms.len()
                != match ring.ring_form {
                    RingForm::Pyranose => 6,
                    RingForm::Furanose => 5,
                }
        {
            return false;
        }
        let mut roles = BTreeSet::new();
        for index in 0..ring.atoms.len() {
            let atom = &ring.atoms[index];
            let next = &ring.atoms[(index + 1) % ring.atoms.len()];
            let bond = &ring.bonds[index];
            if atoms.insert(atom.clone(), ring.node_id).is_some()
                || !ring_bonds.insert(bond.clone())
                || edges.get(bond) != Some(&[atom.clone(), next.clone()])
            {
                return false;
            }
            let Some([start, end]) = geometry.get(bond) else {
                return false;
            };
            if coordinates.get(atom) != Some(start)
                || coordinates.get(next) != Some(end)
                || start == end
            {
                return false;
            }
            match bonds.get(bond) {
                Some(BondDepiction::Back { face: Face::Back }) => {}
                Some(BondDepiction::HaworthFront {
                    edge_role,
                    face: Face::Front,
                }) => {
                    roles.insert(*edge_role);
                }
                _ => return false,
            }
        }
        if roles
            != BTreeSet::from([
                WedgeEdgeRole::LeftShoulder,
                WedgeEdgeRole::Center,
                WedgeEdgeRole::RightShoulder,
            ])
        {
            return false;
        }
    }
    if ring_bonds.len() != bonds.len()
        || edges.len() != bonds.len()
        || geometry.len() != bonds.len()
    {
        return false;
    }
    let mut incoming = BTreeSet::new();
    let mut attachments = BTreeSet::new();
    let mut children = BTreeMap::<u32, Vec<u32>>::new();
    for (bond, link) in links {
        let Some(topology) = link_topology.get(bond) else {
            return false;
        };
        if topology.parent_ring == topology.child_ring
            || atoms.get(&topology.parent_atom) != Some(&topology.parent_ring)
            || atoms.get(&topology.child_atom) != Some(&topology.child_ring)
            || coordinates.get(&topology.parent_atom) != Some(&link.parent)
            || coordinates.get(&topology.child_atom) != Some(&link.child)
            || !attachments.insert((topology.parent_ring, topology.parent_atom.clone()))
            || !attachments.insert((topology.child_ring, topology.child_atom.clone()))
            || !incoming.insert(topology.child_ring)
        {
            return false;
        }
        children
            .entry(topology.parent_ring)
            .or_default()
            .push(topology.child_ring);
    }
    if link_topology.len() != links.len()
        || link_topology.keys().any(|bond| !links.contains_key(bond))
    {
        return false;
    }
    let roots = nodes.difference(&incoming).copied().collect::<Vec<_>>();
    if roots.len() != 1 {
        return false;
    }
    let mut seen = BTreeSet::new();
    let mut queue = VecDeque::from([roots[0]]);
    while let Some(node) = queue.pop_front() {
        if !seen.insert(node) {
            return false;
        }
        if let Some(next) = children.get(&node) {
            queue.extend(next);
        }
    }
    seen == nodes
}
