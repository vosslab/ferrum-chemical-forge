//! Private whole-document native-artifact preparation for Ferrum.
//!
//! This owner prepares immutable bytes from one authenticated observation.  Qt
//! chooses and fences destinations; `artifact_publication_v1` owns the actual
//! descriptor-relative replacement protocol.

use std::{num::NonZeroU32, path::PathBuf};

use crate::{
    DocumentRenderObservationErrorV1, SessionDocumentObservationV1,
    artifact_publication_v1::{
        ArtifactPublicationErrorV1, ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1,
        RetainedSourceFileGuardV1, publish_artifact_v1,
    },
    derive_document_render_observation_from_accepted_operation_v1,
};
use ferrum_render::{
    DocumentRenderOutcomeV1, DocumentRenderPlanCompositionError, LOCAL_PDF_COMPLETED_BYTES_V1,
    LOCAL_PDF_DRAW_PATH_COMMANDS_V1, LOCAL_PDF_PLAN_ITEMS_V1, LOCAL_PNG_ENCODED_BYTES_V1,
    LOCAL_PNG_RAW_RGBA_BYTES_V1, LOCAL_SVG_COMPLETED_BYTES_V1, PdfRenderError, PngBackgroundV1,
    PngPixelSizeV1, PngRenderError, SvgOutputBudgetV1, SvgRenderError,
    compose_document_render_plan_v1, local_pdf_render_request_v1, local_png_render_request_v1,
    render_document_plan_to_pdf_v1, render_document_plan_to_png_v1,
    render_document_plan_to_svg_with_budget_v1,
};
use thiserror::Error;

/// Closed artifact profiles available to the ordinary native Qt export route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DocumentNativeArtifactProfileV1 {
    /// Complete whole-page SVG within the ordinary local SVG policy.
    Svg,
    /// Complete whole-page vector PDF within the ordinary local PDF policy.
    Pdf,
    /// Transparent PNG with one output pixel per Rust page point.
    PngOnePixelPerPointTransparent,
}

impl DocumentNativeArtifactProfileV1 {
    /// Return the stable format label used by the private adapter.
    #[must_use]
    pub const fn format_name(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Pdf => "pdf",
            Self::PngOnePixelPerPointTransparent => "png",
        }
    }
}

/// Immutable completed bytes bound to one observed document revision and digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDocumentNativeArtifactV1 {
    profile: DocumentNativeArtifactProfileV1,
    source_revision: u64,
    source_digest: [u8; 32],
    bytes: Vec<u8>,
}

impl PreparedDocumentNativeArtifactV1 {
    /// Return the closed artifact profile.
    #[must_use]
    pub const fn profile(&self) -> DocumentNativeArtifactProfileV1 {
        self.profile
    }

    /// Return the authenticated source revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Return the authenticated source digest.
    #[must_use]
    pub const fn source_digest(&self) -> &[u8; 32] {
        &self.source_digest
    }

    /// Return the completed immutable artifact bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

/// Prepare one complete native artifact from an exact immutable observation.
///
/// The observation is copied from its UI-affine session before this call.  The
/// request fence is checked before rendering, and no mutable session is touched.
///
/// # Errors
///
/// Returns a typed provenance, complete-plan, page-geometry, or sink-policy
/// failure before any destination publication begins.
pub fn prepare_document_native_artifact_v1(
    observation: &SessionDocumentObservationV1,
    expected_revision: u64,
    expected_digest: [u8; 32],
    profile: DocumentNativeArtifactProfileV1,
) -> Result<PreparedDocumentNativeArtifactV1, DocumentNativeArtifactErrorV1> {
    let snapshot = observation.snapshot();
    if snapshot.revision() != expected_revision || snapshot.digest() != &expected_digest {
        return Err(DocumentNativeArtifactErrorV1::ProvenanceMismatch);
    }
    let render_observation =
        derive_document_render_observation_from_accepted_operation_v1(observation)?;
    let plan = compose_document_render_plan_v1(render_observation.resolved())?;
    if plan
        .outcomes()
        .iter()
        .any(|outcome| matches!(outcome, DocumentRenderOutcomeV1::Exclusion(_)))
    {
        return Err(DocumentNativeArtifactErrorV1::ExcludedRoots);
    }

    let bytes = match profile {
        DocumentNativeArtifactProfileV1::Svg => {
            let budget = SvgOutputBudgetV1::new(LOCAL_SVG_COMPLETED_BYTES_V1)
                .map_err(DocumentNativeArtifactErrorV1::Svg)?;
            render_document_plan_to_svg_with_budget_v1(&plan, budget)
                .map_err(DocumentNativeArtifactErrorV1::Svg)?
                .into_artifact()
                .into_string()
                .into_bytes()
        }
        DocumentNativeArtifactProfileV1::Pdf => {
            let request = local_pdf_render_request_v1(
                LOCAL_PDF_COMPLETED_BYTES_V1,
                LOCAL_PDF_PLAN_ITEMS_V1,
                LOCAL_PDF_DRAW_PATH_COMMANDS_V1,
            )
            .map_err(DocumentNativeArtifactErrorV1::Pdf)?;
            render_document_plan_to_pdf_v1(&plan, request)
                .map_err(DocumentNativeArtifactErrorV1::Pdf)?
                .into_artifact()
                .into_bytes()
        }
        DocumentNativeArtifactProfileV1::PngOnePixelPerPointTransparent => {
            let pixels = page_pixels(plan.page().width(), plan.page().height())?;
            let request = local_png_render_request_v1(
                pixels,
                PngBackgroundV1::Transparent,
                LOCAL_PNG_RAW_RGBA_BYTES_V1,
                LOCAL_PNG_ENCODED_BYTES_V1,
            );
            render_document_plan_to_png_v1(&plan, request)
                .map_err(DocumentNativeArtifactErrorV1::Png)?
                .into_artifact()
                .into_bytes()
        }
    };

    Ok(PreparedDocumentNativeArtifactV1 {
        profile,
        source_revision: snapshot.revision(),
        source_digest: *snapshot.digest(),
        bytes,
    })
}

/// Publish a prepared receipt through the authoritative descriptor-relative owner.
///
/// An optional retained source guard rejects direct and observed hard-link aliases
/// of a locally admitted source.  Callers consume the receipt because a failed
/// publication may have reached the destination and must not be replayed blindly.
///
/// # Errors
///
/// Returns the publisher's typed not-started, invalid-destination, or
/// possibly-published outcome.
pub fn publish_prepared_document_native_artifact_v1(
    receipt: PreparedDocumentNativeArtifactV1,
    destination: PathBuf,
    retained_source: Option<RetainedSourceFileGuardV1>,
) -> Result<ArtifactPublicationOutcomeV1, ArtifactPublicationErrorV1> {
    let request = ArtifactPublicationRequestV1::new(destination, receipt.into_bytes());
    let request = match retained_source {
        Some(source) => request.with_retained_source(source),
        None => request,
    };
    publish_artifact_v1(request)
}

fn page_pixels(width: f64, height: f64) -> Result<PngPixelSizeV1, DocumentNativeArtifactErrorV1> {
    let width = page_dimension_to_pixels(width, "width")?;
    let height = page_dimension_to_pixels(height, "height")?;
    Ok(PngPixelSizeV1::new(width, height))
}

fn page_dimension_to_pixels(
    dimension: f64,
    axis: &'static str,
) -> Result<NonZeroU32, DocumentNativeArtifactErrorV1> {
    let rounded = dimension.ceil();
    if !rounded.is_finite() || rounded <= 0.0 || rounded > f64::from(u32::MAX) {
        return Err(DocumentNativeArtifactErrorV1::PageDimension { axis, dimension });
    }
    let pixels = rounded as u32;
    NonZeroU32::new(pixels).ok_or(DocumentNativeArtifactErrorV1::PageDimension { axis, dimension })
}

/// Failure while preparing an ordinary native document artifact.
#[derive(Debug, Error)]
pub enum DocumentNativeArtifactErrorV1 {
    /// The caller's revision/digest fence does not name this observation.
    #[error("native artifact source provenance did not match the requested revision and digest")]
    ProvenanceMismatch,
    /// The immutable observation could not produce Ferrum's closed render observation.
    #[error(transparent)]
    Observation(#[from] DocumentRenderObservationErrorV1),
    /// The immutable observation could not compose into one whole-page plan.
    #[error(transparent)]
    Composition(#[from] DocumentRenderPlanCompositionError),
    /// A normal complete artifact would omit one or more document roots.
    #[error("the render plan excluded one or more document roots")]
    ExcludedRoots,
    /// The page cannot be represented as one-pixel-per-point PNG dimensions.
    #[error("native PNG page {axis} {dimension} cannot become a positive u32 pixel dimension")]
    PageDimension { axis: &'static str, dimension: f64 },
    /// The SVG sink rejected the complete document or local profile.
    #[error(transparent)]
    Svg(#[from] SvgRenderError),
    /// The PDF sink rejected the complete document or local profile.
    #[error(transparent)]
    Pdf(#[from] PdfRenderError),
    /// The PNG sink rejected the complete document or local profile.
    #[error(transparent)]
    Png(#[from] PngRenderError),
}
