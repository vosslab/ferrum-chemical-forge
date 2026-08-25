//! Owned paint-only recording of one authenticated document composite.
//!
//! This is an internal desktop seam, not a stable Ferrum wire, CLI, or binding API.

use thiserror::Error;

use crate::draw_stream_v1::{
    DrawEllipseV1, DrawLineCapV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1,
    DrawRootKindV1, DrawSinkV1, DrawStreamErrorV1, DrawStyleV1,
    lower_document_render_composite_to_sink_v1,
};
use crate::{
    BatchSpace, DocumentRenderCompositeV1, Paint, PositiveFinite, RenderPoint, RenderProvenance,
    RenderTarget, RenderViewportV1, VectorFillRuleV1, VectorStrokeLineJoinV1,
};

/// Caller-owned structural limits for one composite recording.
///
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompositeRecordingBudgetV1 {
    pub max_roots: usize,
    pub max_target_groups: usize,
    pub max_events: usize,
    pub max_path_commands: usize,
    pub max_transform_depth: usize,
    pub max_text_scopes: usize,
}

/// One resource that can reject a bounded recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeRecordingResourceV1 {
    Roots,
    TargetGroups,
    Events,
    PathCommands,
    TransformDepth,
    TextScopes,
}

/// Typed recording failures; no partial record is returned.
#[derive(Debug, Error, Eq, PartialEq)]
pub enum CompositeRecordingErrorV1 {
    #[error("composite recording {resource:?} counter overflowed")]
    CounterOverflow {
        resource: CompositeRecordingResourceV1,
    },
    #[error("composite recording exceeded its {resource:?} budget")]
    BudgetExceeded {
        resource: CompositeRecordingResourceV1,
    },
    #[error("could not allocate composite recording storage for {resource:?}")]
    ResourceExhausted {
        resource: CompositeRecordingResourceV1,
    },
    #[error("authenticated composite could not be lowered")]
    InvalidComposite,
    #[error("composite recording stream violated its lexical grammar")]
    InvalidStream,
}

/// Private lowering context retained without inferring root content from paint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeRootKindV1 {
    Molecule,
    Text,
    Vector,
}

/// Explicit fill rule, including the nonzero rule used by non-vector paint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeFillRuleV1 {
    NonZero,
    EvenOdd,
}

/// Explicit stroke cap.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeLineCapV1 {
    Butt,
    Round,
}

/// Explicit stroke join.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositeLineJoinV1 {
    Miter,
}

/// Owned explicit fill profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositeFillV1 {
    paint: Paint,
    rule: CompositeFillRuleV1,
}

impl CompositeFillV1 {
    #[must_use]
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }
    #[must_use]
    pub const fn rule(&self) -> CompositeFillRuleV1 {
        self.rule
    }
}

/// Owned explicit stroke profile.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeStrokeV1 {
    paint: Paint,
    width: PositiveFinite,
    cap: CompositeLineCapV1,
    join: CompositeLineJoinV1,
    miter_limit: f64,
}

impl CompositeStrokeV1 {
    #[must_use]
    pub const fn paint(&self) -> &Paint {
        &self.paint
    }
    #[must_use]
    pub const fn width(&self) -> PositiveFinite {
        self.width
    }
    #[must_use]
    pub const fn cap(&self) -> CompositeLineCapV1 {
        self.cap
    }
    #[must_use]
    pub const fn join(&self) -> CompositeLineJoinV1 {
        self.join
    }
    #[must_use]
    pub const fn miter_limit(&self) -> f64 {
        self.miter_limit
    }
}

/// Owned explicit path style.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeStyleV1 {
    fill: Option<CompositeFillV1>,
    stroke: Option<CompositeStrokeV1>,
}

impl CompositeStyleV1 {
    #[must_use]
    pub const fn fill(&self) -> Option<&CompositeFillV1> {
        self.fill.as_ref()
    }
    #[must_use]
    pub const fn stroke(&self) -> Option<&CompositeStrokeV1> {
        self.stroke.as_ref()
    }
}

/// The restricted path grammar consumable by a desktop painter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CompositePathCommandV1 {
    MoveTo(RenderPoint),
    LineTo(RenderPoint),
    CubicTo {
        control_1: RenderPoint,
        control_2: RenderPoint,
        end: RenderPoint,
    },
    Close,
}

/// One paint-event category with its source-backed z fact when applicable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompositePaintKindV1 {
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

/// One owned lexical event. Paint primitives carry monotonic `paint_index`.
#[derive(Clone, Debug, PartialEq)]
pub enum CompositeRecordingEventV1 {
    PageBegin {
        viewport: RenderViewportV1,
        provenance: RenderProvenance,
    },
    RootBegin {
        document_object_id: ferrum_document_projection::DocumentObjectIdV1,
        paint_order: u32,
        kind: CompositeRootKindV1,
    },
    RootEnd,
    TargetBegin {
        target: Option<RenderTarget>,
        space: Option<BatchSpace>,
        direct: bool,
    },
    TargetEnd,
    DocumentTextBegin,
    DocumentTextEnd,
    TextOperationBegin {
        z: i32,
        paint: Paint,
    },
    TextOperationEnd,
    Save,
    Translate {
        point: RenderPoint,
    },
    Restore,
    FillRect {
        origin: RenderPoint,
        width: PositiveFinite,
        height: PositiveFinite,
        paint: Paint,
        kind: CompositePaintKindV1,
        paint_index: u64,
    },
    Path {
        commands: Vec<CompositePathCommandV1>,
        style: CompositeStyleV1,
        kind: CompositePaintKindV1,
        paint_index: u64,
    },
    Ellipse {
        center: RenderPoint,
        radius_x: PositiveFinite,
        radius_y: PositiveFinite,
        rotation_degrees: f64,
        style: CompositeStyleV1,
        kind: CompositePaintKindV1,
        paint_index: u64,
    },
    PageEnd,
}

/// Owned whole-document paint recording for one exact authenticated composite.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositeRecordingV1 {
    provenance: RenderProvenance,
    events: Vec<CompositeRecordingEventV1>,
}

impl CompositeRecordingV1 {
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }
    #[must_use]
    pub fn events(&self) -> &[CompositeRecordingEventV1] {
        &self.events
    }
}

/// Record all paint from one opaque authenticated composite.
pub fn record_document_render_composite_v1(
    composite: &DocumentRenderCompositeV1,
    budget: CompositeRecordingBudgetV1,
) -> Result<CompositeRecordingV1, CompositeRecordingErrorV1> {
    let mut sink = RecordingSink::new(composite.provenance(), budget);
    lower_document_render_composite_to_sink_v1(composite, &mut sink).map_err(map_lower_error)?;
    sink.finish()
}

fn map_lower_error(
    error: DrawStreamErrorV1<CompositeRecordingErrorV1>,
) -> CompositeRecordingErrorV1 {
    match error {
        DrawStreamErrorV1::Sink(error) => error,
        DrawStreamErrorV1::InvalidComposite => CompositeRecordingErrorV1::InvalidComposite,
        DrawStreamErrorV1::ResourceExhausted => CompositeRecordingErrorV1::ResourceExhausted {
            resource: CompositeRecordingResourceV1::Events,
        },
        DrawStreamErrorV1::NonFiniteGeometry
        | DrawStreamErrorV1::Font(_)
        | DrawStreamErrorV1::MissingGlyphOutline { .. } => CompositeRecordingErrorV1::InvalidStream,
    }
}

pub(crate) struct RecordingSink {
    provenance: RenderProvenance,
    budget: CompositeRecordingBudgetV1,
    events: Vec<CompositeRecordingEventV1>,
    roots: usize,
    target_groups: usize,
    event_count: usize,
    path_commands: usize,
    transform_depth: usize,
    text_scopes: usize,
    paint_index: u64,
    page_open: bool,
    page_started: bool,
    page_finished: bool,
    root_open: bool,
    target_open: bool,
    document_text_open: bool,
    text_operation_open: bool,
    root_kind: Option<CompositeRootKindV1>,
    last_root_paint_order: Option<u32>,
}

impl RecordingSink {
    fn new(provenance: RenderProvenance, budget: CompositeRecordingBudgetV1) -> Self {
        Self {
            provenance,
            budget,
            events: Vec::new(),
            roots: 0,
            target_groups: 0,
            event_count: 0,
            path_commands: 0,
            transform_depth: 0,
            text_scopes: 0,
            paint_index: 0,
            page_open: false,
            page_started: false,
            page_finished: false,
            root_open: false,
            target_open: false,
            document_text_open: false,
            text_operation_open: false,
            root_kind: None,
            last_root_paint_order: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        provenance: RenderProvenance,
        budget: CompositeRecordingBudgetV1,
    ) -> Self {
        Self::new(provenance, budget)
    }

    #[cfg(test)]
    pub(crate) fn test_events(&self) -> &[CompositeRecordingEventV1] {
        &self.events
    }

    #[cfg(test)]
    pub(crate) fn finish_for_test(self) -> Result<CompositeRecordingV1, CompositeRecordingErrorV1> {
        self.finish()
    }

    fn count(
        &mut self,
        resource: CompositeRecordingResourceV1,
        maximum: usize,
    ) -> Result<(), CompositeRecordingErrorV1> {
        let count = match resource {
            CompositeRecordingResourceV1::Roots => &mut self.roots,
            CompositeRecordingResourceV1::TargetGroups => &mut self.target_groups,
            CompositeRecordingResourceV1::Events => &mut self.event_count,
            CompositeRecordingResourceV1::PathCommands => &mut self.path_commands,
            CompositeRecordingResourceV1::TransformDepth => &mut self.transform_depth,
            CompositeRecordingResourceV1::TextScopes => &mut self.text_scopes,
        };
        *count = count
            .checked_add(1)
            .ok_or(CompositeRecordingErrorV1::CounterOverflow { resource })?;
        if *count > maximum {
            return Err(CompositeRecordingErrorV1::BudgetExceeded { resource });
        }
        Ok(())
    }

    fn push(&mut self, event: CompositeRecordingEventV1) -> Result<(), CompositeRecordingErrorV1> {
        self.count(CompositeRecordingResourceV1::Events, self.budget.max_events)?;
        self.events
            .try_reserve(1)
            .map_err(|_| CompositeRecordingErrorV1::ResourceExhausted {
                resource: CompositeRecordingResourceV1::Events,
            })?;
        self.events.push(event);
        Ok(())
    }

    fn paint_index(&mut self) -> Result<u64, CompositeRecordingErrorV1> {
        let index = self.paint_index;
        self.paint_index =
            self.paint_index
                .checked_add(1)
                .ok_or(CompositeRecordingErrorV1::CounterOverflow {
                    resource: CompositeRecordingResourceV1::Events,
                })?;
        Ok(index)
    }

    fn require_paint_scope(&self) -> Result<(), CompositeRecordingErrorV1> {
        if self.root_open
            && (self.target_open
                || self.document_text_open
                || self.root_kind == Some(CompositeRootKindV1::Vector))
        {
            Ok(())
        } else {
            Err(CompositeRecordingErrorV1::InvalidStream)
        }
    }

    fn style(
        &mut self,
        style: DrawStyleV1<'_>,
    ) -> Result<CompositeStyleV1, CompositeRecordingErrorV1> {
        let fill = style
            .fill
            .map(|paint| {
                let paint = paint.clone();
                Ok(CompositeFillV1 {
                    paint,
                    rule: match style.fill_rule {
                        Some(VectorFillRuleV1::EvenOdd) => CompositeFillRuleV1::EvenOdd,
                        None => CompositeFillRuleV1::NonZero,
                    },
                })
            })
            .transpose()?;
        let stroke = style
            .stroke
            .map(|stroke| {
                let paint = stroke.paint.clone();
                if !stroke.miter_limit.is_finite() {
                    return Err(CompositeRecordingErrorV1::InvalidStream);
                }
                Ok(CompositeStrokeV1 {
                    paint,
                    width: stroke.width,
                    cap: match stroke.line_cap {
                        DrawLineCapV1::Butt => CompositeLineCapV1::Butt,
                        DrawLineCapV1::Round => CompositeLineCapV1::Round,
                    },
                    join: match stroke.line_join {
                        VectorStrokeLineJoinV1::Miter => CompositeLineJoinV1::Miter,
                    },
                    miter_limit: stroke.miter_limit,
                })
            })
            .transpose()?;
        Ok(CompositeStyleV1 { fill, stroke })
    }

    fn path(
        &mut self,
        path: &DrawPathV1,
    ) -> Result<Vec<CompositePathCommandV1>, CompositeRecordingErrorV1> {
        let next = self.path_commands.checked_add(path.commands.len()).ok_or(
            CompositeRecordingErrorV1::CounterOverflow {
                resource: CompositeRecordingResourceV1::PathCommands,
            },
        )?;
        if next > self.budget.max_path_commands {
            return Err(CompositeRecordingErrorV1::BudgetExceeded {
                resource: CompositeRecordingResourceV1::PathCommands,
            });
        }
        let mut result = Vec::new();
        result.try_reserve(path.commands.len()).map_err(|_| {
            CompositeRecordingErrorV1::ResourceExhausted {
                resource: CompositeRecordingResourceV1::PathCommands,
            }
        })?;
        self.path_commands = next;
        let mut current = None;
        for command in &path.commands {
            let next = match *command {
                DrawPathCommandV1::MoveTo(point) => {
                    current = Some(point);
                    CompositePathCommandV1::MoveTo(point)
                }
                DrawPathCommandV1::LineTo(point) => {
                    current = Some(point);
                    CompositePathCommandV1::LineTo(point)
                }
                DrawPathCommandV1::CubicTo {
                    control_1,
                    control_2,
                    end,
                } => {
                    current = Some(end);
                    CompositePathCommandV1::CubicTo {
                        control_1,
                        control_2,
                        end,
                    }
                }
                DrawPathCommandV1::QuadraticTo { control, end } => {
                    let start = current.ok_or(CompositeRecordingErrorV1::InvalidStream)?;
                    let control_1 = point(
                        start.x() + (2.0 / 3.0) * (control.x() - start.x()),
                        start.y() + (2.0 / 3.0) * (control.y() - start.y()),
                    )?;
                    let control_2 = point(
                        end.x() + (2.0 / 3.0) * (control.x() - end.x()),
                        end.y() + (2.0 / 3.0) * (control.y() - end.y()),
                    )?;
                    current = Some(end);
                    CompositePathCommandV1::CubicTo {
                        control_1,
                        control_2,
                        end,
                    }
                }
                DrawPathCommandV1::Close => CompositePathCommandV1::Close,
            };
            result.push(next);
        }
        Ok(result)
    }

    fn finish(self) -> Result<CompositeRecordingV1, CompositeRecordingErrorV1> {
        if !self.page_started
            || !self.page_finished
            || self.page_open
            || self.root_open
            || self.target_open
            || self.document_text_open
            || self.text_operation_open
            || self.transform_depth != 0
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        Ok(CompositeRecordingV1 {
            provenance: self.provenance,
            events: self.events,
        })
    }
}

fn point(x: f64, y: f64) -> Result<RenderPoint, CompositeRecordingErrorV1> {
    RenderPoint::new(x, y).map_err(|_| CompositeRecordingErrorV1::InvalidStream)
}

fn kind(metadata: DrawMetadataV1) -> CompositePaintKindV1 {
    match metadata {
        DrawMetadataV1::MoleculeLine { z } => CompositePaintKindV1::MoleculeLine { z },
        DrawMetadataV1::MoleculePath { z } => CompositePaintKindV1::MoleculePath { z },
        DrawMetadataV1::MoleculeMask { z } => CompositePaintKindV1::MoleculeMask { z },
        DrawMetadataV1::MoleculeEllipse { z } => CompositePaintKindV1::MoleculeEllipse { z },
        DrawMetadataV1::MoleculeText { z } => CompositePaintKindV1::MoleculeText { z },
        DrawMetadataV1::DocumentTextBackground => CompositePaintKindV1::DocumentTextBackground,
        DrawMetadataV1::DocumentVectorPath => CompositePaintKindV1::DocumentVectorPath,
        DrawMetadataV1::DocumentVectorEllipse => CompositePaintKindV1::DocumentVectorEllipse,
        DrawMetadataV1::DirectGlycosidicOrdinary => CompositePaintKindV1::DirectGlycosidicOrdinary,
        DrawMetadataV1::DirectGlycosidicQ1 => CompositePaintKindV1::DirectGlycosidicQ1,
        DrawMetadataV1::DirectGlycosidicW1 => CompositePaintKindV1::DirectGlycosidicW1,
    }
}

impl DrawSinkV1 for RecordingSink {
    type Error = CompositeRecordingErrorV1;
    fn begin_page(&mut self, page: RenderViewportV1) -> Result<(), Self::Error> {
        if self.page_open || self.page_started || self.page_finished {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.page_open = true;
        self.page_started = true;
        self.push(CompositeRecordingEventV1::PageBegin {
            viewport: page,
            provenance: self.provenance,
        })
    }
    fn begin_root(
        &mut self,
        _: u32,
        _: &ferrum_document_projection::DocumentObjectIdV1,
    ) -> Result<(), Self::Error> {
        Err(CompositeRecordingErrorV1::InvalidStream)
    }
    fn begin_root_with_kind(
        &mut self,
        paint_order: u32,
        document_object_id: &ferrum_document_projection::DocumentObjectIdV1,
        root_kind: DrawRootKindV1,
    ) -> Result<(), Self::Error> {
        if !self.page_open
            || self.root_open
            || self
                .last_root_paint_order
                .is_some_and(|last| last >= paint_order)
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.count(CompositeRecordingResourceV1::Roots, self.budget.max_roots)?;
        let kind = match root_kind {
            DrawRootKindV1::Molecule => CompositeRootKindV1::Molecule,
            DrawRootKindV1::Text => CompositeRootKindV1::Text,
            DrawRootKindV1::Vector => CompositeRootKindV1::Vector,
        };
        self.root_open = true;
        self.root_kind = Some(kind);
        self.last_root_paint_order = Some(paint_order);
        self.push(CompositeRecordingEventV1::RootBegin {
            document_object_id: document_object_id.clone(),
            paint_order,
            kind,
        })
    }
    fn end_root(&mut self) -> Result<(), Self::Error> {
        if !self.root_open
            || self.target_open
            || self.document_text_open
            || self.text_operation_open
            || self.transform_depth != 0
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.root_open = false;
        self.root_kind = None;
        self.push(CompositeRecordingEventV1::RootEnd)
    }
    fn begin_molecule_batch(&mut self, _: u32, _: BatchSpace) -> Result<(), Self::Error> {
        Err(CompositeRecordingErrorV1::InvalidStream)
    }
    fn begin_molecule_target_group(
        &mut self,
        target: &RenderTarget,
        _: u32,
        space: BatchSpace,
    ) -> Result<(), Self::Error> {
        if !self.root_open
            || self.root_kind != Some(CompositeRootKindV1::Molecule)
            || self.target_open
            || self.document_text_open
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.count(
            CompositeRecordingResourceV1::TargetGroups,
            self.budget.max_target_groups,
        )?;
        let target = target.clone();
        self.target_open = true;
        self.push(CompositeRecordingEventV1::TargetBegin {
            target: Some(target),
            space: Some(space),
            direct: false,
        })
    }
    fn end_molecule_batch(&mut self) -> Result<(), Self::Error> {
        if !self.target_open || self.transform_depth != 0 {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.target_open = false;
        self.push(CompositeRecordingEventV1::TargetEnd)
    }
    fn begin_direct_target_group(&mut self, _: u32) -> Result<(), Self::Error> {
        if !self.root_open
            || self.root_kind != Some(CompositeRootKindV1::Molecule)
            || self.target_open
            || self.document_text_open
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.count(
            CompositeRecordingResourceV1::TargetGroups,
            self.budget.max_target_groups,
        )?;
        self.target_open = true;
        self.push(CompositeRecordingEventV1::TargetBegin {
            target: None,
            space: Some(BatchSpace::Scene),
            direct: true,
        })
    }
    fn end_direct_target_group(&mut self) -> Result<(), Self::Error> {
        self.end_molecule_batch()
    }
    fn begin_document_text(&mut self) -> Result<(), Self::Error> {
        if !self.root_open
            || self.root_kind != Some(CompositeRootKindV1::Text)
            || self.target_open
            || self.document_text_open
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.count(
            CompositeRecordingResourceV1::TextScopes,
            self.budget.max_text_scopes,
        )?;
        self.document_text_open = true;
        self.push(CompositeRecordingEventV1::DocumentTextBegin)
    }
    fn end_document_text(&mut self) -> Result<(), Self::Error> {
        if !self.document_text_open || self.text_operation_open || self.transform_depth != 0 {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.document_text_open = false;
        self.push(CompositeRecordingEventV1::DocumentTextEnd)
    }
    fn begin_text_operation(&mut self, z: i32, paint: &Paint) -> Result<(), Self::Error> {
        if !self.root_open
            || (!self.document_text_open && !self.target_open)
            || self.text_operation_open
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.count(
            CompositeRecordingResourceV1::TextScopes,
            self.budget.max_text_scopes,
        )?;
        let paint = paint.clone();
        self.text_operation_open = true;
        self.push(CompositeRecordingEventV1::TextOperationBegin { z, paint })
    }
    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        if !self.text_operation_open {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.text_operation_open = false;
        self.push(CompositeRecordingEventV1::TextOperationEnd)
    }
    fn save(&mut self) -> Result<(), Self::Error> {
        if !self.root_open {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.count(
            CompositeRecordingResourceV1::TransformDepth,
            self.budget.max_transform_depth,
        )?;
        self.push(CompositeRecordingEventV1::Save)
    }
    fn concat_translate(&mut self, point: RenderPoint) -> Result<(), Self::Error> {
        if self.transform_depth == 0 {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.push(CompositeRecordingEventV1::Translate { point })
    }
    fn restore(&mut self) -> Result<(), Self::Error> {
        if self.transform_depth == 0 {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.transform_depth -= 1;
        self.push(CompositeRecordingEventV1::Restore)
    }
    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        paint: &Paint,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.require_paint_scope()?;
        let paint = paint.clone();
        let paint_index = self.paint_index()?;
        self.push(CompositeRecordingEventV1::FillRect {
            origin: rect.origin,
            width: rect.width,
            height: rect.height,
            paint,
            kind: kind(metadata),
            paint_index,
        })
    }
    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.require_paint_scope()?;
        let commands = self.path(path)?;
        let style = self.style(style)?;
        let paint_index = self.paint_index()?;
        self.push(CompositeRecordingEventV1::Path {
            commands,
            style,
            kind: kind(metadata),
            paint_index,
        })
    }
    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        metadata: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        self.require_paint_scope()?;
        if !ellipse.rotation_degrees.is_finite() {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        let style = self.style(style)?;
        let paint_index = self.paint_index()?;
        self.push(CompositeRecordingEventV1::Ellipse {
            center: ellipse.center,
            radius_x: ellipse.radius_x,
            radius_y: ellipse.radius_y,
            rotation_degrees: ellipse.rotation_degrees,
            style,
            kind: kind(metadata),
            paint_index,
        })
    }
    fn finish_page(&mut self) -> Result<(), Self::Error> {
        if !self.page_open
            || self.root_open
            || self.target_open
            || self.document_text_open
            || self.text_operation_open
            || self.transform_depth != 0
        {
            return Err(CompositeRecordingErrorV1::InvalidStream);
        }
        self.page_open = false;
        self.page_finished = true;
        self.push(CompositeRecordingEventV1::PageEnd)
    }
}
