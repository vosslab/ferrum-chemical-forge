//! SDF-specific translation for the format-neutral interchange model.

use crate::{
    ChemEngine, INTERCHANGE_MAX_TEXT_BYTES_V1, ImportedSdfRecord, InterchangeCodecErrorV1,
    InterchangeFormatV1, InterchangePropertyV1, InterchangeRecordV1, MolblockVersion, SdfProperty,
    compose_sdf_record, validate_sdf_input,
};

/// Convert one parser-validated SDF record into Ferrum's generic interchange record.
#[must_use]
pub fn interchange_record_from_sdf_v1(record: ImportedSdfRecord) -> InterchangeRecordV1 {
    let properties = record
        .properties()
        .iter()
        .map(|property| {
            InterchangePropertyV1::new(property.name(), property.value())
                .expect("validated SDF property must satisfy interchange property invariants")
        })
        .collect();
    InterchangeRecordV1::new(
        record.molecule().molecule().clone(),
        Some(record.title().to_owned()),
        properties,
    )
}

pub(crate) fn decode_sdf_interchange_v1(
    engine: &dyn ChemEngine,
    text: &str,
) -> Result<Vec<InterchangeRecordV1>, InterchangeCodecErrorV1> {
    validate_sdf_input(text)?;
    Ok(engine
        .sdf_to_records(text)?
        .into_iter()
        .map(interchange_record_from_sdf_v1)
        .collect())
}

pub(crate) fn encode_sdf_interchange_v1(
    engine: &dyn ChemEngine,
    version: MolblockVersion,
    records: &[InterchangeRecordV1],
) -> Result<String, InterchangeCodecErrorV1> {
    let mut output = String::new();
    for record in records {
        let molblock = engine.molecule_to_molblock_with_title(
            record.molecule(),
            version,
            record.title().unwrap_or_default(),
        )?;
        let properties = record
            .properties()
            .iter()
            .map(|property| {
                SdfProperty::new(property.name(), property.value())
                    .map_err(InterchangeCodecErrorV1::SdfRecord)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fragment = compose_sdf_record(&molblock, &properties)
            .map_err(InterchangeCodecErrorV1::SdfRecord)?;
        let projected = output.len().checked_add(fragment.len()).ok_or(
            InterchangeCodecErrorV1::OutputTooLarge {
                format: sdf_format(version),
                limit: INTERCHANGE_MAX_TEXT_BYTES_V1,
                observed_at_least: usize::MAX,
            },
        )?;
        if projected > INTERCHANGE_MAX_TEXT_BYTES_V1 {
            return Err(InterchangeCodecErrorV1::OutputTooLarge {
                format: sdf_format(version),
                limit: INTERCHANGE_MAX_TEXT_BYTES_V1,
                observed_at_least: projected,
            });
        }
        output.push_str(&fragment);
    }
    Ok(output)
}

const fn sdf_format(version: MolblockVersion) -> InterchangeFormatV1 {
    match version {
        MolblockVersion::V2000 => InterchangeFormatV1::SdfV2000,
        MolblockVersion::V3000 => InterchangeFormatV1::SdfV3000,
    }
}
