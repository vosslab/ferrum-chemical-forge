//! Rust-owned conversion of ordered native SDF records into one document batch.

use ferrum_chemistry::{
    ChemEngine, ChemistryError, ImportedSdfRecord, KekulizeOptions, KekulizeOptionsError,
};
use ferrum_document::{
    MoleculeInsertionAtomV1, MoleculeInsertionV1, MoleculeInsertionV1Error, Point3V1,
    ProjectionError, SdfPropertyInsertionV1, SdfRecordBatchInsertionV1, SdfRecordInsertionV1,
    SdfRecordInsertionV1Error,
};
use ferrum_geometry::{GeometryError, MoleculePlacementV1, Point2};
use thiserror::Error;

use crate::complete_graph_molecule_insertion_v1::{
    CompleteGraphMoleculeInsertionError, build_complete_graph_molecule_insertion_v1,
    validate_supported_complete_graph_facts_v1,
};

/// Convert every imported SDF record into one ordered atomic document batch.
///
/// A single record retains the ordinary atom-centroid placement contract. Multiple
/// records form one nonoverlapping horizontal row whose complete bounds are centered
/// on the requested anchor; adjacent source-scaled bounds have one target bond length
/// of whitespace. Titles and ordered properties remain attached to their own record.
pub fn build_sdf_record_batch_insertion_v1<E: ChemEngine>(
    engine: &E,
    records: &[ImportedSdfRecord],
    placement: MoleculePlacementV1,
) -> Result<SdfRecordBatchInsertionV1, SdfMoleculeBuildError> {
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
        let mut graph = record.molecule().molecule().clone();
        validate_supported_complete_graph_facts_v1(&graph)?;
        if graph
            .atoms()
            .iter()
            .any(ferrum_chemistry::MolAtom::is_aromatic)
            || graph
                .bonds()
                .iter()
                .any(ferrum_chemistry::MolBond::is_aromatic)
        {
            let options = KekulizeOptions::new(true, true, 100)?;
            graph = engine.kekulize(&graph, options)?;
        }
        molecules.push(build_complete_graph_molecule_insertion_v1(
            &graph,
            build_placement,
        )?);
    }
    if molecules.len() > 1 {
        molecules = arrange_record_row(molecules, placement)?;
    }

    let records = records
        .iter()
        .zip(molecules)
        .map(|(source, molecule)| {
            let properties = source
                .properties()
                .iter()
                .map(|property| SdfPropertyInsertionV1::new(property.name(), property.value()))
                .collect::<Result<Vec<_>, _>>()?;
            SdfRecordInsertionV1::new(molecule, source.title(), properties)
        })
        .collect::<Result<Vec<_>, _>>()?;
    SdfRecordBatchInsertionV1::new(records).map_err(Into::into)
}

fn arrange_record_row(
    molecules: Vec<MoleculeInsertionV1>,
    placement: MoleculePlacementV1,
) -> Result<Vec<MoleculeInsertionV1>, SdfMoleculeBuildError> {
    let bounds = molecules
        .iter()
        .map(horizontal_bounds)
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
                translate_molecule(&molecule, cursor - minimum, placement.anchor().y())?;
            cursor += width + placement.bond_length();
            if !cursor.is_finite() {
                return Err(GeometryError::UnrepresentableGeometry.into());
            }
            Ok(translated)
        })
        .collect()
}

fn horizontal_bounds(molecule: &MoleculeInsertionV1) -> Result<(f64, f64), SdfMoleculeBuildError> {
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

fn translate_molecule(
    molecule: &MoleculeInsertionV1,
    delta_x: f64,
    delta_y: f64,
) -> Result<MoleculeInsertionV1, SdfMoleculeBuildError> {
    let atoms = molecule
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
        .collect::<Result<Vec<_>, SdfMoleculeBuildError>>()?;
    MoleculeInsertionV1::new(atoms, molecule.bonds().to_vec()).map_err(Into::into)
}

/// Failure while converting untrusted native SDF records into document facts.
#[derive(Debug, Error)]
pub enum SdfMoleculeBuildError {
    /// Native chemistry normalization failed.
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    /// The explicit aromaticity-resolution request was invalid.
    #[error(transparent)]
    KekulizeOptions(#[from] KekulizeOptionsError),
    /// A complete chemistry graph cannot be represented in CDML V1 facts.
    #[error(transparent)]
    CompleteGraph(#[from] CompleteGraphMoleculeInsertionError),
    /// Multi-record placement could not produce finite geometry.
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    /// A translated document point unexpectedly failed finite validation.
    #[error(transparent)]
    Position(#[from] ProjectionError),
    /// Rebuilding translated insertion facts failed.
    #[error(transparent)]
    Insertion(#[from] MoleculeInsertionV1Error),
    /// SDF title, property, or nonempty-batch validation failed.
    #[error(transparent)]
    Metadata(#[from] SdfRecordInsertionV1Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn molecule(element: &str, xs: &[f64]) -> MoleculeInsertionV1 {
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
        MoleculeInsertionV1::new(atoms, Vec::new()).expect("test molecule is valid")
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
        let first = horizontal_bounds(&arranged[0]).expect("first bounds are finite");
        let second = horizontal_bounds(&arranged[1]).expect("second bounds are finite");

        assert_eq!(first, (5.5, 7.5));
        assert_eq!(second, (10.5, 14.5));
        assert_eq!(second.0 - first.1, placement.bond_length());
        assert_eq!((first.0 + second.1) / 2.0, placement.anchor().x());
        assert!(
            arranged
                .iter()
                .flat_map(MoleculeInsertionV1::atoms)
                .all(|atom| atom.position().y() == placement.anchor().y())
        );
    }
}
