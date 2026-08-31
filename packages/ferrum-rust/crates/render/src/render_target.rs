//! Durable render-target identity and private lowering context.

use ferrum_core::RecordId;
use ferrum_document_projection::DocumentObjectIdV1;
use serde::{Deserialize, Serialize};

/// The sole persisted selector for a rendered document object.
///
/// Source records and ordering are renderer-local lowering facts. They belong
/// to the private render-plan entry context, never to this public wire value.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderTarget {
    document_object_id: DocumentObjectIdV1,
}

impl RenderTarget {
    /// Construct a durable target admitted by the document model.
    #[must_use]
    pub const fn document_object(document_object_id: DocumentObjectIdV1) -> Self {
        Self { document_object_id }
    }

    /// Return the durable document object identity.
    #[must_use]
    pub const fn document_object_id(&self) -> &DocumentObjectIdV1 {
        &self.document_object_id
    }
}

/// Renderer-local facts needed while lowering one document-plan entry.
///
/// This is intentionally crate-private and has no serialization contract. A
/// source record is used only to resolve endpoints and compose primitives;
/// consumers receive the durable target and public plan paint order instead.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RenderPlanEntryContextV1 {
    target: RenderTarget,
    record_id: RecordId,
    paint_order: u32,
    owner_molecule_object_id: Option<DocumentObjectIdV1>,
}

impl RenderPlanEntryContextV1 {
    pub(crate) const fn new(
        target: RenderTarget,
        record_id: RecordId,
        paint_order: u32,
        owner_molecule_object_id: Option<DocumentObjectIdV1>,
    ) -> Self {
        Self {
            target,
            record_id,
            paint_order,
            owner_molecule_object_id,
        }
    }

    pub(crate) const fn target(&self) -> &RenderTarget {
        &self.target
    }

    pub(crate) const fn record_id(&self) -> &RecordId {
        &self.record_id
    }

    pub(crate) const fn paint_order(&self) -> u32 {
        self.paint_order
    }
}

#[cfg(test)]
mod tests {
    use ferrum_core::{Identifier, RecordKind};

    use super::*;
    use crate::{
        AtomLabelFacts, AtomLabelFontProfile, AtomLabelRenderV1, AtomRenderBatchV1, FontFace,
        PositiveFinite, RenderBatchV4, RenderPaintV3, RenderPoint, Rgb24,
    };

    fn durable_id(byte: u8) -> DocumentObjectIdV1 {
        DocumentObjectIdV1::from_entropy_bytes([byte; 16])
    }

    fn point(x: f64, y: f64) -> RenderPoint {
        RenderPoint::new(x, y).expect("finite test point")
    }

    fn size(value: f64) -> PositiveFinite {
        PositiveFinite::new(value).expect("positive test extent")
    }

    fn label() -> AtomLabelRenderV1 {
        RenderBatchV4::test_atom_label_from_facts(
            None,
            AtomLabelFacts::new("C", None, 0, 0).expect("label facts"),
            AtomLabelFontProfile::new(
                FontFace::molecule_label(),
                size(12.0),
                RenderPaintV3::authored_rgb24(Rgb24::new("000000").expect("paint")),
            ),
            1,
        )
        .expect("verified atom label")
    }

    #[test]
    fn public_target_serializes_only_its_durable_document_identity() {
        let target = RenderTarget::document_object(durable_id(0x41));
        let wire = serde_json::to_value(target).expect("target serializes");

        assert_eq!(wire.as_object().expect("target object").len(), 1);
        assert_eq!(
            wire["document_object_id"],
            serde_json::Value::String(durable_id(0x41).as_str().to_owned())
        );
        assert!(wire.get("record_id").is_none());
        assert!(wire.get("paint_order").is_none());
    }

    #[test]
    fn private_context_enforces_source_kind_before_public_batch_construction() {
        let context = RenderPlanEntryContextV1::new(
            RenderTarget::document_object(durable_id(0x42)),
            RecordId::new(
                RecordKind::Atom,
                Identifier::new("atom_source").expect("valid source identifier"),
            )
            .expect("valid atom record ID"),
            7,
            Some(durable_id(0x43)),
        );

        let content =
            AtomRenderBatchV1::new(point(0.0, 0.0), label(), Vec::new()).expect("atom content");
        let batch =
            RenderBatchV4::atom(context, content).expect("atom source accepts atom content");
        assert_eq!(batch.paint_order(), 7);

        let bond_context = RenderPlanEntryContextV1::new(
            RenderTarget::document_object(durable_id(0x44)),
            RecordId::new(
                RecordKind::Bond,
                Identifier::new("bond_source").expect("valid source identifier"),
            )
            .expect("valid bond record ID"),
            8,
            Some(durable_id(0x45)),
        );
        let error = RenderBatchV4::atom(
            bond_context,
            AtomRenderBatchV1::new(point(0.0, 0.0), label(), Vec::new()).expect("atom content"),
        )
        .expect_err("bond source cannot create atom content");
        assert!(error.to_string().contains("content kind"));
    }
}
