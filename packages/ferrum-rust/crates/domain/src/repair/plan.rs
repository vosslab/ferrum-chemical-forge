//! Deterministic coordinate-only repair planners.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use ferrum_core::RecordId;
use ferrum_geometry::{HexGrid, Point2, straighten_depiction};

use crate::repair::{
    CoordinatePatch, DepictionGraph, RepairError, RepairKind, RepairOutcome, RepairRequest,
};

type AngleComponent = (Vec<RecordId>, Option<RecordId>);

/// Calculate a sparse replacement patch without mutating the request graph.
pub fn plan_repair(request: &RepairRequest) -> Result<CoordinatePatch, RepairError> {
    plan_repair_with_outcome(request).map(RepairOutcome::into_patch)
}

/// Calculate a guarded sparse patch and any complete operation-specific result.
pub fn plan_repair_with_outcome(request: &RepairRequest) -> Result<RepairOutcome, RepairError> {
    match request.kind() {
        RepairKind::SnapToHexGrid { spacing, origin } => {
            snap_to_hex(request.graph(), spacing, origin).map(RepairOutcome::from_patch)
        }
        RepairKind::Straighten { minimize_rotation } => {
            straighten(request.graph(), minimize_rotation)
        }
        RepairKind::StraightenTerminalBonds => {
            straighten_terminal_bonds(request.graph()).map(RepairOutcome::from_patch)
        }
        RepairKind::NormalizeBondLengths { spacing } => {
            normalize_bond_lengths(request.graph(), spacing).map(RepairOutcome::from_patch)
        }
        RepairKind::NormalizeSingleRing { spacing } => {
            normalize_single_ring(request.graph(), spacing).map(RepairOutcome::from_patch)
        }
        RepairKind::NormalizeBondAngles { spacing } => {
            normalize_bond_angles(request.graph(), spacing).map(RepairOutcome::from_patch)
        }
    }
}

fn straighten_terminal_bonds(graph: &DepictionGraph) -> Result<CoordinatePatch, RepairError> {
    let neighbors = adjacency(graph);
    let mut candidates = Vec::new();
    for (terminal_id, adjacent) in &neighbors {
        if adjacent.len() != 1 {
            continue;
        }
        let anchor_id = &adjacent[0];
        if neighbors[anchor_id].len() == 1 && terminal_id < anchor_id {
            continue;
        }
        let terminal = graph.coordinates()[terminal_id];
        let anchor = graph.coordinates()[anchor_id];
        let dx = terminal.x() - anchor.x();
        let dy = terminal.y() - anchor.y();
        let length = dx.hypot(dy);
        if length == 0.0 {
            continue;
        }
        let increment = std::f64::consts::PI / 6.0;
        let represented_slot = (dy.atan2(dx) / increment + 0.5).floor() as i32;
        let (unit_x, unit_y) = canonical_thirty_degree_direction(represented_slot);
        let replacement = Point2::new(anchor.x() + length * unit_x, anchor.y() + length * unit_y)?;
        candidates.push((terminal_id.clone(), replacement));
    }
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        candidates,
    ))
}

fn normalize_bond_lengths(
    graph: &DepictionGraph,
    spacing: f64,
) -> Result<CoordinatePatch, RepairError> {
    if !spacing.is_finite() {
        return Err(ferrum_geometry::GeometryError::NonFiniteCoordinate.into());
    }
    if spacing <= 0.0 {
        return Err(ferrum_geometry::GeometryError::NonPositiveExtent.into());
    }
    let neighbors = adjacency(graph);
    let ring = ring_atoms(&neighbors);
    let mut candidates = graph.coordinates().clone();
    let mut visited = ring.clone();
    for anchor in &ring {
        for child in &neighbors[anchor] {
            if !visited.contains(child) {
                place_normalized_branch(
                    anchor,
                    child,
                    spacing,
                    &neighbors,
                    graph.coordinates(),
                    &mut candidates,
                    &mut visited,
                )?;
            }
        }
    }
    for start in neighbors.keys() {
        if visited.contains(start) {
            continue;
        }
        let component = unvisited_component(start, &neighbors, &visited);
        let root = component
            .iter()
            .min_by_key(|id| (Reverse(neighbors[*id].len()), (*id).clone()))
            .expect("component contains its starting atom")
            .clone();
        visited.insert(root.clone());
        for child in &neighbors[&root] {
            if !visited.contains(child) {
                place_normalized_branch(
                    &root,
                    child,
                    spacing,
                    &neighbors,
                    graph.coordinates(),
                    &mut candidates,
                    &mut visited,
                )?;
            }
        }
    }
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        candidates,
    ))
}

fn normalize_single_ring(
    graph: &DepictionGraph,
    spacing: f64,
) -> Result<CoordinatePatch, RepairError> {
    if !spacing.is_finite() {
        return Err(ferrum_geometry::GeometryError::NonFiniteCoordinate.into());
    }
    if spacing <= 0.0 {
        return Err(ferrum_geometry::GeometryError::NonPositiveExtent.into());
    }
    let neighbors = adjacency(graph);
    match independent_cycle_rank(graph) {
        0 => return Ok(CoordinatePatch::default()),
        1 => {}
        _ => {
            return Err(RepairError::UnsupportedTopology(
                "single-ring normalization supports exactly one independent cycle",
            ));
        }
    }
    let ring = ring_atoms(&neighbors);
    let walk = canonical_ring_walk(&ring, &neighbors)?;
    let components = ring_substituent_components(&ring, &neighbors)?;
    let count = walk.len() as f64;
    let center_x = walk
        .iter()
        .map(|id| graph.coordinates()[id].x())
        .sum::<f64>()
        / count;
    let center_y = walk
        .iter()
        .map(|id| graph.coordinates()[id].y())
        .sum::<f64>()
        / count;
    let first = graph.coordinates()[&walk[0]];
    let start_dx = first.x() - center_x;
    let start_dy = first.y() - center_y;
    let start_angle = if start_dx.hypot(start_dy) > 1.0e-9 {
        start_dy.atan2(start_dx)
    } else {
        0.0
    };
    let radius = spacing / (2.0 * (std::f64::consts::PI / count).sin());
    if !radius.is_finite() {
        return Err(ferrum_geometry::GeometryError::UnrepresentableGeometry.into());
    }
    let mut candidates = graph.coordinates().clone();
    for (index, atom_id) in walk.iter().enumerate() {
        // Ferrum's geometry frame is y-up, while the durable ring walk is
        // defined in authored CDML's y-down frame. Advancing the authored walk
        // therefore decreases the Cartesian angle here.
        let angle = start_angle - std::f64::consts::TAU * index as f64 / count;
        candidates.insert(
            atom_id.clone(),
            Point2::new(
                center_x + radius * angle.cos(),
                center_y + radius * angle.sin(),
            )?,
        );
    }
    for (component, anchor) in components {
        let old_anchor = graph.coordinates()[&anchor];
        let new_anchor = candidates[&anchor];
        let shift_x = new_anchor.x() - old_anchor.x();
        let shift_y = new_anchor.y() - old_anchor.y();
        for atom_id in component {
            let original = graph.coordinates()[&atom_id];
            candidates.insert(
                atom_id,
                Point2::new(original.x() + shift_x, original.y() + shift_y)?,
            );
        }
    }
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        candidates,
    ))
}

fn normalize_bond_angles(
    graph: &DepictionGraph,
    spacing: f64,
) -> Result<CoordinatePatch, RepairError> {
    if !spacing.is_finite() {
        return Err(ferrum_geometry::GeometryError::NonFiniteCoordinate.into());
    }
    if spacing <= 0.0 {
        return Err(ferrum_geometry::GeometryError::NonPositiveExtent.into());
    }
    let neighbors = source_adjacency(graph);
    let ring = ring_atoms(&neighbors);
    let components = non_ring_components(graph, &ring, &neighbors)?;
    let mut candidates = graph.coordinates().clone();
    for (component, anchor) in components {
        let component_set = component.iter().cloned().collect::<BTreeSet<_>>();
        let (root, incoming) = if let Some(anchor) = anchor {
            let root = neighbors[&anchor]
                .iter()
                .find(|neighbor| component_set.contains(*neighbor))
                .expect("validated anchor has one component neighbor")
                .clone();
            (root, Some(anchor))
        } else {
            let root = component
                .iter()
                .min_by_key(|id| Reverse(neighbors[*id].len()))
                .expect("non-ring component is nonempty")
                .clone();
            (root, None)
        };
        let mut visited = BTreeSet::from([root.clone()]);
        let mut queue = VecDeque::from([(root, incoming)]);
        while let Some((parent, incoming_parent)) = queue.pop_front() {
            let mut used_slots = BTreeSet::new();
            if let Some(incoming_parent) = &incoming_parent {
                used_slots.insert(authored_sixty_degree_slot(
                    candidates[incoming_parent],
                    candidates[&parent],
                ));
            }
            for neighbor in &neighbors[&parent] {
                if ring.contains(neighbor) {
                    used_slots.insert(authored_sixty_degree_slot(
                        candidates[neighbor],
                        candidates[&parent],
                    ));
                }
            }
            let children = neighbors[&parent]
                .iter()
                .filter(|neighbor| {
                    component_set.contains(*neighbor) && !visited.contains(*neighbor)
                })
                .cloned()
                .collect::<Vec<_>>();
            for child in children {
                let parent_point = candidates[&parent];
                let child_point = candidates[&child];
                let mut slot = authored_sixty_degree_slot(child_point, parent_point);
                let mut found = false;
                for _ in 0..6 {
                    if !used_slots.contains(&slot) {
                        found = true;
                        break;
                    }
                    slot = (slot + 1) % 6;
                }
                if !found {
                    return Err(RepairError::UnsupportedTopology(
                        "bond-angle normalization has no free 60-degree slot",
                    ));
                }
                used_slots.insert(slot);
                let source_distance = parent_point.distance_to(child_point);
                let distance = if source_distance == 0.0 {
                    spacing
                } else {
                    source_distance
                };
                let (unit_x, unit_y) = geometry_direction_for_authored_sixty_slot(slot);
                let replacement = Point2::new(
                    parent_point.x() + distance * unit_x,
                    parent_point.y() + distance * unit_y,
                )?;
                let shift_x = replacement.x() - child_point.x();
                let shift_y = replacement.y() - child_point.y();
                for atom_id in movable_angle_subtree(&child, &parent, &visited, &ring, &neighbors) {
                    let current = candidates[&atom_id];
                    candidates.insert(
                        atom_id,
                        Point2::new(current.x() + shift_x, current.y() + shift_y)?,
                    );
                }
                visited.insert(child.clone());
                queue.push_back((child, Some(parent.clone())));
            }
        }
    }
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        candidates,
    ))
}

fn non_ring_components(
    graph: &DepictionGraph,
    ring: &BTreeSet<RecordId>,
    neighbors: &BTreeMap<RecordId, Vec<RecordId>>,
) -> Result<Vec<AngleComponent>, RepairError> {
    let mut components = Vec::new();
    let mut visited = ring.clone();
    for start in graph.source_atom_order() {
        if visited.contains(start) {
            continue;
        }
        let mut member_set = BTreeSet::from([start.clone()]);
        let mut anchors = BTreeSet::new();
        let mut queue = VecDeque::from([start.clone()]);
        visited.insert(start.clone());
        while let Some(atom) = queue.pop_front() {
            for neighbor in &neighbors[&atom] {
                if ring.contains(neighbor) {
                    anchors.insert(neighbor.clone());
                } else if visited.insert(neighbor.clone()) {
                    member_set.insert(neighbor.clone());
                    queue.push_back(neighbor.clone());
                }
            }
        }
        if anchors.len() > 1 {
            return Err(RepairError::UnsupportedTopology(
                "bond-angle normalization does not support a non-ring component attached to multiple ring anchors",
            ));
        }
        let members = graph
            .source_atom_order()
            .iter()
            .filter(|id| member_set.contains(*id))
            .cloned()
            .collect();
        components.push((members, anchors.into_iter().next()));
    }
    Ok(components)
}

fn movable_angle_subtree(
    root: &RecordId,
    excluded_parent: &RecordId,
    already_visited: &BTreeSet<RecordId>,
    ring: &BTreeSet<RecordId>,
    neighbors: &BTreeMap<RecordId, Vec<RecordId>>,
) -> BTreeSet<RecordId> {
    let mut subtree = BTreeSet::from([root.clone()]);
    let mut seen = BTreeSet::from([root.clone(), excluded_parent.clone()]);
    let mut queue = VecDeque::from([root.clone()]);
    while let Some(atom) = queue.pop_front() {
        for neighbor in &neighbors[&atom] {
            if seen.contains(neighbor)
                || already_visited.contains(neighbor)
                || ring.contains(neighbor)
            {
                continue;
            }
            seen.insert(neighbor.clone());
            subtree.insert(neighbor.clone());
            queue.push_back(neighbor.clone());
        }
    }
    subtree
}

fn authored_sixty_degree_slot(point: Point2, origin: Point2) -> i32 {
    let authored_angle = (origin.y() - point.y()).atan2(point.x() - origin.x());
    let step = std::f64::consts::PI / 3.0;
    let scaled = authored_angle.rem_euclid(std::f64::consts::TAU) / step;
    let lower = scaled.floor();
    let rounded = if scaled - lower >= 0.5 {
        lower + 1.0
    } else {
        lower
    };
    rounded as i32 % 6
}

fn geometry_direction_for_authored_sixty_slot(slot: i32) -> (f64, f64) {
    let diagonal = 3.0_f64.sqrt() / 2.0;
    match slot.rem_euclid(6) {
        0 => (1.0, 0.0),
        1 => (0.5, -diagonal),
        2 => (-0.5, -diagonal),
        3 => (-1.0, 0.0),
        4 => (-0.5, diagonal),
        5 => (0.5, diagonal),
        _ => unreachable!("Euclidean remainder is one of six directions"),
    }
}

fn canonical_ring_walk(
    ring: &BTreeSet<RecordId>,
    neighbors: &BTreeMap<RecordId, Vec<RecordId>>,
) -> Result<Vec<RecordId>, RepairError> {
    if ring.len() < 3 {
        return Err(RepairError::UnsupportedTopology(
            "ring normalization requires a simple cycle with at least three atoms",
        ));
    }
    if ring.iter().any(|id| {
        neighbors[id]
            .iter()
            .filter(|neighbor| ring.contains(*neighbor))
            .count()
            != 2
    }) {
        return Err(RepairError::UnsupportedTopology(
            "ring normalization requires a simple unbranched ring",
        ));
    }
    let start = ring.iter().next().expect("nonempty ring").clone();
    let first = neighbors[&start]
        .iter()
        .filter(|neighbor| ring.contains(*neighbor))
        .min()
        .expect("simple ring start has two ring neighbors")
        .clone();
    let mut walk = vec![start.clone(), first];
    while walk.len() < ring.len() {
        let previous = &walk[walk.len() - 2];
        let current = &walk[walk.len() - 1];
        let next = neighbors[current]
            .iter()
            .find(|candidate| ring.contains(*candidate) && *candidate != previous)
            .expect("simple ring atom has one forward neighbor")
            .clone();
        if walk.contains(&next) {
            return Err(RepairError::UnsupportedTopology(
                "ring normalization could not form one unambiguous ring walk",
            ));
        }
        walk.push(next);
    }
    if !neighbors[walk.last().expect("walk is nonempty")].contains(&start) {
        return Err(RepairError::UnsupportedTopology(
            "ring normalization could not close the ring walk",
        ));
    }
    Ok(walk)
}

fn ring_substituent_components(
    ring: &BTreeSet<RecordId>,
    neighbors: &BTreeMap<RecordId, Vec<RecordId>>,
) -> Result<Vec<(BTreeSet<RecordId>, RecordId)>, RepairError> {
    let mut components = Vec::new();
    let mut visited = ring.clone();
    for start in neighbors.keys() {
        if visited.contains(start) {
            continue;
        }
        let mut component = BTreeSet::from([start.clone()]);
        let mut anchors = BTreeSet::new();
        let mut queue = VecDeque::from([start.clone()]);
        visited.insert(start.clone());
        while let Some(atom) = queue.pop_front() {
            for neighbor in &neighbors[&atom] {
                if ring.contains(neighbor) {
                    anchors.insert(neighbor.clone());
                } else if visited.insert(neighbor.clone()) {
                    component.insert(neighbor.clone());
                    queue.push_back(neighbor.clone());
                }
            }
        }
        if anchors.len() != 1 {
            return Err(RepairError::UnsupportedTopology(
                "ring normalization requires every non-ring component to have one ring anchor",
            ));
        }
        components.push((
            component,
            anchors.into_iter().next().expect("exactly one anchor"),
        ));
    }
    Ok(components)
}

fn place_normalized_branch(
    anchor: &RecordId,
    first: &RecordId,
    spacing: f64,
    neighbors: &BTreeMap<RecordId, Vec<RecordId>>,
    original: &BTreeMap<RecordId, Point2>,
    candidates: &mut BTreeMap<RecordId, Point2>,
    visited: &mut BTreeSet<RecordId>,
) -> Result<(), RepairError> {
    let mut queue = VecDeque::from([(anchor.clone(), first.clone())]);
    while let Some((parent, child)) = queue.pop_front() {
        if !visited.insert(child.clone()) {
            continue;
        }
        let source_vector = original[&child] - original[&parent];
        let length = source_vector.length();
        let (unit_x, unit_y) = if length == 0.0 {
            (1.0, 0.0)
        } else {
            (source_vector.x() / length, source_vector.y() / length)
        };
        let parent_position = candidates[&parent];
        let child_position = Point2::new(
            parent_position.x() + spacing * unit_x,
            parent_position.y() + spacing * unit_y,
        )?;
        candidates.insert(child.clone(), child_position);
        for neighbor in &neighbors[&child] {
            if !visited.contains(neighbor) {
                queue.push_back((child.clone(), neighbor.clone()));
            }
        }
    }
    Ok(())
}

fn adjacency(graph: &DepictionGraph) -> BTreeMap<RecordId, Vec<RecordId>> {
    let mut neighbors = graph
        .coordinates()
        .keys()
        .cloned()
        .map(|id| (id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for (start, end) in graph.edges().values() {
        neighbors
            .get_mut(start)
            .expect("validated graph contains every bond endpoint")
            .push(end.clone());
        neighbors
            .get_mut(end)
            .expect("validated graph contains every bond endpoint")
            .push(start.clone());
    }
    for adjacent in neighbors.values_mut() {
        adjacent.sort();
    }
    neighbors
}

fn source_adjacency(graph: &DepictionGraph) -> BTreeMap<RecordId, Vec<RecordId>> {
    let mut neighbors = graph
        .coordinates()
        .keys()
        .cloned()
        .map(|id| (id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for (start, end) in graph.source_edges() {
        neighbors
            .get_mut(start)
            .expect("validated graph contains every bond endpoint")
            .push(end.clone());
        neighbors
            .get_mut(end)
            .expect("validated graph contains every bond endpoint")
            .push(start.clone());
    }
    neighbors
}

fn ring_atoms(neighbors: &BTreeMap<RecordId, Vec<RecordId>>) -> BTreeSet<RecordId> {
    let mut degrees = neighbors
        .iter()
        .map(|(id, adjacent)| (id.clone(), adjacent.len()))
        .collect::<BTreeMap<_, _>>();
    let mut leaves = degrees
        .iter()
        .filter(|(_, degree)| **degree < 2)
        .map(|(id, _)| id.clone())
        .collect::<VecDeque<_>>();
    while let Some(removed) = leaves.pop_front() {
        if degrees[&removed] == 0 {
            continue;
        }
        degrees.insert(removed.clone(), 0);
        for neighbor in &neighbors[&removed] {
            let degree = degrees
                .get_mut(neighbor)
                .expect("validated adjacency contains every neighbor");
            if *degree > 0 {
                *degree -= 1;
                if *degree == 1 {
                    leaves.push_back(neighbor.clone());
                }
            }
        }
    }
    degrees
        .into_iter()
        .filter_map(|(id, degree)| (degree >= 2).then_some(id))
        .collect()
}

fn unvisited_component(
    start: &RecordId,
    neighbors: &BTreeMap<RecordId, Vec<RecordId>>,
    visited: &BTreeSet<RecordId>,
) -> BTreeSet<RecordId> {
    let mut component = BTreeSet::from([start.clone()]);
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(atom) = queue.pop_front() {
        for neighbor in &neighbors[&atom] {
            if !visited.contains(neighbor) && component.insert(neighbor.clone()) {
                queue.push_back(neighbor.clone());
            }
        }
    }
    component
}

fn canonical_thirty_degree_direction(slot: i32) -> (f64, f64) {
    let diagonal = 3.0_f64.sqrt() / 2.0;
    match slot.rem_euclid(12) {
        0 => (1.0, 0.0),
        1 => (diagonal, 0.5),
        2 => (0.5, diagonal),
        3 => (0.0, 1.0),
        4 => (-0.5, diagonal),
        5 => (-diagonal, 0.5),
        6 => (-1.0, 0.0),
        7 => (-diagonal, -0.5),
        8 => (-0.5, -diagonal),
        9 => (0.0, -1.0),
        10 => (0.5, -diagonal),
        11 => (diagonal, -0.5),
        _ => unreachable!("Euclidean remainder is one of twelve directions"),
    }
}

fn snap_to_hex(
    graph: &DepictionGraph,
    spacing: f64,
    origin: Point2,
) -> Result<CoordinatePatch, RepairError> {
    let grid = HexGrid::new(spacing, origin)?;
    let candidates = graph
        .coordinates()
        .iter()
        .map(|(id, point)| grid.snap(*point).map(|snapped| (id.clone(), snapped)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CoordinatePatch::from_candidates(
        graph.coordinates(),
        candidates,
    ))
}

fn straighten(
    graph: &DepictionGraph,
    minimize_rotation: bool,
) -> Result<RepairOutcome, RepairError> {
    let ids: Vec<_> = graph.coordinates().keys().cloned().collect();
    let coordinates: Vec<_> = graph.coordinates().values().copied().collect();
    let indices: std::collections::BTreeMap<_, _> = ids
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, id)| (id, index))
        .collect();
    let bonds = graph
        .edges()
        .values()
        .map(|(start, end)| (indices[start], indices[end]))
        .collect::<Vec<_>>();
    let straightened = straighten_depiction(&coordinates, &bonds, minimize_rotation)?;
    let coordinates = ids.into_iter().zip(straightened.coordinates).collect();
    Ok(RepairOutcome::from_straightening(
        graph.coordinates(),
        coordinates,
        straightened.rotation_radians,
    ))
}

fn independent_cycle_rank(graph: &DepictionGraph) -> usize {
    let indices: BTreeMap<_, _> = graph
        .coordinates()
        .keys()
        .enumerate()
        .map(|(index, id)| (id.clone(), index))
        .collect();
    let mut components = DisjointSet::new(indices.len());
    graph
        .edges()
        .values()
        .filter(|(start, end)| !components.join(indices[start], indices[end]))
        .count()
}

struct DisjointSet {
    parents: Vec<usize>,
    ranks: Vec<u8>,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parents: (0..size).collect(),
            ranks: vec![0; size],
        }
    }

    fn find(&mut self, index: usize) -> usize {
        if self.parents[index] != index {
            let root = self.find(self.parents[index]);
            self.parents[index] = root;
        }
        self.parents[index]
    }

    fn join(&mut self, left: usize, right: usize) -> bool {
        let left_root = self.find(left);
        let right_root = self.find(right);
        if left_root == right_root {
            return false;
        }
        match self.ranks[left_root].cmp(&self.ranks[right_root]) {
            std::cmp::Ordering::Less => self.parents[left_root] = right_root,
            std::cmp::Ordering::Greater => self.parents[right_root] = left_root,
            std::cmp::Ordering::Equal => {
                self.parents[right_root] = left_root;
                self.ranks[left_root] += 1;
            }
        }
        true
    }
}
