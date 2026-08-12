use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::provenance::{records, valid_identifier};
use super::{CatalogError, CatalogProvenance, VerifiedCatalog};

const SOURCE: &str = include_str!("../../assets/catalogs/functional_groups.tsv");
const PROVENANCE: CatalogProvenance = CatalogProvenance::new(
    "crates/domain/assets/catalogs/functional_groups.tsv",
    "2026-08-12",
    "CC0-1.0",
    "Ferrum-authored representative seed data; not imported from OASA.",
    "f968c073d821f0a7f42db0cf2a25f69b8c7d6b4afd1b557b54bf75e22b3f5672",
);

/// A named functional-group lookup entry, not a molecular graph template.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionalGroup {
    identifier: String,
    display_name: String,
    structural_summary: String,
    description: String,
}

impl FunctionalGroup {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Human-readable structural metadata only.
    ///
    /// This is deliberately not a molecular query, a graph template, or a
    /// source-language expression. A future typed query profile owns parsing
    /// and matching semantics.
    pub fn structural_summary(&self) -> &str {
        &self.structural_summary
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Namespace for the verified functional-group reference catalog.
pub struct FunctionalGroupCatalog;

impl FunctionalGroupCatalog {
    pub fn load() -> Result<&'static VerifiedCatalog<FunctionalGroup>, CatalogError> {
        static CATALOG: OnceLock<Result<VerifiedCatalog<FunctionalGroup>, CatalogError>> =
            OnceLock::new();
        CATALOG.get_or_init(parse).as_ref().map_err(Clone::clone)
    }

    pub fn find(identifier: &str) -> Result<Option<&'static FunctionalGroup>, CatalogError> {
        Ok(Self::load()?
            .entries()
            .iter()
            .find(|entry| entry.identifier == identifier))
    }
}

fn parse() -> Result<VerifiedCatalog<FunctionalGroup>, CatalogError> {
    let mut identifiers = BTreeSet::new();
    let mut entries = Vec::new();
    for (line, record) in records(SOURCE) {
        let fields: Vec<_> = record.split('\t').collect();
        let [identifier, display_name, structural_summary, description] = fields.as_slice() else {
            return Err(CatalogError::MalformedRecord {
                catalog: "functional-group",
                line,
            });
        };
        if !valid_identifier(identifier) {
            return Err(CatalogError::InvalidField {
                catalog: "functional-group",
                line,
                field: "identifier",
            });
        }
        if display_name.is_empty() || structural_summary.is_empty() || description.is_empty() {
            return Err(CatalogError::InvalidField {
                catalog: "functional-group",
                line,
                field: "required value",
            });
        }
        if !identifiers.insert(*identifier) {
            return Err(CatalogError::DuplicateIdentifier {
                catalog: "functional-group",
                identifier: (*identifier).to_owned(),
            });
        }
        entries.push(FunctionalGroup {
            identifier: (*identifier).to_owned(),
            display_name: (*display_name).to_owned(),
            structural_summary: (*structural_summary).to_owned(),
            description: (*description).to_owned(),
        });
    }
    VerifiedCatalog::verify(SOURCE, PROVENANCE, entries)
}

#[cfg(test)]
mod tests {
    use super::FunctionalGroupCatalog;

    #[test]
    fn catalog_is_verified_and_lookup_is_stable() {
        let catalog = FunctionalGroupCatalog::load().expect("valid authored catalog");
        assert_eq!(catalog.entries().len(), 3);
        assert_eq!(
            FunctionalGroupCatalog::find("hydroxyl")
                .expect("valid catalog")
                .expect("known identifier")
                .structural_summary(),
            "oxygen bearing one hydrogen"
        );
    }
}
