use super::*;
const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m1\"><atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><atom id=\"a2\" name=\"O\"><point x=\"20\" y=\"0\"/></atom><bond id=\"b1\" start=\"a1\" end=\"a2\" type=\"n1\"/></molecule><molecule id=\"m2\"><atom id=\"a3\" name=\"N\"><point x=\"60\" y=\"0\"/></atom></molecule></cdml>";
const MIXED_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\"><atom id=\"atom\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><plus id=\"plus\"><point x=\"40\" y=\"0\"/></plus></cdml>";
fn fence(session: &RenderInteractionSessionV1) -> DocumentFenceV1 {
    let snapshot = session.snapshot().expect("snapshot");
    DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
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
        .find(|target| target.identifier() == "ab")
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
            0.0,
            0.0,
            RenderInteractionSnapV1::free(),
        )
        .expect("begin");
    let preview = session
        .preview_render_interaction_translation_v1(&gesture, 5.0, -2.0)
        .expect("preview");
    let committed = session
        .commit_render_interaction_translation_v1(&gesture, &preview)
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
fn toggled_render_roots_keep_document_source_order() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let later = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "m2".to_owned(),
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select later root");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            Some(&later),
            RenderInteractionQueryV1::Root {
                identifier: "m1".to_owned(),
                modifier: RenderInteractionModifierV1::Toggle,
            },
        )
        .expect("toggle earlier root");

    assert_eq!(
        selection
            .roots()
            .iter()
            .map(|root| root.identifier())
            .collect::<Vec<_>>(),
        ["m1", "m2"]
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
    assert_eq!(commit.removed_atoms(), ["a"]);
    assert_eq!(commit.removed_bonds(), ["ab"]);
    let molecule = &commit.result().observation().projection().molecules()[0];
    assert_eq!(molecule.atoms().len(), 1);
    assert!(molecule.bonds().is_empty());
    assert!(matches!(
        session.commit_structure_deletion_v1(&selection),
        Err(RenderInteractionErrorV1::StaleRevision)
    ));
}

#[test]
fn view_hex_grid_policy_snaps_preview_delta_in_rust() {
    let session = RenderInteractionSessionV1::new(DocumentSession::load(SOURCE).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "m1".to_owned(),
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
    let molecule = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "molecule".to_owned(),
                modifier: RenderInteractionModifierV1::Replace,
            },
        )
        .expect("select molecule");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            Some(&molecule),
            RenderInteractionQueryV1::Root {
                identifier: "plus".to_owned(),
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
    let targets = selection
        .roots
        .iter()
        .map(|root| TopLevelRootSelectorV1::new(root.identifier.clone(), root.kind))
        .collect::<Result<Vec<_>, _>>()
        .expect("valid selected roots");
    let anchor = session
        .session
        .observe_top_level_translation_anchor_v1(selection.fence.revision(), targets)
        .expect("native mixed-root anchor");
    let (anchor_x, anchor_y) = anchor.anchor();
    let grid = HexGrid::new(
        VIEW_HEX_GRID_SPACING_PT_V1,
        Point2::new(0.0, 0.0).expect("finite grid origin"),
    )
    .expect("grid");
    let expected = grid
        .snap(
            Point2::new(
                anchor_x + release.0 - press.0,
                anchor_y + release.1 - press.1,
            )
            .expect("finite translated anchor"),
        )
        .expect("snap translated anchor");
    let legacy_release = grid
        .snap(Point2::new(release.0, release.1).expect("finite release"))
        .expect("snap release");
    let legacy_press = grid
        .snap(Point2::new(press.0, press.1).expect("finite press"))
        .expect("snap press");
    let expected_delta = (expected.x() - anchor_x, expected.y() - anchor_y);
    let legacy_delta = (
        legacy_release.x() - legacy_press.x(),
        legacy_release.y() - legacy_press.y(),
    );
    assert_ne!(expected_delta, legacy_delta);
    assert_eq!((preview.dx(), preview.dy()), expected_delta);
}

#[test]
fn view_hex_grid_exact_click_keeps_an_off_lattice_root_unchanged() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\"><atom id=\"atom\" name=\"C\"><point x=\"11\" y=\"17\"/></atom></molecule></cdml>";
    let mut session = RenderInteractionSessionV1::new(DocumentSession::load(source).expect("load"));
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("observe");
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "molecule".to_owned(),
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
        .commit_render_interaction_translation_v1(&gesture, &preview)
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
    assert_eq!(observation.exclusions()[0].identifier(), "m");
    assert_eq!(
        observation.exclusions()[0].reason(),
        RenderInteractionExclusionReasonV1::UnrenderableDepiction
    );
    assert!(matches!(
        session.select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "m".to_owned(),
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
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><fragment><bond id=\"m\"/></fragment></molecule></cdml>";
    let mut session = RenderInteractionSessionV1::new(
        DocumentSession::load(source).expect("fragment reference fixture loads"),
    );
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("fragment reference fixture observes");
    assert_eq!(observation.roots().len(), 1);
    let selection = session
        .select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: "m".to_owned(),
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
    let preview = session
        .preview_render_interaction_translation_v1(&gesture, 3.0, 0.0)
        .expect("preview move");
    assert_eq!(
        session
            .commit_render_interaction_translation_v1(&gesture, &preview)
            .expect("IDREF-safe move commits")
            .result()
            .observation()
            .snapshot()
            .revision(),
        1
    );
}

#[test]
fn idless_presentation_root_is_display_only_not_a_transform_target() {
    let session = RenderInteractionSessionV1::new(
        DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><plus><point x=\"4\" y=\"5\"/></plus></cdml>",
        )
        .expect("display-only fixture loads"),
    );
    let observation = session
        .observe_render_interaction_v1(fence(&session))
        .expect("display-only fixture observes");
    assert!(observation.roots().is_empty());
    let [exclusion] = observation.exclusions() else {
        panic!("idless root must have one diagnostic");
    };
    assert_eq!(
        exclusion.reason(),
        RenderInteractionExclusionReasonV1::DisplayOnly
    );
    assert!(matches!(
        session.select_render_interaction_roots_v1(
            &observation,
            None,
            RenderInteractionQueryV1::Root {
                identifier: exclusion.identifier().to_owned(),
                modifier: RenderInteractionModifierV1::Replace,
            },
        ),
        Err(RenderInteractionErrorV1::DisplayOnly)
    ));
}

#[test]
fn reaction_authoring_classifies_renderable_vectors_and_kind_mismatches() {
    assert_eq!(
        reaction_root_exclusion_reason(DirectCdmlRootKindV1::Other, TopLevelRootKindV1::Rectangle,),
        ReactionAuthoringExclusionReasonV1::DisplayOnly
    );
    assert_eq!(
        reaction_root_exclusion_reason(DirectCdmlRootKindV1::Arrow, TopLevelRootKindV1::Rectangle,),
        ReactionAuthoringExclusionReasonV1::KindMismatch
    );
    assert_eq!(
        reaction_exclusion_recovery(ReactionAuthoringExclusionReasonV1::KindMismatch),
        ReactionAuthoringExclusionRecoveryV1::RepairDocument
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
    let preview = session
        .preview_render_interaction_translation_v1(&gesture, 7.0, 4.0)
        .expect("preview mixed move");
    let committed = session
        .commit_render_interaction_translation_v1(&gesture, &preview)
        .expect("one mixed session operation");
    assert_eq!(committed.result().observation().snapshot().revision(), 1);
    let projection = committed.result().observation().projection();
    assert!((projection.molecules()[0].atoms()[0].position().x() - 7.0).abs() < 0.01);
    let PresentationRootProjectionV1::Plus { plus } = &projection.presentation_stack().roots()[0]
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
