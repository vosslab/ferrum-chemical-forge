use ferrum_core::RecordKind;

use crate::repair::{
    DepictionGraph, PatchPreconditionError, RepairKind, RepairRequest, plan_repair,
};

use super::fixtures::{id, point, vertex};

#[test]
fn patch_preconditions_reject_a_coordinate_changed_after_planning() {
    let atom_id = id(RecordKind::Atom, "a");
    let patch = plan_repair(&RepairRequest::new(
        DepictionGraph::new(vec![vertex("a", 0.2, 0.2)], vec![]).expect("graph must validate"),
        RepairKind::SnapToHexGrid {
            spacing: 1.0,
            origin: point(0.0, 0.0),
        },
    ))
    .expect("snap must be representable");
    assert_eq!(
        patch.validate_preconditions([(atom_id.clone(), point(0.2, 0.2))]),
        Ok(())
    );
    assert_eq!(
        patch.validate_preconditions([(atom_id.clone(), point(0.6, 0.2))]),
        Err(PatchPreconditionError::StaleCoordinate { atom_id })
    );
}
