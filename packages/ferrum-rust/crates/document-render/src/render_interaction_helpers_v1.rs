use super::*;

pub(super) fn toggle_roots(
    mut current: Vec<RenderInteractionRootV1>,
    candidates: Vec<RenderInteractionRootV1>,
) -> Vec<RenderInteractionRootV1> {
    for candidate in candidates {
        if let Some(index) = current
            .iter()
            .position(|root| root.identifier == candidate.identifier)
        {
            current.remove(index);
        } else {
            current.push(candidate);
        }
    }
    // Membership follows the gesture, while representation follows the
    // document's Rust-owned canonical source order.
    current.sort_by_key(|root| root.source_order);
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

/// Return a conservative, renderer-issued envelope for a path-only bond.
///
/// P0.3 intentionally refuses path depictions rather than collapsing a wedge,
/// hash, or future filled bond into a fake editable centerline.  The envelope
/// includes every lowered path command and physical stroke width solely to
/// route an actual displayed primitive to the typed `DisplayOnly` recovery.
pub(super) fn path_bounds(path: &PathOpV2) -> RenderInteractionBoundsV1 {
    let mut points = Vec::new();
    for command in path.commands() {
        match command {
            ScenePathCommandV2::MoveTo(point) | ScenePathCommandV2::LineTo(point) => {
                points.push((point.x(), point.y()));
            }
            ScenePathCommandV2::CubicTo {
                control_1,
                control_2,
                end,
            } => {
                points.push((control_1.x(), control_1.y()));
                points.push((control_2.x(), control_2.y()));
                points.push((end.x(), end.y()));
            }
            ScenePathCommandV2::Close => {}
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
pub(super) fn structure_deletion_receipt(
    receipt: StructureDeletionReceiptV1,
) -> (
    Vec<String>,
    Vec<String>,
    Vec<StructureDeletionComponentFactsV1>,
) {
    let atoms = receipt
        .removed_atom_ids()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    let bonds = receipt
        .removed_bond_ids()
        .iter()
        .map(|id| id.as_str().to_owned())
        .collect();
    let components = receipt
        .components()
        .iter()
        .map(|component| StructureDeletionComponentFactsV1 {
            molecule_id: component.molecule_id().as_str().to_owned(),
            atom_ids: component
                .atom_ids()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
            bond_ids: component
                .bond_ids()
                .iter()
                .map(|id| id.as_str().to_owned())
                .collect(),
        })
        .collect();
    (atoms, bonds, components)
}
pub(super) fn toggle_structure_targets(
    mut current: Vec<StructureInteractionTargetV1>,
    candidates: Vec<StructureInteractionTargetV1>,
) -> Vec<StructureInteractionTargetV1> {
    for candidate in candidates {
        if let Some(index) = current.iter().position(|value| {
            value.kind == candidate.kind
                && value.identifier == candidate.identifier
                && value.molecule_id == candidate.molecule_id
        }) {
            current.remove(index);
        } else {
            current.push(candidate);
        }
    }
    current
}
pub(super) fn roots_from_render(
    rendered: &RenderObservationV1,
    identities: &CompleteDocumentIdentityFactsV1,
) -> (
    Vec<RenderInteractionRootV1>,
    Vec<RenderInteractionExclusionV1>,
) {
    let mut planned = HashMap::new();
    for entry in rendered.molecule_plans() {
        let Some(identifier) = entry.molecule().source_id() else {
            continue;
        };
        if let Ok(bounds) = measure_molecule_render_plan_bounds_v1(entry.plan()) {
            planned.insert(
                identifier.to_owned(),
                RenderInteractionBoundsV1 {
                    left: bounds.left(),
                    top: bounds.top(),
                    right: bounds.right(),
                    bottom: bounds.bottom(),
                },
            );
        }
    }
    let mut roots = Vec::new();
    let mut exclusions = Vec::new();
    let mut emitted_exclusions = HashSet::new();
    for molecule in rendered.document().projection().molecules() {
        let (Some(_), Some(identifier)) = (molecule.id(), molecule.source_id()) else {
            continue;
        };
        let exclusion = root_exclusion_reason(identifier, identities, planned.get(identifier));
        if exclusion.is_none() {
            let bounds = planned
                .get(identifier)
                .expect("authorable root has a measured render-plan bounds");
            roots.push(RenderInteractionRootV1 {
                identifier: identifier.to_owned(),
                source_order: molecule.source_order(),
                bounds: *bounds,
                kind: TopLevelRootKindV1::Molecule,
            });
        }
        if let Some(reason) = exclusion
            && emitted_exclusions.insert(identifier.to_owned())
        {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier: identifier.to_owned(),
                reason,
            });
        }
    }
    for root in rendered
        .document()
        .projection()
        .presentation_stack()
        .roots()
    {
        let target = root.target();
        let diagnostic_identifier = target.source_id().map_or_else(
            || target.projection_key().as_str().to_owned(),
            str::to_owned,
        );
        let Some(identifier) = target.source_id().filter(|_| target.id().is_some()) else {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier: diagnostic_identifier,
                reason: RenderInteractionExclusionReasonV1::DisplayOnly,
            });
            continue;
        };
        let bounds = presentation_bounds_from_render(root, rendered);
        let exclusion = root_exclusion_reason(identifier, identities, bounds.as_ref());
        if let Some(bounds) = bounds.filter(|_| exclusion.is_none()) {
            roots.push(RenderInteractionRootV1 {
                identifier: identifier.to_owned(),
                source_order: target.source_order(),
                bounds,
                kind: presentation_root_kind(root),
            });
        }
        if let Some(reason) = exclusion
            && emitted_exclusions.insert(identifier.to_owned())
        {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier: identifier.to_owned(),
                reason,
            });
        }
    }
    for issue in rendered
        .document()
        .projection()
        .presentation_stack()
        .issues()
    {
        let target = issue.target();
        let identifier = target.source_id().map_or_else(
            || target.projection_key().as_str().to_owned(),
            str::to_owned,
        );
        if emitted_exclusions.insert(identifier.clone()) {
            exclusions.push(RenderInteractionExclusionV1 {
                identifier,
                reason: RenderInteractionExclusionReasonV1::DisplayOnly,
            });
        }
    }
    roots.sort_by_key(RenderInteractionRootV1::source_order);
    (roots, exclusions)
}

pub(super) fn presentation_root_kind(root: &PresentationRootProjectionV1) -> TopLevelRootKindV1 {
    match root {
        PresentationRootProjectionV1::Arrow { .. } => TopLevelRootKindV1::Arrow,
        PresentationRootProjectionV1::Plus { .. } => TopLevelRootKindV1::Plus,
        PresentationRootProjectionV1::Text { .. } => TopLevelRootKindV1::Text,
        PresentationRootProjectionV1::Rectangle { .. } => TopLevelRootKindV1::Rectangle,
        PresentationRootProjectionV1::Square { .. } => TopLevelRootKindV1::Square,
        PresentationRootProjectionV1::Oval { .. } => TopLevelRootKindV1::Oval,
        PresentationRootProjectionV1::Circle { .. } => TopLevelRootKindV1::Circle,
        PresentationRootProjectionV1::Polygon { .. } => TopLevelRootKindV1::Polygon,
        PresentationRootProjectionV1::Polyline { .. }
        | PresentationRootProjectionV1::Wavy { .. }
        | PresentationRootProjectionV1::RoundBracket { .. } => TopLevelRootKindV1::Polyline,
    }
}

pub(super) fn reaction_choice_kind(
    semantic: DirectCdmlRootKindV1,
    observed: TopLevelRootKindV1,
) -> Option<ReactionAuthoringChoiceKindV1> {
    match (semantic, observed) {
        (DirectCdmlRootKindV1::Molecule, TopLevelRootKindV1::Molecule) => {
            Some(ReactionAuthoringChoiceKindV1::Molecule)
        }
        (DirectCdmlRootKindV1::Arrow, TopLevelRootKindV1::Arrow) => {
            Some(ReactionAuthoringChoiceKindV1::Arrow)
        }
        (DirectCdmlRootKindV1::Plus, TopLevelRootKindV1::Plus) => {
            Some(ReactionAuthoringChoiceKindV1::Plus)
        }
        (DirectCdmlRootKindV1::Text, TopLevelRootKindV1::Text) => {
            Some(ReactionAuthoringChoiceKindV1::ConditionText)
        }
        _ => None,
    }
}

pub(super) fn reaction_root_exclusion_reason(
    semantic: DirectCdmlRootKindV1,
    observed: TopLevelRootKindV1,
) -> ReactionAuthoringExclusionReasonV1 {
    if direct_reaction_choice_kind(semantic).is_some() {
        debug_assert!(reaction_choice_kind(semantic, observed).is_none());
        ReactionAuthoringExclusionReasonV1::KindMismatch
    } else {
        ReactionAuthoringExclusionReasonV1::DisplayOnly
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

pub(super) fn reaction_exclusion_label(
    reason: ReactionAuthoringExclusionReasonV1,
    identifier: &str,
) -> String {
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
    format!("{description} {identifier}")
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

pub(super) fn direct_reaction_choice_kind(
    kind: DirectCdmlRootKindV1,
) -> Option<ReactionAuthoringChoiceKindV1> {
    match kind {
        DirectCdmlRootKindV1::Molecule => Some(ReactionAuthoringChoiceKindV1::Molecule),
        DirectCdmlRootKindV1::Arrow => Some(ReactionAuthoringChoiceKindV1::Arrow),
        DirectCdmlRootKindV1::Plus => Some(ReactionAuthoringChoiceKindV1::Plus),
        DirectCdmlRootKindV1::Text => Some(ReactionAuthoringChoiceKindV1::ConditionText),
        DirectCdmlRootKindV1::Reaction | DirectCdmlRootKindV1::Other => None,
    }
}

pub(super) fn reaction_choice_label(
    kind: ReactionAuthoringChoiceKindV1,
    identifier: &str,
) -> String {
    let name = match kind {
        ReactionAuthoringChoiceKindV1::Molecule => "Molecule",
        ReactionAuthoringChoiceKindV1::Arrow => "Arrow",
        ReactionAuthoringChoiceKindV1::Plus => "Plus",
        ReactionAuthoringChoiceKindV1::ConditionText => "Condition text",
    };
    format!("{name} {identifier}")
}

pub(super) fn presentation_bounds_from_render(
    root: &PresentationRootProjectionV1,
    rendered: &RenderObservationV1,
) -> Option<RenderInteractionBoundsV1> {
    match root {
        PresentationRootProjectionV1::Plus { plus } => rendered
            .plus_renders()
            .iter()
            .find(|value| value.target().projection_key() == plus.target().projection_key())
            .map(|value| text_bounds(value.anchor().x(), value.anchor().y(), value.bounds())),
        PresentationRootProjectionV1::Text { text } => rendered
            .text_renders()
            .iter()
            .find(|value| value.target().projection_key() == text.target().projection_key())
            .map(|value| text_bounds(value.anchor().x(), value.anchor().y(), value.bounds())),
        PresentationRootProjectionV1::Arrow { arrow } => match arrow.geometry() {
            ferrum_document::ArrowDisplayGeometryV1::Normal {
                axis_path, heads, ..
            } => bounds_from_points(
                axis_path
                    .points()
                    .iter()
                    .chain(heads.iter().flat_map(|head| head.points().iter())),
                arrow.stroke().width().value(),
            ),
            ferrum_document::ArrowDisplayGeometryV1::Equilibrium { axes, heads } => {
                bounds_from_points(
                    axes.iter()
                        .flat_map(|axis| axis.points().iter())
                        .chain(heads.iter().flat_map(|head| head.points().iter())),
                    arrow.stroke().width().value(),
                )
            }
            ferrum_document::ArrowDisplayGeometryV1::CurvedEquilibrium { axes, heads, .. } => {
                bounds_from_points(
                    axes.iter()
                        .flat_map(|axis| axis.points().iter())
                        .chain(heads.iter().flat_map(|head| head.points().iter())),
                    arrow.stroke().width().value(),
                )
            }
            ferrum_document::ArrowDisplayGeometryV1::CurvedTerminal {
                axis_path, head, ..
            } => bounds_from_points(
                axis_path.points().iter().chain(head.points().iter()),
                arrow.stroke().width().value(),
            ),
        },
        PresentationRootProjectionV1::Polyline { polyline }
        | PresentationRootProjectionV1::Wavy { polyline }
        | PresentationRootProjectionV1::RoundBracket { polyline } => bounds_from_points(
            polyline.path().points().iter(),
            polyline.stroke().width().value(),
        ),
        PresentationRootProjectionV1::Rectangle { shape }
        | PresentationRootProjectionV1::Square { shape }
        | PresentationRootProjectionV1::Oval { shape }
        | PresentationRootProjectionV1::Circle { shape } => {
            let bounds = shape.bounds();
            Some(inflate_bounds(
                RenderInteractionBoundsV1 {
                    left: bounds.left(),
                    top: bounds.top(),
                    right: bounds.right(),
                    bottom: bounds.bottom(),
                },
                shape.stroke().width().value(),
            ))
        }
        PresentationRootProjectionV1::Polygon { polygon } => bounds_from_points(
            polygon.path().points().iter(),
            polygon.stroke().width().value(),
        ),
    }
}

pub(super) fn text_bounds(
    anchor_x: f64,
    anchor_y: f64,
    bounds: ferrum_render::PresentationTextBoundsV1,
) -> RenderInteractionBoundsV1 {
    RenderInteractionBoundsV1 {
        left: anchor_x + bounds.left(),
        top: anchor_y + bounds.top(),
        right: anchor_x + bounds.right(),
        bottom: anchor_y + bounds.bottom(),
    }
}

pub(super) fn bounds_from_points<'a>(
    points: impl Iterator<Item = &'a Point3V1>,
    stroke_width: f64,
) -> Option<RenderInteractionBoundsV1> {
    let mut points = points.peekable();
    let first = *points.peek()?;
    let mut bounds = RenderInteractionBoundsV1 {
        left: first.x(),
        top: first.y(),
        right: first.x(),
        bottom: first.y(),
    };
    for point in points {
        bounds.left = bounds.left.min(point.x());
        bounds.top = bounds.top.min(point.y());
        bounds.right = bounds.right.max(point.x());
        bounds.bottom = bounds.bottom.max(point.y());
    }
    Some(inflate_bounds(bounds, stroke_width))
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

pub(super) fn root_exclusion_reason(
    identifier: &str,
    identities: &CompleteDocumentIdentityFactsV1,
    bounds: Option<&RenderInteractionBoundsV1>,
) -> Option<RenderInteractionExclusionReasonV1> {
    if identities.is_ambiguous_identifier(identifier) {
        Some(RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier)
    } else if bounds.is_none() {
        Some(RenderInteractionExclusionReasonV1::UnrenderableDepiction)
    } else {
        None
    }
}
