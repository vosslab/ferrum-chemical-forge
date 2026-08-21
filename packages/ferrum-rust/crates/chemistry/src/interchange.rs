//! Closed non-CDML molecular interchange codecs.
//!
//! CDML is deliberately excluded: typed-document parsing and serialization are
//! composed by the delivery layer from these owned records.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ChemEngine, ChemistryError, InchiMode, MOLBLOCK_MAX_INPUT_BYTES, MolGraph, MolblockVersion,
    NATIVE_SMILES_MAX_INPUT_BYTES, SDF_MAX_INPUT_BYTES, SdfError, SmilesMolecule,
    validate_inchi_input, validate_molblock_input, validate_smiles_input,
};

/// The maximum text accepted or returned by one interchange codec operation.
pub const INTERCHANGE_MAX_TEXT_BYTES_V1: usize = SDF_MAX_INPUT_BYTES;

/// Exact closed profile identity reserved for the Rust-owned CML/CML2 importer.
///
/// This is a codec capability identifier only.  The decoder is introduced in a
/// later M2a phase; keeping the identity here lets API registry validation prove
/// that presentation code cannot invent a second CML profile table.
pub const CML_SIMPLE_MOLECULE_IMPORT_PROFILE_ID_V1: &str =
    "ferrum-cml-simple-molecule-import-profile-v1";

/// Closed syntax vocabulary. `Cdml` is retained as a wire value for callers
/// that dispatch CDML composition outside this crate.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
pub enum InterchangeFormatV1 {
    #[serde(rename = "smiles")]
    Smiles,
    #[serde(rename = "inchi_standard")]
    InchiStandard,
    #[serde(rename = "inchi_fixed_h")]
    InchiFixedHydrogen,
    #[serde(rename = "molblock_v2000")]
    MolblockV2000,
    #[serde(rename = "molblock_v3000")]
    MolblockV3000,
    #[serde(rename = "sdf_v2000")]
    SdfV2000,
    #[serde(rename = "sdf_v3000")]
    SdfV3000,
    #[serde(rename = "cdml")]
    Cdml,
}
impl InterchangeFormatV1 {
    #[must_use]
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::Smiles => "smiles",
            Self::InchiStandard => "inchi_standard",
            Self::InchiFixedHydrogen => "inchi_fixed_h",
            Self::MolblockV2000 => "molblock_v2000",
            Self::MolblockV3000 => "molblock_v3000",
            Self::SdfV2000 => "sdf_v2000",
            Self::SdfV3000 => "sdf_v3000",
            Self::Cdml => "cdml",
        }
    }
    fn is_single_record(self) -> bool {
        !matches!(self, Self::SdfV2000 | Self::SdfV3000 | Self::Cdml)
    }
    fn molblock_version(self) -> Option<MolblockVersion> {
        match self {
            Self::MolblockV2000 | Self::SdfV2000 => Some(MolblockVersion::V2000),
            Self::MolblockV3000 | Self::SdfV3000 => Some(MolblockVersion::V3000),
            _ => None,
        }
    }
}
/// One ordered text property retained without map normalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterchangePropertyV1 {
    name: String,
    value: String,
}
impl InterchangePropertyV1 {
    pub fn new(
        name: impl Into<String>,
        value: impl Into<String>,
    ) -> Result<Self, InterchangeCodecErrorV1> {
        let name = name.into();
        let value = value.into();
        if name.is_empty() || name.contains(['\0', '\r', '\n']) || value.contains('\0') {
            return Err(InterchangeCodecErrorV1::InvalidInterchangeProperty);
        }
        Ok(Self { name, value })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}
/// One fully owned molecular record, including optional display title and ordered metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct InterchangeRecordV1 {
    molecule: MolGraph,
    title: Option<String>,
    properties: Vec<InterchangePropertyV1>,
}
impl InterchangeRecordV1 {
    #[must_use]
    pub fn new(
        molecule: MolGraph,
        title: Option<String>,
        properties: Vec<InterchangePropertyV1>,
    ) -> Self {
        Self {
            molecule,
            title,
            properties,
        }
    }
    #[must_use]
    pub fn molecule(&self) -> &MolGraph {
        &self.molecule
    }
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    #[must_use]
    pub fn properties(&self) -> &[InterchangePropertyV1] {
        &self.properties
    }
}
/// Decode any native syntax into complete, owned chemistry records.
pub fn decode_non_cdml_interchange_v1(
    engine: &dyn ChemEngine,
    format: InterchangeFormatV1,
    text: &str,
) -> Result<Vec<InterchangeRecordV1>, InterchangeCodecErrorV1> {
    if format == InterchangeFormatV1::Cdml {
        return Err(InterchangeCodecErrorV1::CdmlRequiresDocumentComposition);
    }
    enforce_input_limit(format, text)?;
    let records = match format {
        InterchangeFormatV1::Smiles => {
            validate_smiles_input(text)?;
            vec![from_smiles(engine.smiles_to_molecule(text)?, None)]
        }
        InterchangeFormatV1::InchiStandard | InterchangeFormatV1::InchiFixedHydrogen => {
            validate_inchi_input(text)?;
            vec![from_smiles(engine.inchi_to_molecule(text)?, None)]
        }
        InterchangeFormatV1::MolblockV2000 | InterchangeFormatV1::MolblockV3000 => {
            validate_molblock_input(text)?;
            vec![from_smiles(
                engine.molblock_to_molecule(text)?,
                molblock_title(text),
            )]
        }
        InterchangeFormatV1::SdfV2000 | InterchangeFormatV1::SdfV3000 => {
            crate::interchange_sdf::decode_sdf_interchange_v1(engine, text)?
        }
        InterchangeFormatV1::Cdml => unreachable!("checked above"),
    };
    if records.is_empty() {
        return Err(InterchangeCodecErrorV1::NoMolecularRecords);
    }
    Ok(records)
}
/// Encode owned chemistry records into any native syntax.
pub fn encode_non_cdml_interchange_v1(
    engine: &dyn ChemEngine,
    format: InterchangeFormatV1,
    records: &[InterchangeRecordV1],
) -> Result<String, InterchangeCodecErrorV1> {
    if format == InterchangeFormatV1::Cdml {
        return Err(InterchangeCodecErrorV1::CdmlRequiresDocumentComposition);
    }
    if records.is_empty() {
        return Err(InterchangeCodecErrorV1::NoMolecularRecords);
    }
    if format.is_single_record() && records.len() != 1 {
        return Err(InterchangeCodecErrorV1::MultiRecordUnsupported {
            format,
            record_count: records.len(),
        });
    }
    let output = match format {
        InterchangeFormatV1::Smiles => engine.molecule_to_smiles(records[0].molecule())?,
        InterchangeFormatV1::InchiStandard => {
            engine.molecule_to_inchi(records[0].molecule(), InchiMode::Standard)?
        }
        InterchangeFormatV1::InchiFixedHydrogen => {
            engine.molecule_to_inchi(records[0].molecule(), InchiMode::FixedHydrogen)?
        }
        InterchangeFormatV1::MolblockV2000 | InterchangeFormatV1::MolblockV3000 => engine
            .molecule_to_molblock_with_title(
                records[0].molecule(),
                format.molblock_version().expect("molblock format"),
                records[0].title().unwrap_or_default(),
            )?,
        InterchangeFormatV1::SdfV2000 | InterchangeFormatV1::SdfV3000 => {
            crate::interchange_sdf::encode_sdf_interchange_v1(
                engine,
                format.molblock_version().expect("SDF format"),
                records,
            )?
        }
        InterchangeFormatV1::Cdml => unreachable!("checked above"),
    };
    enforce_output_limit(format, &output)?;
    Ok(output)
}
fn from_smiles(molecule: SmilesMolecule, title: Option<String>) -> InterchangeRecordV1 {
    InterchangeRecordV1::new(molecule.molecule().clone(), title, Vec::new())
}
fn molblock_title(text: &str) -> Option<String> {
    text.lines().next().map(str::to_owned)
}
fn enforce_input_limit(
    format: InterchangeFormatV1,
    text: &str,
) -> Result<(), InterchangeCodecErrorV1> {
    let limit = match format {
        InterchangeFormatV1::Smiles => NATIVE_SMILES_MAX_INPUT_BYTES,
        InterchangeFormatV1::InchiStandard | InterchangeFormatV1::InchiFixedHydrogen => {
            crate::INCHI_MAX_INPUT_BYTES
        }
        InterchangeFormatV1::MolblockV2000 | InterchangeFormatV1::MolblockV3000 => {
            MOLBLOCK_MAX_INPUT_BYTES
        }
        _ => SDF_MAX_INPUT_BYTES,
    };
    if text.len() > limit {
        return Err(InterchangeCodecErrorV1::InputTooLarge {
            format,
            limit,
            observed_at_least: text.len(),
        });
    }
    Ok(())
}
fn enforce_output_limit(
    format: InterchangeFormatV1,
    text: &str,
) -> Result<(), InterchangeCodecErrorV1> {
    if text.len() > INTERCHANGE_MAX_TEXT_BYTES_V1 {
        return Err(InterchangeCodecErrorV1::OutputTooLarge {
            format,
            limit: INTERCHANGE_MAX_TEXT_BYTES_V1,
            observed_at_least: text.len(),
        });
    }
    Ok(())
}
/// A refused native source or target codec operation.
#[derive(Debug, Error)]
pub enum InterchangeCodecErrorV1 {
    #[error("interchange property is empty or contains an unretainable control character")]
    InvalidInterchangeProperty,
    #[error("{format:?} input exceeds {limit} bytes (observed {observed_at_least})")]
    InputTooLarge {
        format: InterchangeFormatV1,
        limit: usize,
        observed_at_least: usize,
    },
    #[error("{format:?} output exceeds {limit} bytes (observed {observed_at_least})")]
    OutputTooLarge {
        format: InterchangeFormatV1,
        limit: usize,
        observed_at_least: usize,
    },
    #[error("{format:?} cannot represent {record_count} records")]
    MultiRecordUnsupported {
        format: InterchangeFormatV1,
        record_count: usize,
    },
    #[error("interchange input contains no molecular records")]
    NoMolecularRecords,
    #[error("CDML interchange requires document composition")]
    CdmlRequiresDocumentComposition,
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error(transparent)]
    SdfRecord(#[from] SdfError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AtomicNumber, BondOrder, Coordinates, KekulizeOptions, MolAtom, MolBond, Point2};

    struct SdfEngine;

    impl ChemEngine for SdfEngine {
        fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles_to_molecule",
            })
        }

        fn generate_2d_coordinates(
            &self,
            molecule: &MolGraph,
        ) -> Result<Coordinates, ChemistryError> {
            Ok(molecule.coordinates().expect("test graph").clone())
        }

        fn molecule_to_molblock_with_title(
            &self,
            _molecule: &MolGraph,
            _version: MolblockVersion,
            title: &str,
        ) -> Result<String, ChemistryError> {
            Ok(format!("{title}\n  Ferrum\n\nM  END\n"))
        }

        fn kekulize(
            &self,
            molecule: &MolGraph,
            _options: KekulizeOptions,
        ) -> Result<MolGraph, ChemistryError> {
            Ok(molecule.clone())
        }
    }

    fn graph() -> MolGraph {
        MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::try_from(6).expect("carbon"),
                    None,
                    None,
                    None,
                    false,
                )
                .expect("atom"),
                MolAtom::new(
                    AtomicNumber::try_from(8).expect("oxygen"),
                    None,
                    None,
                    None,
                    false,
                )
                .expect("atom"),
            ],
            vec![MolBond::new(0, 1, BondOrder::Single, false)],
            Some(Coordinates::new(vec![
                Point2::new(0.0, 0.0).expect("point"),
                Point2::new(1.0, 0.0).expect("point"),
            ])),
        )
        .expect("graph")
    }

    #[test]
    fn sdf_encoding_preserves_duplicate_property_names_and_order() {
        let properties = vec![
            InterchangePropertyV1::new("SOURCE", "first").expect("property"),
            InterchangePropertyV1::new("SOURCE", "second").expect("property"),
            InterchangePropertyV1::new("NOTE", "third").expect("property"),
        ];
        let records = vec![InterchangeRecordV1::new(
            graph(),
            Some("record".to_owned()),
            properties,
        )];
        let output =
            encode_non_cdml_interchange_v1(&SdfEngine, InterchangeFormatV1::SdfV2000, &records)
                .expect("SDF output");
        let first = output.find(">  <SOURCE>\nfirst").expect("first property");
        let second = output.find(">  <SOURCE>\nsecond").expect("second property");
        let third = output.find(">  <NOTE>\nthird").expect("third property");
        assert!(first < second && second < third);
    }
}
