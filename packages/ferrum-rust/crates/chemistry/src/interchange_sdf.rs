//! SDF-specific translation for the format-neutral interchange model.

use crate::{
    ChemEngine, ImportedSdfRecord, InterchangeCodecErrorV1, InterchangePropertyV1,
    InterchangeRecordV1, MolblockVersion, NativeTextOutputLimit, SdfProperty, SdfRecord,
    validate_sdf_input,
};

/// The whole interchange SDF aggregate must fit the native text response envelope.
///
/// Passing this budget to the aggregate writer makes the native preflight own
/// titles, properties, record aggregation, and the resulting text allocation.
const INTERCHANGE_SDF_TEXT_OUTPUT_LIMIT: NativeTextOutputLimit =
    NativeTextOutputLimit::ADAPTER_MAXIMUM;

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
    let sdf_records = records
        .iter()
        .map(|record| {
            let properties = record
                .properties()
                .iter()
                .map(|property| {
                    SdfProperty::new(property.name(), property.value())
                        .map_err(InterchangeCodecErrorV1::SdfRecord)
                })
                .collect::<Result<Vec<_>, _>>()?;
            SdfRecord::new(
                record.molecule().clone(),
                record.title().unwrap_or_default(),
                properties,
            )
            .map_err(InterchangeCodecErrorV1::SdfRecord)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(engine.records_to_sdf(&sdf_records, version, INTERCHANGE_SDF_TEXT_OUTPUT_LIMIT)?)
}
