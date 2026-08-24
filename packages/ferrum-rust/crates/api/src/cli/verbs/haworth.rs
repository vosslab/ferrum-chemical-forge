//! `ferrum haworth` renders one bounded structural-SMILES Haworth SVG.

use std::io::{Read, Write};
use std::path::Path;

use ferrum_domain::haworth::build_direct_haworth_from_text_smiles_v1;
use ferrum_geometry::{MoleculePlacementV1, Point2};
use ferrum_render::{
    DirectGlycosidicHaworthRenderRequestV1, Paint, PositiveFinite, RenderProvenance,
    RenderRevision, Rgb24, SvgViewportV1, lower_direct_glycosidic_haworth_v1,
    render_direct_glycosidic_haworth_to_svg_v1,
};

use super::{VerbCliError, publish_or_write};

const INPUT_LIMIT: usize = 4_096;
const BOND_LENGTH: f64 = 40.0;

pub(crate) fn run(
    input: &str,
    output: Option<&Path>,
    stdin: &mut dyn Read,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), VerbCliError> {
    let smiles = if input == "-" {
        read_standard_input(stdin)?
    } else {
        input.to_owned()
    };
    let placement = MoleculePlacementV1::new(
        BOND_LENGTH,
        Point2::new(0.0, 0.0).expect("fixed Haworth anchor is finite"),
    )
    .expect("fixed Haworth placement is valid");
    let prepared = build_direct_haworth_from_text_smiles_v1(&smiles, placement)
        .map_err(|error| VerbCliError::HaworthInput(error.to_string()))?;
    let request = DirectGlycosidicHaworthRenderRequestV1::new(
        RenderProvenance::new(
            RenderRevision::new(0).expect("zero is a valid detached revision"),
            [0; 32],
        ),
        prepared.receipt().source_spec().clone(),
        Paint::rgb24(Rgb24::new("000000").expect("fixed paint is valid")),
        PositiveFinite::new(1.0).expect("fixed line width is valid"),
        PositiveFinite::new(5.0).expect("fixed wedge width is valid"),
    );
    let plan = lower_direct_glycosidic_haworth_v1(&request)
        .map_err(|error| VerbCliError::HaworthRender(error.to_string()))?;
    let svg = render_direct_glycosidic_haworth_to_svg_v1(
        &plan,
        SvgViewportV1::new(-160.0, -120.0, 320.0, 240.0).expect("fixed Haworth viewport is valid"),
    )
    .map_err(|error| VerbCliError::HaworthRender(error.to_string()))?;
    publish_or_write(output, svg.into_string().into_bytes(), None, stdout, stderr)
}

fn read_standard_input(stdin: &mut dyn Read) -> Result<String, VerbCliError> {
    let mut bytes = Vec::new();
    stdin
        .take(
            u64::try_from(INPUT_LIMIT)
                .expect("input bound fits u64")
                .saturating_add(1),
        )
        .read_to_end(&mut bytes)
        .map_err(|source| VerbCliError::Input {
            input: "standard input".to_owned(),
            source,
        })?;
    if bytes.len() > INPUT_LIMIT {
        return Err(VerbCliError::InputTooLarge {
            input: "standard input".to_owned(),
            limit: INPUT_LIMIT,
        });
    }
    String::from_utf8(bytes).map_err(|source| VerbCliError::InvalidUtf8 {
        input: "standard input".to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::run;

    const DIRECT_GLYCOSIDE: &str = "O1CCCCC1OC2CCCCO2";

    #[test]
    fn direct_smiles_renders_deterministic_svg_to_standard_output() {
        let mut first = Vec::new();
        let mut stderr = Vec::new();
        run(
            DIRECT_GLYCOSIDE,
            None,
            &mut std::io::empty(),
            &mut first,
            &mut stderr,
        )
        .expect("render succeeds");
        let mut second = Vec::new();
        run(
            DIRECT_GLYCOSIDE,
            None,
            &mut std::io::empty(),
            &mut second,
            &mut stderr,
        )
        .expect("second render succeeds");
        assert_eq!(first, second);
        assert!(first.starts_with(b"<svg"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn standard_input_is_accepted_without_an_engine_bundle() {
        let mut stdin = DIRECT_GLYCOSIDE.as_bytes();
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        run("-", None, &mut stdin, &mut stdout, &mut stderr).expect("stdin render succeeds");
        assert!(stdout.starts_with(b"<svg"));
        assert!(stderr.is_empty());
    }

    #[test]
    fn invalid_structural_smiles_is_a_typed_input_error() {
        let error = run(
            "C=O",
            None,
            &mut std::io::empty(),
            &mut Vec::new(),
            &mut Vec::new(),
        )
        .expect_err("unsupported syntax refuses");
        assert!(matches!(error, super::VerbCliError::HaworthInput(_)));
    }
}
