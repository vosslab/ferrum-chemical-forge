//! Versioned resource policy for ordinary local, uncompressed CDML operations.
//!
//! This profile is an operational desktop/CLI allocation envelope, not a file-
//! format validity rule or a promise that every historical document fits. A
//! future population or resource model can add a new profile without silently
//! changing the meaning of V1.

use std::path::Path;

use ferrum_document::{DocumentSession, XmlInputBudgetV1};
use ferrum_render::{
    PdfOutputBudgetV1, PdfPlanComplexityBudgetV1, PdfRenderError, PdfRenderRequestV1,
    PngBackgroundV1, PngOutputBudgetV1, PngPixelSizeV1, PngRenderRequestV1,
};

use crate::{
    CdmlIngressBudgetV1, DocumentIngressErrorV1, DocumentIngressFormatV1,
    load_document_file_with_budget,
};

/// Stable identifier for the first ordinary local-CDML admission profile.
pub const LOCAL_CDML_INGRESS_PROFILE_V1: &str = "ferrum-local-cdml-ingress-v1";

/// Maximum completed SVG bytes returned by the first local render profile.
pub const LOCAL_SVG_COMPLETED_BYTES_V1: usize = 64 * 1024 * 1024;

/// Stable identifier for the first ordinary local vector-PDF render policy.
pub const LOCAL_PDF_RENDER_PROFILE_V1: &str = "ferrum-local-pdf-render-v1";

/// Maximum completed PDF bytes under the ordinary local V1 policy.
pub const LOCAL_PDF_COMPLETED_BYTES_V1: usize = 64 * 1024 * 1024;

/// Maximum counted PDF traversal items under the ordinary local V1 policy.
pub const LOCAL_PDF_PLAN_ITEMS_V1: usize = 1024 * 1024;

/// Maximum lowered PDF path commands under the ordinary local V1 policy.
pub const LOCAL_PDF_DRAW_PATH_COMMANDS_V1: usize = 8 * 1024 * 1024;

/// Stable identifier for the first ordinary local raster-PNG render policy.
pub const LOCAL_PNG_RENDER_PROFILE_V1: &str = "ferrum-local-png-render-v1";

/// Maximum pre-allocation RGBA bytes under the ordinary local V1 policy.
pub const LOCAL_PNG_RAW_RGBA_BYTES_V1: usize = 256 * 1024 * 1024;

/// Maximum completed PNG bytes under the ordinary local V1 policy.
pub const LOCAL_PNG_ENCODED_BYTES_V1: usize = 64 * 1024 * 1024;

/// Return the immutable V1 policy for uncompressed local CDML or bounded stdin.
///
/// The 16 MiB source envelope is the dominant allocation guard. Independent
/// structural ceilings stop compact adversarial XML from trading source bytes
/// for excessive tree fan-out, attributes, text retention, or parser depth.
/// CD-SVG and compressed containers deliberately require separate profiles.
#[must_use]
pub const fn local_cdml_ingress_format_v1() -> DocumentIngressFormatV1 {
    DocumentIngressFormatV1::Cdml(CdmlIngressBudgetV1 {
        xml: XmlInputBudgetV1 {
            max_utf8_bytes: 16 * 1024 * 1024,
            max_elements: 262_144,
            max_depth: 64,
            max_attributes: 1_048_576,
            max_text_bytes: 8 * 1024 * 1024,
        },
    })
}

/// Admit one ordinary local, uncompressed CDML file through the immutable V1 profile.
///
/// This is the product-facing file boundary for desktop and other local callers.
/// Callers choose the path, while Rust owns the complete named resource policy.
///
/// # Errors
///
/// Returns [`DocumentIngressErrorV1`] when the path, source bytes, XML, typed CDML,
/// or revision-zero session does not satisfy the profile.
pub fn load_local_cdml_file_v1(path: &Path) -> Result<DocumentSession, DocumentIngressErrorV1> {
    load_document_file_with_budget(path, local_cdml_ingress_format_v1())
}

/// Build the complete ordinary local PDF policy from explicit caller caps.
///
/// The complete-plan API refuses exclusions before PDF preflight, so exclusion
/// report bytes are required to remain zero. Callers may select stricter or
/// broader output and work caps without changing the named default profile.
///
/// # Errors
///
/// Returns [`PdfRenderError::InvalidOutputBudget`] for a zero completed cap.
pub fn local_pdf_render_request_v1(
    max_completed_bytes: usize,
    max_plan_items: usize,
    max_draw_path_commands: usize,
) -> Result<PdfRenderRequestV1, PdfRenderError> {
    Ok(PdfRenderRequestV1 {
        output: PdfOutputBudgetV1::new(max_completed_bytes)?,
        complexity: PdfPlanComplexityBudgetV1 {
            max_plan_items,
            max_draw_path_commands,
            max_exclusion_report_bytes: 0,
        },
    })
}

/// Build one local PNG request from exact caller-owned raster facts and caps.
///
/// Pixel dimensions and background are artifact semantics, while the two byte
/// ceilings contain allocation before the pixmap and after native encoding.
#[must_use]
pub const fn local_png_render_request_v1(
    pixels: PngPixelSizeV1,
    background: PngBackgroundV1,
    max_raw_rgba_bytes: usize,
    max_encoded_bytes: usize,
) -> PngRenderRequestV1 {
    PngRenderRequestV1 {
        pixels,
        background,
        budget: PngOutputBudgetV1 {
            max_raw_rgba_bytes,
            max_encoded_bytes,
        },
    }
}
