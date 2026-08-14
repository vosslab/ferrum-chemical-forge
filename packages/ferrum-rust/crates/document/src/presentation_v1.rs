//! Immutable presentation facts extracted from typed CDML without toolkit defaults.

use serde::Serialize;

/// A validated positive finite scalar used in presentation facts.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PositiveFiniteV1(f64);

impl PositiveFiniteV1 {
    /// Construct a positive finite value.
    pub fn new(value: f64) -> Option<Self> {
        (value.is_finite() && value > 0.0).then_some(Self(value))
    }

    /// Return the carried value.
    #[must_use]
    pub fn value(self) -> f64 {
        self.0
    }
}

/// An exact opaque RGB colour validated from CDML `#rgb` or `#rrggbb` spelling.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Rgb24V1(String);

impl Rgb24V1 {
    /// Validate and retain a canonical `#rrggbb` colour spelling.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let digits = value.strip_prefix('#')?;
        match digits.len() {
            3 if digits.as_bytes().iter().all(u8::is_ascii_hexdigit) => {
                let mut canonical = String::with_capacity(7);
                canonical.push('#');
                for digit in digits.bytes() {
                    canonical.push(char::from(digit).to_ascii_lowercase());
                    canonical.push(char::from(digit).to_ascii_lowercase());
                }
                Some(Self(canonical))
            }
            6 if digits.as_bytes().iter().all(u8::is_ascii_hexdigit) => {
                Some(Self(value.to_ascii_lowercase()))
            }
            _ => None,
        }
    }

    /// Return the canonical colour spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A valid presentation colour or an intentional transparent mask.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum TransparentOrRgb24V1 {
    /// No label mask is emitted.
    Transparent,
    /// An authored opaque colour.
    Rgb24(Rgb24V1),
}

impl TransparentOrRgb24V1 {
    /// Parse established CDML transparency spellings or a validated RGB value.
    pub fn new(value: &str) -> Option<Self> {
        if value.is_empty() || value == "none" {
            Some(Self::Transparent)
        } else {
            Rgb24V1::new(value).map(Self::Rgb24)
        }
    }
}

/// The explicit unit used by a positive presentation length.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum PresentationLengthUnitV1 {
    /// Bare CDML presentation lengths use Ferrum scene points.
    Point,
    /// Existing CDML `px` lengths use Ferrum scene points in V1.
    Pixel,
}

/// A parsed positive presentation length with an explicit source unit.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PresentationLengthV1 {
    value: PositiveFiniteV1,
    unit: PresentationLengthUnitV1,
}

impl PresentationLengthV1 {
    /// Parse a V1 width. Only bare values and `px` are accepted as scene points.
    pub fn parse(value: &str) -> Option<Self> {
        let (number, unit) = match value.strip_suffix("px") {
            Some(number) => (number, PresentationLengthUnitV1::Pixel),
            None => (value, PresentationLengthUnitV1::Point),
        };
        PositiveFiniteV1::new(number.parse().ok()?).map(|value| Self { value, unit })
    }

    /// Return the normalized scene-point value.
    #[must_use]
    pub fn value(self) -> PositiveFiniteV1 {
        self.value
    }

    /// Return the explicit source unit.
    #[must_use]
    pub fn unit(self) -> PresentationLengthUnitV1 {
        self.unit
    }
}

/// A typed CDML boolean used for atom visibility and hydrogen display.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum VisibilityV1 {
    /// The source explicitly enables the feature.
    Enabled,
    /// The source explicitly disables the feature.
    Disabled,
}

impl VisibilityV1 {
    /// Parse the closed case-insensitive V1 boolean spelling set.
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(Self::Enabled),
            "0" | "false" | "no" | "off" => Some(Self::Disabled),
            _ => None,
        }
    }
}

/// Persisted font facts; absence is intentionally represented by `None` fields.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FontFactsV1 {
    family: Option<String>,
    size: Option<PositiveFiniteV1>,
    color: Option<Rgb24V1>,
}

impl FontFactsV1 {
    /// Construct extracted font facts without choosing missing values.
    #[must_use]
    pub fn new(
        family: Option<String>,
        size: Option<PositiveFiniteV1>,
        color: Option<Rgb24V1>,
    ) -> Self {
        Self {
            family,
            size,
            color,
        }
    }

    /// Return the authored font family.
    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    /// Return the authored font size.
    #[must_use]
    pub fn size(&self) -> Option<PositiveFiniteV1> {
        self.size
    }

    /// Return the authored font colour.
    #[must_use]
    pub fn color(&self) -> Option<&Rgb24V1> {
        self.color.as_ref()
    }
}

/// Persisted rich text with no interpretation by the projection layer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RichTextV1(String);

impl RichTextV1 {
    /// Retain authored text exactly as typed CDML supplied it.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Return the retained text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Persisted molecule-wide drawing facts from the document standard.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DrawingStandardV1 {
    line_width: Option<PositiveFiniteV1>,
    bond_width: Option<PositiveFiniteV1>,
    wedge_width: Option<PositiveFiniteV1>,
    font_size: Option<PositiveFiniteV1>,
    font_family: Option<String>,
    line_color: Option<Rgb24V1>,
    show_hydrogens: Option<VisibilityV1>,
    area_color: Option<TransparentOrRgb24V1>,
}

impl DrawingStandardV1 {
    /// Construct standard facts without synthesizing absent fields.
    #[expect(
        clippy::too_many_arguments,
        reason = "each retained standard fact remains explicit at the document projection boundary"
    )]
    #[must_use]
    pub fn new(
        line_width: Option<PositiveFiniteV1>,
        bond_width: Option<PositiveFiniteV1>,
        wedge_width: Option<PositiveFiniteV1>,
        font_size: Option<PositiveFiniteV1>,
        font_family: Option<String>,
        line_color: Option<Rgb24V1>,
        show_hydrogens: Option<VisibilityV1>,
        area_color: Option<TransparentOrRgb24V1>,
    ) -> Self {
        Self {
            line_width,
            bond_width,
            wedge_width,
            font_size,
            font_family,
            line_color,
            show_hydrogens,
            area_color,
        }
    }

    /// Return the persisted standard line width.
    #[must_use]
    pub fn line_width(&self) -> Option<PositiveFiniteV1> {
        self.line_width
    }
    /// Return the persisted standard spacing between parallel bond lanes.
    #[must_use]
    pub fn bond_width(&self) -> Option<PositiveFiniteV1> {
        self.bond_width
    }
    /// Return the persisted standard width for wedge bonds.
    #[must_use]
    pub fn wedge_width(&self) -> Option<PositiveFiniteV1> {
        self.wedge_width
    }
    /// Return the persisted standard font size.
    #[must_use]
    pub fn font_size(&self) -> Option<PositiveFiniteV1> {
        self.font_size
    }
    /// Return the persisted standard font family.
    #[must_use]
    pub fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }
    /// Return the persisted standard line colour.
    #[must_use]
    pub fn line_color(&self) -> Option<&Rgb24V1> {
        self.line_color.as_ref()
    }
    /// Return the persisted standard hydrogen display fact.
    #[must_use]
    pub fn show_hydrogens(&self) -> Option<VisibilityV1> {
        self.show_hydrogens
    }
    /// Return the persisted label-mask colour or transparent directive.
    #[must_use]
    pub fn area_color(&self) -> Option<&TransparentOrRgb24V1> {
        self.area_color.as_ref()
    }
}
