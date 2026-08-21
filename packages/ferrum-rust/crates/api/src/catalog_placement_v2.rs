//! Thin transport facade for the candidate-bound catalog preview bridge.

use ferrum_document::{DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1};
pub use ferrum_document_render::{
    CatalogPlacementCategoryV2, CatalogPlacementErrorV2, CatalogPlacementRecoveryV2,
    CommittedCatalogPlacementV2,
};
use ferrum_document_render::{
    CatalogPlacementGestureV2, CatalogPlacementPreviewV2, PreparedCatalogPlacementV2,
};

#[derive(Clone, Debug)]
pub struct ApiCatalogPlacementGestureV2(CatalogPlacementGestureV2);
#[derive(Debug)]
pub struct ApiCatalogPlacementPreviewV2(CatalogPlacementPreviewV2);
#[derive(Debug)]
pub struct ApiCatalogPlacementPreparedV2(PreparedCatalogPlacementV2);

impl ApiCatalogPlacementPreviewV2 {
    #[must_use]
    pub fn molecule_plan(&self) -> Option<&ferrum_render::MoleculeRenderPlan> {
        self.0.molecule_plan()
    }
    #[must_use]
    pub const fn source_order(&self) -> u32 {
        self.0.overlay().source_order()
    }
}

pub fn begin_api_catalog_placement_v2(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    key: &str,
) -> Result<ApiCatalogPlacementGestureV2, CatalogPlacementErrorV2> {
    ferrum_document_render::begin_catalog_placement_v2(session, fence, key)
        .map(ApiCatalogPlacementGestureV2)
}
pub fn preview_api_catalog_placement_v2(
    session: &mut DocumentSession,
    gesture: &ApiCatalogPlacementGestureV2,
    anchor: PresentationGesturePoint2V1,
) -> Result<ApiCatalogPlacementPreviewV2, CatalogPlacementErrorV2> {
    ferrum_document_render::preview_catalog_placement_v2(session, &gesture.0, anchor)
        .map(ApiCatalogPlacementPreviewV2)
}
pub fn prepare_api_catalog_placement_v2(
    session: &mut DocumentSession,
    gesture: &ApiCatalogPlacementGestureV2,
    preview: &mut ApiCatalogPlacementPreviewV2,
) -> Result<ApiCatalogPlacementPreparedV2, CatalogPlacementErrorV2> {
    ferrum_document_render::prepare_catalog_placement_v2(session, &gesture.0, &mut preview.0)
        .map(ApiCatalogPlacementPreparedV2)
}
pub fn commit_api_catalog_placement_v2(
    session: &mut DocumentSession,
    prepared: &mut ApiCatalogPlacementPreparedV2,
) -> Result<CommittedCatalogPlacementV2, CatalogPlacementErrorV2> {
    ferrum_document_render::commit_catalog_placement_v2(session, &mut prepared.0)
}
pub fn release_api_catalog_placement_preview_v2(preview: &mut ApiCatalogPlacementPreviewV2) {
    ferrum_document_render::release_catalog_placement_preview_v2(&mut preview.0);
}

pub fn cancel_api_catalog_placement_gesture_v2(gesture: ApiCatalogPlacementGestureV2) {
    ferrum_document_render::cancel_catalog_placement_gesture_v2(gesture.0);
}
