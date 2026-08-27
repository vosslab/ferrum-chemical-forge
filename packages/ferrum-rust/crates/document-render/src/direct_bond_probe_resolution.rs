//! Session-bound resolution of immutable direct-bond pointer evidence.

use ferrum_document::{
    DirectBondEndpointIntent, DirectBondPoint2V1, DocumentFenceV1, DocumentObjectIdV1,
    DocumentSession,
};

use crate::direct_bond_pointer::{
    DirectBondPointerHitState, DirectBondPointerProbe, DirectBondPointerProbeError,
};

const POINTER_PICK_TOLERANCE_PX_V1: f64 = 6.0;
const TIE_EPSILON_PX_SQUARED_V1: f64 = 1.0e-9;

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), DirectBondPointerProbeError> {
    let snapshot = session
        .snapshot()
        .map_err(|_| DirectBondPointerProbeError::StaleRevision)?;
    if snapshot.revision() != fence.revision() {
        return Err(DirectBondPointerProbeError::StaleRevision);
    }
    if *snapshot.digest() != fence.digest() {
        return Err(DirectBondPointerProbeError::StaleDigest);
    }
    Ok(())
}

pub(crate) fn resolve_probe(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    probe: &DirectBondPointerProbe,
) -> Result<DirectBondEndpointIntent, DirectBondPointerProbeError> {
    require_fence(session, fence)?;
    let observation = session
        .observe(fence.revision())
        .map_err(|_| DirectBondPointerProbeError::StaleRevision)?;
    let atoms = observation
        .projection()
        .molecules()
        .iter()
        .flat_map(|molecule| molecule.atoms());
    match probe.direct_hit_state {
        DirectBondPointerHitState::AmbiguousAtom => Err(DirectBondPointerProbeError::AmbiguousAtom),
        DirectBondPointerHitState::UniqueAtom => {
            let direct_object_id = probe
                .direct_atom_object_id
                .as_ref()
                .expect("validated at construction");
            let mut matching_atoms =
                atoms.filter(|atom| atom.document_object_id() == direct_object_id);
            let Some(atom) = matching_atoms.next() else {
                return Err(DirectBondPointerProbeError::UnknownDirectAtom);
            };
            if matching_atoms.next().is_some() {
                return Err(DirectBondPointerProbeError::AmbiguousAtom);
            }
            Ok(DirectBondEndpointIntent::ExistingAtom {
                atom: atom.document_object_id().clone(),
            })
        }
        DirectBondPointerHitState::None => {
            let pointer_viewport = probe
                .viewport_to_scene
                .viewport_point_for(probe.scene_point)?;
            let mut closest: Option<(f64, DocumentObjectIdV1)> = None;
            let mut tied = false;
            for atom in atoms {
                let atom_id = atom.document_object_id();
                let position = atom.position();
                let atom_point = DirectBondPoint2V1::new(position.x(), position.y())
                    .map_err(|_| DirectBondPointerProbeError::UnknownDirectAtom)?;
                let viewport = probe.viewport_to_scene.viewport_point_for(atom_point)?;
                let distance = (viewport.x() - pointer_viewport.x()).powi(2)
                    + (viewport.y() - pointer_viewport.y()).powi(2);
                if distance > POINTER_PICK_TOLERANCE_PX_V1.powi(2) {
                    continue;
                }
                match &mut closest {
                    None => closest = Some((distance, atom_id.clone())),
                    Some((best, _)) if (distance - *best).abs() <= TIE_EPSILON_PX_SQUARED_V1 => {
                        tied = true
                    }
                    Some((best, id)) if distance < *best => {
                        *best = distance;
                        *id = atom_id.clone();
                        tied = false;
                    }
                    Some(_) => {}
                }
            }
            if tied {
                return Err(DirectBondPointerProbeError::AmbiguousAtom);
            }
            Ok(match closest {
                Some((_, atom)) => DirectBondEndpointIntent::ExistingAtom { atom },
                None => DirectBondEndpointIntent::NewAtomAt {
                    raw_point: probe.scene_point,
                },
            })
        }
    }
}
