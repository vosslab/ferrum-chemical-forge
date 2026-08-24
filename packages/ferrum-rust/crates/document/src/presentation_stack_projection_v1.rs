//! Typed-CDML adapter for immutable direct-root presentation projections.

use std::collections::BTreeSet;

use ferrum_document_projection::{
    PresentationBracketStyleV1, PresentationRecordKindV1, PresentationRootProjectionV1,
    PresentationStackProjectionV1, PresentationTargetV1,
};

use super::presentation_arrow_projection_v1::arrow;
use super::presentation_plus_projection_v1::plus;
use super::presentation_polyline_projection_v1::{
    PolylineProjectionKindV1, RootStrokeDefaultsV1, polyline,
};
use super::presentation_shape_projection_v1::{box_shape, polygon};
use super::presentation_text_projection_v1::text;
use super::{DocumentSnapshot, TypedChild, TypedClass, TypedDocument};

pub(crate) fn project_presentation_stack_v1(
    document: &TypedDocument,
    snapshot: &DocumentSnapshot,
) -> Result<PresentationStackProjectionV1, crate::ProjectionError> {
    let defaults = RootStrokeDefaultsV1::from_document(document);
    let bracket_pairs = super::bracket_pair_projection_v1::bracket_pairs(document);
    let round_members = bracket_pairs
        .iter()
        .filter(|pair| pair.style() == PresentationBracketStyleV1::Round)
        .flat_map(|pair| pair.member_ids().iter().map(String::as_str))
        .collect::<BTreeSet<_>>();
    let mut roots = Vec::new();
    let mut issues = Vec::new();
    for child in document.root().typed_children() {
        match child.record().class() {
            TypedClass::CanvasArrow => push_root(
                &mut roots,
                arrow(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::arrow,
            )?,
            TypedClass::CanvasPlus => push_root(
                &mut roots,
                plus(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::plus,
            )?,
            TypedClass::CanvasText => push_root(
                &mut roots,
                text(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::text,
            )?,
            TypedClass::Polyline => {
                let round_bracket_member = child
                    .record()
                    .attribute("id")
                    .is_some_and(|identifier| round_members.contains(identifier));
                if let Some((kind, polyline)) =
                    polyline(child, defaults, round_bracket_member, &mut issues)?
                {
                    let root = match kind {
                        PolylineProjectionKindV1::Ordinary => {
                            PresentationRootProjectionV1::polyline(polyline)
                        }
                        PolylineProjectionKindV1::Wavy => {
                            PresentationRootProjectionV1::wavy(polyline)
                        }
                        PolylineProjectionKindV1::RoundBracket => {
                            PresentationRootProjectionV1::round_bracket(polyline)
                        }
                    };
                    roots.push(root.map_err(projection_construction_error)?);
                }
            }
            TypedClass::Rectangle => push_root(
                &mut roots,
                box_shape(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::rectangle,
            )?,
            TypedClass::Square => push_root(
                &mut roots,
                box_shape(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::square,
            )?,
            TypedClass::Oval => push_root(
                &mut roots,
                box_shape(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::oval,
            )?,
            TypedClass::Circle => push_root(
                &mut roots,
                box_shape(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::circle,
            )?,
            TypedClass::Polygon => push_root(
                &mut roots,
                polygon(child, defaults, &mut issues)?,
                PresentationRootProjectionV1::polygon,
            )?,
            TypedClass::Cdml
            | TypedClass::Info
            | TypedClass::Metadata
            | TypedClass::Standard
            | TypedClass::Paper
            | TypedClass::Viewport
            | TypedClass::Molecule
            | TypedClass::Reaction
            | TypedClass::ExternalData => {}
            class => {
                return Err(crate::ProjectionError::InvalidValue {
                    context: child.record().path().to_string(),
                    field: "presentation root",
                    value: class.name().to_owned(),
                });
            }
        }
    }
    PresentationStackProjectionV1::new(
        snapshot.revision(),
        *snapshot.digest(),
        roots,
        bracket_pairs,
        issues,
    )
    .map_err(|error| crate::ProjectionError::InvalidValue {
        context: "presentation stack".to_owned(),
        field: "round bracket roots",
        value: error.to_string(),
    })
}

fn push_root<T>(
    roots: &mut Vec<PresentationRootProjectionV1>,
    value: Option<T>,
    wrap: impl FnOnce(
        T,
    ) -> Result<
        PresentationRootProjectionV1,
        ferrum_document_projection::PresentationStackProjectionV1Error,
    >,
) -> Result<(), crate::ProjectionError> {
    if let Some(value) = value {
        roots.push(wrap(value).map_err(projection_construction_error)?);
    }
    Ok(())
}

fn projection_construction_error(
    error: ferrum_document_projection::PresentationStackProjectionV1Error,
) -> crate::ProjectionError {
    crate::ProjectionError::InvalidValue {
        context: "presentation stack".to_owned(),
        field: "projection",
        value: error.to_string(),
    }
}

pub(crate) fn presentation_target_from_child_v1(
    child: &TypedChild,
) -> Result<PresentationTargetV1, crate::ProjectionError> {
    let record = child.record();
    let record_kind = presentation_record_kind_from_class_v1(record.class()).ok_or_else(|| {
        crate::ProjectionError::InvalidValue {
            context: record.path().to_string(),
            field: "presentation record kind",
            value: record.class().name().to_owned(),
        }
    })?;
    PresentationTargetV1::try_new(
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?,
        crate::projection_local_object_key_from_record_v1(record)?,
        record.attribute("id").map(str::to_owned),
        child.position(),
        record_kind,
    )
    .map_err(projection_construction_error)
}

fn presentation_record_kind_from_class_v1(class: TypedClass) -> Option<PresentationRecordKindV1> {
    match class {
        TypedClass::CanvasArrow => Some(PresentationRecordKindV1::Arrow),
        TypedClass::CanvasPlus => Some(PresentationRecordKindV1::Plus),
        TypedClass::CanvasText => Some(PresentationRecordKindV1::Text),
        TypedClass::Polyline => Some(PresentationRecordKindV1::Polyline),
        TypedClass::Rectangle => Some(PresentationRecordKindV1::Rectangle),
        TypedClass::Square => Some(PresentationRecordKindV1::Square),
        TypedClass::Oval => Some(PresentationRecordKindV1::Oval),
        TypedClass::Circle => Some(PresentationRecordKindV1::Circle),
        TypedClass::Polygon => Some(PresentationRecordKindV1::Polygon),
        _ => None,
    }
}
