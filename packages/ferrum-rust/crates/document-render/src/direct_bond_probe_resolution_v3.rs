//! Session-bound resolution of immutable direct-bond pointer evidence.

use ferrum_document::{
    DirectBondEndpointIntent, DirectBondPoint2V1, DocumentFenceV1, DocumentObjectIdV1,
    DocumentSession,
};

use crate::direct_bond_pointer_v3::{
    DirectBondPointerHitStateV3, DirectBondPointerProbeErrorV3, DirectBondPointerProbeV3,
};

const POINTER_PICK_TOLERANCE_PX_V1: f64 = 6.0;
const TIE_EPSILON_PX_SQUARED_V1: f64 = 1.0e-9;

fn require_fence(
    session: &DocumentSession,
    fence: DocumentFenceV1,
) -> Result<(), DirectBondPointerProbeErrorV3> {
    let snapshot = session
        .snapshot()
        .map_err(|_| DirectBondPointerProbeErrorV3::StaleRevision)?;
    if snapshot.revision() != fence.revision() {
        return Err(DirectBondPointerProbeErrorV3::StaleRevision);
    }
    if *snapshot.digest() != fence.digest() {
        return Err(DirectBondPointerProbeErrorV3::StaleDigest);
    }
    Ok(())
}

pub(crate) fn resolve_probe(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    probe: &DirectBondPointerProbeV3,
) -> Result<DirectBondEndpointIntent, DirectBondPointerProbeErrorV3> {
    require_fence(session, fence)?;
    let observation = session
        .observe(fence.revision())
        .map_err(|_| DirectBondPointerProbeErrorV3::StaleRevision)?;
    let atoms = observation
        .projection()
        .molecules()
        .iter()
        .flat_map(|molecule| molecule.atoms());
    match probe.direct_hit_state {
        DirectBondPointerHitStateV3::AmbiguousAtom => {
            Err(DirectBondPointerProbeErrorV3::AmbiguousAtom)
        }
        DirectBondPointerHitStateV3::UniqueAtom => {
            let direct_source_id = probe
                .direct_atom_source_id
                .as_ref()
                .expect("validated at construction");
            let mut matching_atoms =
                atoms.filter(|atom| atom.source_id() == Some(direct_source_id.as_str()));
            let Some(atom) = matching_atoms.next() else {
                return Err(DirectBondPointerProbeErrorV3::UnknownDirectAtom);
            };
            if matching_atoms.next().is_some() {
                return Err(DirectBondPointerProbeErrorV3::AmbiguousAtom);
            }
            let Some(atom) = atom.id() else {
                return Err(DirectBondPointerProbeErrorV3::UnknownDirectAtom);
            };
            Ok(DirectBondEndpointIntent::ExistingAtom { atom: atom.clone() })
        }
        DirectBondPointerHitStateV3::None => {
            let pointer_viewport = probe
                .viewport_to_scene
                .viewport_point_for(probe.scene_point)?;
            let mut closest: Option<(f64, DocumentObjectIdV1)> = None;
            let mut tied = false;
            for atom in atoms {
                let Some(atom_id) = atom.id() else { continue };
                let position = atom.position();
                let atom_point = DirectBondPoint2V1::new(position.x(), position.y())
                    .map_err(|_| DirectBondPointerProbeErrorV3::UnknownDirectAtom)?;
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
                return Err(DirectBondPointerProbeErrorV3::AmbiguousAtom);
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
