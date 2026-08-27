use super::*;
use crate::{
    AttachedCompactGroupReleaseV1, DocumentCompactGroupMaterializationRequestV1, SessionOperation,
    SessionOperationOutcomeV1, SessionOperationTransitionRequestV1, SessionOperationV1,
    TransitionAuthorizationV1, TypedDocument, compose_complete_document_render_plan_v1,
    document_molecule_coordinate_graph_v1,
};
use ferrum_render::RenderOp;

const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>";

fn fence(session: &DocumentSession) -> DocumentFenceV1 {
    DocumentFenceV1::new(session.current_revision_v1(), session.current_digest_v1())
}

fn anchor(session: &DocumentSession) -> DocumentObjectIdV1 {
    session
        .document_observation()
        .expect("observation")
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("source molecule")
        .atoms()
        .iter()
        .find(|atom| atom.source_id() == Some("a"))
        .expect("source anchor")
        .document_object_id()
        .clone()
}

fn target(session: &DocumentSession) -> AttachedCompactGroupTargetV1 {
    let molecule_id = session
        .document_observation()
        .expect("observation")
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("source molecule")
        .document_object_id()
        .clone();
    AttachedCompactGroupTargetV1::new(molecule_id, anchor(session))
}

fn attach_phenyl(session: &mut DocumentSession) -> AttachedCompactGroupCommitResultV1 {
    let mut pending = session
        .prepare_attach_compact_group_v1(
            fence(session),
            target(session),
            AttachCompactGroupV1::new(
                CompactGroupCatalogKeyV1::Phenyl,
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
        )
        .expect("Phenyl attach prepares");
    session
        .commit_attach_compact_group_v1(&mut pending)
        .expect("Phenyl attach commits")
}

fn assert_materialized_phenyl(
    session: &DocumentSession,
    focus_atom_id: &DocumentObjectIdV1,
    exterior_bond_id: &DocumentObjectIdV1,
    compact_group_id: &DocumentObjectIdV1,
) {
    let observation = session.document_observation().expect("observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("source molecule");
    assert!(
        !molecule
            .compact_groups()
            .iter()
            .any(|group| group.id() == compact_group_id)
    );
    let exterior = molecule
        .bonds()
        .iter()
        .find(|bond| bond.document_object_id() == exterior_bond_id)
        .expect("retained exterior bond");
    assert_eq!(exterior.start().source_id(), Some("a"));
    assert_eq!(exterior.end().object_id(), Some(focus_atom_id));
    assert_eq!(
        (exterior.order(), exterior.style()),
        (
            Some(ferrum_core::BondOrder::Single),
            Some(&ferrum_core::BondStyle::Normal)
        ),
    );
}

#[test]
fn attached_phenyl_materialization_uses_the_generic_history_safe_cycle_recipe() {
    let mut session = DocumentSession::load(SOURCE).expect("source");
    let before = session.snapshot().expect("before");
    let mut cancelled = session
        .prepare_attach_compact_group_v1(
            fence(&session),
            target(&session),
            AttachCompactGroupV1::new(
                CompactGroupCatalogKeyV1::Phenyl,
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
        )
        .expect("Phenyl cancellation prepare");
    session
        .cancel_attach_compact_group_v1(&mut cancelled)
        .expect("Phenyl cancellation");
    assert_eq!(session.snapshot().expect("cancel is pure"), before);

    let attached = attach_phenyl(&mut session);
    let attached_snapshot = attached.observation().snapshot();
    let molecule = attached
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("source molecule");
    let exterior_bond_id = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.start().object_id() == Some(attached.focus_object_id())
                && bond.end().object_id() == Some(attached.compact_group_object_id())
        })
        .expect("directed anchor-to-group exterior bond")
        .document_object_id()
        .clone();
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        attached_snapshot.revision(),
        *attached_snapshot.digest(),
        molecule.document_object_id().clone(),
        attached.compact_group_object_id().clone(),
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            attached_snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("Phenyl materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("Phenyl materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("Phenyl materialization returns the focused generic result");
    };
    assert_eq!(
        outcome.compact_group_id(),
        attached.compact_group_object_id()
    );
    let selected_focus = materialized
        .observation()
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .and_then(|molecule| {
            molecule
                .atoms()
                .iter()
                .find(|atom| atom.document_object_id() == outcome.focus_atom_id())
        })
        .expect("committed result resolves the returned Phenyl carbon focus");
    assert_eq!(
        (selected_focus.element(), selected_focus.formal_charge()),
        (Some("C"), None)
    );
    assert_materialized_phenyl(
        &session,
        outcome.focus_atom_id(),
        &exterior_bond_id,
        attached.compact_group_object_id(),
    );

    let undone = session
        .undo(materialized.observation().snapshot().revision())
        .expect("undo");
    assert!(
        undone
            .observation()
            .projection()
            .molecules()
            .iter()
            .flat_map(|molecule| molecule.compact_groups())
            .any(|group| group.id() == attached.compact_group_object_id()
                && group.catalog_key() == CompactGroupCatalogKeyV1::Phenyl)
    );
    let redone = session
        .redo(undone.observation().snapshot().revision())
        .expect("redo");
    assert_materialized_phenyl(
        &session,
        outcome.focus_atom_id(),
        &exterior_bond_id,
        attached.compact_group_object_id(),
    );
    let reopened = DocumentSession::load(redone.observation().snapshot().cdml()).expect("reopen");
    assert_materialized_phenyl(
        &reopened,
        outcome.focus_atom_id(),
        &exterior_bond_id,
        attached.compact_group_object_id(),
    );
}

struct CrossBoundaryPhenylV1 {
    session: DocumentSession,
    revision: u64,
    molecule_id: DocumentObjectIdV1,
    focus_id: DocumentObjectIdV1,
    exterior_bond_id: DocumentObjectIdV1,
    anchor_is_start: bool,
}

fn materialize_authored_phenyl_cross_boundary_v1(anchor_is_start: bool) -> CrossBoundaryPhenylV1 {
    let exterior = if anchor_is_start {
        "<bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/>"
    } else {
        "<bond id=\"outside\" start=\"group\" end=\"anchor\" type=\"n1\"/>"
    };
    let source = format!(
        concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
            "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<compact-group id=\"group\" version=\"1\" catalog-key=\"phenyl\" ",
            "attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/>",
            "</compact-group>{}</molecule></cdml>"
        ),
        exterior
    );
    let mut session = DocumentSession::load(&source).expect("authored Phenyl source loads");
    let observation = session
        .document_observation()
        .expect("authored observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id() == Some("m"))
        .expect("authored molecule");
    let molecule_id = molecule.document_object_id().clone();
    let group_id = molecule
        .compact_groups()
        .iter()
        .find(|group| group.catalog_key() == CompactGroupCatalogKeyV1::Phenyl)
        .expect("authored Phenyl group")
        .id()
        .clone();
    let exterior_bond_id = molecule
        .bonds()
        .iter()
        .find(|bond| bond.source_id() == Some("outside"))
        .expect("authored exterior bond")
        .document_object_id()
        .clone();
    let snapshot = observation.snapshot();
    let request = DocumentCompactGroupMaterializationRequestV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        molecule_id.clone(),
        group_id,
    );
    let mut pending = session
        .prepare_session_operation_transition_v1(SessionOperationTransitionRequestV1::new(
            snapshot.revision(),
            SessionOperation::V1(SessionOperationV1::MaterializeCompactGroupV1(request)),
            TransitionAuthorizationV1::None,
        ))
        .expect("generic Phenyl materialization prepares");
    let materialized = session
        .commit_session_operation_transition_v1(&mut pending)
        .expect("generic Phenyl materialization commits");
    let SessionOperationOutcomeV1::CompactGroupMaterializedV1(outcome) = materialized.outcome()
    else {
        panic!("materialization returns the focused generic result");
    };
    CrossBoundaryPhenylV1 {
        revision: materialized.observation().snapshot().revision(),
        session,
        molecule_id,
        focus_id: outcome.focus_atom_id().clone(),
        exterior_bond_id,
        anchor_is_start,
    }
}

fn assert_cross_boundary_exterior_v1(case: &CrossBoundaryPhenylV1) {
    let observation = case
        .session
        .document_observation()
        .expect("materialized observation");
    let molecule = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.document_object_id() == &case.molecule_id)
        .expect("materialized molecule remains addressed");
    let exterior = molecule
        .bonds()
        .iter()
        .find(|bond| bond.document_object_id() == &case.exterior_bond_id)
        .expect("materialized exterior bond retains its durable ID");
    assert_eq!(
        (exterior.order(), exterior.style()),
        (
            Some(ferrum_core::BondOrder::Single),
            Some(&ferrum_core::BondStyle::Normal)
        )
    );
    if case.anchor_is_start {
        assert_eq!(exterior.start().source_id(), Some("anchor"));
        assert_eq!(exterior.end().object_id(), Some(&case.focus_id));
    } else {
        assert_eq!(exterior.end().source_id(), Some("anchor"));
        assert_eq!(exterior.start().object_id(), Some(&case.focus_id));
    }
}

struct PhenylRingRolesV1 {
    focus: DocumentObjectIdV1,
    single_branch: DocumentObjectIdV1,
    after_single_double: DocumentObjectIdV1,
    after_single_double_single: DocumentObjectIdV1,
    after_single_double_single_double: DocumentObjectIdV1,
    double_branch: DocumentObjectIdV1,
    focus_single_bond: DocumentObjectIdV1,
    single_double_bond: DocumentObjectIdV1,
    double_single_bond: DocumentObjectIdV1,
    single_double_second_bond: DocumentObjectIdV1,
    double_single_second_bond: DocumentObjectIdV1,
    double_focus_bond: DocumentObjectIdV1,
}

fn other_endpoint_id_v1(
    bond: &crate::BondProjectionV1,
    known: &DocumentObjectIdV1,
) -> DocumentObjectIdV1 {
    let start = bond.start().object_id();
    let end = bond.end().object_id();
    if start == Some(known) {
        return end.expect("ring bond end is an atom").clone();
    }
    if end == Some(known) {
        return start.expect("ring bond start is an atom").clone();
    }
    panic!("named Phenyl bond is incident to its named role");
}

fn bond_from_role_v1(
    molecule: &crate::MoleculeProjectionV1,
    role: &DocumentObjectIdV1,
    order: ferrum_core::BondOrder,
    excluded: &[DocumentObjectIdV1],
) -> (DocumentObjectIdV1, DocumentObjectIdV1) {
    let bond = molecule
        .bonds()
        .iter()
        .find(|bond| {
            bond.order() == Some(order)
                && bond.style() == Some(&ferrum_core::BondStyle::Normal)
                && (bond.start().object_id() == Some(role) || bond.end().object_id() == Some(role))
        })
        .filter(|bond| !excluded.iter().any(|id| bond.document_object_id() == id))
        .expect("named Phenyl role has its required normal-order edge");
    (
        bond.document_object_id().clone(),
        other_endpoint_id_v1(bond, role),
    )
}

fn assert_document_edge_v1(
    molecule: &crate::MoleculeProjectionV1,
    bond_id: &DocumentObjectIdV1,
    start: &DocumentObjectIdV1,
    end: &DocumentObjectIdV1,
    order: ferrum_core::BondOrder,
) {
    let bond = molecule
        .bonds()
        .iter()
        .find(|bond| bond.document_object_id() == bond_id)
        .expect("named Phenyl edge remains durable");
    assert_eq!(bond.order(), Some(order));
    assert_eq!(bond.style(), Some(&ferrum_core::BondStyle::Normal));
    assert!(
        (bond.start().object_id() == Some(start) && bond.end().object_id() == Some(end))
            || (bond.start().object_id() == Some(end) && bond.end().object_id() == Some(start)),
        "named Phenyl edge joins its named endpoint roles"
    );
}

fn phenyl_ring_roles_v1(
    molecule: &crate::MoleculeProjectionV1,
    focus: &DocumentObjectIdV1,
    exterior_bond_id: &DocumentObjectIdV1,
) -> PhenylRingRolesV1 {
    let (focus_single_bond, single_branch) = bond_from_role_v1(
        molecule,
        focus,
        ferrum_core::BondOrder::Single,
        std::slice::from_ref(exterior_bond_id),
    );
    let (double_focus_bond, double_branch) =
        bond_from_role_v1(molecule, focus, ferrum_core::BondOrder::Double, &[]);
    let (single_double_bond, after_single_double) = bond_from_role_v1(
        molecule,
        &single_branch,
        ferrum_core::BondOrder::Double,
        &[],
    );
    let (double_single_bond, after_single_double_single) = bond_from_role_v1(
        molecule,
        &after_single_double,
        ferrum_core::BondOrder::Single,
        &[],
    );
    let (single_double_second_bond, after_single_double_single_double) = bond_from_role_v1(
        molecule,
        &after_single_double_single,
        ferrum_core::BondOrder::Double,
        &[],
    );
    let (double_single_second_bond, closing_branch) = bond_from_role_v1(
        molecule,
        &after_single_double_single_double,
        ferrum_core::BondOrder::Single,
        &[],
    );
    assert_eq!(
        closing_branch, double_branch,
        "alternating ring path closes at double branch"
    );
    let roles = PhenylRingRolesV1 {
        focus: focus.clone(),
        single_branch,
        after_single_double,
        after_single_double_single,
        after_single_double_single_double,
        double_branch,
        focus_single_bond,
        single_double_bond,
        double_single_bond,
        single_double_second_bond,
        double_single_second_bond,
        double_focus_bond,
    };
    assert_document_edge_v1(
        molecule,
        &roles.focus_single_bond,
        &roles.focus,
        &roles.single_branch,
        ferrum_core::BondOrder::Single,
    );
    assert_document_edge_v1(
        molecule,
        &roles.single_double_bond,
        &roles.single_branch,
        &roles.after_single_double,
        ferrum_core::BondOrder::Double,
    );
    assert_document_edge_v1(
        molecule,
        &roles.double_single_bond,
        &roles.after_single_double,
        &roles.after_single_double_single,
        ferrum_core::BondOrder::Single,
    );
    assert_document_edge_v1(
        molecule,
        &roles.single_double_second_bond,
        &roles.after_single_double_single,
        &roles.after_single_double_single_double,
        ferrum_core::BondOrder::Double,
    );
    assert_document_edge_v1(
        molecule,
        &roles.double_single_second_bond,
        &roles.after_single_double_single_double,
        &roles.double_branch,
        ferrum_core::BondOrder::Single,
    );
    assert_document_edge_v1(
        molecule,
        &roles.double_focus_bond,
        &roles.double_branch,
        &roles.focus,
        ferrum_core::BondOrder::Double,
    );
    roles
}

fn assert_distinct_neutral_carbons_v1(
    molecule: &crate::MoleculeProjectionV1,
    roles: &PhenylRingRolesV1,
) {
    let named = [
        &roles.focus,
        &roles.single_branch,
        &roles.after_single_double,
        &roles.after_single_double_single,
        &roles.after_single_double_single_double,
        &roles.double_branch,
    ];
    for role in named {
        let atom = molecule
            .atoms()
            .iter()
            .find(|atom| atom.document_object_id() == role)
            .expect("named ring role resolves to atom");
        assert_eq!((atom.element(), atom.formal_charge()), (Some("C"), None));
    }
    assert_ne!(roles.focus, roles.single_branch);
    assert_ne!(roles.focus, roles.after_single_double);
    assert_ne!(roles.focus, roles.after_single_double_single);
    assert_ne!(roles.focus, roles.after_single_double_single_double);
    assert_ne!(roles.focus, roles.double_branch);
    assert_ne!(roles.single_branch, roles.after_single_double);
    assert_ne!(roles.single_branch, roles.after_single_double_single);
    assert_ne!(roles.single_branch, roles.after_single_double_single_double);
    assert_ne!(roles.single_branch, roles.double_branch);
    assert_ne!(roles.after_single_double, roles.after_single_double_single);
    assert_ne!(
        roles.after_single_double,
        roles.after_single_double_single_double
    );
    assert_ne!(roles.after_single_double, roles.double_branch);
    assert_ne!(
        roles.after_single_double_single,
        roles.after_single_double_single_double
    );
    assert_ne!(roles.after_single_double_single, roles.double_branch);
    assert_ne!(roles.after_single_double_single_double, roles.double_branch);
}

fn assert_finite_nonzero_parallel_lines_v1(
    first: &ferrum_render::LineOp,
    second: &ferrum_render::LineOp,
) {
    let vector = |line: &ferrum_render::LineOp| {
        let dx = line.end().x() - line.start().x();
        let dy = line.end().y() - line.start().y();
        assert!(line.start().x().is_finite());
        assert!(line.start().y().is_finite());
        assert!(line.end().x().is_finite());
        assert!(line.end().y().is_finite());
        assert!(dx != 0.0 || dy != 0.0, "ordinary line is nondegenerate");
        (dx, dy)
    };
    let (first_x, first_y) = vector(first);
    let (second_x, second_y) = vector(second);
    let determinant = first_x * second_y - first_y * second_x;
    let scale = (first_x.hypot(first_y) * second_x.hypot(second_y)).max(1.0);
    assert!(
        determinant.abs() <= scale * 1.0e-12,
        "ordinary double-bond lanes are parallel"
    );
}

fn assert_finite_nonzero_line_v1(line: &ferrum_render::LineOp) {
    assert!(line.start().x().is_finite());
    assert!(line.start().y().is_finite());
    assert!(line.end().x().is_finite());
    assert!(line.end().y().is_finite());
    assert!(line.start() != line.end(), "ordinary line is nondegenerate");
}

fn assert_native_role_edge_v1(
    edges: &[(usize, usize)],
    records: &[ferrum_core::RecordId],
    graph: &ferrum_chemistry::MolGraph,
    start: &ferrum_core::RecordId,
    end: &ferrum_core::RecordId,
    order: ferrum_chemistry::BondOrder,
) {
    let native = edges
        .iter()
        .zip(graph.bonds())
        .find(|((start_position, end_position), _)| {
            let native_start = records.get(*start_position);
            let native_end = records.get(*end_position);
            (native_start == Some(start) && native_end == Some(end))
                || (native_start == Some(end) && native_end == Some(start))
        })
        .expect("named document edge retains its native graph mapping");
    assert_eq!(native.1.order(), order);
    assert!(!native.1.is_aromatic());
}

fn assert_native_coordinate_v1(
    records: &[ferrum_core::RecordId],
    points: &[ferrum_chemistry::Point2],
    record: &ferrum_core::RecordId,
    document_position: ferrum_core::Position,
) {
    let point = records
        .iter()
        .zip(points)
        .find_map(|(mapped, point)| (mapped == record).then_some(point))
        .expect("named ring role retains native coordinate mapping");
    assert!(point.x().is_finite() && point.y().is_finite());
    assert_eq!(point.x(), document_position.x());
    assert_eq!(point.y(), -document_position.y());
}

fn assert_target_line_role_v1(
    molecule_plan: &ferrum_render::MoleculeRenderPlan,
    bond_id: &DocumentObjectIdV1,
    order: ferrum_core::BondOrder,
) {
    let batch = molecule_plan
        .batches()
        .iter()
        .find(|batch| batch.target().document_object_id() == bond_id)
        .expect("named Phenyl bond owns addressed render batch");
    assert!(
        !batch
            .operations()
            .iter()
            .any(|operation| matches!(operation, RenderOp::DoubleBondCarrierMark(_)))
    );
    let mut lines = batch
        .operations()
        .iter()
        .filter_map(|operation| match operation {
            RenderOp::Line(line) => Some(line),
            _ => None,
        });
    let first = lines
        .next()
        .expect("normal Phenyl bond emits an ordinary line");
    assert_finite_nonzero_line_v1(first);
    if order == ferrum_core::BondOrder::Double {
        let second = lines
            .find(|line| !std::ptr::eq(*line, first))
            .expect("normal double bond emits a distinct second ordinary line");
        assert_finite_nonzero_parallel_lines_v1(first, second);
    }
}

#[test]
fn attached_phenyl_normal_kekule_lowers_to_a_neutral_six_carbon_native_cycle_at_both_exterior_directions()
 {
    for anchor_is_start in [true, false] {
        let case = materialize_authored_phenyl_cross_boundary_v1(anchor_is_start);
        assert_cross_boundary_exterior_v1(&case);
        let materialized_observation = case
            .session
            .document_observation()
            .expect("materialized observation");
        let materialized_molecule = materialized_observation
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.document_object_id() == &case.molecule_id)
            .expect("materialized molecule remains addressed");
        let roles = phenyl_ring_roles_v1(
            materialized_molecule,
            &case.focus_id,
            &case.exterior_bond_id,
        );
        assert_distinct_neutral_carbons_v1(materialized_molecule, &roles);
        let snapshot = case.session.snapshot().expect("committed snapshot");
        let typed = TypedDocument::parse(snapshot.cdml()).expect("committed CDML reparses");
        let projection = typed.core_projection().expect("committed CDML reprojects");
        let molecule = projection
            .molecules()
            .iter()
            .find(|molecule| molecule.source_id().as_str() == "m")
            .expect("committed source molecule");
        let (graph, edges, records) = document_molecule_coordinate_graph_v1(molecule)
            .expect("explicit normal single/double Phenyl lowers to the native graph")
            .into_parts_with_atom_records();
        let record = |role: &DocumentObjectIdV1| {
            let source_id = materialized_molecule
                .atoms()
                .iter()
                .find(|atom| atom.document_object_id() == role)
                .and_then(|atom| atom.source_id())
                .expect("named materialized role retains source ID");
            molecule
                .atoms()
                .iter()
                .find(|atom| atom.source_id().as_str() == source_id)
                .expect("named role reprojects to atom")
        };
        let focus = record(&roles.focus);
        let single_branch = record(&roles.single_branch);
        let after_single_double = record(&roles.after_single_double);
        let after_single_double_single = record(&roles.after_single_double_single);
        let after_single_double_single_double = record(&roles.after_single_double_single_double);
        let double_branch = record(&roles.double_branch);
        assert_native_role_edge_v1(
            &edges,
            &records,
            &graph,
            focus.identity(),
            single_branch.identity(),
            ferrum_chemistry::BondOrder::Single,
        );
        assert_native_role_edge_v1(
            &edges,
            &records,
            &graph,
            single_branch.identity(),
            after_single_double.identity(),
            ferrum_chemistry::BondOrder::Double,
        );
        assert_native_role_edge_v1(
            &edges,
            &records,
            &graph,
            after_single_double.identity(),
            after_single_double_single.identity(),
            ferrum_chemistry::BondOrder::Single,
        );
        assert_native_role_edge_v1(
            &edges,
            &records,
            &graph,
            after_single_double_single.identity(),
            after_single_double_single_double.identity(),
            ferrum_chemistry::BondOrder::Double,
        );
        assert_native_role_edge_v1(
            &edges,
            &records,
            &graph,
            after_single_double_single_double.identity(),
            double_branch.identity(),
            ferrum_chemistry::BondOrder::Single,
        );
        assert_native_role_edge_v1(
            &edges,
            &records,
            &graph,
            double_branch.identity(),
            focus.identity(),
            ferrum_chemistry::BondOrder::Double,
        );
        let points = graph
            .coordinates()
            .expect("coordinate lowering retains named ring coordinates")
            .points();
        for atom in [
            focus,
            single_branch,
            after_single_double,
            after_single_double_single,
            after_single_double_single_double,
            double_branch,
        ] {
            assert_native_coordinate_v1(&records, points, atom.identity(), atom.position());
        }
    }
}

#[test]
fn materialized_phenyl_emits_normal_alternating_ring_draw_ops_for_both_exterior_directions() {
    for anchor_is_start in [true, false] {
        let case = materialize_authored_phenyl_cross_boundary_v1(anchor_is_start);
        assert_cross_boundary_exterior_v1(&case);
        let materialized_observation = case
            .session
            .document_observation()
            .expect("materialized observation");
        let materialized_molecule = materialized_observation
            .projection()
            .molecules()
            .iter()
            .find(|molecule| molecule.document_object_id() == &case.molecule_id)
            .expect("materialized molecule remains addressed");
        let roles = phenyl_ring_roles_v1(
            materialized_molecule,
            &case.focus_id,
            &case.exterior_bond_id,
        );
        let plan = compose_complete_document_render_plan_v1(&case.session, case.revision)
            .expect("materialized Phenyl composes one complete document render plan");
        let molecule_plan = plan
            .outcomes()
            .iter()
            .find_map(|outcome| match outcome {
                ferrum_render::DocumentRenderOutcomeV1::Root(root)
                    if root.target().document_object_id() == &case.molecule_id =>
                {
                    match root.content() {
                        ferrum_render::DocumentRenderContentV1::Molecule(content) => {
                            Some(content.plan())
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .expect("materialized molecule has its addressed render root");
        assert_target_line_role_v1(
            molecule_plan,
            &case.exterior_bond_id,
            ferrum_core::BondOrder::Single,
        );
        assert_target_line_role_v1(
            molecule_plan,
            &roles.focus_single_bond,
            ferrum_core::BondOrder::Single,
        );
        assert_target_line_role_v1(
            molecule_plan,
            &roles.single_double_bond,
            ferrum_core::BondOrder::Double,
        );
        assert_target_line_role_v1(
            molecule_plan,
            &roles.double_single_bond,
            ferrum_core::BondOrder::Single,
        );
        assert_target_line_role_v1(
            molecule_plan,
            &roles.single_double_second_bond,
            ferrum_core::BondOrder::Double,
        );
        assert_target_line_role_v1(
            molecule_plan,
            &roles.double_single_second_bond,
            ferrum_core::BondOrder::Single,
        );
        assert_target_line_role_v1(
            molecule_plan,
            &roles.double_focus_bond,
            ferrum_core::BondOrder::Double,
        );
    }
}
