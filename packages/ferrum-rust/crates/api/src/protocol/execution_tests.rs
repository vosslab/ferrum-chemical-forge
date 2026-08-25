#[cfg(test)]
mod tests {
    use super::super::*;
    use ferrum_chemistry::{
        ChemEngine, ChemistryError, Coordinates, KekulizeOptions, MolGraph, Point2, SmilesMolecule,
    };

    struct CoordinateOnlyEngine;

    impl ChemEngine for CoordinateOnlyEngine {
        fn smiles_to_molecule(&self, _smiles: &str) -> Result<SmilesMolecule, ChemistryError> {
            Err(ChemistryError::OperationUnavailable {
                operation: "smiles_to_molecule",
            })
        }

        fn generate_2d_coordinates(
            &self,
            molecule: &MolGraph,
        ) -> Result<Coordinates, ChemistryError> {
            let points = molecule
                .atoms()
                .iter()
                .map(|_| Point2::new(0.0, 0.0))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| ChemistryError::CoordinateGenerationFailed {
                    reason: error.to_string(),
                })?;
            Ok(Coordinates::new(points))
        }

        fn kekulize(
            &self,
            molecule: &MolGraph,
            _options: KekulizeOptions,
        ) -> Result<MolGraph, ChemistryError> {
            Ok(molecule.clone())
        }
    }

    struct CoordinateOnlyRuntime;

    impl ChemistryRuntimeV1 for CoordinateOnlyRuntime {
        fn with_engine<T>(
            &self,
            operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
        ) -> Result<T, ChemistryRuntimeErrorV1> {
            operation(&CoordinateOnlyEngine)
        }
    }

    struct HostileRuntime;

    impl ChemistryRuntimeV1 for HostileRuntime {
        fn with_engine<T>(
            &self,
            _operation: impl FnOnce(&dyn ChemEngine) -> Result<T, ChemistryRuntimeErrorV1>,
        ) -> Result<T, ChemistryRuntimeErrorV1> {
            Err(ChemistryRuntimeErrorV1::Chemistry(
                ChemistryError::NativeBoundary {
                    reason: HOSTILE_RUNTIME_DETAIL.to_owned(),
                },
            ))
        }
    }

    const CDML: &str = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule></cdml>";
    const HOSTILE_RUNTIME_DETAIL: &str = "/private/ferrum/.dylibs/libferrum_chem.dylib: private_native_adapter dlopen native loader text";

    fn document_fence(document: &str) -> (u64, String) {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "document-fence",
            "operation": {"kind": "document.inspect", "document": document},
        });
        let response = execute_operation_v1(&request.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("document inspection must succeed");
        };
        let OperationProtocolOutcomeV1::Inspect { document_fence, .. } = response.outcome else {
            panic!("document inspection outcome");
        };
        (
            document_fence.expected_revision,
            document_fence.expected_digest_hex,
        )
    }

    #[test]
    fn inspect_echoes_the_admitted_opaque_request_id() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "opaque: request id",
            "operation": {"kind": "document.inspect", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("inspection should succeed");
        };
        assert_eq!(response.request_id, "opaque: request id");
        assert!(matches!(
            response.outcome,
            OperationProtocolOutcomeV1::Inspect { .. }
        ));
    }

    #[test]
    fn reaction_create_protocol_is_canonical_and_rejects_stale_digest() {
        const REACTION_SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><molecule id=\"left\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000001\"><atom id=\"left-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000004\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"product\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000002\"><atom id=\"product-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000005\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"arrow\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000003\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow></cdml>";
        let (expected_revision, digest) = document_fence(REACTION_SOURCE);
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction",
            "operation": {"kind": "reaction.create.v1", "document": REACTION_SOURCE,
                "expected_revision": expected_revision, "expected_digest_hex": digest,
                "reactant_document_object_ids": ["ferrum-document-object-v1/00000000000000000000000000000001"],
                "product_document_object_ids": ["ferrum-document-object-v1/00000000000000000000000000000002"],
                "arrow_document_object_id": "ferrum-document-object-v1/00000000000000000000000000000003",
                "reagent_document_object_ids": [], "plus_document_object_ids": []}
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("reaction should create: {response:?}");
        };
        let OperationProtocolOutcomeV1::ReactionCreate {
            document,
            reaction_document_object_id,
            committed_revision,
            ..
        } = response.outcome
        else {
            panic!("reaction outcome expected");
        };
        assert_ne!(reaction_document_object_id, "rxn-1");
        assert!(
            ferrum_document::DocumentObjectIdV1::parse(reaction_document_object_id).is_ok(),
            "reaction receipt must expose one durable document-object ID"
        );
        assert_eq!(committed_revision, 1);
        assert!(document.contains("<reaction"));
        let mut stale = request;
        stale["operation"]["expected_digest_hex"] = serde_json::json!("00".repeat(32));
        let response = execute_operation_v1(&stale.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("stale digest must refuse");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::DocumentInvalid
        );
        assert!(response.error.reaction_refusal.is_some());
    }

    #[test]
    fn reaction_observation_protocol_lists_observes_and_selects_strict_membership() {
        const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><molecule id=\"left\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000011\"><atom id=\"left-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000015\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000012\"><atom id=\"right-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000016\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><arrow id=\"a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000013\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000014\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
        let (expected_revision, digest) = document_fence(SOURCE);
        let list = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction-list", "operation": { "kind": "reaction.list.v1", "document": SOURCE, "expected_revision": expected_revision, "expected_digest_hex": digest } });
        let response = execute_operation_v1(&list.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("list succeeds: {response:?}");
        };
        let OperationProtocolOutcomeV1::ReactionList { reactions, .. } = response.outcome else {
            panic!("list outcome");
        };
        assert_eq!(reactions.len(), 1);
        let reaction_document_object_id = reactions[0].reaction_document_object_id.clone();
        assert!(
            ferrum_document::DocumentObjectIdV1::parse(&reaction_document_object_id).is_ok(),
            "reaction observations must expose durable document-object IDs"
        );
        assert_eq!(reactions[0].members.len(), 3);
        let observe = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction-observe", "operation": { "kind": "reaction.observe.v1", "document": SOURCE, "expected_revision": expected_revision, "expected_digest_hex": digest, "reaction_document_object_id": reaction_document_object_id } });
        let response = execute_operation_v1(&observe.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("observe succeeds");
        };
        assert!(matches!(
            response.outcome,
            OperationProtocolOutcomeV1::ReactionObserve { .. }
        ));
        let select = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "reaction-select", "operation": { "kind": "reaction.select.v1", "document": SOURCE, "expected_revision": expected_revision, "expected_digest_hex": digest, "reaction_document_object_id": reaction_document_object_id } });
        let response = execute_operation_v1(&select.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("select succeeds");
        };
        let OperationProtocolOutcomeV1::ReactionSelect {
            reaction_document_object_id: selected_reaction_document_object_id,
            ..
        } = response.outcome
        else {
            panic!("select outcome");
        };
        assert_eq!(
            selected_reaction_document_object_id, reaction_document_object_id,
            "selection receipt must retain the observed durable reaction ID"
        );
    }

    #[test]
    fn reaction_lifecycle_protocol_replaces_members_and_deletes_only_definition() {
        const SOURCE: &str = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:object=\"urn:ferrum:document-object:v1\"><molecule id=\"left\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000021\"><atom id=\"left-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000026\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><molecule id=\"right\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000022\"><atom id=\"right-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000027\" name=\"O\"><point x=\"100\" y=\"0\"/></atom></molecule><molecule id=\"third\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000023\"><atom id=\"third-a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000028\" name=\"N\"><point x=\"140\" y=\"0\"/></atom></molecule><arrow id=\"a\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000024\"><point x=\"25\" y=\"0\"/><point x=\"75\" y=\"0\"/></arrow><reaction id=\"r\" object:id=\"ferrum-document-object-v1/00000000000000000000000000000025\"><reactant idref=\"left\"/><product idref=\"right\"/><arrow idref=\"a\"/></reaction></cdml>";
        let (expected_revision, digest) = document_fence(SOURCE);
        let list = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "list", "operation": { "kind": "reaction.list.v1", "document": SOURCE, "expected_revision": expected_revision, "expected_digest_hex": digest } });
        let listed = execute_operation_v1(&list.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = listed else {
            panic!("reaction list succeeds: {listed:?}");
        };
        let OperationProtocolOutcomeV1::ReactionList { reactions, .. } = response.outcome else {
            panic!("reaction list outcome");
        };
        let reaction = reactions.first().expect("one observed reaction");
        let reaction_document_object_id = reaction.reaction_document_object_id.clone();
        let reactant_document_object_id = reaction
            .members
            .iter()
            .find(|member| member.role == "reactant")
            .expect("reactant observation")
            .document_object_id
            .clone();
        let arrow_document_object_id = reaction
            .members
            .iter()
            .find(|member| member.role == "arrow")
            .expect("arrow observation")
            .document_object_id
            .clone();
        let patch = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "patch", "operation": { "kind": "reaction.patch-membership.v1", "document": SOURCE, "expected_revision": expected_revision, "expected_digest_hex": digest, "reaction_document_object_id": reaction_document_object_id, "reactant_document_object_ids": [reactant_document_object_id], "product_document_object_ids": ["ferrum-document-object-v1/00000000000000000000000000000023"], "arrow_document_object_id": arrow_document_object_id, "reagent_document_object_ids": [], "plus_document_object_ids": [] } });
        let response = execute_operation_v1(&patch.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("patch succeeds");
        };
        let OperationProtocolOutcomeV1::ReactionPatchMembership {
            document,
            reaction_document_object_id,
            committed_revision,
            ..
        } = response.outcome
        else {
            panic!("patch outcome");
        };
        assert_eq!(committed_revision, 1);
        assert!(document.contains("<reaction"));
        let (delete_expected_revision, delete_digest) = document_fence(&document);
        let delete = serde_json::json!({ "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1, "request_id": "delete", "operation": { "kind": "reaction.delete-definition.v1", "document": document, "expected_revision": delete_expected_revision, "expected_digest_hex": delete_digest, "reaction_document_object_id": reaction_document_object_id } });
        let response = execute_operation_v1(&delete.to_string()).expect("request");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("delete succeeds");
        };
        let OperationProtocolOutcomeV1::ReactionDeleteDefinition { document, .. } =
            response.outcome
        else {
            panic!("delete outcome");
        };
        assert!(!document.contains("<reaction"));
        assert!(document.contains("<molecule"));
    }

    #[test]
    fn unknown_schema_and_kind_are_closed_before_document_execution() {
        let version = serde_json::json!({
            "schema": "ferrum-operation-request-v2",
            "request_id": "v2",
            "operation": {"kind": "document.inspect", "document": "not CDML"},
        });
        let kind = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "future",
            "operation": {"kind": "document.future", "document": "not CDML"},
        });
        for (request, category) in [
            (
                version,
                OperationProtocolErrorCategoryV1::UnsupportedProtocolVersion,
            ),
            (kind, OperationProtocolErrorCategoryV1::InvalidRequest),
        ] {
            let response = execute_operation_v1(&request.to_string()).expect("JSON input");
            let OperationProtocolEnvelopeV1::Error(response) = response else {
                panic!("unknown schema or operation must be refused");
            };
            assert_eq!(response.error.category, category);
        }
    }

    #[test]
    fn rewrite_result_has_a_structural_rewrite_check() {
        let document = "<cdml xmlns=\"urn:ferrum:cdml\" xmlns:q=\"urn:test\"><q:payload id=\"foreign\"><q:item/></q:payload></cdml>";
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "rewrite",
            "operation": {"kind": "document.rewrite", "document": document},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("rewrite should succeed");
        };
        let OperationProtocolOutcomeV1::Rewrite { document, report } = response.outcome else {
            panic!("rewrite outcome expected");
        };
        assert!(report.valid);
        assert!(document.contains("cdml"));
    }

    #[test]
    fn artifact_result_declares_complete_svg_media_type() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "svg",
            "operation": {"kind": "document.render_artifact", "document": CDML, "format": "svg"},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("SVG should succeed");
        };
        let OperationProtocolOutcomeV1::RenderArtifact {
            media_type,
            artifact_base64,
            ..
        } = response.outcome
        else {
            panic!("artifact outcome expected");
        };
        assert_eq!(media_type, "image/svg+xml");
        let artifact = base64::engine::general_purpose::STANDARD
            .decode(artifact_base64)
            .expect("base64 artifact");
        assert!(artifact.starts_with(b"<svg"));
    }

    #[test]
    fn request_ingress_limit_rejects_before_json_parsing() {
        let error = ensure_request_json_budget("012345", 5).expect_err("limit refusal");
        assert!(matches!(
            error,
            OperationProtocolInputErrorV1::ResourceLimit {
                limit: 5,
                observed: 6,
            }
        ));
    }

    #[test]
    fn request_identifier_exact_boundary_is_echoed_in_the_response() {
        let request_id = "r".repeat(MAX_REQUEST_ID_UTF8_BYTES_V1);
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": request_id,
            "operation": {"kind": "document.inspect", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("boundary identifier must be admitted");
        };
        assert_eq!(response.request_id.len(), MAX_REQUEST_ID_UTF8_BYTES_V1);
    }

    #[test]
    fn oversized_request_identifier_is_not_echoed_in_error_response() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "r".repeat(MAX_REQUEST_ID_UTF8_BYTES_V1 + 1),
            "operation": {"kind": "document.inspect", "document": CDML},
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("oversized identifier must be refused");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::ResourceLimit
        );
        assert_eq!(response.request_id, None);
    }

    #[test]
    fn chemistry_operations_refuse_without_leaking_runtime_details() {
        let requests = [
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "convert-no-runtime",
                "operation": {
                    "kind": "chemistry.convert",
                    "input": {"format": "smiles", "text": "CCO"},
                    "output_format": "inchi_standard",
                },
            }),
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "coords-no-runtime",
                "operation": {"kind": "document.generate_coordinates", "document": CDML},
            }),
        ];
        for request in requests {
            let response = execute_operation_v1(&request.to_string()).expect("JSON input");
            let OperationProtocolEnvelopeV1::Error(response) = response else {
                panic!("missing runtime must be a typed refusal");
            };
            assert_eq!(
                response.error.category,
                OperationProtocolErrorCategoryV1::ChemistryUnavailable
            );
            assert!(!response.error.message.contains('/'));
        }
    }

    #[test]
    fn hostile_runtime_failures_are_redacted_for_all_runtime_backed_operations() {
        let requests = [
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "hostile-convert",
                "operation": {
                    "kind": "chemistry.convert",
                    "input": {"format": "smiles", "text": "CCO"},
                    "output_format": "inchi_standard",
                },
            }),
            serde_json::json!({
                "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
                "request_id": "hostile-coordinates",
                "operation": {"kind": "document.generate_coordinates", "document": CDML},
            }),
        ];
        for request in requests {
            let response = execute_operation_with_runtime_v1(&request.to_string(), &HostileRuntime)
                .expect("request decodes");
            let serialized = serde_json::to_string(&response).expect("response serializes");
            let value: serde_json::Value =
                serde_json::from_str(&serialized).expect("response JSON");
            assert_eq!(value["request_id"], request["request_id"]);
            assert_eq!(value["error"]["category"], "chemistry_unavailable");
            assert_eq!(
                value["error"]["message"],
                "Ferrum chemistry runtime is unavailable"
            );
            for private_detail in [
                HOSTILE_RUNTIME_DETAIL,
                ".dylibs",
                "libferrum_chem",
                "private_native_adapter",
                "dlopen",
            ] {
                assert!(!serialized.contains(private_detail));
            }
        }
    }

    #[test]
    fn cdml_to_cdml_conversion_completes_without_a_runtime() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "cdml-no-runtime",
            "operation": {
                "kind": "chemistry.convert",
                "input": {"format": "cdml", "text": CDML},
                "output_format": "cdml",
            },
        });
        let response = execute_operation_v1(&request.to_string()).expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("pure CDML conversion must not acquire a runtime: {response:?}");
        };
        let OperationProtocolOutcomeV1::ChemistryConvert { record_count, .. } = response.outcome
        else {
            panic!("CDML conversion outcome expected");
        };
        assert_eq!(record_count, 1);
    }

    #[test]
    fn convert_refuses_opaque_nested_cdml_instead_of_rebuilding_it_without_data() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "opaque-cdml",
            "operation": {
                "kind": "chemistry.convert",
                "input": {
                    "format": "cdml",
                    "text": "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\" vendor=\"kept\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>"
                },
                "output_format": "cdml"
            }
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("opaque CDML must be refused rather than projected");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::ConversionUnsupported
        );
    }

    #[test]
    fn schema_includes_the_additive_runtime_backed_operations() {
        let schema = generated_operation_protocol_schema_v1().to_string();
        assert!(schema.contains("chemistry.convert"));
        assert!(schema.contains("document.generate_coordinates"));
        assert!(schema.contains("conversion_unsupported"));
        assert!(schema.contains("coordinate_generation_failed"));
    }

    #[test]
    fn coordinate_generation_uses_one_injected_engine_capability() {
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "coords-runtime",
            "operation": {"kind": "document.generate_coordinates", "document": CDML},
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("runtime-backed coordinate generation should succeed: {response:?}");
        };
        let OperationProtocolOutcomeV1::GenerateCoordinates {
            document,
            regenerated_molecule_count,
        } = response.outcome
        else {
            panic!("coordinate outcome expected");
        };
        assert_eq!(regenerated_molecule_count, 1);
        assert!(document.contains("<cdml xmlns=\"urn:ferrum:cdml\""));
    }

    #[test]
    fn coordinate_generation_commits_all_direct_molecules_as_one_outcome() {
        let document = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule><molecule id=\"second\"><atom id=\"b\" name=\"O\"><point x=\"30\" y=\"40\"/></atom></molecule></cdml>";
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "coords-two-molecules",
            "operation": {"kind": "document.generate_coordinates", "document": document},
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Success(response) = response else {
            panic!("both molecules must commit as one coordinate outcome");
        };
        let OperationProtocolOutcomeV1::GenerateCoordinates {
            regenerated_molecule_count,
            ..
        } = response.outcome
        else {
            panic!("coordinate outcome expected");
        };
        assert_eq!(regenerated_molecule_count, 2);
    }

    #[test]
    fn coordinate_generation_refuses_invalid_later_molecule_without_outcome() {
        let document = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"first\"><atom id=\"a\" name=\"C\"><point x=\"10\" y=\"20\"/></atom></molecule><molecule id=\"second\"><atom id=\"b\"><point x=\"30\" y=\"40\"/></atom></molecule></cdml>";
        let request = serde_json::json!({
            "schema": OPERATION_PROTOCOL_REQUEST_SCHEMA_V1,
            "request_id": "coords-invalid-later",
            "operation": {"kind": "document.generate_coordinates", "document": document},
        });
        let response =
            execute_operation_with_runtime_v1(&request.to_string(), &CoordinateOnlyRuntime)
                .expect("JSON input");
        let OperationProtocolEnvelopeV1::Error(response) = response else {
            panic!("invalid later molecule must reject the complete batch");
        };
        assert_eq!(
            response.error.category,
            OperationProtocolErrorCategoryV1::CoordinateGenerationFailed
        );
    }
}
