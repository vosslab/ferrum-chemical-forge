//! CDXML-specific presentation lowering onto generic document interchange insertion.
//!
//! The chemistry decoder owns CDXML grammar and source-order presentation facts.
//! Generic interchange preparation owns graph validation, kekulization, placement,
//! metadata, and stereo admission. This adapter is the sole conversion from the
//! accepted CDXML presentation carrier into durable CDML bond presentation tokens.

use thiserror::Error;

use crate::{
    DocumentBondPresentationV1, InterchangeRecordBatchInsertionV1, InterchangeRecordInsertionV1,
    MoleculeInsertionBondV1, MoleculeInsertionV1,
};
use ferrum_chemistry::{CdxmlBondPresentationV1, CdxmlDecodedRecordV1, ChemEngine};
use ferrum_geometry::MoleculePlacementV1;

use super::{InterchangeRecordBuildErrorV1, build_interchange_record_batch_insertion_v1};

/// Build one generic document batch while retaining accepted CDXML fixed-single depictions.
///
/// This intentionally delegates graph validation, coordinate placement, stereochemical
/// admission, and metadata retention to the format-neutral preparation path first. It then
/// replaces only the document presentation attached to the same source-order bond index.
pub fn build_cdxml_record_batch_insertion<E: ChemEngine + ?Sized>(
    engine: &E,
    records: &[CdxmlDecodedRecordV1],
    placement: MoleculePlacementV1,
) -> Result<InterchangeRecordBatchInsertionV1, CdxmlRecordBuildError> {
    let interchange_records = records
        .iter()
        .map(CdxmlDecodedRecordV1::record)
        .cloned()
        .collect::<Vec<_>>();
    let prepared =
        build_interchange_record_batch_insertion_v1(engine, &interchange_records, placement)?;

    if prepared.records().len() != records.len() {
        return Err(CdxmlRecordBuildError::BatchCorrespondence);
    }
    let records = records
        .iter()
        .zip(prepared.records())
        .map(|(source, prepared)| overlay_cdxml_presentations(source, prepared))
        .collect::<Result<Vec<_>, _>>()?;
    InterchangeRecordBatchInsertionV1::new(records).map_err(CdxmlRecordBuildError::Metadata)
}

fn overlay_cdxml_presentations(
    source: &CdxmlDecodedRecordV1,
    prepared: &InterchangeRecordInsertionV1,
) -> Result<InterchangeRecordInsertionV1, CdxmlRecordBuildError> {
    let molecule = prepared.request().molecule();
    if source.bond_presentations().len() != molecule.bonds().len() {
        return Err(CdxmlRecordBuildError::CarrierInvariant);
    }
    if !source_bonds_match_prepared_molecule(source, molecule) {
        return Err(CdxmlRecordBuildError::CarrierInvariant);
    }
    let bonds = molecule
        .bonds()
        .iter()
        .zip(source.bond_presentations())
        .map(|(bond, presentation)| match presentation {
            None => Ok(bond.clone()),
            Some(presentation) if prepared_bond_is_plain_single(bond) => {
                Ok(MoleculeInsertionBondV1::new_with_presentation(
                    bond.start(),
                    bond.end(),
                    document_presentation(*presentation),
                ))
            }
            Some(_) => Err(CdxmlRecordBuildError::CarrierInvariant),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let molecule = MoleculeInsertionV1::new(molecule.atoms().to_vec(), bonds)
        .map_err(CdxmlRecordBuildError::Insertion)?;
    let request = crate::MoleculeInsertionRequestV1::with_stereo_reports(
        molecule,
        prepared.request().stereo_semantics().cloned(),
        prepared.request().stereo_depictions().cloned(),
    )
    .map_err(|_| CdxmlRecordBuildError::InvalidPreparedSemantics)?;
    InterchangeRecordInsertionV1::new(request, prepared.title(), prepared.properties().to_vec())
        .map_err(CdxmlRecordBuildError::Metadata)
}

fn source_bonds_match_prepared_molecule(
    source: &CdxmlDecodedRecordV1,
    prepared: &MoleculeInsertionV1,
) -> bool {
    source
        .record()
        .molecule()
        .bonds()
        .iter()
        .zip(prepared.bonds().iter().zip(source.bond_presentations()))
        .all(|(source, (prepared, presentation))| {
            let source_endpoints = normalized_endpoints(source.start(), source.end());
            let prepared_endpoints = normalized_endpoints(prepared.start(), prepared.end());
            source_endpoints == prepared_endpoints
                && source_order_matches_document(source.order(), prepared.order())
                && presentation_bond_is_admissible(source, prepared, *presentation)
        })
        && source.record().molecule().bonds().len() == prepared.bonds().len()
}

/// Admit the exact fixed-single overlay boundary, independently of decoder validation.
///
/// A CDXML presentation and a native bond direction encode distinct depiction facts.  They
/// must never be combined by this adapter: the source fact must be direction-free, and generic
/// preparation must still expose the exact plain-single presentation before overlay.
fn presentation_bond_is_admissible(
    source: &ferrum_chemistry::MolBond,
    prepared: &MoleculeInsertionBondV1,
    presentation: Option<CdxmlBondPresentationV1>,
) -> bool {
    match presentation {
        None => true,
        Some(_) => {
            source.direction() == ferrum_chemistry::BondDirection::None
                && prepared_bond_is_plain_single(prepared)
        }
    }
}

const fn prepared_bond_is_plain_single(bond: &MoleculeInsertionBondV1) -> bool {
    matches!(
        bond.presentation(),
        DocumentBondPresentationV1::Normal(crate::DocumentBondOrderV1::Single)
    )
}

const fn normalized_endpoints(first: usize, second: usize) -> (usize, usize) {
    if first < second {
        (first, second)
    } else {
        (second, first)
    }
}

const fn source_order_matches_document(
    source: ferrum_chemistry::BondOrder,
    document: crate::DocumentBondOrderV1,
) -> bool {
    matches!(
        (source, document),
        (
            ferrum_chemistry::BondOrder::Single,
            crate::DocumentBondOrderV1::Single
        ) | (
            ferrum_chemistry::BondOrder::Double,
            crate::DocumentBondOrderV1::Double
        ) | (
            ferrum_chemistry::BondOrder::Triple,
            crate::DocumentBondOrderV1::Triple
        )
    )
}

const fn document_presentation(
    presentation: CdxmlBondPresentationV1,
) -> DocumentBondPresentationV1 {
    match presentation {
        CdxmlBondPresentationV1::Wavy => DocumentBondPresentationV1::Wavy,
        CdxmlBondPresentationV1::Bold => DocumentBondPresentationV1::Bold,
        CdxmlBondPresentationV1::Dashed => DocumentBondPresentationV1::Dashed,
    }
}

/// Failure while lowering an accepted CDXML presentation carrier into document facts.
#[derive(Debug, Error)]
pub enum CdxmlRecordBuildError {
    /// The generic graph preparation or source metadata admission failed.
    #[error(transparent)]
    Generic(#[from] InterchangeRecordBuildErrorV1),
    /// The generic prepared batch no longer corresponds to the accepted source records.
    #[error("prepared CDXML batch does not preserve source-record correspondence")]
    BatchCorrespondence,
    /// The chemistry-owned presentation carrier violated its closed bond-index invariant.
    #[error("CDXML presentation carrier does not match its prepared molecule")]
    CarrierInvariant,
    /// Rebuilding the presentation-bearing molecule unexpectedly failed document validation.
    #[error(transparent)]
    Insertion(#[from] crate::MoleculeInsertionV1Error),
    /// Existing generic stereo facts were invalid after the presentation-only overlay.
    #[error("prepared CDXML molecule has invalid document stereo semantics")]
    InvalidPreparedSemantics,
    /// The generic record metadata could not be reconstructed.
    #[error(transparent)]
    Metadata(#[from] crate::InterchangeRecordInsertionV1Error),
}

#[cfg(test)]
mod tests {
    use ferrum_chemistry::{
        BondDirection, BondOrder, CdxmlBondPresentationV1, MolBond, UnavailableChemEngine,
        decode_cdxml_bytes_v1,
    };
    use ferrum_geometry::Point2;

    use super::*;

    #[test]
    fn cdxml_presentation_admission_refuses_a_directional_source_bond() {
        let source = MolBond::directed(0, 1, BondOrder::Single, false, BondDirection::BeginWedge)
            .expect("supported directional source bond");
        let prepared = MoleculeInsertionBondV1::new(0, 1, crate::DocumentBondOrderV1::Single);

        assert!(!presentation_bond_is_admissible(
            &source,
            &prepared,
            Some(CdxmlBondPresentationV1::Bold),
        ));
    }

    #[test]
    fn cdxml_presentation_admission_requires_an_exact_plain_single_prepared_bond() {
        let source = MolBond::new(0, 1, BondOrder::Single, false);
        for prepared in [
            MoleculeInsertionBondV1::new(0, 1, crate::DocumentBondOrderV1::Double),
            MoleculeInsertionBondV1::new_with_presentation(
                0,
                1,
                DocumentBondPresentationV1::SolidWedge,
            ),
        ] {
            assert!(!presentation_bond_is_admissible(
                &source,
                &prepared,
                Some(CdxmlBondPresentationV1::Dashed),
            ));
        }
    }

    #[test]
    fn cdxml_adapter_preserves_source_order_fixed_single_presentations() {
        let source = concat!(
            "<CDXML><page><fragment id=\"f\">",
            "<n id=\"a\" p=\"0 0\"/><n id=\"b\" p=\"1 0\"/>",
            "<n id=\"c\" p=\"2 0\"/><n id=\"d\" p=\"3 0\"/>",
            "<b B=\"a\" E=\"b\" Display=\"Bold\"/>",
            "<b B=\"b\" E=\"c\" Display=\"Dash\"/>",
            "<b B=\"c\" E=\"d\" Display=\"Wavy\"/>",
            "</fragment></page></CDXML>"
        );
        let decoded = decode_cdxml_bytes_v1(source.as_bytes()).expect("bounded source decodes");
        let placement =
            MoleculePlacementV1::new(40.0, Point2::new(100.0, 200.0).expect("finite anchor"))
                .expect("valid placement");
        let batch = build_cdxml_record_batch_insertion(
            &UnavailableChemEngine,
            decoded.records(),
            placement,
        )
        .expect("presentation adapter builds a generic batch");
        let bonds = batch.records()[0].request().molecule().bonds();
        assert_eq!(bonds[0].presentation(), DocumentBondPresentationV1::Bold);
        assert_eq!(bonds[1].presentation(), DocumentBondPresentationV1::Dashed);
        assert_eq!(bonds[2].presentation(), DocumentBondPresentationV1::Wavy);
        assert!(
            bonds
                .iter()
                .all(|bond| bond.order() == crate::DocumentBondOrderV1::Single)
        );
    }

    #[test]
    fn cdxml_presentations_survive_one_batch_history_round_trip_and_reopen() {
        let source = concat!(
            "<CDXML><page><fragment id=\"f\">",
            "<n id=\"a\" p=\"0 0\"/><n id=\"b\" p=\"1 0\"/>",
            "<n id=\"c\" p=\"2 0\"/><n id=\"d\" p=\"3 0\"/>",
            "<b B=\"a\" E=\"b\" Display=\"Bold\"/>",
            "<b B=\"b\" E=\"c\" Display=\"Dash\"/>",
            "<b B=\"c\" E=\"d\" Display=\"Wavy\"/>",
            "</fragment></page></CDXML>"
        );
        let decoded = decode_cdxml_bytes_v1(source.as_bytes()).expect("bounded source decodes");
        let placement =
            MoleculePlacementV1::new(40.0, Point2::new(100.0, 200.0).expect("finite anchor"))
                .expect("valid placement");
        let batch = build_cdxml_record_batch_insertion(
            &UnavailableChemEngine,
            decoded.records(),
            placement,
        )
        .expect("presentation adapter builds a generic batch");
        let mut session =
            crate::DocumentSession::create_empty_document_v1().expect("empty document");
        let inserted = session
            .apply_document_operation_v1(
                0,
                crate::SessionOperation::V1(
                    crate::SessionOperationV1::InsertInterchangeRecordBatchV1(batch),
                ),
            )
            .expect("one batch commits");
        assert_eq!(inserted.observation().snapshot().revision(), 1);
        let snapshot = inserted.observation().snapshot();
        for token in ["type=\"b1\"", "type=\"d1\"", "type=\"s1\""] {
            assert!(snapshot.cdml().contains(token));
        }
        let undone = session.undo(1).expect("one batch undoes");
        assert!(
            !undone
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"b1\"")
        );
        let redone = session
            .redo(undone.observation().snapshot().revision())
            .expect("one batch redoes");
        assert!(
            redone
                .observation()
                .snapshot()
                .cdml()
                .contains("type=\"b1\"")
        );
        let reopened = crate::DocumentSession::load(snapshot.cdml()).expect("snapshot reopens");
        let reopened_snapshot = reopened.snapshot().expect("reopened snapshot");
        for token in ["type=\"b1\"", "type=\"d1\"", "type=\"s1\""] {
            assert!(reopened_snapshot.cdml().contains(token));
        }
    }
}
