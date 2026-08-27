//! CDML composition over chemistry-owned interchange records and codecs.

use ferrum_chemistry::{
    ChemEngine, ChemistryError, InterchangeCodecErrorV1 as ChemistryInterchangeCodecErrorV1,
    decode_non_cdml_interchange_v1, encode_non_cdml_interchange_v1,
};
use thiserror::Error;

use crate::{
    CoreProjectionError, DocumentMoleculeGraphError, TypedClass, TypedDocument,
    document_molecule_coordinate_graph_v1,
};

pub use ferrum_chemistry::INTERCHANGE_MAX_TEXT_BYTES_V1;
pub use ferrum_chemistry::{InterchangeFormatV1, InterchangePropertyV1, InterchangeRecordV1};

/// Decode bounded text into complete, owned chemistry records.
pub fn decode_interchange_v1(
    engine: &dyn ChemEngine,
    format: InterchangeFormatV1,
    text: &str,
) -> Result<Vec<InterchangeRecordV1>, InterchangeCodecErrorV1> {
    if format != InterchangeFormatV1::Cdml {
        return decode_non_cdml_interchange_v1(engine, format, text).map_err(Into::into);
    }
    enforce_cdml_input_limit(text)?;
    let records = decode_cdml(text)?;
    if records.is_empty() {
        return Err(InterchangeCodecErrorV1::NoMolecularRecords);
    }
    Ok(records)
}

/// Encode complete owned records into one closed interchange syntax.
pub fn encode_interchange_v1(
    engine: &dyn ChemEngine,
    format: InterchangeFormatV1,
    records: &[InterchangeRecordV1],
) -> Result<String, InterchangeCodecErrorV1> {
    if format != InterchangeFormatV1::Cdml {
        return encode_non_cdml_interchange_v1(engine, format, records).map_err(Into::into);
    }
    if records.is_empty() {
        return Err(InterchangeCodecErrorV1::NoMolecularRecords);
    }
    let output = encode_cdml(records)?;
    if output.len() > INTERCHANGE_MAX_TEXT_BYTES_V1 {
        return Err(InterchangeCodecErrorV1::OutputTooLarge {
            format,
            limit: INTERCHANGE_MAX_TEXT_BYTES_V1,
            observed_at_least: output.len(),
        });
    }
    Ok(output)
}

fn enforce_cdml_input_limit(text: &str) -> Result<(), InterchangeCodecErrorV1> {
    if text.len() > INTERCHANGE_MAX_TEXT_BYTES_V1 {
        return Err(InterchangeCodecErrorV1::InputTooLarge {
            format: InterchangeFormatV1::Cdml,
            limit: INTERCHANGE_MAX_TEXT_BYTES_V1,
            observed_at_least: text.len(),
        });
    }
    Ok(())
}

fn decode_cdml(text: &str) -> Result<Vec<InterchangeRecordV1>, InterchangeCodecErrorV1> {
    let document = TypedDocument::parse(text).map_err(InterchangeCodecErrorV1::TypedDocument)?;
    if has_unrepresentable_cdml_content(document.root())
        || document
            .root()
            .typed_children()
            .iter()
            .any(|child| child.record().class() != TypedClass::Molecule)
    {
        return Err(InterchangeCodecErrorV1::NonMolecularCdml);
    }
    let projection = document
        .core_projection()
        .map_err(InterchangeCodecErrorV1::CdmlProjection)?;
    let mut records = Vec::with_capacity(projection.molecules().len());
    for molecule in projection.molecules() {
        let graph = document_molecule_coordinate_graph_v1(molecule)
            .map_err(InterchangeCodecErrorV1::DocumentGraph)?
            .into_parts()
            .0;
        records.push(InterchangeRecordV1::new(
            graph,
            molecule.name().map(str::to_owned),
            Vec::new(),
        ));
    }
    Ok(records)
}

/// Refuse any opaque data rather than rebuilding a graph-only CDML projection
/// that would silently discard it. This recursive check intentionally covers
/// molecule, atom, bond, and point descendants, not only document-root data.
fn has_unrepresentable_cdml_content(record: &crate::TypedRecord) -> bool {
    !record.unknown_attributes().is_empty()
        || !record.unrecognized_children().is_empty()
        || record
            .typed_children()
            .iter()
            .any(|child| has_unrepresentable_cdml_content(child.record()))
}

fn encode_cdml(records: &[InterchangeRecordV1]) -> Result<String, InterchangeCodecErrorV1> {
    let mut output = String::from("<cdml xmlns=\"urn:ferrum:cdml\" version=\"1.0\">");
    for (record_index, record) in records.iter().enumerate() {
        if !record.properties().is_empty() {
            return Err(
                InterchangeCodecErrorV1::CdmlInterchangePropertiesUnsupported { record_index },
            );
        }
        let graph = record.molecule();
        let points = graph
            .coordinates()
            .ok_or(InterchangeCodecErrorV1::CdmlCoordinatesRequired { record_index })?
            .points();
        output.push_str("<molecule id=\"m");
        output.push_str(&record_index.to_string());
        output.push('\"');
        if let Some(title) = record.title() {
            push_attribute(&mut output, " name", title);
        }
        output.push('>');
        for (atom_index, (atom, point)) in graph.atoms().iter().zip(points).enumerate() {
            output.push_str("<atom id=\"a");
            output.push_str(&record_index.to_string());
            output.push('_');
            output.push_str(&atom_index.to_string());
            output.push_str("\" name=\"");
            output.push_str(atom.atomic_number().symbol());
            output.push('\"');
            if let Some(value) = atom.formal_charge().filter(|value| *value != 0) {
                push_attribute(&mut output, " charge", &value.to_string());
            }
            if let Some(value) = atom.isotope() {
                push_attribute(&mut output, " isotope", &value.to_string());
            }
            if let Some(value) = atom.explicit_hydrogens().filter(|value| *value != 0) {
                push_attribute(&mut output, " explicit_hydrogens", &value.to_string());
            }
            output.push_str("><point x=\"");
            output.push_str(&point.x().to_string());
            output.push_str("\" y=\"");
            output.push_str(&(-point.y()).to_string());
            output.push_str("\"/></atom>");
        }
        for (bond_index, bond) in graph.bonds().iter().enumerate() {
            let order = match bond.order() {
                ferrum_chemistry::BondOrder::Single => 1,
                ferrum_chemistry::BondOrder::Double => 2,
                ferrum_chemistry::BondOrder::Triple => 3,
                _ => {
                    return Err(InterchangeCodecErrorV1::CdmlUnsupportedBond {
                        record_index,
                        bond_index,
                    });
                }
            };
            output.push_str("<bond id=\"b");
            output.push_str(&record_index.to_string());
            output.push('_');
            output.push_str(&bond_index.to_string());
            output.push_str("\" start=\"a");
            output.push_str(&record_index.to_string());
            output.push('_');
            output.push_str(&bond.start().to_string());
            output.push_str("\" end=\"a");
            output.push_str(&record_index.to_string());
            output.push('_');
            output.push_str(&bond.end().to_string());
            output.push_str("\" type=\"n");
            output.push_str(&order.to_string());
            output.push_str("\"/>");
        }
        output.push_str("</molecule>");
    }
    output.push_str("</cdml>");
    Ok(output)
}

fn push_attribute(output: &mut String, name: &str, value: &str) {
    output.push_str(name);
    output.push_str("=\"");
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '\"' => output.push_str("&quot;"),
            _ => output.push(character),
        }
    }
    output.push('\"');
}

/// A refused source, unsupported target projection, or chemistry codec failure.
#[derive(Debug, Error)]
pub enum InterchangeCodecErrorV1 {
    #[error("interchange property is empty or contains an unretainable control character")]
    InvalidInterchangeProperty,
    #[error("{format:?} input exceeds {limit} bytes (observed {observed_at_least})")]
    InputTooLarge {
        format: InterchangeFormatV1,
        limit: usize,
        observed_at_least: usize,
    },
    #[error("{format:?} output exceeds {limit} bytes (observed {observed_at_least})")]
    OutputTooLarge {
        format: InterchangeFormatV1,
        limit: usize,
        observed_at_least: usize,
    },
    #[error("{format:?} cannot represent {record_count} records")]
    MultiRecordUnsupported {
        format: InterchangeFormatV1,
        record_count: usize,
    },
    #[error("interchange input contains no molecular records")]
    NoMolecularRecords,
    #[error("interchange source is available only for opening a new document")]
    DocumentImportOnly,
    #[error(
        "CDML interchange accepts only direct molecular roots with no opaque or presentation content"
    )]
    NonMolecularCdml,
    #[error("CDML record {record_index} has no complete 2D coordinates")]
    CdmlCoordinatesRequired { record_index: usize },
    #[error(
        "CDML record {record_index} bond {bond_index} is not representable by the direct chemistry projection"
    )]
    CdmlUnsupportedBond {
        record_index: usize,
        bond_index: usize,
    },
    #[error("CDML record {record_index} cannot losslessly carry ordered interchange properties")]
    CdmlInterchangePropertiesUnsupported { record_index: usize },
    #[error(transparent)]
    CmlEncoding(#[from] ferrum_chemistry::CmlEncoderErrorV1),
    #[error(transparent)]
    Chemistry(#[from] ChemistryError),
    #[error(transparent)]
    SdfRecord(#[from] ferrum_chemistry::SdfError),
    #[error(transparent)]
    TypedDocument(#[from] crate::TypedDocumentError),
    #[error(transparent)]
    CdmlProjection(#[from] CoreProjectionError),
    #[error(transparent)]
    DocumentGraph(#[from] DocumentMoleculeGraphError),
}

impl From<ChemistryInterchangeCodecErrorV1> for InterchangeCodecErrorV1 {
    fn from(value: ChemistryInterchangeCodecErrorV1) -> Self {
        match value {
            ChemistryInterchangeCodecErrorV1::InvalidInterchangeProperty => {
                Self::InvalidInterchangeProperty
            }
            ChemistryInterchangeCodecErrorV1::InputTooLarge {
                format,
                limit,
                observed_at_least,
            } => Self::InputTooLarge {
                format,
                limit,
                observed_at_least,
            },
            ChemistryInterchangeCodecErrorV1::OutputTooLarge {
                format,
                limit,
                observed_at_least,
            } => Self::OutputTooLarge {
                format,
                limit,
                observed_at_least,
            },
            ChemistryInterchangeCodecErrorV1::MultiRecordUnsupported {
                format,
                record_count,
            } => Self::MultiRecordUnsupported {
                format,
                record_count,
            },
            ChemistryInterchangeCodecErrorV1::NoMolecularRecords => Self::NoMolecularRecords,
            ChemistryInterchangeCodecErrorV1::CdmlRequiresDocumentComposition => {
                unreachable!("CDML dispatch stays in document")
            }
            ChemistryInterchangeCodecErrorV1::DocumentImportOnly => Self::DocumentImportOnly,
            ChemistryInterchangeCodecErrorV1::CmlEncoding(error) => Self::CmlEncoding(error),
            ChemistryInterchangeCodecErrorV1::Chemistry(error) => Self::Chemistry(error),
            ChemistryInterchangeCodecErrorV1::SdfRecord(error) => Self::SdfRecord(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use ferrum_chemistry::{
        AtomicNumber, BondOrder, Coordinates, MolAtom, MolBond, MolGraph, Point2,
        UnavailableChemEngine,
    };

    use super::*;

    fn graph() -> MolGraph {
        MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::try_from(6).expect("carbon"),
                    None,
                    None,
                    None,
                    false,
                )
                .expect("atom"),
                MolAtom::new(
                    AtomicNumber::try_from(8).expect("oxygen"),
                    None,
                    None,
                    None,
                    false,
                )
                .expect("atom"),
            ],
            vec![MolBond::new(0, 1, BondOrder::Single, false)],
            Some(Coordinates::new(vec![
                Point2::new(0.0, 0.0).expect("point"),
                Point2::new(1.0, 0.0).expect("point"),
            ])),
        )
        .expect("complete graph")
    }

    #[test]
    fn cdml_refuses_nested_opaque_attribute_and_child() {
        for source in [
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\" vendor=\"kept\"><point x=\"0\" y=\"0\"/></atom></molecule></cdml>",
            "<cdml xmlns=\"urn:ferrum:cdml\"><molecule id=\"m\"><atom id=\"a\" name=\"C\"><point x=\"0\" y=\"0\"/><vendor:payload xmlns:vendor=\"urn:vendor\"/></atom></molecule></cdml>",
        ] {
            let error =
                decode_interchange_v1(&UnavailableChemEngine, InterchangeFormatV1::Cdml, source)
                    .expect_err("opaque nested content must never be stripped");
            assert!(matches!(error, InterchangeCodecErrorV1::NonMolecularCdml));
        }
    }

    #[test]
    fn cdml_refuses_ordered_sdf_metadata_instead_of_discarding_it() {
        let properties = vec![
            InterchangePropertyV1::new("SOURCE", "first").expect("property"),
            InterchangePropertyV1::new("SOURCE", "second").expect("property"),
        ];
        let records = vec![InterchangeRecordV1::new(
            graph(),
            Some("record".to_owned()),
            properties,
        )];

        let error =
            encode_interchange_v1(&UnavailableChemEngine, InterchangeFormatV1::Cdml, &records)
                .expect_err("ordered SDF metadata has no CDML interchange representation");
        assert!(matches!(
            error,
            InterchangeCodecErrorV1::CdmlInterchangePropertiesUnsupported { record_index: 0 }
        ));
    }
}
