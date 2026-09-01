//! Private atom, bond, and style resolution for the closed Ferrum depiction profile.

use super::depiction_profile::{DepictionIssueCodeV1, DepictionIssueV1, DepictionProfileV1};
use crate::atom_bond::bond::NormalBondEndpointClipPolicy;
use crate::render_target::RenderPlanEntryContextV1;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, AtomMarkRenderFacts, AtomMarkRenderKind,
    AtomNumberLabelFacts, AtomRenderTarget, BondRenderTarget, BondStyle, FontFace, PositiveFinite,
    RenderPaintV3, RenderPoint, RenderTarget, Rgb24, TargetVisibility,
};
use ferrum_core::{BondOrder, BondStyle as DocumentBondStyle, Identifier, RecordId, RecordKind};
use ferrum_document_projection::{
    AtomMarkKindV1, AtomProjectionV1, BondEndpointKindV1, BondProjectionV1,
    DocumentHaworthPositionV1, DocumentObjectIdV1, DocumentProjectionV1,
    DoubleBondCarrierMarkProjectionV1, DoubleBondCarrierMarkV1, PresentationFontFaceV1,
    Rgb24V1 as DocumentRgb24V1, TransparentOrRgb24V1, VisibilityV1,
};
use ferrum_render_contract::MOLECULE_LABEL_RESOURCE_ID;

// BKChem/OASA's default 6 px lane separation accompanies a 1.5 px stroke.
// Preserve that four-stroke proportion in Ferrum's point-space depiction
// instead of carrying the absolute pixel distance across unit systems.
const BUILTIN_BOND_LANE_STROKE_FACTOR: f64 = 4.0;
const BUILTIN_ATOM_NUMBER_FONT_SIZE: f64 = 9.0;
const BUILTIN_ATOM_NUMBER_OFFSET_X: f64 = 8.0;
const BUILTIN_ATOM_NUMBER_OFFSET_Y: f64 = -12.0;
const BUILTIN_BOND_WEDGE_WIDTH: f64 = 5.0;

pub(super) fn apply_double_bond_carrier_marks(
    bonds: &mut [(&BondProjectionV1, BondRenderTarget)],
    marks: &[DoubleBondCarrierMarkProjectionV1],
) -> Result<(), DepictionIssueV1> {
    for mark in marks {
        let central_double_bond = bonds
            .iter()
            .find(|(bond, _)| bond.document_object_id() == mark.central_double_bond())
            .map(|(bond, _)| bond_record_id(bond).expect("resolved bond has a source record ID"))
            .ok_or_else(|| {
                issue(
                    DepictionIssueCodeV1::UnsupportedFeature,
                    mark.central_double_bond().as_str(),
                    "E/Z carrier mark central-double provenance has no render target",
                )
            })?;
        let carrier_index = bonds
            .iter()
            .position(|(bond, _)| bond.document_object_id() == mark.carrier_bond())
            .ok_or_else(|| {
                issue(
                    DepictionIssueCodeV1::UnsupportedFeature,
                    mark.carrier_bond().as_str(),
                    "E/Z carrier mark has no retained carrier render target",
                )
            })?;
        let carrier_bond = bonds[carrier_index].0;
        let shared_endpoint_is_start =
            carrier_bond.start().object_id() == Some(mark.carrier_shared_endpoint());
        if !shared_endpoint_is_start
            && carrier_bond.end().object_id() != Some(mark.carrier_shared_endpoint())
        {
            return Err(issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                mark.carrier_bond().as_str(),
                "E/Z carrier mark shared endpoint is absent from its carrier projection",
            ));
        }
        let direction = match mark.mark() {
            DoubleBondCarrierMarkV1::Up => crate::DoubleBondCarrierMarkDirectionV1::Up,
            DoubleBondCarrierMarkV1::Down => crate::DoubleBondCarrierMarkDirectionV1::Down,
        };
        let carrier = &mut bonds[carrier_index].1;
        *carrier = carrier
            .clone()
            .with_double_bond_carrier_mark(direction, shared_endpoint_is_start, central_double_bond)
            .map_err(|error| {
                issue(
                    DepictionIssueCodeV1::UnsupportedFeature,
                    mark.carrier_bond().as_str(),
                    error.to_string(),
                )
            })?;
    }
    Ok(())
}

pub(super) fn resolve_atom(
    atom: &AtomProjectionV1,
    owner_molecule_object_id: &DocumentObjectIdV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<(AtomRenderTarget, RecordId), DepictionIssueV1> {
    let context = atom_context(atom, owner_molecule_object_id)?;
    let record_id = context.record_id().clone();
    let font = resolved_font(
        projection,
        profile,
        atom.label_font(),
        atom.background_color().or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.area_color())
        }),
    )?;
    let label = resolved_atom_label(atom, projection)?;
    let visibility = match atom.show() {
        Some(VisibilityV1::Disabled) => TargetVisibility::Hidden {
            reason: "author explicitly hid atom".to_owned(),
        },
        _ => TargetVisibility::Visible,
    };
    let number_label = match (atom.number(), atom.show_number()) {
        (Some(number), Some(VisibilityV1::Enabled)) => Some(
            AtomNumberLabelFacts::new(
                number,
                RenderPoint::new(BUILTIN_ATOM_NUMBER_OFFSET_X, BUILTIN_ATOM_NUMBER_OFFSET_Y)
                    .expect("built-in atom number offset is finite"),
                AtomLabelFontProfile::new(
                    font.face().clone(),
                    PositiveFinite::new(BUILTIN_ATOM_NUMBER_FONT_SIZE)
                        .expect("built-in atom number size is positive"),
                    RenderPaintV3::atom_number(),
                ),
            )
            .map_err(|error| {
                issue(
                    DepictionIssueCodeV1::UnsupportedFeature,
                    atom.projection_key().as_str(),
                    error.to_string(),
                )
            })?,
        ),
        _ => None,
    };
    let marks = atom
        .marks()
        .iter()
        .map(|mark| {
            let radians = mark.angle_degrees().to_radians();
            AtomMarkRenderFacts::new(
                render_mark_kind(mark.kind()),
                RenderPoint::new(
                    mark.radial_offset() * radians.cos(),
                    mark.radial_offset() * radians.sin(),
                )?,
                mark.angle_degrees(),
                PositiveFinite::new(mark.size().value())?,
                mark.draw_circle(),
                PositiveFinite::new(mark.line_width().value())?,
                font.paint().clone(),
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                atom.projection_key().as_str(),
                error.to_string(),
            )
        })?;
    let target = AtomRenderTarget::new(
        context,
        RenderPoint::new(atom.position().x(), atom.position().y()).map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                atom.projection_key().as_str(),
                error.to_string(),
            )
        })?,
        label,
        visibility,
    )
    .map(|target| target.with_font_profile(font).with_marks(marks))
    .map_err(|error| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            atom.projection_key().as_str(),
            error.to_string(),
        )
    })?;
    let target = match number_label {
        Some(number_label) => target.with_number_label(number_label),
        None => target,
    };
    Ok((target, record_id))
}

pub(crate) fn resolve_attached_compact_group_anchor_render_facts(
    projection: &DocumentProjectionV1,
    atom: &AtomProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<
    crate::attached_compact_group_pose::AttachedCompactGroupAnchorRenderFacts,
    DepictionIssueV1,
> {
    let font = resolved_font(
        projection,
        profile,
        atom.label_font(),
        atom.background_color().or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.area_color())
        }),
    )?;
    let line_width = resolved_line_width(projection, profile)?;
    let normal_single_clip_policy =
        resolve_normal_single_clip_policy(line_width, font.size(), atom.projection_key().as_str())?;
    Ok(
        crate::attached_compact_group_pose::AttachedCompactGroupAnchorRenderFacts::new(
            RenderPoint::new(atom.position().x(), atom.position().y()).map_err(|error| {
                issue(
                    DepictionIssueCodeV1::UnsupportedFeature,
                    atom.projection_key().as_str(),
                    error.to_string(),
                )
            })?,
            resolved_atom_label(atom, projection)?,
            font,
            resolved_line_paint(projection, profile)?,
            normal_single_clip_policy,
        ),
    )
}

pub(super) fn resolve_normal_single_clip_policy(
    line_width: PositiveFinite,
    font_size: PositiveFinite,
    target_key: &str,
) -> Result<NormalBondEndpointClipPolicy, DepictionIssueV1> {
    NormalBondEndpointClipPolicy::from_depiction(line_width, font_size).map_err(|error| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            target_key,
            format!("normal-single clipping policy is not representable: {error:?}"),
        )
    })
}

fn resolved_atom_label(
    atom: &AtomProjectionV1,
    projection: &DocumentProjectionV1,
) -> Result<AtomLabelFacts, DepictionIssueV1> {
    if atom.label_text().is_some() {
        return Err(issue(
            DepictionIssueCodeV1::UnsupportedRichLabel,
            atom.projection_key().as_str(),
            "V1 supports structured element, hydrogen, and charge labels only",
        ));
    }
    let element = atom.element().ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            atom.projection_key().as_str(),
            "atom element is absent",
        )
    })?;
    let charge = atom
        .formal_charge()
        .unwrap_or_default()
        .try_into()
        .map_err(|_| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                atom.projection_key().as_str(),
                "formal charge is outside V1 label range",
            )
        })?;
    let hydrogens = if effective_hydrogens(atom.hydrogens(), projection) {
        atom.explicit_hydrogens()
            .unwrap_or_default()
            .try_into()
            .map_err(|_| {
                issue(
                    DepictionIssueCodeV1::UnrenderableExplicitHydrogenCount,
                    atom.projection_key().as_str(),
                    "explicit hydrogen count is outside V1 label range",
                )
            })?
    } else {
        0
    };
    AtomLabelFacts::new(element, atom.isotope(), charge, hydrogens).map_err(|error| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            atom.projection_key().as_str(),
            error.to_string(),
        )
    })
}

fn render_mark_kind(kind: AtomMarkKindV1) -> AtomMarkRenderKind {
    match kind {
        AtomMarkKindV1::Plus => AtomMarkRenderKind::Plus,
        AtomMarkKindV1::Minus => AtomMarkRenderKind::Minus,
        AtomMarkKindV1::Radical => AtomMarkRenderKind::Radical,
        AtomMarkKindV1::Biradical => AtomMarkRenderKind::Biradical,
        AtomMarkKindV1::Electronpair => AtomMarkRenderKind::Electronpair,
        AtomMarkKindV1::DottedElectronpair => AtomMarkRenderKind::DottedElectronpair,
        AtomMarkKindV1::PzOrbital => AtomMarkRenderKind::PzOrbital,
    }
}

pub(super) fn resolve_bond(
    bond: &BondProjectionV1,
    owner_molecule_object_id: &DocumentObjectIdV1,
    endpoints: &std::collections::HashMap<DocumentObjectIdV1, RecordId>,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<BondRenderTarget, DepictionIssueV1> {
    let context = bond_context(bond, owner_molecule_object_id)?;
    let first = endpoint_record(bond.start(), endpoints, bond.projection_key().as_str())?;
    let second = endpoint_record(bond.end(), endpoints, bond.projection_key().as_str())?;
    if let Some(value) = bond.bond_width().filter(|value| value.value() < 0.0) {
        // A negative CDML bond_width selects an uncentered double-bond lane side.
        // This profile cannot lower that direction yet.  Keep the authoritative
        // signed fact in the projection and make the durable bond target a plan
        // issue rather than erasing it during positive-scalar resolution.
        return BondRenderTarget::new(
            context,
            first,
            second,
            BondStyle::Unsupported {
                detail: format!(
                    "unsupported signed bond lane placement: bond_width={} requires an uncentered direction-aware double-bond renderer",
                    value.value(),
                ),
            },
            TargetVisibility::Visible,
        )
        .map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                bond.projection_key().as_str(),
                error.to_string(),
            )
        });
    }
    let width = resolved_bond_width(bond, projection, profile)?;
    let lane_spacing = resolved_bond_lane_spacing(bond, projection, profile)?;
    let paint = resolved_bond_paint(bond, projection, profile)?;
    let wedge_width = resolved_bond_wedge_width(bond, projection, profile)?;
    let style = render_bond_style(bond);
    BondRenderTarget::new(context, first, second, style, TargetVisibility::Visible)
        .map(|target| target.with_appearance(width, lane_spacing, wedge_width, paint))
        .map_err(|error| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                bond.projection_key().as_str(),
                error.to_string(),
            )
        })
}

fn render_bond_style(bond: &BondProjectionV1) -> BondStyle {
    match (bond.order(), bond.style(), bond.haworth_position()) {
        (Some(BondOrder::Single), Some(DocumentBondStyle::Normal), _) => BondStyle::NormalSingle,
        (Some(BondOrder::Double), Some(DocumentBondStyle::Normal), _) => BondStyle::Double,
        (Some(BondOrder::Triple), Some(DocumentBondStyle::Normal), _) => BondStyle::Triple,
        (
            Some(BondOrder::Single),
            Some(DocumentBondStyle::Wedge),
            Some(DocumentHaworthPositionV1::Front),
        ) => BondStyle::HaworthFrontWedge,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Wedge), _) => BondStyle::SolidWedge,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Hashed), _) => BondStyle::HashedWedge,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Bold), _) => BondStyle::Bold,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Dashed), _) => BondStyle::Dashed,
        (Some(BondOrder::Single), Some(DocumentBondStyle::Wavy), _) => BondStyle::Wavy,
        (
            Some(BondOrder::Single),
            Some(DocumentBondStyle::HaworthFront),
            Some(DocumentHaworthPositionV1::Front),
        ) => BondStyle::HaworthFrontStroke,
        (order, style, position) => BondStyle::Unsupported {
            detail: format!(
                "unsupported CDML bond type {:?}: order={order:?}, style={style:?}, haworth_position={position:?}",
                bond.source_type(),
            ),
        },
    }
}

fn effective_hydrogens(local: Option<VisibilityV1>, projection: &DocumentProjectionV1) -> bool {
    local.or_else(|| {
        projection
            .drawing_standard()
            .and_then(|standard| standard.show_hydrogens())
    }) == Some(VisibilityV1::Enabled)
}

fn atom_context(
    atom: &AtomProjectionV1,
    owner_molecule_object_id: &DocumentObjectIdV1,
) -> Result<RenderPlanEntryContextV1, DepictionIssueV1> {
    record_context(
        atom.document_object_id(),
        owner_molecule_object_id,
        atom.source_id(),
        atom.source_order(),
        RecordKind::Atom,
        atom.projection_key().as_str(),
    )
}

fn bond_context(
    bond: &BondProjectionV1,
    owner_molecule_object_id: &DocumentObjectIdV1,
) -> Result<RenderPlanEntryContextV1, DepictionIssueV1> {
    record_context(
        bond.document_object_id(),
        owner_molecule_object_id,
        bond.source_id(),
        bond.source_order(),
        RecordKind::Bond,
        bond.projection_key().as_str(),
    )
}

fn record_context(
    durable: &ferrum_document_projection::DocumentObjectIdV1,
    owner_molecule_object_id: &DocumentObjectIdV1,
    source_id: Option<&str>,
    source_order: u32,
    kind: RecordKind,
    local: &str,
) -> Result<RenderPlanEntryContextV1, DepictionIssueV1> {
    let source_id = source_id.ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            local,
            "durable rendering target has no source identifier",
        )
    })?;
    let identifier = Identifier::new(source_id).map_err(|error| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            local,
            error.to_string(),
        )
    })?;
    let record_id = RecordId::new(kind, identifier).map_err(|error| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            local,
            error.to_string(),
        )
    })?;
    Ok(RenderPlanEntryContextV1::new(
        RenderTarget::document_object(durable.clone()),
        record_id,
        source_order,
        Some(owner_molecule_object_id.clone()),
    ))
}

fn endpoint_record(
    endpoint: &ferrum_document_projection::BondEndpointV1,
    endpoints: &std::collections::HashMap<DocumentObjectIdV1, RecordId>,
    local: &str,
) -> Result<RecordId, DepictionIssueV1> {
    let kind = match endpoint.kind() {
        BondEndpointKindV1::Atom => RecordKind::Atom,
        BondEndpointKindV1::Group => RecordKind::Group,
        _ => {
            return Err(issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                local,
                "bond endpoint is not a renderable atom or compact group",
            ));
        }
    };
    let object_id = endpoint.object_id().ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::UnsupportedFeature,
            local,
            "bond endpoint has no durable document identity",
        )
    })?;
    endpoints
        .get(object_id)
        .filter(|record_id| record_id.kind() == kind)
        .cloned()
        .ok_or_else(|| {
            issue(
                DepictionIssueCodeV1::UnsupportedFeature,
                local,
                "bond endpoint has no renderable durable atom or compact group",
            )
        })
}

fn bond_record_id(bond: &BondProjectionV1) -> Result<RecordId, DepictionIssueV1> {
    record_id(
        bond.source_id(),
        RecordKind::Bond,
        bond.document_object_id().as_str(),
    )
}

fn record_id(
    source_id: Option<&str>,
    kind: RecordKind,
    target: &str,
) -> Result<RecordId, DepictionIssueV1> {
    let source_id = source_id.ok_or_else(|| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            target,
            "durable rendering target has no source identifier",
        )
    })?;
    let identifier = Identifier::new(source_id).map_err(|error| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            target,
            error.to_string(),
        )
    })?;
    RecordId::new(kind, identifier).map_err(|error| {
        issue(
            DepictionIssueCodeV1::NonDurableTarget,
            target,
            error.to_string(),
        )
    })
}

pub(super) fn resolved_font(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
    local: Option<&ferrum_document_projection::FontFactsV1>,
    label_mask: Option<&TransparentOrRgb24V1>,
) -> Result<AtomLabelFontProfile, DepictionIssueV1> {
    let family = local.and_then(|font| font.family()).or_else(|| {
        projection
            .drawing_standard()
            .and_then(|standard| standard.font_family())
    });
    if family.is_some_and(|value| PresentationFontFaceV1::from_cdml_family(value).is_none()) {
        return Err(issue(
            DepictionIssueCodeV1::UnsupportedAuthoredFontFamily,
            "document",
            format!("V1 has only the verified {MOLECULE_LABEL_RESOURCE_ID} resource"),
        ));
    }
    let size = local
        .and_then(|font| font.size())
        .or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.font_size())
        })
        .map(|value| value.value())
        .unwrap_or(12.0);
    let paint = local
        .and_then(|font| font.color())
        .or_else(|| {
            projection
                .drawing_standard()
                .and_then(|standard| standard.line_color())
        })
        .map(rgb_paint)
        .unwrap_or_else(RenderPaintV3::document_foreground);
    let mut font = AtomLabelFontProfile::new(FontFace::molecule_label(), positive(size)?, paint);
    if let Some(TransparentOrRgb24V1::Rgb24(mask)) = label_mask {
        font = font.with_label_mask(rgb_paint(mask));
    }
    let _ = profile;
    Ok(font)
}

pub(super) fn resolved_line_width(
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    positive(
        projection
            .drawing_standard()
            .and_then(|standard| standard.line_width())
            .map_or(1.0, |value| value.value()),
    )
}
pub(super) fn resolved_line_paint(
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<RenderPaintV3, DepictionIssueV1> {
    Ok(projection
        .drawing_standard()
        .and_then(|standard| standard.line_color())
        .map(rgb_paint)
        .unwrap_or_else(RenderPaintV3::document_foreground))
}
fn resolved_bond_width(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    bond.line_width()
        .map(|value| positive(value.value()))
        .unwrap_or_else(|| resolved_line_width(projection, profile))
}
pub(super) fn resolved_default_bond_lane_spacing(
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    if let Some(value) = projection
        .drawing_standard()
        .and_then(|standard| standard.bond_width())
    {
        return positive(value.value());
    }
    let line_width = resolved_line_width(projection, profile)?;
    positive(line_width.get() * BUILTIN_BOND_LANE_STROKE_FACTOR)
}
fn resolved_bond_lane_spacing(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    bond.bond_width()
        .map(|value| positive(value.value()))
        .unwrap_or_else(|| resolved_default_bond_lane_spacing(projection, profile))
}
fn resolved_bond_wedge_width(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    _profile: &DepictionProfileV1,
) -> Result<PositiveFinite, DepictionIssueV1> {
    positive(
        bond.wedge_width()
            .map(|value| value.value())
            .or_else(|| {
                projection
                    .drawing_standard()
                    .and_then(|standard| standard.wedge_width())
                    .map(|value| value.value())
            })
            .unwrap_or(BUILTIN_BOND_WEDGE_WIDTH),
    )
}
fn resolved_bond_paint(
    bond: &BondProjectionV1,
    projection: &DocumentProjectionV1,
    profile: &DepictionProfileV1,
) -> Result<RenderPaintV3, DepictionIssueV1> {
    Ok(bond
        .color()
        .map(rgb_paint)
        .unwrap_or(resolved_line_paint(projection, profile)?))
}
pub(super) fn positive(value: f64) -> Result<PositiveFinite, DepictionIssueV1> {
    PositiveFinite::new(value).map_err(|error| {
        issue(
            DepictionIssueCodeV1::InvalidPresentationFact,
            "document",
            error.to_string(),
        )
    })
}
fn rgb_paint(value: &DocumentRgb24V1) -> RenderPaintV3 {
    paint(&value.as_str()[1..])
}
fn paint(value: &str) -> RenderPaintV3 {
    RenderPaintV3::authored_rgb24(Rgb24::new(value).expect("validated profile RGB"))
}
pub(super) fn issue(
    code: DepictionIssueCodeV1,
    target: impl Into<String>,
    detail: impl Into<String>,
) -> DepictionIssueV1 {
    DepictionIssueV1::new(code, target, detail)
}
