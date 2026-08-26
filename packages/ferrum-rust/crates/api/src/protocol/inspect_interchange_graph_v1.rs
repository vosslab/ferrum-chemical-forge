//! Bounded decoded-semantic interchange graph inspection.

use super::*;

const SCHEMA: &str = "ferrum-inspect-interchange-semantic-graph-v1";

pub(super) fn execute_inspect_interchange_graph<R: ChemistryRuntimeV1>(
    request: InspectInterchangeGraphRequestV1,
    runtime: &R,
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let capability =
        crate::InterchangeCapabilityResolverV1::lookup_input_format(request.input.format)
            .ok_or_else(inspection_format_unsupported)?;
    let profile = capability
        .graph_inspection_profile()
        .ok_or_else(inspection_format_unsupported)?;
    if request.input.text.len() > profile.max_source_bytes() {
        return Err(ExecutionFailureV1::interchange_import_refusal(
            crate::InterchangeImportRefusalV1::for_reason(
                crate::InterchangeImportRefusalReasonV1::InputBytesLimit,
            ),
        ));
    }
    match profile.route() {
        crate::interchange_capability_v1::InterchangeGraphInspectionRouteV1::CmlSimpleMolecule => {
            let document =
                crate::document_interchange_import_v1::decode_cml_simple_molecule_document_v1(
                    request.input.text.as_bytes(),
                )
                .map_err(ExecutionFailureV1::interchange_import_refusal)?;
            build_cml_summary(profile, document.records())
        }
        crate::interchange_capability_v1::InterchangeGraphInspectionRouteV1::SdfNativeSemantic => {
            let records = runtime
                .with_engine(|engine| {
                    Ok(ferrum_chemistry::decode_non_cdml_interchange_v1(
                        engine,
                        request.input.format,
                        &request.input.text,
                    )
                    .map_err(|error| ExecutionFailureV1::conversion_failed(error.to_string())))
                })
                .map_err(super::execution_chemistry::map_runtime_conversion_error)??;
            build_sdf_summary(profile, &records)
        }
    }
}

fn build_cml_summary(
    profile: crate::InterchangeGraphInspectionProfileV1,
    records: &[ferrum_chemistry::CmlDecodedRecordV1],
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut summaries = Vec::with_capacity(records.len());
    let mut atom_count = 0_u32;
    let mut bond_count = 0_u32;
    for (index, record) in records.iter().enumerate() {
        let atoms = u32::try_from(record.atoms().len())
            .map_err(|_| ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned()))?;
        let bonds = u32::try_from(record.bonds().len())
            .map_err(|_| ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned()))?;
        atom_count = atom_count.checked_add(atoms).ok_or_else(|| {
            ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned())
        })?;
        bond_count = bond_count.checked_add(bonds).ok_or_else(|| {
            ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned())
        })?;
        summaries.push(InspectInterchangeGraphRecordSummaryV1 {
            record_index: u32::try_from(index).map_err(|_| {
                ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned())
            })?,
            record_source_id: record
                .source_molecule_id()
                .map_or(SourceFactV1::Unknown, |id| SourceFactV1::Known {
                    value: id.to_owned(),
                }),
            record_title: SourceFactV1::Unsupported,
            property_count: SourceFactV1::Unsupported,
            atom_count: atoms,
            bond_count: bonds,
        });
    }
    Ok(OperationProtocolOutcomeV1::InspectInterchangeGraph {
        summary: InspectInterchangeGraphSummaryV1 {
            schema: SCHEMA.to_owned(),
            format_id: profile.format_id().to_owned(),
            profile_id: profile.profile_id().to_owned(),
            graph_meaning: "decoded_semantic".to_owned(),
            record_count: u32::try_from(records.len()).map_err(|_| {
                ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned())
            })?,
            atom_count,
            bond_count,
            records: summaries,
            declared_fact_coverage: profile.fact_coverage(),
            normalization: profile.normalization(),
        },
    })
}

fn inspection_format_unsupported() -> ExecutionFailureV1 {
    ExecutionFailureV1::conversion_unsupported(
        "inspection_format_unsupported:choose_cml_or_sdf".to_owned(),
    )
}

fn build_sdf_summary(
    profile: crate::InterchangeGraphInspectionProfileV1,
    records: &[ferrum_chemistry::InterchangeRecordV1],
) -> Result<OperationProtocolOutcomeV1, ExecutionFailureV1> {
    let mut summaries = Vec::with_capacity(records.len());
    let mut atom_count = 0_u32;
    let mut bond_count = 0_u32;
    for (index, record) in records.iter().enumerate() {
        let atoms = u32::try_from(record.molecule().atoms().len()).map_err(count_overflow)?;
        let bonds = u32::try_from(record.molecule().bonds().len()).map_err(count_overflow)?;
        let properties = u32::try_from(record.properties().len()).map_err(count_overflow)?;
        atom_count = atom_count
            .checked_add(atoms)
            .ok_or_else(|| count_overflow(()))?;
        bond_count = bond_count
            .checked_add(bonds)
            .ok_or_else(|| count_overflow(()))?;
        summaries.push(InspectInterchangeGraphRecordSummaryV1 {
            record_index: u32::try_from(index).map_err(|_| count_overflow(()))?,
            record_source_id: SourceFactV1::Unsupported,
            record_title: record.title().map_or(SourceFactV1::Unknown, |title| {
                SourceFactV1::Known {
                    value: title.to_owned(),
                }
            }),
            property_count: SourceFactV1::Known { value: properties },
            atom_count: atoms,
            bond_count: bonds,
        });
    }
    Ok(OperationProtocolOutcomeV1::InspectInterchangeGraph {
        summary: InspectInterchangeGraphSummaryV1 {
            schema: SCHEMA.to_owned(),
            format_id: profile.format_id().to_owned(),
            profile_id: profile.profile_id().to_owned(),
            graph_meaning: "decoded_semantic".to_owned(),
            record_count: u32::try_from(records.len()).map_err(count_overflow)?,
            atom_count,
            bond_count,
            records: summaries,
            declared_fact_coverage: profile.fact_coverage(),
            normalization: profile.normalization(),
        },
    })
}

fn count_overflow<T>(_: T) -> ExecutionFailureV1 {
    ExecutionFailureV1::internal("inspect_graph_count_overflow".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::runtime::{ChemistryRuntimeErrorV1, ChemistryRuntimeV1};
    use ferrum_chemistry::{
        AtomicNumber, ChemEngine, ChemistryError, Coordinates, ImportedSdfRecord, KekulizeOptions,
        MolAtom, MolGraph, Point2, SdfProperty, SmilesMolecule,
    };

    const CML: &str = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule id="first"><atomArray><atom id="a" elementType="C" x2="0" y2="0"/><atom id="b" elementType="O" x2="1" y2="0"/></atomArray><bondArray><bond atomRefs2="a b" order="1"/></bondArray></molecule><molecule><atomArray><atom id="c" elementType="N" x2="1" y2="2"/><atom id="d" elementType="H" x2="2" y2="2"/></atomArray><bondArray><bond atomRefs2="c d" order="1"/></bondArray></molecule></cml>"#;

    #[test]
    fn projects_complete_ordered_cml_source_records() {
        let outcome = execute_inspect_interchange_graph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                    text: CML.to_owned(),
                },
            },
            &NoChemistryRuntimeV1,
        )
        .expect("CML inspection succeeds");
        let OperationProtocolOutcomeV1::InspectInterchangeGraph { summary } = outcome else {
            panic!("graph outcome expected");
        };
        assert_eq!(
            (summary.record_count, summary.atom_count, summary.bond_count),
            (2, 4, 2)
        );
        assert_eq!(
            (
                summary.records[0].record_index,
                &summary.records[0].record_source_id,
                summary.records[0].atom_count,
                summary.records[0].bond_count,
                summary.records[1].record_index,
                &summary.records[1].record_source_id,
                summary.records[1].atom_count,
                summary.records[1].bond_count,
            ),
            (
                0,
                &SourceFactV1::Known {
                    value: "first".to_owned()
                },
                2,
                1,
                1,
                &SourceFactV1::Unknown,
                2,
                1,
            )
        );
    }

    #[test]
    fn refuses_sdf_when_the_declared_runtime_is_unavailable() {
        let error = execute_inspect_interchange_graph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::SdfV2000,
                    text: String::new(),
                },
            },
            &NoChemistryRuntimeV1,
        )
        .expect_err("SDF has no inspection profile");
        assert_eq!(
            error.category,
            OperationProtocolErrorCategoryV1::ChemistryUnavailable
        );
    }

    struct InjectedSdfEngine;

    impl ChemEngine for InjectedSdfEngine {
        fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles_to_molecule",
            })
        }

        fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "generate_2d_coordinates",
            })
        }

        fn sdf_to_records(&self, _: &str) -> Result<Vec<ImportedSdfRecord>, ChemistryError> {
            let graph = || {
                MolGraph::new(
                    vec![
                        MolAtom::new(
                            AtomicNumber::try_from(6).expect("carbon"),
                            Some(0),
                            None,
                            None,
                            true,
                        )
                        .expect("atom"),
                    ],
                    Vec::new(),
                    Some(Coordinates::new(vec![
                        Point2::new(0.0, 0.0).expect("point"),
                    ])),
                )
                .expect("graph")
            };
            Ok(vec![
                ImportedSdfRecord::new(
                    SmilesMolecule::new("c", graph()).expect("molecule"),
                    "first injected record".to_owned(),
                    vec![
                        SdfProperty::new("duplicate", "first").expect("property"),
                        SdfProperty::new("duplicate", "second").expect("property"),
                    ],
                ),
                ImportedSdfRecord::new(
                    SmilesMolecule::new("c", graph()).expect("molecule"),
                    String::new(),
                    Vec::new(),
                ),
            ])
        }

        fn kekulize(&self, _: &MolGraph, _: KekulizeOptions) -> Result<MolGraph, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "kekulize",
            })
        }
    }

    struct InjectedSdfRuntime(InjectedSdfEngine);

    impl ChemistryRuntimeV1 for InjectedSdfRuntime {
        fn with_engine<T>(
            &self,
            operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
        ) -> Result<T, ChemistryRuntimeErrorV1> {
            operation(&self.0)
        }
    }

    #[test]
    fn injected_sdf_engine_preserves_ordered_titles_duplicate_properties_and_native_disclosure() {
        let outcome = execute_inspect_interchange_graph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::SdfV2000,
                    text: "synthetic SDF accepted by the injected engine".to_owned(),
                },
            },
            &InjectedSdfRuntime(InjectedSdfEngine),
        )
        .expect("injected SDF inspection");
        let OperationProtocolOutcomeV1::InspectInterchangeGraph { summary } = outcome else {
            panic!("graph outcome expected");
        };
        assert_eq!(
            (summary.record_count, summary.atom_count, summary.bond_count),
            (2, 2, 0)
        );
        assert_eq!(summary.records[0].record_index, 0);
        assert_eq!(summary.records[1].record_index, 1);
        assert_eq!(
            (
                summary.records[0].record_index,
                &summary.records[0].record_title,
                summary.records[1].record_index,
                &summary.records[1].record_title,
            ),
            (
                0,
                &SourceFactV1::Known {
                    value: "first injected record".to_owned(),
                },
                1,
                &SourceFactV1::Known {
                    value: String::new(),
                },
            )
        );
        for record in &summary.records {
            assert_eq!(record.record_source_id, SourceFactV1::Unsupported);
        }
        assert_eq!(
            summary.records[0].property_count,
            SourceFactV1::Known { value: 2 }
        );
        assert_eq!(
            summary.records[1].property_count,
            SourceFactV1::Known { value: 0 }
        );
        assert_eq!(
            summary.declared_fact_coverage.aromaticity,
            InspectGraphFactCoverageStatusV1::Known
        );
        assert_eq!(
            summary.declared_fact_coverage.bond_stereo_direction,
            InspectGraphFactCoverageStatusV1::Known
        );
        assert_eq!(summary.normalization.aromaticity, "native_normalized");
        assert_eq!(
            summary.normalization.graph_normalization,
            "native_normalized"
        );
    }

    #[test]
    fn refuses_profile_external_cml_without_a_success_outcome() {
        let error = execute_inspect_interchange_graph(InspectInterchangeGraphRequestV1 {
            input: InspectInterchangeGraphInputV1 {
                format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                text: r#"<cml xmlns="https://example.invalid/not-cml2"><molecule><atomArray><atom id="a" elementType="C" x2="0" y2="0"/></atomArray></molecule></cml>"#.to_owned(),
            },
        }, &NoChemistryRuntimeV1)
        .expect_err("profile-external CML cannot produce an inspection outcome");
        assert_eq!(
            error.category,
            OperationProtocolErrorCategoryV1::ConversionUnsupported
        );
    }

    #[test]
    fn enforces_the_resolver_source_limit_before_decode() {
        let limit = crate::InterchangeCapabilityResolverV1::lookup_input_format(
            ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
        )
        .expect("CML inspection capability")
        .max_source_bytes();
        let error = execute_inspect_interchange_graph(
            InspectInterchangeGraphRequestV1 {
                input: InspectInterchangeGraphInputV1 {
                    format: ferrum_chemistry::InterchangeFormatV1::CmlSimpleMolecule,
                    text: "x".repeat(limit + 1),
                },
            },
            &NoChemistryRuntimeV1,
        )
        .expect_err("over-limit text cannot reach the decoder");
        assert_eq!(
            error.category,
            OperationProtocolErrorCategoryV1::ResourceLimit
        );
    }
}
