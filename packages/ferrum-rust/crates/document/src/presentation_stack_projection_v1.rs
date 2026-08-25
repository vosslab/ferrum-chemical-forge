//! Typed-CDML adapter for immutable direct-root presentation projections.

use std::collections::BTreeSet;

use ferrum_document_projection::{
    BracketPairProjectionV1, DocumentObjectIdV1, PresentationBracketStyleV1,
    PresentationProjectionIssueV1, PresentationRecordKindV1, PresentationRootProjectionV1,
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

pub(crate) struct PresentationProjectionContextV1<'a> {
    defaults: RootStrokeDefaultsV1<'a>,
    bracket_pairs: Vec<BracketPairProjectionV1>,
    round_members: BTreeSet<DocumentObjectIdV1>,
}

impl<'a> PresentationProjectionContextV1<'a> {
    pub(crate) fn new(document: &'a TypedDocument) -> Self {
        let bracket_pairs = crate::bracket_pair_projection_v1::bracket_pairs(document);
        let round_members = bracket_pairs
            .iter()
            .filter(|pair| pair.style() == PresentationBracketStyleV1::Round)
            .flat_map(|pair| pair.members().iter().cloned())
            .collect();
        Self {
            defaults: RootStrokeDefaultsV1::from_document(document),
            bracket_pairs,
            round_members,
        }
    }

    pub(crate) fn project_root(
        &self,
        child: &TypedChild,
        issues: &mut Vec<PresentationProjectionIssueV1>,
    ) -> Result<Option<PresentationRootProjectionV1>, crate::ProjectionError> {
        match child.record().class() {
            TypedClass::CanvasArrow => wrap_root(
                arrow(child, self.defaults, issues)?,
                PresentationRootProjectionV1::arrow,
            ),
            TypedClass::CanvasPlus => wrap_root(
                plus(child, self.defaults, issues)?,
                PresentationRootProjectionV1::plus,
            ),
            TypedClass::CanvasText => wrap_root(
                text(child, self.defaults, issues)?,
                PresentationRootProjectionV1::text,
            ),
            TypedClass::Polyline => {
                let round_bracket_member =
                    crate::projection_identity_v1::projection_document_object_id_from_record_v1(
                        child.record(),
                    )?
                    .is_some_and(|identifier| self.round_members.contains(&identifier));
                let Some((kind, polyline)) =
                    polyline(child, self.defaults, round_bracket_member, issues)?
                else {
                    return Ok(None);
                };
                let root = match kind {
                    PolylineProjectionKindV1::Ordinary => {
                        PresentationRootProjectionV1::polyline(polyline)
                    }
                    PolylineProjectionKindV1::Wavy => PresentationRootProjectionV1::wavy(polyline),
                    PolylineProjectionKindV1::RoundBracket => {
                        PresentationRootProjectionV1::round_bracket(polyline)
                    }
                };
                Ok(Some(root.map_err(projection_construction_error)?))
            }
            TypedClass::Rectangle => wrap_root(
                box_shape(child, self.defaults, issues)?,
                PresentationRootProjectionV1::rectangle,
            ),
            TypedClass::Square => wrap_root(
                box_shape(child, self.defaults, issues)?,
                PresentationRootProjectionV1::square,
            ),
            TypedClass::Oval => wrap_root(
                box_shape(child, self.defaults, issues)?,
                PresentationRootProjectionV1::oval,
            ),
            TypedClass::Circle => wrap_root(
                box_shape(child, self.defaults, issues)?,
                PresentationRootProjectionV1::circle,
            ),
            TypedClass::Polygon => wrap_root(
                polygon(child, self.defaults, issues)?,
                PresentationRootProjectionV1::polygon,
            ),
            class => Err(crate::ProjectionError::InvalidValue {
                context: child.record().path().to_string(),
                field: "presentation root",
                value: class.name().to_owned(),
            }),
        }
    }

    pub(crate) fn into_stack(
        self,
        snapshot: &DocumentSnapshot,
        roots: Vec<PresentationRootProjectionV1>,
        issues: Vec<PresentationProjectionIssueV1>,
    ) -> Result<PresentationStackProjectionV1, crate::ProjectionError> {
        PresentationStackProjectionV1::new(
            snapshot.revision(),
            *snapshot.digest(),
            roots,
            self.bracket_pairs,
            issues,
        )
        .map_err(|error| crate::ProjectionError::InvalidValue {
            context: "presentation stack".to_owned(),
            field: "round bracket roots",
            value: error.to_string(),
        })
    }
}

pub(crate) fn is_presentation_class_v1(class: TypedClass) -> bool {
    presentation_record_kind_from_class_v1(class).is_some()
}

fn wrap_root<T>(
    value: Option<T>,
    wrap: impl FnOnce(
        T,
    ) -> Result<
        PresentationRootProjectionV1,
        ferrum_document_projection::PresentationStackProjectionV1Error,
    >,
) -> Result<Option<PresentationRootProjectionV1>, crate::ProjectionError> {
    value
        .map(wrap)
        .transpose()
        .map_err(projection_construction_error)
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
    Ok(PresentationTargetV1::new(
        crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?
            .ok_or_else(|| crate::ProjectionError::InvalidValue {
                context: record.path().to_string(),
                field: "document object identity",
                value: "missing persisted identity".to_owned(),
            })?,
        record_kind,
    ))
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
