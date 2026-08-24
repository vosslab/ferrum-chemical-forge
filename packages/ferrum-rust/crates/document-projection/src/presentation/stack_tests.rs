//! Tests for immutable presentation-stack projection invariants.

#[cfg(test)]
mod tests {
    use super::super::{
        PRESENTATION_STACK_PROJECTION_SCHEMA_V1, PolylinePathV1, PolylineProjectionV1,
        PresentationFactProvenanceV1, PresentationRecordKindV1, PresentationRootProjectionV1,
        PresentationStackProjectionV1, PresentationStackProjectionV1Error, PresentationStrokeV1,
        PresentationTargetV1,
    };
    use crate::{
        BracketPairProjectionV1, DocumentObjectIdV1, Point3V1, PositiveFiniteV1,
        PresentationBracketStyleV1, ProjectionLocalObjectKeyV1, Rgb24V1,
    };

    fn class_name(kind: PresentationRecordKindV1) -> &'static str {
        match kind {
            PresentationRecordKindV1::Arrow => "cdml/arrow",
            PresentationRecordKindV1::Plus => "cdml/plus",
            PresentationRecordKindV1::Text => "cdml/text",
            PresentationRecordKindV1::Polyline => "cdml/polyline",
            PresentationRecordKindV1::Rectangle => "cdml/rect",
            PresentationRecordKindV1::Square => "cdml/square",
            PresentationRecordKindV1::Oval => "cdml/oval",
            PresentationRecordKindV1::Circle => "cdml/circle",
            PresentationRecordKindV1::Polygon => "cdml/polygon",
        }
    }

    fn target(
        source_id: &str,
        source_order: u32,
        projection_path: u32,
        kind: PresentationRecordKindV1,
    ) -> PresentationTargetV1 {
        PresentationTargetV1::try_new(
            Some(
                DocumentObjectIdV1::from_class_source(class_name(kind), source_id)
                    .expect("test target has a valid durable identity"),
            ),
            ProjectionLocalObjectKeyV1::from_path_components(&[projection_path])
                .expect("test target has a projection-local path"),
            Some(source_id.to_owned()),
            source_order,
            kind,
        )
        .expect("test target has coherent durable and source identities")
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

    fn polyline(source_id: &str, source_order: u32, projection_path: u32) -> PolylineProjectionV1 {
        PolylineProjectionV1::new(
            target(
                source_id,
                source_order,
                projection_path,
                PresentationRecordKindV1::Polyline,
            ),
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
        assert!(stack.roots().is_empty());
        assert!(stack.bracket_pairs().is_empty());
        assert!(stack.issues().is_empty());
    }

    #[test]
    fn stack_rejects_duplicate_durable_source_and_projection_local_key() {
        let duplicate_source = PresentationStackProjectionV1::new(
            0,
            [0; 32],
            vec![
                PresentationRootProjectionV1::polyline(polyline("same", 0, 0))
                    .expect("matching root kind"),
                PresentationRootProjectionV1::polyline(polyline("same", 1, 1))
                    .expect("matching root kind"),
            ],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            duplicate_source,
            Err(PresentationStackProjectionV1Error::DuplicateRootSourceId)
        );

        let duplicate_key = PresentationStackProjectionV1::new(
            0,
            [0; 32],
            vec![
                PresentationRootProjectionV1::polyline(polyline("first", 0, 0))
                    .expect("matching root kind"),
                PresentationRootProjectionV1::polyline(polyline("second", 1, 0))
                    .expect("matching root kind"),
            ],
            Vec::new(),
            Vec::new(),
        );
        assert_eq!(
            duplicate_key,
            Err(PresentationStackProjectionV1Error::DuplicateRootProjectionKey)
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
                target("plus", 0, 0, PresentationRecordKindV1::Plus),
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
        let left = PresentationRootProjectionV1::round_bracket(polyline("left", 0, 0))
            .expect("polyline target is a valid round bracket root");
        let pair = BracketPairProjectionV1::try_new(
            "left".to_owned(),
            ["left".to_owned(), "right".to_owned()],
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
}
