use super::*;

pub(super) fn toggle_roots(
    mut current: Vec<RenderInteractionRootV1>,
    candidates: Vec<RenderInteractionRootV1>,
) -> Vec<RenderInteractionRootV1> {
    for candidate in candidates {
        if let Some(index) = current
            .iter()
            .position(|root| root.document_object_id == candidate.document_object_id)
        {
            current.remove(index);
        } else {
            current.push(candidate);
        }
    }
    // Membership follows the gesture, while representation follows the
    // renderer-issued canonical paint order.
    current.sort_by_key(RenderInteractionRootV1::paint_order);
    current
}
pub(super) fn square_bounds(x: f64, y: f64, radius: f64) -> RenderInteractionBoundsV1 {
    RenderInteractionBoundsV1 {
        left: x - radius,
        top: y - radius,
        right: x + radius,
        bottom: y + radius,
    }
}
pub(super) fn segment_distance(x: f64, y: f64, segment: StructureSegmentV1) -> f64 {
    let dx = segment.end_x - segment.start_x;
    let dy = segment.end_y - segment.start_y;
    let length = dx * dx + dy * dy;
    let t = if length == 0.0 {
        0.0
    } else {
        (((x - segment.start_x) * dx + (y - segment.start_y) * dy) / length).clamp(0.0, 1.0)
    };
    (x - (segment.start_x + t * dx)).hypot(y - (segment.start_y + t * dy))
}
pub(super) fn segment_bounds(segments: &[StructureSegmentV1]) -> RenderInteractionBoundsV1 {
    let first = segments[0];
    segments.iter().skip(1).fold(
        RenderInteractionBoundsV1 {
            left: first.start_x.min(first.end_x) - first.stroke_radius,
            top: first.start_y.min(first.end_y) - first.stroke_radius,
            right: first.start_x.max(first.end_x) + first.stroke_radius,
            bottom: first.start_y.max(first.end_y) + first.stroke_radius,
        },
        |bounds, value| RenderInteractionBoundsV1 {
            left: bounds
                .left
                .min(value.start_x.min(value.end_x) - value.stroke_radius),
            top: bounds
                .top
                .min(value.start_y.min(value.end_y) - value.stroke_radius),
            right: bounds
                .right
                .max(value.start_x.max(value.end_x) + value.stroke_radius),
            bottom: bounds
                .bottom
                .max(value.start_y.max(value.end_y) + value.stroke_radius),
        },
    )
}
pub(super) fn union_bounds(values: &[RenderInteractionBoundsV1]) -> RenderInteractionBoundsV1 {
    let first = values[0];
    values
        .iter()
        .skip(1)
        .fold(first, |bounds, value| RenderInteractionBoundsV1 {
            left: bounds.left.min(value.left),
            top: bounds.top.min(value.top),
            right: bounds.right.max(value.right),
            bottom: bounds.bottom.max(value.bottom),
        })
}

/// Return the finite renderer-issued bounds for one lowered scene path.
pub(super) fn path_bounds(path: &PathOpV3) -> RenderInteractionBoundsV1 {
    let mut points = Vec::new();
    for command in path.commands() {
        match command {
            ScenePathCommandV3::MoveTo(point) | ScenePathCommandV3::LineTo(point) => {
                points.push((point.x(), point.y()));
            }
            ScenePathCommandV3::CubicTo {
                control_1,
                control_2,
                end,
            } => {
                points.push((control_1.x(), control_1.y()));
                points.push((control_2.x(), control_2.y()));
                points.push((end.x(), end.y()));
            }
            ScenePathCommandV3::Close => {}
        }
    }
    let (first_x, first_y) = points[0];
    let bounds = points.iter().skip(1).fold(
        RenderInteractionBoundsV1 {
            left: first_x,
            top: first_y,
            right: first_x,
            bottom: first_y,
        },
        |bounds, (x, y)| RenderInteractionBoundsV1 {
            left: bounds.left.min(*x),
            top: bounds.top.min(*y),
            right: bounds.right.max(*x),
            bottom: bounds.bottom.max(*y),
        },
    );
    inflate_bounds(
        bounds,
        path.stroke().map_or(0.0, |stroke| stroke.width().get()),
    )
}
pub(super) fn toggle_structure_targets(
    mut current: Vec<StructureInteractionTargetV1>,
    candidates: Vec<StructureInteractionTargetV1>,
) -> Vec<StructureInteractionTargetV1> {
    for candidate in candidates {
        if let Some(index) = current.iter().position(|value| {
            value.kind == candidate.kind
                && value.object_id == candidate.object_id
                && value.molecule_object_id == candidate.molecule_object_id
        }) {
            current.remove(index);
        } else {
            current.push(candidate);
        }
    }
    current
}
pub(super) fn roots_from_render(
    rendered: &DocumentRenderObservationV1,
    presentation_plan: &PresentationRenderPlanV1,
    _identities: &CompleteDocumentIdentityFactsV1,
) -> (
    Vec<RenderInteractionRootV1>,
    Vec<RenderInteractionExclusionV1>,
) {
    let mut presentation_roots = HashMap::new();
    for root in presentation_plan.roots() {
        let target = root.target();
        if let Some(bounds) = presentation_bounds_from_plan(root) {
            presentation_roots.insert(
                target.document_object_id().clone(),
                (presentation_root_kind(target.record_kind()), bounds),
            );
        }
    }
    let mut roots = Vec::new();
    let mut exclusions = Vec::new();
    let mut emitted_exclusions = HashSet::new();
    let document_plan = compose_document_render_plan_v1(rendered.resolved())
        .expect("authenticated render observation composes into a document render plan");
    for outcome in document_plan.outcomes() {
        match outcome {
            DocumentRenderOutcomeV1::Root(root) => {
                let document_object_id = root.target().document_object_id().clone();
                let root_facts = match root.content() {
                    DocumentRenderContentV1::Molecule(plan) => {
                        measure_molecule_render_plan_bounds_v1(plan)
                            .ok()
                            .map(|bounds| {
                                (
                                    TopLevelRootKindV1::Molecule,
                                    RenderInteractionBoundsV1 {
                                        left: bounds.left(),
                                        top: bounds.top(),
                                        right: bounds.right(),
                                        bottom: bounds.bottom(),
                                    },
                                )
                            })
                    }
                    DocumentRenderContentV1::Text(_) | DocumentRenderContentV1::Vector(_) => {
                        presentation_roots.get(&document_object_id).copied()
                    }
                };
                if let Some((kind, bounds)) = root_facts {
                    roots.push(RenderInteractionRootV1 {
                        document_object_id,
                        paint_order: root.paint_order(),
                        bounds,
                        kind,
                    });
                } else if emitted_exclusions.insert(document_object_id.clone()) {
                    exclusions.push(RenderInteractionExclusionV1 {
                        document_object_id,
                        reason: RenderInteractionExclusionReasonV1::UnrenderableDepiction,
                    });
                }
            }
            DocumentRenderOutcomeV1::Exclusion(exclusion) => {
                let document_object_id = exclusion.target().document_object_id().clone();
                if emitted_exclusions.insert(document_object_id.clone()) {
                    exclusions.push(RenderInteractionExclusionV1 {
                        document_object_id,
                        reason: RenderInteractionExclusionReasonV1::DisplayOnly,
                    });
                }
            }
        }
    }
    roots.sort_by_key(RenderInteractionRootV1::paint_order);
    (roots, exclusions)
}

pub(super) fn presentation_root_kind(kind: PresentationRecordKindV1) -> TopLevelRootKindV1 {
    match kind {
        PresentationRecordKindV1::Arrow => TopLevelRootKindV1::Arrow,
        PresentationRecordKindV1::Plus => TopLevelRootKindV1::Plus,
        PresentationRecordKindV1::Text => TopLevelRootKindV1::Text,
        PresentationRecordKindV1::Rectangle => TopLevelRootKindV1::Rectangle,
        PresentationRecordKindV1::Square => TopLevelRootKindV1::Square,
        PresentationRecordKindV1::Oval => TopLevelRootKindV1::Oval,
        PresentationRecordKindV1::Circle => TopLevelRootKindV1::Circle,
        PresentationRecordKindV1::Polygon => TopLevelRootKindV1::Polygon,
        PresentationRecordKindV1::Polyline => TopLevelRootKindV1::Polyline,
    }
}

pub(super) fn reaction_choice_kind(
    root_kind: TopLevelRootKindV1,
) -> Option<ReactionAuthoringChoiceKindV1> {
    match root_kind {
        TopLevelRootKindV1::Molecule => Some(ReactionAuthoringChoiceKindV1::Molecule),
        TopLevelRootKindV1::Arrow => Some(ReactionAuthoringChoiceKindV1::Arrow),
        TopLevelRootKindV1::Plus => Some(ReactionAuthoringChoiceKindV1::Plus),
        TopLevelRootKindV1::Text => Some(ReactionAuthoringChoiceKindV1::ConditionText),
        _ => None,
    }
}

pub(super) fn reaction_exclusion_recovery(
    reason: ReactionAuthoringExclusionReasonV1,
) -> ReactionAuthoringExclusionRecoveryV1 {
    match reason {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => {
            ReactionAuthoringExclusionRecoveryV1::ChooseSupportedMember
        }
        ReactionAuthoringExclusionReasonV1::Unrenderable
        | ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity
        | ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity
        | ReactionAuthoringExclusionReasonV1::KindMismatch => {
            ReactionAuthoringExclusionRecoveryV1::RepairDocument
        }
    }
}

pub(super) fn reaction_exclusion_label(reason: ReactionAuthoringExclusionReasonV1) -> String {
    let description = match reason {
        ReactionAuthoringExclusionReasonV1::DisplayOnly => "Display-only root",
        ReactionAuthoringExclusionReasonV1::Unrenderable => "Unrenderable root",
        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity => {
            "Root missing direct CDML identity"
        }
        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity => {
            "Root with ambiguous direct CDML identity"
        }
        ReactionAuthoringExclusionReasonV1::KindMismatch => {
            "Root with renderer/semantic kind mismatch"
        }
    };
    description.to_owned()
}

pub(super) fn push_reaction_exclusion(
    exclusions: &mut Vec<ReactionAuthoringExclusionV1>,
    diagnosed: &mut HashSet<String>,
    identifier: &str,
    reason: ReactionAuthoringExclusionReasonV1,
    label: String,
) {
    if diagnosed.insert(identifier.to_owned()) {
        exclusions.push(ReactionAuthoringExclusionV1 {
            diagnostic_key: identifier.to_owned(),
            reason,
            recovery: reaction_exclusion_recovery(reason),
            label,
        });
    }
}

pub(super) fn reaction_choice_label(kind: ReactionAuthoringChoiceKindV1) -> String {
    let name = match kind {
        ReactionAuthoringChoiceKindV1::Molecule => "Molecule",
        ReactionAuthoringChoiceKindV1::Arrow => "Arrow",
        ReactionAuthoringChoiceKindV1::Plus => "Plus",
        ReactionAuthoringChoiceKindV1::ConditionText => "Condition text",
    };
    name.to_owned()
}

pub(super) fn presentation_bounds_from_plan(
    root: &PresentationRenderRootV1,
) -> Option<RenderInteractionBoundsV1> {
    let bounds = root.bounds();
    Some(RenderInteractionBoundsV1 {
        left: bounds.left(),
        top: bounds.top(),
        right: bounds.right(),
        bottom: bounds.bottom(),
    })
}

pub(super) fn inflate_bounds(
    bounds: RenderInteractionBoundsV1,
    stroke_width: f64,
) -> RenderInteractionBoundsV1 {
    let half = stroke_width / 2.0;
    RenderInteractionBoundsV1 {
        left: bounds.left - half,
        top: bounds.top - half,
        right: bounds.right + half,
        bottom: bounds.bottom + half,
    }
}
