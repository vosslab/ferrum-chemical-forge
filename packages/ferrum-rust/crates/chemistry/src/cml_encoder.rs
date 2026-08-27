//! Canonical CML2 serialization for the closed Ferrum chemistry profile.

use super::*;
use crate::{
    AtomChirality, BondDirection, BondOrder, BondStereo, InterchangeRecordV1, MolAtom, MolBond,
};

const CML2_NAMESPACE: &str = "http://www.xml-cml.org/schema/cml2/core";
const MAX_OUTPUT_BYTES: usize = 1_048_576;

type Result<T> = std::result::Result<T, CmlEncoderErrorV1>;

fn refused<T>(reason: CmlEncoderRefusalReasonV1) -> Result<T> {
    Err(CmlEncoderErrorV1 { reason })
}

/// Serialize parser-validated CML source records as one canonical CML2 document.
///
/// This preserves the validated source molecule and atom identifiers because the
/// value has never crossed Ferrum's durable document-identity boundary.
pub fn encode_cml_decoded_document_v1(document: &CmlDecodedDocumentV1) -> Result<String> {
    if document.records().is_empty() {
        return refused(CmlEncoderRefusalReasonV1::EmptyDocument);
    }
    let mut output = root_open();
    for record in document.records() {
        output.push_str("<molecule");
        if let Some(id) = record.source_molecule_id() {
            attribute(&mut output, "id", id);
        }
        output.push('>');
        output.push_str("<atomArray>");
        for atom in record.atoms() {
            output.push_str("<atom");
            attribute(&mut output, "id", atom.source_id());
            attribute(&mut output, "elementType", atom.element().symbol());
            decimal_attribute(&mut output, "x2", atom.x2());
            decimal_attribute(&mut output, "y2", atom.y2());
            if let Some(charge) = atom.formal_charge() {
                attribute(&mut output, "formalCharge", &charge.to_string());
            }
            if let Some(isotope) = atom.isotope() {
                attribute(&mut output, "isotopeNumber", &isotope.to_string());
            }
            output.push_str("/>");
        }
        output.push_str("</atomArray>");
        append_source_bonds(&mut output, record);
        output.push_str("</molecule>");
    }
    finish(output)
}

/// Serialize generic Ferrum interchange records as one canonical CML2 document.
///
/// Generic records deliberately receive local XML atom identifiers.  Durable
/// document identifiers, titles, and arbitrary interchange properties are not
/// CML facts in this closed profile.
pub fn encode_cml_interchange_records_v1(records: &[InterchangeRecordV1]) -> Result<String> {
    if records.is_empty() {
        return refused(CmlEncoderRefusalReasonV1::EmptyDocument);
    }
    let mut output = root_open();
    for record in records {
        if record.title().is_some() {
            return refused(CmlEncoderRefusalReasonV1::TitleUnsupported);
        }
        if !record.properties().is_empty() {
            return refused(CmlEncoderRefusalReasonV1::PropertiesUnsupported);
        }
        let molecule = record.molecule();
        let coordinates = molecule
            .coordinates()
            .ok_or(CmlEncoderErrorV1 {
                reason: CmlEncoderRefusalReasonV1::CoordinatesRequired,
            })?
            .points();
        if molecule.atoms().is_empty() {
            return refused(CmlEncoderRefusalReasonV1::GeneratedDocumentRejected);
        }

        output.push_str("<molecule><atomArray>");
        for (index, (atom, point)) in molecule.atoms().iter().zip(coordinates).enumerate() {
            validate_atom(atom)?;
            let x2 = point.x() / 30.0;
            let y2 = -point.y() / 30.0;
            if !x2.is_finite() || !y2.is_finite() || x2.abs() > 100_000.0 || y2.abs() > 100_000.0 {
                return refused(CmlEncoderRefusalReasonV1::CoordinateOutOfRange);
            }
            output.push_str("<atom");
            attribute(&mut output, "id", &format!("a{}", index + 1));
            attribute(&mut output, "elementType", atom.atomic_number().symbol());
            decimal_attribute(&mut output, "x2", x2);
            decimal_attribute(&mut output, "y2", y2);
            if let Some(charge) = atom.formal_charge() {
                attribute(&mut output, "formalCharge", &charge.to_string());
            }
            if let Some(isotope) = atom.isotope() {
                attribute(&mut output, "isotopeNumber", &isotope.to_string());
            }
            output.push_str("/>");
        }
        output.push_str("</atomArray>");
        append_generic_bonds(&mut output, molecule.bonds())?;
        output.push_str("</molecule>");
    }
    finish(output)
}

fn root_open() -> String {
    format!("<cml xmlns=\"{CML2_NAMESPACE}\">")
}

fn append_source_bonds(output: &mut String, record: &CmlDecodedRecordV1) {
    if record.bonds().is_empty() {
        return;
    }
    output.push_str("<bondArray>");
    for bond in record.bonds() {
        let start = record.atoms()[bond.start()].source_id();
        let end = record.atoms()[bond.end()].source_id();
        output.push_str("<bond");
        attribute(output, "atomRefs2", &format!("{start} {end}"));
        attribute(output, "order", bond_order_text(bond.order()));
        match bond.direction() {
            None => output.push_str("/>"),
            Some(BondDirection::BeginWedge) => output.push_str("><stereo>W</stereo></bond>"),
            Some(BondDirection::BeginDash) => output.push_str("><stereo>H</stereo></bond>"),
            Some(_) => unreachable!("CML source bonds retain only wedge or hash directions"),
        }
    }
    output.push_str("</bondArray>");
}

fn append_generic_bonds(output: &mut String, bonds: &[MolBond]) -> Result<()> {
    if bonds.is_empty() {
        return Ok(());
    }
    output.push_str("<bondArray>");
    for bond in bonds {
        if bond.is_aromatic()
            || !matches!(bond.stereo(), BondStereo::None)
            || !matches!(bond.direction(), BondDirection::None)
            || bond.stereo_atoms().is_some()
        {
            return refused(CmlEncoderRefusalReasonV1::BondChemistryUnsupported);
        }
        let Some(order) = cml_bond_order_text(bond.order()) else {
            return refused(CmlEncoderRefusalReasonV1::BondChemistryUnsupported);
        };
        output.push_str("<bond");
        attribute(
            output,
            "atomRefs2",
            &format!("a{} a{}", bond.start() + 1, bond.end() + 1),
        );
        attribute(output, "order", order);
        output.push_str("/>");
    }
    output.push_str("</bondArray>");
    Ok(())
}

fn validate_atom(atom: &MolAtom) -> Result<()> {
    if atom.is_aromatic()
        || atom.explicit_hydrogens().is_some()
        || !matches!(atom.chirality(), AtomChirality::Unspecified)
        || atom.radical_electrons() != 0
        || atom.no_implicit()
        || atom.atom_map_number().is_some()
        || atom
            .formal_charge()
            .is_some_and(|charge| !(-8..=8).contains(&charge))
        || atom
            .isotope()
            .is_some_and(|isotope| !(1..=400).contains(&isotope))
    {
        return refused(CmlEncoderRefusalReasonV1::AtomChemistryUnsupported);
    }
    Ok(())
}

fn finish(mut output: String) -> Result<String> {
    output.push_str("</cml>");
    if output.len() > MAX_OUTPUT_BYTES {
        return refused(CmlEncoderRefusalReasonV1::OutputBytesLimit);
    }
    decode_cml_bytes_v1(output.as_bytes()).map_err(|_| CmlEncoderErrorV1 {
        reason: CmlEncoderRefusalReasonV1::GeneratedDocumentRejected,
    })?;
    Ok(output)
}

fn attribute(output: &mut String, name: &str, value: &str) {
    output.push(' ');
    output.push_str(name);
    output.push_str("=\"");
    output.push_str(value);
    output.push('\"');
}

fn decimal_attribute(output: &mut String, name: &str, value: f64) {
    attribute(output, name, &format!("{value:.17}"));
}

fn bond_order_text(order: BondOrder) -> &'static str {
    match order {
        BondOrder::Single => "1",
        BondOrder::Double => "2",
        BondOrder::Triple => "3",
        BondOrder::Aromatic | BondOrder::Quadruple => {
            unreachable!("decoder only admits single, double, and triple bonds")
        }
    }
}

fn cml_bond_order_text(order: BondOrder) -> Option<&'static str> {
    match order {
        BondOrder::Single => Some("1"),
        BondOrder::Double => Some("2"),
        BondOrder::Triple => Some("3"),
        BondOrder::Aromatic | BondOrder::Quadruple => None,
    }
}
