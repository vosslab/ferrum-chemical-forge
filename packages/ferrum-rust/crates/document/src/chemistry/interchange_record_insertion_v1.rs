//! Format-neutral conversion of chemistry-owned interchange records into one document batch.

use crate::{
    InterchangePropertyInsertionV1, InterchangeRecordBatchInsertionV1,
    InterchangeRecordInsertionV1, InterchangeRecordInsertionV1Error, MoleculeInsertionAtomV1,
    MoleculeInsertionV1, MoleculeInsertionV1Error, Point3V1, PreparedDocumentMoleculeV2,
    ProjectionError,
};
use ferrum_chemistry::{ChemEngine, ChemistryError, InterchangeRecordV1};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2};
use thiserror::Error;

use super::{DocumentMoleculePreparationErrorV2, prepare_complete_graph_for_document_v2};

/// Convert every decoded interchange record into one ordered atomic document batch.
///
/// A single record retains the ordinary atom-centroid placement contract. Multiple
/// records form one nonoverlapping horizontal row whose complete bounds are centered
/// on the requested anchor; adjacent source-scaled bounds have one target bond length
/// of whitespace. Titles and ordered properties remain attached to their own record.
pub fn build_interchange_record_batch_insertion_v1<E: ChemEngine + ?Sized>(
    engine: &E,
    records: &[InterchangeRecordV1],
    placement: MoleculePlacementV1,
) -> Result<InterchangeRecordBatchInsertionV1, InterchangeRecordBuildErrorV1> {
    let build_placement = if records.len() == 1 {
        placement
    } else {
        MoleculePlacementV1::new(
            placement.bond_length(),
            Point2::new(0.0, 0.0).expect("the coordinate origin is finite"),
        )?
    };
    let mut molecules = Vec::with_capacity(records.len());
    for record in records {
        molecules.push(prepare_complete_graph_for_document_v2(
            engine,
            record.molecule(),
            build_placement,
        )?);
    }
    if molecules.len() > 1 {
        molecules = arrange_record_row(molecules, placement)?;
    }

    let records = records
        .iter()
        .zip(molecules)
        .map(
            |(source, molecule)| -> Result<_, InterchangeRecordBuildErrorV1> {
                let properties = source
                    .properties()
                    .iter()
                    .map(|property| {
                        InterchangePropertyInsertionV1::new(property.name(), property.value())
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let request = molecule
                    .into_molecule_insertion_request_v1()
                    .map_err(|_| InterchangeRecordBuildErrorV1::InvalidPreparedSemantics)?;
                Ok(InterchangeRecordInsertionV1::new(
                    request,
                    source.title().unwrap_or_default(),
                    properties,
                )?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    InterchangeRecordBatchInsertionV1::new(records).map_err(Into::into)
}

fn arrange_record_row(
    molecules: Vec<PreparedDocumentMoleculeV2>,
    placement: MoleculePlacementV1,
) -> Result<Vec<PreparedDocumentMoleculeV2>, InterchangeRecordBuildErrorV1> {
    let bounds = molecules
        .iter()
        .map(|molecule| horizontal_bounds(molecule.molecule_insertion()))
        .collect::<Result<Vec<_>, _>>()?;
    let widths = bounds
        .iter()
        .map(|(minimum, maximum)| maximum - minimum)
        .collect::<Vec<_>>();
    let total_width = widths.iter().copied().sum::<f64>()
        + placement.bond_length() * (molecules.len() - 1) as f64;
    if !total_width.is_finite() {
        return Err(GeometryError::UnrepresentableGeometry.into());
    }
    let mut cursor = placement.anchor().x() - total_width / 2.0;
    if !cursor.is_finite() {
        return Err(GeometryError::UnrepresentableGeometry.into());
    }

    molecules
        .into_iter()
        .zip(bounds)
        .zip(widths)
        .map(|((molecule, (minimum, _)), width)| {
            let translated =
                translate_prepared_molecule(&molecule, cursor - minimum, placement.anchor().y())?;
            cursor += width + placement.bond_length();
            if !cursor.is_finite() {
                return Err(GeometryError::UnrepresentableGeometry.into());
            }
            Ok(translated)
        })
        .collect()
}

fn horizontal_bounds(
    molecule: &MoleculeInsertionV1,
) -> Result<(f64, f64), InterchangeRecordBuildErrorV1> {
    let mut positions = molecule.atoms().iter().map(|atom| atom.position().x());
    let first = positions
        .next()
        .expect("validated molecule insertions contain at least one atom");
    let (minimum, maximum) = positions.fold((first, first), |(minimum, maximum), value| {
        (minimum.min(value), maximum.max(value))
    });
    if !minimum.is_finite() || !maximum.is_finite() {
        return Err(GeometryError::UnrepresentableGeometry.into());
    }
    Ok((minimum, maximum))
}

fn translate_prepared_molecule(
    molecule: &PreparedDocumentMoleculeV2,
    delta_x: f64,
    delta_y: f64,
) -> Result<PreparedDocumentMoleculeV2, InterchangeRecordBuildErrorV1> {
    let molecule_insertion = molecule.molecule_insertion();
    let atoms = molecule_insertion
        .atoms()
        .iter()
        .map(|atom| {
            let position = atom.position();
            let position =
                Point3V1::new(position.x() + delta_x, position.y() + delta_y, position.z())?;
            MoleculeInsertionAtomV1::new(
                atom.element(),
                position,
                atom.formal_charge(),
                atom.isotope(),
                atom.explicit_hydrogens(),
            )
            .map_err(Into::into)
        })
        .collect::<Result<Vec<_>, InterchangeRecordBuildErrorV1>>()?;
    let translated = MoleculeInsertionV1::new(atoms, molecule_insertion.bonds().to_vec())?;
    PreparedDocumentMoleculeV2::with_stereo_reports(
        translated,
        molecule.stereo_semantics().cloned(),
        molecule.stereo_depictions().cloned(),
    )
    .map_err(|_| InterchangeRecordBuildErrorV1::InvalidPreparedSemantics)
}

/// Failure while converting decoded interchange records into document facts.
#[derive(Debug, Error)]
pub enum InterchangeRecordBuildErrorV1 {
    /// Native chemistry normalization failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// A complete chemistry graph cannot be represented in durable document facts.
    #[error(transparent)]
    Preparation(#[from] DocumentMoleculePreparationErrorV2),
    /// Multi-record placement could not produce finite geometry.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// A translated document point unexpectedly failed finite validation.
    #[error(transparent)]
    Position(#[from] ProjectionError),
    /// Rebuilding translated insertion facts failed.
    #[error(transparent)]
    Insertion(#[from] MoleculeInsertionV1Error),
    /// Interchange title, property, or nonempty-batch validation failed.
    #[error(transparent)]
    Metadata(#[from] InterchangeRecordInsertionV1Error),
    /// A previously admitted detached payload unexpectedly failed request lowering.
    #[error("prepared interchange molecule has invalid document semantics")]
    InvalidPreparedSemantics,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn molecule(element: &str, xs: &[f64]) -> PreparedDocumentMoleculeV2 {
        let atoms = xs
            .iter()
            .map(|x| {
                MoleculeInsertionAtomV1::new(
                    element,
                    Point3V1::new(*x, 0.0, 0.0).expect("test point is finite"),
                    None,
                    None,
                    None,
                )
                .expect("test atom is valid")
            })
            .collect();
        PreparedDocumentMoleculeV2::new(
            MoleculeInsertionV1::new(atoms, Vec::new()).expect("test molecule is valid"),
        )
    }

    #[test]
    fn record_row_centers_complete_bounds_with_one_bond_length_gap() {
        let placement =
            MoleculePlacementV1::new(3.0, Point2::new(10.0, 20.0).expect("test anchor is finite"))
                .expect("test placement is valid");
        let arranged = arrange_record_row(
            vec![molecule("C", &[-2.0, 0.0]), molecule("N", &[0.0, 4.0])],
            placement,
        )
        .expect("record row must arrange");
        let first =
            horizontal_bounds(arranged[0].molecule_insertion()).expect("first bounds are finite");
        let second =
            horizontal_bounds(arranged[1].molecule_insertion()).expect("second bounds are finite");

        assert_eq!(first, (5.5, 7.5));
        assert_eq!(second, (10.5, 14.5));
        assert_eq!(second.0 - first.1, placement.bond_length());
        assert_eq!((first.0 + second.1) / 2.0, placement.anchor().x());
        assert!(
            arranged
                .iter()
                .flat_map(|molecule| molecule.molecule_insertion().atoms())
                .all(|atom| atom.position().y() == placement.anchor().y())
        );
    }
}
