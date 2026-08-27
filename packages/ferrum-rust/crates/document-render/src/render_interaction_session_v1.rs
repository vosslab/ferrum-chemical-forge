use super::*;
use ferrum_core::BondStyle;

impl RenderInteractionSessionV1 {
    /// Validate that a root selection was issued by this live session and still
    /// refers to its current document fence. This exposes no selection facts.
    pub fn validate_render_interaction_selection_v1(
        &self,
        selection: &RenderInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        self.require_selection(selection)
    }

    #[must_use]
    pub fn new(session: DocumentSession) -> Self {
        Self {
            session,
            origin: NEXT_ORIGIN.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// Issue one opaque receipt for a gesture that may request an authoring
    /// transition. The document owns its lifecycle after preparation.
    #[must_use]
    pub(crate) fn issue_authoring_capability_v1(&self) -> ferrum_document::AuthoringCapabilityV1 {
        self.session.issue_authoring_capability_v1()
    }

    /// Begin a presentation gesture using the same Rust render facts that the
    /// committed Plus will expose. Arrow geometry remains document-owned.
    pub fn begin_presentation_creation_gesture_v1(
        &self,
        fence: DocumentFenceV1,
        kind: PresentationGestureKindV1,
        start: PresentationGesturePoint2V1,
        style: PresentationGestureStyleV1,
        snap: PresentationGestureSnapPolicyV1,
    ) -> Result<PresentationCreationGestureV1, PresentationGestureErrorV1> {
        self.session
            .begin_presentation_creation_gesture_v1(fence, kind, start, style, snap)
    }

    /// Prepare the current document revision for the public SMARTS operation.
    ///
    /// The document session owns target admission, graph construction, and
    /// revision refusal. The renderer facade deliberately adds no policy.
    pub fn prepare_smarts_snapshot_v1(
        &self,
        expected_revision: u64,
    ) -> Result<PreparedDocumentSmartsSnapshotV1, DocumentSmartsSnapshotErrorV1> {
        self.session.prepare_smarts_snapshot_v1(expected_revision)
    }

    pub fn observe_render_interaction_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<RenderInteractionObservationV1, RenderInteractionErrorV1> {
        self.require_fence(fence)?;
        let rendered = self
            .session
            .observe_render_v1(fence.revision())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        let presentation_plan =
            render_presentation_stack_v1(rendered.document().projection().presentation_stack())
                .map_err(|_| RenderInteractionErrorV1::Observation)?;
        self.observe_render_interaction_with_presentation_plan_v1(fence, &presentation_plan)
    }

    /// Build one fenced interaction observation from a renderer-issued presentation plan.
    ///
    /// The plan has no mutation authority. This boundary accepts it only when its
    /// immutable provenance matches the session observation exactly, so canvas and
    /// interaction consumers cannot combine roots from different document states.
    pub fn observe_render_interaction_with_presentation_plan_v1(
        &self,
        fence: DocumentFenceV1,
        presentation_plan: &PresentationRenderPlanV1,
    ) -> Result<RenderInteractionObservationV1, RenderInteractionErrorV1> {
        self.require_fence(fence)?;
        let rendered = self
            .session
            .observe_render_v1(fence.revision())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        self.observe_render_interaction_from_rendered_plan_v1(fence, &rendered, presentation_plan)
    }

    fn observe_render_interaction_from_rendered_plan_v1(
        &self,
        fence: DocumentFenceV1,
        rendered: &DocumentRenderObservationV1,
        presentation_plan: &PresentationRenderPlanV1,
    ) -> Result<RenderInteractionObservationV1, RenderInteractionErrorV1> {
        if rendered.document().snapshot().revision() != fence.revision() {
            return Err(RenderInteractionErrorV1::StaleRevision);
        }
        if rendered.document().snapshot().digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        if presentation_plan.revision() != fence.revision() {
            return Err(RenderInteractionErrorV1::StaleRevision);
        }
        if presentation_plan.digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        let identities = self
            .session
            .observe_complete_document_identity_facts_v1(fence.revision())
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        let (roots, exclusions) = roots_from_render(rendered, presentation_plan, &identities);
        let reaction_members = self
            .session
            .observe_reaction_list_v1()
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?
            .reactions()
            .iter()
            .flat_map(|reaction| reaction.members().iter())
            .map(|member| member.object_id().clone())
            .collect::<HashSet<DocumentObjectIdV1>>();
        let reaction_authoring =
            Self::reaction_authoring_observation(&roots, &exclusions, &reaction_members);
        Ok(RenderInteractionObservationV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            fence,
            roots,
            exclusions,
            reaction_authoring,
        })
    }

    fn reaction_authoring_observation(
        roots: &[RenderInteractionRootV1],
        root_exclusions: &[RenderInteractionExclusionV1],
        reaction_members: &HashSet<DocumentObjectIdV1>,
    ) -> ReactionAuthoringObservationV1 {
        let mut choices = Vec::new();
        let mut exclusions = Vec::new();
        let mut diagnosed = HashSet::new();
        for root in roots {
            match reaction_choice_kind(root.kind()) {
                Some(kind) => {
                    let availability = if reaction_members.contains(root.document_object_id()) {
                        ReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction
                    } else {
                        ReactionAuthoringChoiceAvailabilityV1::Eligible
                    };
                    choices.push(ReactionAuthoringChoiceV1 {
                        document_object_id: root.document_object_id().clone(),
                        paint_order: root.paint_order(),
                        kind,
                        availability,
                        label: reaction_choice_label(kind),
                        bounds: root.bounds(),
                    });
                }
                None => push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    root.document_object_id().as_str(),
                    ReactionAuthoringExclusionReasonV1::DisplayOnly,
                    reaction_exclusion_label(ReactionAuthoringExclusionReasonV1::DisplayOnly),
                ),
            }
        }
        for value in root_exclusions {
            let reason = match value.reason() {
                RenderInteractionExclusionReasonV1::DisplayOnly => {
                    ReactionAuthoringExclusionReasonV1::DisplayOnly
                }
                RenderInteractionExclusionReasonV1::UnrenderableDepiction => {
                    ReactionAuthoringExclusionReasonV1::Unrenderable
                }
                RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier => {
                    ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity
                }
            };
            push_reaction_exclusion(
                &mut exclusions,
                &mut diagnosed,
                value.document_object_id().as_str(),
                reason,
                reaction_exclusion_label(reason),
            );
        }
        choices.sort_by_key(ReactionAuthoringChoiceV1::paint_order);
        ReactionAuthoringObservationV1 {
            choices,
            exclusions,
        }
    }

    /// Observe exact direct atom/bond target envelopes for structural selection.
    pub fn observe_structure_interaction_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<StructureInteractionObservationV1, RenderInteractionErrorV1> {
        self.require_fence(fence)?;
        let rendered = self
            .session
            .observe_render_v1(fence.revision())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        if rendered.document().snapshot().digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        let mut targets = Vec::new();
        for molecule in rendered.document().projection().molecules() {
            let molecule_object_id = molecule.document_object_id();
            let plan = rendered
                .resolved()
                .molecule_plans()
                .iter()
                .find(|entry| entry.molecule().document_object_id() == molecule_object_id)
                .ok_or(RenderInteractionErrorV1::Observation)?;
            for atom in molecule.atoms() {
                let atom_object_id = atom.document_object_id();
                let batch = plan
                    .batches()
                    .iter()
                    .find(|batch| batch.target().document_object_id() == atom_object_id)
                    .ok_or(RenderInteractionErrorV1::Observation)?;
                let point = atom.position();
                targets.push(StructureInteractionTargetV1 {
                    molecule_object_id: molecule_object_id.clone(),
                    object_id: atom_object_id.clone(),
                    source_order: batch.paint_order(),
                    kind: StructureTargetKindV1::Atom,
                    // `contains_point` owns the single shared hit slop.  The
                    // issued atom envelope itself stays at the atom anchor so
                    // nearby bond clicks are not accidentally atom hits.
                    bounds: square_bounds(point.x(), point.y(), 0.0),
                    geometry: StructureInteractionGeometryV1::Atom {
                        x: point.x(),
                        y: point.y(),
                    },
                });
            }
            for group in plan.compact_group_primitives() {
                let object_id = group.target().document_object_id();
                let bounds = group.bounds();
                let anchor = group.anchor();
                targets.push(StructureInteractionTargetV1 {
                    molecule_object_id: molecule_object_id.clone(),
                    object_id: object_id.clone(),
                    source_order: group.batch().paint_order(),
                    kind: StructureTargetKindV1::CompactGroup,
                    bounds: RenderInteractionBoundsV1 {
                        left: anchor.x() + bounds.min_x(),
                        top: anchor.y() + bounds.min_y(),
                        right: anchor.x() + bounds.max_x(),
                        bottom: anchor.y() + bounds.max_y(),
                    },
                    geometry: StructureInteractionGeometryV1::CompactGroup,
                });
            }
            for bond in molecule.bonds() {
                let bond_object_id = bond.document_object_id();
                let batch = plan
                    .batches()
                    .iter()
                    .find(|batch| batch.target().document_object_id() == bond_object_id)
                    .ok_or(RenderInteractionErrorV1::Observation)?;
                let mut segments = Vec::new();
                let mut path_primitive_bounds = Vec::new();
                for operation in batch.operations() {
                    match operation {
                        RenderOp::Line(line) => {
                            let segment = StructureSegmentV1 {
                                start_x: line.start().x(),
                                start_y: line.start().y(),
                                end_x: line.end().x(),
                                end_y: line.end().y(),
                                stroke_radius: line.width().get() / 2.0,
                            };
                            segments.push(segment);
                        }
                        RenderOp::Path(path) => {
                            path_primitive_bounds.push(path_bounds(path));
                        }
                        RenderOp::Text(_)
                        | RenderOp::Mask(_)
                        | RenderOp::Ellipse(_)
                        | RenderOp::DoubleBondCarrierMark(_) => {}
                    }
                }
                if segments.is_empty() && path_primitive_bounds.is_empty() {
                    continue;
                }
                let directed_stereo =
                    matches!(bond.style(), Some(BondStyle::Wedge | BondStyle::Hashed));
                let has_path_primitive = !path_primitive_bounds.is_empty();
                let primitive_bounds = path_primitive_bounds
                    .into_iter()
                    .chain((!segments.is_empty()).then(|| segment_bounds(&segments)))
                    .collect::<Vec<_>>();
                let uses_envelope = directed_stereo || has_path_primitive;
                let (bounds, geometry) = if uses_envelope {
                    (
                        union_bounds(&primitive_bounds),
                        StructureInteractionGeometryV1::DirectedStereoBondEnvelope,
                    )
                } else {
                    (
                        segment_bounds(&segments),
                        StructureInteractionGeometryV1::Bond {
                            segments,
                            hit_slop: HIT_SLOP_PT_V1,
                        },
                    )
                };
                targets.push(StructureInteractionTargetV1 {
                    molecule_object_id: molecule_object_id.clone(),
                    object_id: bond_object_id.clone(),
                    source_order: batch.paint_order(),
                    kind: StructureTargetKindV1::Bond,
                    bounds,
                    geometry,
                });
            }
        }
        targets.sort_by_key(StructureInteractionTargetV1::source_order);
        Ok(StructureInteractionObservationV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            fence,
            targets,
        })
    }

    /// Resolve one point, full-containment marquee, or clear request entirely in Rust.
    pub fn select_structure_interaction_v1(
        &self,
        observation: &StructureInteractionObservationV1,
        previous: Option<&StructureInteractionSelectionV1>,
        query: StructureInteractionQueryV1,
    ) -> Result<StructureInteractionSelectionV1, RenderInteractionErrorV1> {
        self.require_structure_observation(observation)?;
        if let Some(selection) = previous {
            self.require_structure_selection(selection)?;
        }
        let (candidates, toggle) = match query {
            StructureInteractionQueryV1::Clear => (Vec::new(), false),
            StructureInteractionQueryV1::Point { x, y, modifier } => {
                if !x.is_finite() || !y.is_finite() {
                    return Err(RenderInteractionErrorV1::NonFinitePoint);
                }
                let atom_or_group = observation
                    .targets
                    .iter()
                    .filter(|target| {
                        matches!(
                            target.kind,
                            StructureTargetKindV1::Atom | StructureTargetKindV1::CompactGroup
                        ) && target.hit(x, y)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                let values = if atom_or_group.is_empty() {
                    observation
                        .targets
                        .iter()
                        .filter(|target| {
                            target.kind == StructureTargetKindV1::Bond && target.hit(x, y)
                        })
                        .cloned()
                        .collect::<Vec<_>>()
                } else {
                    atom_or_group
                };
                (values, modifier == RenderInteractionModifierV1::Toggle)
            }
            StructureInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                modifier,
            } => {
                if !left.is_finite()
                    || !top.is_finite()
                    || !right.is_finite()
                    || !bottom.is_finite()
                    || left > right
                    || top > bottom
                {
                    return Err(RenderInteractionErrorV1::InvalidRectangle);
                }
                let rectangle = RenderInteractionBoundsV1 {
                    left,
                    top,
                    right,
                    bottom,
                };
                let candidates = observation
                    .targets
                    .iter()
                    .filter(|target| target.fully_contained_by(rectangle))
                    .cloned()
                    .collect::<Vec<_>>();
                (candidates, modifier == RenderInteractionModifierV1::Toggle)
            }
        };
        let mut targets = if toggle {
            toggle_structure_targets(
                previous.map_or_else(Vec::new, |value| value.targets.clone()),
                candidates,
            )
        } else {
            candidates
        };
        targets.sort_by_key(StructureInteractionTargetV1::source_order);
        if targets
            .iter()
            .map(StructureInteractionTargetV1::molecule_object_id)
            .collect::<HashSet<_>>()
            .len()
            > 1
        {
            return Err(RenderInteractionErrorV1::CrossMoleculeSelection);
        }
        Ok(StructureInteractionSelectionV1 {
            origin: self.origin,
            capability: observation.capability,
            fence: observation.fence,
            targets,
        })
    }

    /// Commit the opaque direct-child selection as one fenced structural mutation.
    pub fn commit_structure_deletion_v1(
        &mut self,
        selection: &StructureInteractionSelectionV1,
    ) -> Result<CommittedStructureDeletionV1, RenderInteractionErrorV1> {
        self.require_structure_selection(selection)?;
        if selection.targets.is_empty() {
            return Err(RenderInteractionErrorV1::EmptySelection);
        }
        let molecule_object_id = selection.targets[0].molecule_object_id.clone();
        if selection
            .targets
            .iter()
            .any(|target| target.molecule_object_id != molecule_object_id)
        {
            return Err(RenderInteractionErrorV1::CrossMoleculeSelection);
        }
        let compact_groups = selection
            .targets
            .iter()
            .filter(|target| target.kind == StructureTargetKindV1::CompactGroup)
            .collect::<Vec<_>>();
        if !compact_groups.is_empty() {
            if selection.targets.len() != 1 || compact_groups.len() != 1 {
                return Err(RenderInteractionErrorV1::InvalidCompactGroupDeletionSelection);
            }
            let target = compact_groups[0];
            let molecule_object_id = target.molecule_object_id.clone();
            let compact_group_object_id = target.object_id.clone();
            let mut pending = self
                .session
                .prepare_delete_compact_group_v1(
                    selection.fence.revision(),
                    &molecule_object_id,
                    &compact_group_object_id,
                )
                .map_err(structure_deletion_prepare_error)?;
            let result = self
                .session
                .commit_delete_compact_group_v1(selection.fence.revision(), &mut pending)
                .map_err(structure_deletion_commit_error)?;
            return Ok(CommittedStructureDeletionV1 {
                result,
                removed_atom_count: 0,
                removed_bond_count: 1,
                removed_compact_group_count: 1,
            });
        }
        let (atom_ids, bond_ids) = structure_deletion_targets(selection);
        let mut pending = self
            .session
            .prepare_delete_structure_v1(
                selection.fence.revision(),
                &molecule_object_id,
                &atom_ids,
                &bond_ids,
            )
            .map_err(structure_deletion_prepare_error)?;
        let receipt = pending.receipt().clone();
        let result = self
            .session
            .commit_delete_structure_v1(selection.fence.revision(), &mut pending)
            .map_err(structure_deletion_commit_error)?;
        Ok(CommittedStructureDeletionV1 {
            result,
            removed_atom_count: receipt.removed_atom_ids().len(),
            removed_bond_count: receipt.removed_bond_ids().len(),
            removed_compact_group_count: 0,
        })
    }

    pub fn select_render_interaction_roots_v1(
        &self,
        observation: &RenderInteractionObservationV1,
        previous: Option<&RenderInteractionSelectionV1>,
        query: RenderInteractionQueryV1,
    ) -> Result<RenderInteractionSelectionV1, RenderInteractionErrorV1> {
        self.require_observation(observation)?;
        if let Some(value) = previous {
            self.require_selection(value)?;
        }
        let candidates = match &query {
            RenderInteractionQueryV1::Clear => Vec::new(),
            RenderInteractionQueryV1::Point { x, y, .. } => {
                if !x.is_finite() || !y.is_finite() {
                    return Err(RenderInteractionErrorV1::NonFinitePoint);
                }
                observation
                    .roots
                    .iter()
                    .filter(|root| root.bounds.contains_point(*x, *y))
                    .cloned()
                    .collect()
            }
            RenderInteractionQueryV1::Marquee {
                left,
                top,
                right,
                bottom,
                ..
            } => {
                if !left.is_finite()
                    || !top.is_finite()
                    || !right.is_finite()
                    || !bottom.is_finite()
                    || left > right
                    || top > bottom
                {
                    return Err(RenderInteractionErrorV1::InvalidRectangle);
                }
                let rectangle = RenderInteractionBoundsV1 {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                };
                observation
                    .roots
                    .iter()
                    .filter(|root| root.bounds.contained_by(rectangle))
                    .cloned()
                    .collect()
            }
            RenderInteractionQueryV1::Root {
                document_object_id, ..
            } => {
                if let Some(exclusion) = observation
                    .exclusions
                    .iter()
                    .find(|exclusion| exclusion.document_object_id() == document_object_id)
                {
                    return Err(match exclusion.reason {
                        RenderInteractionExclusionReasonV1::UnrenderableDepiction => {
                            RenderInteractionErrorV1::UnrenderableDepiction
                        }
                        RenderInteractionExclusionReasonV1::AmbiguousRootIdentifier => {
                            RenderInteractionErrorV1::AmbiguousRootIdentifier
                        }
                        RenderInteractionExclusionReasonV1::DisplayOnly => {
                            RenderInteractionErrorV1::DisplayOnly
                        }
                    });
                }
                observation
                    .roots
                    .iter()
                    .find(|root| root.document_object_id() == document_object_id)
                    .cloned()
                    .map_or_else(
                        || Err(RenderInteractionErrorV1::NoTarget),
                        |root| Ok(vec![root]),
                    )?
            }
        };
        let toggle = matches!(
            query,
            RenderInteractionQueryV1::Point {
                modifier: RenderInteractionModifierV1::Toggle,
                ..
            } | RenderInteractionQueryV1::Marquee {
                modifier: RenderInteractionModifierV1::Toggle,
                ..
            } | RenderInteractionQueryV1::Root {
                modifier: RenderInteractionModifierV1::Toggle,
                ..
            }
        );
        let roots = if toggle {
            toggle_roots(
                previous.map_or_else(Vec::new, |value| value.roots.clone()),
                candidates,
            )
        } else {
            candidates
        };
        Ok(RenderInteractionSelectionV1 {
            origin: self.origin,
            fence: observation.fence,
            roots,
        })
    }

    pub fn render_interaction_selection_contains_point_v1(
        &self,
        selection: &RenderInteractionSelectionV1,
        x: f64,
        y: f64,
    ) -> Result<bool, RenderInteractionErrorV1> {
        self.require_selection(selection)?;
        if !x.is_finite() || !y.is_finite() {
            return Err(RenderInteractionErrorV1::NonFinitePoint);
        }
        Ok(selection
            .roots
            .iter()
            .any(|root| root.bounds.contains_point(x, y)))
    }

    pub fn begin_render_interaction_translation_v1(
        &self,
        selection: &RenderInteractionSelectionV1,
        press_x: f64,
        press_y: f64,
        snap: RenderInteractionSnapV1,
    ) -> Result<RenderInteractionTranslationGestureV1, RenderInteractionErrorV1> {
        root_translation_interaction_v1::begin_root_translation_interaction_v1(
            self, selection, press_x, press_y, snap,
        )
    }

    pub fn preview_render_interaction_translation_v1(
        &self,
        gesture: &RenderInteractionTranslationGestureV1,
        pointer_x: f64,
        pointer_y: f64,
    ) -> Result<RenderInteractionTranslationPreviewV1, RenderInteractionErrorV1> {
        root_translation_interaction_v1::preview_root_translation_interaction_v1(
            self, gesture, pointer_x, pointer_y,
        )
    }

    pub fn commit_render_interaction_translation_v1(
        &mut self,
        gesture: RenderInteractionTranslationGestureV1,
        release_x: f64,
        release_y: f64,
    ) -> Result<CommittedRenderInteractionTranslationV1, RenderInteractionErrorV1> {
        root_translation_interaction_v1::commit_root_translation_interaction_v1(
            self, gesture, release_x, release_y,
        )
    }

    fn require_fence(&self, fence: DocumentFenceV1) -> Result<(), RenderInteractionErrorV1> {
        let snapshot = self
            .session
            .snapshot()
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        if snapshot.revision() != fence.revision() {
            return Err(RenderInteractionErrorV1::StaleRevision);
        }
        if snapshot.digest() != &fence.digest() {
            return Err(RenderInteractionErrorV1::StaleDigest);
        }
        Ok(())
    }
    fn require_observation(
        &self,
        value: &RenderInteractionObservationV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if value.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(value.fence)
    }
    pub(super) fn require_selection(
        &self,
        value: &RenderInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        self.require_fence(value.fence)
    }
    pub(super) fn require_gesture(
        &self,
        value: &RenderInteractionTranslationGestureV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        self.require_fence(value.selection.fence)
    }
    fn require_structure_observation(
        &self,
        value: &StructureInteractionObservationV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if value.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(value.fence)
    }
    fn require_structure_selection(
        &self,
        value: &StructureInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if value.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(value.fence)
    }
}

/// Separate durable atom and bond selections for the document-owned deletion boundary.
fn structure_deletion_targets(
    selection: &StructureInteractionSelectionV1,
) -> (Vec<DocumentObjectIdV1>, Vec<DocumentObjectIdV1>) {
    let atom_ids = selection
        .targets
        .iter()
        .filter(|target| target.kind == StructureTargetKindV1::Atom)
        .map(|target| target.object_id.clone())
        .collect();
    let bond_ids = selection
        .targets
        .iter()
        .filter(|target| target.kind == StructureTargetKindV1::Bond)
        .map(|target| target.object_id.clone())
        .collect();
    (atom_ids, bond_ids)
}

fn structure_deletion_prepare_error(error: DocumentSessionError) -> RenderInteractionErrorV1 {
    match error {
        DocumentSessionError::Operation(ferrum_document::SessionOperationError::Candidate(
            ferrum_document::TypedDocumentError::InvalidCompactGroupDeletionTopology(_),
        )) => RenderInteractionErrorV1::InvalidCompactGroupDeletionTopology,
        DocumentSessionError::RendererAdmission => RenderInteractionErrorV1::UnrenderableCandidate,
        _ => RenderInteractionErrorV1::UnsupportedTarget,
    }
}

fn structure_deletion_commit_error(error: DocumentSessionError) -> RenderInteractionErrorV1 {
    match error {
        DocumentSessionError::RendererAdmission => RenderInteractionErrorV1::UnrenderableCandidate,
        _ => RenderInteractionErrorV1::SessionConflict,
    }
}
impl Deref for RenderInteractionSessionV1 {
    type Target = DocumentSession;
    fn deref(&self) -> &Self::Target {
        &self.session
    }
}
impl DerefMut for RenderInteractionSessionV1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}
