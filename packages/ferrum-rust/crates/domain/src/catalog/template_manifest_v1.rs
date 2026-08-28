//! Immutable Ferrum-authored catalog summaries, never legacy template payloads.
use serde::Serialize;

use crate::haworth::StandaloneDGlucoseHaworthRecipeV1;

pub const CATALOG_MANIFEST_SCHEMA_V1: &str = "ferrum-template-catalog-v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogFamilyV1 {
    System,
    Biomolecule,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogKeyV1(&'static str);
impl CatalogKeyV1 {
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogCategoryV1 {
    key: &'static str,
    label: &'static str,
    order: u16,
}
impl CatalogCategoryV1 {
    pub const fn key(self) -> &'static str {
        self.key
    }
    pub const fn label(self) -> &'static str {
        self.label
    }
    pub const fn order(self) -> u16 {
        self.order
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogProvenanceV1 {
    source_kind: &'static str,
    source_id: &'static str,
    license_spdx: &'static str,
    reviewed_on: &'static str,
    chemistry_scope: &'static str,
}
impl CatalogProvenanceV1 {
    pub const fn source_kind(self) -> &'static str {
        self.source_kind
    }
    pub const fn source_id(self) -> &'static str {
        self.source_id
    }
    pub const fn license_spdx(self) -> &'static str {
        self.license_spdx
    }
    pub const fn reviewed_on(self) -> &'static str {
        self.reviewed_on
    }
    pub const fn chemistry_scope(self) -> &'static str {
        self.chemistry_scope
    }
}
/// Closed native compiler selector. It is never emitted over public transports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogRecipeKindV1 {
    Benzene,
    Cyclopropane,
    Cyclobutane,
    Cyclopentane,
    Cyclohexane,
    Thiophene,
    Furan,
    Pyrrole,
    Purine,
    HaworthBiomolecule(StandaloneDGlucoseHaworthRecipeV1),
}
impl CatalogRecipeKindV1 {
    /// Stable closed semantic descriptor for catalog identity protocols.
    #[must_use]
    pub const fn canonical_descriptor(self) -> &'static str {
        match self {
            Self::Benzene => "ring:benzene;vertices=6;aromatic=true",
            Self::Cyclopropane => "ring:cyclopropane;vertices=3",
            Self::Cyclobutane => "ring:cyclobutane;vertices=4",
            Self::Cyclopentane => "ring:cyclopentane;vertices=5",
            Self::Cyclohexane => "ring:cyclohexane;vertices=6",
            Self::Thiophene => "heterocycle:thiophene;vertices=5;hetero=sulfur;aromatic=true",
            Self::Furan => "heterocycle:furan;vertices=5;hetero=oxygen;aromatic=true",
            Self::Pyrrole => "heterocycle:pyrrole;vertices=5;hetero=nitrogen;aromatic=true",
            Self::Purine => "heterocycle:purine;fused=imidazole+pyrimidine;aromatic=true",
            Self::HaworthBiomolecule(StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose) => {
                "haworth:d-glucose;configuration=d;anomer=alpha;ring=pyranose;detached=true;translation_only=true"
            }
            Self::HaworthBiomolecule(StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose) => {
                "haworth:d-glucose;configuration=d;anomer=beta;ring=pyranose;detached=true;translation_only=true"
            }
            Self::HaworthBiomolecule(StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucofuranose) => {
                "haworth:d-glucose;configuration=d;anomer=alpha;ring=furanose;detached=true;translation_only=true"
            }
            Self::HaworthBiomolecule(StandaloneDGlucoseHaworthRecipeV1::BetaDGlucofuranose) => {
                "haworth:d-glucose;configuration=d;anomer=beta;ring=furanose;detached=true;translation_only=true"
            }
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogEntrySummaryV1 {
    key: CatalogKeyV1,
    family: CatalogFamilyV1,
    category: CatalogCategoryV1,
    label: &'static str,
    terms: &'static [&'static str],
    provenance: CatalogProvenanceV1,
    #[serde(skip)]
    recipe: CatalogRecipeKindV1,
}
impl CatalogEntrySummaryV1 {
    pub const fn key(self) -> CatalogKeyV1 {
        self.key
    }
    pub const fn family(self) -> CatalogFamilyV1 {
        self.family
    }
    pub const fn category(self) -> CatalogCategoryV1 {
        self.category
    }
    pub const fn label(self) -> &'static str {
        self.label
    }
    /// Return the closed, Ferrum-authored search vocabulary for this entry.
    #[must_use]
    pub const fn terms(self) -> &'static [&'static str] {
        self.terms
    }
    pub const fn provenance(self) -> CatalogProvenanceV1 {
        self.provenance
    }
    pub const fn recipe(self) -> CatalogRecipeKindV1 {
        self.recipe
    }
    fn matches(self, needle: &str) -> bool {
        [
            self.label,
            self.category.label,
            self.category.key,
            self.key.as_str(),
        ]
        .iter()
        .chain(self.terms.iter())
        .any(|value| value.to_ascii_lowercase().contains(needle))
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct CatalogManifestV1 {
    schema: &'static str,
    catalog_version: &'static str,
    entries: &'static [CatalogEntrySummaryV1],
}
impl CatalogManifestV1 {
    pub const fn schema(self) -> &'static str {
        self.schema
    }
    pub const fn catalog_version(self) -> &'static str {
        self.catalog_version
    }
    pub const fn entries(self) -> &'static [CatalogEntrySummaryV1] {
        self.entries
    }
}

const RINGS: CatalogCategoryV1 = CatalogCategoryV1 {
    key: "rings",
    label: "Rings",
    order: 10,
};
const HETEROCYCLES: CatalogCategoryV1 = CatalogCategoryV1 {
    key: "heterocycles",
    label: "Heterocycles",
    order: 20,
};
const D_GLUCOSE: CatalogCategoryV1 = CatalogCategoryV1 {
    key: "carbohydrates_d_glucose",
    label: "Carbohydrates / D-glucose",
    order: 100,
};
// Semantic/topology and geometry source: independently authored canonical regular
// polygons plus the explicitly documented 40-point fused purine graph below.
const FERRUM_AUTHORED: CatalogProvenanceV1 = CatalogProvenanceV1 {
    source_kind: "curated_ferrum",
    source_id: "ferrum-authored-regular-ring-recipes-v1",
    license_spdx: "LGPL-3.0-only",
    reviewed_on: "2026-08-19",
    chemistry_scope: "Independently authored standard ring topology and Ferrum geometry; no historical template coordinates or payloads.",
};
// These literal coordinate tables are Ferrum-authored depiction facts. They
// are not a sugar parser, import payload, or generic graph-layout recipe.
const FERRUM_AUTHORED_HAWORTH: CatalogProvenanceV1 = CatalogProvenanceV1 {
    source_kind: "curated_ferrum",
    source_id: "ferrum-authored-d-glucose-haworth-depictions-v1",
    license_spdx: "LGPL-3.0-only",
    reviewed_on: "2026-08-19",
    chemistry_scope: "Independently authored closed D-glucose Haworth depictions; alpha/beta five/six-member forms only; detached translation-only placement; no sugar inference or attachment.",
};
const fn entry(
    key: &'static str,
    category: CatalogCategoryV1,
    label: &'static str,
    terms: &'static [&'static str],
    recipe: CatalogRecipeKindV1,
) -> CatalogEntrySummaryV1 {
    CatalogEntrySummaryV1 {
        key: CatalogKeyV1(key),
        family: CatalogFamilyV1::System,
        category,
        label,
        terms,
        provenance: FERRUM_AUTHORED,
        recipe,
    }
}
const fn biomolecule_entry(
    key: &'static str,
    label: &'static str,
    terms: &'static [&'static str],
    recipe: StandaloneDGlucoseHaworthRecipeV1,
) -> CatalogEntrySummaryV1 {
    CatalogEntrySummaryV1 {
        key: CatalogKeyV1(key),
        family: CatalogFamilyV1::Biomolecule,
        category: D_GLUCOSE,
        label,
        terms,
        provenance: FERRUM_AUTHORED_HAWORTH,
        recipe: CatalogRecipeKindV1::HaworthBiomolecule(recipe),
    }
}
const ENTRIES: &[CatalogEntrySummaryV1] = &[
    entry(
        "system/rings/benzene",
        RINGS,
        "Benzene",
        &["aromatic", "c6h6"],
        CatalogRecipeKindV1::Benzene,
    ),
    entry(
        "system/rings/cyclopropane",
        RINGS,
        "Cyclopropane",
        &["three membered ring", "c3h6"],
        CatalogRecipeKindV1::Cyclopropane,
    ),
    entry(
        "system/rings/cyclobutane",
        RINGS,
        "Cyclobutane",
        &["four membered ring", "c4h8"],
        CatalogRecipeKindV1::Cyclobutane,
    ),
    entry(
        "system/rings/cyclopentane",
        RINGS,
        "Cyclopentane",
        &["five membered ring", "c5h10"],
        CatalogRecipeKindV1::Cyclopentane,
    ),
    entry(
        "system/rings/cyclohexane",
        RINGS,
        "Cyclohexane",
        &["six membered ring", "c6h12"],
        CatalogRecipeKindV1::Cyclohexane,
    ),
    entry(
        "system/heterocycles/thiophene",
        HETEROCYCLES,
        "Thiophene",
        &["sulfur", "aromatic", "c4h4s"],
        CatalogRecipeKindV1::Thiophene,
    ),
    entry(
        "system/heterocycles/furan",
        HETEROCYCLES,
        "Furan",
        &["oxygen", "aromatic", "c4h4o"],
        CatalogRecipeKindV1::Furan,
    ),
    entry(
        "system/heterocycles/pyrrole",
        HETEROCYCLES,
        "Pyrrole",
        &["nitrogen", "aromatic", "c4h5n"],
        CatalogRecipeKindV1::Pyrrole,
    ),
    entry(
        "system/heterocycles/purine",
        HETEROCYCLES,
        "Purine",
        &["fused", "nitrogen heterocycle", "c5h4n4"],
        CatalogRecipeKindV1::Purine,
    ),
    biomolecule_entry(
        "biomolecules/carbohydrates/d-glucose/alpha-d-glucopyranose",
        "alpha-D-glucopyranose",
        &[
            "d-glucose",
            "haworth",
            "pyranose",
            "alpha",
            "detached drawing",
        ],
        StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose,
    ),
    biomolecule_entry(
        "biomolecules/carbohydrates/d-glucose/beta-d-glucopyranose",
        "beta-D-glucopyranose",
        &[
            "d-glucose",
            "haworth",
            "pyranose",
            "beta",
            "detached drawing",
        ],
        StandaloneDGlucoseHaworthRecipeV1::BetaDGlucopyranose,
    ),
    biomolecule_entry(
        "biomolecules/carbohydrates/d-glucose/alpha-d-glucofuranose",
        "alpha-D-glucofuranose",
        &[
            "d-glucose",
            "haworth",
            "furanose",
            "alpha",
            "detached drawing",
        ],
        StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucofuranose,
    ),
    biomolecule_entry(
        "biomolecules/carbohydrates/d-glucose/beta-d-glucofuranose",
        "beta-D-glucofuranose",
        &[
            "d-glucose",
            "haworth",
            "furanose",
            "beta",
            "detached drawing",
        ],
        StandaloneDGlucoseHaworthRecipeV1::BetaDGlucofuranose,
    ),
];
const MANIFEST: CatalogManifestV1 = CatalogManifestV1 {
    schema: CATALOG_MANIFEST_SCHEMA_V1,
    catalog_version: "2026.08.1",
    entries: ENTRIES,
};
pub const fn catalog_manifest_v1() -> CatalogManifestV1 {
    MANIFEST
}
pub fn catalog_entry_v1(key: &str) -> Option<CatalogEntrySummaryV1> {
    ENTRIES
        .iter()
        .copied()
        .find(|entry| entry.key.as_str() == key)
}
pub fn search_catalog_v1(
    family: Option<CatalogFamilyV1>,
    category: Option<&str>,
    query: Option<&str>,
) -> Vec<CatalogEntrySummaryV1> {
    let needle = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_ascii_lowercase);
    ENTRIES
        .iter()
        .copied()
        .filter(|entry| {
            family.is_none_or(|value| entry.family == value)
                && category.is_none_or(|value| entry.category.key == value)
                && needle.as_ref().is_none_or(|value| entry.matches(value))
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn manifest_is_ferrum_owned_and_payload_free() {
        assert_eq!(catalog_manifest_v1().entries().len(), 13);
        assert!(
            catalog_manifest_v1()
                .entries()
                .iter()
                .all(|entry| entry.provenance().source_kind() == "curated_ferrum"
                    && entry.provenance().license_spdx() == "LGPL-3.0-only")
        );
    }
    #[test]
    fn filtering_is_deterministic_and_uses_safe_summary_terms() {
        assert_eq!(
            search_catalog_v1(Some(CatalogFamilyV1::System), Some("rings"), None).len(),
            5
        );
        assert_eq!(
            search_catalog_v1(None, Some("heterocycles"), Some("sulfur"))[0]
                .key()
                .as_str(),
            "system/heterocycles/thiophene"
        );
        assert_eq!(
            search_catalog_v1(Some(CatalogFamilyV1::Biomolecule), None, None).len(),
            4
        );
    }
    #[test]
    fn recipe_descriptors_are_closed_and_debug_independent() {
        assert_eq!(
            CatalogRecipeKindV1::HaworthBiomolecule(
                StandaloneDGlucoseHaworthRecipeV1::AlphaDGlucopyranose
            )
            .canonical_descriptor(),
            "haworth:d-glucose;configuration=d;anomer=alpha;ring=pyranose;detached=true;translation_only=true"
        );
        assert_ne!(
            CatalogRecipeKindV1::Benzene.canonical_descriptor(),
            CatalogRecipeKindV1::Cyclohexane.canonical_descriptor()
        );
    }
}
