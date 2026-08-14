//! Page-owning private stream lowering for the established M14 direct plan.

use crate::RenderViewportV1;
use crate::direct_glycosidic_haworth::{
    DirectGlycosidicHaworthDrawOpV1, DirectGlycosidicHaworthRenderPlanV1,
};
use crate::draw_stream_v1::{
    DrawLineCapV1, DrawMetadataV1, DrawPathV1, DrawSinkV1, DrawStreamErrorV1, DrawStyleV1,
    direct_command, direct_path,
};

pub(crate) fn lower_direct_glycosidic_haworth_plan_to_sink_v1<S: DrawSinkV1>(
    plan: &DirectGlycosidicHaworthRenderPlanV1,
    page: RenderViewportV1,
    sink: &mut S,
) -> Result<(), DrawStreamErrorV1<S::Error>> {
    sink.begin_page(page).map_err(DrawStreamErrorV1::Sink)?;
    for operation in plan.operations() {
        match operation {
            DirectGlycosidicHaworthDrawOpV1::OrdinaryLine {
                endpoints, width, ..
            } => direct_path(
                sink,
                *endpoints,
                plan.paint(),
                *width,
                DrawLineCapV1::Butt,
                DrawMetadataV1::DirectGlycosidicOrdinary,
            )?,
            DirectGlycosidicHaworthDrawOpV1::HaworthFrontStroke {
                endpoints, width, ..
            } => direct_path(
                sink,
                *endpoints,
                plan.paint(),
                *width,
                DrawLineCapV1::Round,
                DrawMetadataV1::DirectGlycosidicQ1,
            )?,
            DirectGlycosidicHaworthDrawOpV1::RoundedFrontWedge { commands, .. } => {
                let mut lowered_commands = Vec::new();
                lowered_commands
                    .try_reserve(commands.len())
                    .map_err(|_| DrawStreamErrorV1::ResourceExhausted)?;
                for command in commands {
                    lowered_commands.push(direct_command(*command));
                }
                sink.draw_path(
                    &DrawPathV1 {
                        commands: lowered_commands,
                    },
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
    }
    sink.finish_page().map_err(DrawStreamErrorV1::Sink)
}
