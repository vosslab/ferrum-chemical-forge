//! CLI ownership for bounded whole-document artifact rendering and publication.

use std::io::{Read, Write};
use std::num::NonZeroU32;
use std::path::Path;

use ferrum_document::DocumentSession;
use ferrum_document::artifact_publication_v1::{
    ArtifactPublicationOutcomeV1, ArtifactPublicationRequestV1, RetainedSourceFileGuardV1,
    publish_artifact_v1,
};
use ferrum_render::{PngBackgroundV1, PngPixelSizeV1, SvgOutputBudgetV1};

use crate::streams::{is_standard_stream, write_report};
use crate::{
    CliError, DocumentIngressOriginV1, DocumentPdfArtifactErrorV1, DocumentSvgArtifactErrorV1,
    load_document_file_for_publication_with_budget, load_document_reader_with_budget,
    local_cdml_ingress_format_v1, local_pdf_render_request_v1, local_png_render_request_v1,
    render_document_session_to_pdf_v1, render_document_session_to_png_v1,
    render_document_session_to_svg_v1,
};

pub(crate) struct PdfCliRenderPolicyV1 {
    pub(crate) max_output_bytes: usize,
    pub(crate) max_plan_items: usize,
    pub(crate) max_path_commands: usize,
}

pub(crate) struct PngCliRenderPolicyV1 {
    pub(crate) width: NonZeroU32,
    pub(crate) height: NonZeroU32,
    pub(crate) background: PngBackgroundV1,
    pub(crate) max_raw_rgba_bytes: usize,
    pub(crate) max_output_bytes: usize,
}

pub(crate) fn render_svg(
    input: &Path,
    output: &Path,
    max_output_bytes: usize,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    let output_budget =
        SvgOutputBudgetV1::new(max_output_bytes).map_err(DocumentSvgArtifactErrorV1::Render)?;
    let (session, retained_source) = load_render_input(input, stdin)?;
    let artifact = render_document_session_to_svg_v1(&session, 0, output_budget)?;
    let bytes = artifact.into_artifact().into_string().into_bytes();
    publish_or_write(output, bytes, retained_source, stdout, stderr)
}

pub(crate) fn render_pdf(
    input: &Path,
    output: &Path,
    policy: PdfCliRenderPolicyV1,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    let request = local_pdf_render_request_v1(
        policy.max_output_bytes,
        policy.max_plan_items,
        policy.max_path_commands,
    )
    .map_err(DocumentPdfArtifactErrorV1::Render)?;
    let (session, retained_source) = load_render_input(input, stdin)?;
    let artifact = render_document_session_to_pdf_v1(&session, 0, request)?;
    let bytes = artifact.into_artifact().into_bytes();
    publish_or_write(output, bytes, retained_source, stdout, stderr)
}

pub(crate) fn render_png(
    input: &Path,
    output: &Path,
    policy: PngCliRenderPolicyV1,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    let request = local_png_render_request_v1(
        PngPixelSizeV1::new(policy.width, policy.height),
        policy.background,
        policy.max_raw_rgba_bytes,
        policy.max_output_bytes,
    );
    let (session, retained_source) = load_render_input(input, stdin)?;
    let artifact = render_document_session_to_png_v1(&session, 0, request)?;
    let bytes = artifact.into_artifact().into_bytes();
    publish_or_write(output, bytes, retained_source, stdout, stderr)
}

fn load_render_input(
    input: &Path,
    stdin: &mut dyn Read,
) -> Result<(DocumentSession, Option<RetainedSourceFileGuardV1>), CliError> {
    if is_standard_stream(input) {
        return Ok((
            load_document_reader_with_budget(
                stdin,
                DocumentIngressOriginV1::StandardInput,
                local_cdml_ingress_format_v1(),
            )?,
            None,
        ));
    }
    let admitted =
        load_document_file_for_publication_with_budget(input, local_cdml_ingress_format_v1())?;
    let (session, retained_source) = admitted.into_parts();
    Ok((session, Some(retained_source)))
}

fn publish_or_write(
    output: &Path,
    bytes: Vec<u8>,
    retained_source: Option<RetainedSourceFileGuardV1>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    if is_standard_stream(output) {
        return write_report(&bytes, stdout);
    }
    let mut request = ArtifactPublicationRequestV1::new(output.to_path_buf(), bytes);
    if let Some(retained_source) = retained_source {
        request = request.with_retained_source(retained_source);
    }
    match publish_artifact_v1(request)? {
        ArtifactPublicationOutcomeV1::ConfirmedDurable(_) => Ok(()),
        ArtifactPublicationOutcomeV1::DirectoryEntryUnconfirmed(_) => stderr
            .write_all(
                b"ferrum: warning: artifact data was published, but directory-entry durability \
could not be confirmed\n",
            )
            .map_err(|source| CliError::Write {
                output: "standard error".to_owned(),
                source,
            }),
    }
}
