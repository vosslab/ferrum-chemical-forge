//! Toolkit-neutral, fully laid-out glyph measurement for render-plan generation.

use ferrum_geometry::Vector2;

use crate::atom_bond::AtomLabelRunRole;
use crate::glyph_outline_support::GlyphOutlineSupport;
use crate::{
    AtomLabelFacts, AtomLabelFontProfile, PositiveFinite, RenderError, RenderPoint, TextRun,
    TextScript, VerifiedMoleculeLabelGlyphMetrics,
};

/// Finite visible-ink extents relative to a label-local origin.
///
/// Atom-label layout receipts remain crate-private. This immutable bounds DTO
/// stays public because document-render consumes compact-group hit bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphBounds {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl GlyphBounds {
    /// Construct finite, nonempty visible-ink bounds.
    pub fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Result<Self, RenderError> {
        if !min_x.is_finite() || !min_y.is_finite() || !max_x.is_finite() || !max_y.is_finite() {
            return Err(RenderError::InvalidRequest(
                "glyph bounds must be finite".to_owned(),
            ));
        }
        if min_x >= max_x || min_y >= max_y {
            return Err(RenderError::InvalidRequest(
                "glyph bounds must be nonempty".to_owned(),
            ));
        }
        Ok(Self {
            min_x,
            min_y,
            max_x,
            max_y,
        })
    }

    /// Return the left extent relative to the label origin.
    #[must_use]
    pub const fn min_x(self) -> f64 {
        self.min_x
    }
    /// Return the lower extent relative to the label origin.
    #[must_use]
    pub const fn min_y(self) -> f64 {
        self.min_y
    }
    /// Return the right extent relative to the label origin.
    #[must_use]
    pub const fn max_x(self) -> f64 {
        self.max_x
    }
    /// Return the upper extent relative to the label origin.
    #[must_use]
    pub const fn max_y(self) -> f64 {
        self.max_y
    }

    /// Canonicalize a verified core outline as symmetric bounds at the origin.
    ///
    /// The caller must first prove that the underlying glyph placement is the
    /// exact Atkinson Hyperlegible Next-centered placement. This representation removes only the
    /// final floating-point addition residual from an otherwise centered
    /// outline; it never makes an arbitrary translated outline valid.
    pub(crate) fn canonical_centered_at_origin(self) -> Result<Self, RenderError> {
        let half_width = (self.max_x - self.min_x) / 2.0;
        let half_height = (self.max_y - self.min_y) / 2.0;
        Self::new(-half_width, -half_height, half_width, half_height)
    }
}

/// Atom-label geometry that attaches bonds to the structural element run.
///
/// The source atom identity, rather than a later bond lowerer, determines the
/// core run.  This keeps alignment valid for labels with hydrogens and charge
/// annotations whose total advance is intentionally asymmetric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AtomLabelAttachmentGeometry {
    core_element_ink_bounds: GlyphBounds,
    core_element_ink_center: RenderPoint,
}

impl AtomLabelAttachmentGeometry {
    /// Construct verified core-element ink geometry.
    pub(crate) fn new(core_element_ink_bounds: GlyphBounds) -> Result<Self, RenderError> {
        let calculated_center = RenderPoint::new(
            (core_element_ink_bounds.min_x() + core_element_ink_bounds.max_x()) / 2.0,
            (core_element_ink_bounds.min_y() + core_element_ink_bounds.max_y()) / 2.0,
        )?;
        if calculated_center.x() != 0.0 || calculated_center.y() != 0.0 {
            return Err(RenderError::InvalidRequest(
                "atom-label core element ink must be centered at the local atom origin".to_owned(),
            ));
        }
        Ok(Self {
            core_element_ink_bounds,
            core_element_ink_center: RenderPoint::new(0.0, 0.0)?,
        })
    }
    /// Return the exact structural-element ink rectangle.
    #[must_use]
    pub(crate) const fn core_element_ink_bounds(self) -> GlyphBounds {
        self.core_element_ink_bounds
    }

    /// Return the exact structural-element ink center.
    #[must_use]
    pub(crate) const fn core_element_ink_center(self) -> RenderPoint {
        self.core_element_ink_center
    }
}

/// Final bond-ink corridor that must remain clear of non-core label ink.
///
/// Coordinates are atom-local. The transverse interval contains the complete
/// terminal footprint, while `decoration_clearance` expands label rectangles
/// exactly as final admission does. The axial ray begins after the structural
/// core outline and its optical gap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AtomLabelAttachmentCorridor {
    direction: Vector2,
    transverse_minimum: f64,
    transverse_maximum: f64,
    optical_gap: PositiveFinite,
    decoration_clearance: PositiveFinite,
}

impl AtomLabelAttachmentCorridor {
    pub(crate) fn new(
        direction: Vector2,
        transverse_minimum: f64,
        transverse_maximum: f64,
        optical_gap: PositiveFinite,
        decoration_clearance: PositiveFinite,
    ) -> Result<Self, RenderError> {
        let length = direction.length();
        if !length.is_finite()
            || (length - 1.0).abs() > 1.0e-12
            || !transverse_minimum.is_finite()
            || !transverse_maximum.is_finite()
            || transverse_minimum > transverse_maximum
        {
            return Err(RenderError::InvalidRequest(
                "atom-label attachment corridor must be finite, normalized, and ordered".to_owned(),
            ));
        }
        Ok(Self {
            direction,
            transverse_minimum,
            transverse_maximum,
            optical_gap,
            decoration_clearance,
        })
    }

    fn intersects(self, bounds: GlyphBounds, core_outline_support: &GlyphOutlineSupport) -> bool {
        let (axial_minimum, axial_maximum) =
            project_bounds_with_clearance(bounds, self.direction, self.decoration_clearance);
        let perpendicular = self.direction.perpendicular_left();
        let (transverse_minimum, transverse_maximum) =
            project_bounds_with_clearance(bounds, perpendicular, self.decoration_clearance);
        let axial_start =
            core_outline_support.directional_extent(self.direction) + self.optical_gap.get();
        axial_maximum >= axial_start
            && axial_minimum.is_finite()
            && transverse_maximum >= self.transverse_minimum
            && transverse_minimum <= self.transverse_maximum
    }

    fn transverse_clearance_distance(
        self,
        bounds: GlyphBounds,
        movement: Vector2,
        separation: PositiveFinite,
    ) -> Option<f64> {
        let perpendicular = self.direction.perpendicular_left();
        let coefficient = movement.dot(perpendicular);
        if coefficient.abs() <= 1.0e-12 {
            return None;
        }
        let (minimum, maximum) =
            project_bounds_with_clearance(bounds, perpendicular, self.decoration_clearance);
        let distance = if coefficient > 0.0 {
            (self.transverse_maximum + separation.get() - minimum) / coefficient
        } else {
            (maximum - self.transverse_minimum + separation.get()) / -coefficient
        };
        distance.is_finite().then_some(distance.max(0.0))
    }
}

fn project_bounds_with_clearance(
    bounds: GlyphBounds,
    axis: Vector2,
    clearance: PositiveFinite,
) -> (f64, f64) {
    let (minimum, maximum) = project_bounds(bounds, axis);
    let projected_clearance = clearance.get() * (axis.x().abs() + axis.y().abs());
    (minimum - projected_clearance, maximum + projected_clearance)
}

fn project_bounds(bounds: GlyphBounds, axis: Vector2) -> (f64, f64) {
    let corners = [
        (bounds.min_x(), bounds.min_y()),
        (bounds.max_x(), bounds.min_y()),
        (bounds.max_x(), bounds.max_y()),
        (bounds.min_x(), bounds.max_y()),
    ];
    corners.iter().fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(minimum, maximum), (x, y)| {
            let projection = x * axis.x() + y * axis.y();
            (minimum.min(projection), maximum.max(projection))
        },
    )
}

/// Fully placed semantic runs and the bounds of those exact runs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LaidOutAtomLabel {
    runs: Vec<TextRun>,
    bounds: GlyphBounds,
    attachment: AtomLabelAttachmentGeometry,
    core_outline_support: GlyphOutlineSupport,
    core_element_run_index: u32,
    run_roles: Vec<AtomLabelRunRole>,
    non_core_run_ink_bounds: Vec<GlyphBounds>,
}

impl LaidOutAtomLabel {
    /// Construct a nonempty, fully positioned label and its clipping bounds.
    pub(crate) fn new(
        runs: Vec<TextRun>,
        bounds: GlyphBounds,
        attachment: AtomLabelAttachmentGeometry,
        core_outline_support: GlyphOutlineSupport,
        core_element_run_index: u32,
        run_roles: Vec<AtomLabelRunRole>,
        non_core_run_ink_bounds: Vec<GlyphBounds>,
    ) -> Result<Self, RenderError> {
        if runs.is_empty() {
            return Err(RenderError::InvalidRequest(
                "laid-out atom label requires at least one run".to_owned(),
            ));
        }
        if run_roles.len() != runs.len() {
            return Err(RenderError::InvalidRequest(
                "atom-label run roles must match the exact laid-out runs".to_owned(),
            ));
        }
        let center = attachment.core_element_ink_center();
        if center.x() != 0.0 || center.y() != 0.0 {
            return Err(RenderError::InvalidRequest(
                "atom-label core element ink center must equal the local atom origin".to_owned(),
            ));
        }
        let core = attachment.core_element_ink_bounds();
        if core.min_x() < bounds.min_x()
            || core.min_y() < bounds.min_y()
            || core.max_x() > bounds.max_x()
            || core.max_y() > bounds.max_y()
        {
            return Err(RenderError::InvalidRequest(
                "atom-label core element ink must lie within the full visible ink bounds"
                    .to_owned(),
            ));
        }
        let core_index = usize::try_from(core_element_run_index).map_err(|_| {
            RenderError::InvalidRequest("atom-label core run index is not addressable".to_owned())
        })?;
        let core_run = runs.get(core_index).ok_or_else(|| {
            RenderError::InvalidRequest(
                "atom-label core run index is outside laid-out label runs".to_owned(),
            )
        })?;
        if core_run.script() != TextScript::Baseline {
            return Err(RenderError::InvalidRequest(
                "atom-label core run must use baseline script".to_owned(),
            ));
        }
        if run_roles[core_index] != AtomLabelRunRole::CoreElement
            || run_roles
                .iter()
                .filter(|role| **role == AtomLabelRunRole::CoreElement)
                .count()
                != 1
        {
            return Err(RenderError::InvalidRequest(
                "atom label requires exactly one role-identified structural run".to_owned(),
            ));
        }
        Ok(Self {
            runs,
            bounds,
            attachment,
            core_outline_support,
            core_element_run_index,
            run_roles,
            non_core_run_ink_bounds,
        })
    }
    /// Return drawing runs with fully explicit local geometry.
    #[must_use]
    pub(crate) fn runs(&self) -> &[TextRun] {
        &self.runs
    }
    /// Return the clipping rectangle for those exact drawing runs.
    #[must_use]
    pub(crate) const fn bounds(&self) -> GlyphBounds {
        self.bounds
    }

    /// Return the structural attachment geometry for the exact positioned runs.
    #[must_use]
    pub(crate) const fn attachment(&self) -> AtomLabelAttachmentGeometry {
        self.attachment
    }

    /// Return exact verified-Atkinson Hyperlegible Next outline support for the structural run.
    #[must_use]
    pub(crate) const fn core_outline_support(&self) -> &GlyphOutlineSupport {
        &self.core_outline_support
    }

    /// Return the source-issued structural element run in `runs`.
    ///
    /// This is not inferred by the atom lowerer or a presentation consumer.
    #[must_use]
    pub(crate) const fn core_element_run_index(&self) -> u32 {
        self.core_element_run_index
    }

    /// Return exact non-core Atkinson Hyperlegible Next run ink rectangles for directional clipping.
    #[must_use]
    pub(crate) fn non_core_run_ink_bounds(&self) -> &[GlyphBounds] {
        &self.non_core_run_ink_bounds
    }

    /// Choose the first conventional decoration layout that clears every bond corridor.
    ///
    /// The structural element stays invariant. Isotope placement remains in
    /// the upper-left semantic register, explicit hydrogen may occupy either
    /// baseline side, and formal charge prefers the upper-right before the
    /// open space above, below, or upper-left of the core. These are semantic
    /// placements, not molecule-specific pixel offsets.
    pub(crate) fn place_decorations_around_attachment_corridors(
        self,
        corridors: &[AtomLabelAttachmentCorridor],
        spacing: PositiveFinite,
        size: PositiveFinite,
        metrics: &VerifiedMoleculeLabelGlyphMetrics,
    ) -> Result<Self, RenderError> {
        let core_index = self.core_element_run_index as usize;
        let core_bounds = self.attachment.core_element_ink_bounds();
        let mut run_bounds = Vec::with_capacity(self.runs.len());
        let mut non_core_bounds = self.non_core_run_ink_bounds.iter().copied();
        for index in 0..self.runs.len() {
            run_bounds.push(if index == core_index {
                core_bounds
            } else {
                non_core_bounds
                    .next()
                    .expect("laid-out non-core bounds match run identity")
            });
        }
        let hydrogen_index = self
            .run_roles
            .iter()
            .position(|role| *role == AtomLabelRunRole::ExplicitHydrogen);
        let hydrogen_end = hydrogen_index.map(|index| {
            if self.run_roles.get(index + 1) == Some(&AtomLabelRunRole::HydrogenCount) {
                index + 1
            } else {
                index
            }
        });
        let hydrogen_candidates = hydrogen_translations(
            hydrogen_index.zip(hydrogen_end),
            &run_bounds,
            core_bounds,
            spacing,
            corridors,
        )?;
        let isotope_index = self
            .run_roles
            .iter()
            .position(|role| *role == AtomLabelRunRole::Isotope);
        let isotope_candidates =
            isotope_translations(isotope_index, &run_bounds, core_bounds, spacing, corridors)?;
        let charge_index = self
            .run_roles
            .iter()
            .position(|role| *role == AtomLabelRunRole::FormalCharge);
        let charge_candidates =
            charge_translations(charge_index, &run_bounds, core_bounds, spacing, corridors)?;
        for isotope_translation in isotope_candidates {
            for hydrogen_translation in &hydrogen_candidates {
                for charge_translation in &charge_candidates {
                    let mut translations = vec![RunTranslation::default(); self.runs.len()];
                    if let Some(index) = isotope_index {
                        translations[index] = isotope_translation;
                    }
                    if let Some((start, end)) = hydrogen_index.zip(hydrogen_end) {
                        translations[start..=end].fill(*hydrogen_translation);
                    }
                    if let Some(index) = charge_index {
                        translations[index] = *charge_translation;
                    }
                    if decoration_candidate_is_clear(
                        &run_bounds,
                        &self.run_roles,
                        &translations,
                        corridors,
                        &self.core_outline_support,
                    )? {
                        return self.rebuild_with_translations(&translations, size, metrics);
                    }
                }
            }
        }
        Err(RenderError::InvalidRequest(
            "atom-label decorations have no collision-free admitted bond corridor".to_owned(),
        ))
    }

    fn rebuild_with_translations(
        self,
        translations: &[RunTranslation],
        size: PositiveFinite,
        metrics: &VerifiedMoleculeLabelGlyphMetrics,
    ) -> Result<Self, RenderError> {
        let core_index = self.core_element_run_index as usize;
        let core_bounds = self.attachment.core_element_ink_bounds();
        let runs = self
            .runs
            .iter()
            .zip(translations)
            .map(|(run, translation)| translated_run(run, *translation))
            .collect::<Result<Vec<_>, _>>()?;
        let text_origin = RenderPoint::new(0.0, 0.0)?;
        let mut bounds = None;
        let mut updated_non_core = Vec::with_capacity(self.non_core_run_ink_bounds.len());
        for (index, run) in runs.iter().enumerate() {
            let run_bounds = metrics.run_ink_bounds_at(text_origin, size, run)?;
            bounds = Some(match bounds {
                Some(existing) => union_bounds(existing, run_bounds)?,
                None => run_bounds,
            });
            if index != core_index {
                updated_non_core.push(run_bounds);
            }
        }
        let bounds = union_bounds(
            bounds.expect("laid-out label has at least its structural run"),
            core_bounds,
        )?;
        Self::new(
            runs,
            bounds,
            self.attachment,
            self.core_outline_support,
            self.core_element_run_index,
            self.run_roles,
            updated_non_core,
        )
    }
}

#[derive(Clone, Copy, Default, PartialEq)]
struct RunTranslation {
    x: f64,
    y: f64,
}

fn isotope_translations(
    index: Option<usize>,
    run_bounds: &[GlyphBounds],
    core_bounds: GlyphBounds,
    spacing: PositiveFinite,
    corridors: &[AtomLabelAttachmentCorridor],
) -> Result<Vec<RunTranslation>, RenderError> {
    let Some(index) = index else {
        return Ok(vec![RunTranslation::default()]);
    };
    let isotope = run_bounds[index];
    let left = core_bounds.min_x() - spacing.get() - isotope.max_x();
    let above = core_bounds.min_y() - spacing.get() - isotope.max_y();
    semantic_translations(
        isotope,
        &[
            (RunTranslation::default(), movement(-1.0, -1.0)?),
            (RunTranslation { x: 0.0, y: above }, movement(0.0, -1.0)?),
            (RunTranslation { x: left, y: 0.0 }, movement(-1.0, 0.0)?),
            (RunTranslation { x: left, y: above }, movement(-1.0, -1.0)?),
        ],
        corridors,
        spacing,
    )
}

fn hydrogen_translations(
    group: Option<(usize, usize)>,
    run_bounds: &[GlyphBounds],
    core_bounds: GlyphBounds,
    spacing: PositiveFinite,
    corridors: &[AtomLabelAttachmentCorridor],
) -> Result<Vec<RunTranslation>, RenderError> {
    let Some((start, end)) = group else {
        return Ok(vec![RunTranslation::default()]);
    };
    let group_bounds = run_bounds[start..=end]
        .iter()
        .copied()
        .try_fold(None, |combined, bounds| {
            Ok::<_, RenderError>(Some(match combined {
                Some(combined) => union_bounds(combined, bounds)?,
                None => bounds,
            }))
        })?
        .expect("explicit hydrogen group has at least one run");
    let group_right = run_bounds[start..=end]
        .iter()
        .map(|bounds| bounds.max_x())
        .fold(f64::NEG_INFINITY, f64::max);
    semantic_translations(
        group_bounds,
        &[
            (RunTranslation::default(), movement(1.0, 0.0)?),
            (
                RunTranslation {
                    x: core_bounds.min_x() - spacing.get() - group_right,
                    y: 0.0,
                },
                movement(-1.0, 0.0)?,
            ),
        ],
        corridors,
        spacing,
    )
}

fn charge_translations(
    index: Option<usize>,
    run_bounds: &[GlyphBounds],
    core_bounds: GlyphBounds,
    spacing: PositiveFinite,
    corridors: &[AtomLabelAttachmentCorridor],
) -> Result<Vec<RunTranslation>, RenderError> {
    let Some(index) = index else {
        return Ok(vec![RunTranslation::default()]);
    };
    let charge = run_bounds[index];
    let charge_center_x = (charge.min_x() + charge.max_x()) / 2.0;
    let preferred_gap = spacing.get() * 0.25;
    semantic_translations(
        charge,
        &[
            (
                RunTranslation {
                    x: core_bounds.max_x() + preferred_gap - charge.min_x(),
                    y: 0.0,
                },
                movement(1.0, -1.0)?,
            ),
            (
                RunTranslation {
                    x: -charge_center_x,
                    y: core_bounds.min_y() - spacing.get() - charge.max_y(),
                },
                movement(0.0, -1.0)?,
            ),
            (
                RunTranslation {
                    x: -charge_center_x,
                    y: core_bounds.max_y() + spacing.get() - charge.min_y(),
                },
                movement(0.0, 1.0)?,
            ),
            (
                RunTranslation {
                    x: core_bounds.min_x() - preferred_gap - charge.max_x(),
                    y: 0.0,
                },
                movement(-1.0, -1.0)?,
            ),
        ],
        corridors,
        spacing,
    )
}

fn movement(x: f64, y: f64) -> Result<Vector2, RenderError> {
    Vector2::new(x, y)
        .and_then(Vector2::normalized)
        .map_err(|error| RenderError::InvalidRequest(error.to_string()))
}

fn semantic_translations(
    bounds: GlyphBounds,
    bases: &[(RunTranslation, Vector2)],
    corridors: &[AtomLabelAttachmentCorridor],
    spacing: PositiveFinite,
) -> Result<Vec<RunTranslation>, RenderError> {
    let mut candidates = Vec::with_capacity(bases.len() * 2);
    for (base, movement) in bases {
        candidates.push(*base);
        let translated = translated_bounds(bounds, *base)?;
        let distance = corridors
            .iter()
            .filter_map(|corridor| {
                corridor.transverse_clearance_distance(translated, *movement, spacing)
            })
            .fold(0.0_f64, f64::max);
        if distance > 0.0 {
            candidates.push(RunTranslation {
                x: base.x + movement.x() * distance,
                y: base.y + movement.y() * distance,
            });
        }
    }
    candidates.sort_by(|first, second| first.x.hypot(first.y).total_cmp(&second.x.hypot(second.y)));
    candidates.dedup();
    Ok(candidates)
}

fn decoration_candidate_is_clear(
    run_bounds: &[GlyphBounds],
    roles: &[AtomLabelRunRole],
    translations: &[RunTranslation],
    corridors: &[AtomLabelAttachmentCorridor],
    core_outline_support: &GlyphOutlineSupport,
) -> Result<bool, RenderError> {
    let translated = run_bounds
        .iter()
        .zip(translations)
        .map(|(bounds, translation)| translated_bounds(*bounds, *translation))
        .collect::<Result<Vec<_>, _>>()?;
    for (index, bounds) in translated.iter().enumerate() {
        if roles[index] == AtomLabelRunRole::CoreElement {
            continue;
        }
        if corridors
            .iter()
            .any(|corridor| corridor.intersects(*bounds, core_outline_support))
        {
            return Ok(false);
        }
    }
    for first in 0..translated.len() {
        for second in first + 1..translated.len() {
            let first_group = semantic_group(roles[first]);
            let second_group = semantic_group(roles[second]);
            let isotope_hydrogen_pair = matches!((first_group, second_group), (1, 2) | (2, 1));
            if first_group != second_group
                && !isotope_hydrogen_pair
                && bounds_overlap(translated[first], translated[second])
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn semantic_group(role: AtomLabelRunRole) -> u8 {
    match role {
        AtomLabelRunRole::CoreElement => 0,
        AtomLabelRunRole::Isotope => 1,
        AtomLabelRunRole::ExplicitHydrogen | AtomLabelRunRole::HydrogenCount => 2,
        AtomLabelRunRole::FormalCharge => 3,
    }
}

fn bounds_overlap(first: GlyphBounds, second: GlyphBounds) -> bool {
    first.min_x() < second.max_x()
        && first.max_x() > second.min_x()
        && first.min_y() < second.max_y()
        && first.max_y() > second.min_y()
}

fn translated_run(run: &TextRun, translation: RunTranslation) -> Result<TextRun, RenderError> {
    TextRun::new(
        run.text(),
        run.script(),
        RenderPoint::new(
            run.origin().x() + translation.x,
            run.origin().y() + translation.y,
        )?,
        run.glyphs().to_vec(),
        run.scale(),
    )
}

fn translated_bounds(
    bounds: GlyphBounds,
    translation: RunTranslation,
) -> Result<GlyphBounds, RenderError> {
    GlyphBounds::new(
        bounds.min_x() + translation.x,
        bounds.min_y() + translation.y,
        bounds.max_x() + translation.x,
        bounds.max_y() + translation.y,
    )
}

fn union_bounds(first: GlyphBounds, second: GlyphBounds) -> Result<GlyphBounds, RenderError> {
    GlyphBounds::new(
        first.min_x().min(second.min_x()),
        first.min_y().min(second.min_y()),
        first.max_x().max(second.max_x()),
        first.max_y().max(second.max_y()),
    )
}

/// Shapes and measures one complete atom label without leaking toolkit types.
///
/// Implementors own any font-system state. They must return the same explicit
/// run geometry that a consumer paints and finite visible-ink bounds for those
/// exact runs; otherwise they return an actionable error.
pub(crate) trait GlyphMetrics {
    /// Lay out and measure the exact label under the exact requested font profile.
    fn layout_atom_label(
        &self,
        label: &AtomLabelFacts,
        font: &AtomLabelFontProfile,
    ) -> Result<LaidOutAtomLabel, RenderError>;

    /// Lay out one canonical positive decimal atom-number annotation.
    fn layout_atom_number(
        &self,
        number: u64,
        font: &AtomLabelFontProfile,
    ) -> Result<TextRun, RenderError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AtomLabelRenderV1, FerrumFontEnvironment, FontFace, InkBoundsV1, PositiveFinite,
        RenderPaintV3, RenderPoint, Rgb24, TextOp, TextScript, VerifiedMoleculeLabelGlyphMetrics,
    };
    use ferrum_geometry::Vector2;

    fn size(value: f64) -> PositiveFinite {
        PositiveFinite::new(value).expect("test font size is valid")
    }

    fn font() -> AtomLabelFontProfile {
        AtomLabelFontProfile::new(
            FontFace::molecule_label(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint is valid")),
        )
    }

    #[test]
    fn verified_molecule_label_font_issues_exact_centered_structural_core_runs() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        for (element, charge, hydrogens) in [("C", 0, 0), ("O", 0, 0), ("Cl", 0, 0), ("N", 1, 3)] {
            let facts = AtomLabelFacts::new(element, None, charge, hydrogens)
                .expect("test atom facts are admitted");
            let layout = metrics
                .layout_atom_label(&facts, &font())
                .expect("verified Atkinson Hyperlegible Next lays out test label");
            assert_eq!(layout.core_element_run_index(), 0);
            assert_eq!(layout.runs()[0].text(), element);
            assert_eq!(layout.runs()[0].script(), TextScript::Baseline);
            let core = layout.attachment().core_element_ink_bounds();
            assert_eq!((core.min_x() + core.max_x()) / 2.0, 0.0);
            assert_eq!((core.min_y() + core.max_y()) / 2.0, 0.0);
            let text = TextOp::new(
                RenderPoint::new(0.0, 0.0).expect("test origin is finite"),
                layout.runs().to_vec(),
                FontFace::molecule_label(),
                size(12.0),
                RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
                30,
            )
            .expect("verified Atkinson Hyperlegible Next text is valid");
            let core_index = layout.core_element_run_index();
            let full = InkBoundsV1::from_glyph_bounds(
                metrics
                    .atom_label_ink_bounds(&text, core_index as usize)
                    .expect("canonical full label ink is available"),
            );
            let selected_core = InkBoundsV1::from_glyph_bounds(
                metrics
                    .centered_core_run_ink_bounds(&text, core_index as usize)
                    .expect("canonical core ink is available"),
            );
            AtomLabelRenderV1::new(None, text, core_index, size(1.5), full, selected_core)
                .expect("durable atom label accepts the issued core run and bounds");
        }
    }

    #[test]
    fn verified_molecule_label_font_outline_support_matches_cardinal_core_bounds() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        for element in ["C", "O", "Cl", "I"] {
            let facts =
                AtomLabelFacts::new(element, None, 0, 0).expect("test atom facts are admitted");
            let layout = metrics
                .layout_atom_label(&facts, &font())
                .expect("verified Atkinson Hyperlegible Next lays out test label");
            let core = layout.attachment().core_element_ink_bounds();
            let support = layout.core_outline_support();
            let cases = [
                (Vector2::new(1.0, 0.0).expect("direction"), core.max_x()),
                (Vector2::new(-1.0, 0.0).expect("direction"), -core.min_x()),
                (Vector2::new(0.0, 1.0).expect("direction"), core.max_y()),
                (Vector2::new(0.0, -1.0).expect("direction"), -core.min_y()),
            ];
            for (direction, expected) in cases {
                let actual = support.directional_extent(direction);
                assert!(
                    (actual - expected).abs() < 1.0e-12,
                    "{element}: {actual} != {expected}"
                );
            }
        }
    }

    #[test]
    fn decorated_label_selects_a_clear_parallel_terminal_layout() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        let facts = AtomLabelFacts::new("C", Some(13), 1, 3).expect("decorated carbon facts");
        let canonical = metrics
            .layout_atom_label(&facts, &font())
            .expect("canonical decorated layout");
        let corridor = AtomLabelAttachmentCorridor::new(
            Vector2::new(1.0, 0.0).expect("rightward direction"),
            -3.5,
            3.5,
            size(0.75),
            size(0.375),
        )
        .expect("parallel terminal corridor");
        let placed = canonical
            .clone()
            .place_decorations_around_attachment_corridors(
                &[corridor],
                size(0.6),
                size(12.0),
                &metrics,
            )
            .expect("decorated label has one admitted layout");
        assert_eq!(placed.runs()[0].origin(), canonical.runs()[0].origin());
        assert_eq!(placed.runs()[1].origin(), canonical.runs()[1].origin());
        assert!(placed.runs()[2].origin().x() < placed.runs()[1].origin().x());
        assert!(placed.runs()[3].origin().x() < placed.runs()[1].origin().x());
        assert!(placed.runs()[4].origin().y() < canonical.runs()[4].origin().y());
        assert_eq!(
            placed.core_outline_support(),
            canonical.core_outline_support()
        );
        assert!(
            placed
                .non_core_run_ink_bounds()
                .iter()
                .all(|bounds| !corridor.intersects(*bounds, placed.core_outline_support()))
        );
        let text = TextOp::new(
            RenderPoint::new(0.0, 0.0).expect("test origin"),
            placed.runs().to_vec(),
            FontFace::molecule_label(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
            30,
        )
        .expect("relocated label text");
        assert_eq!(
            placed.bounds(),
            metrics
                .atom_label_ink_bounds(&text, placed.core_element_run_index() as usize)
                .expect("relocated runs retain exact Atkinson Hyperlegible Next bounds")
        );
    }

    #[test]
    fn decorated_label_clears_parallel_terminals_in_eight_directions() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        let facts = AtomLabelFacts::new("P", Some(31), 1, 4).expect("decorated phosphorus facts");
        let canonical = metrics
            .layout_atom_label(&facts, &font())
            .expect("canonical decorated layout");
        let diagonal = std::f64::consts::FRAC_1_SQRT_2;
        for (x, y) in [
            (1.0, 0.0),
            (diagonal, diagonal),
            (0.0, 1.0),
            (-diagonal, diagonal),
            (-1.0, 0.0),
            (-diagonal, -diagonal),
            (0.0, -1.0),
            (diagonal, -diagonal),
        ] {
            let corridor = AtomLabelAttachmentCorridor::new(
                Vector2::new(x, y).expect("test direction is normalized"),
                -3.5,
                3.5,
                size(0.75),
                size(0.375),
            )
            .expect("parallel terminal corridor");
            let placed = canonical
                .clone()
                .place_decorations_around_attachment_corridors(
                    &[corridor],
                    size(0.6),
                    size(12.0),
                    &metrics,
                )
                .unwrap_or_else(|error| {
                    panic!(
                        "decorated label has no admitted layout for direction ({x}, {y}): {error}"
                    )
                });
            assert_eq!(
                placed.attachment(),
                canonical.attachment(),
                "structural core changed for direction ({x}, {y})"
            );
            assert!(
                placed
                    .non_core_run_ink_bounds()
                    .iter()
                    .all(|bounds| !corridor.intersects(*bounds, placed.core_outline_support())),
                "decoration entered the terminal corridor for direction ({x}, {y})"
            );
        }
    }

    #[test]
    fn attachment_rejects_tiny_forged_nonzero_core_center() {
        let forged = GlyphBounds::new(-1.0, -1.0, 1.0 + 1.0e-15, 1.0)
            .expect("forged bounds are geometrically nonempty");
        assert!(AtomLabelAttachmentGeometry::new(forged).is_err());
    }
    #[test]
    fn durable_label_rejects_reordered_baseline_run_as_the_structural_core() {
        let environment =
            FerrumFontEnvironment::load().expect("bundled Atkinson Hyperlegible Next is verified");
        let metrics = VerifiedMoleculeLabelGlyphMetrics::new(&environment)
            .expect("Atkinson Hyperlegible Next metrics are available");
        let facts = AtomLabelFacts::new("N", None, 1, 3).expect("test atom facts are admitted");
        let layout = metrics
            .layout_atom_label(&facts, &font())
            .expect("verified Atkinson Hyperlegible Next lays out ammonium");
        let mut reordered_runs = layout.runs().to_vec();
        reordered_runs.swap(0, 1);
        assert_eq!(reordered_runs[0].script(), TextScript::Baseline);
        assert_eq!(reordered_runs[1].script(), TextScript::Baseline);
        let text = TextOp::new(
            RenderPoint::new(0.0, 0.0).expect("test origin is finite"),
            reordered_runs,
            FontFace::molecule_label(),
            size(12.0),
            RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("test paint")),
            30,
        )
        .expect("reordered runs retain exact glyph placements");
        let core_index = layout.core_element_run_index();
        let full = InkBoundsV1::from_glyph_bounds(
            metrics
                .text_ink_bounds(&text)
                .expect("reordered text still has exact Atkinson Hyperlegible Next ink"),
        );
        let original_core =
            InkBoundsV1::from_glyph_bounds(layout.attachment().core_element_ink_bounds());
        assert!(
            AtomLabelRenderV1::new(None, text, core_index, size(1.5), full, original_core).is_err()
        );
    }
}
