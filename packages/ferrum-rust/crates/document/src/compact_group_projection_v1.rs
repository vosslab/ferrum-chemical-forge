//! Typed compact-group V1 projection from its closed persisted record.

use ferrum_document_projection::{
    CompactGroupAttachmentV1, CompactGroupCatalogKeyV1, CompactGroupProjectionV1, CompactGroupV1,
    ProjectionError,
};

use crate::projection_v1::point;
use crate::{TypedChild, TypedClass, TypedRecord};

pub(crate) fn compact_group(
    child: &TypedChild,
) -> Result<CompactGroupProjectionV1, ProjectionError> {
    let record = child.record();
    let path = record.path().to_string();
    let version = required_field(record, "version", &path)?;
    if version != "1" {
        return Err(invalid_field(record, "version", &path));
    }
    let id = crate::projection_identity_v1::projection_document_object_id_from_record_v1(record)?;
    let raw_catalog_key = required_field(record, "catalog-key", &path)?;
    let catalog_key = CompactGroupCatalogKeyV1::parse(raw_catalog_key).ok_or_else(|| {
        ProjectionError::CompactGroup {
            path: path.clone(),
            source: crate::CompactGroupV1Error::UnsupportedCatalogKey(raw_catalog_key.to_owned()),
        }
    })?;
    let attachment_index = required_field(record, "attachment-index", &path)?
        .parse::<u8>()
        .map_err(|_| invalid_field(record, "attachment-index", &path))?;
    let orientation_degrees = required_field(record, "orientation-degrees", &path)?
        .parse::<f64>()
        .map_err(|_| invalid_field(record, "orientation-degrees", &path))?;
    let attachment =
        CompactGroupAttachmentV1::new(catalog_key, attachment_index, orientation_degrees).map_err(
            |source| ProjectionError::CompactGroup {
                path: path.clone(),
                source,
            },
        )?;
    let anchor_record = record
        .children_of(TypedClass::Point)
        .next()
        .ok_or_else(|| missing_field(record, "anchor", &path))?;
    let group = CompactGroupV1::new(id, catalog_key, point(anchor_record)?, attachment);
    Ok(CompactGroupProjectionV1::from_group(
        &group,
        child.position(),
    ))
}

fn required_field<'a>(
    record: &'a TypedRecord,
    field: &'static str,
    path: &str,
) -> Result<&'a str, ProjectionError> {
    record
        .attribute(field)
        .ok_or_else(|| missing_field(record, field, path))
}

fn missing_field(record: &TypedRecord, field: &'static str, path: &str) -> ProjectionError {
    ProjectionError::InvalidCompactGroupField {
        path: path.to_owned(),
        field,
        value: record.attribute(field).unwrap_or("<absent>").to_owned(),
    }
}

fn invalid_field(record: &TypedRecord, field: &'static str, path: &str) -> ProjectionError {
    ProjectionError::InvalidCompactGroupField {
        path: path.to_owned(),
        field,
        value: record.attribute(field).unwrap_or_default().to_owned(),
    }
}
