//! Narrow renderer-admission snap outcome for complete-root translations.

use ferrum_geometry::{HexGrid, Point2};
use thiserror::Error;

use crate::TopLevelRootSelectorV1;

use super::*;

/// The renderer-visible snapped delta for one complete-root translation preview.
///
/// The document owns the selected-root validation and authored anchor calculation.
/// The renderer supplies its view grid and receives only the resulting movement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RendererTranslationSnapDeltaV1 {
    dx: f64,
    dy: f64,
}

impl RendererTranslationSnapDeltaV1 {
    #[must_use]
    pub const fn dx(self) -> f64 {
        self.dx
    }

    #[must_use]
    pub const fn dy(self) -> f64 {
        self.dy
    }
}

/// Why document-owned translation snapping cannot issue a renderer delta.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RendererTranslationSnapRefusalV1 {
    #[error("translation gesture revision is stale")]
    StaleRevision,
    #[error("translation selection is not a complete transformable root set")]
    Selection,
    #[error("translation delta must remain finite")]
    NonFiniteDelta,
    #[error("renderer grid cannot snap the translation anchor")]
    Grid,
}

impl DocumentSession {
    /// Calculate the renderer grid snap for one exact complete-root selection.
    ///
    /// The document retains the authored anchor and all selection validation. The
    /// explicit renderer-admission namespace exposes only the snapped movement,
    /// so no outer layer can reuse source geometry or fabricate an anchor receipt.
    pub fn snap_top_level_translation_for_renderer_v1(
        &self,
        expected_revision: u64,
        targets: Vec<TopLevelRootSelectorV1>,
        raw_dx: f64,
        raw_dy: f64,
        grid: HexGrid,
    ) -> Result<RendererTranslationSnapDeltaV1, RendererTranslationSnapRefusalV1> {
        self.require_current(expected_revision)
            .map_err(map_document_refusal)?;
        let (anchor_x, anchor_y) = self
            .current_state_v1()
            .document()
            .top_level_translation_anchor_v1(targets)
            .map_err(|_| RendererTranslationSnapRefusalV1::Selection)?;
        let translated = Point2::new(anchor_x + raw_dx, anchor_y + raw_dy)
            .map_err(|_| RendererTranslationSnapRefusalV1::NonFiniteDelta)?;
        let snapped = grid
            .snap(translated)
            .map_err(|_| RendererTranslationSnapRefusalV1::Grid)?;
        Ok(RendererTranslationSnapDeltaV1 {
            dx: snapped.x() - anchor_x,
            dy: snapped.y() - anchor_y,
        })
    }
}

fn map_document_refusal(error: DocumentSessionError) -> RendererTranslationSnapRefusalV1 {
    match error {
        DocumentSessionError::RevisionConflict { .. } => {
            RendererTranslationSnapRefusalV1::StaleRevision
        }
        _ => RendererTranslationSnapRefusalV1::Selection,
    }
}
