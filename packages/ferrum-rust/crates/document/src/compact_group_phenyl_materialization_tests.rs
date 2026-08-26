use std::collections::BTreeMap;

use crate::compact_group_materialization_v1::TypedCompactGroupMaterializationRequestV1;
use crate::{PersistentId, TypedDocument};

fn id(value: &str) -> PersistentId {
    PersistentId::new(value.to_owned()).expect("test ID")
}

fn endpoint_source_id(endpoint: &ferrum_core::VertexRef) -> String {
    match endpoint {
        ferrum_core::VertexRef::Atom(id) => id.source_id().as_str().to_owned(),
        other => panic!("Phenyl endpoint must be an atom, got {other:?}"),
    }
}

#[test]
fn attached_phenyl_materializes_the_exact_normal_kekule_cycle() {
    let source = "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom><compact-group id=\"group\" version=\"1\" catalog-key=\"phenyl\" attachment-index=\"0\" orientation-degrees=\"0\"><point x=\"20\" y=\"0\"/></compact-group><bond id=\"outside\" start=\"anchor\" end=\"group\" type=\"n1\"/></molecule></cdml>";
    let document = TypedDocument::parse(source).expect("typed source");
    let result = document
        .prepare_compact_group_materialization_v1(TypedCompactGroupMaterializationRequestV1::new(
            id("m"),
            id("group"),
            [
                "attachment",
                "ortho-upper",
                "meta-upper",
                "para",
                "meta-lower",
                "ortho-lower",
            ]
            .into_iter()
            .map(id)
            .collect(),
            [
                "attachment-ortho-upper",
                "ortho-upper-meta-upper",
                "meta-upper-para",
                "para-meta-lower",
                "meta-lower-ortho-lower",
                "ortho-lower-attachment",
            ]
            .into_iter()
            .map(id)
            .collect(),
        ))
        .and_then(|plan| document.materialize_compact_group_v1(&plan))
        .expect("Phenyl materialization");
    let projected = result
        .candidate()
        .core_projection()
        .expect("candidate projects");
    let molecule = projected
        .molecules()
        .iter()
        .find(|molecule| molecule.source_id().as_str() == "m")
        .expect("source molecule");
    assert_eq!(result.attachment_focus().as_str(), "attachment");
    assert!(
        !molecule
            .groups()
            .iter()
            .any(|group| group.source_id().as_str() == "group")
    );
    let atoms: BTreeMap<_, _> = molecule
        .atoms()
        .iter()
        .map(|atom| {
            (
                atom.source_id().as_str(),
                (atom.element(), atom.formal_charge()),
            )
        })
        .collect();
    assert_eq!(
        atoms,
        BTreeMap::from([
            ("anchor", (Some("C"), None)),
            ("attachment", (Some("C"), None)),
            ("ortho-upper", (Some("C"), None)),
            ("meta-upper", (Some("C"), None)),
            ("para", (Some("C"), None)),
            ("meta-lower", (Some("C"), None)),
            ("ortho-lower", (Some("C"), None)),
        ])
    );
    let bonds: BTreeMap<_, _> = molecule
        .bonds()
        .iter()
        .map(|bond| {
            (
                (
                    endpoint_source_id(bond.start()),
                    endpoint_source_id(bond.end()),
                ),
                (bond.order(), bond.style()),
            )
        })
        .collect();
    assert_eq!(
        bonds,
        BTreeMap::from([
            (
                ("anchor".to_owned(), "attachment".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Single),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
            (
                ("attachment".to_owned(), "ortho-upper".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Double),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
            (
                ("ortho-upper".to_owned(), "meta-upper".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Single),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
            (
                ("meta-upper".to_owned(), "para".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Double),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
            (
                ("para".to_owned(), "meta-lower".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Single),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
            (
                ("meta-lower".to_owned(), "ortho-lower".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Double),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
            (
                ("ortho-lower".to_owned(), "attachment".to_owned()),
                (
                    Some(ferrum_core::BondOrder::Single),
                    Some(&ferrum_core::BondStyle::Normal)
                )
            ),
        ])
    );
}
