//! Immutable atom projection values without document traversal authority.

use serde::Serialize;

use super::AtomMarkProjectionV1;
use crate::{
    DocumentObjectIdV1, FontFactsV1, Point3V1, ProjectionLocalObjectKeyV1, RichTextV1,
    TransparentOrRgb24V1, VisibilityV1,
};

/// Immutable atom facts in source order.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AtomProjectionV1 {
    id: DocumentObjectIdV1,
    projection_key: ProjectionLocalObjectKeyV1,
    source_id: Option<String>,
    source_order: u32,
    element: Option<String>,
    position: Point3V1,
    formal_charge: Option<i32>,
    isotope: Option<u16>,
    explicit_hydrogens: Option<u16>,
    valence: Option<u16>,
    multiplicity: Option<u16>,
    free_sites: Option<u16>,
    number: Option<u64>,
    show_number: Option<VisibilityV1>,
    marks: Vec<AtomMarkProjectionV1>,
    label_font: Option<FontFactsV1>,
    label_text: Option<RichTextV1>,
    show: Option<VisibilityV1>,
    hydrogens: Option<VisibilityV1>,
    background_color: Option<TransparentOrRgb24V1>,
}

impl AtomProjectionV1 {
    /// Construct an atom projection from already-validated source facts.
    #[expect(
        clippy::too_many_arguments,
        reason = "each immutable atom fact remains explicit at the projection boundary"
    )]
    #[must_use]
    pub fn new(
        id: DocumentObjectIdV1,
        projection_key: ProjectionLocalObjectKeyV1,
        source_id: Option<String>,
        source_order: u32,
        element: Option<String>,
        position: Point3V1,
        formal_charge: Option<i32>,
        isotope: Option<u16>,
        explicit_hydrogens: Option<u16>,
        valence: Option<u16>,
        multiplicity: Option<u16>,
        free_sites: Option<u16>,
        number: Option<u64>,
        show_number: Option<VisibilityV1>,
        marks: Vec<AtomMarkProjectionV1>,
        label_font: Option<FontFactsV1>,
        label_text: Option<RichTextV1>,
        show: Option<VisibilityV1>,
        hydrogens: Option<VisibilityV1>,
        background_color: Option<TransparentOrRgb24V1>,
    ) -> Self {
        Self {
            id,
            projection_key,
            source_id,
            source_order,
            element,
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
            valence,
            multiplicity,
            free_sites,
            number,
            show_number,
            marks,
            label_font,
            label_text,
            show,
            hydrogens,
            background_color,
        }
    }
    /// Return the exact durable document object ID for this retained atom.
    #[must_use]
    pub fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.id
    }
    /// Return the non-operation key unique within this projection.
    #[must_use]
    pub fn projection_key(&self) -> &ProjectionLocalObjectKeyV1 {
        &self.projection_key
    }
    /// Return the literal CDML ID when authored.
    #[must_use]
    pub fn source_id(&self) -> Option<&str> {
        self.source_id.as_deref()
    }
    /// Return the child position in its molecule.
    #[must_use]
    pub fn source_order(&self) -> u32 {
        self.source_order
    }
    /// Return the authored element spelling.
    #[must_use]
    pub fn element(&self) -> Option<&str> {
        self.element.as_deref()
    }
    /// Return finite atom coordinates.
    #[must_use]
    pub fn position(&self) -> Point3V1 {
        self.position
    }
    /// Return authored formal charge.
    #[must_use]
    pub fn formal_charge(&self) -> Option<i32> {
        self.formal_charge
    }
    /// Return authored isotope mass number.
    #[must_use]
    pub fn isotope(&self) -> Option<u16> {
        self.isotope
    }
    /// Return authored explicit hydrogens.
    #[must_use]
    pub fn explicit_hydrogens(&self) -> Option<u16> {
        self.explicit_hydrogens
    }
    /// Return authored valence.
    #[must_use]
    pub fn valence(&self) -> Option<u16> {
        self.valence
    }
    /// Return authored multiplicity.
    #[must_use]
    pub fn multiplicity(&self) -> Option<u16> {
        self.multiplicity
    }
    /// Return authored free-site count.
    #[must_use]
    pub fn free_sites(&self) -> Option<u16> {
        self.free_sites
    }
    /// Return the authored positive decimal atom number when valid.
    #[must_use]
    pub fn number(&self) -> Option<u64> {
        self.number
    }
    /// Return the authored number-label visibility fact.
    #[must_use]
    pub fn show_number(&self) -> Option<VisibilityV1> {
        self.show_number
    }
    /// Return supported direct atom marks in persistent child order.
    #[must_use]
    pub fn marks(&self) -> &[AtomMarkProjectionV1] {
        &self.marks
    }
    /// Return authored label font facts.
    #[must_use]
    pub fn label_font(&self) -> Option<&FontFactsV1> {
        self.label_font.as_ref()
    }
    /// Return authored formatted label text.
    #[must_use]
    pub fn label_text(&self) -> Option<&RichTextV1> {
        self.label_text.as_ref()
    }
    /// Return the typed persisted atom show fact.
    #[must_use]
    pub fn show(&self) -> Option<VisibilityV1> {
        self.show
    }
    /// Return the typed persisted atom hydrogen display fact.
    #[must_use]
    pub fn hydrogens(&self) -> Option<VisibilityV1> {
        self.hydrogens
    }
    /// Return the authored label-mask colour or transparent directive.
    #[must_use]
    pub fn background_color(&self) -> Option<&TransparentOrRgb24V1> {
        self.background_color.as_ref()
    }
}
