//! Read-only, direct-root Haworth template observations.

use std::collections::HashSet;

use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_document::{
    CoreProjectionError, DocumentObjectIdV1, PersistentId, SessionDocumentObservationV1,
    TypedDocument, TypedDocumentError,
};
use ferrum_domain::haworth::{
    HaworthError, HaworthRingNode, HaworthTopologyBuilder, HaworthTreeRequest, HaworthVertex,
    RingForm, layout_tree,
};
use ferrum_render::{
    HaworthRenderRequest, MoleculeRenderPlan, Paint, PositiveFinite, RenderError, RenderProvenance,
    RenderRevision, lower_haworth_fragment,
};
use thiserror::Error;

/// A checked request for one direct molecule's local Haworth template.
///
/// The selected cycle is ordered by the caller, but domain validation
/// canonicalizes accepted rotations and reversals. This request does not carry
/// page placement, source coordinates, stereochemistry, or a mutation.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentHaworthObservationRequestV1 {
    molecule_id: DocumentObjectIdV1,
    ring_form: RingForm,
    cycle_atom_ids: Vec<PersistentId>,
    anomeric_atom_id: PersistentId,
    scale: PositiveFinite,
    line_width: PositiveFinite,
    line_paint: Paint,
}

impl DocumentHaworthObservationRequestV1 {
    /// Validate request-local shape before inspecting a document observation.
    pub fn new(
        molecule_id: DocumentObjectIdV1,
        ring_form: RingForm,
        cycle_atom_ids: Vec<PersistentId>,
        anomeric_atom_id: PersistentId,
        scale: PositiveFinite,
        line_width: PositiveFinite,
        line_paint: Paint,
    ) -> Result<Self, DocumentHaworthRequestErrorV1> {
        let expected = match ring_form {
            RingForm::Pyranose => 6,
            RingForm::Furanose => 5,
        };
        if cycle_atom_ids.len() != expected {
            return Err(DocumentHaworthRequestErrorV1::WrongCycleMemberCount {
                ring_form,
                actual: cycle_atom_ids.len(),
            });
        }
        let unique = cycle_atom_ids.iter().collect::<HashSet<_>>();
        if unique.len() != cycle_atom_ids.len() {
            return Err(DocumentHaworthRequestErrorV1::DuplicateCycleAtom);
        }
        if !cycle_atom_ids.contains(&anomeric_atom_id) {
            return Err(DocumentHaworthRequestErrorV1::AnomericAtomNotInCycle);
        }
        Ok(Self {
            molecule_id,
            ring_form,
            cycle_atom_ids,
            anomeric_atom_id,
            scale,
            line_width,
            line_paint,
        })
    }

    /// Return the durable selected direct molecule identity.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the explicitly requested ring form.
    #[must_use]
    pub const fn ring_form(&self) -> RingForm {
        self.ring_form
    }

    /// Return the caller-provided ordered selected atom identities.
    #[must_use]
    pub fn cycle_atom_ids(&self) -> &[PersistentId] {
        &self.cycle_atom_ids
    }

    /// Return the selected anomeric atom identity.
    #[must_use]
    pub const fn anomeric_atom_id(&self) -> &PersistentId {
        &self.anomeric_atom_id
    }

    /// Return the explicit template edge scale.
    #[must_use]
    pub const fn scale(&self) -> PositiveFinite {
        self.scale
    }

    /// Return the explicit output line width.
    #[must_use]
    pub const fn line_width(&self) -> PositiveFinite {
        self.line_width
    }

    /// Return the explicit output line paint.
    #[must_use]
    pub const fn line_paint(&self) -> &Paint {
        &self.line_paint
    }
}

/// Request-local failures that require no document lookup.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum DocumentHaworthRequestErrorV1 {
    /// The named ring form requires a different selected member count.
    #[error("{ring_form:?} requires a different cycle-member count; received {actual}")]
    WrongCycleMemberCount {
        /// Requested ring form.
        ring_form: RingForm,
        /// Submitted selected member count.
        actual: usize,
    },
    /// The submitted ordered cycle repeated an atom identity.
    #[error("Haworth cycle atom identities must be unique")]
    DuplicateCycleAtom,
    /// The selected anomeric atom was not one of the cycle members.
    #[error("Haworth anomeric atom must be one selected cycle member")]
    AnomericAtomNotInCycle,
}

/// The selected molecule's direct CDML-root identity and order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentHaworthRootV1 {
    molecule_id: DocumentObjectIdV1,
    projection_key: String,
    source_id: String,
    document_root_order: u32,
}

impl DocumentHaworthRootV1 {
    /// Return the durable selected direct-molecule identity.
    #[must_use]
    pub const fn molecule_id(&self) -> &DocumentObjectIdV1 {
        &self.molecule_id
    }

    /// Return the exact projection-local key from this observation.
    #[must_use]
    pub fn projection_key(&self) -> &str {
        &self.projection_key
    }

    /// Return the literal CDML molecule source ID.
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    /// Return this molecule's direct CDML-root position.
    ///
    /// This is separate from molecule-local bond `source_order` values carried
    /// by the returned render plan.
    #[must_use]
    pub const fn document_root_order(&self) -> u32 {
        self.document_root_order
    }
}

/// Finite bounds in a Haworth template's local coordinate system.
///
/// These are neither CDML coordinates, a document-page viewport, nor a page
/// placement transform.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HaworthTemplateBoundsV1 {
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
}

impl HaworthTemplateBoundsV1 {
    fn new(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Option<Self> {
        (min_x.is_finite()
            && min_y.is_finite()
            && max_x.is_finite()
            && max_y.is_finite()
            && min_x <= max_x
            && min_y <= max_y)
            .then_some(Self {
                min_x,
                min_y,
                max_x,
                max_y,
            })
    }

    /// Return the local minimum x coordinate.
    #[must_use]
    pub const fn min_x(self) -> f64 {
        self.min_x
    }

    /// Return the local minimum y coordinate.
    #[must_use]
    pub const fn min_y(self) -> f64 {
        self.min_y
    }

    /// Return the local maximum x coordinate.
    #[must_use]
    pub const fn max_x(self) -> f64 {
        self.max_x
    }

    /// Return the local maximum y coordinate.
    #[must_use]
    pub const fn max_y(self) -> f64 {
        self.max_y
    }
}

/// One immutable Haworth template plan bound to one exact source observation.
#[derive(Clone, Debug, PartialEq)]
pub struct DocumentHaworthObservationV1 {
    provenance: RenderProvenance,
    root: DocumentHaworthRootV1,
    template_bounds: HaworthTemplateBoundsV1,
    plan: MoleculeRenderPlan,
}

impl DocumentHaworthObservationV1 {
    /// Return the exact revision and digest that produced this observation.
    #[must_use]
    pub const fn provenance(&self) -> RenderProvenance {
        self.provenance
    }

    /// Return the source document revision.
    #[must_use]
    pub const fn source_revision(&self) -> u64 {
        self.provenance.revision().get()
    }

    /// Return the source document digest.
    #[must_use]
    pub const fn source_digest(&self) -> [u8; 32] {
        self.provenance.digest()
    }

    /// Return selected direct-root facts only.
    #[must_use]
    pub const fn root(&self) -> &DocumentHaworthRootV1 {
        &self.root
    }

    /// Return finite bounds in the Haworth template's own local coordinates.
    #[must_use]
    pub const fn template_bounds(&self) -> HaworthTemplateBoundsV1 {
        self.template_bounds
    }

    /// Return the selected molecule-local Haworth render plan.
    #[must_use]
    pub const fn plan(&self) -> &MoleculeRenderPlan {
        &self.plan
    }
}

/// Observe one validated Haworth template without changing session state.
///
/// The caller must first obtain `observation` with the intended
/// `DocumentSession::observe` revision. This adapter does not expose document
/// page composition, PNG/PDF output, CDML placement, stereochemistry inference,
/// RDKit, OASA, or mutation authority.
pub fn observe_document_haworth_v1(
    observation: &SessionDocumentObservationV1,
    request: &DocumentHaworthObservationRequestV1,
) -> Result<DocumentHaworthObservationV1, DocumentHaworthObservationErrorV1> {
    let projection_root = observation
        .projection()
        .molecules()
        .iter()
        .find(|molecule| molecule.id() == Some(request.molecule_id()))
        .ok_or_else(
            || DocumentHaworthObservationErrorV1::UnknownDirectMolecule {
                object_id: request.molecule_id().as_str().to_owned(),
            },
        )?;
    let root = DocumentHaworthRootV1 {
        molecule_id: request.molecule_id().clone(),
        projection_key: projection_root.projection_key().as_str().to_owned(),
        source_id: projection_root
            .source_id()
            .ok_or(DocumentHaworthObservationErrorV1::ProjectionRootMismatch)?
            .to_owned(),
        document_root_order: projection_root.source_order(),
    };

    let document = TypedDocument::parse(observation.snapshot().cdml())?;
    if observation.snapshot().revision() != observation.projection().revision()
        || observation.snapshot().digest() != observation.projection().digest()
    {
        return Err(DocumentHaworthObservationErrorV1::ProjectionRootMismatch);
    }
    let molecule = document
        .core_molecule(request.molecule_id())?
        .ok_or(DocumentHaworthObservationErrorV1::ProjectionRootMismatch)?;
    if molecule.source_id().map(Identifier::as_str) != Some(root.source_id()) {
        return Err(DocumentHaworthObservationErrorV1::ProjectionRootMismatch);
    }

    let member_ids = molecule
        .atoms()
        .iter()
        .map(|atom| atom.identity().clone())
        .collect::<HashSet<RecordId>>();
    let selected = request
        .cycle_atom_ids()
        .iter()
        .map(persistent_atom_id)
        .collect::<Result<Vec<_>, _>>()?;
    for (persistent_id, atom_id) in request.cycle_atom_ids().iter().zip(&selected) {
        if !member_ids.contains(atom_id) {
            return Err(
                DocumentHaworthObservationErrorV1::SelectedAtomNotInMolecule {
                    atom_id: persistent_id.as_str().to_owned(),
                },
            );
        }
    }
    let anomeric = persistent_atom_id(request.anomeric_atom_id())?;
    if !member_ids.contains(&anomeric) {
        return Err(
            DocumentHaworthObservationErrorV1::SelectedAtomNotInMolecule {
                atom_id: request.anomeric_atom_id().as_str().to_owned(),
            },
        );
    }

    let topology = HaworthTopologyBuilder::new(
        request.ring_form(),
        anomeric,
        selected
            .into_iter()
            .map(|atom| HaworthVertex { atom })
            .collect(),
    )
    .build(&molecule)?;
    let fragment = layout_tree(&HaworthTreeRequest {
        molecule,
        rings: vec![HaworthRingNode {
            node_id: 0,
            topology,
        }],
        links: Vec::new(),
        root: 0,
        scale: request.scale().get(),
    })?;
    let [minimum, maximum] = fragment.bounds();
    let template_bounds = HaworthTemplateBoundsV1::new(minimum.x, minimum.y, maximum.x, maximum.y)
        .ok_or(DocumentHaworthObservationErrorV1::ProjectionRootMismatch)?;
    let provenance = RenderProvenance::new(
        RenderRevision::new(observation.snapshot().revision())?,
        *observation.snapshot().digest(),
    );
    let plan = lower_haworth_fragment(&HaworthRenderRequest {
        provenance,
        fragment,
        line_width: request.line_width(),
        line_paint: request.line_paint().clone(),
    })?;
    if plan.provenance() != provenance {
        return Err(DocumentHaworthObservationErrorV1::ProvenanceMismatch);
    }
    Ok(DocumentHaworthObservationV1 {
        provenance,
        root,
        template_bounds,
        plan,
    })
}

fn persistent_atom_id(
    persistent_id: &PersistentId,
) -> Result<RecordId, DocumentHaworthObservationErrorV1> {
    let identifier = Identifier::new(persistent_id.as_str())
        .map_err(|_| DocumentHaworthObservationErrorV1::ProjectionRootMismatch)?;
    Ok(RecordId::from_source(RecordKind::Atom, &identifier))
}

/// Failures while deriving one document-bound Haworth template observation.
#[derive(Debug, Error)]
pub enum DocumentHaworthObservationErrorV1 {
    /// The immutable source snapshot could not be reparsed.
    #[error(transparent)]
    Document(#[from] TypedDocumentError),
    /// A direct durable molecule could not form its core graph.
    #[error(transparent)]
    CoreProjection(#[from] CoreProjectionError),
    /// The requested selector was not a durable direct-root molecule.
    #[error(
        "document object is not a durable direct-root molecule in this observation: {object_id}"
    )]
    UnknownDirectMolecule {
        /// Opaque requested selector.
        object_id: String,
    },
    /// A selected source-backed atom was not in the selected direct molecule.
    #[error("selected Haworth atom is not in the selected direct molecule: {atom_id}")]
    SelectedAtomNotInMolecule {
        /// Submitted atom ID, without resolving it elsewhere in the document.
        atom_id: String,
    },
    /// The selected cycle cannot meet the isolated Haworth topology profile.
    #[error(transparent)]
    Topology(#[from] HaworthError),
    /// The accepted fragment could not be lowered to the render-plan grammar.
    #[error(transparent)]
    Render(#[from] RenderError),
    /// Projection and typed-core facts disagreed after direct-root proof.
    #[error("document projection and typed core disagree for the selected direct molecule")]
    ProjectionRootMismatch,
    /// The lowerer did not retain the exact source observation provenance.
    #[error("Haworth render plan provenance does not match its source observation")]
    ProvenanceMismatch,
}
