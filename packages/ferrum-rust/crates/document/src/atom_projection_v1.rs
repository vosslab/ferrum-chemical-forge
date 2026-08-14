//! Immutable atom facts retained in document projections.

use serde::Serialize;

use super::{
    AtomMarkProjectionV1, DocumentObjectIdV1, FontFactsV1, Point3V1, ProjectionLocalObjectKeyV1,
    RichTextV1, TransparentOrRgb24V1, VisibilityV1,
};

/// Immutable atom facts in source order.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AtomProjectionV1 {
    pub(crate) id: Option<DocumentObjectIdV1>,
    pub(crate) projection_key: ProjectionLocalObjectKeyV1,
    pub(crate) source_id: Option<String>,
    pub(crate) source_order: u32,
    pub(crate) element: Option<String>,
    pub(crate) position: Point3V1,
    pub(crate) formal_charge: Option<i32>,
    pub(crate) isotope: Option<u16>,
    pub(crate) explicit_hydrogens: Option<u16>,
    pub(crate) valence: Option<u16>,
    pub(crate) multiplicity: Option<u16>,
    pub(crate) free_sites: Option<u16>,
    pub(crate) number: Option<u64>,
    pub(crate) show_number: Option<VisibilityV1>,
    pub(crate) marks: Vec<AtomMarkProjectionV1>,
    pub(crate) label_font: Option<FontFactsV1>,
    pub(crate) label_text: Option<RichTextV1>,
    pub(crate) show: Option<VisibilityV1>,
    pub(crate) hydrogens: Option<VisibilityV1>,
    pub(crate) background_color: Option<TransparentOrRgb24V1>,
}

impl AtomProjectionV1 {
    /// Return the stable object key.
    #[must_use]
    pub fn id(&self) -> Option<&DocumentObjectIdV1> {
        self.id.as_ref()
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
