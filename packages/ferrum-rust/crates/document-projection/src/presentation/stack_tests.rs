//! Tests for immutable presentation-stack projection invariants.

#[cfg(test)]
mod tests {
    use super::super::{
        PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PolylinePathV1, PolylineProjectionV1,
        PresentationFactProvenanceV1, PresentationProjectionIssueCodeV1,
        PresentationProjectionIssueV1, PresentationRecordKindV1, PresentationRootProjectionV1,
        PresentationStackProjectionV1, PresentationStackProjectionV1Error, PresentationStrokeV1,
        PresentationTargetV1,
    };
    use crate::{
        BracketPairProjectionV1, DocumentObjectIdV1, Point3V1, PositiveFiniteV1,
        PresentationBracketStyleV1, Rgb24V1,
    };

    fn target(object_id_byte: u8, kind: PresentationRecordKindV1) -> PresentationTargetV1 {
        PresentationTargetV1::new(
            DocumentObjectIdV1::from_entropy_bytes([object_id_byte; 16]),
            kind,
        )
    }

    fn stroke() -> PresentationStrokeV1 {
        PresentationStrokeV1::new(
            Rgb24V1::new("#000000").expect("valid builtin line color"),
            PresentationFactProvenanceV1::Builtin,
            PositiveFiniteV1::new(1.0).expect("valid builtin line width"),
            PresentationFactProvenanceV1::Builtin,
        )
        .expect("builtin stroke facts are coherent")
    }

    fn polyline(object_id_byte: u8) -> PolylineProjectionV1 {
        PolylineProjectionV1::new(
            target(object_id_byte, PresentationRecordKindV1::Polyline),
            PolylinePathV1::try_new(vec![
                Point3V1::new(0.0, 0.0, 0.0).expect("finite test point"),
                Point3V1::new(1.0, 1.0, 0.0).expect("finite test point"),
            ])
            .expect("two test points form a path"),
            stroke(),
        )
        .expect("polyline target matches its root payload")
    }

    #[test]
    fn new_owns_an_empty_stack_with_the_closed_schema() {
        let stack =
            PresentationStackProjectionV1::new(7, [3; 32], Vec::new(), Vec::new(), Vec::new())
                .expect("an empty presentation stack has no bracket consistency conflict");

        assert_eq!(stack.schema(), PRESENTATION_STACK_PROJECTION_SCHEMA_V1);
        assert_eq!(stack.revision(), 7);
        assert_eq!(stack.digest(), &[3; 32]);
        assert!(stack.entries().is_empty());
        assert!(stack.bracket_pairs().is_empty());
        assert!(stack.issues().is_empty());
    }

    #[test]
    fn stack_rejects_duplicate_durable_targets() {
        let duplicate_target = PresentationStackProjectionV1::new(
            0,
            [0; 32],
            vec![
                PresentationRootProjectionV1::polyline(polyline(1)).expect("matching root kind"),
                PresentationRootProjectionV1::polyline(polyline(1)).expect("matching root kind"),
            ],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            duplicate_target,
            Err(PresentationStackProjectionV1Error::DuplicateRootDurableId)
        );
    }
    #[test]
    fn public_constructors_reject_invalid_path_and_root_kind() {
        assert_eq!(
            PolylinePathV1::try_new(vec![
                Point3V1::new(0.0, 0.0, 0.0).expect("finite test point"),
            ]),
            Err(PresentationStackProjectionV1Error::InvalidPolylinePath)
        );
        assert_eq!(
            PolylineProjectionV1::new(
                target(1, PresentationRecordKindV1::Plus),
                PolylinePathV1::try_new(vec![
                    Point3V1::new(0.0, 0.0, 0.0).expect("finite test point"),
                    Point3V1::new(1.0, 1.0, 0.0).expect("finite test point"),
                ])
                .expect("two test points form a path"),
                stroke(),
            ),
            Err(PresentationStackProjectionV1Error::RootKindMismatch)
        );
    }

    #[test]
    fn stack_rejects_round_bracket_root_pair_disagreement() {
        let left = PresentationRootProjectionV1::round_bracket(polyline(1))
            .expect("polyline target is a valid round bracket root");
        let pair = BracketPairProjectionV1::try_new(
            [
                DocumentObjectIdV1::from_entropy_bytes([1; 16]),
                DocumentObjectIdV1::from_entropy_bytes([2; 16]),
            ],
            PresentationBracketStyleV1::Round,
            None,
            None,
        )
        .expect("distinct durable test bracket members form a pair");
        assert_eq!(
            PresentationStackProjectionV1::new(0, [0; 32], vec![left], vec![pair], Vec::new()),
            Err(PresentationStackProjectionV1Error::RoundBracketPairMismatch)
        );
    }

    #[test]
    fn target_serialization_is_durable_only_and_rejects_old_or_unknown_fields() {
        let target = target(7, PresentationRecordKindV1::Text);
        let value = serde_json::to_value(&target).expect("target serializes");
        assert_eq!(value.as_object().expect("target is an object").len(), 2);
        assert!(value.get("document_object_id").is_some());
        assert_eq!(value.get("record_kind"), Some(&serde_json::json!("text")));

        for prohibited in ["source_id", "projection_key", "source_order", "unknown"] {
            let json = format!(
                r#"{{\"document_object_id\":\"{}\",\"record_kind\":\"text\",\"{prohibited}\":\"old\"}}"#,
                target.document_object_id().as_str()
            );
            assert!(serde_json::from_str::<PresentationTargetV1>(&json).is_err());
        }
    }

    #[test]
    fn issues_reject_retired_paint_order_and_legacy_fields() {
        let issue = PresentationProjectionIssueV1::new(
            target(7, PresentationRecordKindV1::Text),
            PresentationProjectionIssueCodeV1::InvalidTextContent,
            "invalid text",
        );
        let value = serde_json::to_value(&issue).expect("issue serializes");

        assert!(value.get("paint_order").is_none());

        for prohibited in [
            "paint_order",
            "source_id",
            "projection_key",
            "source_order",
            "unknown",
        ] {
            let mut invalid = value.clone();
            invalid
                .as_object_mut()
                .expect("issue serializes as an object")
                .insert(prohibited.to_owned(), serde_json::json!("old"));
            assert!(serde_json::from_value::<PresentationProjectionIssueV1>(invalid).is_err());
        }
    }

    #[test]
    fn stack_entries_preserve_content_order_and_bracket_membership() {
        let left_id = DocumentObjectIdV1::from_entropy_bytes([1; 16]);
        let right_id = DocumentObjectIdV1::from_entropy_bytes([2; 16]);
        let left = PresentationRootProjectionV1::round_bracket(polyline(1))
            .expect("polyline target is a valid round bracket root");
        let right = PresentationRootProjectionV1::round_bracket(polyline(2))
            .expect("polyline target is a valid round bracket root");
        let pair = BracketPairProjectionV1::try_new(
            [left_id.clone(), right_id.clone()],
            PresentationBracketStyleV1::Round,
            None,
            None,
        )
        .expect("durable members form a bracket pair");
        let stack = PresentationStackProjectionV1::new(
            0,
            [0; 32],
            vec![left, right],
            vec![pair],
            Vec::new(),
        )
        .expect("durable bracket roots match the pair members");

        assert_eq!(
            stack.entries()[0].root().target().document_object_id(),
            &left_id
        );
        assert_eq!(
            stack.entries()[1].root().target().document_object_id(),
            &right_id
        );
        assert_eq!(stack.bracket_pairs()[0].members(), &[left_id, right_id]);
    }
}
