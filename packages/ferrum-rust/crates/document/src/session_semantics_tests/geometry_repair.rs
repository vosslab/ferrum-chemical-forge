use crate::{
    DocumentSession, DocumentSessionError, GeometryRepairKindV1, GeometryRepairV1,
    GeometryRepairV1Error, SessionOperation, SessionOperationError, SessionOperationV1,
    TypedDocumentError,
};

const HALF_AUTHORED_UNIT_POINTS: f64 = (0.001 * 72.0 / 2.54) / 2.0;

fn operation(repair: GeometryRepairV1) -> SessionOperation {
    SessionOperation::V1(SessionOperationV1::RepairGeometry { repair })
}

fn direct_molecule_id(
    session: &DocumentSession,
    revision: u64,
    index: usize,
) -> crate::DocumentObjectIdV1 {
    session
        .observe(revision)
        .expect("fixture observation")
        .projection()
        .molecules()[index]
        .id()
        .expect("fixture direct molecule has a durable ID")
        .clone()
}

#[test]
fn prepared_whole_depictions_keep_caller_order_complete_positions_and_angles() {
    let source = concat!(
        "<cdml><molecule id=\"first\"><atom id=\"z\" name=\"C\"><point x=\"1\" y=\"1\"/>",
        "</atom><atom id=\"a\" name=\"O\"><point x=\"0\" y=\"0\"/></atom>",
        "<bond id=\"first_bond\" start=\"z\" end=\"a\" type=\"n1\"/></molecule>",
        "<molecule id=\"second\"><atom id=\"b\" name=\"N\"><point x=\"0\" y=\"0\"/>",
        "</atom><atom id=\"c\" name=\"C\"><point x=\"0\" y=\"2\"/></atom>",
        "<bond id=\"second_bond\" start=\"b\" end=\"c\" type=\"n1\"/></molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    let first = direct_molecule_id(&session, 0, 0);
    let second = direct_molecule_id(&session, 0, 1);
    let prepared = session
        .prepare_straighten_depictions_v1(0, vec![second.clone(), first.clone()], false)
        .expect("preparation succeeds");
    assert_eq!(prepared.source_revision(), 0);
    assert_eq!(
        prepared.source_digest(),
        session.snapshot().expect("snapshot").digest()
    );
    assert_eq!(prepared.molecules()[0].molecule_id(), &second);
    assert_eq!(prepared.molecules()[1].molecule_id(), &first);
    assert_eq!(prepared.molecules()[0].positions().len(), 2);
    assert_eq!(prepared.molecules()[1].positions().len(), 2);
    let only_second = session
        .prepare_straighten_depictions_v1(0, vec![second], false)
        .expect("same source preparation succeeds");
    assert_eq!(
        prepared.molecules()[0].applied_rotation_radians(),
        only_second.molecules()[0].applied_rotation_radians()
    );
    let current = crate::TypedDocument::parse(source).expect("fixture parses");
    let mismatch = SessionOperation::V1(SessionOperationV1::ApplyPreparedStraightenDepictions {
        update: prepared,
    })
    .prepare(&current, 0, &[0; 32]);
    assert!(matches!(
        mismatch,
        Err(SessionOperationError::MoleculeCoordinateDigestMismatch)
    ));
}

#[test]
fn prepared_whole_depictions_are_revision_digest_bound_and_apply_as_one_history_entry() {
    let source = concat!(
        "<cdml><molecule id=\"first\" retained=\"yes\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\" z=\"7\"/></atom><atom id=\"b\" name=\"O\">",
        "<point x=\"1\" y=\"1\"/></atom><bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<fragment id=\"opaque\" retained=\"yes\"><vendor/></fragment></molecule>",
        "<molecule id=\"second\"><atom id=\"c\" name=\"N\"><point x=\"0\" y=\"0\"/>",
        "</atom><atom id=\"d\" name=\"C\"><point x=\"0\" y=\"2\"/></atom>",
        "<bond id=\"cd\" start=\"c\" end=\"d\" type=\"n1\"/></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let first = direct_molecule_id(&session, 0, 0);
    let second = direct_molecule_id(&session, 0, 1);
    let prepared = session
        .prepare_straighten_depictions_v1(0, vec![first, second], false)
        .expect("preparation succeeds");
    let applied = session
        .submit(
            0,
            SessionOperation::V1(SessionOperationV1::ApplyPreparedStraightenDepictions {
                update: prepared.clone(),
            }),
        )
        .expect("whole batch applies");
    assert_eq!(applied.observation().snapshot().revision(), 1);
    assert_eq!(
        applied.observation().projection().molecules()[0].atoms()[0]
            .position()
            .z(),
        7.0
    );
    assert!(applied
        .observation()
        .snapshot()
        .cdml()
        .contains("<vendor/>"));
    assert_eq!(
        session
            .undo(1)
            .expect("one batch history entry")
            .observation()
            .snapshot()
            .revision(),
        2
    );
    let before = session.snapshot().expect("snapshot");
    let stale = session
        .submit(
            2,
            SessionOperation::V1(SessionOperationV1::ApplyPreparedStraightenDepictions {
                update: prepared,
            }),
        )
        .expect_err("old provenance rejects without mutation");
    assert!(matches!(
        stale,
        DocumentSessionError::Operation(
            SessionOperationError::MoleculeCoordinateRevisionMismatch { .. }
        )
    ));
    let after = session.snapshot().expect("snapshot");
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.digest(), before.digest());
}

#[test]
fn whole_depiction_preparation_rejects_a_later_unsupported_target_without_mutation() {
    let source = concat!(
        "<cdml><molecule id=\"good\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom></molecule><molecule id=\"bad\"><group id=\"g\"><point x=\"1\" y=\"1\"/>",
        "</group></molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot");
    let error = session
        .prepare_straighten_depictions_v1(
            0,
            vec![
                direct_molecule_id(&session, 0, 0),
                direct_molecule_id(&session, 0, 1),
            ],
            true,
        )
        .expect_err("later unsupported target rejects the whole preparation");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::Candidate(
            TypedDocumentError::UnsupportedGeometryRepairMolecule(_)
        ))
    ));
    let after = session.snapshot().expect("snapshot");
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.digest(), before.digest());
}

#[test]
fn whole_depiction_preparation_rejects_duplicate_valid_target_without_mutation() {
    let source = concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><atom id=\"b\" name=\"O\"><point x=\"1\" y=\"1\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule>",
        "<molecule id=\"other\"><atom id=\"c\" name=\"N\"><point x=\"2\" y=\"2\"/>",
        "</atom></molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    let molecule = direct_molecule_id(&session, 0, 0);
    let before = session.snapshot().expect("snapshot");
    let error = session
        .prepare_straighten_depictions_v1(0, vec![molecule.clone(), molecule], true)
        .expect_err("duplicate valid molecule target rejects before planning");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::InvalidStraightenDepiction(detail))
            if detail.contains("must be unique")
    ));
    let after = session.snapshot().expect("snapshot");
    assert_eq!(after, before);
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.digest(), before.digest());
}

#[test]
fn prepared_whole_depiction_accepts_fused_topology() {
    let source = concat!(
        "<cdml><molecule id=\"fused\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><atom id=\"b\" name=\"C\"><point x=\"1\" y=\"0\"/></atom>",
        "<atom id=\"c\" name=\"C\"><point x=\"1\" y=\"1\"/></atom>",
        "<atom id=\"d\" name=\"C\"><point x=\"0\" y=\"1\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<bond id=\"bc\" start=\"b\" end=\"c\" type=\"n1\"/>",
        "<bond id=\"ca\" start=\"c\" end=\"a\" type=\"n1\"/>",
        "<bond id=\"cd\" start=\"c\" end=\"d\" type=\"n1\"/>",
        "<bond id=\"da\" start=\"d\" end=\"a\" type=\"n1\"/></molecule></cdml>",
    );
    let session = DocumentSession::load(source).expect("fixture loads");
    let prepared = session
        .prepare_straighten_depictions_v1(0, vec![direct_molecule_id(&session, 0, 0)], true)
        .expect("whole-depiction straightening supports fused topology");
    assert_eq!(prepared.molecules()[0].positions().len(), 4);
}

#[test]
fn hex_snap_is_one_sparse_history_entry_and_preserves_unowned_content() {
    let source = concat!(
        "<cdml><molecule id=\"m\" retained=\"yes\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0.2\" y=\"0.2\" z=\"4\"/></atom>",
        "<atom id=\"b\" name=\"O\"><point x=\"0\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
        "<bond id=\"ab\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
        "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
        "<fragment id=\"richer\" type=\"linear_form\" retained=\"opaque\"><extension/>",
        "</fragment></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::SnapToHexGrid,
        1.0,
    )
    .expect("fixture request");
    let repaired = session
        .submit(0, operation(repair))
        .expect("repair succeeds");
    let atoms = repaired.observation().projection().molecules()[0].atoms();
    assert!(atoms[0].position().x().abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert!(atoms[0].position().y().abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert_eq!(atoms[0].position().z(), 4.0);
    assert_eq!(atoms[1].position().x(), 0.0);
    let cdml = repaired.observation().snapshot().cdml();
    assert!(!cdml.contains("id=\"owned\""));
    assert!(cdml.contains("id=\"richer\""));
    assert!(cdml.contains("retained=\"opaque\""));
    assert!(cdml.contains("retained=\"yes\""));

    let undone = session.undo(1).expect("repair is one history entry");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0]
            .position()
            .x(),
        0.2
    );

    let mut snapped = DocumentSession::load(concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
    ))
    .expect("already snapped fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::SnapToHexGrid,
        1.0,
    )
    .expect("fixture request");
    let unchanged = snapped
        .submit(0, operation(repair))
        .expect("already snapped repair succeeds");
    assert_eq!(unchanged.observation().snapshot().revision(), 0);
}

#[test]
fn repair_envelope_and_later_unsupported_target_are_atomic() {
    assert_eq!(
        GeometryRepairV1::new(
            vec!["m".to_owned(), "m".to_owned()],
            GeometryRepairKindV1::SnapToHexGrid,
            1.0,
        ),
        Err(GeometryRepairV1Error::DuplicateMolecule)
    );
    assert_eq!(
        GeometryRepairV1::new(
            vec!["m".to_owned()],
            GeometryRepairKindV1::SnapToHexGrid,
            0.0,
        ),
        Err(GeometryRepairV1Error::InvalidTargetSpacing)
    );
    let source = concat!(
        "<cdml><molecule id=\"good\"><atom id=\"a\" name=\"C\">",
        "<point x=\"0.2\" y=\"0.2\"/></atom></molecule>",
        "<molecule id=\"bad\"><group id=\"g\"><point x=\"1\" y=\"1\"/>",
        "</group></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session.snapshot().expect("snapshot");
    let repair = GeometryRepairV1::new(
        vec!["good".to_owned(), "bad".to_owned()],
        GeometryRepairKindV1::SnapToHexGrid,
        1.0,
    )
    .expect("structurally valid request");
    let error = session
        .submit(0, operation(repair))
        .expect_err("unsupported later target rejects every patch");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::Candidate(
            TypedDocumentError::UnsupportedGeometryRepairMolecule(_)
        ))
    ));
    let after = session.snapshot().expect("snapshot");
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.digest(), before.digest());

    let missing = GeometryRepairV1::new(
        vec!["missing".to_owned()],
        GeometryRepairKindV1::SnapToHexGrid,
        1.0,
    )
    .expect("structurally valid missing request");
    let error = session
        .submit(0, operation(missing))
        .expect_err("missing molecule is typed, not a panic");
    assert!(matches!(
        error,
        DocumentSessionError::Operation(SessionOperationError::Candidate(
            TypedDocumentError::UnknownGeometryRepairMolecule(_)
        ))
    ));
    let after_missing = session.snapshot().expect("snapshot");
    assert_eq!(after_missing.revision(), before.revision());
    assert_eq!(after_missing.digest(), before.digest());
}

#[test]
fn straighten_bonds_moves_only_terminal_endpoint_with_lexical_two_atom_anchor() {
    let half_slot = std::f64::consts::PI / 12.0;
    let source = format!(
        concat!(
            "<cdml><molecule id=\"m\" retained=\"yes\">",
            "<atom id=\"z\" name=\"O\"><point x=\"{}\" y=\"{}\" z=\"7\"/></atom>",
            "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "<bond id=\"az\" start=\"a\" end=\"z\" type=\"n1\"/>",
            "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
            "<bond id=\"az\"/><vertex id=\"a\"/><vertex id=\"z\"/>",
            "<property name=\"bond_length\" value=\"10\" type=\"IntType\"/></fragment>",
            "</molecule></cdml>"
        ),
        half_slot.cos(),
        -half_slot.sin(),
    );
    let mut session = DocumentSession::load(&source).expect("fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::StraightenBonds,
        777.0,
    )
    .expect("common envelope validates unused spacing");
    let repaired = session
        .submit(0, operation(repair))
        .expect("terminal repair succeeds");
    let atoms = repaired.observation().projection().molecules()[0].atoms();
    assert!((atoms[0].position().x() - 3.0_f64.sqrt() / 2.0).abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert!((atoms[0].position().y() + 0.5).abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert_eq!(atoms[0].position().z(), 7.0);
    assert_eq!(atoms[1].position().x(), 0.0);
    assert_eq!(atoms[1].position().y(), 0.0);
    let cdml = repaired.observation().snapshot().cdml();
    assert!(!cdml.contains("id=\"owned\""));
    assert!(cdml.contains("retained=\"yes\""));

    let undone = session.undo(1).expect("repair is one history entry");
    assert_eq!(
        undone.observation().projection().molecules()[0].atoms()[0]
            .position()
            .x(),
        half_slot.cos()
    );
}

#[test]
fn normalize_lengths_preserves_directions_and_authored_content() {
    let source = concat!(
        "<cdml xmlns:v=\"urn:vendor\"><molecule id=\"m\" retained=\"yes\">",
        "<atom id=\"a\" name=\"C\"><point x=\"-20\" y=\"0\" z=\"5\"/>",
        "<v:note>keep</v:note></atom>",
        "<atom id=\"b\" name=\"N\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"c\" name=\"O\"><point x=\"0\" y=\"30\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<bond id=\"bc\" start=\"b\" end=\"c\" type=\"n1\"/>",
        "<fragment id=\"owned\" type=\"linear_form\"><name>linear_form</name>",
        "<bond id=\"ab\"/><bond id=\"bc\"/><vertex id=\"a\"/><vertex id=\"b\"/>",
        "<vertex id=\"c\"/><property name=\"bond_length\" value=\"10\" ",
        "type=\"IntType\"/></fragment></molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::NormalizeBondLengths,
        10.0,
    )
    .expect("length repair request validates");
    let repaired = session
        .submit(0, operation(repair))
        .expect("length normalization succeeds");
    let atoms = repaired.observation().projection().molecules()[0].atoms();
    assert!((atoms[0].position().x() + 10.0).abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert_eq!(atoms[0].position().y(), 0.0);
    assert_eq!(atoms[0].position().z(), 5.0);
    assert_eq!(
        (atoms[1].position().x(), atoms[1].position().y()),
        (0.0, 0.0)
    );
    assert_eq!(atoms[2].position().x(), 0.0);
    assert!((atoms[2].position().y() - 10.0).abs() <= HALF_AUTHORED_UNIT_POINTS);
    let cdml = repaired.observation().snapshot().cdml();
    assert!(!cdml.contains("id=\"owned\""));
    assert!(cdml.contains("<v:note>keep</v:note>"));
    assert!(cdml.contains("retained=\"yes\""));
    assert_eq!(
        session
            .undo(1)
            .expect("one history entry")
            .observation()
            .snapshot()
            .revision(),
        2
    );

    let mut canonical = DocumentSession::load(concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><atom id=\"b\" name=\"O\"><point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
    ))
    .expect("canonical fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::NormalizeBondLengths,
        10.0,
    )
    .expect("canonical request validates");
    let unchanged = canonical
        .submit(0, operation(repair))
        .expect("no-op succeeds");
    assert_eq!(unchanged.observation().snapshot().revision(), 0);
}

#[test]
fn normalize_ring_preserves_centroid_side_length_and_substituent_geometry() {
    let source = concat!(
        "<cdml xmlns:v=\"urn:vendor\"><molecule id=\"m\" retained=\"yes\">",
        "<atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"b\" name=\"C\"><point x=\"20\" y=\"0\"/></atom>",
        "<atom id=\"c\" name=\"C\"><point x=\"15\" y=\"10\"/></atom>",
        "<atom id=\"d\" name=\"C\"><point x=\"0\" y=\"10\"/></atom>",
        "<atom id=\"side\" name=\"O\"><point x=\"-10\" y=\"10\" z=\"6\"/>",
        "<v:note>keep</v:note></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/>",
        "<bond id=\"bc\" start=\"b\" end=\"c\" type=\"n1\"/>",
        "<bond id=\"cd\" start=\"c\" end=\"d\" type=\"n1\"/>",
        "<bond id=\"da\" start=\"d\" end=\"a\" type=\"n1\"/>",
        "<bond id=\"ds\" start=\"d\" end=\"side\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let before = session
        .observe(0)
        .expect("observation")
        .projection()
        .molecules()[0]
        .atoms()
        .iter()
        .map(|atom| (atom.source_id().unwrap().to_owned(), atom.position()))
        .collect::<std::collections::HashMap<_, _>>();
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::NormalizeRings,
        20.0,
    )
    .expect("ring repair request validates");
    let repaired = session
        .submit(0, operation(repair))
        .expect("ring repair succeeds");
    let after = repaired.observation().projection().molecules()[0]
        .atoms()
        .iter()
        .map(|atom| (atom.source_id().unwrap().to_owned(), atom.position()))
        .collect::<std::collections::HashMap<_, _>>();
    let ring = ["a", "b", "c", "d"];
    let before_centroid = ring.iter().fold((0.0, 0.0), |(x, y), id| {
        (x + before[*id].x() / 4.0, y + before[*id].y() / 4.0)
    });
    let after_centroid = ring.iter().fold((0.0, 0.0), |(x, y), id| {
        (x + after[*id].x() / 4.0, y + after[*id].y() / 4.0)
    });
    assert!((after_centroid.0 - before_centroid.0).abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert!((after_centroid.1 - before_centroid.1).abs() <= HALF_AUTHORED_UNIT_POINTS);
    for index in 0..ring.len() {
        let left = after[ring[index]];
        let right = after[ring[(index + 1) % ring.len()]];
        let distance = (left.x() - right.x()).hypot(left.y() - right.y());
        assert!((distance - 20.0).abs() <= 2.0 * HALF_AUTHORED_UNIT_POINTS);
    }
    assert_eq!(after["side"].z(), 6.0);
    assert!(
        ((after["side"].x() - before["side"].x()) - (after["d"].x() - before["d"].x())).abs()
            <= HALF_AUTHORED_UNIT_POINTS
    );
    assert!(repaired
        .observation()
        .snapshot()
        .cdml()
        .contains("<v:note>keep</v:note>"));
    assert_eq!(
        session
            .undo(1)
            .expect("one history entry")
            .observation()
            .snapshot()
            .revision(),
        2
    );

    let mut tree = DocumentSession::load(concat!(
        "<cdml><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/>",
        "</atom><atom id=\"b\" name=\"O\"><point x=\"10\" y=\"0\"/></atom>",
        "<bond id=\"ab\" start=\"a\" end=\"b\" type=\"n1\"/></molecule></cdml>",
    ))
    .expect("ring-free fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::NormalizeRings,
        20.0,
    )
    .expect("ring-free request validates");
    assert_eq!(
        tree.submit(0, operation(repair))
            .expect("ring-free repair is a no-op")
            .observation()
            .snapshot()
            .revision(),
        0
    );
}

#[test]
fn normalize_angles_uses_authored_order_and_preserves_non_coordinate_content() {
    let source = concat!(
        "<cdml xmlns:v=\"urn:vendor\"><molecule id=\"m\" retained=\"yes\">",
        "<atom id=\"root\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
        "<atom id=\"z_first\" name=\"N\"><point x=\"10\" y=\"1\" z=\"8\"/>",
        "<v:note>keep</v:note></atom>",
        "<atom id=\"a_second\" name=\"O\"><point x=\"10\" y=\"2\"/></atom>",
        "<bond id=\"z_first_bond\" start=\"root\" end=\"z_first\" type=\"n1\"/>",
        "<bond id=\"a_second_bond\" start=\"root\" end=\"a_second\" type=\"n1\"/>",
        "</molecule></cdml>",
    );
    let mut session = DocumentSession::load(source).expect("fixture loads");
    let repair = GeometryRepairV1::new(
        vec!["m".to_owned()],
        GeometryRepairKindV1::NormalizeBondAngles,
        20.0,
    )
    .expect("angle repair request validates");
    let repaired = session
        .submit(0, operation(repair))
        .expect("angle normalization succeeds");
    let atoms = repaired.observation().projection().molecules()[0]
        .atoms()
        .iter()
        .map(|atom| (atom.source_id().unwrap().to_owned(), atom.position()))
        .collect::<std::collections::HashMap<_, _>>();
    let first_distance = 10.0_f64.hypot(1.0);
    let second_distance = 10.0_f64.hypot(2.0);
    assert!((atoms["z_first"].x() - first_distance).abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert!(atoms["z_first"].y().abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert_eq!(atoms["z_first"].z(), 8.0);
    assert!((atoms["a_second"].x() - second_distance / 2.0).abs() <= HALF_AUTHORED_UNIT_POINTS);
    assert!(
        (atoms["a_second"].y() - second_distance * 3.0_f64.sqrt() / 2.0).abs()
            <= HALF_AUTHORED_UNIT_POINTS
    );
    assert_eq!((atoms["root"].x(), atoms["root"].y()), (0.0, 0.0));
    let cdml = repaired.observation().snapshot().cdml();
    assert!(cdml.contains("<v:note>keep</v:note>"));
    assert!(cdml.contains("retained=\"yes\""));
    assert_eq!(
        session
            .undo(1)
            .expect("one history entry")
            .observation()
            .snapshot()
            .revision(),
        2
    );
}
