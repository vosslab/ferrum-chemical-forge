use super::*;
use ferrum_document::PresentationRootProjectionV1;
const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m1\"><atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"a2\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"n1\"/></molecule><molecule id=\"m2\"><atom id=\"a3\" name=\"N\"><point x=\"60\" y=\"0\"/></atom></molecule></cdml>";
const MIXED_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\"><atom id=\"atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><plus id=\"plus\"><point x=\"40\" y=\"0\"/></plus></cdml>";
fn fence(session: &RenderInteractionSessionV1) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("snapshot");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
}

#[test]
fn render_session_exposes_revision_bound_smarts_preparation() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let prepared = session
        .prepare_smarts_snapshot_v1(snapshot.revision())
        .expect("prepare current revision");
    assert_eq!(prepared.revision(), snapshot.revision());
    assert_eq!(prepared.digest(), snapshot.digest());
    assert!(matches!(
        session.prepare_smarts_snapshot_v1(snapshot.revision() + 1),
        Err(DocumentSmartsSnapshotErrorV1::StaleRevision { .. })
    ));
}

#[test]
fn structural_line_hit_and_marquee_follow_the_rendered_stroke_not_its_box() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"20\" y=\"20\"/></atom>",
        "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/>",
        "</molecule></cdml>",
    );
    let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let observation = session
        .observe_structure_interaction_v1(fence(&session))
        .expect("observe");
    let corner = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: 1.0,
                y: 19.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("corner query");
    assert!(corner.is_empty());
    let line = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: 10.0,
                y: 10.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("line query");
    assert!(
        line.targets()
            .iter()
            .any(|target| target.kind() == StructureTargetKindV1::Bond)
    );
    let clipped = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Marquee {
                left: 0.0,
                top: 0.0,
                right: 20.0,
                bottom: 20.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("clipped marquee");
    assert!(
        !clipped
            .targets()
            .iter()
            .any(|target| target.kind() == StructureTargetKindV1::Bond)
    );
    let contained = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Marquee {
                left: -1.0,
                top: -1.0,
                right: 21.0,
                bottom: 21.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("stroke-contained marquee");
    assert!(
        contained
            .targets()
            .iter()
            .any(|target| target.kind() == StructureTargetKindV1::Bond)
    );
}

#[test]
fn structural_path_bond_is_a_typed_display_only_target() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"30\" y=\"0\"/></atom>",
        "<bond id=\"ab\" type=\"w1\" start=\"a\" end=\"b\"/>",
        "</molecule></cdml>",
    );
    let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let observation = session
        .observe_structure_interaction_v1(fence(&session))
        .expect("observe");
    let display = observation
        .targets()
        .iter()
        .find(|target| target.kind() == StructureTargetKindV1::DisplayOnly)
        .expect("wedge target remains visible to the interaction facade");
    assert_eq!(display.kind(), StructureTargetKindV1::DisplayOnly);
    let bounds = display.bounds();
    assert!(matches!(
        session.select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: (bounds.left() + bounds.right()) / 2.0,
                y: (bounds.top() + bounds.bottom()) / 2.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        ),
        Err(RenderInteractionErrorV1::DisplayOnly)
    ));
}

#[test]
fn compact_group_render_primitive_is_visible_and_selectable_without_changing_atom_selection() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
        "<atom id=\"atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"45\"><point x=\"36\" y=\"18\"/></compact-group>",
        "</molecule></cdml>",
    );
    let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let rendered = session
        .observe_render_v1(snapshot.revision())
        .expect("render");
    let plan = rendered
        .resolved()
        .molecule_plans()
        .first()
        .expect("molecule plan");
    let primitive = plan
        .compact_group_primitives()
        .first()
        .expect("typed compact-group render primitive");
    let bounds = primitive.bounds();
    assert!(
        [
            bounds.min_x(),
            bounds.min_y(),
            bounds.max_x(),
            bounds.max_y()
        ]
        .into_iter()
        .all(f64::is_finite)
    );
    assert!(bounds.min_x() < bounds.max_x() && bounds.min_y() < bounds.max_y());

    let observation = session
        .observe_structure_interaction_v1(fence(&session))
        .expect("interaction observation");
    let group = observation
        .targets()
        .iter()
        .find(|target| target.kind() == StructureTargetKindV1::CompactGroup)
        .expect("visible compact-group target");
    let group_bounds = group.bounds();
    let selected_group = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: (group_bounds.left() + group_bounds.right()) / 2.0,
                y: (group_bounds.top() + group_bounds.bottom()) / 2.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("group selection");
    assert_eq!(selected_group.targets().len(), 1);
    assert_eq!(
        selected_group.targets()[0].kind(),
        StructureTargetKindV1::CompactGroup
    );
    assert_eq!(selected_group.targets()[0].object_id(), group.object_id());

    let selected_atom = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: 0.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("atom selection");
    assert_eq!(selected_atom.targets().len(), 1);
    assert_eq!(
        selected_atom.targets()[0].kind(),
        StructureTargetKindV1::Atom
    );
}

#[test]
fn typed_compact_group_exterior_bond_reaches_the_complete_document_render_plan() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
        "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"group\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"36\" y=\"0\"/></compact-group>",
        "<bond id=\"attachment\" start=\"anchor\" end=\"group\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let rendered = session
        .observe_render_v1(snapshot.revision())
        .expect("render");
    let molecule = rendered
        .resolved()
        .molecule_plans()
        .first()
        .expect("molecule render plan");
    let group = molecule
        .compact_group_primitives()
        .first()
        .expect("visible compact-group primitive");
    let endpoint = group.bond_endpoint().expect("compact-group endpoint");
    let line = molecule
        .batches()
        .iter()
        .flat_map(|batch| batch.operations())
        .find_map(|operation| match operation {
            ferrum_render::RenderOp::Line(line) => Some(line),
            _ => None,
        })
        .expect("normal exterior line");
    assert!(
        [
            endpoint.position().x(),
            endpoint.position().y(),
            line.start().x(),
            line.start().y(),
            line.end().x(),
            line.end().y(),
        ]
        .into_iter()
        .all(f64::is_finite)
    );
    let observation = session
        .observe_structure_interaction_v1(fence(&session))
        .expect("interaction observation");
    assert!(observation.targets().iter().any(|target| {
        target.kind() == StructureTargetKindV1::CompactGroup
            && target.object_id() == group.target().document_object_id()
    }));
}

#[test]
fn render_plan_controls_point_marquee_translate_and_undo() {
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    assert_eq!(observation.roots().len(), 2);
    let selected = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Point {
                x: 0.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("point hit");
    assert_eq!(selected.roots().len(), 1);
    let clipped = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Marquee {
                left: -1.0,
                top: -1.0,
                right: 10.0,
                bottom: 1.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("marquee");
    assert!(clipped.is_empty());
    let gesture = session
        .begin_render_interaction_translation_v1(
            &selected,
            10.0,
            20.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin");
    let preview = session
        .preview_render_interaction_translation_v1(&gesture, 14.0, 17.0)
        .expect("preview");
    assert_eq!((preview.dx(), preview.dy()), (4.0, -3.0));
    let committed = session
        .commit_render_interaction_translation_v1(gesture, 19.0, 18.0)
        .expect("commit");
    assert!(committed.changed());
    assert_eq!(committed.result().observation().snapshot().revision(), 1);
    assert_eq!(
        session
            .undo(1)
            .expect("undo")
            .observation()
            .snapshot()
            .revision(),
        2
    );
}

#[test]
fn stale_translation_gesture_preserves_current_document_and_history() {
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe stale gesture source");
    let root_id = observation.roots()[0].document_object_id().clone();
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: root_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select stale gesture root");
    let stale_gesture = session
        .begin_render_interaction_translation_v1(
            &selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin stale gesture");
    let _stale_preview = session
        .preview_render_interaction_translation_v1(&stale_gesture, 5.0, 0.0)
        .expect("preview stale gesture");

    let advancing_gesture = session
        .begin_render_interaction_translation_v1(
            &selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin advancing gesture");
    let _advancing_preview = session
        .preview_render_interaction_translation_v1(&advancing_gesture, 2.0, 0.0)
        .expect("preview advancing gesture");
    session
        .commit_render_interaction_translation_v1(advancing_gesture, 2.0, 0.0)
        .expect("advance revision");
    let before = session.snapshot().expect("snapshot after advance");

    assert!(matches!(
        session.commit_render_interaction_translation_v1(stale_gesture, 5.0, 0.0),
        Err(RenderInteractionErrorV1::StaleRevision)
    ));
    assert_eq!(session.snapshot().expect("snapshot after refusal"), before);
    assert_eq!(
        session
            .undo(1)
            .expect("the one admitted commit remains undoable")
            .observation()
            .snapshot()
            .revision(),
        2
    );
}

#[test]
fn interaction_refuses_renderer_plan_with_stale_provenance() {
    let mut session =
        RenderInteractionSessionV1::new(DocumentSession::load(MIXED_SOURCE).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let stale_plan = ferrum_render::render_presentation_stack_v1(
        session
            .observe(snapshot.revision())
            .expect("document observation")
            .projection()
            .presentation_stack(),
    )
    .expect("current semantic stack renders");
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let molecule_id = observation.roots()[0].document_object_id().clone();
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: molecule_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select molecule");
    let gesture = session
        .begin_render_interaction_translation_v1(
            &selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin translation");
    let _preview = session
        .preview_render_interaction_translation_v1(&gesture, 5.0, 0.0)
        .expect("preview translation");
    session
        .commit_render_interaction_translation_v1(gesture, 5.0, 0.0)
        .expect("advance revision");
    assert!(matches!(
        session.observe_render_interaction_with_presentation_plan_v1(fence(&session), &stale_plan),
        Err(RenderInteractionErrorV1::StaleRevision)
    ));

    let mut other = RenderInteractionSessionV1::new(
        DocumentSession::load(MIXED_SOURCE).expect("other document"),
    );
    let other_observation = other
        .observe_render_interaction_v1(fence(&other))
        .expect("other observation");
    let other_molecule_id = other_observation.roots()[0].document_object_id().clone();
    let other_selection = other
        .select_render_interaction_roots_v1(
            &other_observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: other_molecule_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select other molecule");
    let other_gesture = other
        .begin_render_interaction_translation_v1(
            &other_selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin other translation");
    let _other_preview = other
        .preview_render_interaction_translation_v1(&other_gesture, 7.0, 0.0)
        .expect("preview other translation");
    other
        .commit_render_interaction_translation_v1(other_gesture, 7.0, 0.0)
        .expect("advance other revision");
    let other_snapshot = other.snapshot().expect("other snapshot");
    let other_plan = ferrum_render::render_presentation_stack_v1(
        other
            .observe(other_snapshot.revision())
            .expect("other observation")
            .projection()
            .presentation_stack(),
    )
    .expect("other semantic stack renders");
    assert!(matches!(
        session.observe_render_interaction_with_presentation_plan_v1(fence(&session), &other_plan),
        Err(RenderInteractionErrorV1::StaleDigest)
    ));
}

#[test]
fn toggled_render_roots_keep_renderer_paint_order() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let first_root_id = observation.roots()[0].document_object_id().clone();
    let second_root_id = observation.roots()[1].document_object_id().clone();
    let later = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: second_root_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select later root");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            Some(&later),
            RenderInteractionQueryV1::Root {
                document_object_id: first_root_id,
                modifier: RenderInteractionModifierV1::Toggle,
            },
        )
        .expect("toggle earlier root");

    assert_eq!(
        selection
            .roots()
            .iter()
            .map(RenderInteractionRootV1::paint_order)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn structure_selection_deletes_atom_and_incident_bond_in_one_fenced_commit() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"20\" y=\"0\"/></atom>",
        "<bond id=\"ab\" type=\"n1\" start=\"a\" end=\"b\"/>",
        "</molecule></cdml>",
    );
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let snapshot = session.snapshot().expect("snapshot");
    let observation = session
        .observe_structure_interaction_v1(DocumentFenceV1::new(
            snapshot.revision(),
            *snapshot.digest(),
        ))
        .expect("observe");
    let atom = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: 0.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("atom select");
    assert_eq!(atom.targets().len(), 1);
    assert_eq!(atom.targets()[0].kind(), StructureTargetKindV1::Atom);
    let selection = session
        .select_structure_interaction_v1(
            &observation,
            Some(&atom),
            StructureInteractionQueryV1::Point {
                x: 10.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Toggle,
            },
        )
        .expect("bond toggle");
    assert_eq!(selection.targets().len(), 2);
    let commit = session
        .commit_structure_deletion_v1(&selection)
        .expect("delete");
    assert_eq!(commit.removed_atom_count(), 1);
    assert_eq!(commit.removed_bond_count(), 1);
    assert_eq!(commit.removed_compact_group_count(), 0);
    let molecule = &commit.result().observation().projection().molecules()[0];
    assert_eq!(molecule.atoms().len(), 1);
    assert!(molecule.bonds().is_empty());
    assert!(matches!(
        session.commit_structure_deletion_v1(&selection),
        Err(RenderInteractionErrorV1::StaleRevision)
    ));
}

#[test]
fn compact_group_deletion_topology_requires_document_repair() {
    let source = concat!(
        "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<compact-group id=\"g\" version=\"1\" catalog-key=\"methyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group>",
        "</molecule></cdml>",
    );
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let observation = session
        .observe_structure_interaction_v1(fence(&session))
        .expect("observe");
    let selection = session
        .select_structure_interaction_v1(
            &observation,
            None,
            StructureInteractionQueryV1::Point {
                x: 20.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select compact group");

    let error = session
        .commit_structure_deletion_v1(&selection)
        .expect_err("invalid compact topology refuses");
    assert_eq!(
        error,
        RenderInteractionErrorV1::InvalidCompactGroupDeletionTopology
    );
    assert_eq!(
        error.to_string(),
        "the compact group deletion topology requires document repair before retry"
    );
}

#[test]
fn view_hex_grid_policy_snaps_preview_delta_in_rust() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let root_id = observation.roots()[0].document_object_id().clone();
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: root_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select");
    let raw = session
        .begin_render_interaction_translation_v1(
            &selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("raw gesture");
    let snapped = session
        .begin_render_interaction_translation_v1(
            &selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::with_grid_policy(
                RenderInteractionAxisV1::Free,
                RenderInteractionGridSnapPolicyV1::ViewHexGrid,
            ),
        )
        .expect("grid gesture");
    let raw_preview = session
        .preview_render_interaction_translation_v1(&raw, 38.0, 18.0)
        .expect("raw preview");
    let grid_preview = session
        .preview_render_interaction_translation_v1(&snapped, 38.0, 18.0)
        .expect("grid preview");
    assert_eq!((raw_preview.dx(), raw_preview.dy()), (38.0, 18.0));
    assert_ne!(
        (grid_preview.dx(), grid_preview.dy()),
        (raw_preview.dx(), raw_preview.dy())
    );
}

#[test]
fn view_hex_grid_snaps_the_mixed_root_anchor_after_an_off_lattice_drag() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\"><atom id=\"atom\" name=\"C\"><point x=\"11\" y=\"17\"/></atom></molecule><plus id=\"plus\"><point x=\"53\" y=\"9\"/></plus></cdml>";
    let session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let molecule_id = observation.roots()[0].document_object_id().clone();
    let plus_id = observation.roots()[1].document_object_id().clone();
    let molecule = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: molecule_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select molecule");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            Some(&molecule),
            RenderInteractionQueryV1::Root {
                document_object_id: plus_id,
                modifier: RenderInteractionModifierV1::Toggle,
            },
        )
        .expect("select mixed roots");
    let press = (7.0, 11.0);
    let release = (31.0, 23.0);
    let free = session
        .begin_render_interaction_translation_v1(
            &selection,
            press.0,
            press.1,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin free gesture");
    assert_eq!(
        session
            .preview_render_interaction_translation_v1(&free, release.0, release.1)
            .expect("preview free gesture")
            .dx(),
        release.0 - press.0
    );
    let snapped = session
        .begin_render_interaction_translation_v1(
            &selection,
            press.0,
            press.1,
            RenderInteractionSnapV1::with_grid_policy(
                RenderInteractionAxisV1::Free,
                RenderInteractionGridSnapPolicyV1::ViewHexGrid,
            ),
        )
        .expect("begin snapped gesture");
    let preview = session
        .preview_render_interaction_translation_v1(&snapped, release.0, release.1)
        .expect("preview snapped gesture");
    let expected_delta = (20.0 * 3.0_f64.sqrt() - 11.0, 11.0);
    assert!((preview.dx() - expected_delta.0).abs() < f64::EPSILON);
    assert_eq!(preview.dy(), expected_delta.1);
}

#[test]
fn view_hex_grid_exact_click_keeps_an_off_lattice_root_unchanged() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\"><atom id=\"atom\" name=\"C\"><point x=\"11\" y=\"17\"/></atom></molecule></cdml>";
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let root_id = observation.roots()[0].document_object_id().clone();
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: root_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select off-lattice root");
    let gesture = session
        .begin_render_interaction_translation_v1(
            &selection,
            7.0,
            11.0,
            RenderInteractionSnapV1::with_grid_policy(
                RenderInteractionAxisV1::Free,
                RenderInteractionGridSnapPolicyV1::ViewHexGrid,
            ),
        )
        .expect("begin grid gesture");
    let preview = session
        .preview_render_interaction_translation_v1(&gesture, 7.0, 11.0)
        .expect("preview exact click");
    assert_eq!((preview.dx(), preview.dy()), (0.0, 0.0));
    let committed = session
        .commit_render_interaction_translation_v1(gesture, 7.0, 11.0)
        .expect("commit exact click");
    assert!(!committed.changed());
    assert_eq!(committed.result().observation().snapshot().revision(), 0);
}
#[test]
fn unsupported_and_foreign_handles_are_refused_without_mutation() {
    let unsupported = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/><ftext><b>rich</b></ftext></atom></molecule></cdml>";
    let session =
        RenderInteractionSessionV1::new(DocumentSession::load(unsupported).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    assert!(observation.roots().is_empty());
    assert_eq!(observation.exclusions().len(), 1);
    let exclusion_id = observation.exclusions()[0].document_object_id().clone();
    assert_eq!(
        observation.exclusions()[0].reason(),
        RenderInteractionExclusionReasonV1::UnrenderableDepiction
    );
    assert!(matches!(
        session.select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: exclusion_id,
                modifier: RenderInteractionModifierV1::Replace,
            }
        ),
        Err(RenderInteractionErrorV1::UnrenderableDepiction)
    ));
    assert!(
        session
            .select_render_interaction_roots_v1(
                &observation,
                None,
                RenderInteractionQueryV1::Point {
                    x: 0.0,
                    y: 0.0,
                    modifier: RenderInteractionModifierV1::Replace,
                }
            )
            .expect("blank is not an excluded-root refusal")
            .is_empty()
    );
    let other = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    assert!(matches!(
        other.select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Clear
        ),
        Err(RenderInteractionErrorV1::ForeignSession)
    ));
    assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
}

#[test]
fn fragment_member_idref_does_not_exclude_renderable_root() {
    let opaque_declaration_collision = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><extension><fragment><bond id=\"m\"/></fragment></extension></cdml>";
    assert!(DocumentSession::load(opaque_declaration_collision).is_err());
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><fragment id=\"fragment\"><bond id=\"m\"/></fragment></molecule></cdml>";
    let mut session = RenderInteractionSessionV1::new(
        DocumentSession::load(source).expect("fragment reference fixture loads"),
    );
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("fragment reference fixture observes");
    assert_eq!(observation.roots().len(), 1);
    let root_id = observation.roots()[0].document_object_id().clone();
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                document_object_id: root_id,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("IDREF does not make molecule ambiguous");
    let gesture = session
        .begin_render_interaction_translation_v1(
            &selection,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin move");
    let _preview = session
        .preview_render_interaction_translation_v1(&gesture, 3.0, 0.0)
        .expect("preview move");
    assert_eq!(
        session
            .commit_render_interaction_translation_v1(gesture, 3.0, 0.0)
            .expect("IDREF-safe move commits")
            .result()
            .observation()
            .snapshot()
            .revision(),
        1
    );
}

#[test]
fn reaction_authoring_choices_keep_renderer_paint_order() {
    let session =
        RenderInteractionSessionV1::new(DocumentSession::load(MIXED_SOURCE).expect("load"));
    let choices = session
        .observe_reaction_authoring_choices_v1(fence(&session))
        .expect("observe reaction authoring choices");
    assert_eq!(
        choices
            .choices()
            .iter()
            .map(ReactionAuthoringChoiceV1::kind)
            .collect::<Vec<_>>(),
        [
            ReactionAuthoringChoiceKindV1::Molecule,
            ReactionAuthoringChoiceKindV1::Plus,
        ]
    );
    assert_eq!(
        choices
            .choices()
            .iter()
            .map(ReactionAuthoringChoiceV1::paint_order)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn mixed_molecule_and_plus_selection_moves_in_one_history_commit() {
    let mut session = RenderInteractionSessionV1::new(
        DocumentSession::load(MIXED_SOURCE).expect("mixed fixture loads"),
    );
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("mixed fixture observes");
    assert_eq!(observation.roots().len(), 2);
    let molecule = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Point {
                x: 0.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("molecule hit");
    let selected = session
        .select_render_interaction_roots_v1(
            &observation,
            Some(&molecule),
            RenderInteractionQueryV1::Point {
                x: 40.0,
                y: 0.0,
                modifier: RenderInteractionModifierV1::Toggle,
            },
        )
        .expect("plus render-layout hit");
    assert_eq!(selected.roots().len(), 2);
    let gesture = session
        .begin_render_interaction_translation_v1(
            &selected,
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin mixed move");
    let _preview = session
        .preview_render_interaction_translation_v1(&gesture, 7.0, 4.0)
        .expect("preview mixed move");
    let committed = session
        .commit_render_interaction_translation_v1(gesture, 7.0, 4.0)
        .expect("one mixed session operation");
    assert_eq!(committed.result().observation().snapshot().revision(), 1);
    let projection = committed.result().observation().projection();
    assert!((projection.molecules()[0].atoms()[0].position().x() - 7.0).abs() < 0.01);
    let PresentationRootProjectionV1::Plus { plus } =
        projection.presentation_stack().entries()[0].root()
    else {
        panic!("fixture must retain plus");
    };
    assert!((plus.anchor().x() - 47.0).abs() < 0.01);
    assert!((plus.anchor().y() - 4.0).abs() < 0.01);
    assert_eq!(
        session
            .undo(1)
            .expect("one mixed undo")
            .observation()
            .snapshot()
            .revision(),
        2
    );
}
