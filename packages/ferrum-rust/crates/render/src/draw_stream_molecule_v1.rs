//! Molecule operation lowering for the private draw stream.
//!
//! Kept separate from document-root dispatch so the shared lowering contract
//! remains readable without making any additional renderer API public.

use ttf_parser::Face;

use crate::draw_stream_v1::{
    DrawEllipseV1, DrawLineCapV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1,
    DrawSinkV1, DrawStreamErrorV1, DrawStrokeV1, DrawStyleV1, scoped_translate,
};
use crate::{
    BatchSpace, EllipseOp, LineOp, MaskOp, MoleculeRenderPlan, PathOpV2, RenderOp,
    ScenePathCommandV2, VectorStrokeLineCapV1, VectorStrokeLineJoinV1,
};

/// Lower one molecule plan through the common private draw stream.
pub(crate) fn lower_molecule_plan<S: DrawSinkV1>(
    plan: &MoleculeRenderPlan,
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    for batch in plan.batches() {
        lower_molecule_batch(batch, face, sink)?;
    }
    Ok(())
}

pub(crate) fn lower_molecule_batch<S: DrawSinkV1>(
    batch: &crate::RenderBatch,
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    sink.begin_molecule_target_group(batch.target(), batch.coordinate_space().clone())
        .map_err(DrawStreamErrorV1::Sink)?;
    if let BatchSpace::AtomLocal { anchor } = batch.coordinate_space() {
        scoped_translate(*anchor, sink, |sink| {
            lower_molecule_operations(batch.operations(), face, sink)
        })?;
    } else {
        lower_molecule_operations(batch.operations(), face, sink)?;
    }
    sink.end_molecule_batch().map_err(DrawStreamErrorV1::Sink)?;
    Ok(())
}

fn lower_molecule_operations<S: DrawSinkV1>(
    operations: &[RenderOp],
    face: &Face<'_>,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    for operation in operations {
        match operation {
            RenderOp::Line(line) => lower_line(line, sink)?,
            RenderOp::Mask(mask) => lower_mask(mask, sink)?,
            RenderOp::Ellipse(ellipse) => lower_ellipse(ellipse, sink)?,
            RenderOp::Path(path) => lower_path(path, sink)?,
            RenderOp::Text(text) => crate::draw_stream_v1::lower_text(text, face, sink)?,
        }
    }
    Ok(())
}

fn lower_path<S: DrawSinkV1>(
    path: &PathOpV2,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let commands = path
        .commands()
        .iter()
        .map(|command| match command {
            ScenePathCommandV2::MoveTo(point) => DrawPathCommandV1::MoveTo(*point),
            ScenePathCommandV2::LineTo(point) => DrawPathCommandV1::LineTo(*point),
            ScenePathCommandV2::CubicTo {
                control_1,
                control_2,
                end,
            } => DrawPathCommandV1::CubicTo {
                control_1: *control_1,
                control_2: *control_2,
                end: *end,
            },
            ScenePathCommandV2::Close => DrawPathCommandV1::Close,
        })
        .collect();
    sink.draw_path(
        &DrawPathV1 { commands },
        DrawStyleV1 {
            fill: path.fill(),
            stroke: path.stroke().map(|stroke| DrawStrokeV1 {
                paint: stroke.paint(),
                width: stroke.width(),
                line_cap: match stroke.line_cap() {
                    VectorStrokeLineCapV1::Butt => DrawLineCapV1::Butt,
                    VectorStrokeLineCapV1::Round => DrawLineCapV1::Round,
                },
                line_join: stroke.line_join(),
                miter_limit: stroke.miter_limit(),
            }),
            fill_rule: path.fill_rule(),
        },
        DrawMetadataV1::MoleculePath { z: path.z() },
    )
    .map_err(DrawStreamErrorV1::Sink)
}

fn lower_line<S: DrawSinkV1>(
    line: &LineOp,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    let path = DrawPathV1 {
        commands: vec![
            DrawPathCommandV1::MoveTo(line.start()),
            DrawPathCommandV1::LineTo(line.end()),
        ],
    };
    sink.draw_path(
        &path,
        DrawStyleV1 {
            fill: None,
            stroke: Some(DrawStrokeV1 {
                paint: line.paint(),
                width: line.width(),
                line_cap: DrawLineCapV1::Butt,
                line_join: VectorStrokeLineJoinV1::v1(),
                miter_limit: VectorStrokeLineJoinV1::v1().miter_limit(),
            }),
            fill_rule: None,
        },
        DrawMetadataV1::MoleculeLine { z: line.z() },
    )
    .map_err(DrawStreamErrorV1::Sink)
}

fn lower_mask<S: DrawSinkV1>(
    mask: &MaskOp,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    sink.fill_rect(
        DrawRectV1 {
            origin: mask.origin(),
            width: mask.width(),
            height: mask.height(),
        },
        mask.paint(),
        DrawMetadataV1::MoleculeMask { z: mask.z() },
    )
    .map_err(DrawStreamErrorV1::Sink)
}

fn lower_ellipse<S: DrawSinkV1>(
    ellipse: &EllipseOp,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    sink.draw_ellipse(
        DrawEllipseV1 {
            center: ellipse.center(),
            radius_x: ellipse.radius_x(),
            radius_y: ellipse.radius_y(),
            rotation_degrees: ellipse.rotation_degrees(),
        },
        DrawStyleV1 {
            fill: ellipse.fill_paint(),
            stroke: ellipse
                .stroke_paint()
                .zip(ellipse.stroke_width())
                .map(|(paint, width)| DrawStrokeV1 {
                    paint,
                    width,
                    line_cap: DrawLineCapV1::Butt,
                    line_join: VectorStrokeLineJoinV1::v1(),
                    miter_limit: VectorStrokeLineJoinV1::v1().miter_limit(),
                }),
            fill_rule: None,
        },
        DrawMetadataV1::MoleculeEllipse { z: ellipse.z() },
    )
    .map_err(DrawStreamErrorV1::Sink)
}
