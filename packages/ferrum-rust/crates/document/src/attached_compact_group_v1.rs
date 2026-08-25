//! Closed reviewed compact-group attachment facts and typed-CDML mutation.

use thiserror::Error;
use xot::Xot;

use ferrum_document_model::{
    is_reviewed_attached_compact_group_key_v1, reviewed_attached_compact_group_keys_v1,
};

use crate::{
    CDML_NAMESPACE, CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, PersistentId, Point3V1,
    TypedDocument, TypedDocumentError, element_name,
};

const EPSILON: f64 = 1.0e-10;

/// One finite scene release point for the closed compact-group authoring route.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachedCompactGroupReleaseV1 {
    x: f64,
    y: f64,
}

/// One closed compact-group attachment request.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttachCompactGroupV1 {
    catalog_key: CompactGroupCatalogKeyV1,
    release: AttachedCompactGroupReleaseV1,
}

impl AttachCompactGroupV1 {
    /// Build one request for a persisted catalog key and finite release point.
    #[must_use]
    pub const fn new(
        catalog_key: CompactGroupCatalogKeyV1,
        release: AttachedCompactGroupReleaseV1,
    ) -> Self {
        Self {
            catalog_key,
            release,
        }
    }

    /// Return the requested persisted catalog key.
    #[must_use]
    pub const fn catalog_key(self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    /// Return the finite scene release point.
    #[must_use]
    pub const fn release(self) -> AttachedCompactGroupReleaseV1 {
        self.release
    }
}

/// One Rust-owned choice exposed by the reviewed attached compact-group route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AttachedCompactGroupChoiceV1 {
    catalog_key: CompactGroupCatalogKeyV1,
}

impl AttachedCompactGroupChoiceV1 {
    const fn new(catalog_key: CompactGroupCatalogKeyV1) -> Self {
        Self { catalog_key }
    }

    /// Return the persisted key selected by this choice.
    #[must_use]
    pub const fn catalog_key(self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    /// Return the catalog-derived user-visible label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        self.catalog_key.label()
    }
}

/// Return the reviewed closed choices for attached compact-group authoring.
#[must_use]
pub const fn attached_compact_group_choices_v1() -> [AttachedCompactGroupChoiceV1; 2] {
    let [first, second] = reviewed_attached_compact_group_keys_v1();
    [
        AttachedCompactGroupChoiceV1::new(first),
        AttachedCompactGroupChoiceV1::new(second),
    ]
}

impl AttachedCompactGroupReleaseV1 {
    pub fn new(x: f64, y: f64) -> Result<Self, AttachedCompactGroupErrorV1> {
        if !x.is_finite() || !y.is_finite() {
            return Err(AttachedCompactGroupErrorV1::InvalidPose);
        }
        Ok(Self { x, y })
    }
}

/// Closed refusals before durable identity allocation or session mutation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum AttachedCompactGroupErrorV1 {
    #[error("compact-group attachment pose is invalid")]
    InvalidPose,
    #[error("compact-group attachment catalog key is not reviewed for attachment")]
    UnsupportedCatalogKey,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AttachedCompactGroupCandidateV1 {
    anchor: Point3V1,
    catalog_key: CompactGroupCatalogKeyV1,
    attachment: CompactGroupAttachmentV1,
}

impl AttachedCompactGroupCandidateV1 {
    pub(crate) const fn anchor(self) -> Point3V1 {
        self.anchor
    }

    pub(crate) const fn catalog_key(self) -> CompactGroupCatalogKeyV1 {
        self.catalog_key
    }

    pub(crate) const fn attachment(self) -> CompactGroupAttachmentV1 {
        self.attachment
    }
}

/// Derive one reviewed persisted compact-group pose from the direct release vector.
pub(crate) fn attached_compact_group_candidate_v1(
    anchor: Point3V1,
    request: AttachCompactGroupV1,
) -> Result<AttachedCompactGroupCandidateV1, AttachedCompactGroupErrorV1> {
    let catalog_key = request.catalog_key();
    if !is_reviewed_attached_compact_group_key_v1(catalog_key) {
        return Err(AttachedCompactGroupErrorV1::UnsupportedCatalogKey);
    }
    let release = request.release();
    let dx = release.x - anchor.x();
    let dy = release.y - anchor.y();
    if !dx.is_finite() || !dy.is_finite() || dx.hypot(dy) <= EPSILON {
        return Err(AttachedCompactGroupErrorV1::InvalidPose);
    }
    let group_anchor = Point3V1::new(release.x, release.y, anchor.z())
        .map_err(|_| AttachedCompactGroupErrorV1::InvalidPose)?;
    let attachment = CompactGroupAttachmentV1::new(catalog_key, 0, dy.atan2(dx).to_degrees())
        .map_err(|_| AttachedCompactGroupErrorV1::InvalidPose)?;
    Ok(AttachedCompactGroupCandidateV1 {
        anchor: group_anchor,
        catalog_key,
        attachment,
    })
}

impl TypedDocument {
    /// Build exactly one direct compact group plus one normal exterior bond.
    pub(crate) fn with_attach_compact_group_v1(
        &self,
        molecule_id: &PersistentId,
        anchor_atom_id: &PersistentId,
        group_id: &PersistentId,
        bond_id: &PersistentId,
        candidate: AttachedCompactGroupCandidateV1,
    ) -> Result<Self, TypedDocumentError> {
        if self.indexed().resolve_id(group_id).is_some() {
            return Err(TypedDocumentError::DuplicateGroupId(group_id.clone()));
        }
        if self.indexed().resolve_id(bond_id).is_some() {
            return Err(TypedDocumentError::DuplicateBondId(bond_id.clone()));
        }
        let mut detached = self.detached_candidate()?;
        let indexed = detached.detached_indexed_mut();
        let tree = &mut indexed.xml.tree;
        let root = tree
            .document_element(indexed.xml.document)
            .expect("parsed CDML has a root");
        let id_name = tree.add_name("id");
        let Some((molecule, namespace)) = tree.descendants(root).find_map(|node| {
            let (local, namespace) = element_name(tree, node)?;
            (local == "molecule"
                && namespace == CDML_NAMESPACE
                && tree.get_attribute(node, id_name) == Some(molecule_id.as_str()))
            .then_some((node, namespace))
        }) else {
            return Err(TypedDocumentError::UnknownMolecule(molecule_id.clone()));
        };
        let anchor_exists = tree.children(molecule).any(|node| {
            element_name(tree, node).is_some_and(|(local, node_namespace)| {
                local == "atom"
                    && node_namespace == CDML_NAMESPACE
                    && tree.get_attribute(node, id_name) == Some(anchor_atom_id.as_str())
            })
        });
        if !anchor_exists {
            return Err(TypedDocumentError::InvalidBondEndpoint(
                anchor_atom_id.clone(),
            ));
        }
        let group_name = element_name_id(tree, "compact-group", &namespace);
        let point_name = element_name_id(tree, "point", &namespace);
        let bond_name = element_name_id(tree, "bond", &namespace);
        let group = tree.new_element(group_name);
        for (name, value) in [
            ("id", group_id.as_str()),
            ("version", "1"),
            ("catalog-key", candidate.catalog_key().as_str()),
        ] {
            let attribute = tree.add_name(name);
            tree.set_attribute(group, attribute, value);
        }
        let attachment_index_name = tree.add_name("attachment-index");
        tree.set_attribute(
            group,
            attachment_index_name,
            candidate.attachment().attachment_index().to_string(),
        );
        let orientation_name = tree.add_name("orientation-degrees");
        tree.set_attribute(
            group,
            orientation_name,
            candidate.attachment().orientation_degrees().to_string(),
        );
        let point = tree.new_element(point_name);
        let x_name = tree.add_name("x");
        let y_name = tree.add_name("y");
        tree.set_attribute(point, x_name, candidate.anchor().x().to_string());
        tree.set_attribute(point, y_name, candidate.anchor().y().to_string());
        tree.append(group, point)
            .map_err(TypedDocumentError::Mutation)?;
        let bond = tree.new_element(bond_name);
        for (name, value) in [
            ("id", bond_id.as_str()),
            ("start", anchor_atom_id.as_str()),
            ("end", group_id.as_str()),
            ("type", "n1"),
        ] {
            let attribute = tree.add_name(name);
            tree.set_attribute(bond, attribute, value);
        }
        tree.append(molecule, group)
            .map_err(TypedDocumentError::Mutation)?;
        tree.append(molecule, bond)
            .map_err(TypedDocumentError::Mutation)?;
        let serialized = detached.to_xml()?;
        Self::parse(&serialized)
    }
}

fn element_name_id(tree: &mut Xot, local: &str, namespace: &str) -> xot::NameId {
    let namespace = tree.add_namespace(namespace);
    tree.add_name_ns(local, namespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> PersistentId {
        PersistentId::new(value).expect("test identifier")
    }

    #[test]
    fn reviewed_candidate_derives_a_finite_canonical_pose() {
        let anchor = Point3V1::new(0.0, 0.0, 0.0).expect("anchor");
        let candidate = attached_compact_group_candidate_v1(
            anchor,
            AttachCompactGroupV1::new(
                CompactGroupCatalogKeyV1::Methyl,
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
        )
        .expect("candidate");
        assert_eq!(
            candidate.anchor(),
            Point3V1::new(20.0, 0.0, 0.0).expect("point")
        );
        assert_eq!(candidate.catalog_key(), CompactGroupCatalogKeyV1::Methyl);
        assert_eq!(candidate.attachment().attachment_index(), 0);
        assert_eq!(candidate.attachment().orientation_degrees(), 0.0);
    }

    #[test]
    fn attached_compact_group_reports_the_colliding_group_identifier() {
        let document = TypedDocument::parse(concat!(
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"molecule\">",
            "<atom id=\"anchor\" name=\"C\"><point x=\"0\" y=\"0\"/></atom>",
            "</molecule></cdml>",
        ))
        .expect("typed source");
        let group_id = id("anchor");
        let candidate = attached_compact_group_candidate_v1(
            Point3V1::new(0.0, 0.0, 0.0).expect("anchor point"),
            AttachCompactGroupV1::new(
                CompactGroupCatalogKeyV1::Methyl,
                AttachedCompactGroupReleaseV1::new(20.0, 0.0).expect("release"),
            ),
        )
        .expect("candidate");

        let error = document
            .with_attach_compact_group_v1(
                &id("molecule"),
                &id("anchor"),
                &group_id,
                &id("bond"),
                candidate,
            )
            .expect_err("colliding compact-group ID must refuse");

        match error {
            TypedDocumentError::DuplicateGroupId(colliding_id) => {
                assert_eq!(colliding_id, group_id);
            }
            other => panic!("expected DuplicateGroupId, got {other:?}"),
        }
    }
}
