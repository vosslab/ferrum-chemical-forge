//! Private borrowed lowering stream shared by renderer backends.
//!
//! This module owns interpretation of the validated document plan. Sinks only
//! serialize or paint the already selected geometry and explicit V1 profiles.

use std::convert::Infallible;

use thiserror::Error;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

use crate::DocumentRenderCompositeV1;
use crate::authored_direct_glycosidic_haworth::{
    AuthoredDirectGlycosidicHaworthDrawOpV1, AuthoredDirectGlycosidicHaworthRenderPlanV1,
};
use crate::direct_glycosidic_haworth::DirectGlycosidicHaworthPathCommandV1;
use crate::draw_stream_molecule_v1::{lower_molecule_batch, lower_molecule_plan};
use crate::verified_telex_glyph_metrics::is_verified_outlineless_whitespace_glyph;
use crate::{
    BatchSpace, DocumentRenderContentV1, DocumentRenderOutcomeV1, DocumentRenderPlanV1,
    DocumentTextLayoutV1, DocumentTextOpV1, DocumentVectorOpV1, DocumentVectorRootV1,
    FerrumFontEnvironmentV1, FerrumFontId, GlyphPlacement, MoleculeRenderPlanV4, PathCommandV1,
    PositiveFinite, PresentationGlyphRun, PresentationTextOp, RenderPaintV3, RenderPoint,
    RenderTarget, RenderViewportV1, StrokeV1, TextOp, TextRun, VectorFillRuleV1,
    VectorStrokeLineCapV1, VectorStrokeLineJoinV1,
};

/// A private streaming backend for one already validated render plan.
pub(crate) trait DrawSinkV1 {
    type Error;

    fn begin_page(&mut self, page: RenderViewportV1) -> Result<(), Self::Error>;
    fn begin_root(
        &mut self,
        paint_order: u32,
        document_object_id: &ferrum_document_projection::DocumentObjectIdV1,
    ) -> Result<(), Self::Error>;
    fn begin_root_with_kind(
        &mut self,
        paint_order: u32,
        document_object_id: &ferrum_document_projection::DocumentObjectIdV1,
        _: DrawRootKindV1,
    ) -> Result<(), Self::Error> {
        self.begin_root(paint_order, document_object_id)
    }
    fn end_root(&mut self) -> Result<(), Self::Error>;
    fn begin_molecule_batch(
        &mut self,
        paint_order: u32,
        space: BatchSpace,
    ) -> Result<(), Self::Error>;
    fn begin_molecule_target_group(
        &mut self,
        target: &RenderTarget,
        paint_order: u32,
        space: BatchSpace,
    ) -> Result<(), Self::Error> {
        let _ = target;
        self.begin_molecule_batch(paint_order, space)
    }
    fn end_molecule_batch(&mut self) -> Result<(), Self::Error>;
    fn begin_direct_target_group(&mut self, _: u32) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_direct_target_group(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_document_text(&mut self) -> Result<(), Self::Error>;
    fn end_document_text(&mut self) -> Result<(), Self::Error>;
    fn begin_text_operation(&mut self, z: i32, paint: &RenderPaintV3) -> Result<(), Self::Error>;
    fn end_text_operation(&mut self) -> Result<(), Self::Error>;
    fn save(&mut self) -> Result<(), Self::Error>;
    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error>;
    fn restore(&mut self) -> Result<(), Self::Error>;
    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        paint: &RenderPaintV3,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error>;
    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error>;
    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error>;
    fn finish_page(&mut self) -> Result<(), Self::Error>;
}

/// Private source context issued only by document-plan lowering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawRootKindV1 {
    Molecule,
    Text,
    Vector,
}

fn root_kind(content: &DocumentRenderContentV1) -> DrawRootKindV1 {
    match content {
        DocumentRenderContentV1::Molecule(_) => DrawRootKindV1::Molecule,
        DocumentRenderContentV1::Text(_) => DrawRootKindV1::Text,
        DocumentRenderContentV1::Vector(_) => DrawRootKindV1::Vector,
    }
}

/// A finite rectangle whose extent is positive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawRectV1 {
    pub(crate) origin: RenderPoint,
    pub(crate) width: PositiveFinite,
    pub(crate) height: PositiveFinite,
}

/// One stream ellipse, including the molecule grammar's explicit rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawEllipseV1 {
    pub(crate) center: RenderPoint,
    pub(crate) radius_x: PositiveFinite,
    pub(crate) radius_y: PositiveFinite,
    pub(crate) rotation_degrees: f64,
}

/// One path command after text outline coordinates become scene coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum DrawPathCommandV1 {
    MoveTo(RenderPoint),
    LineTo(RenderPoint),
    QuadraticTo {
        control: RenderPoint,
        end: RenderPoint,
    },
    CubicTo {
        control_1: RenderPoint,
        control_2: RenderPoint,
        end: RenderPoint,
    },
    Close,
}

/// Borrowed commands for one immediately consumed path.
#[derive(Debug, PartialEq)]
pub(crate) struct DrawPathV1 {
    pub(crate) commands: Vec<DrawPathCommandV1>,
}

/// The complete V1 stroke profile; no sink obtains a toolkit default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawStrokeV1<'a> {
    pub(crate) paint: &'a RenderPaintV3,
    pub(crate) width: PositiveFinite,
    pub(crate) line_cap: DrawLineCapV1,
    pub(crate) line_join: VectorStrokeLineJoinV1,
    pub(crate) miter_limit: f64,
}

/// Private sink cap profile; public document-vector V1 stays butt-only.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawLineCapV1 {
    Butt,
    Round,
}

impl DrawLineCapV1 {
    pub(crate) const fn svg_keyword(self) -> &'static str {
        match self {
            Self::Butt => "butt",
            Self::Round => "round",
        }
    }
}

/// Explicit fill/stroke semantics for one primitive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawStyleV1<'a> {
    pub(crate) fill: Option<&'a RenderPaintV3>,
    pub(crate) stroke: Option<DrawStrokeV1<'a>>,
    pub(crate) fill_rule: Option<VectorFillRuleV1>,
}

/// SVG-only diagnostic tags retained without becoming a public DTO.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DrawMetadataV1 {
    MoleculeLine { z: i32 },
    MoleculePath { z: i32 },
    MoleculeMask { z: i32 },
    MoleculeEllipse { z: i32 },
    MoleculeText { z: i32 },
    DocumentTextBackground,
    DocumentVectorPath,
    DocumentVectorEllipse,
    DirectGlycosidicOrdinary,
    DirectGlycosidicQ1,
    DirectGlycosidicW1,
}

/// Typed failures before a sink publishes an owned artifact.
#[derive(Debug, Error)]
pub(crate) enum DrawStreamErrorV1<E> {
    #[error("could not allocate private draw geometry")]
    ResourceExhausted,
    #[error("derived render geometry is not finite")]
    NonFiniteGeometry,
    #[error("could not parse verified Telex outline face: {0}")]
    Font(String),
    #[error("required Telex glyph {glyph_index} has no usable outline")]
    MissingGlyphOutline { glyph_index: u32 },
    #[error("checked composite no longer matches its retained document plan")]
    #[cfg_attr(not(test), allow(dead_code))]
    InvalidComposite,
    #[error("draw sink refused the operation")]
    Sink(#[source] E),
}

/// Lower a whole page in direct-root paint order, omitting named exclusions.
pub(crate) fn lower_document_plan_to_sink_v1<S: DrawSinkV1>(
    plan: &DocumentRenderPlanV1,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let environment = FerrumFontEnvironmentV1::load()
        .map_err(|error| DrawStreamErrorV1::Font(error.to_string()))?;
    let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
    let face = Face::parse(descriptor.data(), 0)
        .map_err(|error| DrawStreamErrorV1::Font(error.to_string()))?;
    sink.begin_page(plan.page())
        .map_err(DrawStreamErrorV1::Sink)?;
    for outcome in plan.outcomes() {
        let DocumentRenderOutcomeV1::Root(root) = outcome else {
            continue;
        };
        sink.begin_root_with_kind(
            root.paint_order(),
            root.target().document_object_id(),
            root_kind(root.content()),
        )
        .map_err(DrawStreamErrorV1::Sink)?;
        match root.content() {
            DocumentRenderContentV1::Molecule(plan) => lower_molecule_plan(plan, &face, sink)?,
            DocumentRenderContentV1::Text(text) => lower_document_text(text, &face, sink)?,
            DocumentRenderContentV1::Vector(vector) => lower_document_vector(vector, sink)?,
        }
        sink.end_root().map_err(DrawStreamErrorV1::Sink)?;
    }
    sink.finish_page().map_err(DrawStreamErrorV1::Sink)
}

/// Lower a molecule-only plan through the same operations used by document roots.
pub(crate) fn lower_molecule_plan_to_sink_v1<S: DrawSinkV1>(
    plan: &MoleculeRenderPlanV4,
    page: RenderViewportV1,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let environment = FerrumFontEnvironmentV1::load()
        .map_err(|error| DrawStreamErrorV1::Font(error.to_string()))?;
    let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
    let face = Face::parse(descriptor.data(), 0)
        .map_err(|error| DrawStreamErrorV1::Font(error.to_string()))?;
    sink.begin_page(page).map_err(DrawStreamErrorV1::Sink)?;
    lower_molecule_plan(plan, &face, sink)?;
    sink.finish_page().map_err(DrawStreamErrorV1::Sink)
}

/// Lower the bounded direct Haworth profile without widening the public V1 grammar.
pub(crate) use crate::direct_draw_stream_v1::lower_direct_glycosidic_haworth_plan_to_sink_v1;

/// Lower a checked bond-replacement composite without widening public backends.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn lower_document_render_composite_to_sink_v1<S: DrawSinkV1>(
    composite: &DocumentRenderCompositeV1,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let environment = FerrumFontEnvironmentV1::load()
        .map_err(|error| DrawStreamErrorV1::Font(error.to_string()))?;
    let descriptor = environment.descriptor(FerrumFontId::TelexRegular);
    let face = Face::parse(descriptor.data(), 0)
        .map_err(|error| DrawStreamErrorV1::Font(error.to_string()))?;
    let established = composite.established();
    let replacement = composite.replacement();
    sink.begin_page(established.page())
        .map_err(DrawStreamErrorV1::Sink)?;
    for outcome in established.outcomes() {
        let DocumentRenderOutcomeV1::Root(root) = outcome else {
            continue;
        };
        sink.begin_root_with_kind(
            root.paint_order(),
            root.target().document_object_id(),
            root_kind(root.content()),
        )
        .map_err(DrawStreamErrorV1::Sink)?;
        if root.target() == replacement.root_target()
            && root.paint_order() == replacement.root_paint_order()
        {
            let DocumentRenderContentV1::Molecule(plan) = root.content() else {
                return Err(DrawStreamErrorV1::InvalidComposite);
            };
            lower_replaced_molecule(
                plan,
                replacement.selected_keys(),
                replacement.direct(),
                &face,
                sink,
            )?;
        } else {
            match root.content() {
                DocumentRenderContentV1::Molecule(plan) => lower_molecule_plan(plan, &face, sink)?,
                DocumentRenderContentV1::Text(text) => lower_document_text(text, &face, sink)?,
                DocumentRenderContentV1::Vector(vector) => lower_document_vector(vector, sink)?,
            }
        }
        sink.end_root().map_err(DrawStreamErrorV1::Sink)?;
    }
    sink.finish_page().map_err(DrawStreamErrorV1::Sink)
}

#[cfg_attr(not(test), allow(dead_code))]
fn lower_replaced_molecule<S: DrawSinkV1>(
    plan: &MoleculeRenderPlanV4,
    selected: &std::collections::HashSet<(ferrum_document_projection::DocumentObjectIdV1, u32)>,
    direct: &AuthoredDirectGlycosidicHaworthRenderPlanV1,
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let mut batches = plan.batches().iter().peekable();
    let mut issues = plan.issues().iter().peekable();
    let mut emitted_direct = false;
    while batches.peek().is_some() || issues.peek().is_some() {
        let batch_next = batches.peek().map(|batch| batch.paint_order());
        let issue_next = issues.peek().map(|issue| issue.paint_order());
        let selected_target = match (batch_next, issue_next) {
            (Some(batch), Some(issue)) if batch < issue => {
                let Some(batch) = batches.next() else {
                    return Err(DrawStreamErrorV1::InvalidComposite);
                };
                if selected.contains(&(
                    batch.target().document_object_id().clone(),
                    batch.paint_order(),
                )) {
                    true
                } else {
                    lower_molecule_batch(batch, face, sink)?;
                    false
                }
            }
            (Some(_), Some(_)) | (None, Some(_)) => {
                let Some(issue) = issues.next() else {
                    return Err(DrawStreamErrorV1::InvalidComposite);
                };
                selected.contains(&(
                    issue.target().document_object_id().clone(),
                    issue.paint_order(),
                ))
            }
            (Some(_), None) => {
                let Some(batch) = batches.next() else {
                    return Err(DrawStreamErrorV1::InvalidComposite);
                };
                if selected.contains(&(
                    batch.target().document_object_id().clone(),
                    batch.paint_order(),
                )) {
                    true
                } else {
                    lower_molecule_batch(batch, face, sink)?;
                    false
                }
            }
            (None, None) => break,
        };
        if selected_target && !emitted_direct {
            lower_authored_direct_operations_to_sink_v1(direct, sink)?;
            emitted_direct = true;
        }
    }
    Ok(())
}

#[cfg_attr(not(test), allow(dead_code))]
fn lower_authored_direct_operations_to_sink_v1<S: DrawSinkV1>(
    plan: &AuthoredDirectGlycosidicHaworthRenderPlanV1,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    for operation in plan.operations() {
        sink.begin_direct_target_group(operation.authored_child_order())
            .map_err(DrawStreamErrorV1::Sink)?;
        match operation {
            AuthoredDirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
                endpoints, width, ..
            } => direct_path(
                sink,
                *endpoints,
                plan.paint(),
                *width,
                DrawLineCapV1::Butt,
                DrawMetadataV1::DirectGlycosidicOrdinary,
            )?,
            AuthoredDirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke {
                endpoints,
                width,
                ..
            } => direct_path(
                sink,
                *endpoints,
                plan.paint(),
                *width,
                DrawLineCapV1::Round,
                DrawMetadataV1::DirectGlycosidicQ1,
            )?,
            AuthoredDirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge { commands, .. } => {
                let mut lowered = Vec::new();
                lowered
                    .try_reserve(commands.len())
                    .map_err(|_| DrawStreamErrorV1::ResourceExhausted)?;
                for command in commands {
                    lowered.push(direct_command(*command));
                }
                sink.draw_path(
                    &DrawPathV1 { commands: lowered },
                    DrawStyleV1 {
                        fill: Some(plan.paint()),
                        stroke: None,
                        fill_rule: None,
                    },
                    DrawMetadataV1::DirectGlycosidicW1,
                )
                .map_err(DrawStreamErrorV1::Sink)?;
            }
        }
        sink.end_direct_target_group()
            .map_err(DrawStreamErrorV1::Sink)?;
    }
    Ok(())
}

pub(crate) fn direct_path<S: DrawSinkV1>(
    sink: &mut S,
    endpoints: [RenderPoint; 2],
    paint: &RenderPaintV3,
    width: PositiveFinite,
    cap: DrawLineCapV1,
    metadata: DrawMetadataV1,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let [start, end] = endpoints;
    let mut commands = Vec::new();
    commands
        .try_reserve(2)
        .map_err(|_| DrawStreamErrorV1::ResourceExhausted)?;
    commands.push(DrawPathCommandV1::MoveTo(start));
    commands.push(DrawPathCommandV1::LineTo(end));
    sink.draw_path(
        &DrawPathV1 { commands },
        DrawStyleV1 {
            fill: None,
            stroke: Some(DrawStrokeV1 {
                paint,
                width,
                line_cap: cap,
                line_join: VectorStrokeLineJoinV1::v1(),
                miter_limit: VectorStrokeLineJoinV1::v1().miter_limit(),
            }),
            fill_rule: None,
        },
        metadata,
    )
    .map_err(DrawStreamErrorV1::Sink)
}

pub(crate) fn direct_command(command: DirectGlycosidicHaworthPathCommandV1) -> DrawPathCommandV1 {
    match command {
        DirectGlycosidicHaworthPathCommandV1::MoveTo(point) => DrawPathCommandV1::MoveTo(point),
        DirectGlycosidicHaworthPathCommandV1::LineTo(point) => DrawPathCommandV1::LineTo(point),
        DirectGlycosidicHaworthPathCommandV1::CubicTo {
            control_1,
            control_2,
            end,
        } => DrawPathCommandV1::CubicTo {
            control_1,
            control_2,
            end,
        },
        DirectGlycosidicHaworthPathCommandV1::Close => DrawPathCommandV1::Close,
    }
}

fn lower_document_text<S: DrawSinkV1>(
    text: &DocumentTextOpV1,
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    sink.begin_document_text()
        .map_err(DrawStreamErrorV1::Sink)?;
    scoped_translate(text.anchor(), sink, |sink| {
        if let Some(background) = text.background() {
            let bounds = text.bounds();
            let width = positive_difference(bounds.max_x(), bounds.min_x())?;
            let height = positive_difference(bounds.max_y(), bounds.min_y())?;
            sink.fill_rect(
                DrawRectV1 {
                    origin: point(bounds.min_x(), bounds.min_y())?,
                    width,
                    height,
                },
                background,
                DrawMetadataV1::DocumentTextBackground,
            )
            .map_err(DrawStreamErrorV1::Sink)?;
        }
        match text.operation() {
            DocumentTextLayoutV1::Fixed(operation) => lower_text(operation, face, sink),
            DocumentTextLayoutV1::Presentation(operation) => {
                lower_presentation_text(operation, face, sink)
            }
        }
    })?;
    sink.end_document_text().map_err(DrawStreamErrorV1::Sink)
}

fn lower_document_vector<S: DrawSinkV1>(
    vector: &DocumentVectorRootV1,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    for operation in vector.operations() {
        match operation {
            DocumentVectorOpV1::Path {
                commands,
                stroke,
                fill,
            } => {
                let path = DrawPathV1 {
                    commands: commands.iter().copied().map(vector_command).collect(),
                };
                sink.draw_path(
                    &path,
                    vector_style(stroke.as_ref(), fill.as_ref(), operation.fill_rule()),
                    DrawMetadataV1::DocumentVectorPath,
                )
                .map_err(DrawStreamErrorV1::Sink)?;
            }
            DocumentVectorOpV1::Ellipse {
                center,
                radius_x,
                radius_y,
                stroke,
                fill,
            } => {
                sink.draw_ellipse(
                    DrawEllipseV1 {
                        center: *center,
                        radius_x: *radius_x,
                        radius_y: *radius_y,
                        rotation_degrees: 0.0,
                    },
                    vector_style(stroke.as_ref(), fill.as_ref(), None),
                    DrawMetadataV1::DocumentVectorEllipse,
                )
                .map_err(DrawStreamErrorV1::Sink)?;
            }
        }
    }
    Ok(())
}

fn vector_command(command: PathCommandV1) -> DrawPathCommandV1 {
    match command {
        PathCommandV1::MoveTo(point) => DrawPathCommandV1::MoveTo(point),
        PathCommandV1::LineTo(point) => DrawPathCommandV1::LineTo(point),
        PathCommandV1::CubicTo {
            control_1,
            control_2,
            end,
        } => DrawPathCommandV1::CubicTo {
            control_1,
            control_2,
            end,
        },
        PathCommandV1::Close => DrawPathCommandV1::Close,
    }
}

fn vector_style<'a>(
    stroke: Option<&'a StrokeV1>,
    fill: Option<&'a RenderPaintV3>,
    fill_rule: Option<VectorFillRuleV1>,
) -> DrawStyleV1<'a> {
    DrawStyleV1 {
        fill,
        stroke: stroke.map(|stroke| DrawStrokeV1 {
            paint: stroke.paint(),
            width: stroke.width(),
            line_cap: match stroke.line_cap() {
                VectorStrokeLineCapV1::Butt => DrawLineCapV1::Butt,
                VectorStrokeLineCapV1::Round => DrawLineCapV1::Round,
            },
            line_join: stroke.line_join(),
            miter_limit: stroke.miter_limit(),
        }),
        fill_rule,
    }
}

pub(crate) fn lower_text<S: DrawSinkV1>(
    text: &TextOp,
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    lower_text_runs(
        text.z(),
        text.paint(),
        text.size().get(),
        text.origin(),
        text.runs(),
        face,
        sink,
    )
}

fn lower_presentation_text<S: DrawSinkV1>(
    text: &PresentationTextOp,
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    lower_presentation_text_runs(
        text.z(),
        text.paint(),
        text.size().get(),
        point(0.0, 0.0)?,
        text.runs(),
        face,
        sink,
    )
}

trait TextRunV1 {
    fn origin(&self) -> RenderPoint;
    fn scale(&self) -> f64;
    fn glyphs(&self) -> &[GlyphPlacement];
}

impl TextRunV1 for TextRun {
    fn origin(&self) -> RenderPoint {
        self.origin()
    }
    fn scale(&self) -> f64 {
        self.scale().get()
    }
    fn glyphs(&self) -> &[GlyphPlacement] {
        self.glyphs()
    }
}

fn lower_text_runs<S: DrawSinkV1, R: TextRunV1>(
    z: i32,
    paint: &RenderPaintV3,
    size: f64,
    operation_origin: RenderPoint,
    runs: &[R],
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let units_per_em = f64::from(face.units_per_em());
    if !units_per_em.is_finite() || units_per_em <= 0.0 {
        return Err(DrawStreamErrorV1::Font(
            "Telex units-per-em is invalid".to_owned(),
        ));
    }
    sink.begin_text_operation(z, paint)
        .map_err(DrawStreamErrorV1::Sink)?;
    for run in runs {
        let run_origin = add_points(operation_origin, run.origin())?;
        let multiplier = checked_product(size, run.scale())? / units_per_em;
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(DrawStreamErrorV1::NonFiniteGeometry);
        }
        for glyph in run.glyphs() {
            let origin = add_points(run_origin, glyph.origin())?;
            let mut builder = OutlinePathBuilder::new(origin, multiplier);
            let outlined = face.outline_glyph(
                GlyphId(u16::try_from(glyph.glyph_index()).map_err(|_| {
                    DrawStreamErrorV1::MissingGlyphOutline {
                        glyph_index: glyph.glyph_index(),
                    }
                })?),
                &mut builder,
            );
            if let Some(error) = builder.error.take() {
                return Err(error);
            }
            if outlined.is_none() || !builder.segments {
                return Err(DrawStreamErrorV1::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                });
            }
            let path = DrawPathV1 {
                commands: builder.commands,
            };
            sink.draw_path(
                &path,
                DrawStyleV1 {
                    fill: Some(paint),
                    stroke: None,
                    fill_rule: None,
                },
                DrawMetadataV1::MoleculeText { z },
            )
            .map_err(DrawStreamErrorV1::Sink)?;
        }
    }
    sink.end_text_operation().map_err(DrawStreamErrorV1::Sink)
}

fn lower_presentation_text_runs<S: DrawSinkV1>(
    z: i32,
    paint: &RenderPaintV3,
    size: f64,
    operation_origin: RenderPoint,
    runs: &[PresentationGlyphRun],
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let units_per_em = f64::from(face.units_per_em());
    if !units_per_em.is_finite() || units_per_em <= 0.0 {
        return Err(DrawStreamErrorV1::Font(
            "Telex units-per-em is invalid".to_owned(),
        ));
    }
    sink.begin_text_operation(z, paint)
        .map_err(DrawStreamErrorV1::Sink)?;
    for run in runs {
        let run_origin = add_points(operation_origin, run.origin())?;
        let multiplier = checked_product(size, run.scale().get())? / units_per_em;
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(DrawStreamErrorV1::NonFiniteGeometry);
        }
        for (scalar, glyph) in run.text().chars().zip(run.glyphs()) {
            let origin = add_points(run_origin, glyph.origin())?;
            let mut builder = OutlinePathBuilder::new(origin, multiplier);
            let glyph_id = u16::try_from(glyph.glyph_index()).map_err(|_| {
                DrawStreamErrorV1::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                }
            })?;
            let outlined = face.outline_glyph(GlyphId(glyph_id), &mut builder);
            if let Some(error) = builder.error.take() {
                return Err(error);
            }
            if outlined.is_none() || !builder.segments {
                if is_verified_outlineless_whitespace_glyph(face, scalar, glyph.glyph_index()) {
                    continue;
                }
                return Err(DrawStreamErrorV1::MissingGlyphOutline {
                    glyph_index: glyph.glyph_index(),
                });
            }
            let path = DrawPathV1 {
                commands: builder.commands,
            };
            sink.draw_path(
                &path,
                DrawStyleV1 {
                    fill: Some(paint),
                    stroke: None,
                    fill_rule: None,
                },
                DrawMetadataV1::MoleculeText { z },
            )
            .map_err(DrawStreamErrorV1::Sink)?;
        }
    }
    sink.end_text_operation().map_err(DrawStreamErrorV1::Sink)
}

pub(crate) fn scoped_translate<S: DrawSinkV1, F>(
    anchor: RenderPoint,
    sink: &mut S,
    lower: F,
) -> Result<(), DrawStreamErrorV1<S::Error>>
where
    F: FnOnce(&mut S) -> Result<(), DrawStreamErrorV1<S::Error>>,
{
    sink.save().map_err(DrawStreamErrorV1::Sink)?;
    let result = sink
        .concat_translate(anchor)
        .map_err(DrawStreamErrorV1::Sink)
        .and_then(|()| lower(sink));
    let restore = sink.restore().map_err(DrawStreamErrorV1::Sink);
    match (result, restore) {
        (Err(operation), _) => Err(operation),
        (Ok(()), Err(restore)) => Err(restore),
        (Ok(()), Ok(())) => Ok(()),
    }
}

fn point<E>(x: f64, y: f64) -> Result<RenderPoint, DrawStreamErrorV1<E>> {
    RenderPoint::new(x, y).map_err(|_| DrawStreamErrorV1::NonFiniteGeometry)
}

fn add_points<E>(
    first: RenderPoint,
    second: RenderPoint,
) -> Result<RenderPoint, DrawStreamErrorV1<E>> {
    point(first.x() + second.x(), first.y() + second.y())
}

fn checked_product<E>(first: f64, second: f64) -> Result<f64, DrawStreamErrorV1<E>> {
    let value = first * second;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(DrawStreamErrorV1::NonFiniteGeometry)
    }
}

fn positive_difference<E>(
    maximum: f64,
    minimum: f64,
) -> Result<PositiveFinite, DrawStreamErrorV1<E>> {
    PositiveFinite::new(maximum - minimum).map_err(|_| DrawStreamErrorV1::NonFiniteGeometry)
}

struct OutlinePathBuilder<E> {
    commands: Vec<DrawPathCommandV1>,
    origin: RenderPoint,
    multiplier: f64,
    segments: bool,
    error: Option<DrawStreamErrorV1<E>>,
}

impl<E> OutlinePathBuilder<E> {
    fn new(origin: RenderPoint, multiplier: f64) -> Self {
        Self {
            commands: Vec::new(),
            origin,
            multiplier,
            segments: false,
            error: None,
        }
    }

    fn point(&mut self, x: f32, y: f32) -> Option<RenderPoint> {
        if self.error.is_some() {
            return None;
        }
        match point(
            self.origin.x() + f64::from(x) * self.multiplier,
            self.origin.y() - f64::from(y) * self.multiplier,
        ) {
            Ok(point) => Some(point),
            Err(error) => {
                self.error = Some(error);
                None
            }
        }
    }
}

impl<E> OutlineBuilder for OutlinePathBuilder<E> {
    fn move_to(&mut self, x: f32, y: f32) {
        if let Some(point) = self.point(x, y) {
            self.commands.push(DrawPathCommandV1::MoveTo(point));
        }
    }
    fn line_to(&mut self, x: f32, y: f32) {
        if let Some(point) = self.point(x, y) {
            self.commands.push(DrawPathCommandV1::LineTo(point));
            self.segments = true;
        }
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let control = self.point(x1, y1);
        let end = self.point(x, y);
        if let (Some(control), Some(end)) = (control, end) {
            self.commands
                .push(DrawPathCommandV1::QuadraticTo { control, end });
            self.segments = true;
        }
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let control_1 = self.point(x1, y1);
        let control_2 = self.point(x2, y2);
        let end = self.point(x, y);
        if let (Some(control_1), Some(control_2), Some(end)) = (control_1, control_2, end) {
            self.commands.push(DrawPathCommandV1::CubicTo {
                control_1,
                control_2,
                end,
            });
            self.segments = true;
        }
    }
    fn close(&mut self) {
        self.commands.push(DrawPathCommandV1::Close);
    }
}

impl DrawSinkV1 for () {
    type Error = Infallible;
    fn begin_page(&mut self, _: RenderViewportV1) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_root(
        &mut self,
        _: u32,
        _: &ferrum_document_projection::DocumentObjectIdV1,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_root(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_molecule_batch(&mut self, _: u32, _: BatchSpace) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_molecule_batch(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_text_operation(&mut self, _: i32, _: &RenderPaintV3) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn save(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn concat_translate(&mut self, _: RenderPoint) -> Result<(), Self::Error> {
        Ok(())
    }
    fn restore(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn fill_rect(
        &mut self,
        _: DrawRectV1,
        _: &RenderPaintV3,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn draw_path(
        &mut self,
        _: &DrawPathV1,
        _: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn draw_ellipse(
        &mut self,
        _: DrawEllipseV1,
        _: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn finish_page(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
