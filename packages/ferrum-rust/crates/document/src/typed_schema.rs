//! Closed typed-CDML schema facts used while retaining XML structure.

use super::TypedClass;

/// Return the typed attribute names recognized for a record class.
pub(crate) fn typed_attribute_names(class: TypedClass) -> &'static [&'static str] {
    use TypedClass as C;
    match class {
        C::Cdml => &["version", "type"],
        C::AuthorProgram => &["version"],
        C::MetadataDocument => &["href"],
        C::Standard => &[
            "line_width",
            "font_size",
            "font_family",
            "line_color",
            "area_color",
            "paper_type",
            "paper_orientation",
            "paper_crop_svg",
            "paper_crop_margin",
        ],
        C::StandardBond => &[
            "length",
            "width",
            "wedge-width",
            "double-ratio",
            "min_wedge_angle",
        ],
        C::StandardArrow => &["length"],
        C::StandardAtom => &["show_hydrogens"],
        C::Paper => &[
            "id",
            "type",
            "orientation",
            "crop_svg",
            "crop_margin",
            "use_real_minus",
            "replace_minus",
            "size_x",
            "size_y",
        ],
        C::Viewport => &["viewport", "id"],
        C::Molecule => &["id", "name"],
        C::CanvasArrow => &[
            "id", "type", "start", "end", "width", "spline", "shape", "color",
        ],
        C::CanvasPlus => &["id", "font_size", "color", "background-color"],
        C::CanvasText => &["id", "background-color"],
        C::Rectangle | C::Square | C::Oval | C::Circle => &[
            "id",
            "x1",
            "y1",
            "x2",
            "y2",
            "area_color",
            "background-color",
            "line_color",
            "color",
            "width",
        ],
        C::Polygon => &[
            "id",
            "area_color",
            "background-color",
            "line_color",
            "color",
            "width",
        ],
        C::Polyline => &[
            "id",
            "line_color",
            "color",
            "width",
            "spline",
            "style",
            "bracket_pair",
            "bracket_side",
        ],
        C::Reaction => &["id"],
        C::ReactionReactant
        | C::ReactionProduct
        | C::ReactionArrow
        | C::ReactionCondition
        | C::ReactionPlus => &["idref"],
        C::Atom => &[
            "id",
            "name",
            "charge",
            "pos",
            "show",
            "hydrogens",
            "show_number",
            "number",
            "background-color",
            "multiplicity",
            "valency",
            "free_sites",
            "isotope",
            "explicit_hydrogens",
        ],
        C::Group => &[
            "id",
            "name",
            "group-type",
            "pos",
            "background-color",
            "show_number",
            "number",
        ],
        C::MoleculeText => &["id", "pos", "background-color", "show_number", "number"],
        C::Query => &[
            "id",
            "name",
            "pos",
            "background-color",
            "show_number",
            "number",
            "free_sites",
        ],
        C::Bond => &[
            "id",
            "type",
            "start",
            "end",
            "line_width",
            "bond_width",
            "wedge_width",
            "double_ratio",
            "center",
            "auto_sign",
            "equithick",
            "simple_double",
            "color",
            "wavy_style",
            "haworth_position",
        ],
        C::Template => &["atom", "bond_first", "bond_second"],
        C::Fragment => &["id", "type"],
        C::FragmentBond | C::FragmentVertex => &["id"],
        C::FragmentProperty => &["name", "value", "type"],
        C::Point => &["x", "y", "z"],
        C::Font => &["size", "family", "color"],
        C::Mark => &[
            "type",
            "x",
            "y",
            "auto",
            "size",
            "line_width",
            "draw_circle",
            "text",
            "refname",
        ],
        C::Info
        | C::Author
        | C::Note
        | C::Metadata
        | C::ExternalData
        | C::DisplayForm
        | C::UserData
        | C::FragmentName
        | C::FormattedText => &[],
    }
}
