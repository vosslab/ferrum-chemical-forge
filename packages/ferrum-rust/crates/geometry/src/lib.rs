//! Geometry primitives and arithmetic depiction utilities.
//!
//! [`Point2`] is the single Ferrum-owned coordinate representation. `kurbo` and
//! `nalgebra` are conversion-only dependencies at the renderer and numerical
//! boundaries, so their types never become persistent document facts.

mod hex_grid;
mod molecule_placement;
mod point;
mod straighten;
mod transform;
mod units;
mod wedge;

pub use hex_grid::{HexEdge, HexGrid, HexIndex};
pub use molecule_placement::{MoleculePlacementV1, place_molecule_depiction_v1};
pub use point::{GeometryError, Point2, Vector2};
pub use straighten::{StraightenedDepiction, straighten_depiction};
pub use transform::Transform2;
pub use units::{CDML_POINTS_PER_CENTIMETRE_V1, CdmlLength, ScenePoints};
pub use wedge::WedgeGeometry;

#[cfg(test)]
mod tests;
