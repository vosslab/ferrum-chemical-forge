//! Checked conversion between CDML centimetres and Ferrum scene points.
//!
//! CDML's physical coordinate convention is a document fact shared by every
//! adapter.  These newtypes keep the conversion policy in the geometry crate
//! while requiring each FFI or UI boundary to validate its scalar input.

use crate::GeometryError;

/// CDML's exact V1 scale: 72 PostScript points per inch and 2.54 cm per inch.
pub const CDML_POINTS_PER_CENTIMETRE_V1: f64 = 72.0 / 2.54;

/// One finite physical CDML length measured in centimetres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CdmlLength(f64);

impl CdmlLength {
    /// Construct a finite CDML centimetre length.
    pub fn try_from_centimetres(centimetres: f64) -> Result<Self, GeometryError> {
        if !centimetres.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self(centimetres))
    }

    /// Return the finite physical value in centimetres.
    #[must_use]
    pub const fn as_centimetres(self) -> f64 {
        self.0
    }

    /// Convert this length into finite Ferrum scene points.
    pub fn as_scene_points(self) -> Result<ScenePoints, GeometryError> {
        let points = self.0 * CDML_POINTS_PER_CENTIMETRE_V1;
        if !points.is_finite() {
            return Err(GeometryError::UnrepresentableGeometry);
        }
        Ok(ScenePoints(points))
    }
}

/// One finite Ferrum scene-space distance measured in PostScript points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScenePoints(f64);

impl ScenePoints {
    /// Construct a finite scene-point distance.
    pub fn try_from_scene_points(points: f64) -> Result<Self, GeometryError> {
        if !points.is_finite() {
            return Err(GeometryError::NonFiniteCoordinate);
        }
        Ok(Self(points))
    }

    /// Return the finite scene-point value.
    #[must_use]
    pub const fn as_scene_points(self) -> f64 {
        self.0
    }

    /// Convert this distance into finite physical CDML centimetres.
    pub fn as_centimetres(self) -> Result<CdmlLength, GeometryError> {
        let centimetres = self.0 / CDML_POINTS_PER_CENTIMETRE_V1;
        if !centimetres.is_finite() {
            return Err(GeometryError::UnrepresentableGeometry);
        }
        Ok(CdmlLength(centimetres))
    }
}
