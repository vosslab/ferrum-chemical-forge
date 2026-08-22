//! One-way conversion from an accepted document observation to renderer facts.

use ferrum_document_model::{
    RenderAtomModelV1, RenderBondEndpointKindModelV1, RenderBondEndpointModelV1, RenderBondModelV1,
    RenderDiagnosticModelV1, RenderDocumentModelV1, RenderIdentityModelV1, RenderMoleculeModelV1,
    RenderPoint3ModelV1, RenderPresentationKindModelV1, RenderPresentationRootModelV1,
};
use thiserror::Error;

use crate::{
    BondEndpointKindV1, PresentationRootProjectionV1, PresentationTargetV1,
    SessionDocumentObservationV1,
};

/// Convert one immutable accepted observation into session-free renderer facts.
pub fn render_document_model_from_observation_v1(
    observation: &SessionDocumentObservationV1,
) -> Result<RenderDocumentModelV1, RenderDocumentModelConversionErrorV1> {
    let projection = observation.projection();
    let molecules = projection
        .molecules()
        .iter()
        .map(|molecule| {
            let atoms = molecule
                .atoms()
                .iter()
                .map(|atom| {
                    let point = atom.position();
                    Ok(RenderAtomModelV1::new(
                        identity(
                            atom.id().map(|value| value.as_str().to_owned()),
                            atom.projection_key().as_str().to_owned(),
                            atom.source_id().map(str::to_owned),
                            atom.source_order(),
                        ),
                        atom.element().map(str::to_owned),
                        RenderPoint3ModelV1::new(point.x(), point.y(), point.z())
                            .map_err(|_| RenderDocumentModelConversionErrorV1::InvalidPoint)?,
                        serde_json::to_value(atom)
                            .map_err(RenderDocumentModelConversionErrorV1::Serialize)?,
                    ))
                })
                .collect::<Result<Vec<_>, RenderDocumentModelConversionErrorV1>>()?;
            let bonds = molecule
                .bonds()
                .iter()
                .map(|bond| {
                    Ok(RenderBondModelV1::new(
                        identity(
                            bond.id().map(|value| value.as_str().to_owned()),
                            bond.projection_key().as_str().to_owned(),
                            bond.source_id().map(str::to_owned),
                            bond.source_order(),
                        ),
                        endpoint(bond.start()),
                        endpoint(bond.end()),
                        serde_json::to_value(bond)
                            .map_err(RenderDocumentModelConversionErrorV1::Serialize)?,
                    ))
                })
                .collect::<Result<Vec<_>, RenderDocumentModelConversionErrorV1>>()?;
            Ok(RenderMoleculeModelV1::new(
                identity(
                    molecule.id().map(|value| value.as_str().to_owned()),
                    molecule.projection_key().as_str().to_owned(),
                    molecule.source_id().map(str::to_owned),
                    molecule.source_order(),
                ),
                molecule.name().map(str::to_owned),
                atoms,
                bonds,
                serde_json::to_value(molecule)
                    .map_err(RenderDocumentModelConversionErrorV1::Serialize)?,
            ))
        })
        .collect::<Result<Vec<_>, RenderDocumentModelConversionErrorV1>>()?;
    let presentation_roots = projection
        .presentation_stack()
        .roots()
        .iter()
        .map(|root| {
            Ok(RenderPresentationRootModelV1::new(
                presentation_identity(root.target()),
                presentation_kind(root),
                serde_json::to_value(root)
                    .map_err(RenderDocumentModelConversionErrorV1::Serialize)?,
            ))
        })
        .collect::<Result<Vec<_>, RenderDocumentModelConversionErrorV1>>()?;
    let diagnostics = projection
        .issues()
        .iter()
        .map(|issue| {
            RenderDiagnosticModelV1::new(
                format!("{:?}", issue.code()),
                issue.path().to_owned(),
                issue.detail().to_owned(),
            )
        })
        .collect();
    let paper = serde_json::to_value(projection.paper_layout())
        .map_err(RenderDocumentModelConversionErrorV1::Serialize)?;
    let drawing_standard = projection
        .drawing_standard()
        .map(serde_json::to_value)
        .transpose()
        .map_err(RenderDocumentModelConversionErrorV1::Serialize)?;
    Ok(RenderDocumentModelV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        paper,
        drawing_standard,
        molecules,
        presentation_roots,
        diagnostics,
    ))
}

fn identity(
    durable_object_id: Option<String>,
    projection_key: String,
    source_id: Option<String>,
    source_order: u32,
) -> RenderIdentityModelV1 {
    RenderIdentityModelV1::new(durable_object_id, projection_key, source_id, source_order)
}

fn presentation_identity(target: &PresentationTargetV1) -> RenderIdentityModelV1 {
    identity(
        target.id().map(|value| value.as_str().to_owned()),
        target.projection_key().as_str().to_owned(),
        target.source_id().map(str::to_owned),
        target.source_order(),
    )
}

fn endpoint(value: &crate::BondEndpointV1) -> RenderBondEndpointModelV1 {
    let kind = match value.kind() {
        BondEndpointKindV1::Atom => RenderBondEndpointKindModelV1::Atom,
        BondEndpointKindV1::Group => RenderBondEndpointKindModelV1::Group,
        BondEndpointKindV1::MoleculeText => RenderBondEndpointKindModelV1::MoleculeText,
        BondEndpointKindV1::Query => RenderBondEndpointKindModelV1::Query,
        BondEndpointKindV1::Unknown => RenderBondEndpointKindModelV1::Unknown,
        BondEndpointKindV1::Missing => RenderBondEndpointKindModelV1::Missing,
    };
    RenderBondEndpointModelV1::new(
        value.source_id().map(str::to_owned),
        value
            .object_id()
            .map(|identifier| identifier.as_str().to_owned()),
        kind,
    )
}

fn presentation_kind(root: &PresentationRootProjectionV1) -> RenderPresentationKindModelV1 {
    match root {
        PresentationRootProjectionV1::Arrow { .. } => RenderPresentationKindModelV1::Arrow,
        PresentationRootProjectionV1::Plus { .. } => RenderPresentationKindModelV1::Plus,
        PresentationRootProjectionV1::Text { .. } => RenderPresentationKindModelV1::Text,
        PresentationRootProjectionV1::Polyline { .. } => RenderPresentationKindModelV1::Polyline,
        PresentationRootProjectionV1::Wavy { .. } => RenderPresentationKindModelV1::Wavy,
        PresentationRootProjectionV1::RoundBracket { .. } => {
            RenderPresentationKindModelV1::RoundBracket
        }
        PresentationRootProjectionV1::Rectangle { .. } => RenderPresentationKindModelV1::Rectangle,
        PresentationRootProjectionV1::Square { .. } => RenderPresentationKindModelV1::Square,
        PresentationRootProjectionV1::Oval { .. } => RenderPresentationKindModelV1::Oval,
        PresentationRootProjectionV1::Circle { .. } => RenderPresentationKindModelV1::Circle,
        PresentationRootProjectionV1::Polygon { .. } => RenderPresentationKindModelV1::Polygon,
    }
}

#[derive(Debug, Error)]
pub enum RenderDocumentModelConversionErrorV1 {
    #[error("document projection contained an invalid finite point")]
    InvalidPoint,
    #[error("document projection could not serialize renderer facts: {0}")]
    Serialize(#[source] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DocumentSession;

    #[test]
    fn transfer_model_is_equal_after_a_serde_round_trip_and_retains_root_identity() {
        let session = DocumentSession::load(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m1\"><atom id=\"a1\" name=\"C\"><point x=\"0\" y=\"0\"/></atom></molecule><plus id=\"p1\"><point x=\"2\" y=\"3\"/></plus></cdml>",
        )
        .expect("fixture loads");
        let observation = session.observe(0).expect("observation");
        let model = render_document_model_from_observation_v1(&observation).expect("transfer");
        let encoded = serde_json::to_string(&model).expect("serializes");
        let decoded: RenderDocumentModelV1 = serde_json::from_str(&encoded).expect("deserializes");
        assert_eq!(decoded, model);
        assert_eq!(model.molecules()[0].identity().source_id(), Some("m1"));
        assert_eq!(
            model.presentation_roots()[0].identity().source_id(),
            Some("p1")
        );
        assert_eq!(model.revision(), observation.snapshot().revision());
        assert_eq!(model.digest(), observation.snapshot().digest());
    }
}
