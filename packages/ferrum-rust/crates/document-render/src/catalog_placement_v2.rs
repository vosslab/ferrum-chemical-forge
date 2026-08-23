//! Candidate-bound renderer overlays for shipped catalog placement.

use ferrum_catalog_placement::{
    CatalogPlacementCategoryV1, CatalogPlacementErrorV1, CatalogPlacementGestureV1,
    CatalogPlacementPreviewV1, CatalogPlacementRecoveryV1, CommittedCatalogPlacementV1,
    PreparedCatalogPlacementV1, begin_catalog_placement_v1, commit_catalog_placement_v1,
    prepare_catalog_placement_v1, preview_catalog_placement_v1,
};
use ferrum_document::{DocumentFenceV1, DocumentSession, PresentationGesturePoint2V1};
use ferrum_render::{
    DocumentRenderContentV1, DocumentRenderOutcomeV1, MoleculeRenderPlan, RenderRootOverlayV1,
    preview_root_render_overlay_v1,
};
use std::sync::{Arc, Mutex};

pub type CatalogPlacementCategoryV2 = CatalogPlacementCategoryV1;
pub type CatalogPlacementRecoveryV2 = CatalogPlacementRecoveryV1;
pub type CatalogPlacementErrorV2 = CatalogPlacementErrorV1;

#[derive(Clone, Debug)]
pub struct CatalogPlacementGestureV2 {
    inner: CatalogPlacementGestureV1,
    lease: Arc<Mutex<CatalogPreviewLeaseV2>>,
}

/// The renderer receipt is private, but lease ownership must remain shared by
/// every opaque handle for a gesture. A later pointer preview retires the
/// former candidate before it can be prepared. V1's authoring capability is
/// the sole authority for document mutation.
#[derive(Debug, Default)]
struct CatalogPreviewLeaseV2 {
    next: u64,
    active: Option<u64>,
    cancelled: bool,
}

/// A disposable renderer projection bound to one exact private candidate.
#[derive(Debug)]
pub struct CatalogPlacementPreviewV2 {
    overlay: RenderRootOverlayV1,
    prepared: Option<PreparedCatalogPlacementV1>,
    lease: Arc<Mutex<CatalogPreviewLeaseV2>>,
    lease_id: u64,
}

impl CatalogPlacementPreviewV2 {
    #[must_use]
    pub const fn overlay(&self) -> &RenderRootOverlayV1 {
        &self.overlay
    }

    #[must_use]
    pub fn molecule_plan(&self) -> Option<&MoleculeRenderPlan> {
        match self.overlay.content() {
            DocumentRenderContentV1::Molecule(plan) => Some(plan),
            DocumentRenderContentV1::Text(_) | DocumentRenderContentV1::Vector(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct PreparedCatalogPlacementV2 {
    inner: Option<PreparedCatalogPlacementV1>,
}

#[derive(Clone, Debug)]
pub struct CommittedCatalogPlacementV2(CommittedCatalogPlacementV1);

impl CommittedCatalogPlacementV2 {
    #[must_use]
    pub fn identifier(&self) -> &str {
        self.0.identifier()
    }

    #[must_use]
    pub fn result(&self) -> &ferrum_document::SessionOperationResultV1 {
        self.0.result()
    }
}

pub fn begin_catalog_placement_v2(
    session: &DocumentSession,
    fence: DocumentFenceV1,
    key: &str,
) -> Result<CatalogPlacementGestureV2, CatalogPlacementErrorV2> {
    begin_catalog_placement_v1(session, fence, key).map(|inner| CatalogPlacementGestureV2 {
        inner,
        lease: Arc::new(Mutex::new(CatalogPreviewLeaseV2::default())),
    })
}

/// Build once: V1's prepared receipt owns the candidate and complete renderer
/// plan before the selected root is exposed as a draw-only overlay.
pub fn preview_catalog_placement_v2(
    session: &mut DocumentSession,
    gesture: &CatalogPlacementGestureV2,
    anchor: PresentationGesturePoint2V1,
) -> Result<CatalogPlacementPreviewV2, CatalogPlacementErrorV2> {
    let lease_id = {
        let mut lease = gesture
            .lease
            .lock()
            .map_err(|_| CatalogPlacementErrorV1::ReplayedGesture)?;
        if lease.cancelled {
            return Err(CatalogPlacementErrorV1::ReplayedGesture);
        }
        lease.next = lease
            .next
            .checked_add(1)
            .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
        lease.active = Some(lease.next);
        lease.next
    };
    let preview: CatalogPlacementPreviewV1 =
        preview_catalog_placement_v1(session, &gesture.inner, anchor)?;
    let prepared = prepare_catalog_placement_v1(session, &gesture.inner, &preview)?;
    let plan = prepared
        .render_plan()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
    // Catalog V1 appends exactly one direct root at the end of the canonical
    // candidate. Use the identity issued by that composed plan rather than
    // reconstructing one from a catalog string; this also preserves a
    // projection-local identity if a compatible reader normalizes the root.
    let identity = match plan.outcomes().last() {
        Some(DocumentRenderOutcomeV1::Root(root)) => root.identity().clone(),
        Some(DocumentRenderOutcomeV1::Exclusion(_)) | None => {
            return Err(CatalogPlacementErrorV1::RenderPreparation);
        }
    };
    let overlay = preview_root_render_overlay_v1(plan, &identity)
        .map_err(|_| CatalogPlacementErrorV1::RenderPreparation)?;
    Ok(CatalogPlacementPreviewV2 {
        overlay,
        prepared: Some(prepared),
        lease: Arc::clone(&gesture.lease),
        lease_id,
    })
}

/// Consume the preview capability only; this path cannot recompile a candidate.
pub fn prepare_catalog_placement_v2(
    _session: &mut DocumentSession,
    gesture: &CatalogPlacementGestureV2,
    preview: &mut CatalogPlacementPreviewV2,
) -> Result<PreparedCatalogPlacementV2, CatalogPlacementErrorV2> {
    if !Arc::ptr_eq(&gesture.lease, &preview.lease) {
        return Err(CatalogPlacementErrorV1::MismatchedPreview);
    }
    let mut lease = preview
        .lease
        .lock()
        .map_err(|_| CatalogPlacementErrorV1::ReplayedGesture)?;
    if lease.cancelled || lease.active != Some(preview.lease_id) {
        return Err(CatalogPlacementErrorV1::ReplayedGesture);
    }
    let inner = preview
        .prepared
        .take()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
    lease.active = None;
    Ok(PreparedCatalogPlacementV2 { inner: Some(inner) })
}

pub fn commit_catalog_placement_v2(
    session: &mut DocumentSession,
    prepared: &mut PreparedCatalogPlacementV2,
) -> Result<CommittedCatalogPlacementV2, CatalogPlacementErrorV2> {
    let mut inner = prepared
        .inner
        .take()
        .ok_or(CatalogPlacementErrorV1::ReplayedGesture)?;
    match commit_catalog_placement_v1(session, &mut inner) {
        Ok(value) => Ok(CommittedCatalogPlacementV2(value)),
        Err(error) => {
            // V1 consumes its receipt only after the atomic document commit.
            // Every refusal leaves it valid, so V2 must return ownership.
            prepared.inner = Some(inner);
            Err(error)
        }
    }
}

pub fn release_catalog_placement_preview_v2(preview: &mut CatalogPlacementPreviewV2) {
    if let Ok(mut lease) = preview.lease.lock()
        && lease.active == Some(preview.lease_id)
    {
        lease.active = None;
    }
    preview.prepared = None;
}

pub fn cancel_catalog_placement_gesture_v2(gesture: CatalogPlacementGestureV2) {
    if let Ok(mut lease) = gesture.lease.lock() {
        lease.cancelled = true;
        lease.active = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fence(session: &DocumentSession) -> DocumentFenceV1 {
        let snapshot = session.snapshot().expect("snapshot");
        DocumentFenceV1::new(snapshot.revision(), *snapshot.digest())
    }

    const HOST_DOCUMENT: &str = r#"<cdml xmlns="urn:ferrum:cdml" version="26.07"><molecule id="host"><atom id="host-a" name="C"><point x="0" y="0"/></atom></molecule></cdml>"#;

    #[test]
    fn newer_preview_invalidates_the_previous_renderer_receipt() {
        let mut session = DocumentSession::load(HOST_DOCUMENT).expect("session");
        let gesture = begin_catalog_placement_v2(&session, fence(&session), "system/rings/benzene")
            .expect("gesture");
        let mut first = preview_catalog_placement_v2(
            &mut session,
            &gesture,
            PresentationGesturePoint2V1::new(10.0, 10.0).expect("point"),
        )
        .expect("first preview");
        let mut second = preview_catalog_placement_v2(
            &mut session,
            &gesture,
            PresentationGesturePoint2V1::new(20.0, 20.0).expect("point"),
        )
        .expect("second preview");
        assert!(matches!(
            prepare_catalog_placement_v2(&mut session, &gesture, &mut first),
            Err(CatalogPlacementErrorV1::ReplayedGesture)
        ));
        let mut prepared = prepare_catalog_placement_v2(&mut session, &gesture, &mut second)
            .expect("latest preview prepares");
        let committed = commit_catalog_placement_v2(&mut session, &mut prepared).expect("commit");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
    }

    #[test]
    fn release_and_cancel_are_nonmutating_and_terminal_for_their_handles() {
        let mut session = DocumentSession::load(HOST_DOCUMENT).expect("session");
        let gesture = begin_catalog_placement_v2(&session, fence(&session), "system/rings/benzene")
            .expect("gesture");
        let mut preview = preview_catalog_placement_v2(
            &mut session,
            &gesture,
            PresentationGesturePoint2V1::new(10.0, 10.0).expect("point"),
        )
        .expect("preview");
        release_catalog_placement_preview_v2(&mut preview);
        assert!(matches!(
            prepare_catalog_placement_v2(&mut session, &gesture, &mut preview),
            Err(CatalogPlacementErrorV1::ReplayedGesture)
        ));
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
        cancel_catalog_placement_gesture_v2(gesture);
        assert_eq!(session.snapshot().expect("snapshot").revision(), 0);
    }

    #[test]
    fn a_foreign_refusal_retains_the_v2_receipt_for_the_owner_retry() {
        let mut owner = DocumentSession::load(HOST_DOCUMENT).expect("owner");
        let mut foreign = DocumentSession::load(HOST_DOCUMENT).expect("foreign");
        let gesture = begin_catalog_placement_v2(&owner, fence(&owner), "system/rings/benzene")
            .expect("gesture");
        let mut preview = preview_catalog_placement_v2(
            &mut owner,
            &gesture,
            PresentationGesturePoint2V1::new(10.0, 10.0).expect("point"),
        )
        .expect("preview");
        let mut prepared =
            prepare_catalog_placement_v2(&mut owner, &gesture, &mut preview).expect("prepared");

        assert!(matches!(
            commit_catalog_placement_v2(&mut foreign, &mut prepared),
            Err(CatalogPlacementErrorV1::ForeignSession)
        ));
        assert_eq!(foreign.snapshot().expect("foreign snapshot").revision(), 0);
        let committed = commit_catalog_placement_v2(&mut owner, &mut prepared)
            .expect("owner retry retains receipt");
        assert_eq!(committed.result().observation().snapshot().revision(), 1);
    }

    #[test]
    fn a_stale_refusal_retains_the_v2_receipt_without_relabeling_it_replayed() {
        let mut session = DocumentSession::load(HOST_DOCUMENT).expect("session");
        let gesture = begin_catalog_placement_v2(&session, fence(&session), "system/rings/benzene")
            .expect("gesture");
        let mut preview = preview_catalog_placement_v2(
            &mut session,
            &gesture,
            PresentationGesturePoint2V1::new(10.0, 10.0).expect("point"),
        )
        .expect("preview");
        let mut prepared =
            prepare_catalog_placement_v2(&mut session, &gesture, &mut preview).expect("prepared");

        session
            .commit_complete_cdml_transaction_v1(fence(&session), HOST_DOCUMENT)
            .expect("external document change");
        assert!(matches!(
            commit_catalog_placement_v2(&mut session, &mut prepared),
            Err(CatalogPlacementErrorV1::StaleSnapshot)
        ));
        assert!(matches!(
            commit_catalog_placement_v2(&mut session, &mut prepared),
            Err(CatalogPlacementErrorV1::StaleSnapshot)
        ));
    }

    #[test]
    fn every_shipped_system_and_haworth_key_has_a_renderer_selected_root_preview() {
        for key in [
            "system/rings/benzene",
            "system/rings/cyclopropane",
            "system/rings/cyclobutane",
            "system/rings/cyclopentane",
            "system/rings/cyclohexane",
            "system/heterocycles/thiophene",
            "system/heterocycles/furan",
            "system/heterocycles/pyrrole",
            "system/heterocycles/purine",
            "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose",
            "biomolecules/carbohydrates/d-glucose/beta-d-glucopyranose",
            "biomolecules/carbohydrates/d-glucose/alpha-d-glucofuranose",
            "biomolecules/carbohydrates/d-glucose/beta-d-glucofuranose",
        ] {
            let mut session = DocumentSession::load(HOST_DOCUMENT).expect("session");
            let gesture = begin_catalog_placement_v2(&session, fence(&session), key)
                .expect("known shipped catalog key");
            let mut preview = preview_catalog_placement_v2(
                &mut session,
                &gesture,
                PresentationGesturePoint2V1::new(80.0, 60.0).expect("point"),
            )
            .expect("renderer-selected root preview");
            assert!(preview.molecule_plan().is_some(), "{key}");
            release_catalog_placement_preview_v2(&mut preview);
            assert_eq!(session.snapshot().expect("snapshot").revision(), 0, "{key}");
        }
    }
}
