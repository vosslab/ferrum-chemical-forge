use std::collections::BTreeSet;
use std::sync::OnceLock;

use super::provenance::{records, valid_identifier};
use super::{CatalogError, CatalogProvenance, VerifiedCatalog};

const SOURCE: &str = include_str!("../../assets/catalogs/biomolecule_templates.tsv");
const PROVENANCE: CatalogProvenance = CatalogProvenance::new(
    "crates/domain/assets/catalogs/biomolecule_templates.tsv",
    "2026-08-12",
    "CC0-1.0",
    "Ferrum-authored representative seed data; not imported from OASA.",
    "3747a72c8313a0e017803e322e48cc1b69aa60dd36dfb5b6e3d1188397eba09d",
);

/// The domain category of a biomolecule reference template.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BiomoleculeKind {
    AminoAcid,
    Nucleotide,
    Cofactor,
}

impl BiomoleculeKind {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "amino_acid" => Some(Self::AminoAcid),
            "nucleotide" => Some(Self::Nucleotide),
            "cofactor" => Some(Self::Cofactor),
            _ => None,
        }
    }
}

/// A named reference template with no implicit structure or drawing semantics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomoleculeTemplate {
    identifier: String,
    display_name: String,
    kind: BiomoleculeKind,
    molecular_formula: String,
    description: String,
}

impl BiomoleculeTemplate {
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    pub const fn kind(&self) -> BiomoleculeKind {
        self.kind
    }

    pub fn molecular_formula(&self) -> &str {
        &self.molecular_formula
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

/// Namespace for the verified biomolecule-template reference catalog.
pub struct BiomoleculeTemplateCatalog;

impl BiomoleculeTemplateCatalog {
    pub fn load() -> Result<&'static VerifiedCatalog<BiomoleculeTemplate>, CatalogError> {
        static CATALOG: OnceLock<Result<VerifiedCatalog<BiomoleculeTemplate>, CatalogError>> =
            OnceLock::new();
        CATALOG.get_or_init(parse).as_ref().map_err(Clone::clone)
    }

    pub fn find(identifier: &str) -> Result<Option<&'static BiomoleculeTemplate>, CatalogError> {
        Ok(Self::load()?
            .entries()
            .iter()
            .find(|entry| entry.identifier == identifier))
    }
}

fn parse() -> Result<VerifiedCatalog<BiomoleculeTemplate>, CatalogError> {
    let mut identifiers = BTreeSet::new();
    let mut entries = Vec::new();
    for (line, record) in records(SOURCE) {
        let fields: Vec<_> = record.split('\t').collect();
        let [
            identifier,
            display_name,
            kind,
            molecular_formula,
            description,
        ] = fields.as_slice()
        else {
            return Err(CatalogError::MalformedRecord {
                catalog: "biomolecule-template",
                line,
            });
        };
        let Some(kind) = BiomoleculeKind::parse(kind) else {
            return Err(CatalogError::InvalidField {
                catalog: "biomolecule-template",
                line,
                field: "kind",
            });
        };
        if !valid_identifier(identifier)
            || display_name.is_empty()
            || molecular_formula.is_empty()
            || description.is_empty()
        {
            return Err(CatalogError::InvalidField {
                catalog: "biomolecule-template",
                line,
                field: "required value",
            });
        }
        if !identifiers.insert(*identifier) {
            return Err(CatalogError::DuplicateIdentifier {
                catalog: "biomolecule-template",
                identifier: (*identifier).to_owned(),
            });
        }
        entries.push(BiomoleculeTemplate {
            identifier: (*identifier).to_owned(),
            display_name: (*display_name).to_owned(),
            kind,
            molecular_formula: (*molecular_formula).to_owned(),
            description: (*description).to_owned(),
        });
    }
    VerifiedCatalog::verify(SOURCE, PROVENANCE, entries)
}

#[cfg(test)]
mod tests {
    use super::{BiomoleculeKind, BiomoleculeTemplateCatalog};

    #[test]
    fn catalog_preserves_explicit_kind_without_structure_claims() {
        let entry = BiomoleculeTemplateCatalog::find("heme_b")
            .expect("valid catalog")
            .expect("known identifier");
        assert_eq!(entry.kind(), BiomoleculeKind::Cofactor);
        assert_eq!(entry.molecular_formula(), "C34H32FeN4O4");
    }
}
