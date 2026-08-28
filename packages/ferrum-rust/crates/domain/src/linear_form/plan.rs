//! Deterministic source-order path and exterior-component planner.

use ferrum_core::{RecordId, RecordKind};
use ferrum_geometry::Point2;

use crate::linear_form::{
    LinearFormMetadataShapeV1, LinearFormPlanErrorV1, LinearFormPlanV1,
    LinearFormPointReplacementV1, LinearFormRequestV1,
};

/// Plan the fixed-point native `linear-form-direction-v1` mutation.
pub fn plan_linear_form_v1(
    request: &LinearFormRequestV1,
) -> Result<LinearFormPlanV1, LinearFormPlanErrorV1> {
    let graph = request.graph();
    validate_graph(graph, request.selected_atoms())?;
    let selected = selected_indices(request)?;
    let induced_bonds = induced_bonds(graph, &selected)?;
    let path = ordered_path(graph, &selected, &induced_bonds)?;
    let selected_replacements = selected_replacements(graph, &path, request.bond_length())?;
    let exterior_replacements =
        exterior_replacements(graph, &selected, &path, &selected_replacements)?;
    build_plan(
        graph,
        &path,
        &induced_bonds,
        selected_replacements,
        exterior_replacements,
        request.bond_length(),
    )
}

fn validate_graph(
    graph: &crate::linear_form::LinearFormGraphV1,
    selected_atoms: &[RecordId],
) -> Result<(), LinearFormPlanErrorV1> {
    if selected_atoms.is_empty() {
        return Err(LinearFormPlanErrorV1::EmptySelection);
    }
    for (index, atom) in graph.atoms().iter().enumerate() {
        if atom.atom_id().kind() != RecordKind::Atom
            || !atom.point().x().is_finite()
            || !atom.point().y().is_finite()
            || graph.atoms()[..index]
                .iter()
                .any(|prior| prior.atom_id() == atom.atom_id())
        {
            return Err(LinearFormPlanErrorV1::UnknownOrForeignAtom);
        }
    }
    for (index, bond) in graph.bonds().iter().enumerate() {
        let duplicate_bond_id = graph.bonds()[..index]
            .iter()
            .any(|prior| prior.bond_id() == bond.bond_id());
        if duplicate_bond_id {
            return Err(LinearFormPlanErrorV1::DuplicateBondId);
        }
        if bond.bond_id().kind() != RecordKind::Bond
            || bond.start().kind() != RecordKind::Atom
            || bond.end().kind() != RecordKind::Atom
            || bond.start() == bond.end()
            || atom_index(graph, bond.start()).is_none()
            || atom_index(graph, bond.end()).is_none()
        {
            return Err(LinearFormPlanErrorV1::UnknownOrForeignAtom);
        }
    }
    Ok(())
}

fn selected_indices(request: &LinearFormRequestV1) -> Result<Vec<usize>, LinearFormPlanErrorV1> {
    let mut selected: Vec<usize> = Vec::new();
    selected
        .try_reserve_exact(request.selected_atoms().len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for atom_id in request.selected_atoms() {
        if atom_id.kind() != RecordKind::Atom {
            return Err(LinearFormPlanErrorV1::UnknownOrForeignAtom);
        }
        if selected
            .iter()
            .any(|index| request.graph().atoms()[*index].atom_id() == atom_id)
        {
            return Err(LinearFormPlanErrorV1::DuplicateAtomId);
        }
        let index = atom_index(request.graph(), atom_id)
            .ok_or(LinearFormPlanErrorV1::UnknownOrForeignAtom)?;
        selected.push(index);
    }
    Ok(selected)
}

fn induced_bonds(
    graph: &crate::linear_form::LinearFormGraphV1,
    selected: &[usize],
) -> Result<Vec<usize>, LinearFormPlanErrorV1> {
    let mut induced = Vec::new();
    induced
        .try_reserve(graph.bonds().len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for (bond_index, bond) in graph.bonds().iter().enumerate() {
        let start =
            atom_index(graph, bond.start()).ok_or(LinearFormPlanErrorV1::UnknownOrForeignAtom)?;
        let end =
            atom_index(graph, bond.end()).ok_or(LinearFormPlanErrorV1::UnknownOrForeignAtom)?;
        if selected.contains(&start) && selected.contains(&end) {
            induced.push(bond_index);
        }
    }
    Ok(induced)
}

fn ordered_path(
    graph: &crate::linear_form::LinearFormGraphV1,
    selected: &[usize],
    induced: &[usize],
) -> Result<Vec<usize>, LinearFormPlanErrorV1> {
    if selected.len() == 1 {
        return if induced.is_empty() {
            let mut path = Vec::new();
            path.try_reserve_exact(1)
                .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
            path.push(selected[0]);
            Ok(path)
        } else {
            Err(LinearFormPlanErrorV1::NotSinglePath)
        };
    }
    let expected = selected
        .len()
        .checked_sub(1)
        .ok_or(LinearFormPlanErrorV1::ResourceExhausted)?;
    if induced.len() != expected {
        return Err(LinearFormPlanErrorV1::NotSinglePath);
    }
    let mut degree = Vec::new();
    degree
        .try_reserve_exact(selected.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    degree.resize(selected.len(), 0_usize);
    for &bond_index in induced {
        let bond = &graph.bonds()[bond_index];
        let start = selected_position(selected, atom_index(graph, bond.start()))?;
        let end = selected_position(selected, atom_index(graph, bond.end()))?;
        degree[start] = degree[start]
            .checked_add(1)
            .ok_or(LinearFormPlanErrorV1::ResourceExhausted)?;
        degree[end] = degree[end]
            .checked_add(1)
            .ok_or(LinearFormPlanErrorV1::ResourceExhausted)?;
    }
    let mut endpoints = Vec::new();
    endpoints
        .try_reserve_exact(2)
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for (index, value) in degree.iter().enumerate() {
        if *value == 1 {
            endpoints.push(index);
        }
    }
    if endpoints.len() != 2 || degree.iter().any(|degree| *degree > 2 || *degree == 0) {
        return Err(LinearFormPlanErrorV1::NotSinglePath);
    }
    let mut path = Vec::new();
    path.try_reserve_exact(selected.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    let first_endpoint = endpoints
        .into_iter()
        .min_by_key(|selected_position| selected[*selected_position])
        .ok_or(LinearFormPlanErrorV1::NotSinglePath)?;
    let mut current = selected[first_endpoint];
    let mut previous = None;
    while path.len() < selected.len() {
        path.push(current);
        let next = induced.iter().find_map(|bond_index| {
            let bond = &graph.bonds()[*bond_index];
            let start = atom_index(graph, bond.start())?;
            let end = atom_index(graph, bond.end())?;
            if start == current && Some(end) != previous {
                Some(end)
            } else if end == current && Some(start) != previous {
                Some(start)
            } else {
                None
            }
        });
        previous = Some(current);
        let Some(next) = next else { break };
        current = next;
    }
    if path.len() != selected.len() {
        return Err(LinearFormPlanErrorV1::NotSinglePath);
    }
    Ok(path)
}

fn selected_replacements(
    graph: &crate::linear_form::LinearFormGraphV1,
    path: &[usize],
    bond_length: crate::linear_form::LinearFormBondLength,
) -> Result<Vec<LinearFormPointReplacementV1>, LinearFormPlanErrorV1> {
    let first = graph.atoms()[path[0]].point();
    let mut replacements = Vec::new();
    replacements
        .try_reserve_exact(path.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for (index, atom_index) in path.iter().enumerate() {
        let offset = u32::try_from(index).map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
        let x = first.x() + bond_length.points() * f64::from(offset);
        let point = Point2::new(x, first.y()).map_err(|_| LinearFormPlanErrorV1::NonFinitePoint)?;
        replacements.push(LinearFormPointReplacementV1::new(
            graph.atoms()[*atom_index].atom_id().clone(),
            point,
        ));
    }
    Ok(replacements)
}

fn exterior_replacements(
    graph: &crate::linear_form::LinearFormGraphV1,
    selected: &[usize],
    path: &[usize],
    selected_replacements: &[LinearFormPointReplacementV1],
) -> Result<Vec<LinearFormPointReplacementV1>, LinearFormPlanErrorV1> {
    let mut visited = Vec::new();
    visited
        .try_reserve_exact(graph.atoms().len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    visited.resize(graph.atoms().len(), false);
    let mut replacements = Vec::new();
    replacements
        .try_reserve(graph.atoms().len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for start in 0..graph.atoms().len() {
        if selected.contains(&start) || visited[start] {
            continue;
        }
        let (component, anchors) = exterior_component(graph, selected, start, &mut visited)?;
        if anchors.len() > 1 {
            return Err(LinearFormPlanErrorV1::ExteriorComponentHasMultipleAnchors);
        }
        let Some(anchor) = anchors.first().copied() else {
            continue;
        };
        let path_position = path
            .iter()
            .position(|index| *index == anchor)
            .ok_or(LinearFormPlanErrorV1::NotSinglePath)?;
        let old_anchor = graph.atoms()[anchor].point();
        let new_anchor = selected_replacements[path_position].point();
        let dx = new_anchor.x() - old_anchor.x();
        let dy = new_anchor.y() - old_anchor.y();
        for atom_index in component {
            let point = graph.atoms()[atom_index].point();
            let translated = Point2::new(point.x() + dx, point.y() + dy)
                .map_err(|_| LinearFormPlanErrorV1::NonFinitePoint)?;
            replacements.push(LinearFormPointReplacementV1::new(
                graph.atoms()[atom_index].atom_id().clone(),
                translated,
            ));
        }
    }
    Ok(replacements)
}

fn exterior_component(
    graph: &crate::linear_form::LinearFormGraphV1,
    selected: &[usize],
    start: usize,
    visited: &mut [bool],
) -> Result<(Vec<usize>, Vec<usize>), LinearFormPlanErrorV1> {
    let mut component = Vec::new();
    component
        .try_reserve(graph.atoms().len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    let mut worklist = Vec::new();
    worklist
        .try_reserve(graph.atoms().len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    let mut anchors = Vec::new();
    anchors
        .try_reserve_exact(selected.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    visited[start] = true;
    worklist.push(start);
    while let Some(current) = worklist.pop() {
        component.push(current);
        for bond in graph.bonds() {
            let start = atom_index(graph, bond.start())
                .ok_or(LinearFormPlanErrorV1::UnknownOrForeignAtom)?;
            let end =
                atom_index(graph, bond.end()).ok_or(LinearFormPlanErrorV1::UnknownOrForeignAtom)?;
            let neighbor = if start == current {
                Some(end)
            } else if end == current {
                Some(start)
            } else {
                None
            };
            let Some(neighbor) = neighbor else { continue };
            if selected.contains(&neighbor) {
                if !anchors.contains(&neighbor) {
                    anchors.push(neighbor);
                }
            } else if !visited[neighbor] {
                visited[neighbor] = true;
                worklist.push(neighbor);
            }
        }
    }
    Ok((component, anchors))
}

fn build_plan(
    graph: &crate::linear_form::LinearFormGraphV1,
    path: &[usize],
    induced: &[usize],
    selected_replacements: Vec<LinearFormPointReplacementV1>,
    exterior_replacements: Vec<LinearFormPointReplacementV1>,
    bond_length: crate::linear_form::LinearFormBondLength,
) -> Result<LinearFormPlanV1, LinearFormPlanErrorV1> {
    let mut ordered_atoms = Vec::new();
    let mut hydrogen_visible_atoms = Vec::new();
    ordered_atoms
        .try_reserve_exact(path.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    hydrogen_visible_atoms
        .try_reserve_exact(path.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for atom_index in path {
        ordered_atoms.push(graph.atoms()[*atom_index].atom_id().clone());
        hydrogen_visible_atoms.push(graph.atoms()[*atom_index].atom_id().clone());
    }
    let mut ordered_bonds = Vec::new();
    ordered_bonds
        .try_reserve_exact(induced.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for pair in path.windows(2) {
        let bond_index = induced
            .iter()
            .copied()
            .find(|bond_index| joins_pair(graph, *bond_index, pair[0], pair[1]))
            .ok_or(LinearFormPlanErrorV1::NotSinglePath)?;
        ordered_bonds.push(graph.bonds()[bond_index].bond_id().clone());
    }
    let metadata =
        LinearFormMetadataShapeV1::new(clone_ids(&ordered_atoms)?, clone_ids(&ordered_bonds)?);
    Ok(LinearFormPlanV1::new(
        ordered_atoms,
        ordered_bonds,
        selected_replacements,
        exterior_replacements,
        hydrogen_visible_atoms,
        metadata,
        bond_length,
    ))
}

fn clone_ids(ids: &[RecordId]) -> Result<Vec<RecordId>, LinearFormPlanErrorV1> {
    let mut clones = Vec::new();
    clones
        .try_reserve_exact(ids.len())
        .map_err(|_| LinearFormPlanErrorV1::ResourceExhausted)?;
    for id in ids {
        clones.push(id.clone());
    }
    Ok(clones)
}

fn atom_index(graph: &crate::linear_form::LinearFormGraphV1, atom_id: &RecordId) -> Option<usize> {
    graph
        .atoms()
        .iter()
        .position(|atom| atom.atom_id() == atom_id)
}

fn joins_pair(
    graph: &crate::linear_form::LinearFormGraphV1,
    bond_index: usize,
    first: usize,
    second: usize,
) -> bool {
    let bond = &graph.bonds()[bond_index];
    let Some(start) = atom_index(graph, bond.start()) else {
        return false;
    };
    let Some(end) = atom_index(graph, bond.end()) else {
        return false;
    };
    (start == first && end == second) || (start == second && end == first)
}

fn selected_position(
    selected: &[usize],
    index: Option<usize>,
) -> Result<usize, LinearFormPlanErrorV1> {
    let index = index.ok_or(LinearFormPlanErrorV1::UnknownOrForeignAtom)?;
    selected
        .iter()
        .position(|selected_index| *selected_index == index)
        .ok_or(LinearFormPlanErrorV1::NotSinglePath)
}
