use std::collections::BTreeSet;

use super::*;
use crate::{Atom, Bond, Identifier, Molecule, NonAtomVertex, Position, RecordKind, RecordOrigin};

fn source(value: &str) -> Identifier {
    Identifier::new(value).expect("test source ID is nonblank")
}

fn atom(index: usize) -> Atom {
    Atom::new(
        Some(source(&format!("a{index}"))),
        Some("C".to_owned()),
        Position::new(index as f64, 0.0, 0.0).expect("test position is finite"),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .expect("test atom is valid")
}

fn vertex(atom: &Atom) -> VertexRef {
    VertexRef::Atom(atom.identity().clone())
}

fn molecule(vertex_count: usize, edges: &[(usize, usize)]) -> Molecule {
    let atoms = (0..vertex_count).map(atom).collect::<Vec<_>>();
    let bonds = edges
        .iter()
        .enumerate()
        .map(|(index, &(start, end))| {
            Bond::new(
                Some(source(&format!("b{index}"))),
                vertex(&atoms[start]),
                vertex(&atoms[end]),
                None,
                None,
                None,
                None,
                None,
            )
            .expect("test bond is valid")
        })
        .collect();
    Molecule::new(
        Some(source("m1")),
        None,
        atoms,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        bonds,
        None,
    )
    .expect("test molecule is valid")
}

fn vertex_source_id(vertex: &VertexRef) -> &str {
    let identity = match vertex {
        VertexRef::Atom(identity)
        | VertexRef::Group(identity)
        | VertexRef::Text(identity)
        | VertexRef::Query(identity) => identity,
    };
    record_source_id(identity)
}

fn record_source_id(identity: &RecordId) -> &str {
    match identity.origin() {
        RecordOrigin::Source(identifier) => identifier.as_str(),
        RecordOrigin::Legacy { .. } => panic!("test records are source-backed"),
    }
}

fn vertex_ids(vertices: &[VertexRef]) -> Vec<&str> {
    vertices.iter().map(vertex_source_id).collect()
}

fn bond_ids(bonds: &[RecordId]) -> Vec<&str> {
    bonds.iter().map(record_source_id).collect()
}

#[test]
fn graph_view_keeps_petgraph_indexes_private_and_all_vertex_kinds_connected() {
    let atoms = vec![atom(0)];
    let group =
        NonAtomVertex::new(RecordKind::Group, Some(source("g1")), None).expect("group is valid");
    let query =
        NonAtomVertex::new(RecordKind::Query, Some(source("q1")), None).expect("query is valid");
    let bonds = vec![
        Bond::new(
            Some(source("b0")),
            vertex(&atoms[0]),
            VertexRef::Group(group.identity().clone()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("atom-group bond is valid"),
        Bond::new(
            Some(source("b1")),
            VertexRef::Group(group.identity().clone()),
            VertexRef::Query(query.identity().clone()),
            None,
            None,
            None,
            None,
            None,
        )
        .expect("group-query bond is valid"),
    ];
    let molecule = Molecule::new(
        Some(source("mixed")),
        None,
        atoms,
        vec![group],
        Vec::new(),
        vec![query],
        bonds,
        None,
    )
    .expect("mixed molecule is valid");
    let graph = molecule.graph();

    assert_eq!(graph.vertex_count(), 3);
    assert_eq!(graph.bond_count(), 2);
    assert_eq!(
        vertex_ids(graph.connected_components()[0].vertices()),
        ["a0", "g1", "q1"]
    );
}

#[test]
fn components_and_connectivity_have_stable_source_order() {
    let molecule = molecule(5, &[(0, 2), (2, 4), (1, 3)]);
    let graph = molecule.graph();
    let components = graph.connected_components();

    assert!(!graph.is_connected());
    assert_eq!(components.len(), 2);
    assert_eq!(vertex_ids(components[0].vertices()), ["a0", "a2", "a4"]);
    assert_eq!(vertex_ids(components[1].vertices()), ["a1", "a3"]);
    assert!(
        graph
            .has_path(&vertex(&molecule.atoms()[0]), &vertex(&molecule.atoms()[4]))
            .expect("both vertices exist")
    );
    assert!(
        !graph
            .has_path(&vertex(&molecule.atoms()[0]), &vertex(&molecule.atoms()[1]))
            .expect("both vertices exist")
    );
}

#[test]
fn unknown_vertex_queries_return_an_owned_typed_error() {
    let molecule = molecule(2, &[(0, 1)]);
    let graph = molecule.graph();
    let missing = vertex(&atom(99));
    let error = graph
        .has_path(&vertex(&molecule.atoms()[0]), &missing)
        .expect_err("foreign vertex is rejected");

    assert_eq!(error.vertex, missing);
}

#[test]
fn bridges_and_articulation_points_return_ferrum_identities_in_stable_order() {
    let molecule = molecule(5, &[(0, 1), (1, 2), (2, 0), (2, 3), (3, 4)]);
    let graph = molecule.graph();

    assert_eq!(bond_ids(&graph.bridges()), ["b3", "b4"]);
    assert_eq!(vertex_ids(&graph.articulation_points()), ["a2", "a3"]);
}

#[test]
fn dijkstra_distances_and_tie_broken_shortest_path_are_deterministic() {
    let molecule = molecule(5, &[(0, 1), (0, 2), (1, 3), (2, 3)]);
    let graph = molecule.graph();
    let source = vertex(&molecule.atoms()[0]);
    let target = vertex(&molecule.atoms()[3]);
    let distances = graph.distances_from(&source).expect("source vertex exists");

    assert_eq!(
        distances
            .iter()
            .map(|entry| (vertex_source_id(entry.vertex()), entry.distance()))
            .collect::<Vec<_>>(),
        [("a0", 0), ("a1", 1), ("a2", 1), ("a3", 2)]
    );
    assert_eq!(
        vertex_ids(
            &graph
                .shortest_path(&source, &target)
                .expect("query vertices exist")
                .expect("target is reachable")
        ),
        ["a0", "a1", "a3"]
    );
}

#[test]
fn floyd_warshall_marks_disconnected_pairs_without_magic_distances() {
    let molecule = molecule(4, &[(0, 1), (2, 3)]);
    let graph = molecule.graph();
    let distances = graph.all_pairs_distances();

    assert_eq!(
        distances
            .distance(&vertex(&molecule.atoms()[0]), &vertex(&molecule.atoms()[1]))
            .expect("vertices exist"),
        Some(1)
    );
    assert_eq!(
        distances
            .distance(&vertex(&molecule.atoms()[0]), &vertex(&molecule.atoms()[2]))
            .expect("vertices exist"),
        None
    );
    assert_eq!(graph.diameter(), 1);
}

#[test]
fn maximum_matching_is_maximal_in_cardinality_and_repeatable() {
    let molecule = molecule(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    let graph = molecule.graph();
    let expected = graph.maximum_matching();

    assert_eq!(expected.len(), 2);
    let matched = expected
        .iter()
        .flat_map(|pair| {
            [
                vertex_source_id(pair.first()),
                vertex_source_id(pair.second()),
            ]
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(matched.len(), 4);
    for _ in 0..100 {
        assert_eq!(graph.maximum_matching(), expected);
    }
}

#[test]
fn shortest_stable_bfs_forest_defines_exact_fundamental_cycles() {
    let molecule = molecule(4, &[(0, 1), (1, 2), (2, 3), (3, 0), (0, 2)]);
    let graph = molecule.graph();
    let basis = graph.cycle_basis();

    assert_eq!(graph.cycle_rank(), 2);
    assert_eq!(basis.len(), graph.cycle_rank());
    assert_eq!(vertex_ids(basis[0].vertices()), ["a0", "a1", "a2"]);
    assert_eq!(bond_ids(basis[0].bonds()), ["b0", "b1", "b4"]);
    assert_eq!(vertex_ids(basis[1].vertices()), ["a0", "a2", "a3"]);
    assert_eq!(bond_ids(basis[1].bonds()), ["b4", "b2", "b3"]);
    for _ in 0..100 {
        assert_eq!(graph.cycle_basis(), basis);
    }
}

#[test]
fn disconnected_and_parallel_edge_cycles_have_the_correct_cycle_rank() {
    let disconnected = molecule(6, &[(0, 1), (1, 2), (2, 0), (3, 4), (4, 5), (5, 3)]);
    let disconnected_graph = disconnected.graph();
    assert_eq!(disconnected_graph.cycle_rank(), 2);
    assert_eq!(disconnected_graph.cycle_basis().len(), 2);

    let parallel = molecule(2, &[(0, 1), (0, 1)]);
    let parallel_basis = parallel.graph().cycle_basis();
    assert_eq!(parallel_basis.len(), 1);
    assert_eq!(vertex_ids(parallel_basis[0].vertices()), ["a0", "a1"]);
    assert_eq!(bond_ids(parallel_basis[0].bonds()), ["b0", "b1"]);
}

#[test]
fn empty_and_single_vertex_graphs_use_mathematical_boundary_values() {
    let empty = molecule(0, &[]);
    let empty_graph = empty.graph();
    assert!(empty_graph.is_connected());
    assert_eq!(empty_graph.cycle_rank(), 0);
    assert_eq!(empty_graph.diameter(), 0);

    let single = molecule(1, &[]);
    let single_graph = single.graph();
    assert!(single_graph.is_connected());
    assert_eq!(single_graph.connected_components().len(), 1);
    assert_eq!(single_graph.cycle_rank(), 0);
    assert_eq!(single_graph.diameter(), 0);
}

#[derive(Clone, Copy)]
enum CycleParity {
    Match(&'static [usize]),
    IntendedDifference {
        reference: &'static [usize],
        ferrum: &'static [usize],
    },
}

impl CycleParity {
    fn ferrum(&self) -> &'static [usize] {
        match self {
            Self::Match(sizes) => sizes,
            Self::IntendedDifference { ferrum, .. } => ferrum,
        }
    }
}

struct ReferenceFixture {
    name: &'static str,
    vertices: usize,
    edges: &'static [(usize, usize)],
    component_sizes: &'static [usize],
    diameter: usize,
    bridges: usize,
    articulation_points: usize,
    maximum_matching: usize,
    cycle_parity: CycleParity,
}

const REFERENCE_FIXTURES: &[ReferenceFixture] = &[
    ReferenceFixture {
        name: "benzene",
        vertices: 6,
        edges: &[(5, 0), (2, 3), (1, 2), (0, 1), (3, 4), (4, 5)],
        component_sizes: &[6],
        diameter: 3,
        bridges: 0,
        articulation_points: 0,
        maximum_matching: 3,
        cycle_parity: CycleParity::Match(&[6]),
    },
    ReferenceFixture {
        name: "cholesterol",
        vertices: 28,
        edges: &[
            (20, 26),
            (6, 8),
            (12, 27),
            (8, 9),
            (12, 8),
            (9, 10),
            (16, 11),
            (10, 11),
            (20, 15),
            (12, 13),
            (11, 12),
            (13, 14),
            (14, 15),
            (18, 19),
            (24, 19),
            (15, 16),
            (16, 17),
            (17, 18),
            (0, 1),
            (19, 20),
            (1, 2),
            (20, 21),
            (1, 3),
            (21, 22),
            (3, 4),
            (22, 23),
            (4, 5),
            (23, 24),
            (5, 6),
            (23, 25),
            (6, 7),
        ],
        component_sizes: &[28],
        diameter: 15,
        bridges: 11,
        articulation_points: 9,
        maximum_matching: 13,
        cycle_parity: CycleParity::Match(&[5, 6, 6, 6]),
    },
    ReferenceFixture {
        name: "naphthalene",
        vertices: 10,
        edges: &[
            (5, 6),
            (9, 0),
            (2, 3),
            (6, 7),
            (8, 3),
            (3, 4),
            (7, 8),
            (0, 1),
            (4, 5),
            (8, 9),
            (1, 2),
        ],
        component_sizes: &[10],
        diameter: 5,
        bridges: 0,
        articulation_points: 0,
        maximum_matching: 5,
        cycle_parity: CycleParity::Match(&[6, 6]),
    },
    ReferenceFixture {
        name: "steroid skeleton",
        vertices: 17,
        edges: &[
            (9, 10),
            (10, 11),
            (11, 12),
            (12, 13),
            (14, 15),
            (13, 14),
            (15, 16),
            (5, 0),
            (0, 1),
            (9, 3),
            (1, 2),
            (16, 8),
            (2, 3),
            (16, 12),
            (3, 4),
            (4, 5),
            (4, 6),
            (6, 7),
            (7, 8),
            (8, 9),
        ],
        component_sizes: &[17],
        diameter: 8,
        bridges: 0,
        articulation_points: 0,
        maximum_matching: 8,
        cycle_parity: CycleParity::Match(&[5, 6, 6, 6]),
    },
    ReferenceFixture {
        name: "caffeine",
        vertices: 14,
        edges: &[
            (2, 3),
            (8, 9),
            (3, 4),
            (9, 11),
            (5, 1),
            (9, 10),
            (4, 5),
            (11, 4),
            (5, 6),
            (1, 2),
            (0, 1),
            (6, 7),
            (11, 12),
            (6, 8),
            (8, 13),
        ],
        component_sizes: &[14],
        diameter: 6,
        bridges: 5,
        articulation_points: 5,
        maximum_matching: 7,
        cycle_parity: CycleParity::Match(&[5, 6]),
    },
    ReferenceFixture {
        name: "hexane",
        vertices: 6,
        edges: &[(1, 2), (3, 4), (4, 5), (2, 3), (0, 1)],
        component_sizes: &[6],
        diameter: 5,
        bridges: 5,
        articulation_points: 4,
        maximum_matching: 3,
        cycle_parity: CycleParity::Match(&[]),
    },
    ReferenceFixture {
        name: "single atom",
        vertices: 1,
        edges: &[],
        component_sizes: &[1],
        diameter: 0,
        bridges: 0,
        articulation_points: 0,
        maximum_matching: 0,
        cycle_parity: CycleParity::Match(&[]),
    },
    ReferenceFixture {
        name: "disconnected",
        vertices: 4,
        edges: &[(0, 1), (2, 3)],
        component_sizes: &[2, 2],
        diameter: 1,
        bridges: 2,
        articulation_points: 0,
        maximum_matching: 2,
        cycle_parity: CycleParity::Match(&[]),
    },
    ReferenceFixture {
        name: "cyclopentane",
        vertices: 5,
        edges: &[(4, 0), (2, 3), (0, 1), (1, 2), (3, 4)],
        component_sizes: &[5],
        diameter: 2,
        bridges: 0,
        articulation_points: 0,
        maximum_matching: 2,
        cycle_parity: CycleParity::Match(&[5]),
    },
    ReferenceFixture {
        name: "bridged bicyclic",
        vertices: 7,
        edges: &[
            (1, 2),
            (4, 5),
            (0, 1),
            (5, 6),
            (5, 0),
            (2, 3),
            (6, 2),
            (3, 4),
        ],
        component_sizes: &[7],
        diameter: 3,
        bridges: 0,
        articulation_points: 0,
        maximum_matching: 3,
        cycle_parity: CycleParity::IntendedDifference {
            reference: &[5, 6],
            ferrum: &[5, 5],
        },
    },
];

#[test]
fn fixed_reference_topologies_match_every_discrete_graph_result() {
    for fixture in REFERENCE_FIXTURES {
        let molecule = molecule(fixture.vertices, fixture.edges);
        let graph = molecule.graph();
        let component_sizes = graph
            .connected_components()
            .iter()
            .map(|component| component.vertices().len())
            .collect::<Vec<_>>();
        let basis = graph.cycle_basis();
        let mut cycle_sizes = basis
            .iter()
            .map(|cycle| cycle.vertices().len())
            .collect::<Vec<_>>();
        cycle_sizes.sort_unstable();
        let matching = graph.maximum_matching();
        for _ in 0..100 {
            assert_eq!(graph.cycle_basis(), basis, "{}", fixture.name);
            assert_eq!(graph.maximum_matching(), matching, "{}", fixture.name);
        }

        assert_eq!(graph.vertex_count(), fixture.vertices, "{}", fixture.name);
        assert_eq!(graph.bond_count(), fixture.edges.len(), "{}", fixture.name);
        assert_eq!(component_sizes, fixture.component_sizes, "{}", fixture.name);
        assert_eq!(
            graph.is_connected(),
            fixture.component_sizes.len() <= 1,
            "{}",
            fixture.name
        );
        assert_eq!(graph.diameter(), fixture.diameter, "{}", fixture.name);
        assert_eq!(graph.bridges().len(), fixture.bridges, "{}", fixture.name);
        assert_eq!(
            graph.articulation_points().len(),
            fixture.articulation_points,
            "{}",
            fixture.name
        );
        assert_eq!(
            graph.maximum_matching().len(),
            fixture.maximum_matching,
            "{}",
            fixture.name
        );
        assert_eq!(
            graph.cycle_rank(),
            fixture.edges.len() + fixture.component_sizes.len() - fixture.vertices,
            "{}",
            fixture.name
        );
        assert_eq!(
            cycle_sizes,
            fixture.cycle_parity.ferrum(),
            "{}",
            fixture.name
        );
        if let CycleParity::IntendedDifference { reference, ferrum } = fixture.cycle_parity {
            assert_ne!(
                reference, ferrum,
                "classification must describe a difference"
            );
        }
    }
}
