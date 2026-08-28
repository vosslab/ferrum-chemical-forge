//! Rust-owned File/Open route catalog for native and interchange documents.

use std::collections::BTreeSet;

use crate::{InterchangeFormatDescriptorV1, InterchangeFormatRegistryV1, InterchangeOperationV1};

const CDML_SUFFIXES_V2: [&str; 1] = [".cdml"];
const CDSVG_SUFFIXES_V2: [&str; 1] = [".svg"];

/// File/Open placement policy owned by the catalog, rather than desktop code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDocumentOpenDispositionV2 {
    ReplacePristineOrNewTab,
    NewDocumentOnly,
}

/// Closed decoding route selected by a catalog-issued handle.
#[derive(Clone, Copy, Debug)]
pub enum LocalDocumentOpenRouteV2 {
    Cdml,
    DecodedCdsvg,
    Interchange(&'static InterchangeFormatDescriptorV1),
}

impl LocalDocumentOpenRouteV2 {
    #[cfg(feature = "python-binding")]
    #[must_use]
    pub(crate) const fn source_kind(self) -> &'static str {
        match self {
            Self::Cdml => "cdml",
            Self::DecodedCdsvg => "decoded_cdsvg",
            Self::Interchange(descriptor) => match descriptor.decoder() {
                crate::InterchangeDecoderKeyV1::CmlSimpleMolecule => "cml",
                crate::InterchangeDecoderKeyV1::CdxmlSimpleMolecule => "cdxml",
                crate::InterchangeDecoderKeyV1::Sdf => "interchange",
            },
        }
    }
}

/// Immutable transport facts for one supported File/Open route.
#[derive(Clone, Copy, Debug)]
pub struct LocalDocumentOpenDescriptorV2 {
    display_name: &'static str,
    suffixes: &'static [&'static str],
    disposition: LocalDocumentOpenDispositionV2,
    route: LocalDocumentOpenRouteV2,
}

impl LocalDocumentOpenDescriptorV2 {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        self.display_name
    }

    #[must_use]
    pub const fn suffixes(self) -> &'static [&'static str] {
        self.suffixes
    }

    #[must_use]
    pub const fn allows_current_tab_replacement(self) -> bool {
        matches!(
            self.disposition,
            LocalDocumentOpenDispositionV2::ReplacePristineOrNewTab
        )
    }

    #[cfg(any(test, feature = "python-binding"))]
    #[must_use]
    pub(crate) const fn route(self) -> LocalDocumentOpenRouteV2 {
        self.route
    }

    #[cfg(feature = "python-binding")]
    #[must_use]
    pub(crate) fn has_same_route(self, other: Self) -> bool {
        match (self.route, other.route) {
            (LocalDocumentOpenRouteV2::Cdml, LocalDocumentOpenRouteV2::Cdml)
            | (LocalDocumentOpenRouteV2::DecodedCdsvg, LocalDocumentOpenRouteV2::DecodedCdsvg) => {
                true
            }
            (
                LocalDocumentOpenRouteV2::Interchange(left),
                LocalDocumentOpenRouteV2::Interchange(right),
            ) => std::ptr::eq(left, right),
            _ => false,
        }
    }
}

/// Catalog construction failure. A route conflict is never resolved by precedence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalDocumentOpenCatalogErrorV2 {
    DuplicateSuffix { suffix: &'static str },
    InvalidNativeDescriptor,
}

/// Sole discovery authority for desktop File/Open routes.
pub struct LocalDocumentOpenCatalogV2;

impl LocalDocumentOpenCatalogV2 {
    /// Build the deterministic native-plus-interchange File/Open snapshot.
    pub fn snapshot() -> Result<Vec<LocalDocumentOpenDescriptorV2>, LocalDocumentOpenCatalogErrorV2>
    {
        let native = [
            LocalDocumentOpenDescriptorV2 {
                display_name: "Ferrum CDML",
                suffixes: &CDML_SUFFIXES_V2,
                disposition: LocalDocumentOpenDispositionV2::ReplacePristineOrNewTab,
                route: LocalDocumentOpenRouteV2::Cdml,
            },
            LocalDocumentOpenDescriptorV2 {
                display_name: "SVG with embedded CDML",
                suffixes: &CDSVG_SUFFIXES_V2,
                disposition: LocalDocumentOpenDispositionV2::ReplacePristineOrNewTab,
                route: LocalDocumentOpenRouteV2::DecodedCdsvg,
            },
        ];
        Self::build(&native)
    }

    fn build(
        native: &[LocalDocumentOpenDescriptorV2],
    ) -> Result<Vec<LocalDocumentOpenDescriptorV2>, LocalDocumentOpenCatalogErrorV2> {
        let mut descriptors =
            Vec::with_capacity(native.len() + InterchangeFormatRegistryV1::descriptors().len());
        descriptors.extend_from_slice(native);
        descriptors.extend(
            InterchangeFormatRegistryV1::descriptors()
                .iter()
                .filter(|descriptor| {
                    descriptor.supports_operation(InterchangeOperationV1::DocumentImportNew)
                })
                .map(|descriptor| LocalDocumentOpenDescriptorV2 {
                    display_name: descriptor.display_name(),
                    suffixes: descriptor.input_suffixes(),
                    disposition: LocalDocumentOpenDispositionV2::NewDocumentOnly,
                    route: LocalDocumentOpenRouteV2::Interchange(descriptor),
                }),
        );
        Self::validate(&descriptors)?;
        Ok(descriptors)
    }

    fn validate(
        descriptors: &[LocalDocumentOpenDescriptorV2],
    ) -> Result<(), LocalDocumentOpenCatalogErrorV2> {
        let mut suffixes = BTreeSet::new();
        for descriptor in descriptors {
            let route_is_defined = matches!(
                descriptor.route,
                LocalDocumentOpenRouteV2::Cdml
                    | LocalDocumentOpenRouteV2::DecodedCdsvg
                    | LocalDocumentOpenRouteV2::Interchange(_)
            );
            if !route_is_defined {
                return Err(LocalDocumentOpenCatalogErrorV2::InvalidNativeDescriptor);
            }
            for suffix in descriptor.suffixes() {
                if !is_lowercase_dotted_ascii_suffix(suffix) {
                    return Err(LocalDocumentOpenCatalogErrorV2::InvalidNativeDescriptor);
                }
                if !suffixes.insert(*suffix) {
                    return Err(LocalDocumentOpenCatalogErrorV2::DuplicateSuffix { suffix });
                }
            }
        }
        Ok(())
    }
}

fn is_lowercase_dotted_ascii_suffix(value: &str) -> bool {
    value.len() > 1
        && value.starts_with('.')
        && value.as_bytes()[1..]
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_joins_native_and_document_import_descriptors_in_registry_order() {
        let snapshot = LocalDocumentOpenCatalogV2::snapshot().expect("valid catalog");
        assert_eq!(snapshot[0].suffixes(), &[".cdml"]);
        assert_eq!(snapshot[1].suffixes(), &[".svg"]);
        assert_eq!(snapshot[2].suffixes(), &[".cml"]);
        assert_eq!(snapshot[3].suffixes(), &[".cdxml"]);
        assert_eq!(snapshot[4].suffixes(), &[".sdf", ".sd"]);
        assert!(snapshot[0].allows_current_tab_replacement());
        assert!(snapshot[1].allows_current_tab_replacement());
        assert!(!snapshot[2].allows_current_tab_replacement());
        assert!(matches!(
            snapshot[4].route(),
            LocalDocumentOpenRouteV2::Interchange(descriptor)
                if std::ptr::eq(descriptor, &InterchangeFormatRegistryV1::descriptors()[2])
        ));
    }

    #[test]
    fn malformed_or_colliding_native_suffixes_fail_closed() {
        let malformed = [LocalDocumentOpenDescriptorV2 {
            display_name: "bad",
            suffixes: &[".CDML"],
            disposition: LocalDocumentOpenDispositionV2::NewDocumentOnly,
            route: LocalDocumentOpenRouteV2::Cdml,
        }];
        assert!(matches!(
            LocalDocumentOpenCatalogV2::build(&malformed),
            Err(LocalDocumentOpenCatalogErrorV2::InvalidNativeDescriptor)
        ));
        let collision = [LocalDocumentOpenDescriptorV2 {
            display_name: "collision",
            suffixes: &[".cml"],
            disposition: LocalDocumentOpenDispositionV2::NewDocumentOnly,
            route: LocalDocumentOpenRouteV2::Cdml,
        }];
        assert!(matches!(
            LocalDocumentOpenCatalogV2::build(&collision),
            Err(LocalDocumentOpenCatalogErrorV2::DuplicateSuffix { suffix: ".cml" })
        ));
    }
}
