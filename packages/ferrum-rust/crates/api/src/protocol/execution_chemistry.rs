//! Chemistry-backed operation execution.

use super::*;

pub(super) fn execute_chemistry_convert<R: ChemistryRuntimeV1>(
    request: ChemistryConvertRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    if request.input.format == InterchangeFormatV1::Cdml
        && request.output_format == InterchangeFormatV1::Cdml
    {
        return execute_cdml_to_cdml_conversion(request);
    }
    let cml_records = if request.input.format == InterchangeFormatV1::CmlSimpleMolecule {
        Some(
            crate::document_interchange_import_v1::decode_cml_simple_molecule_records_v1(
                request.input.text.as_bytes(),
            )
            .map_err(ExecutionFailureV1::interchange_import_refusal)?,
        )
    } else {
        None
    };
    runtime
        .with_engine(|engine| {
            let records = match &cml_records {
                Some(records) => records.clone(),
                None => {
                    match decode_interchange_v1(engine, request.input.format, &request.input.text) {
                        Ok(records) => records,
                        Err(error) => return Ok(Err(map_conversion_error(error))),
                    }
                }
            };
            let record_count = records.len();
            let text = match encode_interchange_v1(engine, request.output_format, &records) {
                Ok(text) => text,
                Err(error) => return Ok(Err(map_conversion_error(error))),
            };
            Ok(Ok(OperationProtocolOutcomeV1::ChemistryConvert {
                format: request.output_format,
                text,
                record_count,
            }))
        })
        .map_err(map_runtime_conversion_error)?
}

/// Complete CDML-to-CDML projection without acquiring a chemistry runtime.
///
/// The document interchange codec preserves its explicit refusal behavior for
/// nonrepresentable presentation or opaque content; the unavailable engine is
/// never reached for this closed same-format conversion.
pub(super) fn execute_cdml_to_cdml_conversion(
    request: ChemistryConvertRequestV1,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let engine = UnavailableChemEngine;
    let records = decode_interchange_v1(&engine, request.input.format, &request.input.text)
        .map_err(map_conversion_error)?;
    let record_count = records.len();
    let text = encode_interchange_v1(&engine, request.output_format, &records)
        .map_err(map_conversion_error)?;
    Ok(OperationProtocolOutcomeV1::ChemistryConvert {
        format: request.output_format,
        text,
        record_count,
    })
}

pub(super) fn execute_generate_coordinates<R: ChemistryRuntimeV1>(
    source: &str,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut session = admit_document(source)?;
    let snapshot = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?;
    let observation = session
        .observe(snapshot.revision())
        .map_err(|error| ExecutionFailureV1::document_invalid(error.to_string()))?;
    // The admitted observation is the sole identity authority for this request.
    // Its direct-root sequence preserves the document's exact global order, and
    // each root carries the durable object ID issued at document admission.
    let ids = observation
        .projection()
        .direct_roots()
        .iter()
        .filter(|root| root.kind() == ferrum_document::DocumentDirectRootKindV1::Molecule)
        .map(|root| root.document_object_id().clone())
        .collect::<Vec<_>>();
    if ids.is_empty() {
        return Err(ExecutionFailureV1::coordinate(
            "coordinate generation requires at least one durable typed molecule".to_owned(),
        ));
    }
    let regenerated_molecule_count = ids.len();
    let batch = runtime
        .with_engine(|engine| {
            let updates = ids
                .iter()
                .map(|id| build_molecule_coordinate_update_v1(engine, &observation, id))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ExecutionFailureV1::coordinate(error.to_string()));
            let updates = match updates {
                Ok(updates) => updates,
                Err(error) => return Ok(Err(error)),
            };
            Ok(MoleculeCoordinateBatchUpdateV1::new(
                snapshot.revision(),
                *snapshot.digest(),
                updates,
            )
            .map_err(|error| ExecutionFailureV1::coordinate(error.to_string())))
        })
        .map_err(map_runtime_error)??;
    session
        .apply_document_operation_v1(
            snapshot.revision(),
            SessionOperation::V1(
                ferrum_document::SessionOperationV1::SetMoleculeAtomPositionsBatch {
                    update: batch,
                },
            ),
        )
        .map_err(|error| ExecutionFailureV1::coordinate(error.to_string()))?;
    let document = session
        .snapshot()
        .map_err(|error| ExecutionFailureV1::internal(error.to_string()))?
        .cdml()
        .to_owned();
    Ok(OperationProtocolOutcomeV1::GenerateCoordinates {
        document,
        regenerated_molecule_count,
    })
}

pub(super) fn map_runtime_error(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    // A runtime error occurs before coordinate generation can establish any
    // user-actionable chemistry result. In particular, a native loader error
    // can contain the trusted adapter path, so it must never become protocol
    // data. Semantic coordinate failures are mapped inside `with_engine`.
    let _ = error;
    ExecutionFailureV1::chemistry_runtime_unavailable()
}

pub(super) fn map_runtime_conversion_error(error: ChemistryRuntimeErrorV1) -> ExecutionFailureV1 {
    // Keep the trusted runtime capability and its native diagnostics entirely
    // inside this execution boundary. Semantic codec refusals are mapped by
    // `map_conversion_error` before the runtime result is returned.
    let _ = error;
    ExecutionFailureV1::chemistry_runtime_unavailable()
}

pub(super) fn map_conversion_error(error: InterchangeCodecErrorV1) -> ExecutionFailureV1 {
    match error {
        InterchangeCodecErrorV1::MultiRecordUnsupported { .. }
        | InterchangeCodecErrorV1::NonMolecularCdml
        | InterchangeCodecErrorV1::CdmlCoordinatesRequired { .. }
        | InterchangeCodecErrorV1::CdmlUnsupportedBond { .. }
        | InterchangeCodecErrorV1::CdmlInterchangePropertiesUnsupported { .. } => {
            ExecutionFailureV1::conversion_unsupported(error.to_string())
        }
        InterchangeCodecErrorV1::InputTooLarge { .. }
        | InterchangeCodecErrorV1::OutputTooLarge { .. } => {
            ExecutionFailureV1::resource_limit(error.to_string())
        }
        _ => ExecutionFailureV1::conversion_failed(error.to_string()),
    }
}
