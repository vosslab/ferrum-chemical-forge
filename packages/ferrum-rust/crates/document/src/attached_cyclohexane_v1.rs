//! Private, atom-sharing cyclohexane topology facts.
//!
//! This is deliberately only the WP-A1 hypothesis seam: it admits a resolved
//! direct atom and constructs the six-member geometry/candidate before any
//! document identity, history, or XML mutation is attempted.  A later session
//! capability will own those stateful concerns.

use thiserror::Error;

use crate::Point3V1;

const SIDE_LENGTH: f64 = 40.0;
const EPSILON: f64 = 1.0e-10;

/// Minimal resolved facts required to decide whether an atom can share C6 vertex zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AttachedCyclohexaneAnchorV1<'a> {
    pub(crate) position: Point3V1,
    pub(crate) element: &'a str,
    pub(crate) formal_charge: Option<i32>,
    pub(crate) explicit_hydrogens: Option<u16>,
    pub(crate) valence: Option<u16>,
    pub(crate) multiplicity: Option<u16>,
    pub(crate) incident_bonds: &'a [AttachedCyclohexaneIncidentBondV1],
}

/// The only pre-existing bond fact accepted by this closed topology.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachedCyclohexaneIncidentBondV1 {
    NormalSingle,
    Other,
}

/// One pointer-derived direction in document y-down coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachedCyclohexaneReleaseV1 {
    x: f64,
    y: f64,
}

impl AttachedCyclohexaneReleaseV1 {
    pub fn new(x: f64, y: f64) -> Result<Self, AttachedCyclohexaneErrorV1> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AttachedCyclohexaneErrorV1::InvalidPose);
        }
        Ok(Self { x, y })
    }
}

/// The closed proposed extension. Vertex zero is the existing anchor.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttachedCyclohexaneCandidateV1 {
    vertices: [Point3V1; 6],
    added_atoms: [Point3V1; 5],
    bonds: [AttachedCyclohexaneBondV1; 6],
}

impl AttachedCyclohexaneCandidateV1 {
    pub(crate) fn vertices(&self) -> &[Point3V1; 6] {
        &self.vertices
    }

    pub(crate) fn added_atoms(&self) -> &[Point3V1; 5] {
        &self.added_atoms
    }

    pub(crate) fn bonds(&self) -> &[AttachedCyclohexaneBondV1; 6] {
        &self.bonds
    }
}

/// An endpoint in the proposed candidate, before durable IDs exist.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AttachedCyclohexaneVertexV1 {
    Anchor,
    Added(u8),
}

/// A normal `n1` edge in the proposed shared-anchor cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AttachedCyclohexaneBondV1 {
    start: AttachedCyclohexaneVertexV1,
    end: AttachedCyclohexaneVertexV1,
}

impl AttachedCyclohexaneBondV1 {
    pub(crate) const fn start(self) -> AttachedCyclohexaneVertexV1 {
        self.start
    }

    pub(crate) const fn end(self) -> AttachedCyclohexaneVertexV1 {
        self.end
    }
}

/// Refusal before identity allocation or session mutation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AttachedCyclohexaneErrorV1 {
    #[error("cyclohexane attachment anchor is ineligible")]
    IneligibleAnchor,
    #[error("cyclohexane attachment pose is invalid")]
    InvalidPose,
}

/// Validate one direct anchor and construct its exact shared-anchor C6 candidate.
pub(crate) fn attached_cyclohexane_candidate_v1(
    anchor: AttachedCyclohexaneAnchorV1<'_>,
    release: AttachedCyclohexaneReleaseV1,
) -> Result<AttachedCyclohexaneCandidateV1, AttachedCyclohexaneErrorV1> {
    validate_anchor(anchor)?;
    let dx = release.x - anchor.position.x();
    let dy = release.y - anchor.position.y();
    let distance = dx.hypot(dy);
    if !distance.is_finite() || distance <= EPSILON {
        return Err(AttachedCyclohexaneErrorV1::InvalidPose);
    }
    let center_x = anchor.position.x() + SIDE_LENGTH * dx / distance;
    let center_y = anchor.position.y() + SIDE_LENGTH * dy / distance;
    let theta = dy.atan2(dx) + std::f64::consts::PI;
    let mut vertices = [anchor.position; 6];
    for (index, vertex) in vertices.iter_mut().enumerate() {
        let angle = theta + std::f64::consts::TAU * index as f64 / 6.0;
        *vertex = Point3V1::new(
            center_x + SIDE_LENGTH * angle.cos(),
            center_y + SIDE_LENGTH * angle.sin(),
            anchor.position.z(),
        )
        .map_err(|_| AttachedCyclohexaneErrorV1::InvalidPose)?;
    }
    // The existing document record is the authoritative shared vertex.  Do not
    // replace it with an almost-equal trigonometric reconstruction.
    vertices[0] = anchor.position;
    if !same_point(vertices[0], anchor.position) {
        return Err(AttachedCyclohexaneErrorV1::InvalidPose);
    }
    let added_atoms = [
        vertices[1],
        vertices[2],
        vertices[3],
        vertices[4],
        vertices[5],
    ];
    let bonds = [
        edge(
            AttachedCyclohexaneVertexV1::Anchor,
            AttachedCyclohexaneVertexV1::Added(0),
        ),
        edge(
            AttachedCyclohexaneVertexV1::Added(0),
            AttachedCyclohexaneVertexV1::Added(1),
        ),
        edge(
            AttachedCyclohexaneVertexV1::Added(1),
            AttachedCyclohexaneVertexV1::Added(2),
        ),
        edge(
            AttachedCyclohexaneVertexV1::Added(2),
            AttachedCyclohexaneVertexV1::Added(3),
        ),
        edge(
            AttachedCyclohexaneVertexV1::Added(3),
            AttachedCyclohexaneVertexV1::Added(4),
        ),
        edge(
            AttachedCyclohexaneVertexV1::Added(4),
            AttachedCyclohexaneVertexV1::Anchor,
        ),
    ];
    Ok(AttachedCyclohexaneCandidateV1 {
        vertices,
        added_atoms,
        bonds,
    })
}

fn edge(
    start: AttachedCyclohexaneVertexV1,
    end: AttachedCyclohexaneVertexV1,
) -> AttachedCyclohexaneBondV1 {
    AttachedCyclohexaneBondV1 { start, end }
}

fn validate_anchor(
    anchor: AttachedCyclohexaneAnchorV1<'_>,
) -> Result<(), AttachedCyclohexaneErrorV1> {
    if anchor.element != "C"
        || anchor.formal_charge.is_some_and(|charge| charge != 0)
        || anchor.valence.is_some()
        || anchor.multiplicity.is_some()
        || anchor.position.z().abs() > EPSILON
        || anchor
            .incident_bonds
            .iter()
            .any(|bond| !matches!(bond, AttachedCyclohexaneIncidentBondV1::NormalSingle))
    {
        return Err(AttachedCyclohexaneErrorV1::IneligibleAnchor);
    }
    let occupancy =
        anchor.incident_bonds.len() + usize::from(anchor.explicit_hydrogens.unwrap_or(0));
    if occupancy > 2 {
        return Err(AttachedCyclohexaneErrorV1::IneligibleAnchor);
    }
    Ok(())
}

fn same_point(left: Point3V1, right: Point3V1) -> bool {
    (left.x() - right.x()).abs() <= EPSILON
        && (left.y() - right.y()).abs() <= EPSILON
        && (left.z() - right.z()).abs() <= EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLE: AttachedCyclohexaneIncidentBondV1 =
        AttachedCyclohexaneIncidentBondV1::NormalSingle;
    const OTHER: AttachedCyclohexaneIncidentBondV1 = AttachedCyclohexaneIncidentBondV1::Other;

    fn point(x: f64, y: f64, z: f64) -> Point3V1 {
        Point3V1::new(x, y, z).expect("test point is finite")
    }

    fn anchor<'a>(
        position: Point3V1,
        incident_bonds: &'a [AttachedCyclohexaneIncidentBondV1],
    ) -> AttachedCyclohexaneAnchorV1<'a> {
        AttachedCyclohexaneAnchorV1 {
            position,
            element: "C",
            formal_charge: None,
            explicit_hydrogens: None,
            valence: None,
            multiplicity: None,
            incident_bonds,
        }
    }

    #[test]
    fn attached_c6_pose_probe_has_shared_anchor_equal_edges_and_stable_y_down_winding() {
        let directions = [
            (1.0, 0.0),
            (0.5, 0.866_025_403_784),
            (-0.5, 0.866_025_403_784),
            (-1.0, 0.0),
            (-0.5, -0.866_025_403_784),
            (0.5, -0.866_025_403_784),
        ];
        for (x, y) in directions {
            let anchor_point = point(0.0, 0.0, 0.0);
            let candidate = attached_cyclohexane_candidate_v1(
                anchor(anchor_point, &[]),
                AttachedCyclohexaneReleaseV1::new(40.0 * x, 40.0 * y).expect("finite release"),
            )
            .expect("neutral carbon accepts");
            let vertices = candidate.vertices();
            assert_eq!(vertices[0], anchor_point);
            assert_eq!(candidate.added_atoms().len(), 5);
            assert_eq!(candidate.bonds().len(), 6);
            assert!(vertices
                .iter()
                .all(|vertex| vertex.x().is_finite() && vertex.y().is_finite()));
            for (index, start) in vertices.iter().enumerate() {
                let end = vertices[(index + 1) % vertices.len()];
                assert!(
                    ((start.x() - end.x()).hypot(start.y() - end.y()) - SIDE_LENGTH).abs()
                        < EPSILON
                );
                assert!(vertices
                    .iter()
                    .skip(index + 1)
                    .all(|other| !same_point(*start, *other)));
            }
            let center_x = vertices.iter().map(|vertex| vertex.x()).sum::<f64>() / 6.0;
            let center_y = vertices.iter().map(|vertex| vertex.y()).sum::<f64>() / 6.0;
            assert!((center_x - SIDE_LENGTH * x).abs() < 1.0e-9);
            assert!((center_y - SIDE_LENGTH * y).abs() < 1.0e-9);
            let signed_turn = (vertices[1].x() - vertices[0].x())
                * (vertices[2].y() - vertices[1].y())
                - (vertices[1].y() - vertices[0].y()) * (vertices[2].x() - vertices[1].x());
            assert!(
                signed_turn > 0.0,
                "in y-down coordinates increasing angle is clockwise"
            );
        }
        assert_eq!(
            attached_cyclohexane_candidate_v1(
                anchor(point(0.0, 0.0, 0.0), &[]),
                AttachedCyclohexaneReleaseV1::new(0.0, 0.0).expect("finite pointer"),
            ),
            Err(AttachedCyclohexaneErrorV1::InvalidPose)
        );
        assert_eq!(
            AttachedCyclohexaneReleaseV1::new(f64::NAN, 0.0),
            Err(AttachedCyclohexaneErrorV1::InvalidPose)
        );
    }

    #[test]
    fn attached_c6_admission_probe_is_closed_and_has_no_identity_or_session_state() {
        let accepted = [0, 1, 2].map(|count| {
            let bonds = vec![SINGLE; count];
            attached_cyclohexane_candidate_v1(
                anchor(point(0.0, 0.0, 0.0), &bonds),
                AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("finite pointer"),
            )
        });
        for candidate in accepted {
            let candidate = candidate.expect("zero through two n1 bonds accept");
            assert_eq!(candidate.added_atoms().len(), 5);
            assert_eq!(candidate.bonds().len(), 6);
            assert_eq!(
                candidate.bonds()[0].start(),
                AttachedCyclohexaneVertexV1::Anchor
            );
            assert_eq!(
                candidate.bonds()[5].end(),
                AttachedCyclohexaneVertexV1::Anchor
            );
        }
        let invalid = [
            anchor(point(0.0, 0.0, 0.0), &[SINGLE, SINGLE, SINGLE]),
            anchor(point(0.0, 0.0, 0.0), &[OTHER]),
            AttachedCyclohexaneAnchorV1 {
                explicit_hydrogens: Some(3),
                ..anchor(point(0.0, 0.0, 0.0), &[])
            },
            AttachedCyclohexaneAnchorV1 {
                formal_charge: Some(1),
                ..anchor(point(0.0, 0.0, 0.0), &[])
            },
            AttachedCyclohexaneAnchorV1 {
                element: "N",
                ..anchor(point(0.0, 0.0, 0.0), &[])
            },
            AttachedCyclohexaneAnchorV1 {
                position: point(0.0, 0.0, 1.0),
                ..anchor(point(0.0, 0.0, 0.0), &[])
            },
        ];
        for rejected in invalid {
            assert_eq!(
                attached_cyclohexane_candidate_v1(
                    rejected,
                    AttachedCyclohexaneReleaseV1::new(40.0, 0.0).expect("finite pointer"),
                ),
                Err(AttachedCyclohexaneErrorV1::IneligibleAnchor)
            );
        }
    }
}
