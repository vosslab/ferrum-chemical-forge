use super::*;

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

    /// Return the private renderer-session identity for sibling opaque bridges.
    ///
    /// This must not be confused with the embedded document-session identity:
    /// observations and selections are issued by this renderer boundary.
    #[must_use]
    pub(crate) const fn render_interaction_origin_v1(&self) -> u64 {
        self.origin
    }

    /// Return the opaque document authoring authority for receipt lifecycles
    /// that mutate this interaction session's embedded document.
    #[must_use]
    pub(crate) fn authoring_capability_issuer_v1(
        &self,
    ) -> ferrum_document::AuthoringCapabilityIssuerV1 {
        self.session.authoring_capability_issuer_v1()
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
        Ok(RenderInteractionObservationV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            fence,
            roots,
            exclusions,
        })
    }

    /// Classify the exact current renderer-admitted roots for reaction authoring.
    ///
    /// The namespace-aware semantic index supplies durable kind and existing
    /// reaction membership; the direct-root interaction observation supplies
    /// only admitted geometry and identity. Neither source is sufficient alone.
    pub fn observe_reaction_authoring_choices_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<ReactionAuthoringChoicesV1, RenderInteractionErrorV1> {
        let roots = self.observe_render_interaction_v1(fence)?;
        let snapshot = self
            .session
            .snapshot()
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        let index = DirectCdmlSemanticIndexV1::parse(snapshot.cdml())
            .map_err(|_| RenderInteractionErrorV1::Observation)?;
        let members = index
            .roots()
            .iter()
            .filter(|root| root.kind() == DirectCdmlRootKindV1::Reaction)
            .flat_map(|root| root.reaction_members().iter().cloned())
            .collect::<HashSet<_>>();
        let mut choices = Vec::new();
        let mut exclusions = Vec::new();
        let mut diagnosed = HashSet::new();
        for root in roots.roots() {
            let semantic = index
                .roots()
                .iter()
                .filter(|candidate| candidate.identifier() == Some(root.identifier()))
                .collect::<Vec<_>>();
            match semantic.as_slice() {
                [] => push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    root.identifier(),
                    ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity,
                    reaction_exclusion_label(
                        ReactionAuthoringExclusionReasonV1::MissingSemanticIdentity,
                        root.identifier(),
                    ),
                ),
                [semantic] => match reaction_choice_kind(semantic.kind(), root.kind()) {
                    Some(kind) => {
                        let availability = if members.contains(root.identifier()) {
                            ReactionAuthoringChoiceAvailabilityV1::AlreadyInReaction
                        } else {
                            ReactionAuthoringChoiceAvailabilityV1::Eligible
                        };
                        choices.push(ReactionAuthoringChoiceV1 {
                            identifier: root.identifier().to_owned(),
                            source_order: root.source_order(),
                            kind,
                            availability,
                            label: reaction_choice_label(kind, root.identifier()),
                            bounds: root.bounds(),
                        });
                    }
                    None => {
                        let reason = reaction_root_exclusion_reason(semantic.kind(), root.kind());
                        push_reaction_exclusion(
                            &mut exclusions,
                            &mut diagnosed,
                            root.identifier(),
                            reason,
                            reaction_exclusion_label(reason, root.identifier()),
                        );
                    }
                },
                _ => push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    root.identifier(),
                    ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity,
                    reaction_exclusion_label(
                        ReactionAuthoringExclusionReasonV1::AmbiguousSemanticIdentity,
                        root.identifier(),
                    ),
                ),
            }
        }
        for value in roots.exclusions() {
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
            let label = index
                .roots()
                .iter()
                .find(|root| root.identifier() == Some(value.identifier()))
                .and_then(|root| direct_reaction_choice_kind(root.kind()))
                .map_or_else(
                    || reaction_exclusion_label(reason, value.identifier()),
                    |kind| reaction_choice_label(kind, value.identifier()),
                );
            push_reaction_exclusion(
                &mut exclusions,
                &mut diagnosed,
                value.identifier(),
                reason,
                label,
            );
        }
        let observed = roots
            .roots()
            .iter()
            .map(RenderInteractionRootV1::identifier)
            .collect::<HashSet<_>>();
        for root in index.roots() {
            let Some(identifier) = root.identifier() else {
                continue;
            };
            let Some(kind) = direct_reaction_choice_kind(root.kind()) else {
                continue;
            };
            if !observed.contains(identifier) {
                push_reaction_exclusion(
                    &mut exclusions,
                    &mut diagnosed,
                    identifier,
                    ReactionAuthoringExclusionReasonV1::Unrenderable,
                    reaction_choice_label(kind, identifier),
                );
            }
        }
        choices.sort_by_key(ReactionAuthoringChoiceV1::source_order);
        Ok(ReactionAuthoringChoicesV1 {
            origin: self.origin,
            capability: roots.capability,
            fence,
            choices,
            exclusions,
        })
    }

    /// Return all retained direct reaction records with renderer-backed member facts.
    pub fn observe_reaction_list_v1(
        &self,
        fence: DocumentFenceV1,
    ) -> Result<ReactionListObservationV1, RenderInteractionErrorV1> {
        let rendered = self.observe_render_interaction_v1(fence)?;
        reaction_observation_v1::observe_reaction_list_v1(&self.session, self.origin, &rendered)
    }

    /// Refuse a foreign or stale reaction list without mutating CDML.
    pub fn validate_reaction_list_v1(
        &self,
        list: &ReactionListObservationV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        reaction_observation_v1::validate_reaction_list_v1(&self.session, self.origin, list)
    }

    /// Issue an opaque aggregate-selection capability from one fresh list fact.
    pub fn select_reaction_v1(
        &self,
        list: &ReactionListObservationV1,
        reaction_id: &str,
    ) -> Result<ReactionSelectionV1, RenderInteractionErrorV1> {
        reaction_observation_v1::select_reaction_v1(&self.session, self.origin, list, reaction_id)
    }

    /// Validate an opaque reaction selection before a future lifecycle mutation.
    pub fn validate_reaction_selection_v1(
        &self,
        selection: &ReactionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        reaction_observation_v1::validate_reaction_selection_v1(
            &self.session,
            self.origin,
            selection,
        )
    }

    /// Refuse a foreign or stale immutable composer observation without mutation.
    pub fn validate_reaction_authoring_choices_v1(
        &self,
        choices: &ReactionAuthoringChoicesV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if choices.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        if choices.capability == 0 {
            return Err(RenderInteractionErrorV1::SelectionChanged);
        }
        self.require_fence(choices.fence)
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
            let Some(molecule_id) = molecule.source_id() else {
                continue;
            };
            for atom in molecule.atoms() {
                let Some(identifier) = atom.source_id() else {
                    continue;
                };
                let point = atom.position();
                targets.push(StructureInteractionTargetV1 {
                    molecule_id: molecule_id.to_owned(),
                    identifier: identifier.to_owned(),
                    source_order: atom.source_order(),
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
            let Some(plan) = rendered
                .resolved()
                .molecule_plans()
                .iter()
                .find(|entry| entry.molecule().source_id() == Some(molecule_id))
            else {
                continue;
            };
            for group in plan.compact_group_primitives() {
                let bounds = group.bounds();
                let anchor = group.anchor();
                targets.push(StructureInteractionTargetV1 {
                    molecule_id: molecule_id.to_owned(),
                    identifier: group.identifier().to_owned(),
                    source_order: group.target().source_order(),
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
                let Some(identifier) = bond.source_id() else {
                    continue;
                };
                let operations = plan
                    .batches()
                    .iter()
                    .filter(|batch| batch.target().source_order() == bond.source_order())
                    .flat_map(|batch| batch.operations());
                let mut segments = Vec::new();
                let mut primitive_bounds = Vec::new();
                let mut has_path = false;
                for operation in operations {
                    match operation {
                        RenderOp::Line(line) => {
                            let segment = StructureSegmentV1 {
                                start_x: line.start().x(),
                                start_y: line.start().y(),
                                end_x: line.end().x(),
                                end_y: line.end().y(),
                                stroke_radius: line.width().get() / 2.0,
                            };
                            primitive_bounds.push(segment_bounds(std::slice::from_ref(&segment)));
                            segments.push(segment);
                        }
                        RenderOp::Path(path) => {
                            has_path = true;
                            primitive_bounds.push(path_bounds(&path));
                        }
                        RenderOp::Text(_) | RenderOp::Mask(_) | RenderOp::Ellipse(_) => {}
                    }
                }
                if primitive_bounds.is_empty() {
                    continue;
                }
                targets.push(StructureInteractionTargetV1 {
                    molecule_id: molecule_id.to_owned(),
                    identifier: identifier.to_owned(),
                    source_order: bond.source_order(),
                    kind: if has_path {
                        StructureTargetKindV1::DisplayOnly
                    } else {
                        StructureTargetKindV1::Bond
                    },
                    bounds: union_bounds(&primitive_bounds),
                    geometry: if has_path {
                        StructureInteractionGeometryV1::DisplayOnly
                    } else {
                        StructureInteractionGeometryV1::Bond { segments }
                    },
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
                    let bonds = observation
                        .targets
                        .iter()
                        .filter(|target| {
                            target.kind == StructureTargetKindV1::Bond && target.hit(x, y)
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    if bonds.is_empty()
                        && observation.targets.iter().any(|target| {
                            target.kind == StructureTargetKindV1::DisplayOnly && target.hit(x, y)
                        })
                    {
                        return Err(RenderInteractionErrorV1::DisplayOnly);
                    }
                    bonds
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
                    .filter(|target| {
                        target.kind != StructureTargetKindV1::DisplayOnly
                            && target.fully_contained_by(rectangle)
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                if observation.targets.iter().any(|target| {
                    target.kind == StructureTargetKindV1::DisplayOnly
                        && target.fully_contained_by(rectangle)
                }) {
                    return Err(RenderInteractionErrorV1::DisplayOnly);
                }
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
            .map(StructureInteractionTargetV1::molecule_id)
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
        let molecule_id = selection.targets[0].molecule_id.clone();
        if selection
            .targets
            .iter()
            .any(|target| target.molecule_id != molecule_id)
        {
            return Err(RenderInteractionErrorV1::CrossMoleculeSelection);
        }
        let atom_ids = selection
            .targets
            .iter()
            .filter(|target| target.kind == StructureTargetKindV1::Atom)
            .map(|target| target.identifier.clone())
            .collect::<Vec<_>>();
        let bond_ids = selection
            .targets
            .iter()
            .filter(|target| target.kind == StructureTargetKindV1::Bond)
            .map(|target| target.identifier.clone())
            .collect::<Vec<_>>();
        if selection
            .targets
            .iter()
            .any(|target| target.kind == StructureTargetKindV1::DisplayOnly)
        {
            return Err(RenderInteractionErrorV1::DisplayOnly);
        }
        if selection
            .targets
            .iter()
            .any(|target| target.kind == StructureTargetKindV1::CompactGroup)
        {
            return Err(RenderInteractionErrorV1::UnsupportedTarget);
        }
        let mut pending = self
            .session
            .prepare_delete_structure_v1(
                selection.fence.revision(),
                molecule_id,
                atom_ids,
                bond_ids,
            )
            .map_err(structure_deletion_prepare_error)?;
        let receipt = pending.receipt().clone();
        let result = self
            .session
            .commit_delete_structure_v1(selection.fence.revision(), &mut pending)
            .map_err(structure_deletion_commit_error)?;
        let (removed_atoms, removed_bonds, components) = structure_deletion_receipt(receipt);
        Ok(CommittedStructureDeletionV1 {
            result,
            removed_atoms,
            removed_bonds,
            components,
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
            RenderInteractionQueryV1::Root { identifier, .. } => {
                if let Some(exclusion) = observation
                    .exclusions
                    .iter()
                    .find(|exclusion| exclusion.identifier == *identifier)
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
                    .find(|root| root.identifier == *identifier)
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
        self.require_selection(selection)?;
        if selection.is_empty() {
            return Err(RenderInteractionErrorV1::EmptySelection);
        }
        if !press_x.is_finite() || !press_y.is_finite() {
            return Err(RenderInteractionErrorV1::NonFinitePoint);
        }
        Ok(RenderInteractionTranslationGestureV1 {
            origin: self.origin,
            capability: NEXT_CAPABILITY.fetch_add(1, Ordering::Relaxed),
            selection: selection.clone(),
            press_x,
            press_y,
            snap,
        })
    }

    pub fn preview_render_interaction_translation_v1(
        &self,
        gesture: &RenderInteractionTranslationGestureV1,
        pointer_x: f64,
        pointer_y: f64,
    ) -> Result<RenderInteractionTranslationPreviewV1, RenderInteractionErrorV1> {
        self.require_gesture(gesture)?;
        self.require_selection(&gesture.selection)?;
        if !pointer_x.is_finite() || !pointer_y.is_finite() {
            return Err(RenderInteractionErrorV1::NonFinitePoint);
        }
        let raw_dx = pointer_x - gesture.press_x;
        let raw_dy = pointer_y - gesture.press_y;
        let (mut dx, mut dy) = match gesture.snap.grid_policy {
            RenderInteractionGridSnapPolicyV1::Free => (raw_dx, raw_dy),
            RenderInteractionGridSnapPolicyV1::ViewHexGrid => {
                if raw_dx == 0.0 && raw_dy == 0.0 {
                    (0.0, 0.0)
                } else {
                    let targets = gesture
                        .selection
                        .roots
                        .iter()
                        .map(|root| {
                            TopLevelRootSelectorV1::new(root.identifier.clone(), root.kind)
                                .map_err(|_| RenderInteractionErrorV1::SelectionChanged)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let anchor = self
                        .session
                        .observe_top_level_translation_anchor_v1(
                            gesture.selection.fence.revision(),
                            targets,
                        )
                        .map_err(|_| RenderInteractionErrorV1::SelectionChanged)?;
                    let (anchor_x, anchor_y) = anchor.anchor();
                    let origin =
                        Point2::new(0.0, 0.0).map_err(|_| RenderInteractionErrorV1::Observation)?;
                    let grid = HexGrid::new(VIEW_HEX_GRID_SPACING_PT_V1, origin)
                        .map_err(|_| RenderInteractionErrorV1::Observation)?;
                    let snapped_anchor = grid
                        .snap(
                            Point2::new(anchor_x + raw_dx, anchor_y + raw_dy)
                                .map_err(|_| RenderInteractionErrorV1::NonFinitePoint)?,
                        )
                        .map_err(|_| RenderInteractionErrorV1::Observation)?;
                    (snapped_anchor.x() - anchor_x, snapped_anchor.y() - anchor_y)
                }
            }
        };
        match gesture.snap.axis {
            RenderInteractionAxisV1::Free => {}
            RenderInteractionAxisV1::Horizontal => dy = 0.0,
            RenderInteractionAxisV1::Vertical => dx = 0.0,
        }
        Ok(RenderInteractionTranslationPreviewV1 {
            capability: gesture.capability,
            dx,
            dy,
            bounds: gesture
                .selection
                .roots
                .iter()
                .map(|root| root.bounds.translated(dx, dy))
                .collect(),
        })
    }

    pub fn commit_render_interaction_translation_v1(
        &mut self,
        gesture: &RenderInteractionTranslationGestureV1,
        preview: &RenderInteractionTranslationPreviewV1,
    ) -> Result<CommittedRenderInteractionTranslationV1, RenderInteractionErrorV1> {
        self.require_gesture(gesture)?;
        self.require_selection(&gesture.selection)?;
        if preview.capability != gesture.capability {
            return Err(RenderInteractionErrorV1::PreviewMismatch);
        }
        let targets = gesture
            .selection
            .roots
            .iter()
            .map(|root| {
                TopLevelRootSelectorV1::new(root.identifier.clone(), root.kind)
                    .map_err(|_| RenderInteractionErrorV1::SelectionChanged)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let transform = TopLevelTransformV1::new(
            targets,
            TopLevelTransformModeV1::Translate {
                dx: preview.dx,
                dy: preview.dy,
            },
        )
        .map_err(|_| RenderInteractionErrorV1::SelectionChanged)?;
        let result = self
            .session
            .submit(
                gesture.selection.fence.revision(),
                SessionOperation::V1(SessionOperationV1::TransformTopLevelRoots { transform }),
            )
            .map_err(|_| RenderInteractionErrorV1::SessionConflict)?;
        Ok(CommittedRenderInteractionTranslationV1 {
            changed: preview.dx != 0.0 || preview.dy != 0.0,
            result,
            selection: gesture.selection.clone(),
        })
    }

    /// Validate a preview without mutating the document. Tool-specific bridge
    /// owners use this before deriving a renderer-admitted detached candidate.
    pub(crate) fn validate_render_interaction_translation_preview_v1(
        &self,
        gesture: &RenderInteractionTranslationGestureV1,
        preview: &RenderInteractionTranslationPreviewV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        self.require_gesture(gesture)?;
        self.require_selection(&gesture.selection)?;
        (preview.capability == gesture.capability)
            .then_some(())
            .ok_or(RenderInteractionErrorV1::PreviewMismatch)
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
    fn require_selection(
        &self,
        value: &RenderInteractionSelectionV1,
    ) -> Result<(), RenderInteractionErrorV1> {
        if value.origin != self.origin {
            return Err(RenderInteractionErrorV1::ForeignSession);
        }
        self.require_fence(value.fence)
    }
    fn require_gesture(
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

fn structure_deletion_prepare_error(error: DocumentSessionError) -> RenderInteractionErrorV1 {
    match error {
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
