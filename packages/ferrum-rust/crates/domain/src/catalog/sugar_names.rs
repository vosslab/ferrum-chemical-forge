use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::provenance::{records, valid_identifier};
use super::{CatalogError, CatalogProvenance, VerifiedCatalog};

const SOURCE: &str = include_str!("../../assets/catalogs/sugar_names.tsv");
const PROVENANCE: CatalogProvenance = CatalogProvenance::new(
    "crates/domain/assets/catalogs/sugar_names.tsv",
    "2026-08-12",
    "CC0-1.0",
    "Ferrum-authored representative seed data; not imported from OASA.",
    "1b8923b17e9be29060f87ac08ba184cf5cf5b9fa1e64d84d472fec3aa2cbd374",
);

/// A conventional sugar name with explicit family and carbon count.
///
/// This is nomenclature metadata only; it does not imply a ring form,
/// anomer, stereochemical assignment, or molecular graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SugarName {
    identifier: String,
    display_name: String,
    family: String,
    carbon_count: u8,
    description: String,
}

impl SugarName {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub const fn carbon_count(&self) -> u8 {
        self.carbon_count
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Namespace for the verified sugar-name reference catalog.
pub struct SugarNameCatalog;

impl SugarNameCatalog {
    pub fn load() -> Result<&'static VerifiedCatalog<SugarName>, CatalogError> {
        static CATALOG: OnceLock<Result<VerifiedCatalog<SugarName>, CatalogError>> =
            OnceLock::new();
        CATALOG.get_or_init(parse).as_ref().map_err(Clone::clone)
    }

    pub fn find(identifier: &str) -> Result<Option<&'static SugarName>, CatalogError> {
        Ok(Self::load()?
            .entries()
            .iter()
            .find(|entry| entry.identifier == identifier))
    }
}

fn parse() -> Result<VerifiedCatalog<SugarName>, CatalogError> {
    let mut identifiers = BTreeSet::new();
    let mut entries = Vec::new();
    for (line, record) in records(SOURCE) {
        let fields: Vec<_> = record.split('\t').collect();
        let [identifier, display_name, family, carbon_count, description] = fields.as_slice()
        else {
            return Err(CatalogError::MalformedRecord {
                catalog: "sugar-name",
                line,
            });
        };
        let Ok(carbon_count) = carbon_count.parse::<u8>() else {
            return Err(CatalogError::InvalidField {
                catalog: "sugar-name",
                line,
                field: "carbon count",
            });
        };
        if !valid_identifier(identifier)
            || display_name.is_empty()
            || family.is_empty()
            || !(3..=12).contains(&carbon_count)
            || description.is_empty()
        {
            return Err(CatalogError::InvalidField {
                catalog: "sugar-name",
                line,
                field: "required value",
            });
        }
        if !identifiers.insert(*identifier) {
            return Err(CatalogError::DuplicateIdentifier {
                catalog: "sugar-name",
                identifier: (*identifier).to_owned(),
            });
        }
        entries.push(SugarName {
            identifier: (*identifier).to_owned(),
            display_name: (*display_name).to_owned(),
            family: (*family).to_owned(),
            carbon_count,
            description: (*description).to_owned(),
        });
    }
    VerifiedCatalog::verify(SOURCE, PROVENANCE, entries)
}

#[cfg(test)]
mod tests {
    use super::SugarNameCatalog;

    #[test]
    fn sugar_catalog_keeps_name_metadata_separate_from_structure() {
        let entry = SugarNameCatalog::find("d_ribose")
            .expect("valid catalog")
            .expect("known identifier");
        assert_eq!(entry.family(), "aldopentose");
        assert_eq!(entry.carbon_count(), 5);
    }
}
