//! Geometry primitives and arithmetic depiction utilities.
//!
//! [`Point2`] is the single Ferrum-owned coordinate representation. `kurbo` and
//! `nalgebra` are conversion-only dependencies at the renderer and numerical
//! boundaries, so their types never become persistent document facts.

mod hex_grid;
mod point;
mod straighten;
mod transform;
mod wedge;

pub use hex_grid::{HexEdge, HexGrid, HexIndex};
pub use point::{GeometryError, Point2, Vector2};
pub use straighten::{StraightenedDepiction, straighten_depiction};
pub use transform::Transform2;
pub use wedge::WedgeGeometry;

#[cfg(test)]
mod tests;
