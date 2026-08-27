//! Tagged, renderer-owned paint facts for the V3 render-plan grammar.

use ferrum_document_model::is_admitted_atom_symbol_v1;
use serde::{Deserialize, Serialize};

use crate::RenderError;

/// A lowercase six-digit RGB value with no toolkit-specific interpretation.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Rgb24(String);

impl Rgb24 {
    /// Construct a lowercase six-digit RGB value such as `"cc3366"`.
    pub fn new(value: impl Into<String>) -> Result<Self, RenderError> {
        let value = value.into();
        if value.len() != 6
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(RenderError::InvalidRequest(
                "RGB color must contain exactly six lowercase hexadecimal digits".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    /// Return canonical RGB text without a leading hash.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Rgb24 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Closed semantic document-content colors resolved by a display owner.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentContentPaintRoleV1 {
    /// Ordinary Ferrum-supplied molecule and presentation ink.
    DocumentForeground,
    /// Ferrum-supplied persistent atom-number annotation ink.
    AtomNumber,
}

/// Validated element symbol reserved for a future semantic element palette.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ElementSymbolV1(String);

impl ElementSymbolV1 {
    /// Construct a closed element-role symbol accepted by the native document model.
    pub fn new(value: impl Into<String>) -> Result<Self, RenderError> {
        let value = value.into();
        if !is_admitted_atom_symbol_v1(&value) {
            return Err(RenderError::InvalidRequest(
                "element role requires a validated element symbol".to_owned(),
            ));
        }
        Ok(Self(value))
    }

    /// Return the canonical element symbol.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ElementSymbolV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Deterministic export colors for semantic document-content paint roles.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentExportPaletteV1;

impl DocumentExportPaletteV1 {
    /// Resolve one semantic document-content role for headless artifact generation.
    #[must_use]
    pub fn resolve_theme(self, role: DocumentContentPaintRoleV1) -> Rgb24 {
        match role {
            DocumentContentPaintRoleV1::DocumentForeground => {
                Rgb24::new("000000").expect("built-in document foreground is valid RGB")
            }
            DocumentContentPaintRoleV1::AtomNumber => {
                Rgb24::new("0000c8").expect("built-in atom number is valid RGB")
            }
        }
    }

    /// Resolve one admitted element role for headless artifact generation.
    ///
    /// The current profile uses ordinary document ink for each admitted element.
    #[must_use]
    pub fn resolve_element(self, _: &ElementSymbolV1) -> Rgb24 {
        self.resolve_theme(DocumentContentPaintRoleV1::DocumentForeground)
    }
}

/// Closed V3 paint grammar shared by all renderer operations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RenderPaintV3 {
    /// Exact durable document color supplied by an author or document standard.
    AuthoredRgb24 { rgb: Rgb24 },
    /// Ferrum-supplied semantic ink resolved by the Rust export palette.
    ThemeRole { role: DocumentContentPaintRoleV1 },
    /// Native semantic element color resolved by the Rust export palette.
    ElementRole { element: ElementSymbolV1 },
}

impl RenderPaintV3 {
    /// Construct an exact authored document color.
    #[must_use]
    pub const fn authored_rgb24(rgb: Rgb24) -> Self {
        Self::AuthoredRgb24 { rgb }
    }

    /// Construct the built-in ordinary document-ink role.
    #[must_use]
    pub fn document_foreground() -> Self {
        Self::ThemeRole {
            role: DocumentContentPaintRoleV1::DocumentForeground,
        }
    }

    /// Construct the built-in atom-number role.
    #[must_use]
    pub fn atom_number() -> Self {
        Self::ThemeRole {
            role: DocumentContentPaintRoleV1::AtomNumber,
        }
    }

    /// Return the sole exact RGB source for SVG, PDF, PNG, CLI, and other headless output.
    #[must_use]
    pub fn export_rgb(&self) -> Rgb24 {
        match self {
            Self::AuthoredRgb24 { rgb } => rgb.clone(),
            Self::ThemeRole { role } => DocumentExportPaletteV1.resolve_theme(*role),
            Self::ElementRole { element } => DocumentExportPaletteV1.resolve_element(element),
        }
    }

    /// Return a semantic document role when this is theme-resolved content.
    #[must_use]
    pub const fn role(&self) -> Option<DocumentContentPaintRoleV1> {
        match self {
            Self::ThemeRole { role } => Some(*role),
            Self::AuthoredRgb24 { .. } | Self::ElementRole { .. } => None,
        }
    }

    /// Return an element role symbol when this is future element-colored content.
    #[must_use]
    pub fn element(&self) -> Option<&ElementSymbolV1> {
        match self {
            Self::ElementRole { element } => Some(element),
            Self::AuthoredRgb24 { .. } | Self::ThemeRole { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_palette_keeps_semantic_and_authored_paint_distinct() {
        assert_eq!(
            RenderPaintV3::document_foreground().export_rgb().as_str(),
            "000000"
        );
        assert_eq!(RenderPaintV3::atom_number().export_rgb().as_str(), "0000c8");
        let authored = RenderPaintV3::authored_rgb24(Rgb24::new("123456").expect("RGB"));
        assert_eq!(authored.export_rgb().as_str(), "123456");
        assert_eq!(authored.role(), None);
    }

    #[test]
    fn tagged_wire_carries_identity_and_refuses_forged_semantic_rgb() {
        let theme = RenderPaintV3::document_foreground();
        assert_eq!(
            serde_json::to_string(&theme).expect("theme paint serializes"),
            r#"{"kind":"theme_role","role":"document_foreground"}"#,
        );
        let element = RenderPaintV3::ElementRole {
            element: ElementSymbolV1::new("O").expect("admitted element"),
        };
        assert_eq!(
            serde_json::to_string(&element).expect("element paint serializes"),
            r#"{"kind":"element_role","element":"O"}"#,
        );
        assert!(
            serde_json::from_str::<RenderPaintV3>(
                r#"{\"kind\":\"theme_role\",\"role\":\"unknown\",\"export_rgb\":\"000000\"}"#,
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<RenderPaintV3>(
                r#"{\"kind\":\"element_role\",\"element\":\"O\",\"export_rgb\":\"ffffff\"}"#,
            )
            .is_err()
        );
        assert!(serde_json::from_str::<RenderPaintV3>(
            r#"{\"kind\":\"authored_rgb24\",\"rgb\":\"000000\",\"role\":\"document_foreground\"}"#,
        )
        .is_err());
    }
}
